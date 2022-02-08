use super::*;

pub type Id = usize;

#[derive(Clone, Copy, PartialEq)]
pub enum MyColor {
    Green,
    Blue,
    Orange,
    Yellow,
    Red,
    White,
    Void,
}

impl MyColor {
    pub const COL_SET: [MyColor; 6] = [Blue, Green, White, Yellow, Orange, Red];
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

#[derive(Clone, Copy, PartialEq)]
pub enum Face {
    Up,
    Down,
    Front,
    Back,
    Left,
    Right,
}

impl Face {
    pub const FACE_SET: [Face; 6] = [Up, Down, Front, Back, Left, Right];
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

#[derive(Clone, Copy, PartialEq)]
pub enum Rotation {
    Cw,
    Ccw,
}

impl Rotation {
    pub const ROT_SET: [Rotation; 2] = [Cw, Ccw];
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Ccw = self {
            return "'".fmt(f);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum RotType {
    Single,
    Dual,
}

impl RotType {
    pub const TYPE_SET: [RotType; 2] = [Single, Dual];
}

#[derive(Clone, Copy)]
pub struct Move(pub Face, pub Rotation, pub RotType);

impl std::fmt::Display for Move {
    fn fmt(&self, fm: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Move(face, rot, typ) = self;

        write!(
            fm,
            "{}{}",
            face.to_string().bright_yellow(),
            if *typ == Dual {
                "2".bright_red()
            } else {
                rot.to_string().bright_red()
            }
        )
    }
}
