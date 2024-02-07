use crate::emulator::instructions::Instruction;
use crate::emulator::memory::DeviceMemory;

#[derive(Debug)]
pub struct StatusFlags {
    value: u8,
}
macro_rules! create_status_flag {
    ($name:ident, $value:expr) => {
        ::paste::paste! {
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

#[derive(Debug)]
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
    fn print_registers(&self) {
        println!("Registers:");
        println!("A: {:#02x} | X: {:#02x} | Y: {:#02x} | S: {:#02x} | P: {:#02x}", self.register_a, self.register_x, self.register_y, self.register_s, self.register_p);
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