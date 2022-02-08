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
    pub movs: Vec<Move>,
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
        write!(fm, "{}", result)
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

    pub const MOV_SET: [Move; 18] = [
        Move(Left, Ccw, Dual),
        Move(Right, Ccw, Dual),
        Move(Front, Ccw, Dual),
        Move(Back, Ccw, Dual),
        Move(Up, Ccw, Dual),
        Move(Down, Ccw, Dual),
        Move(Left, Ccw, Single),
        Move(Right, Ccw, Single),
        Move(Left, Cw, Single),
        Move(Right, Cw, Single),
        Move(Front, Ccw, Single),
        Move(Back, Ccw, Single),
        Move(Front, Cw, Single),
        Move(Back, Cw, Single),
        Move(Up, Ccw, Single),
        Move(Down, Ccw, Single),
        Move(Up, Cw, Single),
        Move(Down, Cw, Single),
    ];

    pub fn new() -> Self {
        Self {
            ids: (0..27).collect(),
            subs: [
                Corner([Left, Back, Up], [Orange, Yellow, Blue]),
                Edge([Back, Up], [Yellow, Blue]),
                Corner([Right, Back, Up], [Red, Yellow, Blue]),
                Edge([Left, Up], [Orange, Blue]),
                Center(Up, Blue),
                Edge([Right, Up], [Red, Blue]),
                Corner([Left, Front, Up], [Orange, White, Blue]),
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
                Corner([Left, Back, Down], [Orange, Yellow, Green]),
                Edge([Back, Down], [Yellow, Green]),
                Corner([Right, Back, Down], [Red, Yellow, Green]),
                Edge([Left, Down], [Orange, Green]),
                Center(Down, Green),
                Edge([Right, Down], [Red, Green]),
                Corner([Left, Front, Down], [Orange, White, Green]),
                Edge([Front, Down], [White, Green]),
                Corner([Right, Front, Down], [Red, White, Green]),
            ],
            movs: Vec::new(),
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

    pub fn rotate(&mut self, mov: Move, mem: bool) {
        let Move(face, rot, typ) = mov;
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
        if mem {
            self.movs.push(mov);
        }
    }

    pub fn mov_parser(mov: &str) -> Result<Move, String> {
        match mov {
            "U" => Ok(Move(Up, Cw, Single)),
            "U2" => Ok(Move(Up, Cw, Dual)),
            "U'" => Ok(Move(Up, Ccw, Single)),
            "D" => Ok(Move(Down, Cw, Single)),
            "D2" => Ok(Move(Down, Cw, Dual)),
            "D'" => Ok(Move(Down, Ccw, Single)),
            "F" => Ok(Move(Front, Cw, Single)),
            "F2" => Ok(Move(Front, Cw, Dual)),
            "F'" => Ok(Move(Front, Ccw, Single)),
            "B" => Ok(Move(Back, Cw, Single)),
            "B2" => Ok(Move(Back, Cw, Dual)),
            "B'" => Ok(Move(Back, Ccw, Single)),
            "L" => Ok(Move(Left, Cw, Single)),
            "L2" => Ok(Move(Left, Cw, Dual)),
            "L'" => Ok(Move(Left, Ccw, Single)),
            "R" => Ok(Move(Right, Cw, Single)),
            "R2" => Ok(Move(Right, Cw, Dual)),
            "R'" => Ok(Move(Right, Ccw, Single)),
            _ => Err(format!("Face \"{}\" is not recognized", mov)),
        }
    }

    pub fn from_str(s: &str, disp: bool) -> Self {
        let mut result = Cube::new();
        let mut disp_res = String::new();

        for mv in s.split_whitespace().map(|m| Self::mov_parser(m).unwrap()) {
            std::fmt::write(&mut disp_res, format_args!("{} ", mv)).unwrap();
            result.rotate(mv, true);
        }
        if disp {
            println!("{}{}", "MOVES: ".bright_green(), disp_res);
        }
        result
    }

    pub fn from_rand(mov_nb: usize, group: usize, disp: bool) -> Self {
        let mut result = Cube::new();
        let mut lst_mv: Option<Move> = None;
        let mut rng = rand::thread_rng();
        let mut disp_res = String::new();

        for _ in 0..mov_nb {
            loop {
                let mv = *Cube::MOV_SET[..Cube::MOV_SET.len() - group * 4]
                    .choose(&mut rng)
                    .unwrap();
                if let Some(Move(face, ..)) = lst_mv {
                    if face == mv.0 {
                        continue;
                    }
                }
                lst_mv = Some(mv);
                std::fmt::write(&mut disp_res, format_args!("{} ", mv)).unwrap();
                result.rotate(mv, true);
                break;
            }
        }
        if disp {
            println!("{}{}", "MOVES: ".bright_green(), disp_res);
        }
        result
    }
}
