use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};

use crate::mem::Memory;
use crate::{io::IO, rom::Rom};

pub const RAM_START: u16 = 0x0000;
pub const RAM_END: u16 = 0x07FF;
pub const CART_START: u16 = 0x6000;
pub const PRG_START: u16 = 0x6000;
pub const PRG_END: u16 = 0x7FFF;
pub const ROM_START: u16 = 0x8000;
pub const ROM_END: u16 = 0xFFFF;
pub const CART_END: u16 = 0xFFFF;

/// 6502 Memory Mapper
///
/// ### Memory Layout
///
/// https://www.nesdev.org/wiki/CPU_memory_map
///
/// - $0000-$07FF: 2KB internal RAM
///     - $0000-$00FF: Zero Page
///     - $0100-$01FF: Stack
/// - $4020-$FFFF: Cartridge memory (32 KB NROM: https://www.nesdev.org/wiki/NROM)
///     - $6000-$7FFF: PRG RAM, mirrored as necessary to fill entire 8 KiB window, write protectable with an external switch
///     - $8000-$BFFF: First 16 KB of ROM.
///     - $C000-$FFFF: Last 16 KB of ROM (NROM-256) or mirror of $8000-$BFFF (NROM-128).
/// - Interrupt vectors (on cartridge)
///     - $FFFA-$FFFB: NMI vector
///     - $FFFC-$FFFD: Reset vector
///     - $FFFE-$FFFF: IRQ/BRK vector
pub struct Mapper {
    pub mem: Memory,
    pub rom: Arc<Mutex<Rom>>,
}

impl Mapper {
    pub fn new(rom: Arc<Mutex<Rom>>) -> Self {
        let b = Mapper {
            mem: Memory::new(),
            rom,
        };

        b
    }
}

impl IO for Mapper {
    fn read(&mut self, addr: u16) -> u8 {
        // println!("addr: {}", addr);
        match addr {
            RAM_START..=RAM_END => self.mem.read(addr - RAM_START),
            CART_START..=CART_END => self.rom.as_ref().lock().unwrap().read(addr),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.mem.write(addr - RAM_START, data),
            CART_START..=CART_END => self
                .rom
                .as_ref()
                .lock()
                .unwrap()
                .write(addr - CART_START, data),
            _ => {}
        };
    }
}
