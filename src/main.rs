use std::{error, fmt::Error};

#[derive(Debug)]
enum Instruction {
    GroupOne(GroupOneInstruction),
    GroupTwo(GroupTwoInstruction),
    GroupThree(GroupThreeInstruction),
}

#[derive(Debug)]
enum GroupOneMode {
    IndirectZeroPageX, // 0b000; (zero page,X)
    DirectZeroPage,    // 0b001; zero page
    Immediate,         // 0b010; #immediate
    DirectAbsolute,    // 0b011; absolute
    IndirectZeroPageY, // 0b100; (zero page),Y
    DirectZeroPageX,   // 0b101; zero page,X
    DirectAbsoluteY,   // 0b110; absolute,Y
    DirectAbsoluteX,   // 0b111; absolute,X
}
#[derive(Debug)]
enum GroupOneInstruction {
    ORA(GroupOneMode),
    AND(GroupOneMode),
    EOR(GroupOneMode),
    ADC(GroupOneMode),
    STA(GroupOneMode),
    LDA(GroupOneMode),
    CMP(GroupOneMode),
    SBC(GroupOneMode),
}
trait Executable {
    fn execute();
}

impl GroupOneInstruction {
    fn move_parser(&self) -> usize {
        match *self {
            GroupOneInstruction::ORA(_) => 1,
            GroupOneInstruction::AND(_) => 1,
            GroupOneInstruction::EOR(_) => 1,
            GroupOneInstruction::ADC(_) => 1,
            GroupOneInstruction::STA(_) => 1,
            GroupOneInstruction::LDA(_) => 1,
            GroupOneInstruction::CMP(_) => 1,
            GroupOneInstruction::SBC(_) => 1,
        }
    }
}

#[derive(Debug)]
enum GroupTwoMode {
    Immediate,       // 0b000; #immediate
    DirectZeroPage,  // 0b001; zero page
    Accumulator,     // 0b010; accumulator
    DirectAbsolute,  // 0b011; absolute
    DirectZeroPageX, // 0b101; zero page,X
    DirectAbsoluteX, // 0b111; absolute,X
}

#[derive(Debug)]
enum GroupTwoInstruction {
    ASL(GroupTwoMode),
    ROL(GroupTwoMode),
    LSR(GroupTwoMode),
    ROR(GroupTwoMode),
    STX(GroupTwoMode),
    LDX(GroupTwoMode),
    DEC(GroupTwoMode),
    INC(GroupTwoMode),
}

#[derive(Debug)]
enum GroupThreeMode {
    Immediate,       // 0b000; #immediate
    DirectZeroPage,  // 0b001; zero page
    DirectAbsolute,  // 0b011; absolute
    DirectZeroPageX, // 0b101; zero page,X
    DirectAbsoluteX, // 0b111; absolute,X
}

#[derive(Debug)]
enum GroupThreeInstruction {
    BIT(GroupThreeMode),         // 001
    JMP(GroupThreeMode),         // 010
    JMPAbsolute(GroupThreeMode), // 011
    STY(GroupThreeMode),         // 100
    LDY(GroupThreeMode),         // 101
    CPY(GroupThreeMode),         // 110
    CPX(GroupThreeMode),         // 111
}

impl TryFrom<u8> for GroupOneInstruction {
    type Error = ();

    fn try_from(value: u8) -> Result<GroupOneInstruction, Self::Error> {
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        let mode = match mode_bits {
            0b000 => GroupOneMode::IndirectZeroPageX,
            0b001 => GroupOneMode::DirectZeroPage,
            0b010 => GroupOneMode::Immediate,
            0b011 => GroupOneMode::DirectAbsolute,
            0b100 => GroupOneMode::IndirectZeroPageY,
            0b101 => GroupOneMode::DirectZeroPageX,
            0b110 => GroupOneMode::DirectAbsoluteY,
            0b111 => GroupOneMode::DirectAbsoluteX,
            _ => return Err(()),
        };

        match instruction_bits {
            0b000 => Ok(GroupOneInstruction::ORA(mode)),
            0b001 => Ok(GroupOneInstruction::AND(mode)),
            0b010 => Ok(GroupOneInstruction::EOR(mode)),
            0b011 => Ok(GroupOneInstruction::ADC(mode)),
            0b100 => Ok(GroupOneInstruction::STA(mode)),
            0b101 => Ok(GroupOneInstruction::LDA(mode)),
            0b110 => Ok(GroupOneInstruction::CMP(mode)),
            0b111 => Ok(GroupOneInstruction::SBC(mode)),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for GroupTwoInstruction {
    type Error = ();

    fn try_from(value: u8) -> Result<GroupTwoInstruction, Self::Error> {
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        let mode = match mode_bits {
            0b000 => GroupTwoMode::Immediate,
            0b001 => GroupTwoMode::DirectZeroPage,
            0b010 => GroupTwoMode::Accumulator,
            0b011 => GroupTwoMode::DirectAbsolute,
            0b101 => GroupTwoMode::DirectZeroPageX,
            0b111 => GroupTwoMode::DirectAbsoluteX,
            _ => return Err(()),
        };

        match instruction_bits {
            0b000 => Ok(GroupTwoInstruction::ASL(mode)),
            0b001 => Ok(GroupTwoInstruction::ROL(mode)),
            0b010 => Ok(GroupTwoInstruction::LSR(mode)),
            0b011 => Ok(GroupTwoInstruction::ROR(mode)),
            0b100 => Ok(GroupTwoInstruction::STX(mode)),
            0b101 => Ok(GroupTwoInstruction::LDX(mode)),
            0b110 => Ok(GroupTwoInstruction::DEC(mode)),
            0b111 => Ok(GroupTwoInstruction::INC(mode)),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for GroupThreeInstruction {
    type Error = ();

    fn try_from(value: u8) -> Result<GroupThreeInstruction, Self::Error> {
        let instruction_bits = (0b11100000 & value) >> 5;
        let mode_bits = (0b00011100 & value) >> 2;

        let mode = match mode_bits {
            0b000 => GroupThreeMode::Immediate,
            0b001 => GroupThreeMode::DirectZeroPage,
            0b011 => GroupThreeMode::DirectAbsolute,
            0b101 => GroupThreeMode::DirectZeroPageX,
            0b111 => GroupThreeMode::DirectAbsoluteX,
            _ => return Err(()),
        };

        match instruction_bits {
            0b001 => Ok(GroupThreeInstruction::BIT(mode)),
            0b010 => Ok(GroupThreeInstruction::JMP(mode)),
            0b011 => Ok(GroupThreeInstruction::JMPAbsolute(mode)),
            0b100 => Ok(GroupThreeInstruction::STY(mode)),
            0b101 => Ok(GroupThreeInstruction::LDY(mode)),
            0b110 => Ok(GroupThreeInstruction::CPY(mode)),
            0b111 => Ok(GroupThreeInstruction::CPX(mode)),
            _ => Err(()),
        }
    }
}

struct State {
    running: bool,
    program_counter: usize,
    memory: Vec<u8>,
}
impl State {
    fn get_next_instruction(&mut self) -> Option<Instruction> {
        let program_counter = self.program_counter;
        self.program_counter += 1;
        match self.memory.get(program_counter) {
            Some(value) => match Instruction::try_from(*value) {
                Ok(next_instruction) => Some(next_instruction),
                Err(_) => None
            },
            None => {
                self.running = false;
                None
            }
        }
    }
}

impl Instruction {
    fn execute<'a>(&self, state: State) -> State {
        return state;
    }
}

impl TryFrom<u8> for Instruction {
    type Error = ();

    fn try_from(value: u8) -> Result<Instruction, Self::Error> {
        let group_bits = value & 0b11;
        match group_bits {
            0b01 => Ok(Instruction::GroupOne(GroupOneInstruction::try_from(value)?)),
            0b10 => Ok(Instruction::GroupTwo(GroupTwoInstruction::try_from(value)?)),
            0b00 => Ok(Instruction::GroupThree(GroupThreeInstruction::try_from(
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
    let mut program_counter = 0;
    let mut state = State {
        running: true,
        program_counter: 0,
        memory: vec![0xA5, 0x60, 0x65, 0x61, 0x85, 0x62],
    };
    while state.running {
        match state.get_next_instruction() {
            Some(instruction) => {
                println!("Found instruction {:?}", instruction);
                state = instruction.execute(state);
            }
            None => {
                println!("Program finished");
            }
        }
    }

    println!("Hello, world!");
}
