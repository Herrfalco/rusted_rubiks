use super::*;

pub struct Solver<'a> {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    bookmark: Vec<usize>,
    cube: &'a mut Cube,
}

impl<'a> Solver<'a> {
    const EDGE_IDS: [[Id; 4]; 3] = [[1, 5, 7, 3], [9, 11, 17, 15], [19, 23, 25, 21]];

    pub fn new(cube: &mut Cube) -> Solver {
        Solver {
            mov_stack: Vec::with_capacity(128),
            bookmark: Vec::with_capacity(32),
            cube,
        }
    }

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

    fn test_up_edges(&mut self, layer: usize) {}

    fn scan_edges(&mut self) {
        println!("{}", self.cube);
        self.edge_2_up(1);
        self.undo_comb();
        self.edge_2_up(2);
        self.undo_comb();
        println!("{}", self.cube);
    }

    pub fn solve(&mut self) {
        self.scan_edges();
    }
}
