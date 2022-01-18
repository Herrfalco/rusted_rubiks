use super::*;
use compressor::*;
use crossbeam::thread;
use std::collections::HashMap;

trait HashMove {
    fn ins_min(&mut self, key: u64, comb: Vec<u8>);
    fn get_moves(&self, key: u64) -> Box<dyn Iterator<Item = (Face, Rotation, RotType)> + '_>;
    fn save(&self, file: &str, key_sz: usize);
    fn load(&mut self, file: &str, key_sz: usize);
    fn disp(&self, key: u64, title: &str);
    fn exec(&self, key: u64, cube: &mut Cube);
    fn u8_2_mov(mov: u8) -> (Face, Rotation, RotType) {
        (
            Face::FACE_SET[(mov >> 2) as usize],
            Rotation::ROT_SET[((mov >> 1) & 0b1) as usize],
            RotType::TYPE_SET[(mov & 0b1) as usize],
        )
    }
}

impl HashMove for HashMap<u64, Vec<u8>> {
    fn ins_min(&mut self, key: u64, comb: Vec<u8>) {
        match self.get_mut(&key) {
            Some(val) => {
                if val.len() > comb.len() {
                    *val = comb;
                }
            }
            None => {
                self.insert(key, comb);
            }
        }
    }

    fn get_moves(&self, key: u64) -> Box<dyn Iterator<Item = (Face, Rotation, RotType)> + '_> {
        Box::new(self[&key].iter().map(|mv| Self::u8_2_mov(*mv)))
    }

    fn save(&self, file: &str, key_sz: usize) {
        let mut compressor = Compressor::new();

        for (k, v) in self {
            compressor.push(*k, key_sz);
            compressor.push(v.len() as u8, 4);
            for m in v {
                compressor.push(*m, 5);
            }
        }
        compressor.save(file);
    }

    fn load(&mut self, file: &str, key_sz: usize) {
        let mut decompressor = Decompressor::new(file);

        while let Some(key) = decompressor.pop(key_sz) {
            self.insert(
                key,
                (0..decompressor.pop(4).unwrap())
                    .map(|_| decompressor.pop(5).unwrap())
                    .collect::<Vec<u8>>(),
            );
        }
    }

    fn disp(&self, key: u64, title: &str) {
        print!("{}", format!("{}: ", title).bright_green());

        for (face, rot, typ) in self.get_moves(key) {
            disp_mov(face, rot, typ);
        }
    }

    fn exec(&self, key: u64, cube: &mut Cube) {
        for (face, rot, typ) in self.get_moves(key) {
            cube.rotate(face, rot, typ);
        }
    }
}

struct TableInfos {
    key_gen: fn(&Solver) -> u64,
    key_sz: usize,
    set_sz: usize,
    rank: usize,
    cap: usize,
}

pub struct Solver {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    cube: Cube,
}

#[allow(dead_code)]
impl<'de> Solver {
    const TAB_INF: [TableInfos; 4] = [
        TableInfos {
            key_gen: Self::key_gen_1,
            key_sz: 12,
            set_sz: 18,
            rank: 7,
            cap: 2_048,
        },
        TableInfos {
            key_gen: Self::key_gen_2,
            key_sz: 36,
            set_sz: 14,
            rank: 10,
            cap: 1_082_565,
        },
        TableInfos {
            key_gen: Self::key_gen_3,
            key_sz: 16,
            set_sz: 10,
            rank: 13,
            cap: 29_400,
        },
        TableInfos {
            key_gen: Self::key_gen_4,
            key_sz: 40,
            set_sz: 6,
            rank: 15,
            cap: 663_552,
        },
    ];

    pub fn new(cube: Cube) -> Solver {
        Solver {
            mov_stack: Vec::with_capacity(128),
            cube,
        }
    }

    fn do_mov(&mut self, face: Face, rot: Rotation, typ: RotType) {
        self.mov_stack.push((face, rot, typ));
        self.cube.rotate(face, rot, typ);
    }

    fn undo_mov(&mut self) -> (Face, Rotation, RotType) {
        let (face, rot, typ) = self.mov_stack.pop().unwrap();

        self.cube
            .rotate(face, if let Cw = rot { Ccw } else { Cw }, typ);
        (face, rot, typ)
    }

    //12bit key
    fn key_gen_1(&self) -> u64 {
        let mut result: u64 = 0;

        for (face_i, face) in Cube::FACE_CHAINS[2].iter().enumerate() {
            for idx in match face {
                Front | Left => [1, 3, 7],
                _ => [1, 5, 7],
            } {
                if let Edge(dir, col) =
                    &self.cube.subs[self.cube.ids[Cube::FACE_MAP[*face as usize][idx]]]
                {
                    let (face_j, col_i) = Cube::FACE_CHAINS[2]
                        .iter()
                        .enumerate()
                        .find_map(|(face_j, f)| {
                            match col.iter().position(|c| MyColor::COL_SET[*f as usize] == *c) {
                                Some(col_i) => Some((face_j, col_i)),
                                None => None,
                            }
                        })
                        .unwrap();

                    result = (result << 1)
                        | (((face_i + 4 - face_j) % 2) ^ if dir[col_i] == *face { 0 } else { 1 })
                            as u64;
                } else {
                    panic!("Not an edge");
                }
            }
        }
        result
    }

    //36bit key
    fn key_gen_2(&self) -> u64 {
        let mut result = 0;
        let mut id;

        for pos in 0..27 {
            id = self.cube.ids[pos];
            result = match self.cube.subs[id] {
                Corner(dirs, _) => (result << 3) | dirs[0] as u64,
                Edge(_, _) => {
                    (result << 1)
                        | ((Cube::FACE_MAP[Left as usize].contains(&id)
                            || Cube::FACE_MAP[Right as usize].contains(&id))
                            as u64)
                }
                _ => continue,
            }
        }
        result
    }

    //16bit key
    fn key_gen_3(&self) -> u64 {
        let mut result = 0;

        for face in &Cube::FACE_MAP[4..] {
            for pos in face {
                let (dir, col) = match self.cube.subs[self.cube.ids[*pos]] {
                    Edge(dirs, cols) => (dirs[1], cols[1]),
                    Corner(dirs, cols) => (dirs[1], cols[1]),
                    _ => continue,
                };
                result = (result << 1)
                    | (col != MyColor::COL_SET[dir as usize]
                        && col
                            != MyColor::COL_SET[if dir as usize % 2 == 0 {
                                dir as usize + 1
                            } else {
                                dir as usize - 1
                            }]) as u64;
            }
        }
        result
    }

    //40bit key
    fn key_gen_4(&self) -> u64 {
        let mut result = 0;

        for sub in &self.cube.subs {
            let (dir_1, dir_2, col_1, col_2) = match sub {
                Corner(dirs, cols) => (dirs[0], dirs[1], cols[0], cols[1]),
                Edge(dirs, cols) => (dirs[0], dirs[1], cols[0], cols[1]),
                _ => continue,
            };
            result = (result << 2)
                | (((col_1 != MyColor::COL_SET[dir_1 as usize]) as u64) << 1)
                | ((col_2 != MyColor::COL_SET[dir_2 as usize]) as u64)
        }
        result
    }

    fn mov_2_u8(face: Face, rot: Rotation, typ: RotType) -> u8 {
        ((face as u8) << 2) | ((rot as u8) << 1) | (typ as u8)
    }

    fn comb_2_rev_u8(comb: &Vec<(Face, Rotation, RotType)>) -> Vec<u8> {
        comb.iter()
            .rev()
            .map(|(face, rot, typ)| {
                Self::mov_2_u8(*face, if let Cw = *rot { Ccw } else { Cw }, *typ)
            })
            .collect()
    }

    fn rec_search(
        &mut self,
        sol: &mut HashMap<u64, Vec<u8>>,
        key_gen: fn(&Self) -> u64,
        set_sz: usize,
        rank: usize,
    ) {
        sol.ins_min(key_gen(self), Self::comb_2_rev_u8(&self.mov_stack));
        if rank > 0 {
            for (face, rot, typ) in Cube::MOV_SET[..set_sz].iter() {
                if self.mov_stack.last().unwrap().0 != *face {
                    self.do_mov(*face, *rot, *typ);
                    self.rec_search(sol, key_gen, set_sz, rank - 1);
                    self.undo_mov();
                }
            }
        }
    }

    fn mt_search(tab: usize) {
        let file = format!("mt_table_{}", tab);

        println!("Table {} computation started...", tab);
        thread::scope(|s| {
            let tab_inf = &Self::TAB_INF[tab - 1];
            let mut thrds = Vec::with_capacity(tab_inf.set_sz);
            let mut result: HashMap<u64, Vec<u8>> = HashMap::with_capacity(tab_inf.cap);

            for (face, rot, typ) in &Cube::MOV_SET[..tab_inf.set_sz] {
                thrds.push(s.spawn(|_| {
                    let mut solver = Solver::new(Cube::new());
                    let mut table: HashMap<u64, Vec<u8>> = HashMap::with_capacity(tab_inf.cap);

                    table.ins_min((tab_inf.key_gen)(&mut solver), vec![]);
                    solver.do_mov(*face, *rot, *typ);
                    solver.rec_search(
                        &mut table,
                        tab_inf.key_gen,
                        tab_inf.set_sz,
                        tab_inf.rank - 1,
                    );
                    table
                }));
            }

            for thrd in thrds {
                for (key, val) in thrd.join().unwrap() {
                    result.ins_min(key, val);
                }
            }
            result.save(&file, tab_inf.key_sz);
        })
        .unwrap();
        println!("Extracted to file {}", file);
    }

    pub fn table_search(table_ids: Vec<usize>) {
        for id in table_ids {
            match id {
                1 => Self::mt_search(1),
                2 => Self::mt_search(2),
                3 => Self::mt_search(3),
                4 => Self::mt_search(4),
                _ => panic!("unknown table"),
            };
        }
    }

    fn solve_step(&mut self, step: usize) {
        let tab_inf = &Self::TAB_INF[step - 1];
        let mut table: HashMap<u64, Vec<u8>> = HashMap::with_capacity(tab_inf.cap);

        table.load(&format!("mt_table_{}", step), tab_inf.key_sz);
        println!("table {} size: {}", step, table.len());
        let key = (tab_inf.key_gen)(&self);
        table.disp(key, &format!("PHASE {}", step));
        table.exec(key, &mut self.cube);
        println!("\n\n{}", self.cube);
    }

    pub fn solve(&mut self) {
        for step in 1..3 {
            self.solve_step(step);
        }
    }
}
