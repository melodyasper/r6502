mod emulator;
use crate::emulator::state::{State, StatusFlags};
use std::{fmt::Arguments, ops::Add};




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
        memory: vec![
            0x78, 0xd8, 0xa2, 0xff, 0x9a, 0xa9, 0x00, 0x95, 0x00, 0xca, 0xd0, 0xfb, 0x85, 0x00,
            0xa9, 0x30, 0x85, 0x09, 0x4c, 0x00, 0xf0, 0x00, 0xf0, 0x00, 0xf0,
        ],
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
                // println!("{:?} | Executing", instruction);
                match instruction.execute(&mut state) {
                    Ok(_) => (),
                    _ => {
                        println!("Failed to execute instruction {:?}", instruction);
                    }
                }
            }
            None => {
                // println!("Unknown instruction");
            }
        }
    }
    // println!("{:?}", state)
}
