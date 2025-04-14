use std::{fs, path::PathBuf};

use crate::io::IO;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    BadHeader,
}

/// An NES cartridge (currently only supports NROM-128, Mapper 0)
pub struct Cart {
    // 16 byte header
    raw_header: [u8; 0x10],
    // 16 KB PRG ROM
    pub prg_rom: [u8; 0x4000],
    // 8 KB CHR ROM
    pub chr_rom: [u8; 0x2000],
}

impl Cart {
    pub fn new(rom: &[u8]) -> Result<Self, Error> {
        let mut raw_header = [0; 0x10];
        raw_header.clone_from_slice(&rom[0..0x10]);

        if &raw_header[0..0x04] != b"NES\x1A" {
            return Err(Error::BadHeader);
        }

        let mut prg_rom = [0; 0x4000];
        prg_rom.clone_from_slice(&rom[0x10..0x4010]);

        let mut chr_rom = [0; 0x2000];
        chr_rom.clone_from_slice(&rom[0x4010..0x6010]);

        Ok(Self {
            raw_header,
            prg_rom,
            chr_rom,
        })
    }

    pub fn load(rom_path: &PathBuf) -> Result<Self, Error> {
        let data = fs::read(&rom_path).map_err(Error::IoError)?;

        Self::new(data.as_slice())
    }
}

impl IO for Cart {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // 8KB CHR ROM. The PPU reads from 0x000.
            0x000..=0x1FFF => self.chr_rom[addr as usize],
            // Mirror 16 KB PRG ROM. The CPU reads starting from 0x8000.
            0x8000..=0xFFFF => self.prg_rom[((addr as usize) - 0x8000) % 0x4000],
            _ => {
                // TODO: implement CHR ROM
                0
            }
        }
    }

    fn write(&mut self, _addr: u16, _data: u8) {
        // no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_rom() -> Vec<u8> {
        let mut rom: [u8; 0x6010] = [0; 0x6010];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom.to_vec()
    }

    #[test]
    fn test_new_rom() {
        assert!(Cart::new(&create_rom()).is_ok());
    }

    #[test]
    fn test_bad_header() {
        let mut rom: [u8; 0x6010] = [0; 0x6010];
        rom[0..4].copy_from_slice(b"SEGA");
        assert!(matches!(Cart::new(&rom), Err(Error::BadHeader)));
    }

    #[test]
    fn test_nrom_prog_rom_mirroring() {
        let mut rom = create_rom();
        rom[0x10] = 0xAB;
        rom[0x11] = 0xCD;

        let mut cart = Cart::new(&rom).unwrap();
        assert_eq!(cart.read(0x8000), 0xAB);
        assert_eq!(cart.read(0xC000), 0xAB);
        assert_eq!(cart.read(0x8001), 0xCD);
        assert_eq!(cart.read(0xC001), 0xCD);
    }

    #[test]
    fn test_nrom_chr_rom() {
        let mut rom = create_rom();
        rom[0x4020] = 0xAB;

        let mut cart = Cart::new(&rom).unwrap();
        assert_eq!(cart.read(0x0010), 0xAB);
    }
}
