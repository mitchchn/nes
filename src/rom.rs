use std::fs;

use crate::io::IO;

// TODO: NES header offset is 0x10 bytes, split it there.
// For now we are testing with non-compliant ROMs that start at 0x00.
pub const HEADER_OFFSET: u8 = 0x00;

pub struct Rom {
    header: Vec<u8>,
    data: Vec<u8>,
}

impl Rom {
    pub fn new() -> Self {
        Self {
            data: vec![],
            header: vec![],
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        let (header, data) = data.split_at(HEADER_OFFSET as usize);
        self.header = header.iter().cloned().collect();
        self.data = data.iter().cloned().collect();
    }
}

impl IO for Rom {
    fn read(&mut self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
    fn write(&mut self, _addr: u16, _data: u8) {
        // no-op
    }
}
