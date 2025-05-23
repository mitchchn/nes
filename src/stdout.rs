use crate::io::IO;
use std::io::Write;

/// Simple stdout device for 6502
pub struct Stdout {
    buffer: [u8; 4096],
    pos: usize,
}

impl Stdout {
    pub fn new() -> Self {
        Stdout {
            buffer: [0; 4096],
            pos: 0,
        }
    }

    pub fn flush(&mut self) {
        let _ = std::io::stdout().write(&self.buffer);
        self.buffer = [0; 4096];
        self.pos = 0;
    }
}

impl IO for Stdout {
    fn read(&mut self, _addr: u16) -> u8 {
        0
    }
    fn write(&mut self, _addr: u16, data: u8) {
        self.buffer[self.pos] = data;
        self.pos += 1;
    }
}
