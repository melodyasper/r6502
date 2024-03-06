use r6502::emulator::state::{SystemState, SystemFlags};
use r6502::emulator::display::Renderer;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    

    // Instructions from https://codeburst.io/an-introduction-to-6502-assembly-and-low-level-programming-7c11fa6b9cb9
    // LDA   $60
    // ADC   $61
    // STA   $62

    let mut memory = vec![0; 0xF000];
    memory.append(&mut vec![
        0x78, 0xd8, 0xa2, 0xff, 0x9a, 0xa9, 0x00, 0x95, 0x00, 0xca, 0xd0, 0xfb, 0x85, 0x00,
        0xa9, 0x30, 0x85, 0x09, 0x4c, 0x00, 0xf0, 0x00, 0xf0, 0x00, 0xf0,
    ]);

    // https://llx.com/Neil/a2/opcodes.html
    let state = Arc::new(Mutex::new( SystemState {
        running: true,
        pc: 0xF000,
        // memory: vec![0xA5, 0x60, 0x65, 0x61, 0x85, 0x62],
        m: memory,
        a: 0,
        x: 0,
        y: 0,
        s: 0,
        p: SystemFlags::from(0),
    }));


    let state_clone = Arc::clone(&state);
    thread::spawn(move || {

        let renderer = Renderer {state: state_clone};
        let _ = renderer.start();
    });

    
    
    // state.memory.resize(256, 0x00);
    loop
    {
        match state.lock() {
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
