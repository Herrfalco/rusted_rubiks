use std::fs;

struct Compressor {
    buff: u128,
    len: usize,
    bin: Vec<u8>,
}

impl Compressor {
    fn new() -> Compressor {
        Compressor {
            buff: 0,
            len: 0,
            bin: Vec::new(),
        }
    }

    fn append<T>(&mut self, data: T, size: usize)
    where
        T: std::ops::Shl<Output = u128>,
        u128: From<T>,
    {
        self.buff |= (u128::from(data)) << self.len;
        self.len += size;
    }

    fn flush_byte(&mut self) {
        self.bin.push((self.buff & 0xff) as u8);
        self.buff >>= 8;
        self.len = self.len.saturating_sub(8);
    }

    fn flush(&mut self, all: bool) {
        while self.len > if all { 0 } else { 7 } {
            self.flush_byte();
        }
    }

    fn save(&self, file: &str) {
        fs::write(file, &self.bin).unwrap();
    }
}

struct Decompressor {}
