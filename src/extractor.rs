use super::*;
use crossbeam::thread;
use std::collections::HashMap;

pub struct Extractor {
    cube: Cube,
    mov_stack: Vec<Move>,
}

impl Extractor {
    pub const TAB_INF: [TableInfos; 4] = [
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
            key_sz: 16,
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

    const TETRAD: [Id; 4] = [0, 8, 20, 24];

    pub fn new(cube: Cube) -> Self {
        Self {
            cube,
            mov_stack: Vec::with_capacity(128),
        }
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
    fn key_gen_1(cub: &Cube) -> u64 {
        let mut result: u64 = 0;

        for (face_i, face) in Cube::FACE_CHAINS[2].iter().enumerate() {
            for idx in match face {
                Front | Left => [1, 3, 7],
                _ => [1, 5, 7],
            } {
                if let Edge(dir, col) = cub.subs[cub.ids[Cube::FACE_MAP[*face as usize][idx]]] {
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
    fn key_gen_2(cub: &Cube) -> u64 {
        let mut result = 0;
        let mut id;

        for pos in 0..27 {
            id = cub.ids[pos];
            result = match cub.subs[id] {
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
    fn key_gen_3(cub: &Cube) -> u64 {
        let mut result = 0;

        for face in &Cube::FACE_MAP[4..] {
            for pos in face {
                result = match cub.subs[cub.ids[*pos]] {
                    Edge(dirs, cols) => {
                        (result << 1)
                            | (cols[1] != MyColor::COL_SET[dirs[1] as usize]
                                && cols[1]
                                    != MyColor::COL_SET[if dirs[1] as usize % 2 == 0 {
                                        dirs[1] as usize + 1
                                    } else {
                                        dirs[1] as usize - 1
                                    }]) as u64
                    }
                    Corner(..) => {
                        (result << 1)
                            | (Self::TETRAD.contains(pos) ^ Self::TETRAD.contains(&cub.ids[*pos]))
                                as u64
                    }
                    _ => continue,
                };
            }
        }
        result
    }

    //40bit key
    fn key_gen_4(cub: &Cube) -> u64 {
        let mut result = 0;

        for sub in &cub.subs {
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
        key_gen: fn(&Cube) -> u64,
        set_sz: usize,
        rank: usize,
    ) {
        sol.ins_min(key_gen(&self.cube), Self::movs_2_rev_u8(&self.mov_stack));
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
        let map = thread::scope(|s| {
            let mut thrds = Vec::with_capacity(inf.set_sz);
            let mut result: HashMap<u64, Vec<u8>> = HashMap::with_capacity(inf.cap);

            for mv in &Cube::MOV_SET[..inf.set_sz] {
                thrds.push(s.spawn(|_| {
                    let mut extractor = Extractor::new(cub.clone());
                    let mut table: HashMap<u64, Vec<u8>> = HashMap::with_capacity(inf.cap);

                    table.ins_min((inf.key_gen)(&cub), vec![]);
                    extractor.do_mov(*mv);
                    extractor.rec_search(&mut table, inf.key_gen, inf.set_sz, inf.rank - 1);
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
        map
    }

    pub fn table_search(table_ids: Vec<usize>) {
        let mut inf;
        let mut file;

        for id in table_ids {
            inf = &Extractor::TAB_INF[id - 1];
            file = format!("tabs/mt_table_{}", inf.id);
            println!("Table {} extraction:", inf.id);
            Self::mt_search(inf, Cube::new()).save(&file, inf.key_sz);
            println!("Extracted to file {}", file);
        }
    }
}
