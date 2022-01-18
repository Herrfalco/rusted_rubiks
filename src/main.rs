mod compressor;
mod cube;
mod solver;
mod utils;

use clap::{App, Arg};
use colored::*;
use cube::*;
use rand::seq::SliceRandom;
use solver::*;
use utils::*;
use Face::*;
use MyColor::*;
use RotType::*;
use Rotation::*;
use SubCube::*;

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
                .required_unless_present_any(&["rand", "new", "tab"])
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
        .arg(
            Arg::new("tab")
                .long("tab")
                .short('t')
                .exclusive(true)
                .require_equals(true)
                .validator(|v| {
                    let mut tabs: Vec<&str> = v.split(',').collect();
                    let len = tabs.len();

                    tabs.sort();
                    tabs.dedup();
                    if len < 1 || len > 4 {
                        return Err("too many tables");
                    } else if len != tabs.len() {
                        return Err("table duplicate");
                    } else {
                        for tab in tabs {
                            if ["1", "2", "3", "4"].contains(&tab) == false {
                                return Err("invalid table number");
                            }
                        }
                    }
                    Ok(())
                })
                .value_name("IDS")
                .help("Compute tables (<IDS> from 1 to 4 separated by commas)"),
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

fn main() {
    let cmd = new_app().get_matches();

    if cmd.is_present("tab") {
        let tabs = cmd
            .value_of("tab")
            .unwrap()
            .split(",")
            .collect::<Vec<&str>>();
        Solver::table_search(
            tabs.iter()
                .map(|t| usize::from_str_radix(t, 10).unwrap())
                .collect(),
        );
    } else {
        let mut cube = Cube::new();

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
        Solver::new(cube).solve();
    }
}
