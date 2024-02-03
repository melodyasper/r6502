pub enum InstructionGroup {
    GroupOne,
    GroupTwo,
    GroupThree,
    GroupFour,
}

impl TryFrom<u8> for InstructionGroup {
    type Error = ();

    fn try_from(value: u8) -> Result<InstructionGroup, Self::Error> {
        match value {
            0b01 => Ok(InstructionGroup::GroupOne),
            0b10 => Ok(InstructionGroup::GroupTwo),
            0b00 => Ok(InstructionGroup::GroupThree),
            0b11 => Ok(InstructionGroup::GroupFour),
            _ => Err(()),
        }
    }
}


enum GroupOneInstruction {
    ORA = 0b000,	
    AND = 0b001,	
    EOR = 0b010,	
    ADC = 0b011,	
    STA = 0b100,	
    LDA = 0b101,	
    CMP = 0b110,	
    SBC = 0b111,	
}

// enum GroupOneAddressingMode {
// (zero page,X) = 0b000,
// zero page = 0b001,
// #immediate = 0b010,
// absolute = 0b011,
// (zero page),Y = 0b100,
// zero page,X = 0b101,
// absolute,Y = 0b110,
// absolute,X = 0b111,
// }


fn main() {
    // Instructions from https://codeburst.io/an-introduction-to-6502-assembly-and-low-level-programming-7c11fa6b9cb9
    // LDA   $60
    // ADC   $61
    // STA   $62

    // https://llx.com/Neil/a2/opcodes.html
    let instructions: [u8; 6] = [0xA5, 0x60, 0x65, 0x61, 0x85, 0x62];
    let mut program_counter = 0;
    while program_counter < instructions.len() {
        let pprogram_counter_value = instructions.get(program_counter).unwrap();
        let group = InstructionGroup::try_from(pprogram_counter_value & 0b11);
        match group {
            Ok(InstructionGroup::GroupOne) => println!("Instruction group one"),
            Ok(InstructionGroup::GroupTwo) => println!("Instruction group two"),
            Ok(InstructionGroup::GroupThree) => println!("Instruction group three"),
            Ok(InstructionGroup::GroupFour) => println!("Instruction group four"),
            _ => println!("Unknown instruction group"),
        }
        println!("{:?}", pprogram_counter_value);
        program_counter += 1;
    }

    println!("Hello, world!");
}
