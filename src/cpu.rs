use colored::{ColoredString, Colorize};

use crate::io::IO;

use std::cell::RefCell;
use std::rc::Rc;

const DEBUG: bool = true;

bitflags! {
    /// Processor status register
    pub(crate) struct Status: u8 {
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
    /// e.g. `BEQ $AAAA`
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
    /// `ROR` - Rotate Right
    ROR,
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

pub type Instruction = (Opcode, Mode, u8, fn(&mut CPU6502));

const INSTRUCTIONS: [Instruction; 256] = [
    (Opcode::BRK, Mode::IMP, 7, CPU6502::brk),
    (Opcode::ORA, Mode::ZIX, 6, CPU6502::ora),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 3, CPU6502::nop),
    (Opcode::ORA, Mode::ZPG, 3, CPU6502::ora),
    (Opcode::ASL, Mode::ZPG, 5, CPU6502::asl),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::PHP, Mode::IMP, 3, CPU6502::php),
    (Opcode::ORA, Mode::IMM, 2, CPU6502::ora),
    (Opcode::ASL, Mode::ACC, 2, CPU6502::asl_a),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::ORA, Mode::ABS, 4, CPU6502::ora),
    (Opcode::ASL, Mode::ABS, 6, CPU6502::asl),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::BPL, Mode::REL, 2, CPU6502::bpl),
    (Opcode::ORA, Mode::ZIY, 5, CPU6502::ora),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::ORA, Mode::ZPX, 4, CPU6502::ora),
    (Opcode::ASL, Mode::ZPX, 6, CPU6502::asl),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::CLC, Mode::IMP, 2, CPU6502::clc),
    (Opcode::ORA, Mode::ABY, 4, CPU6502::ora),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::ORA, Mode::ABX, 4, CPU6502::ora),
    (Opcode::ASL, Mode::ABX, 7, CPU6502::asl),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::JSR, Mode::ABS, 6, CPU6502::jsr),
    (Opcode::AND, Mode::ZIX, 6, CPU6502::and),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::BIT, Mode::ZPG, 3, CPU6502::bit),
    (Opcode::AND, Mode::ZPG, 3, CPU6502::and),
    (Opcode::ROL, Mode::ZPG, 5, CPU6502::rol),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::PLP, Mode::IMP, 4, CPU6502::plp),
    (Opcode::AND, Mode::IMM, 2, CPU6502::and),
    (Opcode::ROL, Mode::ACC, 2, CPU6502::rol_a),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::BIT, Mode::ABS, 4, CPU6502::bit),
    (Opcode::AND, Mode::ABS, 4, CPU6502::and),
    (Opcode::ROL, Mode::ABS, 6, CPU6502::rol),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::BMI, Mode::REL, 2, CPU6502::bmi),
    (Opcode::AND, Mode::ZIY, 5, CPU6502::and),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::AND, Mode::ZPX, 4, CPU6502::and),
    (Opcode::ROL, Mode::ZPX, 6, CPU6502::rol),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::SEC, Mode::IMP, 2, CPU6502::sec),
    (Opcode::AND, Mode::ABY, 4, CPU6502::and),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::AND, Mode::ABX, 4, CPU6502::and),
    (Opcode::ROL, Mode::ABX, 7, CPU6502::rol),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::RTI, Mode::IMP, 6, CPU6502::rti),
    (Opcode::EOR, Mode::ZIX, 6, CPU6502::eor),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 3, CPU6502::nop),
    (Opcode::EOR, Mode::ZPG, 3, CPU6502::eor),
    (Opcode::LSR, Mode::ZPG, 5, CPU6502::lsr),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::PHA, Mode::IMP, 3, CPU6502::pha),
    (Opcode::EOR, Mode::IMM, 2, CPU6502::eor),
    (Opcode::LSR, Mode::ACC, 2, CPU6502::lsr_a),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::JMP, Mode::ABS, 3, CPU6502::jmp),
    (Opcode::EOR, Mode::ABS, 4, CPU6502::eor),
    (Opcode::LSR, Mode::ABS, 6, CPU6502::lsr),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::BVC, Mode::REL, 2, CPU6502::bvc),
    (Opcode::EOR, Mode::ZIY, 5, CPU6502::eor),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::EOR, Mode::ZPX, 4, CPU6502::eor),
    (Opcode::LSR, Mode::ZPX, 6, CPU6502::lsr),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::CLI, Mode::IMP, 2, CPU6502::cli),
    (Opcode::EOR, Mode::ABY, 4, CPU6502::eor),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::EOR, Mode::ABX, 4, CPU6502::eor),
    (Opcode::LSR, Mode::ABX, 7, CPU6502::lsr),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::RTS, Mode::IMP, 6, CPU6502::rts),
    (Opcode::ADC, Mode::ZIX, 6, CPU6502::adc),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 3, CPU6502::nop),
    (Opcode::ADC, Mode::ZPG, 3, CPU6502::adc),
    (Opcode::ROR, Mode::ZPG, 5, CPU6502::ror),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::PLA, Mode::IMP, 4, CPU6502::pla),
    (Opcode::ADC, Mode::IMM, 2, CPU6502::adc),
    (Opcode::ROR, Mode::ACC, 2, CPU6502::ror_a),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::JMP, Mode::IND, 5, CPU6502::jmp),
    (Opcode::ADC, Mode::ABS, 4, CPU6502::adc),
    (Opcode::ROR, Mode::ABS, 6, CPU6502::ror),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::BVS, Mode::REL, 2, CPU6502::bvs),
    (Opcode::ADC, Mode::ZIY, 5, CPU6502::adc),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::ADC, Mode::ZPX, 4, CPU6502::adc),
    (Opcode::ROR, Mode::ZPX, 6, CPU6502::ror),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::SEI, Mode::IMP, 2, CPU6502::sei),
    (Opcode::ADC, Mode::ABY, 4, CPU6502::adc),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::ADC, Mode::ABX, 4, CPU6502::adc),
    (Opcode::ROR, Mode::ABX, 7, CPU6502::ror),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::STA, Mode::ZIX, 6, CPU6502::sta),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::STY, Mode::ZPG, 3, CPU6502::sty),
    (Opcode::STA, Mode::ZPG, 3, CPU6502::sta),
    (Opcode::STX, Mode::ZPG, 3, CPU6502::stx),
    (Opcode::XXX, Mode::IMP, 3, CPU6502::xxx),
    (Opcode::DEY, Mode::IMP, 2, CPU6502::dey),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::TXA, Mode::IMP, 2, CPU6502::txa),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::STY, Mode::ABS, 4, CPU6502::sty),
    (Opcode::STA, Mode::ABS, 4, CPU6502::sta),
    (Opcode::STX, Mode::ABS, 4, CPU6502::stx),
    (Opcode::XXX, Mode::IMP, 4, CPU6502::xxx),
    (Opcode::BCC, Mode::REL, 2, CPU6502::bcc),
    (Opcode::STA, Mode::ZIY, 6, CPU6502::sta),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::STY, Mode::ZPX, 4, CPU6502::sty),
    (Opcode::STA, Mode::ZPX, 4, CPU6502::sta),
    (Opcode::STX, Mode::ZPY, 4, CPU6502::stx),
    (Opcode::XXX, Mode::IMP, 4, CPU6502::xxx),
    (Opcode::TYA, Mode::IMP, 2, CPU6502::tya),
    (Opcode::STA, Mode::ABY, 5, CPU6502::sta),
    (Opcode::TXS, Mode::IMP, 2, CPU6502::txs),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 5, CPU6502::nop),
    (Opcode::STA, Mode::ABX, 5, CPU6502::sta),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::LDY, Mode::IMM, 2, CPU6502::ldy),
    (Opcode::LDA, Mode::ZIX, 6, CPU6502::lda),
    (Opcode::LDX, Mode::IMM, 2, CPU6502::ldx),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::LDY, Mode::ZPG, 3, CPU6502::ldy),
    (Opcode::LDA, Mode::ZPG, 3, CPU6502::lda),
    (Opcode::LDX, Mode::ZPG, 3, CPU6502::ldx),
    (Opcode::XXX, Mode::IMP, 3, CPU6502::xxx),
    (Opcode::TAY, Mode::IMP, 2, CPU6502::tay),
    (Opcode::LDA, Mode::IMM, 2, CPU6502::lda),
    (Opcode::TAX, Mode::IMP, 2, CPU6502::tax),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::LDY, Mode::ABS, 4, CPU6502::ldy),
    (Opcode::LDA, Mode::ABS, 4, CPU6502::lda),
    (Opcode::LDX, Mode::ABS, 4, CPU6502::ldx),
    (Opcode::XXX, Mode::IMP, 4, CPU6502::xxx),
    (Opcode::BCS, Mode::REL, 2, CPU6502::bcs),
    (Opcode::LDA, Mode::ZIY, 5, CPU6502::lda),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::LDY, Mode::ZPX, 4, CPU6502::ldy),
    (Opcode::LDA, Mode::ZPX, 4, CPU6502::lda),
    (Opcode::LDX, Mode::ZPY, 4, CPU6502::ldx),
    (Opcode::XXX, Mode::IMP, 4, CPU6502::xxx),
    (Opcode::CLV, Mode::IMP, 2, CPU6502::clv),
    (Opcode::LDA, Mode::ABY, 4, CPU6502::lda),
    (Opcode::TSX, Mode::IMP, 2, CPU6502::tsx),
    (Opcode::XXX, Mode::IMP, 4, CPU6502::xxx),
    (Opcode::LDY, Mode::ABX, 4, CPU6502::ldy),
    (Opcode::LDA, Mode::ABX, 4, CPU6502::lda),
    (Opcode::LDX, Mode::ABY, 4, CPU6502::ldx),
    (Opcode::XXX, Mode::IMP, 4, CPU6502::xxx),
    (Opcode::CPY, Mode::IMM, 2, CPU6502::cpy),
    (Opcode::CMP, Mode::ZIX, 6, CPU6502::cmp),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::CPY, Mode::ZPG, 3, CPU6502::cpy),
    (Opcode::CMP, Mode::ZPG, 3, CPU6502::cmp),
    (Opcode::DEC, Mode::ZPG, 5, CPU6502::dec),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::INY, Mode::IMP, 2, CPU6502::iny),
    (Opcode::CMP, Mode::IMM, 2, CPU6502::cmp),
    (Opcode::DEX, Mode::IMP, 2, CPU6502::dex),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::CPY, Mode::ABS, 4, CPU6502::cpy),
    (Opcode::CMP, Mode::ABS, 4, CPU6502::cmp),
    (Opcode::DEC, Mode::ABS, 6, CPU6502::dec),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::BNE, Mode::REL, 2, CPU6502::bne),
    (Opcode::CMP, Mode::ZIY, 5, CPU6502::cmp),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::CMP, Mode::ZPX, 4, CPU6502::cmp),
    (Opcode::DEC, Mode::ZPX, 6, CPU6502::dec),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::CLD, Mode::IMP, 2, CPU6502::cld),
    (Opcode::CMP, Mode::ABY, 4, CPU6502::cmp),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::CMP, Mode::ABX, 4, CPU6502::cmp),
    (Opcode::DEC, Mode::ABX, 7, CPU6502::dec),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::CPX, Mode::IMM, 2, CPU6502::cpx),
    (Opcode::SBC, Mode::ZIX, 6, CPU6502::sbc),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::CPX, Mode::ZPG, 3, CPU6502::cpx),
    (Opcode::SBC, Mode::ZPG, 3, CPU6502::sbc),
    (Opcode::INC, Mode::ZPG, 5, CPU6502::inc),
    (Opcode::XXX, Mode::IMP, 5, CPU6502::xxx),
    (Opcode::INX, Mode::IMP, 2, CPU6502::inx),
    (Opcode::SBC, Mode::IMM, 2, CPU6502::sbc),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::SBC, Mode::IMP, 2, CPU6502::sbc),
    (Opcode::CPX, Mode::ABS, 4, CPU6502::cpx),
    (Opcode::SBC, Mode::ABS, 4, CPU6502::sbc),
    (Opcode::INC, Mode::ABS, 6, CPU6502::inc),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::BEQ, Mode::REL, 2, CPU6502::beq),
    (Opcode::SBC, Mode::ZIY, 5, CPU6502::sbc),
    (Opcode::XXX, Mode::IMP, 2, CPU6502::xxx),
    (Opcode::XXX, Mode::IMP, 8, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::SBC, Mode::ZPX, 4, CPU6502::sbc),
    (Opcode::INC, Mode::ZPX, 6, CPU6502::inc),
    (Opcode::XXX, Mode::IMP, 6, CPU6502::xxx),
    (Opcode::SED, Mode::IMP, 2, CPU6502::sed),
    (Opcode::SBC, Mode::ABY, 4, CPU6502::sbc),
    (Opcode::NOP, Mode::IMP, 2, CPU6502::nop),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
    (Opcode::NOP, Mode::IMP, 4, CPU6502::nop),
    (Opcode::SBC, Mode::ABX, 4, CPU6502::sbc),
    (Opcode::INC, Mode::ABX, 7, CPU6502::inc),
    (Opcode::XXX, Mode::IMP, 7, CPU6502::xxx),
];

pub struct CPU6502 {
    mem: Rc<RefCell<dyn IO>>,

    /// Program counter
    pub(crate) pc: u16,
    /// Accmulator
    pub(crate) a: u8,
    /// X index
    pub(crate) x: u8,
    /// Y index
    pub(crate) y: u8,
    /// Stack pointer
    pub(crate) sp: u8,
    /// Processor status
    pub(crate) p: Status,

    // Total cycle count
    pub(crate) cycles: u64,

    // Current instruction
    pub(crate) instruction: Instruction,
    pub(crate) op_addr: u16,
    pub(crate) cycles_left: u8,
}

impl CPU6502 {
    pub fn new(mem: Rc<RefCell<dyn IO>>) -> Self {
        let cpu = CPU6502 {
            mem,
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: 0,
            p: Status::empty(),
            cycles: 0,
            instruction: INSTRUCTIONS[0],
            op_addr: 0,
            cycles_left: 0,
        };

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

        self.instruction = INSTRUCTIONS[0];
        self.op_addr = 0;
        self.cycles_left = 0;
    }

    pub fn execute(&mut self, (opcode, mode, cycles, op): Instruction) {
        self.instruction = (opcode, mode, cycles, op);

        match mode {
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
            Mode::ACC | Mode::IMP => {}
        };

        self.cycles_left = cycles;

        op(self);
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.pop_u8();

            let instruction = INSTRUCTIONS[opcode as usize];

            self.cycles_left = instruction.2;
            self.execute(instruction);
        }

        self.cycles += 1;
        self.cycles_left -= 1;

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
        let formatted_operand = match self.instruction.1 {
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
        format!("{:#?} {}", self.instruction.0, &formatted_operand)
    }

    // Addresing Modes
    //
    //

    /// Absolute
    fn abs(&mut self) {
        self.op_addr = self.pop_u16();
    }

    // Immediate
    fn imm(&mut self) {
        let addr = self.pc;
        self.pc += 1;
        self.op_addr = addr;
    }

    /// Absolute Indirect
    fn ind(&mut self) {
        let addr_ptr = self.pop_u16();

        let lo = self.read(addr_ptr) as u16;
        let hi = self.read(addr_ptr + 1) as u16;
        let addr = (hi << 8) | lo;

        self.op_addr = addr;
    }

    /// Zero Page
    fn zpg(&mut self) {
        let lo = self.pop_u8();
        let addr = 0x0000 | (lo as u16);

        self.op_addr = addr;
    }

    /// Zero Page, X-Indexed
    fn zpx(&mut self) {
        let lo = self.pop_u8();
        // No carry:
        // Even though the final value is 16 bits,
        // wrap around if the X offset + lo bit > 0xFF.
        let lo_idx = lo.wrapping_add(self.x);
        let addr = 0x0000 | (lo_idx as u16);

        self.op_addr = addr;
    }

    /// Zero Page, Y-Indexed
    fn zpy(&mut self) {
        let lo = self.pop_u8();
        // No carry:
        // Even though the final value is 16 bits,
        // wrap around if the Y offset + lo bit > 0xFF.
        let lo_idx = lo.wrapping_add(self.y);
        let addr = 0x0000 | (lo_idx as u16);

        self.op_addr = addr;
    }

    /// Absolute, X-Indexed
    fn abx(&mut self) {
        self.abs();
        let abs_addr = self.op_addr;
        // Carry offset
        let addr = abs_addr + self.x as u16;
        if self.crossed_page_boundary(addr) {
            self.cycles_left += 1;
        }

        self.op_addr = addr;
    }

    /// Absolute, Y-Indexed
    fn aby(&mut self) {
        self.abs();
        let abs_addr = self.op_addr;

        // Carry offset
        let addr = abs_addr + self.y as u16;
        if self.crossed_page_boundary(addr) {
            self.cycles_left += 1;
        }

        self.op_addr = addr;
    }

    /// Relative
    fn rel(&mut self) {
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
    }

    /// Zero Page Indirect, X-Indexed
    ///
    /// Operand is zero page address.
    /// Absolute address is word in (OP + X, OP + X + 1).
    /// No carry.
    fn zix(&mut self) {
        let ptr_lo = self.pop_u8();
        let ptr_lo_idx = ptr_lo.wrapping_add(self.x);
        let ptr = 0x0000 | (ptr_lo_idx as u16);

        let lo = self.read(ptr) as u16;
        let hi = self.read(ptr + 1) as u16;
        let addr = (hi << 8) | lo;

        self.op_addr = addr;
    }

    /// Zero Page Indirect, Y-Indexed
    ///
    /// Operand is zero page address.
    /// Absolute address is word in (OP, OP + 1) offset by Y.
    fn ziy(&mut self) {
        self.zpg();
        let ptr = self.op_addr;

        let lo = self.read(ptr) as u16;
        let hi = self.read(ptr + 1) as u16;
        let mut addr = (hi << 8) | lo;
        if self.crossed_page_boundary(addr) {
            self.cycles_left += 1;
        }

        addr += self.y as u16;

        self.op_addr = addr;
    }

    //
    //
    // Operations
    //
    //

    /// XXX - Illegal Instruction
    ///
    fn xxx(&mut self) {
        dbg!(
            "XXX - Illegal Instruction: ({}, {}, {})",
            self.instruction.0,
            self.instruction.1,
            self.instruction.2
        );
    }

    /// ADC - Add with Carry
    ///
    fn adc(&mut self) {
        let acc = self.a;
        let op = self.read(self.op_addr);

        self.add_a_(acc, op);
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

        self.set_arithmetic_status(byte);
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

    /// BRK - Break
    ///
    fn brk(&mut self) {
        self.p.set(Status::B, true);
        self.irq();
    }

    /// SBC - Subtract with Carry
    ///
    fn sbc(&mut self) {
        let acc = self.a;
        // One's complement
        // Don't add 1 since we're adding the carry bit.
        let op = self.read(self.op_addr) ^ 0xFF;

        self.add_a_(acc, op);
    }

    fn add_a_(&mut self, acc: u8, val: u8) {
        let carry_bit = (self.p & Status::C).bits();
        let result = acc.wrapping_add(val).wrapping_add(carry_bit);
        self.a = result;

        // Set Carry flag
        //
        // Carry if MSB flipped.
        // This could _either_ indicate a change of sign or an overflow.
        self.p.set(Status::C, result < acc);

        // Set Overflow flag
        //
        // Indicate overflow to negate the N flag when
        // adding two values with the same sign (P + P or N + N).
        self.p.set(
            Status::V,
            result & 0x80 != acc & 0x80 && result & 0x80 != val & 0x80,
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
        if self.crossed_page_boundary(self.op_addr) {
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
        let result = m + 1;
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
        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = (self.pc) as u8;

        self.push_stack(pc_hi);
        self.push_stack(pc_lo);

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
        self.pc = pc;
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
        self.push_stack(self.p.bits());
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
        self.set_arithmetic_status(self.sp);
    }

    /// TXA - Transfer Y to Accumulator
    /// Y -> A
    fn tya(&mut self) {
        self.a = self.y;
        self.set_arithmetic_status(self.a);
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

        let pc_lo = (0x00FF & self.pc) as u8;
        let pc_hi = ((0xFF00 & self.pc) >> 8) as u8;

        self.push_stack(pc_hi);
        self.push_stack(pc_lo);

        // Push status register onto the stack (with clear B flag)
        self.push_stack((self.p & !Status::B).bits());

        // Set PC to address from vector
        let addr_lo = self.read(vector_addr) as u16;
        let addr_hi = self.read(vector_addr + 1) as u16;
        let addr = (addr_hi << 8) | addr_lo;

        // Set I flag
        self.p.set(Status::I, true);

        self.pc = addr;
    }

    #[inline]
    fn crossed_page_boundary(&self, addr: u16) -> bool {
        addr & 0xFF00 != self.pc & 0xFF00
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
        let stkp = STACK + (self.sp as u16);
        let byte = self.read(stkp + 1);

        self.sp = self.sp.wrapping_add(1);

        byte
    }
}

impl IO for CPU6502 {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem.borrow_mut().read(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.mem.borrow_mut().write(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mem::Memory;

    fn create_test_cpu(mem: &[u8]) -> CPU6502 {
        let mut m = Memory::new();
        m.load(mem, 0);
        CPU6502::new(Rc::new(RefCell::new(m)))
    }

    #[test]
    fn test_reset() {
        let mut c = create_test_cpu(&[]);
        c.reset();

        // Set initial program counter
        c.write(0xFFFC, 0x34);
        c.write(0xFFFC + 1, 0x12);

        assert_eq!(c.mem.borrow_mut().read(0xFFFC), 0x34);

        c.reset();

        assert_eq!(c.p, Status::U);
        assert_eq!(c.pc, 0x1234);
    }

    // #[test]
    // fn test_imm() {
    //     let mut cpu = create_cpu();
    //     cpu.write(0x0000, 0x05);
    //     cpu.imm();
    //     assert_eq!(cpu.read(cpu.op_addr), 0x05);
    // }

    #[test]
    fn test_jmp_ind() {
        let mut c = create_test_cpu(&[
            // JMP ($0120)
            0x6C, 0x20, 0x01,
        ]);
        c.reset();

        // Address to jump to
        c.write(0x0120, 0xFC);
        c.write(0x0121, 0xBA);

        c.write(0x0120, 0xFC);

        c.clock();

        assert_eq!(c.pc, 0xBAFC);
    }

    #[test]
    fn test_jmp_abs() {
        let mut c = create_test_cpu(&[
            // JMP $0120
            0x4C, 0x20, 0x01,
        ]);
        c.reset();

        c.clock();

        assert_eq!(c.pc, 0x0120);
    }

    #[test]
    fn test_lda_imm() {
        let mut c = create_test_cpu(&[
            // LDA #51
            0xA9, 0x33,
        ]);
        c.reset();

        c.clock();

        assert_eq!(c.a, 0x33);
    }

    #[test]
    fn test_lda_abs() {
        let mut c = create_test_cpu(&[
            // LDA $80FC
            0xAD, 0xFC, 0x80,
        ]);
        c.reset();

        // Target value
        c.write(0x80FC, 0x2B);

        c.clock();

        assert_eq!(c.a, 0x2B);
    }

    #[test]
    fn test_sta_abs() {
        let mut c = create_test_cpu(&[
            // STA $0xAB
            0x8D, 0xAB,
        ]);
        c.reset();
        c.a = 0x33;

        c.clock();

        assert_eq!(c.mem.borrow_mut().read(0xAB), 0x33);
    }

    #[test]
    fn test_rel_pos() {
        let mut c = create_test_cpu(&[]);
        c.reset();

        c.write(0x2000, 0x05);
        c.pc = 0x2000;
        c.rel();
        assert_eq!(c.op_addr, 0x2006);
    }

    #[test]
    fn test_rel_neg() {
        let mut c = create_test_cpu(&[]);
        c.reset();

        c.write(0x2000, 0x85);
        c.pc = 0x2000;
        c.rel();
        assert_eq!(c.op_addr, 0x1F86);
    }

    #[test]
    fn test_zix() {
        let mut c = create_test_cpu(&[]);
        c.reset();
        c.x = 0x04;
        c.write(0x0000, 0x20);
        c.write(0x0024, 0x74);
        c.write(0x0025, 0x20);
        c.zix();
        assert_eq!(c.op_addr, 0x2074);
    }

    #[test]
    fn test_ziy() {
        let mut c = create_test_cpu(&[]);
        c.reset();
        c.y = 0x04;
        c.write(0x0000, 0x20);
        c.write(0x0020, 0x74);
        c.write(0x0021, 0x20);
        c.ziy();
        assert_eq!(c.op_addr, 0x2078);
    }

    #[test]
    fn test_zpx() {
        let mut c = create_test_cpu(&[]);
        // Test typical
        c.reset();
        c.x = 0x0F;
        c.write(0x0000, 0x80);
        c.zpx();
        assert_eq!(c.op_addr, 0x008F);

        // Test w/wrap-around in lo bit
        c.reset();
        c.x = 0xFF;
        c.write(0x0000, 0x80);
        c.zpx();
        assert_eq!(c.op_addr, 0x007F);
    }

    #[test]
    fn test_asl() {
        let mut c = create_test_cpu(&[
            // ASL A
            0x0A,
        ]);

        c.reset();

        c.a = 2;
        c.clock();
        assert_eq!(c.a, 4);
        // cpu.a = 2;
        // cpu.write(0x0, 0x0A);
        // cpu.clock();
        // assert_eq!(cpu.a, 4);

        // With carry
        c.reset();
        c.a = 0x90;
        c.clock();
        assert_eq!(c.a, 0x20);
        assert_eq!(c.p & Status::C, Status::C);
    }

    #[test]
    fn test_dex() {
        let mut c = create_test_cpu(&[
            // DEX
            0xCA,
        ]);

        // From positive to positive (5 -> 4)
        c.reset();
        c.x = 0x05;
        c.clock();
        assert_eq!(c.x, 0x04);
        // From positive to zero (1 -> 0)
        c.reset();
        c.x = 0x01;
        c.clock();
        assert_eq!(c.x, 0x00);
        assert_eq!(c.p & Status::Z, Status::Z);

        // From positive to negative (0 -> -1)
        c.reset();
        c.x = 0x00;
        c.clock();
        assert_eq!(c.x, 0xFF);
        assert_eq!(c.p & Status::N, Status::N);
    }

    #[test]
    fn test_and() {
        let mut c = create_test_cpu(&[
            // AND #$0x74
            0x29, 0x74,
        ]);

        c.reset();
        c.a = 0x58;
        c.clock();
        assert_eq!(c.a, 0x50);
    }

    #[test]
    fn test_beq() {
        let mut c = create_test_cpu(&[
            // BEQ ($0x10)
            0xF0, 0x10,
        ]);

        // Take branch
        c.reset();
        c.p.set(Status::Z, true);
        c.clock();
        assert_eq!(c.pc, 0x12);
        assert_eq!(c.cycles_left, 2);

        // Don't take branch
        c.reset();
        c.clock();
        assert_eq!(c.pc, 0x02);
        assert_eq!(c.cycles_left, 1);

        // Pass page boundary
        c.write(0x00F5, 0xF0);
        c.write(0x00F6, 0x40);
        c.reset();
        c.pc = 0x00F5;
        c.p.set(Status::Z, true);
        c.clock();
        assert_eq!(c.pc, 0x0137);
        // assert_eq!(c.pc, 0x11);
        assert_eq!(c.cycles_left, 3);
    }

    #[test]
    fn test_bmi() {
        let mut c = create_test_cpu(&[
            // BMI $0x10
            0x30, 0x10,
        ]);
        c.reset();
        c.p.set(Status::N, true);
        c.clock();
        assert_eq!(c.pc, 0x12);

        c.reset();
        c.p.set(Status::N, false);
        c.clock();
        assert_eq!(c.pc, 0x02);
    }

    #[test]
    fn test_adc() {
        let mut c = create_test_cpu(&[
            // ADC #$05
            0x69, 0x05,
        ]);

        // P + P = P
        // No carry
        // 9 + 5 = 14
        c.reset();
        c.a = 0x09;
        c.clock();
        assert_eq!(c.a, 0x0E);
        assert_eq!(c.p, Status::U);

        let mut c = create_test_cpu(&[
            // ADC #$05
            0x69, 0x05,
        ]);

        // P + P = P (overflow)
        // No carry
        // 127 + 5 = 132
        c.reset();
        c.a = 0x7F;
        c.clock();
        assert_eq!(c.a, 0x84);
        assert_eq!(c.p, Status::U | Status::V | Status::N);

        // P + N = P
        // Carry
        // 127 - 16 = 111

        let mut c = create_test_cpu(&[
            // ADC #$05
            // ADC #$F0 % add -16
            0x69, 0xF0,
        ]);

        c.reset();

        c.a = 0x7F;
        c.clock();
        assert_eq!(c.a, 0x6F);
        assert_eq!(c.p, Status::U | Status::C);

        // P + N = N
        // No carry
        // 16 - 32 = -16
        c.reset();

        let mut c = create_test_cpu(&[
            // ADC #$E0 % add -32
            0x69, 0xE0,
        ]);
        c.reset();

        c.a = 0x10;
        c.clock();
        assert_eq!(c.a, 0xF0);
        assert_eq!(c.p, Status::N | Status::U);

        // N + N = N
        // Carry

        let mut c = create_test_cpu(&[
            // ADC #$FF % add -1
            0x69, 0xFF,
        ]);

        c.reset();

        c.a = 0x90;
        c.clock();
        assert_eq!(c.a, 0x8F);
        assert_eq!(c.p, Status::N | Status::U | Status::C);

        // N + N = N (overflow)
        // Carry
        let mut c = create_test_cpu(&[
            // ADC #$A0
            0x69, 0xA0,
        ]);

        c.reset();
        c.a = 0x90;
        c.clock();
        assert_eq!(c.a, 0x30);
        assert_eq!(c.p, Status::V | Status::U | Status::C);
    }

    #[test]
    fn test_sbc() {
        let mut c = create_test_cpu(&[
            // SBC #$05
            0xE9, 0x05,
        ]);

        // P - P = P
        // Carry bit not set
        // 9 - 5 = 3 (!!)
        c.reset();
        c.a = 0x09;
        c.clock();
        assert_eq!(c.a, 0x03);
        assert_eq!(c.p, Status::U | Status::C);

        // P - P = P
        // Carry bit set
        // 9 - 5 = 4
        let mut c = create_test_cpu(&[
            // SBC #$05
            0xE9, 0x05,
        ]);
        c.reset();
        c.p.set(Status::C, true);
        c.a = 0x09;
        c.clock();
        assert_eq!(c.a, 0x04);
        assert_eq!(c.p, Status::U | Status::C);
    }

    #[test]
    fn test_ora() {
        let mut c = create_test_cpu(&[
            // ORA $AB12
            0x0D, 0x12, 0xAB,
        ]);
        c.a = 0x03;
        c.write(0xab12, 0x05);
        c.clock();
        assert_eq!(c.a, 0x07);
    }

    #[test]
    fn test_pla() {
        let mut c = create_test_cpu(&[
            // PLA
            0x68,
        ]);

        // Stack underflow
        //
        //c.reset();
        // c.sp = 0xFF;
        // m.load(
        //     &[
        //         // PLA
        //         0x68
        //     ]
        // );
        // c.clock();

        // Pull one value
        c.reset();
        c.sp = 0xFE;
        c.write(0x01FF, 0xAB);

        c.clock();
        assert_eq!(c.a, 0xAB);
        assert_eq!(c.sp, 0xFF);
    }

    #[test]
    fn test_pha() {
        let mut c = create_test_cpu(&[
            // PHA
            0x48,
        ]);

        // Stack overflow
        //
        //c.reset();
        // c.sp = 0;
        // c.a = 0xAB;
        // m.load(
        //     &[
        //         // PHA
        //         0x48
        //     ]
        // );
        // c.clock();

        // Push one value
        c.reset();
        c.sp = 0xFF;
        c.a = 0xAB;

        c.clock();
        assert_eq!(c.read(0x01FF), 0xAB);
        assert_eq!(c.sp, 0xFE);
    }

    #[test]
    fn test_plp() {
        let mut c = create_test_cpu(&[
            // SEC; PHP; CLC; PLP
            0x38, 0x08, 0x18, 0x28,
        ]);
        c.reset();
        c.clock();
        assert_eq!(c.p, Status::U | Status::C);
    }

    #[test]
    fn test_tax() {
        let mut c = create_test_cpu(&[
            // TAX
            0xAA,
        ]);

        c.reset();
        c.a = 0xAB;

        c.clock();
        assert_eq!(c.x, 0xAB);
    }

    #[test]
    fn test_bit() {
        let mut c = create_test_cpu(&[]);
        c.a = 0b1;

        c.write(0, 0b0);
        c.bit();
        assert_eq!(c.p, Status::Z);

        c.write(0, 0b1);
        c.bit();
        assert_eq!(c.p, Status::empty());

        c.write(0, 0b11000001);
        c.bit();
        assert_eq!(c.p, Status::N | Status::V);

        c.write(0, 0b10000001);
        c.bit();
        assert_eq!(c.p, Status::N);

        c.write(0, 0b01000001);
        c.bit();
        assert_eq!(c.p, Status::V);
    }
}
