use std::fs;

use crate::io::IO;

// TODO: NES header offset is 0x10 bytes, split it there.
// For now we are testing with non-compliant ROMs that start at 0x00.
pub const HEADER_OFFSET: u8 = 0x00;

// 32 KB NROM
pub struct Rom {
    prg: [u8; 0x800],
    rom: [u8; 0x8000],
}

impl Rom {
    pub fn new() -> Self {
        Self {
            prg: [0; 0x800],
            rom: [0; 0x8000],
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self.rom[i] = *byte;
        }

        // Init reset vector
        self.rom[0x7FFC] = 0x00;
        self.rom[0x7FFD] = 0x80;
    }
}

impl IO for Rom {
    fn read(&mut self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
    fn write(&mut self, _addr: u16, _data: u8) {
        // no-op
    }
}
