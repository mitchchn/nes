use crate::io::IO;
use std::iter::FromIterator;

/// Display from Easy6502
/// 32x32 pixels
///
/// https://skilldrick.github.io/easy6502/
///
/// Responds to addresses $0200 - $05ff.
pub struct Display {
    buffer: [u8; 32 * 32],
}

impl Display {
    pub fn new() -> Self {
        Display {
            buffer: [0; 32 * 32],
        }
    }

    pub fn flush(&mut self) {
        print!(
            "{}",
            String::from_iter(self.buffer.iter().map(|c| *c as char))
        );
        self.buffer = [0; 32 * 32];
    }
}

impl IO for Display {
    fn read(&mut self, addr: u16) -> u8 {
        self.buffer[addr as usize]
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.buffer[addr as usize] = data
    }
}
