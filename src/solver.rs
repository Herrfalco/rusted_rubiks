use super::*;

pub struct Solver<'a> {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    //    bookmark: Vec<usize>,
    cube: &'a mut Cube,
}

impl<'a> Solver<'a> {
    //    const EDGE_IDS: [[Id; 4]; 3] = [[1, 5, 7, 3], [9, 11, 17, 15], [19, 23, 25, 21]];
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

    fn edge_2_up(&mut self, layer: usize) {
        match layer {
            1 => self.do_comb(&vec![
                (Back, Cw, Single),
                (Right, Cw, Single),
                (Front, Cw, Single),
                (Back, Ccw, Single),
                (Left, Cw, Single),
                (Back, Cw, Single),
            ]),
            2 => self.do_comb(
                &Cube::FACE_CHAINS[2]
                    .iter()
                    .map(|face| (*face, Cw, Dual))
                    .collect::<Vec<(Face, Rotation, RotType)>>(),
            ),
            _ => (),
        }
    }
    */

    /*
    fn sub_from_face_idx(&mut self, face: Face, idx: usize) -> &mut SubCube {
        &mut self.cube.subs[self.cube.ids[Cube::FACE_MAP[face as usize][idx]]]
    }
    */

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

    pub fn solve(&mut self) {
        println!("{:012b}", self.scan_edges());
    }
}
