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
        while !self.cpu.halted() {
            self.cpu.clock();
        }
        println!("Total cycles: {}", self.cpu.cycles())
    }

    pub fn reset(&mut self) {
        self.cpu.reset()
    }

    pub fn load(&self, rom: &[u8]) {
        for (i, b) in rom.iter().enumerate() {
            self.bus.borrow_mut().write(i as u16, *b);
        }
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

    #[test]
    fn test_machine() {
        let mut m = Machine::new();

        // let mul = &[
        // 0xa2, 0x10, 0x69, 0x02, 0xca, 0xd0, 0xfb
        // ];

        // let mul = &[0xA9, 0x05, 0xA2, 0x04, 0xCA, 0x69, 0x05, 0xCA, 0xD0, 0xFB];

        let five_by_4 = &[0xA9, 0x05, 0xA2, 0x04, 0x69, 0x05, 0xCA];

        // m.load(&[
        //     // LDA #$05
        //     0xA9, 0x05,
        //     // ADC #$04
        //     0x69, 0x04,
        //     // SEC
        //     0x38,
        //     // SBC #$07
        //     0xE9, 0x07,
        // ]);

        // ; multiply 5 * 4
        //   lda #$05
        //   ldx #$04
        // loop:
        //   adc #$05
        //   dex

        m.load(five_by_4);

        m.reset();

        m.run();
    }
}
