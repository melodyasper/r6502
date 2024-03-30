use std::sync::{Arc, Mutex};

use crate::{instructions::{Instruction, OpCode}, state::{SystemAction, SystemCycle, SystemState}};
use anyhow::Result;
use derive_builder::Builder;

#[derive(Builder)]
pub struct CPUEmulator<M>
where M: VirtualMemory {
    memory: Arc<Mutex<M>>,
    pub state: SystemState,
}


impl <M> CPUEmulator <M>
where M: VirtualMemory {
    pub fn execute_next_instruction(&mut self) -> Result<Instruction, Option<Instruction>> {
        if !self.state.running {
            return Err(None);
        }
        let ibyte = self.memory.lock().unwrap().read(self.state.pc);

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
}
impl <M> VirtualMemory for CPUEmulator <M>
where M: VirtualMemory {
    fn read(&mut self, address: u16) -> u8 {
        let byte = self.memory.lock().unwrap().read(address);
        self.state.cycles.push(SystemCycle {address, value: byte, action: SystemAction::READ});
        byte
    }
    
    fn write(&mut self, address: u16, value: u8) {
        self.memory.lock().unwrap().write(address, value);
        self.state.cycles.push(SystemCycle {address, value, action: SystemAction::WRITE});
    }
}



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefaultVirtualMemory {
    m: Vec<u8>
}

impl <'a> Default for DefaultVirtualMemory{
    fn default() -> Self {
        Self { m: vec![0; 0x10000] }
    }
}

impl From<Vec<u8>> for DefaultVirtualMemory {
    fn from(value: Vec<u8>) -> Self {
        let mut nvec: Vec<u8> = vec![];
        nvec.extend(value);
        nvec.resize(0x10000, 0);
        Self { m: nvec }
    }
}

impl<'a> IntoIterator for &'a DefaultVirtualMemory {
    type Item = &'a u8;

    type IntoIter = <&'a Vec<u8> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.m.iter()
    }
}

pub trait VirtualMemory {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}
impl VirtualMemory for DefaultVirtualMemory {
    fn read(&mut self, address: u16) -> u8 {
        *self.m.get(address as usize).unwrap_or(&0)
    }
    fn write(&mut self, address: u16, value: u8) {
        self.m[address as usize] = value;
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
