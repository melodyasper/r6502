mod emulator;
use crate::emulator::state::{State, StatusFlags};

fn main() {
    // Instructions from https://codeburst.io/an-introduction-to-6502-assembly-and-low-level-programming-7c11fa6b9cb9
    // LDA   $60
    // ADC   $61
    // STA   $62

    // https://llx.com/Neil/a2/opcodes.html
    let mut state = State {
        running: true,
        program_counter: 0xF000,
        // memory: vec![0xA5, 0x60, 0x65, 0x61, 0x85, 0x62],
        memory: Vec::new(),
        register_a: 0,
        register_x: 0,
        register_y: 0,
        register_s: 0,
        register_p: 0,
        status_flags: StatusFlags::new(0),
    };
    state.memory.resize(0xF000, 0x0);
    state.memory.append(&mut vec![
        0x78, 0xd8, 0xa2, 0xff, 0x9a, 0xa9, 0x00, 0x95, 0x00, 0xca, 0xd0, 0xfb, 0x85, 0x00,
        0xa9, 0x30, 0x85, 0x09, 0x4c, 0x00, 0xf0, 0x00, 0xf0, 0x00, 0xf0,
    ]);
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
