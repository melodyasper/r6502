use crate::emulator::instructions::Instruction;
use crate::emulator::memory::DeviceMemory;
use paste::paste;

use std::sync::Mutex;
use std::{fs::File, sync::Arc};
use std::io::Read;
use serde_json::{Result as SerdeResult, Value};


#[derive(Debug, PartialEq, Eq)]
pub struct StatusFlags {
    value: u8,
}
macro_rules! create_status_flag {
    ($name:ident, $value:expr) => {
        paste! {
            pub fn [< $name _flag >] (&self) -> bool {
                (self.value & $value) != 0
            }
            pub fn [<set_ $name _flag >](&mut self, set: bool) {
                if set {
                    self.value |= $value;
                } else {
                    self.value &= !$value;
                }
            }
        }
    };
}

impl StatusFlags {
    pub fn new(value: u8) -> Self {
        StatusFlags { value }
    }

    create_status_flag!(negative,          0b10000000); 
    create_status_flag!(overflow,          0b01000000); 
    create_status_flag!(expansion,         0b00100000);
    create_status_flag!(break_command,     0b00010000);
    create_status_flag!(decimal,           0b00001000);
    create_status_flag!(interrupt_disable, 0b00000100);
    create_status_flag!(zero,              0b00000010);
    create_status_flag!(carry,             0b00000001);

    // You can add more getters and setters for other bits following the pattern above.
}

#[derive(Debug, PartialEq, Eq)]
pub struct State {
    pub running: bool,
    pub program_counter: usize,
    pub memory: Vec<u8>,
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub register_s: u8,
    pub register_p: u8,
    pub status_flags: StatusFlags,
}

impl State {
    pub fn print_registers(&self) {
        println!("PC: {:#02x} | A: {:#02x} | X: {:#02x} | Y: {:#02x} | S: {:#02x} | P: {:#02x}", self.program_counter, self.register_a, self.register_x, self.register_y, self.register_s, self.register_p);
    }
    pub fn get_next_instruction(&mut self) -> Option<Instruction> {
        let next_instruction = self.consume_byte();
        match next_instruction {
            Some(value) => match Instruction::try_from(value) {
                Ok(next_instruction) => Some(next_instruction),
                Err(_) => {
                    if value != 0x0 {
                        println!("Couldn't figure out instruction {:#02x}", value);
                    }
                    None
                },
            },
            None => {
                self.running = false;
                None
            }
        }
    }
    pub fn consume_byte(&mut self) -> Option<u8> {
        let program_counter = self.program_counter;
        self.program_counter += 1;
        // TODO: Can't use `fetch_memory` here until we fix our little hack in it.
        // Otherwise program space will grow indefinitely.
        match self.memory.get(program_counter) {
            Some(value) => Some(*value),
            None => None,
        }
    }
    pub fn fetch_memory(&mut self, address: usize) -> Result<u8, ()> {
        // println!("Read from memory @ {}", address);
        let length = self.memory.len();
        if length < address {
            // TODO: Remove this hack.
            self.memory.resize(address + 1, 0);
        }
        match self.memory.get(address) {
            Some(value) => Ok(*value),
            None => Err(()),
        }
    }
    pub fn write_memory(&mut self, address: usize, value: u8) -> Result<(), ()> {
        println!("Writing to {:x} a value of {:x}", address, value);
        // println!("Insert into memory @ {} value {}", address, value);
        if address <= u8::MAX.into() {
            let device_memory = DeviceMemory::try_from(address as u8);
            match device_memory {
                Ok(device_memory) => println!("Write to {:?}", device_memory),
                _ => ()
            }
        }

        let length = self.memory.len();
        if length < address {
            // TODO: Remove this hack.
            self.memory.resize(address + 1, 0);
        }
        self.memory[address] = value;
        Ok(())
    }
}





// #[derive(Deserialize)]
// struct TestRam {
//     data: (usize, u8),
// }
// struct TestRegisters {
//     pc: usize,
//     s: u8,
//     a: u8, 
//     x: u8,
//     y: u8,
//     p: u8,
//     ram: Vec<TestRam>
// }
// struct TestCase<'a> {
//     name: &'a str,
//     initial: TestRegisters,
//     r#final: TestRegisters,
//     cycles: Vec<(usize, u8, String)>,
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_a1() {
        let mut file = File::open("external/ProcessorTests/6502/v1/a0.json").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        
        let v: Value = serde_json::from_str(&contents).unwrap();
        for value in v.as_array().unwrap().into_iter() {
            let mut memory: Vec<u8> = Vec::new();
            memory.resize(128_000, 0x0);
            let state = Arc::new(Mutex::new( State {
                running: true,
                program_counter: value["initial"]["pc"].as_u64().unwrap() as usize,
                // memory: vec![0xA5, 0x60, 0x65, 0x61, 0x85, 0x62],
                memory: memory,
                register_a: value["initial"]["a"].as_u64().unwrap() as u8,
                register_x: value["initial"]["x"].as_u64().unwrap() as u8,
                register_y: value["initial"]["y"].as_u64().unwrap() as u8,
                register_s: value["initial"]["s"].as_u64().unwrap() as u8,
                register_p: value["initial"]["p"].as_u64().unwrap() as u8,
                status_flags: StatusFlags::new(0),
            }));

            let mut state = state.lock().unwrap();
            for memory in value["initial"]["ram"].as_array().unwrap().iter() {
                let memory = memory.as_array().unwrap();
                let address = memory.get(0).unwrap().as_u64().unwrap() as usize;
                let value = memory.get(1).unwrap().as_u64().unwrap() as u8;
                state.write_memory(address, value).unwrap();
            }
            

            loop
            {
                if state.running {
                    match state.get_next_instruction() {
                        Some(instruction) => {
                            // println!("{:?} | Executing", instruction);
                            match instruction.execute(&mut state) {
                                Ok(_) => (),
                                _ => {
                                    println!("Failed to execute instruction {:?}", instruction);
                                    state.running = false;
                                }
                            }
                        }
                        None => {
                            // println!("Unknown instruction");
                            state.running = false;
                        }
                    }
                }
                else {
                    break;
                }
            }

        }
        
        assert_eq!(4, 2+1);
    }
}