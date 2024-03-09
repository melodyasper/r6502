use r6502::{emulator::{self, DefaultVirtualMemory, Emulator, EmulatorBuilder}, state::{SystemFlags, SystemState}};
use std::sync::{Arc, Mutex};

fn main() {
    

    // Instructions from https://codeburst.io/an-introduction-to-6502-assembly-and-low-level-programming-7c11fa6b9cb9
    // LDA   $60
    // ADC   $61
    // STA   $62

    let mut memory = vec![0; 0x10000];
    memory.append(&mut vec![
        0x78, 0xd8, 0xa2, 0xff, 0x9a, 0xa9, 0x00, 0x95, 0x00, 0xca, 0xd0, 0xfb, 0x85, 0x00,
        0xa9, 0x30, 0x85, 0x09, 0x4c, 0x00, 0xf0, 0x00, 0xf0, 0x00, 0xf0,
    ]);

    let emulator = EmulatorBuilder::default().memory(DefaultVirtualMemory::default()).build().unwrap();
    // https://llx.com/Neil/a2/opcodes.html
    let emulator = Arc::new(Mutex::new(emulator));



    loop
    {
        match emulator.lock() {
            Ok(mut state) => {
                match state.execute_next_instruction() {
                    Ok(instruction) => {
                        
                        println!("{:?} | executed", instruction);
                    }
                    Err(Some(instruction)) => {
                        println!("Failed to execute the instruction {:?}", instruction);
                        break;
                    }
                    Err(None) => {
                        println!("Failed to read");
                        break;
                    }
                }
            }
            Err(_) => todo!(),
        }
    }
    // println!("{:?}", state)
}
