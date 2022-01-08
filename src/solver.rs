use super::*;
use std::fs;

pub struct Solver<'a> {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    //    bookmark: Vec<usize>,
    cube: &'a mut Cube,
}

impl<'a> Solver<'a> {
    const FACE_EDGES: [[usize; 3]; 2] = [[1, 3, 7], [1, 5, 7]];

    pub fn new(cube: &mut Cube) -> Solver {
        Solver {
            mov_stack: Vec::with_capacity(128),
            //           bookmark: Vec::with_capacity(32),
            cube,
        }
    }

    /*
        fn do_comb(&mut self, combs: &[(Face, Rotation, RotType)]) {
            self.bookmark.push(self.mov_stack.len());
            for comb in combs {
                self.mov_stack.push(*comb);
                self.cube.rotate(comb.0, comb.1, comb.2);
            }
        }

        fn undo_comb(&mut self) {
            let bookmark = self.bookmark.pop().unwrap();

            while self.mov_stack.len() > bookmark {
                if let Some((face, rot, dual)) = self.mov_stack.pop() {
                    self.cube
                        .rotate(face, if let Cw = rot { Ccw } else { Cw }, dual);
                }
            }
        }

    */

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

    fn scan_edges(&mut self) -> u16 {
        let mut result: u16 = 0;

        for (face_i, face) in Cube::FACE_CHAINS[2].iter().enumerate() {
            for idx in Self::FACE_EDGES[match face {
                Front | Left => 0,
                _ => 1,
            }] {
                if let Edge(dir, col) =
                    &mut self.cube.subs[self.cube.ids[Cube::FACE_MAP[*face as usize][idx]]]
                {
                    let (face_j, col_i) = Cube::FACE_CHAINS[2]
                        .iter()
                        .enumerate()
                        .find_map(|(face_j, f)| {
                            for (col_i, c) in col.iter().enumerate() {
                                if Cube::COLOR_MAP[*f as usize] == *c {
                                    return Some((face_j, col_i));
                                }
                            }
                            None
                        })
                        .unwrap();

                    result = (result << 1)
                        | (((face_i + 4 - face_j) % 2) ^ if dir[col_i] == *face { 0 } else { 1 })
                            as u16;
                } else {
                    panic!("Not an edge")
                }
            }
        }
        result
    }

    fn u8_2_mov(mov: u8) -> (Face, Rotation, RotType) {
        (
            Face::FACE_SET[(mov >> 4) as usize],
            Rotation::ROT_SET[((mov >> 1) & 0b1) as usize],
            RotType::TYPE_SET[(mov & 0b1) as usize],
        )
    }

    /*
    fn mov_2_u8(face: Face, rot: Rotation, typ: RotType) -> u8 {
        ((face as u8) << 4) | ((rot as u8) << 1) | (typ as u8)
    }

    fn stack_2_u8_sol(&self) -> Vec<u8> {
        self.mov_stack
            .iter()
            .rev()
            .map(|(face, rot, typ)| {
                Self::mov_2_u8(*face, if let Cw = *rot { Ccw } else { Cw }, *typ)
            })
            .collect()
    }

    fn save_result(res: &mut [Option<Vec<u8>>]) {
        fs::write(
            "data_1",
            res.iter_mut()
                .map(|x| match x.take() {
                    Some(mut movs) => {
                        movs.push(0xff);
                        movs
                    }
                    _ => vec![0xff],
                })
                .flatten()
                .collect::<Vec<u8>>(),
        )
        .unwrap();
    }
    */

    fn load_table(table: &str) -> Vec<Vec<(Face, Rotation, RotType)>> {
        fs::read(table)
            .unwrap()
            .split(|x| *x == 0xff)
            .map(|movs| {
                movs.iter()
                    .map(|mov| Solver::u8_2_mov(*mov))
                    .collect::<Vec<(Face, Rotation, RotType)>>()
            })
            .collect::<Vec<Vec<(Face, Rotation, RotType)>>>()
    }

    /*
    fn rec_search(&mut self, sol: &mut [Option<Vec<u8>>], rank: usize) {
        match &mut sol[self.scan_edges() as usize] {
            Some(movs) => {
                if movs.len() > self.mov_stack.len() {
                    *movs = self.stack_2_u8_sol();
                }
            }
            _ => sol[self.scan_edges() as usize] = Some(self.stack_2_u8_sol()),
        }

        if rank > 0 {
            for face in Face::FACE_SET {
                for rot in Rotation::ROT_SET.iter().rev() {
                    if let Ccw = rot {
                        for typ in RotType::TYPE_SET {
                            self.do_mov(face, *rot, typ);
                            self.rec_search(sol, rank - 1);
                            self.undo_mov();
                        }
                    } else {
                        self.do_mov(face, *rot, Single);
                        self.rec_search(sol, rank - 1);
                        self.undo_mov();
                    }
                }
            }
        }
    }
    */

    pub fn solve(&mut self) {
        /*
        let mut sol: Vec<Option<Vec<u8>>> = vec![None; 4096];

        self.rec_search(&mut sol, 7);
        Self::save_result(&mut sol);
        println!("saved");

        let load = fs::read("data_1").unwrap();
        println!("{}", load.iter().filter(|byte| **byte == 0xff).count());
        */

        let load = Self::load_table("data_1");

        print!("{}", "PHASE 1: ".bright_green());
        for (face, rot, typ) in &Self::load_table("data_1")[self.scan_edges() as usize] {
            //a factoriser
            print!(
                "{}{} ",
                face.to_string().bright_yellow(),
                if let Dual = typ {
                    "2".bright_red()
                } else {
                    rot.to_string().bright_red()
                }
            );
            self.cube.rotate(*face, *rot, *typ);
        }
        println!("\n\n{}", self.cube);
    }
}
