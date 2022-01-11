use super::*;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;

trait HashMove<K> {
    fn ins_min(&mut self, key: K, comb: &Vec<(Face, Rotation, RotType)>);
    fn get_moves(&self, key: K) -> Box<dyn Iterator<Item = (Face, Rotation, RotType)> + '_>;
    fn save(&self, file: &str);
    fn load(&mut self, file: &str);
    fn disp(&self, key: K, title: &str);
    fn exec(&self, key: K, cube: &mut Cube);

    fn mov_2_u8(face: Face, rot: Rotation, typ: RotType) -> u8 {
        ((face as u8) << 4) | ((rot as u8) << 1) | (typ as u8)
    }
    fn comb_2_rev_u8(comb: &Vec<(Face, Rotation, RotType)>) -> Vec<u8> {
        comb.iter()
            .rev()
            .map(|(face, rot, typ)| {
                Self::mov_2_u8(*face, if let Cw = *rot { Ccw } else { Cw }, *typ)
            })
            .collect()
    }
    fn u8_2_mov(mov: u8) -> (Face, Rotation, RotType) {
        (
            Face::FACE_SET[(mov >> 4) as usize],
            Rotation::ROT_SET[((mov >> 1) & 0b1) as usize],
            RotType::TYPE_SET[(mov & 0b1) as usize],
        )
    }
}

impl<K> HashMove<K> for HashMap<K, Vec<u8>>
where
    K: Eq + Hash + Serialize + DeserializeOwned,
{
    fn ins_min(&mut self, key: K, comb: &Vec<(Face, Rotation, RotType)>) {
        match self.get_mut(&key) {
            Some(val) => {
                if val.len() > comb.len() {
                    *val = Self::comb_2_rev_u8(comb);
                }
            }
            None => {
                self.insert(key, Self::comb_2_rev_u8(comb));
            }
        }
    }

    fn get_moves(&self, key: K) -> Box<dyn Iterator<Item = (Face, Rotation, RotType)> + '_> {
        Box::new(self[&key].iter().map(|mv| Self::u8_2_mov(*mv)))
    }

    fn save(&self, file: &str) {
        fs::write(file, bincode::serialize(self).unwrap()).unwrap();
    }

    fn load(&mut self, file: &str) {
        let bin = fs::read(file).unwrap();

        *self = bincode::deserialize(&bin).unwrap();
    }

    fn disp(&self, key: K, title: &str) {
        print!("{}", format!("{}: ", title).bright_green());

        for (face, rot, typ) in self.get_moves(key) {
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
    }

    fn exec(&self, key: K, cube: &mut Cube) {
        for (face, rot, typ) in self.get_moves(key) {
            cube.rotate(face, rot, typ);
        }
    }
}

pub struct Solver<'a> {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    cube: &'a mut Cube,
}

impl<'a, 'de> Solver<'a> {
    pub fn new(cube: &mut Cube) -> Solver {
        Solver {
            mov_stack: Vec::with_capacity(128),
            cube,
        }
    }

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

    //12bit key
    fn key_gen_1(&mut self) -> u16 {
        let mut result: u16 = 0;

        for (face_i, face) in Cube::FACE_CHAINS[2].iter().enumerate() {
            for idx in match face {
                Front | Left => [1, 3, 7],
                _ => [1, 5, 7],
            } {
                if let Edge(dir, col) =
                    &mut self.cube.subs[self.cube.ids[Cube::FACE_MAP[*face as usize][idx]]]
                {
                    let (face_j, col_i) = Cube::FACE_CHAINS[2]
                        .iter()
                        .enumerate()
                        .find_map(|(face_j, f)| {
                            match col.iter().position(|c| Cube::COLOR_MAP[*f as usize] == *c) {
                                Some(col_i) => Some((face_j, col_i)),
                                None => None,
                            }
                        })
                        .unwrap();

                    result = (result << 1)
                        | (((face_i + 4 - face_j) % 2) ^ if dir[col_i] == *face { 0 } else { 1 })
                            as u16;
                } else {
                    panic!("Not an edge");
                }
            }
        }
        result
    }

    //36bit key
    fn key_gen_2(&mut self) -> u64 {
        let mut result = 0;

        for id in 0..27 {
            result = match self.cube.subs[id] {
                Corner(dirs, cols) => (result << 3) | dirs[0] as u64,
                Edge(dirs, cols) => {
                    (result << 1)
                        | ((Cube::FACE_MAP[Left as usize].contains(&id)
                            || Cube::FACE_MAP[Right as usize].contains(&id))
                            as u64)
                }
                _ => continue,
            }
        }
        result
    }

    //16bit key
    fn key_gen_3(&mut self) -> u16 {
        let mut result = 0;

        for face in &Cube::FACE_MAP[4..] {
            for pos in face {
                let (dir, col) = match self.cube.subs[self.cube.ids[*pos]] {
                    Edge(dirs, cols) => (dirs[1], cols[1]),
                    Corner(dirs, cols) => (dirs[1], cols[1]),
                    _ => continue,
                };
                result = (result << 1)
                    | (col != Cube::COLOR_MAP[dir as usize]
                        && col
                            != Cube::COLOR_MAP[if dir as usize % 2 == 0 {
                                dir as usize + 1
                            } else {
                                dir as usize - 1
                            }]) as u16;
            }
        }
        result
    }

    fn rec_search<K>(
        &mut self,
        sol: &mut HashMap<K, Vec<u8>>,
        key_gen: fn(&mut Self) -> K,
        set_sz: usize,
        rank: usize,
    ) where
        K: Eq + Hash + Serialize + DeserializeOwned,
    {
        sol.ins_min(key_gen(self), &mut self.mov_stack);
        if rank > 0 {
            for (face, rot, typ) in Cube::MOV_SET[..set_sz].iter() {
                self.do_mov(*face, *rot, *typ);
                self.rec_search(sol, key_gen, set_sz, rank - 1);
                self.undo_mov();
            }
        }
    }

    pub fn extract_table_1(&mut self) {
        //capacity ?
        let mut table: HashMap<u16, Vec<u8>> = HashMap::with_capacity(2048);

        self.rec_search(&mut table, Self::key_gen_1, Cube::MOV_SET.len(), 7);
        table.save("table_1");
    }

    pub fn extract_table_2(&mut self) {
        //capacity ?
        let mut table: HashMap<u64, Vec<u8>> = HashMap::with_capacity(1_082_565);

        self.rec_search(&mut table, Self::key_gen_2, Cube::MOV_SET.len() - 4, 10);
        table.save("table_2");
    }

    pub fn extract_table_3(&mut self) {
        //capacity ?
        let mut table: HashMap<u16, Vec<u8>> = HashMap::with_capacity(29_400);

        self.rec_search(&mut table, Self::key_gen_3, Cube::MOV_SET.len() - 8, 13);
        table.save("table_3");
    }

    pub fn solve(&mut self) {
        //  self.extract_table_1();
        // self.extract_table_2();
        // self.extract_table_3();
        println!("{:012b}", self.key_gen_1());
        println!("{:036b}", self.key_gen_2());
        println!("{:016b}", self.key_gen_3());

        /*
        //capacity ?
        let mut table_1: HashMap<u16, Vec<u8>> = HashMap::with_capacity(2048);

        table_1.load("table_1");

        let key = self.key_gen_1();
        table_1.disp(key, "PHASE 1");
        table_1.exec(key, self.cube);
        println!("\n\n{}", self.cube);
        */

        /*
        //capacity ?
        let mut table_2: HashMap<u32, Vec<u8>> = HashMap::with_capacity(6561);

        table_2.load("table_2");

        print!("{}", "PHASE 2: ".bright_green());

        for (face, rot, typ) in table_2[&self.key_gen_2()]
            .iter()
            .map(|mv| Self::u8_2_mov(*mv))
        {
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
            self.cube.rotate(face, rot, typ);
        }
        println!("\n\n{}", self.cube);
        */
    }
}
