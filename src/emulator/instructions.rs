
use crate::emulator::state::State;

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
    Relative
}


#[derive(Debug)]
pub enum Instruction {
    GroupMultipleByte(MultipleByteInstruction, AddressingMode),
    GroupSingleByte(SingleByteInstruction),
}
#[derive(Debug)]
pub enum MultipleByteInstruction {
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
    PHP = 0x08,
    PLP = 0x28,
    PHA = 0x48,
    PLA = 0x68,
    DEY = 0x88,
    TAY = 0xA8,
    INY = 0xC8,
    INX = 0xE8,
    CLC = 0x18,
    SEC = 0x38,
    CLI = 0x58,
    SEI = 0x78,
    TYA = 0x98,
    CLV = 0xB8,
    CLD = 0xD8,
    SED = 0xF8,
    TXA = 0x8A,
    TXS = 0x9A,
    TAX = 0xAA,
    TSX = 0xBA,
    DEX = 0xCA,
    NOP = 0xEA,
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
            0x10 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BPL, AddressingMode::Relative)),
            0x30 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BMI, AddressingMode::Relative)),
            0x50 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BVC, AddressingMode::Relative)),
            0x70 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BVS, AddressingMode::Relative)),
            0x90 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BCC, AddressingMode::Relative)),
            0xB0 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BCS, AddressingMode::Relative)),
            0xD0 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BNE, AddressingMode::Relative)),
            0xF0 => return Ok(Instruction::GroupMultipleByte(MultipleByteInstruction::BEQ, AddressingMode::Relative)),
            _ => (),
        };    

        match group_bits {
            0b01 => {
                let instruction = match instruction_bits {
                    0b000 => MultipleByteInstruction::ORA,
                    0b001 => MultipleByteInstruction::AND,
                    0b010 => MultipleByteInstruction::EOR,
                    0b011 => MultipleByteInstruction::ADC,
                    0b100 => MultipleByteInstruction::STA,
                    0b101 => MultipleByteInstruction::LDA,
                    0b110 => MultipleByteInstruction::CMP,
                    0b111 => MultipleByteInstruction::SBC,
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
            },
            0b10 => {
                let instruction = match instruction_bits {
                    0b000 => MultipleByteInstruction::ASL,
                    0b001 => MultipleByteInstruction::ROL,
                    0b010 => MultipleByteInstruction::LSR,
                    0b011 => MultipleByteInstruction::ROR,
                    0b100 => MultipleByteInstruction::STX,
                    0b101 => MultipleByteInstruction::LDX,
                    0b110 => MultipleByteInstruction::DEC,
                    0b111 => MultipleByteInstruction::INC,
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
            },
            0b00 => {
                let instruction = match instruction_bits {
                    0b001 => MultipleByteInstruction::BIT,
                    0b010 => MultipleByteInstruction::JMP,
                    0b011 => MultipleByteInstruction::JMPAbsolute,
                    0b100 => MultipleByteInstruction::STY,
                    0b101 => MultipleByteInstruction::LDY,
                    0b110 => MultipleByteInstruction::CPY,
                    0b111 => MultipleByteInstruction::CPX,
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
    pub fn execute<'a>(&self, state: &mut State) -> Result<(), ()> {
        
        let argument: u16 = match *self {
            Instruction::GroupMultipleByte(_, AddressingMode::Immediate) | Instruction::GroupMultipleByte(_, AddressingMode::Relative) => match state.consume_byte() {
                Some(argument) => argument.into(),
                _ => return Err(()),
            },
            Instruction::GroupMultipleByte(_, AddressingMode::DirectZeroPage) => {
                match state.consume_byte() {
                    Some(argument) => argument.into(),
                    _ => return Err(()),
                }
            },
            Instruction::GroupMultipleByte(_, AddressingMode::DirectZeroPageX) => {
                match state.consume_byte() {
                    Some(byte) => byte.overflowing_add(state.register_x).0.into(),
                    _ => return Err(()),
                }
            },
            Instruction::GroupMultipleByte(_, AddressingMode::DirectAbsolute) => {
                // In absolute addressing, the second byte of the instruction specifies the eight low order bits of the effective address while the third byte specifies the eight high order bits. Thus, the absolute addressing mode allows access to the entire 65 K bytes of addressable memory.
                match (state.consume_byte(), state.consume_byte()) {
                    (Some(low_byte), Some(high_byte)) => {
                        let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                        address
                    }
                    _ => return Err(()),
                }
            },
            Instruction::GroupSingleByte(_) => 0,
            _ => return Err(())
        };

        match *self {
            Instruction::GroupMultipleByte(ref instruction, _) => {
                println!("{:?} {:#02x}", instruction, argument);
            }
            Instruction::GroupSingleByte(ref instruction) => {
                println!("{:?}", instruction);
            }
        };

        match *self {
            Instruction::GroupSingleByte(ref instruction) => match instruction {
                SingleByteInstruction::SEI => {
                    state.status_flags.set_interrupt_disable_flag(true);
                },
                SingleByteInstruction::CLD => {
                    state.status_flags.set_decimal_flag(false);
                },
                SingleByteInstruction::TXS => {
                    state.register_s = state.register_x;
                },
                SingleByteInstruction::DEX => {
                    state.register_x = state.register_x.overflowing_add(255).0;
                    state.status_flags.set_zero_flag(state.register_x == 0);
                    state
                        .status_flags
                        .set_negative_flag((state.register_x & 0b01000000) == 0b01000000);
                },
                _ => return Err(()),
            },
            Instruction::GroupMultipleByte(MultipleByteInstruction::LDA,_) => {
                state.register_a = argument as u8;
            },
            Instruction::GroupMultipleByte(MultipleByteInstruction::LDX,_) => {
                state.register_x = argument as u8;
                state.status_flags.set_zero_flag(argument == 0);
                state
                    .status_flags
                    .set_negative_flag((argument & 0b01000000) == 0b01000000);

            },
            Instruction::GroupMultipleByte(MultipleByteInstruction::STA,_) => {
                let _ = state.write_memory(argument.into(), state.register_a);
            },
            Instruction::GroupMultipleByte(MultipleByteInstruction::JMP,_) => {
                state.program_counter = argument.into();
            }
            Instruction::GroupMultipleByte(MultipleByteInstruction::BNE, AddressingMode::Relative) => {
                if state.status_flags.zero_flag() == false {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.program_counter = state.program_counter.wrapping_add(argument as usize);
                    } else {
                        state.program_counter = state.program_counter.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultipleByteInstruction::BEQ, AddressingMode::Relative) => {
                if state.status_flags.zero_flag() {
                    let argument = argument as i8; // Convert back to i8 to handle negatives correctly
                    if argument >= 0 {
                        state.program_counter = state.program_counter.wrapping_add(argument as usize);
                    } else {
                        state.program_counter = state.program_counter.wrapping_sub(argument.abs() as usize);
                    }
                }
            },
            Instruction::GroupMultipleByte(MultipleByteInstruction::ADC,_) => {
                let (argument, overflowing) = match state.status_flags.overflow_flag() {
                    true => argument.overflowing_add(1),
                    false => (argument, false),
                };

                let (argument, overflowing) =
                    match state.register_a.overflowing_add(argument as u8) {
                        (value, second_overflowing) => {
                            (value, overflowing || second_overflowing)
                        }
                    };

                state.register_a = argument;
                state.status_flags.set_overflow_flag(overflowing);
                state.status_flags.set_zero_flag(argument == 0);
                state
                    .status_flags
                    .set_negative_flag((argument & 0b01000000) == 0b01000000);
            },
            _ => return Err(()),
        }
        state.print_registers();
        Ok(())
    }
}