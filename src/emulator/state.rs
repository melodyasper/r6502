use crate::emulator::instructions::{Instruction, OpCode};
use paste::paste;
use anyhow::{Result, anyhow};
use tabled::Tabled;

#[derive(Debug, PartialEq, Eq)]
pub struct StatusFlags {
    pub value: u8,
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

    create_status_flag!(negative, 0b10000000);
    create_status_flag!(overflow, 0b01000000);
    create_status_flag!(expansion, 0b00100000);
    create_status_flag!(break_command, 0b00010000);
    create_status_flag!(decimal, 0b00001000);
    create_status_flag!(interrupt_disable, 0b00000100);
    create_status_flag!(zero, 0b00000010);
    create_status_flag!(carry, 0b00000001);

    // You can add more getters and setters for other bits following the pattern above.
}

#[derive(Debug, PartialEq, Eq, Tabled)]
pub struct SystemState {
    pub running: bool,
    pub pc: u16,
    #[tabled(skip)]
    pub m: Vec<u8>,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    // Stack Pointer
    // The processor supports a 256 byte stack located between $0100 and $01FF
    pub s: u8,
    #[tabled(skip)]
    pub p: StatusFlags,
}

impl Default for SystemState {
    fn default() -> Self {
        let mut memory: Vec<u8> = Vec::new();
        memory.resize(128_000, 0x00);
        Self {
            running: Default::default(),
            pc: Default::default(),
            m: memory,
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: StatusFlags { value: 0 },
        }
    }
}

#[derive(Debug)]
pub enum EmulatorError {
    MemoryReadError,
    MemoryWriteError,
    UnimplementedInstruction,
    InvalidInstructionMode,
    ExpectedMemoryPair,
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryReadError =>  write!(f, "Memory read error"),
            Self::MemoryWriteError => write!(f, "Memory write error"),
            Self::UnimplementedInstruction => write!(f, "Instruction not implemented"),
            Self::InvalidInstructionMode => write!(f, "Instruction mode is not a valid mode"),
            Self::ExpectedMemoryPair => write!(f, "Memory pair was expected but received None"),
        }
    }
}

impl SystemState {
    pub fn execute_next_instruction(&mut self) -> Result<Instruction, Option<Instruction>> {
        let mut location = self.pc();
        let next_instruction = self.read(location);
        let ibyte = match next_instruction {
            Ok(ibyte) => ibyte,
            Err(_) => {
                self.running = false;
                return Err(None);
            }
        };
        let instruction = Instruction::from(ibyte);
        match instruction.opcode {
            OpCode::UnknownInstruction(_) => {
                self.running = false;
                return Err(Some(instruction));
            },
            OpCode::BadInstruction(_) => {
                self.running = false;
                return Err(Some(instruction));
            },
            _ => ()
        };

        location = location.wrapping_add(1);

        match instruction.execute(self, &mut location) {
            Ok(_) => {
                // println!("\tpc advanced from {:#08x} to {:#08x}", self.pc, location);
                self.set_pc(location);
                Ok(instruction)
            }
            Err(message) => {
                println!("{}", message);
                self.running = false;
                return Err(Some(instruction));
            },
        }
        
    }
    pub fn read(&mut self, address: u16) -> Result<u8> {
        
        let byte = self.m.get(address as usize).ok_or(anyhow!(EmulatorError::MemoryReadError).context(format!("Memory read error at address {}", address)))?;
        // println!("Reading from address {:#04x} yielded byte {:#04x}", address, *byte);
        Ok(*byte)
    }
    
    pub fn pc(&self) -> u16 {
        self.pc
    }
    pub fn set_pc(&mut self, address: u16) -> () {
        self.pc = address
    }
    pub fn write(&mut self, address: u16, value: u8) -> Result<()> {
        // println!("Writing to {:x} a value of {:x}", address, value);
        // println!("Insert into memory @ {} value {}", address, value);

        let length = self.m.len();
        if length < address.into() {
            // TODO: Remove this hack.
            self.m.resize(address as usize + 1, 0x00);
        }
        self.m[address as usize] = value;
        Ok(())
    }
}

