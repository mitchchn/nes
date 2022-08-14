use std::cell::RefCell;
use std::rc::Rc;

use colored::{ColoredString, Colorize};

use crate::cpu::{Mode, Status, CPU6502};
use crate::mapper::{Mapper, ROM_START};
use crate::{io::IO, rom::Rom};

pub struct Machine {
    pub mapper: Rc<RefCell<Mapper>>,
    pub rom: Rc<RefCell<Rom>>,
    pub cpu: CPU6502,
    pub instruction_log: Vec<String>,
}

impl Machine {
    pub fn new() -> Self {
        let rom = Rc::new(RefCell::new(Rom::new()));

        let mapper = Rc::new(RefCell::new(Mapper::new(rom.clone())));
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

            self.print_state();

            let decoded_instruction = self.decode_instruction();
            self.instruction_log.push(format!(
                "{:04X}  {:02X} {}  {}",
                opcode_addr, opcode_hex, operand, decoded_instruction
            ));
        }
        println!("Total cycles: {}", self.cpu.cycles());
        println!("{:#?}", self.instruction_log);
    }

    pub fn decode_instruction(&mut self) -> String {
        let formatted_operand = match self.cpu.instruction.1 {
            Mode::IMP => "".to_string(),
            Mode::IMM => format!("#${:02X}", self.read(self.cpu.op_addr)),
            Mode::ACC => "A".to_string(),
            Mode::ABS => format!("${:04X}", self.cpu.op_addr),
            Mode::ABX => format!("${:04X},X", self.cpu.op_addr),
            Mode::ABY => format!("${:04X},Y", self.cpu.op_addr),
            Mode::ZPG => format!("${:02X}", self.cpu.op_addr),
            Mode::ZPX => format!("${:02X},X", self.cpu.op_addr),
            Mode::ZPY => format!("${:02X},Y", self.cpu.op_addr),
            Mode::ZIX => format!("(${:02X},X)", self.cpu.op_addr),
            Mode::ZIY => format!("(${:02X},Y)", self.cpu.op_addr),
            Mode::IND => format!("(${:04X})", self.cpu.op_addr),
            Mode::REL => format!("${:04X}", self.cpu.op_addr),
        };
        format!("{:#?} {}", self.cpu.instruction.0, &formatted_operand)
    }

    pub fn print_state(&mut self) {
        let color_flag = |f: u8| {
            if f == 1 {
                f.to_string().green()
            } else {
                ColoredString::from(f.to_string().as_str())
            }
        };

        let f: [u8; 8] = [
            if self.cpu.p.contains(Status::N) { 1 } else { 0 },
            if self.cpu.p.contains(Status::V) { 1 } else { 0 },
            if self.cpu.p.contains(Status::U) { 1 } else { 0 },
            if self.cpu.p.contains(Status::B) { 1 } else { 0 },
            if self.cpu.p.contains(Status::D) { 1 } else { 0 },
            if self.cpu.p.contains(Status::I) { 1 } else { 0 },
            if self.cpu.p.contains(Status::Z) { 1 } else { 0 },
            if self.cpu.p.contains(Status::C) { 1 } else { 0 },
        ];

        println!("{}", self.decode_instruction());

        println!(
            "{}",
            "PC    A  X  Y    SP    N V - B D I Z C".white().on_blue(),
        );
        println!(
            "{:04X}  {:02X} {:02X} {:02X}   {:02X}    {} {} {} {} {} {} {} {}\n",
            self.cpu.pc,
            self.cpu.a,
            self.cpu.x,
            self.cpu.y,
            self.cpu.sp,
            color_flag(f[0]),
            color_flag(f[1]),
            color_flag(f[2]),
            color_flag(f[3]),
            color_flag(f[4]),
            color_flag(f[5]),
            color_flag(f[6]),
            color_flag(f[7])
        );
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
        let lo = (ROM_START & 0x00FF) as u8;
        let hi = ((ROM_START & 0xFF00) >> 8) as u8;
        self.mapper.borrow_mut().write(0xFFFC, lo);
        self.mapper.borrow_mut().write(0xFFFD, hi);

        self.cpu.reset()
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.rom.borrow_mut().load(rom);
    }
}

impl IO for Machine {
    fn read(&mut self, addr: u16) -> u8 {
        self.mapper.borrow_mut().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.mapper.borrow_mut().write(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_machine() {
        let mut m = Machine::new();
        // let rom = fs::read("src/tolower.bin").expect("Could not open file");

        // let rom = [
        //     // LDA #51
        //     0xA9, 0x33, // BRK
        //     0x00,
        // ];

        // let rom = [
        //     0xa9, 0x00, 0xa2, 0x08, 0x4e, 0x34, 0x12, 0x90, 0x04, 0x18, 0x6d, 0xff, 0xff, 0x6a,
        //     0x6e, 0x34, 0x12, 0xca, 0xd0, 0xf3, 0x8d, 0x12, 0x34, 0xad, 0x34, 0x12, 0x60,
        // ];

        let rom = [0xa9, 0x05, 0x8d, 0x34, 0x12, 0xae, 0x34, 0x12, 0xe8, 0x00];
        m.load(&rom);

        m.reset();

        m.run();
    }
}
