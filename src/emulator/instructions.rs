use crate::emulator::state::{EmulatorError, SystemState, SystemFlags};
use anyhow::{anyhow, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    DirectAbsolute,
    DirectAbsoluteX,
    DirectAbsoluteY,
    // AbsoluteIndirect
    DirectZeroPage,
    DirectZeroPageX,
    DirectZeroPageY,
    IndirectZeroPageX,
    IndirectZeroPageY,
    Relative,
}   

#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: OpCode,
    pub mode: Option<AddressingMode>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpCode {
    ORA,
    AND,
    EOR,
    ADC,
    STA,
    LDA,
    CMP,
    SBC,
    ASL,
    ROL,
    LSR,
    ROR,
    STX,
    LDX,
    DEC,
    INC,
    BIT,         // 001
    JMP,         // 010
    JSR,
    JMPAbsolute, // 011
    STY,         // 100
    LDY,         // 101
    CPY,         // 110
    CPX,         // 111
    // Special bytes
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    PHP,
    PLP,
    PHA,
    PLA,
    DEY,
    TAY,
    INY,
    INX,
    CLC,
    SEC,
    CLI,
    SEI,
    TYA,
    CLV,
    CLD,
    SED,
    TXA,
    TXS,
    TAX,
    TSX,
    DEX,
    NOP,
    BRK,
    RTI,
    RTS,
    BadInstruction(u8),
    UnknownInstruction(u8),
    // Illegal Opcode Callout.
    // https://www.masswerk.at/nowgobang/2021/6502-illegal-opcodes
    // https://www.oxyron.de/html/opcodes02.html
    ALR,
    ANC,
    ANC2,
    ANE,
    ARR,
    DCP,
    ISC,
    LAS,
    LAX,
    LXA,
    RLA,
    RRA,
    SAX,
    SBX,
    SHA,
    SHX,
    SHY,
    SLO,
    SRE,
    TAS,
    USBC,
    INOP,
    KIL,
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        let group_bits = value & 0b11;
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;
        // Illegal Opcode Callout.
        // https://www.masswerk.at/nowgobang/2021/6502-illegal-opcodes
        // https://www.oxyron.de/html/opcodes02.html
        //
        match value {
            0x4b => return Instruction{ opcode: OpCode::ALR, mode: Some(AddressingMode::Immediate)},
            0x0b => return Instruction{ opcode: OpCode::ANC, mode: Some(AddressingMode::Immediate)},
            0x2b => return Instruction{ opcode: OpCode::ANC2, mode: Some(AddressingMode::Immediate)},
            0x8b => return Instruction{ opcode: OpCode::ANE, mode: Some(AddressingMode::Immediate)},
            0x6b => return Instruction{ opcode: OpCode::ARR, mode: Some(AddressingMode::Immediate)},
            0xc7 => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::DirectZeroPage)},
            0xd7 => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::DirectZeroPageX)},
            0xcf => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::DirectAbsolute)},
            0xdf => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0xdb => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::DirectAbsoluteY)},
            0xc3 => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::IndirectZeroPageX)},
            0xd3 => return Instruction{ opcode: OpCode::DCP, mode: Some(AddressingMode::IndirectZeroPageY)},
            0xe7 => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::DirectZeroPage)},
            0xf7 => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::DirectZeroPageX)},
            0xef => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::DirectAbsolute)},
            0xff => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::DirectAbsoluteX)},
            0xfb => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::DirectAbsoluteY)},
            0xe3 => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::IndirectZeroPageX)},
            0xf3 => return Instruction{ opcode: OpCode::ISC, mode: Some(AddressingMode::IndirectZeroPageY)},
            0xbb => return Instruction{ opcode: OpCode::LAS, mode: Some(AddressingMode::DirectAbsoluteY)},
            0xa7 => return Instruction{ opcode: OpCode::LAX, mode: Some(AddressingMode::DirectZeroPage)},
            0xb7 => return Instruction{ opcode: OpCode::LAX, mode: Some(AddressingMode::DirectZeroPageY)},
            0xaf => return Instruction{ opcode: OpCode::LAX, mode: Some(AddressingMode::DirectAbsolute)},
            0xbf => return Instruction{ opcode: OpCode::LAX, mode: Some(AddressingMode::DirectAbsoluteY)},
            0xa3 => return Instruction{ opcode: OpCode::LAX, mode: Some(AddressingMode::IndirectZeroPageX)},
            0xb3 => return Instruction{ opcode: OpCode::LAX, mode: Some(AddressingMode::IndirectZeroPageY)},
            0xab => return Instruction{ opcode: OpCode::LXA, mode: Some(AddressingMode::Immediate)},
            0x27 => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::DirectZeroPage)},
            0x37 => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::DirectZeroPageX)},
            0x2f => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::DirectAbsolute)},
            0x3f => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x3b => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::DirectAbsoluteY)},
            0x23 => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::IndirectZeroPageX)},
            0x33 => return Instruction{ opcode: OpCode::RLA, mode: Some(AddressingMode::IndirectZeroPageY)},
            0x67 => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::DirectZeroPage)},
            0x77 => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::DirectZeroPageX)},
            0x6f => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::DirectAbsolute)},
            0x7f => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x7b => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::DirectAbsoluteY)},
            0x63 => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::IndirectZeroPageX)},
            0x73 => return Instruction{ opcode: OpCode::RRA, mode: Some(AddressingMode::IndirectZeroPageY)},
            0x87 => return Instruction{ opcode: OpCode::SAX, mode: Some(AddressingMode::DirectZeroPage)},
            0x97 => return Instruction{ opcode: OpCode::SAX, mode: Some(AddressingMode::DirectZeroPageY)},
            0x8f => return Instruction{ opcode: OpCode::SAX, mode: Some(AddressingMode::DirectAbsolute)},
            0x83 => return Instruction{ opcode: OpCode::SAX, mode: Some(AddressingMode::IndirectZeroPageX)},
            0xcb => return Instruction{ opcode: OpCode::SBX, mode: Some(AddressingMode::Immediate)},
            0x9f => return Instruction{ opcode: OpCode::SHA, mode: Some(AddressingMode::DirectAbsoluteY)},
            0x93 => return Instruction{ opcode: OpCode::SHA, mode: Some(AddressingMode::IndirectZeroPageY)},
            0x9e => return Instruction{ opcode: OpCode::SHX, mode: Some(AddressingMode::DirectAbsoluteY)},
            0x9c => return Instruction{ opcode: OpCode::SHY, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x07 => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::DirectZeroPage)},
            0x17 => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::DirectZeroPageX)},
            0x0f => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::DirectAbsolute)},
            0x1f => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x1b => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::DirectAbsoluteY)},
            0x03 => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::IndirectZeroPageX)},
            0x13 => return Instruction{ opcode: OpCode::SLO, mode: Some(AddressingMode::IndirectZeroPageY)},
            0x47 => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::DirectZeroPage)},
            0x57 => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::DirectZeroPageX)},
            0x4f => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::DirectAbsolute)},
            0x5f => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x5b => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::DirectAbsoluteY)},
            0x43 => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::IndirectZeroPageX)},
            0x53 => return Instruction{ opcode: OpCode::SRE, mode: Some(AddressingMode::IndirectZeroPageY)},
            0x9b => return Instruction{ opcode: OpCode::TAS, mode: Some(AddressingMode::DirectAbsoluteY)},
            0xeb => return Instruction{ opcode: OpCode::USBC, mode: Some(AddressingMode::Immediate)},
            0x1a => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Implied)},
            0x3a => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Implied)},
            0x5a => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Implied)},
            0x7a => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Implied)},
            0xda => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Implied)},
            0xfa => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Implied)},
            0x80 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Immediate)},
            0x82 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Immediate)},
            0x89 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Immediate)},
            0xc2 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Immediate)},
            0xe2 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::Immediate)},
            0x04 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPage)},
            0x44 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPage)},
            0x64 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPage)},
            0x14 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPageX)},
            0x34 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPageX)},
            0x54 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPageX)},
            0x74 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPageX)},
            0xd4 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPageX)},
            0xf4 => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectZeroPageX)},
            0x0c => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsolute)},
            0x1c => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x3c => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x5c => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x7c => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0xdc => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0xfc => return Instruction{ opcode: OpCode::INOP, mode: Some(AddressingMode::DirectAbsoluteX)},
            0x02 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x12 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x22 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x32 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x42 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x52 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x62 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x72 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0x92 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0xb2 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0xd2 => return Instruction{ opcode: OpCode::KIL, mode: None},
            0xf2 => return Instruction{ opcode: OpCode::KIL, mode: None},
            _ => ()
        }

        // Single byte and special multibyte carveout as an exception
        match value {
            // https://www.masswerk.at/6502/6502_instruction_set.html
            0x00 => return Instruction { opcode: OpCode::BRK, mode: None},
            0x08 => return Instruction { opcode: OpCode::PHP, mode: None },
            0x28 => return Instruction { opcode: OpCode::PLP, mode: None },
            0x48 => return Instruction { opcode: OpCode::PHA, mode: None },
            0x68 => return Instruction { opcode: OpCode::PLA, mode: None },
            0x88 => return Instruction { opcode: OpCode::DEY, mode: None },
            0xA8 => return Instruction { opcode: OpCode::TAY, mode: None },
            0xC8 => return Instruction { opcode: OpCode::INY, mode: None },
            0xE8 => return Instruction { opcode: OpCode::INX, mode: None },
            0x18 => return Instruction { opcode: OpCode::CLC, mode: None },
            0x38 => return Instruction { opcode: OpCode::SEC, mode: None },
            0x58 => return Instruction { opcode: OpCode::CLI, mode: None },
            0x78 => return Instruction { opcode: OpCode::SEI, mode: None },
            0x98 => return Instruction { opcode: OpCode::TYA, mode: None },
            0xB8 => return Instruction { opcode: OpCode::CLV, mode: None },
            0xD8 => return Instruction { opcode: OpCode::CLD, mode: None },
            0xF8 => return Instruction { opcode: OpCode::SED, mode: None },
            0x8A => return Instruction { opcode: OpCode::TXA, mode: None },
            0x9A => return Instruction { opcode: OpCode::TXS, mode: None },
            0xAA => return Instruction { opcode: OpCode::TAX, mode: None },
            0xBA => return Instruction { opcode: OpCode::TSX, mode: None },
            0xCA => return Instruction { opcode: OpCode::DEX, mode: None },
            0xEA => return Instruction { opcode: OpCode::NOP, mode: None },
            0x10 => return Instruction {opcode: OpCode::BPL, mode: Some(AddressingMode::Relative) },
            0x30 => return Instruction {opcode: OpCode::BMI, mode: Some(AddressingMode::Relative) },
            0x50 => return Instruction {opcode: OpCode::BVC, mode: Some(AddressingMode::Relative) },
            0x70 => return Instruction {opcode: OpCode::BVS, mode: Some(AddressingMode::Relative) },
            0x90 => return Instruction {opcode: OpCode::BCC, mode: Some(AddressingMode::Relative) },
            0xB0 => return Instruction {opcode: OpCode::BCS, mode: Some(AddressingMode::Relative) },
            0xD0 => return Instruction {opcode: OpCode::BNE, mode: Some(AddressingMode::Relative) },
            0xF0 => return Instruction {opcode: OpCode::BEQ, mode: Some(AddressingMode::Relative) },
            // note: resolved from official table. This isn't mapped onto reality though.
            0x80 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x02 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x12 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x22 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x32 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x42 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x52 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x62 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x72 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x82 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x92 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xb2 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xc2 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xd2 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xe2 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xf2 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x03 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x13 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x23 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x33 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x43 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x53 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x63 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x73 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x83 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x93 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xa3 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xb3 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xc3 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xd3 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xe3 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xf3 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x04 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x14 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x34 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x44 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x54 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x64 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x74 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xd4 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xf4 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x07 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x17 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x27 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x37 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x47 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x57 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x67 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x77 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x87 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x97 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xa7 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xb7 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xc7 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xd7 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xe7 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xf7 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x89 => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x1a => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x3a => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x5a => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x7a => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xda => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xfa => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x0b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x1b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x2b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x3b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x4b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x5b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x6b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x7b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x8b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x9b => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xab => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xbb => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xcb => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xdb => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xeb => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xfb => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x0c => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x1c => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x3c => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x5c => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x7c => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x9c => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xdc => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xfc => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x0f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x1f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x2f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x3f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x4f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x5f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x6f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x7f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x8f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0x9f => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xaf => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xbf => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xcf => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xdf => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xef => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            0xff => return Instruction {opcode: OpCode::BadInstruction(value), mode: None},
            _ => (),
        };

        match group_bits {
            0b01 => {
                let instruction = match instruction_bits {
                    0b000 => OpCode::ORA,
                    0b001 => OpCode::AND,
                    0b010 => OpCode::EOR,
                    0b011 => OpCode::ADC,
                    0b100 => OpCode::STA,
                    0b101 => OpCode::LDA,
                    0b110 => OpCode::CMP,
                    0b111 => OpCode::SBC,
                    _ => return Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
                };

                let mode = match mode_bits {
                    0b000 => AddressingMode::IndirectZeroPageX,
                    0b001 => AddressingMode::DirectZeroPage,
                    0b010 => AddressingMode::Immediate,
                    0b011 => AddressingMode::DirectAbsolute,
                    0b100 => AddressingMode::IndirectZeroPageY,
                    0b101 => AddressingMode::DirectZeroPageX,
                    0b110 => AddressingMode::DirectAbsoluteY,
                    0b111 => AddressingMode::DirectAbsoluteX,
                    _ => return Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
                };

                Instruction{ opcode: instruction, mode: Some(mode) }
            }
            0b10 => {
                let instruction = match instruction_bits {
                    0b000 => OpCode::ASL,
                    0b001 => OpCode::ROL,
                    0b010 => OpCode::LSR,
                    0b011 => OpCode::ROR,
                    0b100 => OpCode::STX,
                    0b101 => OpCode::LDX,
                    0b110 => OpCode::DEC,
                    0b111 => OpCode::INC,
                    _ => return Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
                };

                let mode = match mode_bits {
                    0b000 => AddressingMode::Immediate,
                    0b001 => AddressingMode::DirectZeroPage,
                    0b010 => AddressingMode::Accumulator,
                    0b011 => AddressingMode::DirectAbsolute,
                    0b101 => AddressingMode::DirectZeroPageX,
                    0b111 => AddressingMode::DirectAbsoluteX,
                    _ => return Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
                };

                Instruction{ opcode: instruction, mode: Some(mode) }
            }
            0b00 => {
                let instruction = match instruction_bits {
                    0b001 => OpCode::BIT,
                    0b010 => OpCode::JMP,
                    0b011 => OpCode::JMPAbsolute,
                    0b100 => OpCode::STY,
                    0b101 => OpCode::LDY,
                    0b110 => OpCode::CPY,
                    0b111 => OpCode::CPX,
                    _ => return Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
                };

                let mode = match mode_bits {
                    0b000 => AddressingMode::Immediate,
                    0b001 => AddressingMode::DirectZeroPage,
                    0b011 => AddressingMode::DirectAbsolute,
                    0b101 => AddressingMode::DirectZeroPageX,
                    0b111 => AddressingMode::DirectAbsoluteX,
                    _ => return Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
                };

                Instruction{ opcode: instruction, mode: Some(mode) }
            }
            _ => Instruction {opcode: OpCode::UnknownInstruction(value), mode: None},
        }
    }
}



#[derive(Debug)]
pub struct MemoryPair {
    pub address: u16,
    pub value: u8,
}

impl Instruction {
    pub fn execute(&self, state: &mut SystemState) -> Result<()> {
        let memory_pair = match self.mode {
            Some(AddressingMode::Immediate | AddressingMode::Relative) => {
                let address = state.pc();
                let value = state.read(address)?;
                state.set_pc(state.pc().wrapping_add(1));
                Some(MemoryPair { address , value})
            }
            Some(AddressingMode::DirectZeroPage) => {
                let address = state.pc();
                let address = state.read(address)? as u16;
                state.set_pc(state.pc().wrapping_add(1));
                let value = state.read(address)?;
                Some(MemoryPair { address , value})
            }
            Some(AddressingMode::DirectZeroPageX) => {
                let address = state.pc();
                let address = state.read(address)?.overflowing_add(state.x).0;
                state.set_pc(state.pc().wrapping_add(1));
                let value = state.read(address.into())?;
                Some(MemoryPair { address: address.into(), value })
            }
            Some(AddressingMode::DirectAbsolute) => {
                // In absolute addressing, the second byte of the instruction specifies the eight low order bits of the effective address while the third byte specifies the eight high order bits. Thus, the absolute addressing mode allows access to the entire 65 K bytes of addressable memory.
                
                let low_byte = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let high_byte = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                let value = state.read(address)?;
                Some(MemoryPair { address, value })
            },
            Some(AddressingMode::DirectAbsoluteX) => {
                let low_byte = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let high_byte = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                let address = address.overflowing_add(state.x.into()).0;
                let value = state.read(address)?;
                Some(MemoryPair { address, value })
            }
            Some(AddressingMode::DirectAbsoluteY) => {
                let low_byte = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let high_byte = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                let address = address.overflowing_add(state.y.into()).0;
                let value = state.read(address)?;
                Some(MemoryPair { address, value })
            },
            Some(AddressingMode::IndirectZeroPageX) => {
                
                let zero_page_address =(state.read(state.pc())?).overflowing_add(state.x).0.into();
                state.set_pc(state.pc().wrapping_add(1));
                let low_byte = state.read(zero_page_address)?;
                let high_byte = state.read((zero_page_address as u8).wrapping_add(1) as u16)?;
                
                let address = ((high_byte as u16) << 8) + low_byte as u16;
                let value = state.read(address)?;
                Some(MemoryPair { address, value })
            }
            Some(AddressingMode::Accumulator) => {
                None
            }
            Some(AddressingMode::IndirectZeroPageY) => {
                
                // In indirect indexed addressing, the second byte of the instruction points to a memory 
                //location in page zero. The contents of this memory location is added to the contents of 
                //the Y index register, the result being the low order eight bits of the effective address. 
                //The carry from this addition is added to the contents of the next page zero memory location, 
                //the result being the high order eight bits of the effective address.
                let next_address = state.read(state.pc())?;
                state.set_pc(state.pc().wrapping_add(1));
                let (low_byte, overflow) = (state.read(next_address as u16)?).overflowing_add(state.y);
                let overflow = match overflow {
                    true => 1u8,
                    false => 0u8,
                };
                let high_byte = state.read(next_address.wrapping_add(1) as u16)?.overflowing_add(overflow).0;
                let address = ((high_byte as u16) << 8) + low_byte as u16;
                let value = state.read(address)?;
                Some(MemoryPair { address, value })
            }
            None => None,
        };

        match self.opcode {
            OpCode::ADC => {
                let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;

                // TODO: Decimal mode
                let carry_flag: u16 = match state.p.contains(SystemFlags::carry) {
                    true => 1,
                    false => 0,
                };
                let result: u16 = state.a as u16 + argument as u16 + carry_flag;
                state.p.set(SystemFlags::carry, result > u8::MAX.into());
                // TODO: bad logic
                state.p.set(SystemFlags::overflow, result > u8::MAX.into());
                
                state.a = result as u8;
                state.p.set(SystemFlags::zero, state.a == 0);
            },
            OpCode::AND => {
                let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;

                state.a &= argument;
                state.p.set(SystemFlags::zero, state.a == 0);
                state.p.set(SystemFlags::negative, (state.a & 0b10000000) == 0b10000000);
            },
            OpCode::ASL => {
                let (value, overflow) = match self.mode {
                    Some(AddressingMode::Accumulator) => {
                        let out = state.a.overflowing_shl(1);
                        state.a = out.0;
                        out
                    },
                    _ => {
                        let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                        let address = memory_pair.address;
                        let value = memory_pair.value;
                        let out = value.overflowing_shl(1);
                        state.write(address, out.0)?;
                        out
                    }
                };

                state.p.set(SystemFlags::carry, overflow);
                state.p.set(SystemFlags::zero, value == 0);
                state.p.set(SystemFlags::negative, (value & 0b10000000) == 0b10000000);
            },
            OpCode::BCC => {
                // TODO: Evaluate this.
                if !state.p.contains(SystemFlags::carry) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp: u16 = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BCS => {
                // TODO: Evaluate this.
                if !state.p.contains(SystemFlags::carry) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BEQ => {
                // TODO: Evaluate this.
                if state.p.contains(SystemFlags::zero) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BIT => {
                let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;
                let result = argument & state.a;
                state.p.set(SystemFlags::zero, result == 0);
                state.p.set(SystemFlags::overflow, (argument & 0b01000000)  == 0b01000000);
                state.p.set(SystemFlags::negative, (argument & 0b10000000) == 0b10000000);
            },
            OpCode::BMI => {
                // TODO: Evaluate this.
                if state.p.contains(SystemFlags::negative) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BNE => {
                // TODO: Evaluate this.
                if !state.p.contains(SystemFlags::zero) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BPL => {
                // TODO: Evaluate this.
                if !state.p.contains(SystemFlags::negative) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BRK => {
                let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.address;
                let low_byte = (state.pc() & 0xFF) as u8;
                let high_byte = (state.pc().overflowing_shr(8).0 & 0xFF) as u8;

                state.write(state.s.into(), high_byte)?;
                state.s = state.s.wrapping_sub(1);
                state.write(state.s.into(), low_byte)?;
                state.s = state.s.wrapping_sub(1);
                state.write(state.s.into(), state.p.bits())?;
                state.s = state.s.wrapping_sub(1);
                // We don't mutate PC, we mutate base address which mutates PC
                state.set_pc(argument);

            }
            OpCode::BVC => {
                if !state.p.contains(SystemFlags::overflow) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::BVS => {
                if state.p.contains(SystemFlags::overflow) {
                    let argument = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        // We don't mutate PC, we mutate base address which mutates PC
                        state.set_pc(state.pc().overflowing_add(argument as u16).0);
                    } else {
                        // We don't mutate PC, we mutate base address which mutates PC
                        let temp = if argument == i8::MIN {
                            (i8::MAX as u16) + 1
                        } else {
                            argument.unsigned_abs() as u16
                        };
                        state.set_pc(state.pc().overflowing_sub(temp).0);
                    }
                }
            },
            OpCode::CLC => {
                state.p.remove(SystemFlags::carry);
            },
            OpCode::CLD => {
                state.p.remove(SystemFlags::decimal);
            },
            OpCode::CLI => {
                state.p.remove(SystemFlags::interrupt_disable);
            },
            OpCode::CLV => {
                state.p.remove(SystemFlags::overflow);
            },
            OpCode::CMP => {
                let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                let value = memory_pair.value;
                let result = state.a.overflowing_sub(value).0;
                state.p.set(SystemFlags::zero, result == 0);
                state.p.set(SystemFlags::carry, value <= state.a);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000)
            },
            OpCode::CPX => {
                let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                let value = memory_pair.value;
                let result = state.x.overflowing_sub(value).0;
                state.p.set(SystemFlags::zero, result == 0);
                state.p.set(SystemFlags::carry, state.x >= value);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000)
            },
            OpCode::CPY => {
                let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                let value = memory_pair.value;

                let result = state.y.overflowing_sub(value).0;
                state.p.set(SystemFlags::zero, result == 0);
                state.p.set(SystemFlags::carry, state.y >= value);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000)
            },
            OpCode::DEC => {
                let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                let address = memory_pair.address;
                let value = memory_pair.value;

                let value = value.wrapping_sub(1);
                state.write(address, value)?;

                state.p.set(SystemFlags::zero, value == 0);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000);
            },
            OpCode::DEX => {
                state.y = state.x.overflowing_sub(1).0;
                state.p.set(SystemFlags::zero, state.x == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.x & 0b10000000) == 0b10000000);
            },
            OpCode::DEY => {
                state.y = state.x.overflowing_sub(1).0;
                state.p.set(SystemFlags::zero, state.y == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.y & 0b10000000) == 0b10000000);
            },
            OpCode::EOR => {
                let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                let _ = memory_pair.address;
                let value = memory_pair.value;
                state.a ^= value;
                state.p.set(SystemFlags::zero, state.a == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.a & 0b10000000) == 0b10000000);
            },
            OpCode::INC => {
                let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                let address = memory_pair.address;
                let value = memory_pair.value;
                let value = value.wrapping_add(1);

                state.write(address, value)?;

                state.p.set(SystemFlags::zero, value == 0);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000);
            },
            OpCode::INX => {
                let value = state.x.wrapping_add(1);

                state.p.set(SystemFlags::zero, value == 0);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000);
            },
            OpCode::INY => {
                let value = state.y.wrapping_add(1);

                state.p.set(SystemFlags::zero, value == 0);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000);
            },
            OpCode::JMP => { // this should be 4c
                let address = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.address;
                // We don't mutate PC, we mutate base address which mutates PC
                state.set_pc(address);
            },
            OpCode::JSR => {
                let address = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.address;
                let low_byte = (state.pc() & 0xFF) as u8;
                let high_byte = (state.pc().overflowing_shr(8).0 & 0xFF) as u8;

                state.write(state.s.into(), high_byte)?;
                state.s = state.s.wrapping_sub(1);
                state.write(state.s.into(), low_byte)?;
                state.s = state.s.wrapping_sub(1);

                // We don't mutate PC, we mutate base address which mutates PC
                state.set_pc(address);
            },
            OpCode::LDA => {
                let value = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;
                state.a = value as u8;
                state.p.set(SystemFlags::zero, state.a == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.a & 0b10000000) == 0b10000000);
            },
            OpCode::LDX => {
                let value = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;
                state.x = value as u8;
                state.p.set(SystemFlags::zero, state.x == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.x & 0b01000000) == 0b01000000);
            },
            OpCode::LDY => {
                let value = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;
                state.y = value as u8;
                state.p.set(SystemFlags::zero, state.y == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.y & 0b01000000) == 0b01000000);
            },
            OpCode::LSR => {
                let (value, overflow) = match self.mode { 
                    Some(AddressingMode::Accumulator) =>  {
                        let out = state.a.overflowing_shr(1);
                        state.a = out.0;
                        out
                    },
                    _ => {
                        let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                        let address = memory_pair.address;
                        let value = memory_pair.value;

                        let out = value.overflowing_shr(1);
                        state.write(address, out.0)?;
                        out
                    }
                };
                state.p.set(SystemFlags::carry, overflow);
                state.p.set(SystemFlags::zero, value == 0);
                state
                    .p
                    .set(SystemFlags::negative, (value & 0b10000000) == 0b10000000);
            },
            OpCode::NOP => (),
            OpCode::ORA => {
                let value = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.value;
                state.a |= value;

                state.p.set(SystemFlags::zero, state.a == 0);
                state
                    .p
                    .set(SystemFlags::negative, (state.a & 0b10000000) == 0b10000000);
            },
            OpCode::PHA => {
                state.write(0x100 + state.s as u16, state.a)?;
                state.s = state.s.wrapping_sub(1);
                
            }
            OpCode::PHP => {
                state.write(0x100 + state.s as u16, state.p.bits())?;
                state.s = state.s.wrapping_sub(1);
            }
            OpCode::PLA => {
                state.s = state.s.wrapping_add(1);
                state.a = state.read(0x100 + state.s as u16)?;
            }
            OpCode::PLP => {
                state.s = state.s.wrapping_add(1);
                state.p = SystemFlags::from_bits_retain(state.read(0x100 + state.s as u16)?);
            }
            OpCode::ROL => {
                let (input, output) = match self.mode {
                    Some(AddressingMode::Accumulator) => {
                        let input = state.a;
                        let output = match state.p.contains(SystemFlags::carry) {
                            false => input << 1,
                            true =>  (input << 1) | 0x1,
                        };
                        state.a = output;
                        (input, output)
                    },
                    _ => {
                        let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                        println!("MemoryPair@ROL: {:?}", memory_pair);
                        let address = memory_pair.address;
                        let value: u8 = memory_pair.value;
                        let input = value;
                        let output = match state.p.contains(SystemFlags::carry) {
                            false => input << 1,
                            true =>  (input << 1) | 0x1,
                        };
                        state.write(address, output)?;
                        (input, output)

                    }
                };

                state.p.set(SystemFlags::carry, (input & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, output == 0);
                state
                    .p
                    .set(SystemFlags::negative, (output & 0b01000000) == 0b01000000);
            }
            
            OpCode::ROR => {
                let (input, output) = match self.mode {
                    Some(AddressingMode::Accumulator) => {
                        let input = state.a;
                        let output = match state.p.contains(SystemFlags::carry) {
                            false => input >> 1,
                            true =>  (input >> 1) | (0x1 << 7),
                        };
                        state.a = output;
                        (input, output)
                    },
                    _ => {
                        let memory_pair = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?;
                        let address = memory_pair.address;
                        let value = memory_pair.value;
                        let input = value;
                        let output = match state.p.contains(SystemFlags::carry) {
                            false => input >> 1,
                            true =>  (input >> 1) | 0x1,
                        };
                        state.write(address, output)?;
                        (input, output)

                    }
                };

                state.p.set(SystemFlags::negative, state.p.contains(SystemFlags::carry)) ;
                state.p.set(SystemFlags::carry, (input & 0b00000001) == 0b00000001);
                state.p.set(SystemFlags::zero, output == 0);
            }
            OpCode::RTI => {

                state.s = state.s.wrapping_add(1);
                let r1 = state.read(0x100 + state.s as u16)?;
                state.s = state.s.wrapping_add(1);
                let r2 = state.read(0x100 + state.s as u16)?;
                state.s = state.s.wrapping_add(1);
                let r3 = state.read(0x100 + state.s as u16)?;
                
                state.p =  SystemFlags::from_bits_retain(r1);
                state.set_pc((r2 as u16).overflowing_add((r3 as u16).overflowing_shl(8).0).0);
            }
            OpCode::RTS => {
                state.s = state.s.wrapping_add(1);
                let r1 = state.read(0x100 + state.s as u16)?;
                state.s = state.s.wrapping_add(1);
                let r2 = state.read(0x100 + state.s as u16)?;

                state.set_pc((r1 as u16).overflowing_add((r2 as u16).overflowing_shl(8).0).0);
            }
            OpCode::SEI => {
                state.p.insert(SystemFlags::interrupt_disable);
            },
            OpCode::STA => {
                let address = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.address;
                state.write(address, state.a)?;
            },
            OpCode::STX => {
                let address = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.address;
                state.write(address, state.x)?;
            },
            OpCode::STY => {
                let address = memory_pair.ok_or(anyhow!(EmulatorError::ExpectedMemoryPair))?.address;
                state.write(address, state.y)?;
            },
            OpCode::TAX => {
                let value = state.a;
                state.x = value;
                state.p.set(SystemFlags::carry, (value & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, value == 0);
            }
            OpCode::TAY => {
                let value = state.a;
                state.y = value;
                state.p.set(SystemFlags::carry, (value & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, value == 0);
            }
            OpCode::TSX => {
                let value = state.s;
                state.x = value;
                state.p.set(SystemFlags::carry, (value & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, value == 0);
            }
            OpCode::TXA => {
                let value = state.x;
                state.a = value;
                state.p.set(SystemFlags::carry, (value & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, value == 0);
            }
            OpCode::TXS => {
                let value = state.x;
                state.s = value;
                state.p.set(SystemFlags::carry, (value & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, value == 0);
            },
            OpCode::TYA => {
                let value = state.y;
                state.a = value;
                state.p.set(SystemFlags::carry, (value & 0b10000000) == 0b10000000);
                state.p.set(SystemFlags::zero, value == 0);
            },
            
            _ => return Err(anyhow!(EmulatorError::UnimplementedInstruction)),
        }
        // state.print_registers();
        Ok(())
    }
}
