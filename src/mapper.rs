use std::{cell::RefCell, rc::Rc};

use crate::mem::Memory;
use crate::{io::IO, rom::Rom};

pub const RAM_START: u16 = 0x0600;
pub const RAM_END: u16 = 0x7FFF;
pub const ROM_START: u16 = 0x8000;
pub const ROM_END: u16 = 0xFFFB;

/// 6502 Memory Mapper
///
/// ### Memory Layout
///
///
/// | Address range  | Use       |
/// |----------------|-----------|
/// | $0000 - $00FF  | Zero Page |
/// | $0100 - $01FF  | Stack     |
/// | $0200 - $05FF  | I/O       |
/// | $0600 - $7FFF  | RAM       |
/// | $8000 - $FFFF  | ROM       |
/// | $FFFC - $FFFF  | BRK/reset |
///
///
pub struct Mapper {
    pub mem: Memory,
    pub rom: Rc<RefCell<Rom>>,
}

impl Mapper {
    pub fn new(rom: Rc<RefCell<Rom>>) -> Self {
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
            ROM_START..=ROM_END => self.rom.as_ref().borrow_mut().read(addr - ROM_START),
            _ => self.mem.read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mem.write(addr, data);
    }
}
