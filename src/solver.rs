use super::*;
use crossbeam::thread;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fs, hash::Hash, mem::size_of};
use LoadStep::*;

#[derive(Clone, Copy)]
enum LoadStep {
    InKey,
    InSize,
    InMove,
}

trait HashMove<K> {
    fn ins_min(&mut self, key: K, comb: Vec<u8>);
    fn get_moves(&self, key: K) -> Box<dyn Iterator<Item = (Face, Rotation, RotType)> + '_>;
    fn save(&self, file: &str);
    fn load(&mut self, file: &str);
    fn disp(&self, key: K, title: &str);
    fn exec(&self, key: K, cube: &mut Cube);
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
    K: Eq + std::fmt::Display + Hash + Copy + Serialize + DeserializeOwned,
{
    fn ins_min(&mut self, key: K, comb: Vec<u8>) {
        match self.get_mut(&key) {
            Some(val) => {
                if val.len() > comb.len() {
                    *val = comb;
                }
            }
            None => {
                self.insert(key, comb);
            }
        }
    }

    fn get_moves(&self, key: K) -> Box<dyn Iterator<Item = (Face, Rotation, RotType)> + '_> {
        Box::new(self[&key].iter().map(|mv| Self::u8_2_mov(*mv)))
    }

    fn save(&self, file: &str) {
        let mut bin: Vec<u8> = Vec::new();

        for (k, v) in self {
            for b in bincode::serialize(&k).unwrap() {
                bin.push(b);
            }
            bin.push(v.len() as u8);
            for m in v {
                bin.push(*m);
            }
        }
        fs::write(file, bin).unwrap();
    }

    fn load(&mut self, file: &str) {
        let mut key: Vec<u8> = Vec::with_capacity(size_of::<K>());
        let mut size: usize = 0;
        let mut movs: Vec<u8> = Vec::new();
        let mut step = InKey;

        for byte in fs::read(file).unwrap() {
            match step {
                InKey => {
                    key.push(byte);
                    if key.len() == size_of::<K>() {
                        step = InSize;
                    }
                }
                InSize => {
                    size = byte as usize;
                    step = if size > 0 {
                        InMove
                    } else {
                        self.insert(bincode::deserialize(&key).unwrap(), movs.clone());
                        key.clear();
                        InKey
                    };
                }
                InMove => {
                    movs.push(byte);
                    size -= 1;
                    if size == 0 {
                        self.insert(bincode::deserialize(&key).unwrap(), movs.clone());
                        key.clear();
                        movs.clear();
                        step = InKey;
                    }
                }
            }
        }
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

pub struct Solver {
    mov_stack: Vec<(Face, Rotation, RotType)>,
    cube: Cube,
}

impl<'de> Solver {
    pub fn new(cube: Cube) -> Solver {
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

    //OK
    //40bit key
    fn key_gen_4(&mut self) -> u64 {
        let mut result = 0;

        for sub in &self.cube.subs {
            let (dir_1, dir_2, col_1, col_2) = match sub {
                Corner(dirs, cols) => (dirs[0], dirs[1], cols[0], cols[1]),
                Edge(dirs, cols) => (dirs[0], dirs[1], cols[0], cols[1]),
                _ => continue,
            };
            result = (result << 2)
                | (((col_1 != Cube::COLOR_MAP[dir_1 as usize]) as u64) << 1)
                | ((col_2 != Cube::COLOR_MAP[dir_2 as usize]) as u64)
        }
        result
    }

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

    fn rec_search<K>(
        &mut self,
        sol: &mut HashMap<K, Vec<u8>>,
        key_gen: fn(&mut Self) -> K,
        set_sz: usize,
        rank: usize,
    ) where
        K: Eq + std::fmt::Display + Hash + Copy + Serialize + DeserializeOwned,
    {
        sol.ins_min(key_gen(self), Self::comb_2_rev_u8(&self.mov_stack));
        if rank > 0 {
            for (face, rot, typ) in Cube::MOV_SET[..set_sz].iter() {
                self.do_mov(*face, *rot, *typ);
                self.rec_search(sol, key_gen, set_sz, rank - 1);
                self.undo_mov();
            }
        }
    }

    fn mt_search<K>(file: &str, key_gen: fn(&mut Self) -> K, set_sz: usize, rank: usize, cap: usize)
    where
        K: Eq + std::fmt::Display + Hash + Copy + Serialize + DeserializeOwned + std::marker::Send,
    {
        thread::scope(|s| {
            let mut thrds = Vec::with_capacity(set_sz);
            let mut result: HashMap<K, Vec<u8>> = HashMap::with_capacity(cap);

            for (face, rot, typ) in &Cube::MOV_SET[..set_sz] {
                thrds.push(s.spawn(|_| {
                    let mut solver = Solver::new(Cube::new());
                    let mut table: HashMap<K, Vec<u8>> = HashMap::with_capacity(cap);

                    table.ins_min(key_gen(&mut solver), Self::comb_2_rev_u8(&solver.mov_stack));
                    solver.do_mov(*face, *rot, *typ);
                    solver.rec_search(&mut table, key_gen, set_sz, rank - 1);
                    table
                }));
            }

            for thrd in thrds {
                for (key, val) in thrd.join().unwrap() {
                    result.ins_min(key, val);
                }
            }
            result.save(file);
        })
        .unwrap();
    }

    pub fn table_search() {
        Self::mt_search("mt_table_1", Self::key_gen_1, Cube::MOV_SET.len(), 7, 2_048);
        println!("table_1 completed");
        Self::mt_search(
            "mt_table_2",
            Self::key_gen_2,
            Cube::MOV_SET.len() - 4,
            10,
            1_082_565,
        );
        println!("table_2 completed");
        Self::mt_search(
            "mt_table_3",
            Self::key_gen_3,
            Cube::MOV_SET.len() - 8,
            13,
            29_400,
        );
        println!("table_3 completed");
    }

    pub fn solve(&mut self) {
        //capacity ?
        /*
        let mut table_1: HashMap<u16, Vec<u8>> = HashMap::with_capacity(2048);

        table_1.load("mt_table_1");
        let key = self.key_gen_1();
        table_1.disp(key, "PHASE 1");
        table_1.exec(key, &mut self.cube);
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
