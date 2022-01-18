use super::*;

#[derive(Clone)]
pub enum SubCube {
    Core,
    Center(Face, MyColor),
    Edge([Face; 2], [MyColor; 2]),
    Corner([Face; 3], [MyColor; 3]),
}

#[derive(Clone)]
pub struct Cube {
    pub ids: Vec<Id>,
    pub subs: [SubCube; 27],
}

impl std::fmt::Display for Cube {
    fn fmt(&self, fm: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = String::with_capacity(0x100);

        for line in Self::FACE_MAP[Up as usize].chunks(3) {
            result.push_str(&format!("         {}\n", self.row_2_str(line, Up, false)));
        }
        result.push_str("\n");
        for (l, (f, (r, b))) in Self::FACE_MAP[Left as usize].chunks(3).zip(
            Self::FACE_MAP[Front as usize].chunks(3).zip(
                Self::FACE_MAP[Right as usize]
                    .chunks(3)
                    .zip(Self::FACE_MAP[Back as usize].chunks(3)),
            ),
        ) {
            result.push_str(&format!(
                " {}  {}  {}  {}\n",
                self.row_2_str(l, Left, false),
                self.row_2_str(f, Front, false),
                self.row_2_str(r, Right, true),
                self.row_2_str(b, Back, true),
            ));
        }
        result.push_str("\n");
        for line in Self::FACE_MAP[Down as usize].chunks(3).rev() {
            result.push_str(&format!("         {}\n", self.row_2_str(line, Down, false),));
        }
        result.fmt(fm)
    }
}

impl Cube {
    pub const FACE_CHAINS: [[Face; 4]; 3] = [
        [Up, Right, Down, Left],
        [Up, Front, Down, Back],
        [Back, Right, Front, Left],
    ];

    pub const FACE_MAP: [[Id; 9]; 6] = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8],
        [18, 19, 20, 21, 22, 23, 24, 25, 26],
        [6, 7, 8, 15, 16, 17, 24, 25, 26],
        [0, 1, 2, 9, 10, 11, 18, 19, 20],
        [0, 3, 6, 9, 12, 15, 18, 21, 24],
        [2, 5, 8, 11, 14, 17, 20, 23, 26],
    ];

    pub const MOV_SET: [(Face, Rotation, RotType); 18] = [
        (Left, Ccw, Dual),
        (Right, Ccw, Dual),
        (Front, Ccw, Dual),
        (Back, Ccw, Dual),
        (Up, Ccw, Dual),
        (Down, Ccw, Dual),
        (Left, Ccw, Single),
        (Right, Ccw, Single),
        (Left, Cw, Single),
        (Right, Cw, Single),
        (Front, Ccw, Single),
        (Back, Ccw, Single),
        (Front, Cw, Single),
        (Back, Cw, Single),
        (Up, Ccw, Single),
        (Down, Ccw, Single),
        (Up, Cw, Single),
        (Down, Cw, Single),
    ];

    pub fn new() -> Self {
        Self {
            ids: (0..27).collect(),
            subs: [
                Corner([Left, Back, Up], [Orange, Yellow, Blue]),
                Edge([Back, Up], [Yellow, Blue]),
                Corner([Right, Up, Back], [Red, Blue, Yellow]),
                Edge([Left, Up], [Orange, Blue]),
                Center(Up, Blue),
                Edge([Right, Up], [Red, Blue]),
                Corner([Left, Up, Front], [Orange, Blue, White]),
                Edge([Front, Up], [White, Blue]),
                Corner([Right, Front, Up], [Red, White, Blue]),
                Edge([Left, Back], [Orange, Yellow]),
                Center(Back, Yellow),
                Edge([Right, Back], [Red, Yellow]),
                Center(Left, Orange),
                Core,
                Center(Right, Red),
                Edge([Left, Front], [Orange, White]),
                Center(Front, White),
                Edge([Right, Front], [Red, White]),
                Corner([Left, Down, Back], [Orange, Green, Yellow]),
                Edge([Back, Down], [Yellow, Green]),
                Corner([Right, Back, Down], [Red, Yellow, Green]),
                Edge([Left, Down], [Orange, Green]),
                Center(Down, Green),
                Edge([Right, Down], [Red, Green]),
                Corner([Left, Front, Down], [Orange, White, Green]),
                Edge([Front, Down], [White, Green]),
                Corner([Right, Down, Front], [Red, Green, White]),
            ],
        }
    }

    fn sub_2_str(&self, id: Id, face: Face) -> String {
        match self.subs[id] {
            Center(_, col) => col,
            Edge(dir, col) => col[dir.iter().position(|d| *d == face).unwrap()],
            Corner(dir, col) => col[dir.iter().position(|d| *d == face).unwrap()],
            _ => Void,
        }
        .to_string()
    }

    fn row_2_str(&self, pos: &[usize], face: Face, rev: bool) -> String {
        let i = if rev { [2, 1, 0] } else { [0, 1, 2] };

        format!(
            "{}{}{}",
            self.sub_2_str(self.ids[pos[i[0]]], face),
            self.sub_2_str(self.ids[pos[i[1]]], face),
            self.sub_2_str(self.ids[pos[i[2]]], face),
        )
    }

    fn rotate_dir(dir: &mut Face, face: Face, chain: &[Face], step: isize) {
        if *dir != face {
            *dir = chain[((chain.iter().position(|x| x == dir).unwrap() + chain.len()) as isize
                + step) as usize
                % 4];
        }
    }

    fn rotate_sub(&mut self, id: Id, face: Face, step: isize) {
        let chain = &Self::FACE_CHAINS[match face {
            Front | Back => 0,
            Left | Right => 1,
            Up | Down => 2,
        }];

        match &mut self.subs[id] {
            Edge(dir, _) => {
                for d in dir {
                    Self::rotate_dir(d, face, chain, step);
                }
            }
            Corner(dir, _) => {
                for d in dir {
                    Self::rotate_dir(d, face, chain, step);
                }
            }
            _ => (),
        }
    }

    pub fn rotate(&mut self, face: Face, rot: Rotation, typ: RotType) {
        let rev = match (face, rot) {
            (Front, Cw) | (Back, Ccw) | (Up, Cw) | (Down, Ccw) | (Left, Cw) | (Right, Ccw) => true,
            _ => false,
        };

        let win_size = if let Dual = typ { 3 } else { 2 };
        for chain in [[0_usize, 2, 8, 6], [1, 5, 7, 3]] {
            for swap in if rev {
                Box::new(chain.windows(win_size).rev()) as Box<dyn Iterator<Item = &[usize]>>
            } else {
                Box::new(chain.windows(win_size)) as Box<dyn Iterator<Item = &[usize]>>
            } {
                self.ids.swap(
                    Self::FACE_MAP[face as usize][swap[0]],
                    Self::FACE_MAP[face as usize][swap[swap.len() - 1]],
                );
            }
        }

        for pos in Self::FACE_MAP[face as usize] {
            self.rotate_sub(
                self.ids[pos],
                face,
                if rev { 1 } else { -1 } * if let Dual = typ { 2 } else { 1 },
            );
        }
    }
}
