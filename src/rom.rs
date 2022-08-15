use std::fs;

use crate::io::IO;

// TODO: NES header offset is 0x10 bytes, split it there.
// For now we are testing with non-compliant ROMs that start at 0x00.
pub const HEADER_OFFSET: usize = 0x10;

// 32 KB NROM
pub struct Rom {
    header: [u8; HEADER_OFFSET],
    prg: [u8; 0x2000],
    rom: [u8; 0x4000],
}

impl Rom {
    pub fn new() -> Self {
        Self {
            prg: [0; 0x2000],
            rom: [0; 0x4000],
            header: [0; HEADER_OFFSET],
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        let (header, rom) = data.split_at(HEADER_OFFSET);

        for (dst, byte) in self.header.iter_mut().zip(header) {
            *dst = *byte;
        }

        for (dst, byte) in self.rom.iter_mut().zip(rom) {
            *dst = *byte;
        }

        // Init reset vector at end of cart so it's there when the machine looks at 0xFFFC and 0xFFFD
        self.rom[0x3FFC] = 0x00;
        self.rom[0x3FFD] = 0xC0;
    }
}

impl IO for Rom {
    fn read(&mut self, addr: u16) -> u8 {
        // TODO: do this in the mapper instead of mapping twice?

        // In a 16 KB cart, 0xC000 - 0xFFFF is a mirror of $8000 - $BFFF
        match addr {
            0x6000..=0x7FFF => self.prg[(addr - 0x6000) as usize],
            0x8000..=0xBFFF => self.rom[(addr - 0x8000) as usize],
            0xC000..=0xFFFF => self.rom[(addr - 0x8000 - 0x4000) as usize],
            _ => 0,
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        // Only allow writing to the PRG (battery save)
        match addr {
            0x000..=0x1FFF => {
                self.prg[addr as usize] = data;
            }
            _ => {}
        }
    }
}
