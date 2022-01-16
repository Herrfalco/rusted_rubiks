#![allow(dead_code, unused_imports, unused_variables)]
mod compressor;
mod solver;

use clap::{App, Arg};
use colored::*;
use rand::{seq::SliceRandom, Rng};
use solver::*;
use std::collections::HashMap;
use Face::*;
use MyColor::*;
use RotType::*;
use Rotation::*;
use SubCube::*;

type Id = usize;

//penser a supprimer les derive et debug inutils

#[derive(Clone, Copy, Debug, PartialEq)]
enum MyColor {
    Green,
    Blue,
    Orange,
    Yellow,
    Red,
    White,
    Void,
}

impl std::fmt::Display for MyColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Green => "  ".on_bright_green(),
            Blue => "  ".on_bright_blue(),
            Orange => "  ".on_yellow(),
            Yellow => "  ".on_bright_yellow(),
            Red => "  ".on_bright_red(),
            White => "  ".on_bright_white(),
            Void => "  ".on_black(),
        }
        .fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Face {
    Up,
    Down,
    Front,
    Back,
    Left,
    Right,
}

impl Face {
    const FACE_SET: [Face; 6] = [Up, Down, Front, Back, Left, Right];
}

impl std::fmt::Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Up => "U",
            Down => "D",
            Front => "F",
            Back => "B",
            Left => "L",
            Right => "R",
        }
        .fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Rotation {
    Cw,
    Ccw,
}

impl Rotation {
    const ROT_SET: [Rotation; 2] = [Cw, Ccw];
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Ccw = self {
            return "'".fmt(f);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RotType {
    Single,
    Dual,
}

impl RotType {
    const TYPE_SET: [RotType; 2] = [Single, Dual];
}

#[derive(Debug)]
enum SubCube {
    Core,
    Center(Face, MyColor),
    Edge([Face; 2], [MyColor; 2]),
    Corner([Face; 3], [MyColor; 3]),
}

//debug
impl std::fmt::Display for SubCube {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        format!("{:?}", self).fmt(f)
    }
}

pub struct Cube {
    ids: Vec<Id>,
    subs: [SubCube; 27],
}

//debug
impl std::fmt::Debug for Cube {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for layer in self
            .ids
            .iter()
            .map(|id| (id, &self.subs[*id]))
            .collect::<Vec<_>>()
            .chunks(3)
            .collect::<Vec<_>>()
            .chunks(3)
        {
            for row in layer {
                writeln!(
                    f,
                    "{:<3}{:45} {:<3}{:45} {:<3}{:45}",
                    row[0].0, row[0].1, row[1].0, row[1].1, row[2].0, row[2].1
                )?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
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
    const FACE_CHAINS: [[Face; 4]; 3] = [
        [Up, Right, Down, Left],
        [Up, Front, Down, Back],
        [Back, Right, Front, Left],
    ];

    const FACE_MAP: [[Id; 9]; 6] = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8],
        [18, 19, 20, 21, 22, 23, 24, 25, 26],
        [6, 7, 8, 15, 16, 17, 24, 25, 26],
        [0, 1, 2, 9, 10, 11, 18, 19, 20],
        [0, 3, 6, 9, 12, 15, 18, 21, 24],
        [2, 5, 8, 11, 14, 17, 20, 23, 26],
    ];

    const COLOR_MAP: [MyColor; 6] = [Blue, Green, White, Yellow, Orange, Red];

    const MOV_SET: [(Face, Rotation, RotType); 18] = [
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

    fn new() -> Self {
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

    fn rotate(&mut self, face: Face, rot: Rotation, typ: RotType) {
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

fn mov_parser(mov: &str) -> Result<(Face, Rotation, RotType), String> {
    match mov {
        "U" => Ok((Up, Cw, Single)),
        "U2" => Ok((Up, Cw, Dual)),
        "U'" => Ok((Up, Ccw, Single)),
        "D" => Ok((Down, Cw, Single)),
        "D2" => Ok((Down, Cw, Dual)),
        "D'" => Ok((Down, Ccw, Single)),
        "F" => Ok((Front, Cw, Single)),
        "F2" => Ok((Front, Cw, Dual)),
        "F'" => Ok((Front, Ccw, Single)),
        "B" => Ok((Back, Cw, Single)),
        "B2" => Ok((Back, Cw, Dual)),
        "B'" => Ok((Back, Ccw, Single)),
        "L" => Ok((Left, Cw, Single)),
        "L2" => Ok((Left, Cw, Dual)),
        "L'" => Ok((Left, Ccw, Single)),
        "R" => Ok((Right, Cw, Single)),
        "R2" => Ok((Right, Cw, Dual)),
        "R'" => Ok((Right, Ccw, Single)),
        _ => Err(format!("Face \"{}\" is not recognized", mov)),
    }
}

fn input_checker(input: &str) -> Result<(), String> {
    for mov in input.split_whitespace() {
        mov_parser(mov)?;
    }
    Ok(())
}

fn new_app() -> App<'static> {
    App::new("Rubik Solver")
        .author("Cadet Florian, cadet.florian@gmail.com")
        .arg(
            Arg::new("MOVES")
                .validator(input_checker)
                .exclusive(true)
                .required_unless_present_any(&["rand", "new"])
                .help(
                    "Face rotations splited by whitespaces.\n\
                    U, D, F, B, L, R for Up, Down, Front, Back, Left and Right\n\
                    (append 2 for half turn and ' for counterclockwise)",
                ),
        )
        .arg(
            Arg::new("rand")
                .long("rand")
                .short('r')
                .help("<NB> of random moves")
                .require_equals(true)
                .value_name("NB")
                .validator(|arg| usize::from_str_radix(arg, 10)),
        )
        .arg(
            Arg::new("group")
                .long("group")
                .short('g')
                .requires("rand")
                .help("Allowed moves when the cube is shuffled")
                .require_equals(true)
                .value_name("GR")
                .possible_values(["0", "1", "2", "3"]),
        )
        .arg(
            Arg::new("new")
                .long("new")
                .short('n')
                .conflicts_with("rand")
                .help("Start with an unaltered cube"),
        )
}

fn pick_mov(group: usize, rng: &mut rand::rngs::ThreadRng) -> (Face, Rotation, RotType) {
    *Cube::MOV_SET[..Cube::MOV_SET.len() - group * 4]
        .choose(rng)
        .unwrap()
}

fn rand_movs(mov_nb: usize, group: usize) -> Vec<(Face, Rotation, RotType)> {
    let mut rng = rand::thread_rng();
    let mut result: Vec<(Face, Rotation, RotType)> = Vec::with_capacity(mov_nb);

    for _ in 0..mov_nb {
        result.push({
            let mut mv = pick_mov(group, &mut rng);

            if result.len() != 0 {
                while mv.0 == result.last().unwrap().0 {
                    mv = pick_mov(group, &mut rng);
                }
            }
            mv
        })
    }
    result
}

fn disp_mov(face: Face, rot: Rotation, typ: RotType) {
    print!(
        "{}{} ",
        face.to_string().bright_yellow(),
        if let Dual = typ {
            "2".bright_red()
        } else {
            rot.to_string().bright_red()
        }
    );
}

fn app_init(cube: &mut Cube) {
    let cmd = new_app().get_matches();

    println!("");
    if !cmd.is_present("new") {
        print!("{}", "MOVES: ".bright_green());
        for (face, rot, typ) in if cmd.is_present("rand") {
            rand_movs(
                usize::from_str_radix(cmd.value_of("rand").unwrap(), 10).unwrap(),
                usize::from_str_radix(cmd.value_of("group").unwrap_or("0"), 10).unwrap(),
            )
        } else {
            cmd.value_of("MOVES")
                .unwrap()
                .split_whitespace()
                .map(|mov| mov_parser(mov).unwrap())
                .collect()
        } {
            disp_mov(face, rot, typ);
            cube.rotate(face, rot, typ);
        }
        println!("\n");
    }
    println!("{}", cube);
}

fn main() {
    let mut cube = Cube::new();

    app_init(&mut cube);
    let mut solver = Solver::new(cube);

    solver.solve();
}
