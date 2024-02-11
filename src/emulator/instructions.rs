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
pub struct Instruction {
    opcode: OpCode,
    mode: Option<AddressingMode>,
}

#[derive(Debug)]
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
}


impl TryFrom<u8> for Instruction {
    type Error = ();
    fn try_from(value: u8) -> Result<Instruction, Self::Error> {
        let group_bits = value & 0b11;
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        // Single byte and special multibyte carveout as an exception
        match value {
            0x08 => return Ok(Instruction { opcode: OpCode::PHP, mode: None }),
            0x28 => return Ok(Instruction { opcode: OpCode::PLP, mode: None }),
            0x48 => return Ok(Instruction { opcode: OpCode::PHA, mode: None }),
            0x68 => return Ok(Instruction { opcode: OpCode::PLA, mode: None }),
            0x88 => return Ok(Instruction { opcode: OpCode::DEY, mode: None }),
            0xA8 => return Ok(Instruction { opcode: OpCode::TAY, mode: None }),
            0xC8 => return Ok(Instruction { opcode: OpCode::INY, mode: None }),
            0xE8 => return Ok(Instruction { opcode: OpCode::INX, mode: None }),
            0x18 => return Ok(Instruction { opcode: OpCode::CLC, mode: None }),
            0x38 => return Ok(Instruction { opcode: OpCode::SEC, mode: None }),
            0x58 => return Ok(Instruction { opcode: OpCode::CLI, mode: None }),
            0x78 => return Ok(Instruction { opcode: OpCode::SEI, mode: None }),
            0x98 => return Ok(Instruction { opcode: OpCode::TYA, mode: None }),
            0xB8 => return Ok(Instruction { opcode: OpCode::CLV, mode: None }),
            0xD8 => return Ok(Instruction { opcode: OpCode::CLD, mode: None }),
            0xF8 => return Ok(Instruction { opcode: OpCode::SED, mode: None }),
            0x8A => return Ok(Instruction { opcode: OpCode::TXA, mode: None }),
            0x9A => return Ok(Instruction { opcode: OpCode::TXS, mode: None }),
            0xAA => return Ok(Instruction { opcode: OpCode::TAX, mode: None }),
            0xBA => return Ok(Instruction { opcode: OpCode::TSX, mode: None }),
            0xCA => return Ok(Instruction { opcode: OpCode::DEX, mode: None }),
            0xEA => return Ok(Instruction { opcode: OpCode::NOP, mode: None }),
            0x10 => return Ok(Instruction {opcode: OpCode::BPL, mode: Some(AddressingMode::Relative) }),
            0x30 => return Ok(Instruction {opcode: OpCode::BMI, mode: Some(AddressingMode::Relative) }),
            0x50 => return Ok(Instruction {opcode: OpCode::BVC, mode: Some(AddressingMode::Relative) }),
            0x70 => return Ok(Instruction {opcode: OpCode::BVS, mode: Some(AddressingMode::Relative) }),
            0x90 => return Ok(Instruction {opcode: OpCode::BCC, mode: Some(AddressingMode::Relative) }),
            0xB0 => return Ok(Instruction {opcode: OpCode::BCS, mode: Some(AddressingMode::Relative) }),
            0xD0 => return Ok(Instruction {opcode: OpCode::BNE, mode: Some(AddressingMode::Relative) }),
            0xF0 => return Ok(Instruction {opcode: OpCode::BEQ, mode: Some(AddressingMode::Relative) }),
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

                Ok(Instruction{ opcode: instruction, mode: Some(mode) })
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

                Ok(Instruction{ opcode: instruction, mode: Some(mode) })
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

                Ok(Instruction{ opcode: instruction, mode: Some(mode) })
            }
            _ => Err(()),
        }
    }
}

impl Instruction {
    pub fn execute<'a>(&self, state: &mut SystemState) -> Result<(), ()> {
        let argument: u16 = match self.mode {
            Some(AddressingMode::Immediate | AddressingMode::Relative) => {
                match state.consume_byte() {
                    Some(argument) => argument.into(),
                    _ => return Err(()),
                }
            }
            Some(AddressingMode::DirectZeroPage) => {
                match state.consume_byte() {
                    Some(argument) => argument.into(),
                    _ => return Err(()),
                }
            }
            Some(AddressingMode::DirectZeroPageX) => {
                match state.consume_byte() {
                    Some(byte) => byte.overflowing_add(state.x).0.into(),
                    _ => return Err(()),
                }
            }
            Some(AddressingMode::DirectAbsolute) => {
                // In absolute addressing, the second byte of the instruction specifies the eight low order bits of the effective address while the third byte specifies the eight high order bits. Thus, the absolute addressing mode allows access to the entire 65 K bytes of addressable memory.
                match (state.consume_byte(), state.consume_byte()) {
                    (Some(low_byte), Some(high_byte)) => {
                        let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                        address
                    }
                    _ => return Err(()),
                }
            }
            None => 0,
            _ => return Err(()),
        };

        match self.opcode {
            OpCode::ADC => {
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
            },
            OpCode::AND => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };
                state.a = argument & state.a;
                state.flags.set_zero_flag(state.a == 0);
                state
                    .flags
                    .set_negative_flag((state.a & 0b10000000) == 0b10000000)
            },
            OpCode::ASL => {
                let location = match self.mode {
                    Some(AddressingMode::Accumulator) => state.a,
                    _ => match state.fetch_memory(argument.into()) {
                        Ok(a) => a,
                        Err(_) => return Err(()),
                    },
                };
                let (value, overflow) = location.overflowing_shl(1);

                match self.mode {
                    Some(AddressingMode::Accumulator) => {
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
            },
            OpCode::BCC => {
                if state.flags.carry_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BCS => {
                if state.flags.carry_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BEQ => {
                if state.flags.zero_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BIT => {
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
            OpCode::BMI => {
                if state.flags.negative_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BNE => {
                if state.flags.zero_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BPL => {
                if state.flags.negative_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BVC => {
                if state.flags.overflow_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::BVS => {
                if state.flags.overflow_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.pc = state.pc.wrapping_add(argument as usize);
                    } else {
                        state.pc = state.pc.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            OpCode::CLC => {
                state.flags.set_carry_flag(false);
            },
            OpCode::CLD => {
                state.flags.set_decimal_flag(false);
            },
            OpCode::CLI => {
                state.flags.set_interrupt_disable_flag(false);
            },
            OpCode::CLV => {
                state.flags.set_overflow_flag(false);
            },
            OpCode::CMP => {
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
            OpCode::CPX => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };
                let result = state.x - argument;
                state.flags.set_zero_flag(result == 0);
                state.flags.set_carry_flag(state.x >= argument);
                state
                    .flags
                    .set_negative_flag((argument & 0b10000000) == 0b10000000)
            },
            OpCode::CPY => {
                let argument = match state.fetch_memory(argument.into()) {
                    Ok(argument) => argument,
                    Err(_) => return Err(()),
                };
                let result = state.y - argument;
                state.flags.set_zero_flag(result == 0);
                state.flags.set_carry_flag(state.y >= argument);
                state
                    .flags
                    .set_negative_flag((argument & 0b10000000) == 0b10000000)
            },
            OpCode::DEC => {
                let m = match state.fetch_memory(argument.into()) {
                    Ok(m) => m,
                    Err(_) => return Err(()),
                };

                let (value, _) = m.overflowing_sub(1);

                match state.write_memory(argument.into(), value) {
                    Err(_) => return Err(()),
                    _ => (),
                };

                state.flags.set_zero_flag(value == 0);
                state
                    .flags
                    .set_negative_flag((value & 0b10000000) == 0b10000000);
            },
            OpCode::DEX => {
                state.y = state.x.overflowing_sub(1).0;
                state.flags.set_zero_flag(state.x == 0);
                state
                    .flags
                    .set_negative_flag((state.x & 0b10000000) == 0b10000000);
            },
            OpCode::DEY => {
                state.y = state.x.overflowing_sub(1).0;
                state.flags.set_zero_flag(state.y == 0);
                state
                    .flags
                    .set_negative_flag((state.y & 0b10000000) == 0b10000000);
            },
            OpCode::EOR => {
                let m = match state.fetch_memory(argument.into()) {
                    Ok(m) => m,
                    Err(_) => return Err(()),
                };
                state.a = state.a ^ m;
                state.flags.set_zero_flag(state.a == 0);
                state
                    .flags
                    .set_negative_flag((state.a & 0b10000000) == 0b10000000);
            },
            OpCode::INC => {
                let m = match state.fetch_memory(argument.into()) {
                    Ok(m) => m,
                    Err(_) => return Err(()),
                };

                let (value, _) = m.overflowing_add(1);

                match state.write_memory(argument.into(), value) {
                    Err(_) => return Err(()),
                    _ => (),
                };

                state.flags.set_zero_flag(value == 0);
                state
                    .flags
                    .set_negative_flag((value & 0b10000000) == 0b10000000);
            },
            OpCode::INX => {
                let (value, _) = state.x.overflowing_add(1);

                state.flags.set_zero_flag(value == 0);
                state
                    .flags
                    .set_negative_flag((value & 0b10000000) == 0b10000000);
            },
            OpCode::INY => {
                let (value, _) = state.y.overflowing_add(1);

                state.flags.set_zero_flag(value == 0);
                state
                    .flags
                    .set_negative_flag((value & 0b10000000) == 0b10000000);
            },
            OpCode::JMP => { // this should be 4c
                state.pc = argument.into();
            },
            OpCode::JSR => {
                let low_byte = (state.pc & 0xFF) as u8;
                let high_byte = (state.pc.overflowing_shr(8).0 & 0xFF) as u8;

                let first_write = state.write_memory(state.sp.into(), high_byte);
                let second_write = state.write_memory(state.sp.into(), low_byte);
                match (first_write, second_write) {
                    (Ok(_), Ok(_)) => (),
                    _ => return Err(()),
                };
                
                state.pc = argument.into();
            },
            OpCode::LDA => {
                state.a = argument as u8;
                state.flags.set_zero_flag(state.a == 0);
                state
                    .flags
                    .set_negative_flag((state.a & 0b10000000) == 0b10000000);
            },
            OpCode::LDX => {
                state.x = argument as u8;
                state.flags.set_zero_flag(state.x == 0);
                state
                    .flags
                    .set_negative_flag((state.x & 0b01000000) == 0b01000000);
            },
            OpCode::LDY => {
                state.y = argument as u8;
                state.flags.set_zero_flag(state.y == 0);
                state
                    .flags
                    .set_negative_flag((state.y & 0b01000000) == 0b01000000);
            },
            OpCode::LSR => {
                let location = match self.mode {
                    Some(AddressingMode::Accumulator) => state.a,
                    _ => match state.fetch_memory(argument.into()) {
                        Ok(a) => a,
                        Err(_) => return Err(()),
                    },
                };
                let (value, overflow) = location.overflowing_shr(1);

                match self.mode {
                    Some(AddressingMode::Accumulator) => {
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
            },
            OpCode::NOP => (),
            
            OpCode::SEI => {
                state.flags.set_interrupt_disable_flag(true);
            },
            OpCode::STA => {
                let _ = state.write_memory(argument.into(), state.a);
            },
            OpCode::TXS => {
                state.s = state.x;
            },
            
            
            _ => return Err(()),
        }
        // state.print_registers();
        Ok(())
    }
}
