use super::*;

pub struct Solver<'a> {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    cube: &'a mut Cube,
}

impl<'a> Solver<'a> {
    pub fn new(cube: &mut Cube) -> Solver {
        Solver {
            mov_stack: Vec::with_capacity(128),
            cube,
        }
    }

    fn do_mov(&mut self, face: Face, rot: Rotation, rot_type: RotType) {
        self.mov_stack.push((face, rot, rot_type));
        self.cube.rotate(face, rot, rot_type);
    }

    fn undo_all_rot(&mut self) {
        while let Some((face, rot, dual)) = self.mov_stack.pop() {
            self.cube
                .rotate(face, if let Cw = rot { Ccw } else { Cw }, dual);
        }
    }

    fn edge_2_up(&mut self, layer: usize) {
        match layer {
            1 => {
                self.do_mov(Back, Cw, Single);
                self.do_mov(Right, Cw, Single);
                self.do_mov(Front, Cw, Single);
                self.do_mov(Back, Ccw, Single);
                self.do_mov(Left, Cw, Single);
                self.do_mov(Back, Cw, Single);
            }
            2 => {
                for face in Cube::FACE_CHAINS[2] {
                    self.do_mov(face, Cw, Dual);
                    println!("{}", self.cube);
                }
            }
            _ => (),
        }
    }

    fn scan_edges(&mut self) {
        /*
        println!("---------------");
        self.edge_2_up(1);
        println!("{}", self.cube);
        self.undo_all_rot();
        println!("{}", self.cube);
        println!("---------------");
        self.edge_2_up(2);
        println!("{}", self.cube);
        self.undo_all_rot();
        */
        println!("{}", self.cube);
    }

    pub fn solve(&mut self) {
        self.scan_edges();
    }
}
