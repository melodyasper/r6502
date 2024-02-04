#[derive(Debug)]
enum Instruction {
    GroupOne(GroupOneMode),
    GroupTwo(GroupTwoMode),
    GroupThree(GroupThreeMode),
    GroupSingleByte(SingleByteInstruction),
}

#[derive(Debug)]
enum GroupOneMode {
    IndirectZeroPageX(GroupOneInstruction), // 0b000; (zero page,X)
    DirectZeroPage(GroupOneInstruction),    // 0b001; zero page
    Immediate(GroupOneInstruction),         // 0b010; #immediate
    DirectAbsolute(GroupOneInstruction),    // 0b011; absolute
    IndirectZeroPageY(GroupOneInstruction), // 0b100; (zero page),Y
    DirectZeroPageX(GroupOneInstruction),   // 0b101; zero page,X
    DirectAbsoluteY(GroupOneInstruction),   // 0b110; absolute,Y
    DirectAbsoluteX(GroupOneInstruction),   // 0b111; absolute,X
}
#[derive(Debug)]
enum GroupOneInstruction {
    ORA,
    AND,
    EOR,
    ADC,
    STA,
    LDA,
    CMP,
    SBC,
}

#[derive(Debug)]
enum GroupTwoMode {
    Immediate(GroupTwoInstruction),       // 0b000; #immediate
    DirectZeroPage(GroupTwoInstruction),  // 0b001; zero page
    Accumulator(GroupTwoInstruction),     // 0b010; accumulator
    DirectAbsolute(GroupTwoInstruction),  // 0b011; absolute
    DirectZeroPageX(GroupTwoInstruction), // 0b101; zero page,X
    DirectAbsoluteX(GroupTwoInstruction), // 0b111; absolute,X
}

#[derive(Debug)]
enum GroupTwoInstruction {
    ASL,
    ROL,
    LSR,
    ROR,
    STX,
    LDX,
    DEC,
    INC,
}

#[derive(Debug)]
enum GroupThreeMode {
    Immediate(GroupThreeInstruction),       // 0b000; #immediate
    DirectZeroPage(GroupThreeInstruction),  // 0b001; zero page
    DirectAbsolute(GroupThreeInstruction),  // 0b011; absolute
    DirectZeroPageX(GroupThreeInstruction), // 0b101; zero page,X
    DirectAbsoluteX(GroupThreeInstruction), // 0b111; absolute,X
}

#[repr(u8)]
#[derive(Debug)]
enum SingleByteInstruction {
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

impl TryFrom<u8> for SingleByteInstruction {
    type Error = ();
    fn try_from(value: u8) -> Result<SingleByteInstruction, Self::Error> {
        match value {
            0x08 => Ok(SingleByteInstruction::PHP),
            0x28 => Ok(SingleByteInstruction::PLP),
            0x48 => Ok(SingleByteInstruction::PHA),
            0x68 => Ok(SingleByteInstruction::PLA),
            0x88 => Ok(SingleByteInstruction::DEY),
            0xA8 => Ok(SingleByteInstruction::TAY),
            0xC8 => Ok(SingleByteInstruction::INY),
            0xE8 => Ok(SingleByteInstruction::INX),
            0x18 => Ok(SingleByteInstruction::CLC),
            0x38 => Ok(SingleByteInstruction::SEC),
            0x58 => Ok(SingleByteInstruction::CLI),
            0x78 => Ok(SingleByteInstruction::SEI),
            0x98 => Ok(SingleByteInstruction::TYA),
            0xB8 => Ok(SingleByteInstruction::CLV),
            0xD8 => Ok(SingleByteInstruction::CLD),
            0xF8 => Ok(SingleByteInstruction::SED),
            0x8A => Ok(SingleByteInstruction::TXA),
            0x9A => Ok(SingleByteInstruction::TXS),
            0xAA => Ok(SingleByteInstruction::TAX),
            0xBA => Ok(SingleByteInstruction::TSX),
            0xCA => Ok(SingleByteInstruction::DEX),
            0xEA => Ok(SingleByteInstruction::NOP),
            _ => Err(())
        }
    }
}
#[derive(Debug)]
enum GroupThreeInstruction {
    BIT,         // 001
    JMP,         // 010
    JMPAbsolute, // 011
    STY,         // 100
    LDY,         // 101
    CPY,         // 110
    CPX,         // 111
}

impl TryFrom<u8> for GroupOneMode {
    type Error = ();

    fn try_from(value: u8) -> Result<GroupOneMode, Self::Error> {
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        let instruction = match instruction_bits {
            0b000 => GroupOneInstruction::ORA,
            0b001 => GroupOneInstruction::AND,
            0b010 => GroupOneInstruction::EOR,
            0b011 => GroupOneInstruction::ADC,
            0b100 => GroupOneInstruction::STA,
            0b101 => GroupOneInstruction::LDA,
            0b110 => GroupOneInstruction::CMP,
            0b111 => GroupOneInstruction::SBC,
            _ => return Err(()),
        };

         match mode_bits {
            0b000 => Ok(GroupOneMode::IndirectZeroPageX(instruction)),
            0b001 => Ok(GroupOneMode::DirectZeroPage(instruction)),
            0b010 => Ok(GroupOneMode::Immediate(instruction)),
            0b011 => Ok(GroupOneMode::DirectAbsolute(instruction)),
            0b100 => Ok(GroupOneMode::IndirectZeroPageY(instruction)),
            0b101 => Ok(GroupOneMode::DirectZeroPageX(instruction)),
            0b110 => Ok(GroupOneMode::DirectAbsoluteY(instruction)),
            0b111 => Ok(GroupOneMode::DirectAbsoluteX(instruction)),
            _ => return Err(()),
        }

    }
}

impl TryFrom<u8> for GroupTwoMode {
    type Error = ();

    fn try_from(value: u8) -> Result<GroupTwoMode, Self::Error> {
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;
        let instruction = match instruction_bits {
            0b000 => GroupTwoInstruction::ASL,
            0b001 => GroupTwoInstruction::ROL,
            0b010 => GroupTwoInstruction::LSR,
            0b011 => GroupTwoInstruction::ROR,
            0b100 => GroupTwoInstruction::STX,
            0b101 => GroupTwoInstruction::LDX,
            0b110 => GroupTwoInstruction::DEC,
            0b111 => GroupTwoInstruction::INC,
            _ => return Err(()),
        };

        match mode_bits {
            0b000 => Ok(GroupTwoMode::Immediate(instruction)),
            0b001 => Ok(GroupTwoMode::DirectZeroPage(instruction)),
            0b010 => Ok(GroupTwoMode::Accumulator(instruction)),
            0b011 => Ok(GroupTwoMode::DirectAbsolute(instruction)),
            0b101 => Ok(GroupTwoMode::DirectZeroPageX(instruction)),
            0b111 => Ok(GroupTwoMode::DirectAbsoluteX(instruction)),
            _ => Err(()),
        }

        
    }
}

impl TryFrom<u8> for GroupThreeMode {
    type Error = ();

    fn try_from(value: u8) -> Result<GroupThreeMode, Self::Error> {
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        let instruction = match instruction_bits {
            0b001 => GroupThreeInstruction::BIT,
            0b010 => GroupThreeInstruction::JMP,
            0b011 => GroupThreeInstruction::JMPAbsolute,
            0b100 => GroupThreeInstruction::STY,
            0b101 => GroupThreeInstruction::LDY,
            0b110 => GroupThreeInstruction::CPY,
            0b111 => GroupThreeInstruction::CPX,
            _ => return Err(()),
        };

        match mode_bits {
            0b000 => Ok(GroupThreeMode::Immediate(instruction)),
            0b001 => Ok(GroupThreeMode::DirectZeroPage(instruction)),
            0b011 => Ok(GroupThreeMode::DirectAbsolute(instruction)),
            0b101 => Ok(GroupThreeMode::DirectZeroPageX(instruction)),
            0b111 => Ok(GroupThreeMode::DirectAbsoluteX(instruction)),
            _ => Err(()),
        }

    
    }
}

#[derive(Debug)]
struct StatusFlags {
    value: u8,
}
macro_rules! create_status_flag {
    ($name:ident, $value:expr) => {
        ::paste::paste! {
            fn [< $name _flag >] (&self) -> bool {
                (self.value & $value) != 0
            }
            fn [<set_ $name _flag >](&mut self, set: bool) {
                if set {
                    self.value |= $value;
                } else {
                    self.value &= !$value;
                }
            }
        }
    }
}

impl StatusFlags {
    fn new(value: u8) -> Self {
        StatusFlags { value }
    }

    create_status_flag!(negative, 7);
    create_status_flag!(overflow, 6);
    create_status_flag!(expansion, 5);
    create_status_flag!(break_command, 4);
    create_status_flag!(decimal, 3);
    create_status_flag!(interrupt_disable, 2);
    create_status_flag!(zero, 1);
    create_status_flag!(carry, 0);

    // You can add more getters and setters for other bits following the pattern above.
}


#[derive(Debug)]
struct State {
    running: bool,
    program_counter: usize,
    memory: Vec<u8>,
    register_a: u8,
    register_x: u8,
    register_y: u8,
    register_s: u8,
    register_p: u8,
    status_flags: StatusFlags,    
}
impl State {
    fn get_next_instruction(&mut self) -> Option<Instruction> {
        match self.consume_byte() {
            Some(value) => match Instruction::try_from(value) {
                Ok(next_instruction) => Some(next_instruction),
                Err(_) => None
            },
            None => {
                self.running = false;
                None
            }
        }
    }
    fn consume_byte(&mut self) -> Option<u8> {
        let program_counter = self.program_counter;
        self.program_counter += 1;
        // TODO: Can't use `fetch_memory` here until we fix our little hack in it.
        // Otherwise program space will grow indefinitely.
        match self.memory.get(program_counter) {
            Some(value) => Some(*value),
            None => None
        }
    }
    fn fetch_memory(&mut self, address: usize) -> Result<u8, ()> {
        println!("Read from memory @ {}", address);
        let length = self.memory.len();
        if length < address {
            // TODO: Remove this hack.
            self.memory.resize(length, 0);
        }
        match self.memory.get(address) {
            Some(value) => Ok(*value),
            None => Err(())
        }
    }
}

impl Instruction {
    fn execute<'a>(&self, state: &mut State) -> Result<(), ()>{
        let (instruction, argument) = match *self {
            Instruction::GroupOne(GroupOneMode::DirectZeroPage(ref instruction)) => {
                // When instruction LDA is executed by the microprocessor, data is transferred from memory to the accumulator and stored in the accumulator.
                match state.consume_byte() {
                    Some(byte) => match state.fetch_memory(byte.into()) {
                        Ok(argument) => (instruction, argument),
                        _ => return Err(())
                    }
                    _ => return Err(())
                }
            },
            Instruction::GroupOne(GroupOneMode::DirectAbsolute(ref instruction)) => {
                // In absolute addressing, the second byte of the instruction specifies the eight low order bits of the effective address while the third byte specifies the eight high order bits. Thus, the absolute addressing mode allows access to the entire 65 K bytes of addressable memory.
                 match (state.consume_byte(), state.consume_byte()) {
                    (Some(low_byte), Some(high_byte)) => {
                        let address: u16 = ((high_byte as u16) << 8) + low_byte as u16;
                        match state.fetch_memory(address.into()) {
                            Ok(argument) => (instruction, argument),
                            _ => return Err(())
                        }
                    },
                    _ => return Err(())
                }
            },
            _ => {
                println!("Unimplemented.");
                return Ok(())
            }
        };
        
        match instruction {
            GroupOneInstruction::LDA => {
                state.register_a = argument;
            }
            GroupOneInstruction::ADC => {
                
                
                let (argument, overflowing) = match state.status_flags.overflow_flag() {
                    true => argument.overflowing_add(1),
                    false => (argument, false)
                };

                
                let (argument, overflowing) = match state.register_a.overflowing_add(argument) {
                    (value, second_overflowing) => {
                        (value, overflowing || second_overflowing)
                    }
                };

                state.register_a = argument;
                state.status_flags.set_overflow_flag(overflowing);
                state.status_flags.set_zero_flag(argument == 0);
                state.status_flags.set_negative_flag((argument & 0b01000000) == 0b01000000);

            }
            _ => {
                println!("Instruction {:?} implemented yet", instruction);
            }
        }

        Ok(())
    }
}

impl TryFrom<u8> for Instruction {
    type Error = ();

    fn try_from(value: u8) -> Result<Instruction, Self::Error> {
        // This needs a carve out as a special exception
        match SingleByteInstruction::try_from(value) {
            Ok(instruction) => return Ok(Instruction::GroupSingleByte(instruction)),
            Err(_) => ()
        };

        let group_bits = value & 0b11;
        match group_bits {
            0b01 => Ok(Instruction::GroupOne(GroupOneMode::try_from(value)?)),
            0b10 => Ok(Instruction::GroupTwo(GroupTwoMode::try_from(value)?)),
            0b00 => Ok(Instruction::GroupThree(GroupThreeMode::try_from(
                value,
            )?)),
            _ => Err(()),
        }
    }
}

// 0xA5 = 10100101 ( 165 )
// aaabbbcc. The aaa and cc bits determine the opcode, and the bbb bits determine the addressing mode.
// bbb = 001
// cc = 01
// aaa = 101

// bbb	addressing mode
// 000	(zero page,X)
// 001	zero page
// 010	#immediate
// 011	absolute
// 100	(zero page),Y
// 101	zero page,X
// 110	absolute,Y
// 111	absolute,X

fn main() {
    // Instructions from https://codeburst.io/an-introduction-to-6502-assembly-and-low-level-programming-7c11fa6b9cb9
    // LDA   $60
    // ADC   $61
    // STA   $62

    // https://llx.com/Neil/a2/opcodes.html
    let mut state = State {
        running: true,
        program_counter: 0,
        // memory: vec![0xA5, 0x60, 0x65, 0x61, 0x85, 0x62],
        memory: vec![0xA8],
        register_a: 0,
        register_x: 0,
        register_y: 0,
        register_s: 0,
        register_p: 0,
        status_flags: StatusFlags::new(0),
        
    };
    
    // state.memory.resize(256, 0xAA);
    while state.running {
        match state.get_next_instruction() {
            Some(instruction) => {
                println!("Found instruction {:?}", instruction);
                instruction.execute(&mut state);
            }
            None => {
                println!("Unknown instruction");
            }
        }
    }
    // println!("{:?}", state)
}
