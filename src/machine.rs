use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Mutex, Arc};

use crate::cpu::{Mode, CPU6502};
use crate::mapper::Mapper;
use crate::{io::IO, rom::Rom};

pub struct Machine {
    pub mapper: Arc<Mutex<Mapper>>,
    pub rom: Arc<Mutex<Rom>>,
    pub cpu: CPU6502,
    pub instruction_log: Vec<String>,
}

impl Machine {
    pub fn new() -> Self {
        let rom = Arc::new(Mutex::new(Rom::new()));

        let mapper = Arc::new(Mutex::new(Mapper::new(rom.clone())));
        let cpu = CPU6502::new(mapper.clone());

        let m = Machine {
            mapper,
            cpu,
            rom,
            instruction_log: vec![],
        };
        m
    }

    pub fn run(&mut self) {
        while !self.cpu.halted() {
            let opcode_addr = self.cpu.pc;
            let opcode_hex = self.read(self.cpu.pc);

            self.cpu.clock();

            if self.cpu.cycles_left != self.cpu.instruction.2 - 1 {
                continue;
            }

            let operand = match self.cpu.instruction.1 {
                Mode::IMP | Mode::ACC => "     ".to_string(),
                Mode::IMM => format!("{:02X}   ", self.read(self.cpu.op_addr)),
                Mode::ABS | Mode::ABX | Mode::ABY => {
                    format!(
                        "{:02X} {:02X}",
                        self.cpu.op_addr as u8,
                        (self.cpu.op_addr) >> 8 as u8
                    )
                }
                Mode::REL => format!("{:02X}   ", self.read(opcode_addr + 1)),
                _ => format!("{:02X}   ", self.cpu.op_addr),
            };

            // self.cpu.print_state();

            let decoded_instruction = self.cpu.decode_instruction();
            self.instruction_log.push(format!(
                "{:04X}  {:02X} {}  {}",
                opcode_addr, opcode_hex, operand, decoded_instruction
            ));
        }
        println!("Total cycles: {}", self.cpu.cycles());
        println!("{:#?}", self.instruction_log);
    }

    pub fn debug(&mut self) {
        let mut stdin = std::io::stdin();

        self.cpu.reset();
        loop {
            if self.read(self.cpu.pc) != 0 {
                self.cpu.clock();
                continue;
            }

            println!("continue (g) >");
            let mut input = String::new();
            let _ = stdin.read_line(&mut input);
            match input.to_ascii_lowercase().trim() {
                "g" => {
                    self.cpu.pc += 1;
                    continue;
                }
                _ => {
                    break;
                }
            };
        }
    }

    pub fn reset(&mut self) {
        // Init reset vector
        // let lo = (ROM_START & 0x00FF) as u8;
        // let hi = ((ROM_START & 0xFF00) >> 8) as u8;
        // self.mapper.borrow_mut().write(0xFFFC, lo);
        // self.mapper.borrow_mut().write(0xFFFD, hi);

        self.cpu.reset()
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.rom.lock().unwrap().load(rom);
    }
}

impl IO for Machine {
    fn read(&mut self, addr: u16) -> u8 {
        self.mapper.lock().unwrap().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.mapper.lock().unwrap().write(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_machine() {
        let mut m = Machine::new();
        // let rom = fs::read("src/nestest.nes").expect("Could not open file");

        // let rom = [
        //     // LDA #51
        //     0xA9, 0x33, // BRK
        //     0x00,
        // ];

        // let rom = [
        //     0xa9, 0x00, 0xa2, 0x08, 0x4e, 0x34, 0x12, 0x90, 0x04, 0x18, 0x6d, 0xff, 0xff, 0x6a,
        //     0x6e, 0x34, 0x12, 0xca, 0xd0, 0xf3, 0x8d, 0x12, 0x34, 0xad, 0x34, 0x12, 0x60,
        // ];

        // let header: [u8; 0x10] = [0; 0x10];
        // let data: [u8; 10] = [0xa9, 0x05, 0x8d, 0x00, 0x05, 0xae, 0x00, 0x05, 0xe8, 0x00];

        // let rom: Vec<u8> = header
        //     .into_iter()
        //     .chain(data.into_iter())
        //     .cloned()
        //     .collect();


        let rom = [
            // LDA #51
            0xA9, 0x33,
            // TAX
            0xAA,
            // DEXq
            0xCA,
            // TAY
            0xA8,
            // INY
            0xC8,
            // BRK
            0x00,
        ];


        m.load(&rom);

        m.reset();
        m.run();
    }
}
