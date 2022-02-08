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

fn input_checker(input: &str) -> Result<(), String> {
    for mov in input.split_whitespace() {
        Cube::mov_parser(mov)?;
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
        let cube = if cmd.is_present("new") {
            Cube::new()
        } else {
            if cmd.is_present("rand") {
                Cube::from_rand(
                    usize::from_str_radix(cmd.value_of("rand").unwrap(), 10).unwrap(),
                    usize::from_str_radix(cmd.value_of("group").unwrap_or("0"), 10).unwrap(),
                    true,
                )
            } else {
                Cube::from_str(cmd.value_of("MOVES").unwrap(), true)
            }
        };
        println!("\n{}", cube);
        Solver::new(cube).solve();
    }
}
