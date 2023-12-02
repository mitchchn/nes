use colored::{ColoredString, Colorize};

use crate::bus::Bus;
use crate::io::IO;
use crate::mem::Memory;

use parking_lot::{Mutex};
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;

const DEBUG: bool = false;

bitflags! {
    /// Processor status register
    pub struct Status: u8 {
        /// Negative
        const N = 1 << 7;
        /// Overflow
        const V = 1 << 6;
        /// Unused
        const U = 1 << 5;
        /// Breakpoint
        const B = 1 << 4;
        /// Binary-coded decimal (BCD)
        const D = 1 << 3;
        /// Interrupt
        const I = 1 << 2;
        /// Zero
        const Z = 1 << 1;
        /// Carry
        const C = 1 << 0;
    }
}

/// The stack on the 6502 is a 256-byte array hardcoded to Page 1.
/// It is traversed by the stack pointer (SP).
/// The 6502 uses a descending stack that grows downward.
const STACK: u16 = 0x0100;

/// Each instruction on the 6502 uses one of thirteen
/// memory addressing modes. These determine how the operand (if any) is looked up.
///
/// ### References
///
/// - http://www.obelisk.me.uk/6502/addressing.html
/// - https://emudev.de/nes-emulator/opcodes-and-addressing-modes-the-6502/
/// - https://lowendgaming.neocities.org/6502_addressing_modes.htm
/// - https://slark.me/c64-downloads/6502-addressing-modes.pdf
/// - https://www.pagetable.com/c64ref/6502/?tab=3
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// Implied
    ///
    /// e.g. `BRK`
    IMP,
    /// Accumulator
    ///
    /// e.g. `ASL A`
    ACC,
    /// Immediate
    ///
    /// e.g. `LDA #$AA`
    IMM,
    /// Absolute
    ///
    /// e.g. `LDA $AAAA`
    ABS,
    /// Absolute, X-Indexed
    ///
    /// e.g. `LDA $AAAA,X`
    ABX,
    /// Absolute, Y-Indexed
    ///
    /// e.g. `LDA $AAAA,Y`
    ABY,
    /// Zero Page
    ///
    /// e.g. `LDA $NN`
    ZPG,
    /// Zero Page, X-Indexed
    ///
    /// e.g. `LDA $AA,X`
    ZPX,
    /// Zero Page, Y-Indexed
    ///
    /// e.g. `LDA $AA,Y`
    ZPY,
    /// Zero Page Indirect, X-Indexed
    ///
    /// e.g. `LDA ($AA,X)`
    ZIX,
    /// Zero Page Indirect, Y-Indexed
    ///
    /// e.g. `LDA ($AA,Y)`
    ZIY,
    /// Relative
    ///
    /// e.g. `BEQ $AAAA`adc_dec
    REL,
    /// Absolute Indirect
    ///
    /// e.g. `JMP ($AAAA)`
    IND,
}

#[derive(Debug, Clone, Copy)]
/// All 55 opcodes on the 6502 plus `XXX`, which represenst an illegal opcode.
pub enum Opcode {
    /// `ADC` - Add with Carry
    ADC,
    /// `AND` - Logical AND
    AND,
    /// `ASL` - Arithmetic Shift Left
    ASL,
        /// `ASL (Accumulator)` - Arithmetic Shift Left
        ASL_A,
    /// `BCC` - Branch if Carry Clear
    BCC,
    /// `BCS` - Brancy if Carry Set
    BCS,
    /// `BEQ` - Branch if Equal
    BEQ,
    /// `BIT` - Check Bits
    BIT,
    /// `BMI` - Branch if Minus
    BMI,
    /// `BNE` - Branch if Not Equal
    BNE,
    /// `BPL` - Branch if Plus
    BPL,
    /// `BRK` - Break
    BRK,
    /// `BVC` - Branch if Overflow Clear
    BVC,
    /// `BVS` - Branch if Overflow Set
    BVS,
    /// `CLC` - Clear Carry
    CLC,
    /// `CLD` - Clear Decimal
    CLD,
    /// `CLI` - Clear Interrupt
    CLI,
    /// `CLV` - Clear Overflow
    CLV,
    /// `CMP` - Compare
    CMP,
    /// `CPX` - Compare X
    CPX,
    /// `CPY` - Compare Y
    CPY,
    /// `DEC` - Decrement
    DEC,
    /// `DEX` - Decrement X
    DEX,
    /// `DEY` - Decrement Y
    DEY,
    /// `EOR` - Exclusive OR
    EOR,
    /// `INC` - Increment
    INC,
    /// `INX` - Increment X
    INX,
    /// `INY` - Increment Y
    INY,
    /// `JMP` - Jump
    JMP,
    /// `JSR` - Jump to Subroutine
    JSR,
    /// `LDA` - Load Accumulator
    LDA,
    /// `LDX` - Load X
    LDX,
    /// `LDY` - Load Y
    LDY,
    /// `LSR` - Logical Shift Right
    LSR,
        /// `LSR (Accumulator)` - Logical Shift Right
        LSR_A,

    /// `NOP` - No Operation
    NOP,
    /// `ORA` - Logical OR
    ORA,
    /// `PHA` - Push Accumulator to Stack
    PHA,
    /// `PHP` - Push Processor Status to Stack
    PHP,
    /// `PLA` - Pull Accumulator from Stack
    PLA,
    /// `PHP` - Pull Processor Status from Stack
    PLP,
    /// `ROL` - Rotate Left
    ROL,
    /// `ROL (Accumulator)` - Rotate Left
    ROL_A,
    /// `ROR` - Rotate Right
    ROR,
    /// `ROR (Accumulator)` - Rotate Right
    ROR_A,
    /// `RTI` - Return from Interrupt
    RTI,
    /// `RTS` - Return from Subroutine
    RTS,
    /// `SBC` - Subctract with Carry
    SBC,
    /// `SEC` - Set Carry
    SEC,
    /// `SED` - Set Decimal
    SED,
    /// `SEI` - Set Interrupt
    SEI,
    /// `STA` - Store Accumulator
    STA,
    /// `STX` - Store X
    STX,
    /// `STY` - Store Y
    STY,
    /// `TAX` - Transfer Accumulator to X
    TAX,
    /// `TAY` - Transfer Accumulator to Y
    TAY,
    /// `TSX` - Transfer Stack to X
    TSX,
    /// `TXA` - Transfer X to Accumulator
    TXA,
    /// `TXS` - Transfer X to Stack
    TXS,
    /// `TYA` - Transfer Y to Accumulator
    TYA,
    /// `XXX` - Illegal Opcode
    XXX,
}

pub type Instruction = (Opcode, Mode, u8, bool);

pub const INSTRUCTIONS: [Instruction; 256] = [
    (Opcode::BRK, Mode::IMP, 7, false,),
    (Opcode::ORA, Mode::ZIX, 6, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 3, false,),
    (Opcode::ORA, Mode::ZPG, 3, false,),
    (Opcode::ASL, Mode::ZPG, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::PHP, Mode::IMP, 3, false,),
    (Opcode::ORA, Mode::IMM, 2, false,),
    (Opcode::ASL_A, Mode::ACC, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::ORA, Mode::ABS, 4, false,),
    (Opcode::ASL, Mode::ABS, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::BPL, Mode::REL, 2, false,),
    (Opcode::ORA, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::ORA, Mode::ZPX, 4, false,),
    (Opcode::ASL, Mode::ZPX, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::CLC, Mode::IMP, 2, false,),
    (Opcode::ORA, Mode::ABY, 4, true, ),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::ORA, Mode::ABX, 4, true, ),
    (Opcode::ASL, Mode::ABX, 7, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::JSR, Mode::ABS, 6, false,),
    (Opcode::AND, Mode::ZIX, 6, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::BIT, Mode::ZPG, 3, false,),
    (Opcode::AND, Mode::ZPG, 3, false,),
    (Opcode::ROL, Mode::ZPG, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::PLP, Mode::IMP, 4, false,),
    (Opcode::AND, Mode::IMM, 2, false,),
    (Opcode::ROL_A, Mode::ACC, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::BIT, Mode::ABS, 4, false,),
    (Opcode::AND, Mode::ABS, 4, false,),
    (Opcode::ROL, Mode::ABS, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::BMI, Mode::REL, 2, false,),
    (Opcode::AND, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::AND, Mode::ZPX, 4, false,),
    (Opcode::ROL, Mode::ZPX, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::SEC, Mode::IMP, 2, false,),
    (Opcode::AND, Mode::ABY, 4, true, ),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::AND, Mode::ABX, 4, true, ),
    (Opcode::ROL, Mode::ABX, 7, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::RTI, Mode::IMP, 6, false,),
    (Opcode::EOR, Mode::ZIX, 6, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 3, false,),
    (Opcode::EOR, Mode::ZPG, 3, false,),
    (Opcode::LSR, Mode::ZPG, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::PHA, Mode::IMP, 3, false,),
    (Opcode::EOR, Mode::IMM, 2, false,),
    (Opcode::LSR_A, Mode::ACC, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::JMP, Mode::ABS, 3, false,),
    (Opcode::EOR, Mode::ABS, 4, false,),
    (Opcode::LSR, Mode::ABS, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::BVC, Mode::REL, 2, true, ),
    (Opcode::EOR, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::EOR, Mode::ZPX, 4, false,),
    (Opcode::LSR, Mode::ZPX, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::CLI, Mode::IMP, 2, false,),
    (Opcode::EOR, Mode::ABY, 4, true, ),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::EOR, Mode::ABX, 4, true, ),
    (Opcode::LSR, Mode::ABX, 7, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::RTS, Mode::IMP, 6, false,),
    (Opcode::ADC, Mode::ZIX, 6, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 3, false,),
    (Opcode::ADC, Mode::ZPG, 3, false,),
    (Opcode::ROR, Mode::ZPG, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::PLA, Mode::IMP, 4, false,),
    (Opcode::ADC, Mode::IMM, 2, false,),
    (Opcode::ROR_A, Mode::ACC, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::JMP, Mode::IND, 5, false,),
    (Opcode::ADC, Mode::ABS, 4, false,),
    (Opcode::ROR, Mode::ABS, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::BVS, Mode::REL, 2, true, ),
    (Opcode::ADC, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::ADC, Mode::ZPX, 4, false,),
    (Opcode::ROR, Mode::ZPX, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::SEI, Mode::IMP, 2, false,),
    (Opcode::ADC, Mode::ABY, 4, true, ),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::ADC, Mode::ABX, 4, true, ),
    (Opcode::ROR, Mode::ABX, 7, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::STA, Mode::ZIX, 6, false,),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::STY, Mode::ZPG, 3, false,),
    (Opcode::STA, Mode::ZPG, 3, false,),
    (Opcode::STX, Mode::ZPG, 3, false,),
    (Opcode::XXX, Mode::IMP, 3, false,),
    (Opcode::DEY, Mode::IMP, 2, false,),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::TXA, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::STY, Mode::ABS, 4, false,),
    (Opcode::STA, Mode::ABS, 4, false,),
    (Opcode::STX, Mode::ABS, 4, false,),
    (Opcode::XXX, Mode::IMP, 4, false,),
    (Opcode::BCC, Mode::REL, 2, true, ),
    (Opcode::STA, Mode::ZIY, 6, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::STY, Mode::ZPX, 4, false,),
    (Opcode::STA, Mode::ZPX, 4, false,),
    (Opcode::STX, Mode::ZPY, 4, false,),
    (Opcode::XXX, Mode::IMP, 4, false,),
    (Opcode::TYA, Mode::IMP, 2, false,),
    (Opcode::STA, Mode::ABY, 5, false,),
    (Opcode::TXS, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::NOP, Mode::IMP, 5, false,),
    (Opcode::STA, Mode::ABX, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::LDY, Mode::IMM, 2, false,),
    (Opcode::LDA, Mode::ZIX, 6, false,),
    (Opcode::LDX, Mode::IMM, 2, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::LDY, Mode::ZPG, 3, false,),
    (Opcode::LDA, Mode::ZPG, 3, false,),
    (Opcode::LDX, Mode::ZPG, 3, false,),
    (Opcode::XXX, Mode::IMP, 3, false,),
    (Opcode::TAY, Mode::IMP, 2, false,),
    (Opcode::LDA, Mode::IMM, 2, false,),
    (Opcode::TAX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::LDY, Mode::ABS, 4, false,),
    (Opcode::LDA, Mode::ABS, 4, false,),
    (Opcode::LDX, Mode::ABS, 4, false,),
    (Opcode::XXX, Mode::IMP, 4, false,),
    (Opcode::BCS, Mode::REL, 2, true, ),
    (Opcode::LDA, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::LDY, Mode::ZPX, 4, false,),
    (Opcode::LDA, Mode::ZPX, 4, false,),
    (Opcode::LDX, Mode::ZPY, 4, false,),
    (Opcode::XXX, Mode::IMP, 4, false,),
    (Opcode::CLV, Mode::IMP, 2, false,),
    (Opcode::LDA, Mode::ABY, 4, true, ),
    (Opcode::TSX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 4, false,),
    (Opcode::LDY, Mode::ABX, 4, true, ),
    (Opcode::LDA, Mode::ABX, 4, true, ),
    (Opcode::LDX, Mode::ABY, 4, true, ),
    (Opcode::XXX, Mode::IMP, 4, false,),
    (Opcode::CPY, Mode::IMM, 2, false,),
    (Opcode::CMP, Mode::ZIX, 6, false,),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::CPY, Mode::ZPG, 3, false,),
    (Opcode::CMP, Mode::ZPG, 3, false,),
    (Opcode::DEC, Mode::ZPG, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::INY, Mode::IMP, 2, false,),
    (Opcode::CMP, Mode::IMM, 2, false,),
    (Opcode::DEX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::CPY, Mode::ABS, 4, false,),
    (Opcode::CMP, Mode::ABS, 4, false,),
    (Opcode::DEC, Mode::ABS, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::BNE, Mode::REL, 2, true, ),
    (Opcode::CMP, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::CMP, Mode::ZPX, 4, false,),
    (Opcode::DEC, Mode::ZPX, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::CLD, Mode::IMP, 2, false,),
    (Opcode::CMP, Mode::ABY, 4, true, ),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::CMP, Mode::ABX, 4, true, ),
    (Opcode::DEC, Mode::ABX, 7, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::CPX, Mode::IMM, 2, false,),
    (Opcode::SBC, Mode::ZIX, 6, false,),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::CPX, Mode::ZPG, 3, false,),
    (Opcode::SBC, Mode::ZPG, 3, false,),
    (Opcode::INC, Mode::ZPG, 5, false,),
    (Opcode::XXX, Mode::IMP, 5, false,),
    (Opcode::INX, Mode::IMP, 2, false,),
    (Opcode::SBC, Mode::IMM, 2, false,),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::SBC, Mode::IMP, 2, false,),
    (Opcode::CPX, Mode::ABS, 4, false,),
    (Opcode::SBC, Mode::ABS, 4, false,),
    (Opcode::INC, Mode::ABS, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::BEQ, Mode::REL, 2, true, ),
    (Opcode::SBC, Mode::ZIY, 5, true, ),
    (Opcode::XXX, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 8, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::SBC, Mode::ZPX, 4, false,),
    (Opcode::INC, Mode::ZPX, 6, false,),
    (Opcode::XXX, Mode::IMP, 6, false,),
    (Opcode::SED, Mode::IMP, 2, false,),
    (Opcode::SBC, Mode::ABY, 4, true, ),
    (Opcode::NOP, Mode::IMP, 2, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
    (Opcode::NOP, Mode::IMP, 4, false,),
    (Opcode::SBC, Mode::ABX, 4, true, ),
    (Opcode::INC, Mode::ABX, 7, false,),
    (Opcode::XXX, Mode::IMP, 7, false,),
];

pub struct CPU6502<T: IO> {
    pub mem: T,
    /// Program counter
    pub pc: u16,
    /// Accmulator
    pub a: u8,
    /// X index
    pub x: u8,
    /// Y index
    pub y: u8,
    /// Stack pointer
    pub sp: u8,
    /// Processor status
    pub p: Status,

    // Total cycle count
    pub cycles: u64,

    // Total number of instructions executed
    pub instructions: usize,

    // Current instruction
    pub instruction: Option<(u16, Instruction)>,
    pub op_addr: u16,
    pub cycles_left: u8,
}

impl <T: IO> CPU6502<T> {
    pub fn new(mem: T) -> Self {
        let cpu = CPU6502 {
            mem,
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            p: Status::empty(),
            cycles: 0,
            instruction: None,
            op_addr: 0,
            cycles_left: 0,
            instructions: 0,
        };

        // cpu.instruction_table = Some(
        //     {
        //         let mut table: [Instr<T>; 256] = [
        //             (Self::and, Self::abs); 256                ];
        //         table
        //     },
        // );

        cpu
    }

    /// Reset the CPU to an initial good state.
    pub fn reset(&mut self) {
        // Get the starting program counter address.
        // This is stored at a predetermined location, 0xFFFC.
        let pc_lo = self.read(0xFFFC) as u16;
        let pc_hi = self.read(0xFFFD) as u16;
        let pc = (pc_hi << 8) | pc_lo;

        // Stack poiner counts *down* so start at 0XFF (255).
        let sp = 0xFF;

        // Switch off status Status except for U (Unused) which is always on.
        let status = Status::empty() | Status::U;

        self.pc = pc;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = sp;
        self.p = status;

        self.instruction = None;
        self.op_addr = 0;
        self.cycles_left = 0;
    }

    pub fn execute(&mut self, (opcode, mode, cycles, can_cross_page_boundary): Instruction) {
        let crossed_page_boundary = match mode {
            Mode::ABS => self.abs(),
            Mode::ABX => self.abx(),
            Mode::ABY => self.aby(),
            Mode::IMM => self.imm(),
            Mode::ZPX => self.zpx(),
            Mode::ZPG => self.zpg(),
            Mode::ZPY => self.zpy(),
            Mode::IND => self.ind(),
            Mode::REL => self.rel(),
            Mode::ZIX => self.zix(),
            Mode::ZIY => self.ziy(),
            Mode::ACC => self.acc(),
            Mode::IMP => self.imp(),            
        };


        if crossed_page_boundary && can_cross_page_boundary {
            self.cycles_left += 1;
        }

        match opcode {
            Opcode::ADC => self.adc(),
            Opcode::AND => self.and(),
            Opcode::ASL => self.asl(),
            Opcode::ASL_A => self.asl_a(),
            Opcode::BCC => self.bcc(),
            Opcode::BCS => self.bcs(),
            Opcode::BEQ => self.beq(),
            Opcode::BIT => self.bit(),
            Opcode::BMI => self.bmi(),
            Opcode::BNE => self.bne(),
            Opcode::BPL => self.bpl(),
            Opcode::BRK => self.brk(),
            Opcode::BVC => self.bvc(),
            Opcode::BVS => self.bvs(),
            Opcode::CLC => self.clc(),
            Opcode::CLD => self.cld(),
            Opcode::CLI => self.cli(),
            Opcode::CLV => self.clv(),
            Opcode::CMP => self.cmp(),
            Opcode::CPX => self.cpx(),
            Opcode::CPY => self.cpy(),
            Opcode::DEC => self.dec(),
            Opcode::DEX => self.dex(),
            Opcode::DEY => self.dey(),
            Opcode::EOR => self.eor(),
            Opcode::INC => self.inc(),
            Opcode::INX => self.inx(),
            Opcode::INY => self.iny(),
            Opcode::JMP => self.jmp(),
            Opcode::JSR => self.jsr(),
            Opcode::LDA => self.lda(),
            Opcode::LDX => self.ldx(),
            Opcode::LDY => self.ldy(),
            Opcode::LSR => self.lsr(),
            Opcode::LSR_A => self.lsr_a(),
            Opcode::NOP => self.nop(),
            Opcode::ORA => self.ora(),
            Opcode::PHA => self.pha(),
            Opcode::PHP => self.php(),
            Opcode::PLA => self.pla(),
            Opcode::PLP => self.plp(),
            Opcode::ROL => self.rol(),
            Opcode::ROL_A => self.rol_a(),
            Opcode::ROR => self.ror(),
            Opcode::ROR_A => self.ror_a(),
            Opcode::RTI => self.rti(),
            Opcode::RTS => self.rts(),
            Opcode::SBC => self.sbc(),
            Opcode::SEC => self.sec(),
            Opcode::SED => self.sed(),
            Opcode::SEI => self.sei(),
            Opcode::STA => self.sta(),
            Opcode::STX => self.stx(),
            Opcode::STY => self.sty(),
            Opcode::TAX => self.tax(),
            Opcode::TAY => self.tay(),
            Opcode::TSX => self.tsx(),
            Opcode::TXA => self.txa(),
            Opcode::TXS => self.txs(),
            Opcode::TYA => self.tya(),
            Opcode::XXX => self.xxx(),
        };
    }

    pub fn clock(&mut self) {
        self.cycles += 1;

        if self.cycles_left  > 0 {
            self.cycles_left -= 1;
            return;
        }

        let inst_addr = self.pc;
        let opcode = self.pop_u8();

        let instruction = INSTRUCTIONS[opcode as usize];

        self.instruction = Some((inst_addr, instruction));
        self.instructions += 1;
        self.cycles_left = instruction.2 - 1;

        self.execute(instruction);

        // self.cycles_left = 0;

        if (DEBUG) {
            self.print_state();
        }
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn halted(&self) -> bool {
        self.p.contains(Status::B)
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
            if self.p.contains(Status::N) { 1 } else { 0 },
            if self.p.contains(Status::V) { 1 } else { 0 },
            if self.p.contains(Status::U) { 1 } else { 0 },
            if self.p.contains(Status::B) { 1 } else { 0 },
            if self.p.contains(Status::D) { 1 } else { 0 },
            if self.p.contains(Status::I) { 1 } else { 0 },
            if self.p.contains(Status::Z) { 1 } else { 0 },
            if self.p.contains(Status::C) { 1 } else { 0 },
        ];

        println!("{}", self.decode_instruction());

        println!(
            "{}",
            "PC    A  X  Y    SP    N V - B D I Z C".white().on_blue(),
        );
        println!(
            "{:04X}  {:02X} {:02X} {:02X}   {:02X}    {} {} {} {} {} {} {} {}\n",
            self.pc,
            self.a,
            self.x,
            self.y,
            self.sp,
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

    pub fn decode_instruction(&mut self) -> String {
        if let Some(instruction) = self.instruction {
            let formatted_operand = match instruction.1 .1 {
                Mode::IMP => "".to_string(),
                Mode::IMM => format!("#${:02X}", self.read(self.op_addr)),
                Mode::ACC => "A".to_string(),
                Mode::ABS => format!("${:04X}", self.op_addr),
                Mode::ABX => format!("${:04X},X", self.op_addr),
                Mode::ABY => format!("${:04X},Y", self.op_addr),
                Mode::ZPG => format!("${:02X}", self.op_addr),
                Mode::ZPX => format!("${:02X},X", self.op_addr),
                Mode::ZPY => format!("${:02X},Y", self.op_addr),
                Mode::ZIX => format!("(${:02X},X)", self.op_addr),
                Mode::ZIY => format!("(${:02X},Y)", self.op_addr),
                Mode::IND => format!("(${:04X})", self.op_addr),
                Mode::REL => format!("${:04X}", self.op_addr),
            };
            format!("{:#?} {}", instruction.1, &formatted_operand)
        } else {
            "".to_string()
        }
    }

    // Addresing Modes
    //
    //

    /// Implied
    #[inline]
    fn imp(&mut self) -> bool {
        false
    }

    // Accumulator
    #[inline]
    fn acc(&mut self) -> bool {
        false
    }

    /// Absolute
    #[inline]
    fn abs(&mut self) -> bool {
        self.op_addr = self.pop_u16();
        false
    }

    // Immediate
    #[inline]
    fn imm(&mut self) -> bool {
        let addr = self.pc;
        self.pc += 1;
        self.op_addr = addr;
        false
    }

    /// Absolute Indirect
    #[inline]
    fn ind(&mut self) -> bool {
        let addr_ptr = self.pop_u16();

        let lo = self.read(addr_ptr) as u16;
        let hi = self.read(addr_ptr + 1) as u16;
        let addr = (hi << 8) | lo;

        self.op_addr = addr;
        false
    }

    /// Zero Page
    #[inline]
    fn zpg(&mut self) -> bool {
        let lo = self.pop_u8();
        let addr = 0x0000 | (lo as u16);

        self.op_addr = addr;
        false
    }

    /// Zero Page, X-Indexed
    #[inline]
    fn zpx(&mut self) -> bool {
        let lo = self.pop_u8();
        // No carry:
        // Even though the final value is 16 bits,
        // wrap around if the X offset + lo bit > 0xFF.
        let lo_idx = lo.wrapping_add(self.x);
        let addr = 0x0000 | (lo_idx as u16);

        self.op_addr = addr;
        false
    }

    /// Zero Page, Y-Indexed
    #[inline]
    fn zpy(&mut self) -> bool {
        let lo = self.pop_u8();
        // No carry:
        // Even though the final value is 16 bits,
        // wrap around if the Y offset + lo bit > 0xFF.
        let lo_idx = lo.wrapping_add(self.y);
        let addr = 0x0000 | (lo_idx as u16);

        self.op_addr = addr;
        false
    }

    /// Absolute, X-Indexed
    fn abx(&mut self) -> bool {
        self.abs();
        let abs_addr = self.op_addr;

        // Carry offset
        let addr = abs_addr + self.x as u16;

        self.op_addr = addr;
        self.crossed_page_boundary(abs_addr, addr)
    }

    /// Absolute, Y-Indexed
    #[inline]
    fn aby(&mut self) -> bool {
        self.abs();
        let abs_addr = self.op_addr;

        // Carry offset
        let addr = abs_addr + self.y as u16;

        self.op_addr = addr;
        self.crossed_page_boundary(abs_addr, addr)
    }

    /// Relative
    #[inline]
    fn rel(&mut self) -> bool {
        // offset is a 1-byte signed value:
        //
        // 0x00 - 0xFD is positive (0 - 127)
        // 0x80 - 0xFF is negative (-128 to -1)
        let offset = self.pop_u8();

        // Same as offset > 0x7f
        let addr = if offset & 0x80 == 0x80 {
            // Any 16-bit value + 0xffff wraps around to equal itself.
            // if the offset is, e.g., 0x90 (-112 in two's complement)
            // we add 0xff90 which wraps around to (pc - 112).
            self.pc.wrapping_add(0xff00 | (offset as u16))
        } else {
            self.pc.wrapping_add(offset as u16)
        };

        self.op_addr = addr;
        
        // Branch functions will add a cycle if the page boundary was crossed
        false
    }

    /// Zero Page Indirect, X-Indexed
    ///
    /// Operand is zero page address.
    /// Absolute address is word in (OP + X, OP + X + 1).
    /// No carry.
    #[inline]
    fn zix(&mut self) -> bool {
        let ptr_lo = self.pop_u8();
        let ptr_lo_idx = ptr_lo.wrapping_add(self.x);
        let ptr = 0x0000 | (ptr_lo_idx as u16);

        let lo = self.read(ptr) as u16;
        let hi = self.read(ptr + 1) as u16;
        let addr = (hi << 8) | lo;

        self.op_addr = addr;
        false
    }

    /// Zero Page Indirect, Y-Indexed
    ///
    /// Operand is zero page address.
    /// Absolute address is word in (OP, OP + 1) offset by Y.
    #[inline]
    fn ziy(&mut self) -> bool {
        self.zpg();
        let ptr = self.op_addr;

        let lo = self.read(ptr) as u16;
        let hi = self.read(ptr + 1) as u16;
        let abs_addr = (hi << 8) | lo;
        let addr= abs_addr + self.y as u16;
        self.op_addr = addr;

        self.crossed_page_boundary(abs_addr, addr)
    }

    //
    //
    // Operations
    //
    //

    /// XXX - Illegal Instruction
    ///
    fn xxx(&mut self) {
        dbg!("XXX - Illegal Instruction: ({})", self.instruction);
    }

    /// ADC - Add with Carry
    ///
    fn adc(&mut self) {
        let acc = self.a;
        let op = self.read(self.op_addr);

        if !self.p.contains(Status::D) {
            self.add_a_(acc, op);
        } else {
            self.add_dec_(acc, op);
        }
    }

    /// AND - Logical And
    ///
    fn and(&mut self) {
        let byte = self.read(self.op_addr);
        self.a &= byte;
        self.set_arithmetic_status(self.a);
    }

    /// ASL - Arithmetic Shift Left
    ///
    fn asl(&mut self) {
        let byte = self.read(self.op_addr);

        let asl_value = self.asl_(byte);
        self.write(self.op_addr, asl_value);

        self.set_arithmetic_status(asl_value);
    }

    fn asl_a(&mut self) {
        let acc = self.a;

        let asl_value = self.asl_(acc);
        self.a = asl_value;

        self.set_arithmetic_status(asl_value);
    }

    #[inline]
    fn asl_(&mut self, value: u8) -> u8 {
        // Left shifting will implicitly set bit 0 to 0
        let asl_value = value << 1;

        // Place old bit 7 in the carry flag
        // self.status.set(Status::C, asl_value & 0xf0 != value & 0xf0);
        let seven_bit = value & (1 << 7);
        self.p.set(Status::C, seven_bit != 0);

        asl_value
    }

    /// Exclusive OR
    /// A^M -> A,N,Z
    fn eor(&mut self) {
        let m = self.read(self.op_addr);
        self.a = self.a ^ m;

        self.set_arithmetic_status(self.a);
    }

    /// LSR - Logical Shift Right
    ///
    fn lsr(&mut self) {
        let byte = self.read(self.op_addr);

        let lsr_value = self.lsr_(byte);
        self.write(self.op_addr, lsr_value);

        self.set_arithmetic_status(lsr_value);
    }

    fn lsr_a(&mut self) {
        let acc = self.a;

        let lsr_value = self.lsr_(acc);
        self.a = lsr_value;

        self.set_arithmetic_status(lsr_value);
    }

    fn lsr_(&mut self, value: u8) -> u8 {
        let zero_bit = value & (1 << 0);
        self.p.set(Status::C, zero_bit != 0);
        value >> 1
    }

    /// ROL - Rotate Left
    ///
    fn rol(&mut self) {
        let byte = self.read(self.op_addr);

        let rol_value = self.rol_(byte);
        self.write(self.op_addr, rol_value);

        self.set_arithmetic_status(rol_value);
    }

    fn rol_a(&mut self) {
        let acc = self.a;

        let rol_value = self.rol_(acc);
        self.a = rol_value;

        self.set_arithmetic_status(self.a);
    }

    fn rol_(&mut self, value: u8) -> u8 {
        let carry_bit = (self.p & Status::C).bits();

        // Shift left and change bit 0 to value of old carry bit.
        let mut rol_value = value << 1;
        if carry_bit > 0 {
            rol_value |= 1 << 0;
        } else {
            rol_value &= !(1 << 0);
        }

        // Set carry flag to old bit 7
        let seven_bit = value & (1 << 7);
        self.p.set(Status::C, seven_bit != 0);

        self.p.set(Status::C, seven_bit != 0);
        rol_value
    }

    /// ROR - Rotate Right
    ///
    fn ror(&mut self) {
        let byte = self.read(self.op_addr);

        let ror_value = self.ror_(byte);
        self.write(self.op_addr, ror_value);

        self.set_arithmetic_status(ror_value);
    }

    fn ror_a(&mut self) {
        let acc = self.a;

        let ror_value = self.ror_(acc);
        self.a = ror_value;

        self.set_arithmetic_status(self.a);
    }

    fn ror_(&mut self, value: u8) -> u8 {
        let carry_bit = (self.p & Status::C).bits();

        // Shift right and set bit 7 to contents of old carry bit.
        let mut ror_value = value >> 1;
        if carry_bit > 0 {
            ror_value |= 1 << 7;
        } else {
            ror_value &= !(1 << 7);
        }

        // Set carry flag to old bit 0
        let zero_bit = value & (1 << 0);
        self.p.set(Status::C, zero_bit != 0);

        ror_value
    }

    /// SBC - Subtract with Carry
    ///
    fn sbc(&mut self) {
        let acc = self.a;

        if !self.p.contains(Status::D) {
            // One's complement
            // Don't add 1 since we're adding the carry bit.
            let op = self.read(self.op_addr) ^ 0xFF;
            self.add_a_(acc, op);
        } else {
            // Nine's complement
            let mut op = self.read(self.op_addr);
            let op_lo = 9 - (op & 0xf);
            let op_hi = 9 - (op >> 4);
            op = (op_hi << 4) | op_lo;

            self.add_dec_(acc, op);
        }
    }

    #[inline]
    fn add_a_(&mut self, a: u8, m: u8) {
        let c = (self.p & Status::C).bits();
        let sum: u16 = (a as u16) + (m as u16) + (c as u16);

        self.a = sum as u8;

        // Set carry flag if the sum exceeds 255, otherwise unset it
        // (sum >> 8) == 1 is equivalent to sum > 0xFF
        self.p.set(Status::C, (sum >> 8) == 1);

        // Set Overflow flag
        //
        // Indicate overflow to negate the N flag when
        // adding two values with the same sign (P + P or N + N).
        self.p.set(
            Status::V,
            self.a & 0x80 != a & 0x80 && self.a & 0x80 != m & 0x80,
        );

        self.set_arithmetic_status(self.a);
    }

    #[inline]
    fn add_dec_(&mut self, a: u8, m: u8) {
        // BCD stores two digits (0-9) in a byte
        // Sum hi and lo digits separately, then combine

        // If the sum of the lo-bit digits (plus carry) exceeds 9, add 0x6 to skip the base-16 values.
        // Carry the 1 to the hi-bit digit
        let c = (self.p & Status::C).bits();
        let mut lo_carry = 0;
        let mut lo_sum: u8 = (a & 0xF) + (m & 0xF) + c;
        if lo_sum > 9 {
            lo_sum += 0x6;
            lo_carry = 0x1;
        }

        // If the sum of the hi-bit digits (plus lo-bit carry) exceeds 9, wrap around.
        let mut hi_sum: u8 = (a >> 4) + (m >> 4) + lo_carry;
        if hi_sum > 9 {
            hi_sum -= 10;
            self.p.set(Status::C, true);
        } else {
            self.p.set(Status::C, false);
        }

        let sum: u8 = (hi_sum << 4) | (lo_sum & 0xF);
        self.a = sum;

        // Set Overflow flag
        //
        // Indicate overflow to negate the N flag when
        // adding two values with the same sign (P + P or N + N).
        self.p.set(
            Status::V,
            self.a & 0x80 != a & 0x80 && self.a & 0x80 != m & 0x80,
        );

        self.set_arithmetic_status(self.a);
    }

    /// BCC - Branch if Carry Clear
    fn bcc(&mut self) {
        if !self.p.contains(Status::C) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    /// BCS - Branch if Carry Set
    fn bcs(&mut self) {
        if self.p.contains(Status::C) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    /// BEQ - Branch if Equal
    fn beq(&mut self) {
        if self.p.contains(Status::Z) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    /// BMI - Branch on Result Minus
    fn bmi(&mut self) {
        if self.p.contains(Status::N) {
            self.cycles_left += 1;
            self.branch_()
        }
    }

    /// BNE - Branch Not Equal
    fn bne(&mut self) {
        if !self.p.contains(Status::Z) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    /// BPL - Branch if Positive
    fn bpl(&mut self) {
        if !self.p.contains(Status::N) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    /// BVC - Branch if Overflow Clear
    fn bvc(&mut self) {
        if !self.p.contains(Status::V) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    /// BVS - Branch if Overflow Set
    fn bvs(&mut self) {
        if self.p.contains(Status::V) {
            self.cycles_left += 1;
            self.branch_();
        }
    }

    #[inline]
    fn branch_(&mut self) {
        // Add another cycle if page boundary was crossed.
        if self.crossed_page_boundary(self.pc+1, self.op_addr) {
            self.cycles_left += 1;
        }

        self.pc = self.op_addr;
    }

    /// CLC - Clear Carry
    fn clc(&mut self) {
        self.p.set(Status::C, false);
    }

    /// CLD - Clear Decimal
    fn cld(&mut self) {
        self.p.set(Status::D, false);
    }

    /// CLI - Clear Interrupt Disable
    fn cli(&mut self) {
        self.p.set(Status::I, false);
    }

    /// CLV - Clear Overflow
    fn clv(&mut self) {
        self.p.set(Status::V, false);
    }

    /// DEC - Decrement
    ///
    fn dec(&mut self) {
        let val = self.read(self.op_addr).wrapping_sub(1);

        self.write(self.op_addr, val);
        self.set_arithmetic_status(val);
    }

    /// DEX - Decrement X
    ///
    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_arithmetic_status(self.x);
    }

    /// DEY - Decrement Y
    ///
    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_arithmetic_status(self.y);
    }

    /// INC - Increment
    /// M+1 -> M,N,Z
    fn inc(&mut self) {
        let m = self.read(self.op_addr);
        let result = m.wrapping_add(1);
        self.write(self.op_addr, result);
        self.set_arithmetic_status(result);
    }

    /// INX - Increment X
    ///
    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_arithmetic_status(self.x);
    }

    /// INY - Increment Y
    ///
    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_arithmetic_status(self.y);
    }

    // BIT - Test bits
    //
    fn bit(&mut self) {
        let byte = self.read(self.op_addr);
        self.p.set(Status::Z, (self.a & byte) == 0);

        self.p.set(Status::V, byte >> 6 & 1 != 0);
        self.p.set(Status::N, byte >> 7 & 1 != 0);

        // self.status.set(Status::N,)
    }

    /// CMP - Compare Accumulator
    /// A-M -> Z,C,N
    fn cmp(&mut self) {
        self.cmp_(self.a);
    }

    /// CPX - Compare X
    /// X-M -> Z,C,N
    fn cpx(&mut self) {
        self.cmp_(self.x);
    }

    /// CPY - Compare Y
    /// Y-M -> Z,C,N
    fn cpy(&mut self) {
        self.cmp_(self.y);
    }

    #[inline]
    fn cmp_(&mut self, value: u8) {
        let m = self.read(self.op_addr);

        self.p.set(Status::Z, value == m);
        self.p.set(Status::C, value >= m);
        self.p.set(Status::N, value.wrapping_sub(m) & (1 << 7) != 0);
    }

    /// JMP - Jump
    ///
    /// http://www.obelisk.me.uk/6502/reference.html#JMP
    fn jmp(&mut self) {
        self.pc = self.op_addr;
    }

    /// JSR - Jump to Subroutine
    ///
    /// http://www.obelisk.me.uk/6502/reference.html#JSR
    fn jsr(&mut self) {
        let ret_addr = self.pc - 1;

        let ret_addr_hi = (ret_addr >> 8) as u8;
        let ret_addr_lo = ret_addr as u8;
        self.push_stack(ret_addr_hi);
        self.push_stack(ret_addr_lo);

        self.pc = self.op_addr;
    }

    /// RTI - Return from Interrupt
    ///
    fn rti(&mut self) {
        let status = self.pop_stack();
        self.p =
            Status::from_bits(status).expect("Could not restore status") & !Status::B | Status::U;

        let pc_lo = self.pop_stack() as u16;
        let pc_hi = self.pop_stack() as u16;

        let pc = (pc_hi << 8) | pc_lo;
        self.pc = pc;
    }

    /// RTS - Return from Subroutine
    ///
    /// http://www.obelisk.me.uk/6502/reference.html#RTS
    fn rts(&mut self) {
        let pc_lo = self.pop_stack() as u16;
        let pc_hi = self.pop_stack() as u16;

        let pc = (pc_hi << 8) | pc_lo;
        self.pc = pc + 1;
    }

    /// LDA - Load Accumulator With Memory
    ///
    /// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php?title=LDA
    fn lda(&mut self) {
        self.a = self.read(self.op_addr);
        self.set_arithmetic_status(self.a);
    }

    /// LDX - Load X With Memory
    ///
    /// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php?title=LDA
    fn ldx(&mut self) {
        self.x = self.read(self.op_addr);
        self.set_arithmetic_status(self.x);
    }

    /// LDY - Load Y With Memory
    ///
    /// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php?title=LDA
    fn ldy(&mut self) {
        self.y = self.read(self.op_addr);
        self.set_arithmetic_status(self.y);
    }

    /// NOP - No Operation
    fn nop(&mut self) {}

    /// ORA - OR Memory With Accumulator
    /// A|M -> A
    ///
    fn ora(&mut self) {
        self.a |= self.read(self.op_addr);
        self.set_arithmetic_status(self.a);
    }

    /// PHA - Push Accumulator to Stack
    ///
    fn pha(&mut self) {
        self.push_stack(self.a);
    }

    /// PHP - Push Processor Status
    ///
    fn php(&mut self) {
        // Bits 4 and 5 are set to 1 when pushed to the stack
        let php_bits = self.p | Status::B | Status::U;
        self.push_stack(php_bits.bits());
    }

    /// PLA - Pull Accumulator from Stack
    ///
    fn pla(&mut self) {
        self.a = self.pop_stack();
        self.set_arithmetic_status(self.a);
    }

    /// PLP - Pull Processor Status
    ///
    fn plp(&mut self) {
        self.p = Status::from_bits(self.pop_stack()).expect("Could not restore status register")
            & !(Status::B)
            | Status::U;

        // self.p = Status::from_bits(self.pop_stack()).expect("Could not restore status register")
        //     | (Status::B | Status::U);
    }

    /// SEC - Set Carry
    /// 1 -> C
    ///
    pub fn sec(&mut self) {
        self.p.set(Status::C, true);
    }

    /// SED - Set Decimal
    /// 1 -> D
    ///
    pub fn sed(&mut self) {
        self.p.set(Status::D, true);
    }

    /// SEI - Set Interrupt Disable
    /// 1 -> I
    ///
    pub fn sei(&mut self) {
        self.p.set(Status::I, true);
    }

    /// STA - Store Accumulator
    ///
    /// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php?title=STA
    fn sta(&mut self) {
        self.write(self.op_addr, self.a);
    }

    /// STX - Store X
    /// X -> M
    fn stx(&mut self) {
        self.write(self.op_addr, self.x);
    }

    /// STY - Store Y
    /// Y -> M
    fn sty(&mut self) {
        self.write(self.op_addr, self.y);
    }

    /// TAX - Transfer Accumulator to X
    ///
    fn tax(&mut self) {
        self.x = self.a;
        self.set_arithmetic_status(self.x);
    }

    /// TAY - Transfer Accumulator to Y
    ///
    fn tay(&mut self) {
        self.y = self.a;
        self.set_arithmetic_status(self.y);
    }

    /// TSX - Transfer Stack Pointer to X
    /// SP -> X
    fn tsx(&mut self) {
        self.x = self.sp;
        self.set_arithmetic_status(self.x);
    }

    /// TXA - Transfer X to Accumulator
    /// X -> A
    fn txa(&mut self) {
        self.a = self.x;
        self.set_arithmetic_status(self.a);
    }

    /// TXS - Transfer X to Stack Pointer
    /// X -> SP
    fn txs(&mut self) {
        self.sp = self.x;
    }

    /// TXA - Transfer Y to Accumulator
    /// Y -> A
    fn tya(&mut self) {
        self.a = self.y;
        self.set_arithmetic_status(self.a);
    }

    /// BRK - Break
    ///
    fn brk(&mut self) {
        self.p.set(Status::B, true);
        self.pc += 1;

        self.interrupt_(0xFFFE);
    }

    //
    // End of operations
    //

    // Interrupts

    /// NMI - Non-Maskable Interrupt
    // fn nmi(&mut self) {
    //     self.interrupt_(0xFFFA);
    //     self.cycle_count = 8;
    // }

    /// IRQ - Interrupt
    fn irq(&mut self) {
        if !self.p.contains(Status::I) {
            self.interrupt_(0xFFFE);
            self.cycles_left = 7;
        }
    }

    fn interrupt_(&mut self, vector_addr: u16) {
        // Push PC onto the stack

        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = self.pc as u8;

        self.push_stack(pc_hi);
        self.push_stack(pc_lo);
        self.push_stack((self.p).bits());

        // Set PC to address from vector
        let addr_lo = self.read(vector_addr) as u16;
        let addr_hi = self.read(vector_addr + 1) as u16;
        let addr = (addr_hi << 8) | addr_lo;

        // Set I flag
        self.p.set(Status::I, true);
        self.pc = addr;
    }

    #[inline]
    fn crossed_page_boundary(&self, addr1: u16, addr2: u16) -> bool {
        addr1 & 0xFF00 != addr2 & 0xFF00
    }

    #[inline]
    fn set_arithmetic_status(&mut self, val: u8) {
        // Negative flag
        // 0x00 - 0x7F is positive
        // 0x80 -0xFF is negative
        self.p.set(Status::N, (val & 0x80) != 0);

        // Zero flag
        self.p.set(Status::Z, val == 0);
    }

    /// Read a word (e.g. an address) from the bus.
    ///
    /// Remember: 6502 is little-endian.
    /// Read lo byte followed by hi byte.
    /// Left shift the hi bit to the front of a 16-bit value,
    /// then OR it with the lo bit.
    fn pop_u16(&mut self) -> u16 {
        let lo = self.pop_u8() as u16;
        let hi = self.pop_u8() as u16;
        let addr = (hi << 8) | lo;
        addr
    }

    fn pop_u8(&mut self) -> u8 {
        let addr = self.read(self.pc);
        self.pc += 1;

        addr
    }

    fn push_stack(&mut self, byte: u8) {
        let stkp = STACK + (self.sp as u16);
        self.write(stkp, byte);

        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);

        let stkp = STACK + (self.sp as u16);
        let byte = self.read(stkp);

        byte
    }
}

impl <T: IO> IO for CPU6502<T> {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem.read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.mem.write(addr, data)
    }
}

