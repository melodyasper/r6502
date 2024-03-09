use crate::{instructions::{Instruction, OpCode}, state::{SystemAction, SystemCycle, SystemState}};
use anyhow::Result;
use derive_builder::Builder;

#[derive(Builder)]
pub struct Emulator<M>
where M: VirtualMemory {
    memory_hook: M,
    pub state: SystemState,
}


impl <M> Emulator <M>
where M: VirtualMemory {
    pub fn execute_next_instruction(&mut self) -> Result<Instruction, Option<Instruction>> {
        if !self.state.running {
            return Err(None);
        }
        let ibyte = self.memory_hook.read(self.state.pc);

        let instruction = Instruction::from(ibyte);
        match instruction.opcode {
            OpCode::UnknownInstruction => {
                self.state.running = false;
                return Err(Some(instruction));
            },
            OpCode::BadInstruction => {
                self.state.running = false;
                return Err(Some(instruction));
            },
            _ => ()
        };

        self.state.pc = self.state.pc.wrapping_add(1);

        match instruction.execute(self) {
            Ok(_) => {
                Ok(instruction)
            }
            Err(_) => {
                self.state.running = false;
                Err(Some(instruction))
            },
        }
        
    }
    pub fn read(&mut self, address: u16) -> u8 {
        
        let byte = self.memory_hook.read(address);
        self.state.cycles.push(SystemCycle {address, value: byte, action: SystemAction::READ});
        byte
    }
    
    pub fn write(&mut self, address: u16, value: u8) {
        // println!("Writing to {:x} a value of {:x}", address, value);
        // println!("Insert into memory @ {} value {}", address, value);

        self.memory_hook.write(address, value);
        self.state.cycles.push(SystemCycle {address, value, action: SystemAction::WRITE});
    }
}

#[derive(Clone)]
pub struct DefaultVirtualMemory {
    m: Vec<u8>
}

impl <'a> Default for DefaultVirtualMemory{
    fn default() -> Self {
        Self { m: vec![0; 0x10000] }
    }
}

pub trait VirtualMemory {
    fn read(&self, address: u16) -> u8;
    fn write(&self, address: u16, value: u8);
}
impl VirtualMemory for DefaultVirtualMemory {
    fn read(&self, address: u16) -> u8 {
        return 0;
    }
    fn write(&self, address: u16, value: u8) {
        ()
    }
}



// https://www.nesdev.org/wiki/CPU_ALL

/*
        .--\/--.
 AD1 <- |01  40| -- +5V
 AD2 <- |02  39| -> OUT0
/RST -> |03  38| -> OUT1
 A00 <- |04  37| -> OUT2
 A01 <- |05  36| -> /OE1
 A02 <- |06  35| -> /OE2
 A03 <- |07  34| -> R/W
 A04 <- |08  33| <- /NMI
 A05 <- |09  32| <- /IRQ
 A06 <- |10  31| -> M2
 A07 <- |11  30| <- TST (usually GND)
 A08 <- |12  29| <- CLK
 A09 <- |13  28| <> D0
 A10 <- |14  27| <> D1
 A11 <- |15  26| <> D2
 A12 <- |16  25| <> D3
 A13 <- |17  24| <> D4
 A14 <- |18  23| <> D5
 A15 <- |19  22| <> D6
 GND -- |20  21| <> D7

*/
