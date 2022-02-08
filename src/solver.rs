use super::*;
use compressor::*;
use crossbeam::thread;
use std::collections::HashMap;

//split table generator and solver

trait HashMove {
    fn ins_min(&mut self, key: u64, movs: Vec<u8>);
    fn save(&self, file: &str, key_sz: usize);
    fn load(&mut self, file: &str, key_sz: usize);
    fn exec(&self, key: u64, cube: &mut Cube, phase: Option<usize>);
    fn u8_2_mov(mov: u8) -> Move {
        Move(
            Face::FACE_SET[(mov >> 2) as usize],
            Rotation::ROT_SET[((mov >> 1) & 0b1) as usize],
            RotType::TYPE_SET[(mov & 0b1) as usize],
        )
    }
}

impl HashMove for HashMap<u64, Vec<u8>> {
    fn ins_min(&mut self, key: u64, movs: Vec<u8>) {
        match self.get_mut(&key) {
            Some(val) => {
                if val.len() > movs.len() {
                    *val = movs;
                }
            }
            None => {
                self.insert(key, movs);
            }
        }
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

    fn exec(&self, key: u64, cube: &mut Cube, phase: Option<usize>) {
        let mut disp_res = String::new();

        for mv in self[&key].iter().map(|mv| Self::u8_2_mov(*mv)) {
            std::fmt::write(&mut disp_res, format_args!("{} ", mv)).unwrap();
            cube.rotate(mv, true);
        }
        if let Some(nb) = phase {
            print!("{}{}", format!("PHASE {}: ", nb).bright_green(), disp_res);
        }
    }
}

struct TableInfos {
    id: Id,
    key_gen: fn(&Solver) -> u64,
    key_sz: usize,
    set_sz: usize,
    rank: usize,
    cap: usize,
}

pub struct Solver {
    mov_stack: Vec<Move>,
    cube: Cube,
}

#[allow(dead_code)]
impl<'de> Solver {
    const TAB_INF: [TableInfos; 4] = [
        TableInfos {
            id: 1,
            key_gen: Self::key_gen_1,
            key_sz: 12,
            set_sz: 18,
            rank: 7,
            cap: 2_048,
        },
        TableInfos {
            id: 2,
            key_gen: Self::key_gen_2,
            key_sz: 36,
            set_sz: 14,
            rank: 10,
            cap: 1_082_565,
        },
        TableInfos {
            id: 3,
            key_gen: Self::key_gen_3,
            key_sz: 28,
            set_sz: 10,
            rank: 13,
            cap: 29_400,
        },
        TableInfos {
            id: 4,
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

    fn g3_seeds() -> [Vec<Move>; 8] {
        [
            vec![],
            vec![Move(Up, Cw, Dual), Move(Left, Cw, Dual)],
            vec![Move(Down, Cw, Dual), Move(Front, Cw, Dual)],
            vec![Move(Front, Cw, Dual), Move(Up, Cw, Dual)],
            vec![
                Move(Up, Cw, Dual),
                Move(Left, Cw, Dual),
                Move(Front, Cw, Dual),
            ],
            vec![
                Move(Right, Cw, Dual),
                Move(Down, Cw, Dual),
                Move(Right, Cw, Dual),
            ],
            vec![
                Move(Right, Cw, Dual),
                Move(Back, Cw, Dual),
                Move(Up, Cw, Dual),
            ],
            vec![
                Move(Up, Cw, Dual),
                Move(Front, Cw, Dual),
                Move(Right, Cw, Dual),
                Move(Down, Cw, Dual),
            ],
        ]
    }

    fn do_mov(&mut self, mv: Move) {
        self.mov_stack.push(mv);
        self.cube.rotate(mv, false);
    }

    fn undo_mov(&mut self) {
        let Move(face, rot, typ) = self.mov_stack.pop().unwrap();

        self.cube
            .rotate(Move(face, if let Cw = rot { Ccw } else { Cw }, typ), false);
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

    //28bit key
    fn key_gen_3(&self) -> u64 {
        let mut result = 0;

        for face in &Cube::FACE_MAP[4..] {
            for pos in face {
                let (dir, col) = match self.cube.subs[self.cube.ids[*pos]] {
                    Edge(dirs, cols) => (dirs[1], cols[1]),
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
        for (pair_idx, face_pair) in Cube::FACE_MAP.chunks(2).enumerate() {
            for (pos_1, pos_2) in face_pair[0].iter().zip(face_pair[1].iter()) {
                if let Corner(dirs_1, cols_1) = self.cube.subs[self.cube.ids[*pos_1]] {
                    if let Corner(dirs_2, cols_2) = self.cube.subs[self.cube.ids[*pos_2]] {
                        result = (result << 1)
                            | (cols_1[dirs_1
                                .iter()
                                .position(|x| *x == Face::FACE_SET[pair_idx * 2])
                                .unwrap()]
                                == cols_2[dirs_2
                                    .iter()
                                    .position(|x| *x == Face::FACE_SET[pair_idx * 2 + 1])
                                    .unwrap()]) as u64;
                    }
                }
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

    fn movs_2_rev_u8(movs: &Vec<Move>) -> Vec<u8> {
        movs.iter()
            .rev()
            .map(|Move(face, rot, typ)| {
                ((*face as u8) << 2)
                    | (((if *rot == Cw { Ccw } else { Cw }) as u8) << 1)
                    | (*typ as u8)
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
        sol.ins_min(key_gen(self), Self::movs_2_rev_u8(&self.mov_stack));
        if rank > 0 {
            for mv in Cube::MOV_SET[..set_sz].iter() {
                if self.mov_stack.last().unwrap().0 != mv.0 {
                    self.do_mov(*mv);
                    self.rec_search(sol, key_gen, set_sz, rank - 1);
                    self.undo_mov();
                }
            }
        }
    }

    fn mt_search(inf: &TableInfos, cub: Cube) -> HashMap<u64, Vec<u8>> {
        println!("Computation unit started...");
        let map = thread::scope(|s| {
            let mut thrds = Vec::with_capacity(inf.set_sz);
            let mut result: HashMap<u64, Vec<u8>> = HashMap::with_capacity(inf.cap);

            for mv in &Cube::MOV_SET[..inf.set_sz] {
                thrds.push(s.spawn(|_| {
                    let mut solver = Solver::new(cub.clone());
                    let mut table: HashMap<u64, Vec<u8>> = HashMap::with_capacity(inf.cap);

                    table.ins_min((inf.key_gen)(&mut solver), vec![]);
                    solver.do_mov(*mv);
                    solver.rec_search(&mut table, inf.key_gen, inf.set_sz, inf.rank - 1);
                    table
                }));
            }

            for thrd in thrds {
                for (key, val) in thrd.join().unwrap() {
                    result.ins_min(key, val);
                }
            }
            result
        })
        .unwrap();
        println!("Computation unit ended...");
        map
    }

    pub fn table_search(table_ids: Vec<usize>) {
        let mut inf;
        let mut file;

        for id in table_ids {
            inf = &Solver::TAB_INF[id - 1];
            file = format!("tabs/mt_table_{}", inf.id);
            println!("Table {} extraction:", inf.id);
            match id {
                3 => {
                    let map = thread::scope(|s| {
                        let seeds = Solver::g3_seeds();
                        let mut thrds = Vec::with_capacity(seeds.len());
                        let mut result: HashMap<u64, Vec<u8>> = HashMap::with_capacity(inf.cap);

                        for seed in seeds {
                            let mut cub = Cube::new();

                            for mv in seed {
                                cub.rotate(mv, false);
                            }
                            thrds.push(s.spawn(|_| Self::mt_search(inf, cub)));
                        }

                        for thrd in thrds {
                            for (key, val) in thrd.join().unwrap() {
                                result.ins_min(key, val);
                            }
                        }
                        result
                    })
                    .unwrap();
                    map
                }
                _ => Self::mt_search(inf, Cube::new()),
            }
            .save(&file, inf.key_sz);
            println!("Extracted to file {}", file);
        }
    }

    fn solve_step(&mut self, step: usize) {
        let tab_inf = &Self::TAB_INF[step - 1];
        let mut table: HashMap<u64, Vec<u8>> = HashMap::with_capacity(tab_inf.cap);

        /*
        println!(
            "before: {:0ks$b}",
            (tab_inf.key_gen)(&self),
            ks = tab_inf.key_sz
        );
        */
        table.load(&format!("tabs/mt_table_{}", step), tab_inf.key_sz);
        println!("table {} size: {}", step, table.len());
        let key = (tab_inf.key_gen)(&self);
        table.exec(key, &mut self.cube, Some(step));
        /*
        println!(
            "before: {:0ks$b}",
            (tab_inf.key_gen)(&self),
            ks = tab_inf.key_sz
        );
        */
        println!("\n\n{}", self.cube);
    }

    pub fn solve(&mut self) {
        for step in 1..5 {
            self.solve_step(step);
        }
    }
}
