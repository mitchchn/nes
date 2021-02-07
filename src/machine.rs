use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::Bus;
use crate::cpu::CPU6502;
use crate::io::IO;

pub struct Machine {
    pub bus: Rc<RefCell<Bus>>,
    pub cpu: CPU6502,
}

impl Machine {
    pub fn new() -> Self {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let cpu = CPU6502::new(bus.clone());
        let m = Machine { bus, cpu };
        m
    }

    pub fn run(&mut self) {
        self.cpu.reset();

        while !self.cpu.halted() {
            self.cpu.clock();
        }
        println!("Total cycles: {}", self.cpu.cycles())
    }

    pub fn debug(&mut self, breakpoints: &[u16]) {
        self.cpu.reset();
        while !self.cpu.halted() {
            self.cpu.clock();
            if breakpoints.contains(&self.cpu.pc) {
                break;
            }
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset()
    }

    pub fn load(&self, rom: &[u8], start_at: u16) {
        for (i, b) in rom.iter().enumerate() {
            self.bus.borrow_mut().write(i as u16, *b);
        }

        // Init reset vector
        let lo = (start_at & 0x00FF) as u8;
        let hi = ((start_at & 0xFF00) >> 8) as u8;
        self.bus.borrow_mut().write(0xFFFC, lo);
        self.bus.borrow_mut().write(0xFFFD, hi);
    }
}

impl IO for Machine {
    fn read(&self, addr: u16) -> u8 {
        self.bus.borrow().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.bus.borrow_mut().write(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_machine() {
        let mut m = Machine::new();
        let rom = fs::read("src/asm/main.bin").expect("Could not open file");

        m.load(&rom, 0x0600);

        m.reset();

        m.run();
        m.bus.borrow_mut().flush_display();
    }
}
