use std::{rc::Rc, cell::RefCell};

use crate::{serial::Serial, stdin::Stdin, stdout::Stdout, mem::Memory, display::Display, io::IO, cpu::CPU6502};

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x7FFF;
const SERIAL_START: u16 = 0xA000;
const SERIAL_END: u16 = 0xBFFF;
const ROM_START: u16 = 0xC000;
const ROM_END: u16 = 0xFFFF;

// pub struct CpuBus {
//     pub bus: Rc<RefCell<Bus>>,
//     pub cpu: CPU6502,
// }

// impl CpuBus {
//     pub fn new(bus: Bus) -> Self {
//         let bus = Rc::new(RefCell::new(bus));
//         let cpu = CPU6502::new(bus);
//         Self {
//             bus,
//             cpu
//         }
//     }
// }

pub struct Bus {
    pub mem: Memory,
    pub stdout: Stdout,
    pub stdin: Stdin,
    pub display: Display,
    pub serial: Serial,
}

impl Bus {

}

impl IO for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            SERIAL_START..=SERIAL_END => {
                self.serial.read(addr-SERIAL_START)
            }
            _ => {
                self.mem.read(addr)
            }
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            SERIAL_START..=SERIAL_END => {
                self.serial.write(addr-SERIAL_START, data)
            }
            _ => {
                self.mem.write(addr, data)
            }
        }
    }
}
