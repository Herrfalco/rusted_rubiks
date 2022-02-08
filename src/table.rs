use super::*;
use compressor::*;

pub struct TableInfos {
    pub id: Id,
    pub key_gen: fn(&Cube) -> u64,
    pub key_sz: usize,
    pub set_sz: usize,
    pub rank: usize,
    pub cap: usize,
}

pub trait Table {
    fn ins_min(&mut self, key: u64, movs: Vec<u8>);
    fn save(&self, file: &str, key_sz: usize);
    fn load(&mut self, file: &str, key_sz: usize);
    fn exec(&self, key: u64, cube: &mut Cube) -> String;
    fn u8_2_mov(mov: u8) -> Move {
        Move(
            Face::FACE_SET[(mov >> 2) as usize],
            Rotation::ROT_SET[((mov >> 1) & 0b1) as usize],
            RotType::TYPE_SET[(mov & 0b1) as usize],
        )
    }
}

impl Table for HashMap<u64, Vec<u8>> {
    fn ins_min(&mut self, key: u64, movs: Vec<u8>) {
        match self.get_mut(&key) {
            Some(val) => {
                if val.len() > movs.len() {
                    *val = movs;
                }
            }
            None => {
                self.insert(key, movs);
            }
        }
    }

    fn save(&self, file: &str, key_sz: usize) {
        let mut compressor = Compressor::new();

        for (k, v) in self {
            compressor.push(*k, key_sz);
            compressor.push(v.len() as u8, 4);
            for m in v {
                compressor.push(*m, 5);
            }
        }
        compressor.save(file);
    }

    fn load(&mut self, file: &str, key_sz: usize) {
        let mut decompressor = Decompressor::new(file);

        while let Some(key) = decompressor.pop(key_sz) {
            self.insert(
                key,
                (0..decompressor.pop(4).unwrap())
                    .map(|_| decompressor.pop(5).unwrap())
                    .collect::<Vec<u8>>(),
            );
        }
    }

    fn exec(&self, key: u64, cube: &mut Cube) -> String {
        let mut disp_res = String::new();

        for mv in self[&key].iter().map(|mv| Self::u8_2_mov(*mv)) {
            std::fmt::write(&mut disp_res, format_args!("{} ", mv)).unwrap();
            cube.rotate(mv, true);
        }
        disp_res
    }
}
