use std::ops::Mul;

use crate::emulator::state::SystemState;

#[derive(Debug)]
pub enum AddressingMode {
    IndirectZeroPageX,
    DirectZeroPage,
    Immediate,
    DirectAbsolute,
    IndirectZeroPageY,
    DirectZeroPageX,
    DirectAbsoluteY,
    DirectAbsoluteX,
    Accumulator,
    Relative,
}

#[derive(Debug)]
pub enum Instruction {
    GroupMultipleByte(MultiInstruction, AddressingMode),
    GroupSingleByte(SingleByteInstruction),
}
#[derive(Debug)]
pub enum MultiInstruction {
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
}

#[repr(u8)]
#[derive(Debug)]
pub enum SingleByteInstruction {
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
}

impl TryFrom<u8> for Instruction {
    type Error = ();
    fn try_from(value: u8) -> Result<Instruction, Self::Error> {
        let group_bits = value & 0b11;
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        // Single byte and special multibyte carveout as an exception
        match value {
            0x08 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::PHP)),
            0x28 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::PLP)),
            0x48 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::PHA)),
            0x68 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::PLA)),
            0x88 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::DEY)),
            0xA8 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::TAY)),
            0xC8 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::INY)),
            0xE8 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::INX)),
            0x18 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::CLC)),
            0x38 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::SEC)),
            0x58 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::CLI)),
            0x78 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::SEI)),
            0x98 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::TYA)),
            0xB8 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::CLV)),
            0xD8 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::CLD)),
            0xF8 => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::SED)),
            0x8A => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::TXA)),
            0x9A => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::TXS)),
            0xAA => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::TAX)),
            0xBA => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::TSX)),
            0xCA => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::DEX)),
            0xEA => return Ok(Instruction::GroupSingleByte(SingleByteInstruction::NOP)),
            0x10 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BPL,
                    AddressingMode::Relative,
                ))
            }
            0x30 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BMI,
                    AddressingMode::Relative,
                ))
            }
            0x50 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BVC,
                    AddressingMode::Relative,
                ))
            }
            0x70 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BVS,
                    AddressingMode::Relative,
                ))
            }
            0x90 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BCC,
                    AddressingMode::Relative,
                ))
            }
            0xB0 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BCS,
                    AddressingMode::Relative,
                ))
            }
            0xD0 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BNE,
                    AddressingMode::Relative,
                ))
            }
            0xF0 => {
                return Ok(Instruction::GroupMultipleByte(
                    MultiInstruction::BEQ,
                    AddressingMode::Relative,
                ))
            }
            _ => (),
        };

        match group_bits {
            0b01 => {
                let instruction = match instruction_bits {
                    0b000 => MultiInstruction::ORA,
                    0b001 => MultiInstruction::AND,
                    0b010 => MultiInstruction::EOR,
                    0b011 => MultiInstruction::ADC,
                    0b100 => MultiInstruction::STA,
                    0b101 => MultiInstruction::LDA,
                    0b110 => MultiInstruction::CMP,
                    0b111 => MultiInstruction::SBC,
                    _ => return Err(()),
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
                    _ => return Err(()),
                };

                Ok(Instruction::GroupMultipleByte(instruction, mode))
            }
            0b10 => {
                let instruction = match instruction_bits {
                    0b000 => MultiInstruction::ASL,
                    0b001 => MultiInstruction::ROL,
                    0b010 => MultiInstruction::LSR,
                    0b011 => MultiInstruction::ROR,
                    0b100 => MultiInstruction::STX,
                    0b101 => MultiInstruction::LDX,
                    0b110 => MultiInstruction::DEC,
                    0b111 => MultiInstruction::INC,
                    _ => return Err(()),
                };

                let mode = match mode_bits {
                    0b000 => AddressingMode::Immediate,
                    0b001 => AddressingMode::DirectZeroPage,
                    0b010 => AddressingMode::Accumulator,
                    0b011 => AddressingMode::DirectAbsolute,
                    0b101 => AddressingMode::DirectZeroPageX,
                    0b111 => AddressingMode::DirectAbsoluteX,
                    _ => return Err(()),
                };

                Ok(Instruction::GroupMultipleByte(instruction, mode))
            }
            0b00 => {
                let instruction = match instruction_bits {
                    0b001 => MultiInstruction::BIT,
                    0b010 => MultiInstruction::JMP,
                    0b011 => MultiInstruction::JMPAbsolute,
                    0b100 => MultiInstruction::STY,
                    0b101 => MultiInstruction::LDY,
                    0b110 => MultiInstruction::CPY,
                    0b111 => MultiInstruction::CPX,
                    _ => return Err(()),
                };

                let mode = match mode_bits {
                    0b000 => AddressingMode::Immediate,
                    0b001 => AddressingMode::DirectZeroPage,
                    0b011 => AddressingMode::DirectAbsolute,
                    0b101 => AddressingMode::DirectZeroPageX,
                    0b111 => AddressingMode::DirectAbsoluteX,
                    _ => return Err(()),
                };

                Ok(Instruction::GroupMultipleByte(instruction, mode))
            }
            _ => Err(()),
        }
    }
}

impl Instruction {
    pub fn execute<'a>(&self, state: &mut SystemState) -> Result<(), ()> {
        let argument: u16 = match *self {
            Instruction::GroupMultipleByte(_, AddressingMode::Immediate)
            | Instruction::GroupMultipleByte(_, AddressingMode::Relative) => {
                match state.consume_byte() {
                    Some(argument) => argument.into(),
                    _ => return Err(()),
                }
            }
            Instruction::GroupMultipleByte(_, AddressingMode::DirectZeroPage) => {
                match state.consume_byte() {
                    Some(argument) => argument.into(),
                    _ => return Err(()),
                }
            }
            Instruction::GroupMultipleByte(_, AddressingMode::DirectZeroPageX) => {
                match state.consume_byte() {
                    Some(byte) => byte.overflowing_add(state.x).0.into(),
                    _ => return Err(()),
                }
            }
            Instruction::GroupMultipleByte(_, AddressingMode::DirectAbsolute) => {
                // In absolute addressing, the second byte of the instruction specifies the eight low order bits of the effective address while the third byte specifies the eight high order bits. Thus, the absolute addressing mode allows access to the entire 65 K bytes of addressable memory.
                match (state.consume_byte(), state.consume_byte()) {
                    (Some(low_byte), Some(high_byte)) => {
                        let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                        address
                    }
                    _ => return Err(()),
                }
            }
            Instruction::GroupSingleByte(_) => 0,
            _ => return Err(()),
        };

        match *self {
            Instruction::GroupSingleByte(ref instruction) => match instruction {
                SingleByteInstruction::CLC => {
                    state.flags.set_carry_flag(false);
                },
                SingleByteInstruction::CLD => {
                    state.flags.set_decimal_flag(false);
                },
                SingleByteInstruction::CLI => {
                    state.flags.set_interrupt_disable_flag(false);
                },
                SingleByteInstruction::CLV => {
                    state.flags.set_overflow_flag(false);
                },
                SingleByteInstruction::DEX => {
                    state.x = state.x.overflowing_add(255).0;
                    state.flags.set_zero_flag(state.x == 0);
                    state
                        .flags
                        .set_negative_flag((state.x & 0b10000000) == 0b10000000);
                },
                SingleByteInstruction::SEI => {
                    state.flags.set_interrupt_disable_flag(true);
                },
                SingleByteInstruction::TXS => {
                    state.s = state.x;
                },
                
                _ => return Err(()),
            },
            Instruction::GroupMultipleByte(MultiInstruction::ADC, _) => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };

                // TODO: Decimal mode
                let carry_flag: u16 = match state.flags.carry_flag() {
                    true => 1,
                    false => 0,
                };
                let result: u16 = state.a as u16 + argument as u16 + carry_flag;
                if result > u8::MAX.into() {
                    state.flags.set_carry_flag(true);
                    state.flags.set_overflow_flag(true);
                }
                state.a = result as u8;
                state.flags.set_zero_flag(state.a == 0);
            }
            Instruction::GroupMultipleByte(MultiInstruction::AND, _) => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };
                state.a = argument & state.a;
                state.flags.set_zero_flag(state.a == 0);
                state
                    .flags
                    .set_negative_flag((state.a & 0b10000000) == 0b10000000)
            }
            Instruction::GroupMultipleByte(MultiInstruction::ASL, ref mode) => {
                let location = match mode {
                    AddressingMode::Accumulator => state.a,
                    _ => match state.fetch_memory(argument.into()) {
                        Ok(a) => a,
                        Err(_) => return Err(()),
                    },
                };
                let (value, overflow) = location.overflowing_shl(1);

                match mode {
                    AddressingMode::Accumulator => {
                        state.a = value;
                    }
                    _ => match state.write_memory(argument.into(), value) {
                        Err(_) => return Err(()),
                        _ => (),
                    },
                };

                state.flags.set_carry_flag(overflow);
                state.flags.set_zero_flag(value == 0);
                state
                    .flags
                    .set_negative_flag((value & 0b10000000) == 0b10000000);
            }
            Instruction::GroupMultipleByte(MultiInstruction::BCC, AddressingMode::Relative) => {
                if state.flags.carry_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            }
            Instruction::GroupMultipleByte(MultiInstruction::BCS, AddressingMode::Relative) => {
                if state.flags.carry_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            }
            Instruction::GroupMultipleByte(MultiInstruction::BEQ, AddressingMode::Relative) => {
                if state.flags.zero_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultiInstruction::BIT, _) => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };
                let result = argument & state.a;
                state.flags.set_zero_flag(result == 0);
                state.flags.set_overflow_flag(argument & 0b01000000  == 0b01000000);
                state
                    .flags
                    .set_negative_flag((argument & 0b10000000) == 0b10000000)
            },
            Instruction::GroupMultipleByte(MultiInstruction::BMI, AddressingMode::Relative) => {
                if state.flags.negative_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultiInstruction::BNE, AddressingMode::Relative) => {
                if state.flags.zero_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            }
            Instruction::GroupMultipleByte(MultiInstruction::BPL, AddressingMode::Relative) => {
                if state.flags.negative_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultiInstruction::BVC, AddressingMode::Relative) => {
                if state.flags.overflow_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultiInstruction::BVS, AddressingMode::Relative) => {
                if state.flags.overflow_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultiInstruction::CMP, _) => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };
                let result = state.a - argument;
                state.flags.set_zero_flag(result == 0);
                state.flags.set_carry_flag(argument <= state.a);
                state
                    .flags
                    .set_negative_flag((argument & 0b10000000) == 0b10000000)
            },
            Instruction::GroupMultipleByte(MultiInstruction::LDA, _) => {
                state.a = argument as u8;
            },
            Instruction::GroupMultipleByte(MultiInstruction::LDX, _) => {
                state.x = argument as u8;
                state.flags.set_zero_flag(state.x == 0);
                state
                    .flags
                    .set_negative_flag((state.x & 0b01000000) == 0b01000000);
            },
            Instruction::GroupMultipleByte(MultiInstruction::STA, _) => {
                let _ = state.write_memory(argument.into(), state.a);
            },
            Instruction::GroupMultipleByte(MultiInstruction::JMP, _) => {
                state.pc = argument.into();
            },


            _ => return Err(()),
        }
        // state.print_registers();
        Ok(())
    }
}
