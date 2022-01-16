use super::*;
use compressor::*;
use crossbeam::thread;
//use serde::{de::DeserializeOwned, Serialize};
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
    fn save(&self, file: &str, key_sz: usize);
    fn load(&mut self, file: &str, key_sz: usize);
    //    fn old_load(&mut self, file: &str);
    fn disp(&self, key: K, title: &str);
    fn exec(&self, key: K, cube: &mut Cube);
    fn u8_2_mov(mov: u8) -> (Face, Rotation, RotType) {
        (
            Face::FACE_SET[(mov >> 2) as usize],
            Rotation::ROT_SET[((mov >> 1) & 0b1) as usize],
            RotType::TYPE_SET[(mov & 0b1) as usize],
        )
    }
    /*
    fn old_u8_2_u8(mov: u8) -> u8 {
        (mov & 0x3) | ((mov & 0xf0) >> 2)
    }
    */
}

impl<K> HashMove<K> for HashMap<K, Vec<u8>>
where
    K: Eq + std::fmt::Display + Hash + Copy + std::ops::Shl<Output = K> + TryFrom<u128>,
    u128: From<K>,
    <K as TryFrom<u128>>::Error: std::fmt::Debug,
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

    fn save(&self, file: &str, key_sz: usize) {
        let mut compressor = Compressor::new();

        for (k, v) in self {
            compressor.push(*k, key_sz);
            compressor.push::<u8>(v.len() as u8, 4);
            for m in v {
                compressor.push::<u8>(*m, 5);
            }
        }
        compressor.save(file);
    }

    /*
    fn old_load(&mut self, file: &str) {
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
        for (k, v) in self {
            for m in v {
                *m = Self::old_u8_2_u8(*m);
            }
        }
    }
    */

    fn load(&mut self, file: &str, key_sz: usize) {
        let mut decompressor = Decompressor::new(file);

        while let Some(key) = decompressor.pop::<K>(key_sz) {
            self.insert(
                key,
                (0..decompressor.pop::<usize>(4).unwrap())
                    .map(|_| decompressor.pop::<u8>(5).unwrap())
                    .collect::<Vec<u8>>(),
            );
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
    fn key_gen_1(&self) -> u16 {
        let mut result: u16 = 0;

        for (face_i, face) in Cube::FACE_CHAINS[2].iter().enumerate() {
            for idx in match face {
                Front | Left => [1, 3, 7],
                _ => [1, 5, 7],
            } {
                if let Edge(dir, col) =
                    &self.cube.subs[self.cube.ids[Cube::FACE_MAP[*face as usize][idx]]]
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
    fn key_gen_2(&self) -> u64 {
        let mut result = 0;
        let mut id;

        for pos in 0..27 {
            id = self.cube.ids[pos];
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
    fn key_gen_3(&self) -> u16 {
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

    //40bit key
    fn key_gen_4(&self) -> u64 {
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
        ((face as u8) << 2) | ((rot as u8) << 1) | (typ as u8)
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
        key_gen: fn(&Self) -> K,
        set_sz: usize,
        rank: usize,
    ) where
        K: Eq + std::fmt::Display + Hash + Copy + std::ops::Shl<Output = K> + TryFrom<u128>,
        <K as TryFrom<u128>>::Error: std::fmt::Debug,
        u128: From<K>,
    {
        sol.ins_min(key_gen(self), Self::comb_2_rev_u8(&self.mov_stack));
        if rank > 0 {
            for (face, rot, typ) in Cube::MOV_SET[..set_sz].iter() {
                if self.mov_stack.last().unwrap().0 != *face {
                    self.do_mov(*face, *rot, *typ);
                    self.rec_search(sol, key_gen, set_sz, rank - 1);
                    self.undo_mov();
                }
            }
        }
    }

    fn mt_search<K>(
        file: &str,
        key_gen: (fn(&Self) -> K, usize),
        set_sz: usize,
        rank: usize,
        cap: usize,
    ) where
        K: Eq
            + std::fmt::Display
            + Hash
            + Copy
            + std::marker::Send
            + std::ops::Shl<Output = K>
            + TryFrom<u128>,
        <K as TryFrom<u128>>::Error: std::fmt::Debug,
        u128: From<K>,
    {
        thread::scope(|s| {
            let mut thrds = Vec::with_capacity(set_sz);
            let mut result: HashMap<K, Vec<u8>> = HashMap::with_capacity(cap);

            for (face, rot, typ) in &Cube::MOV_SET[..set_sz] {
                thrds.push(s.spawn(|_| {
                    let mut solver = Solver::new(Cube::new());
                    let mut table: HashMap<K, Vec<u8>> = HashMap::with_capacity(cap);

                    table.ins_min(key_gen.0(&mut solver), vec![]);
                    solver.do_mov(*face, *rot, *typ);
                    solver.rec_search(&mut table, key_gen.0, set_sz, rank - 1);
                    table
                }));
            }

            for thrd in thrds {
                for (key, val) in thrd.join().unwrap() {
                    result.ins_min(key, val);
                }
            }
            result.save(file, key_gen.1);
        })
        .unwrap();
        println!("{} completed", file);
    }

    pub fn table_search(table_ids: Vec<usize>) {
        for id in table_ids {
            match id {
                1 => Self::mt_search("mt_table_1", (Self::key_gen_1, 12), 18, 7, 2_048),
                2 => Self::mt_search("mt_table_2", (Self::key_gen_2, 36), 14, 10, 1_082_565),
                3 => Self::mt_search("mt_table_3", (Self::key_gen_3, 16), 10, 13, 29_400),
                4 => Self::mt_search("mt_table_4", (Self::key_gen_4, 40), 6, 15, 663_552),
                _ => panic!("unknown table"),
            };
        }
    }

    pub fn solve(&mut self) {
        //Self::table_search(vec![1]);
        {
            let mut table_1: HashMap<u16, Vec<u8>> = HashMap::with_capacity(2_048);
            table_1.load("mt_table_1", 12);
            println!("table_1: {}", table_1.len());
            let key = self.key_gen_1();
            table_1.disp(key, "PHASE 1");
            table_1.exec(key, &mut self.cube);
            println!("\n\n{}", self.cube);
        }
        {
            let mut table_2: HashMap<u64, Vec<u8>> = HashMap::with_capacity(1_082_565);
            table_2.load("mt_table_2", 36);
            println!("table_2: {}", table_2.len());
            let key = self.key_gen_2();
            table_2.disp(key, "PHASE 2");
            table_2.exec(key, &mut self.cube);
            println!("\n\n{}", self.cube);
        }
    }
}
