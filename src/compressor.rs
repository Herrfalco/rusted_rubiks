use std::fs;

pub struct Compressor {
    buff: u128,
    len: usize,
    bin: Vec<u8>,
}

impl Compressor {
    pub fn new() -> Compressor {
        Compressor {
            buff: 0,
            len: 0,
            bin: Vec::new(),
        }
    }

    pub fn push<T>(&mut self, data: T, size: usize)
    where
        u128: From<T>,
    {
        if self.len + size > 128 {
            panic!("compressor overflow");
        }
        self.buff |= u128::from(data) << (128 - (self.len + size));
        self.len += size;
        self.flush(false);
    }

    fn flush_byte(&mut self) {
        self.bin.push((self.buff >> 120) as u8);
        self.buff <<= 8;
        self.len = self.len.saturating_sub(8);
    }

    pub fn flush(&mut self, all: bool) {
        while self.len > if all { 0 } else { 7 } {
            self.flush_byte();
        }
    }

    pub fn save(&mut self, file: &str) {
        self.flush(true);
        fs::write(file, &self.bin.iter().rev().cloned().collect::<Vec<u8>>()).unwrap();
    }
}

pub struct Decompressor {
    buff: u128,
    len: usize,
    bin: Vec<u8>,
}

impl Decompressor {
    pub fn new(file: &str) -> Decompressor {
        Decompressor {
            buff: 0,
            len: 0,
            bin: fs::read(file).unwrap(),
        }
    }

    pub fn pop<T>(&mut self, size: usize) -> Option<T>
    where
        T: TryFrom<u128>,
        <T as TryFrom<u128>>::Error: std::fmt::Debug,
    {
        if size > (self.len + self.bin.len() * 8) {
            return None;
        }
        self.fetch_until(size);
        let result =
            T::try_from((self.buff & ((!(!0 << size)) << (self.len - size))) >> (self.len - size))
                .unwrap();
        self.len -= size;
        Some(result)
    }

    fn fetch_until(&mut self, size: usize) {
        while self.len < size {
            self.fetch_byte();
        }
    }

    fn fetch_byte(&mut self) {
        self.buff <<= 8;
        self.buff |= self.bin.pop().unwrap() as u128;
        self.len += 8;
    }
}
