use crate::emulator::instructions::Instruction;
use crate::emulator::memory::DeviceMemory;
use paste::paste;

use serde_json::{Result as SerdeResult, Value};
use std::io::Read;
use std::sync::Mutex;
use std::{fs::File, sync::Arc};
use std::time::Instant;

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

#[derive(Debug, PartialEq, Eq)]
pub struct SystemState {
    pub running: bool,
    pub pc: usize,
    pub m: Vec<u8>,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: u8,
    pub flags: StatusFlags,
}

impl Default for SystemState {
    fn default() -> Self {
        let mut memory: Vec<u8> = Vec::new();
        memory.resize(128_000, 0);
        Self {
            running: Default::default(),
            pc: Default::default(),
            m: memory,
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: 0,
            flags: StatusFlags { value: 0 },
        }
    }
}

impl SystemState {
    pub fn print_registers(&self) {
        println!(
            "PC: {:#02x} | A: {:#02x} | X: {:#02x} | Y: {:#02x} | S: {:#02x} | P: {:#02x}",
            self.pc, self.a, self.x, self.y, self.s, self.p
        );
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
                }
            },
            None => {
                self.running = false;
                None
            }
        }
    }
    pub fn consume_byte(&mut self) -> Option<u8> {
        let program_counter = self.pc;
        self.pc += 1;
        // TODO: Can't use `fetch_memory` here until we fix our little hack in it.
        // Otherwise program space will grow indefinitely.
        match self.m.get(program_counter) {
            Some(value) => Some(*value),
            None => None,
        }
    }
    pub fn fetch_memory(&mut self, address: usize) -> Result<u8, ()> {
        // println!("Read from memory @ {}", address);
        let length = self.m.len();
        if length < address {
            // TODO: Remove this hack.
            self.m.resize(address + 1, 0);
        }
        match self.m.get(address) {
            Some(value) => Ok(*value),
            None => Err(()),
        }
    }
    pub fn write_memory(&mut self, address: usize, value: u8) -> Result<(), ()> {
        // println!("Writing to {:x} a value of {:x}", address, value);
        // println!("Insert into memory @ {} value {}", address, value);

        let length = self.m.len();
        if length < address {
            // TODO: Remove this hack.
            self.m.resize(address + 1, 0);
        }
        self.m[address] = value;
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

    fn json_to_state(state_map: &Value) -> SystemState {
        let mut state = SystemState::default();
        state.pc = state_map["pc"].as_u64().unwrap() as usize;
        state.a = state_map["a"].as_u64().unwrap() as u8;
        state.x = state_map["x"].as_u64().unwrap() as u8;
        state.y = state_map["y"].as_u64().unwrap() as u8;
        state.s = state_map["s"].as_u64().unwrap() as u8;
        state.p = state_map["p"].as_u64().unwrap() as u8;

        for memory in state_map["ram"].as_array().unwrap().iter() {
            let memory = memory.as_array().unwrap();
            let address = memory.get(0).unwrap().as_u64().unwrap() as usize;
            let value = memory.get(1).unwrap().as_u64().unwrap() as u8;
            state.write_memory(address, value).unwrap();
        }

        state
    }

    fn run_to_completion(mut state: &mut SystemState) {
        let time_start = Instant::now();
        loop {
            let time_now = Instant::now();
            let difference = time_now - time_start;
            if difference.as_secs_f32() > 1.0 {
                state.running = false;
            }
            if state.running {
                match state.get_next_instruction() {
                    Some(instruction) => match instruction.execute(&mut state) {
                        Ok(_) => (),
                        _ => {
                            state.running = false;
                        }
                    },
                    None => {
                        state.running = false;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn debug_state_comparison(
        state_expected: &mut SystemState,
        state: &mut SystemState,
        print_me: bool,
    ) -> bool {
        state_expected.flags = StatusFlags { value: 0 };
        state.flags = StatusFlags { value: 0 };
        let result = state_expected == state;
        if result != true && print_me == true {
            print!("R[E,F] | ");
            if state.pc != state_expected.pc {
                print!("pc[{:x}, {:x}] ", state_expected.pc, state.pc);
            }
            if state.x != state_expected.x {
                print!("x[{:x}, {:x}] ", state_expected.x, state.x);
            }
            if state.y != state_expected.y {
                print!("y[{:x}, {:x}] ", state_expected.y, state.y);
            }
            if state.a != state_expected.a {
                print!("a[{:x}, {:x}] ", state_expected.a, state.a);
            }
            if state.s != state_expected.s {
                print!("s[{:x}, {:x}] ", state_expected.s, state.s);
            }
            if state.p != state_expected.p {
                print!("p[{:x}, {:x}] ", state_expected.p, state.p);
            }

            let mvec: Vec<(usize, (u8, u8))> = state_expected
                .m
                .clone()
                .into_iter()
                .zip(state.m.clone().into_iter())
                .enumerate()
                .collect();

            for (address, (em, fm)) in mvec.into_iter() {
                if em != fm {
                    print!("m{:x}[{:x}, {:x}] ", address, em, fm);
                }
            }
            println!("");
        }

        result
    }

    fn run_processor_test(filename: String, instruction: u8) {
        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let v: Value = serde_json::from_str(&contents).unwrap();
        let mut tests_total = 0;
        let mut tests_passed = 0;
        for value in v.as_array().unwrap().into_iter() {
            tests_total += 1;
            let mut state = json_to_state(&value["initial"]);
            let mut final_state = json_to_state(&value["final"]);
            run_to_completion(&mut state);

            if debug_state_comparison(&mut final_state, &mut state, false) {
                tests_passed += 1;
            }
        }
        
        if tests_passed != tests_total {
            match grab_instruction_from_byte(instruction) {
                Some(instruction) => assert!(tests_passed == tests_total, "Instruction {:?} | tests passed: {}/{}", instruction, tests_passed, tests_total),
                None => assert!(tests_passed == tests_total, "Instruction 0x{:x} | tests passed: {}/{}", instruction, tests_passed, tests_total),
            };
        }
    }

    fn grab_instruction_from_byte(instruction: u8) -> Option<Instruction> {
        match Instruction::try_from(instruction) {
            Ok(next_instruction) => Some(next_instruction),
            Err(_) => None
        }
    }
    

    

    #[test]
    fn run_processor_test_00() {
        run_processor_test("external/ProcessorTests/6502/v1/00.json".to_owned(), 0x00);
    }
    #[test]
    fn run_processor_test_01() {
        run_processor_test("external/ProcessorTests/6502/v1/01.json".to_owned(), 0x01);
    }
    #[test]
    fn run_processor_test_02() {
        run_processor_test("external/ProcessorTests/6502/v1/02.json".to_owned(), 0x02);
    }
    #[test]
    fn run_processor_test_03() {
        run_processor_test("external/ProcessorTests/6502/v1/03.json".to_owned(), 0x03);
    }
    #[test]
    fn run_processor_test_04() {
        run_processor_test("external/ProcessorTests/6502/v1/04.json".to_owned(), 0x04);
    }
    #[test]
    fn run_processor_test_05() {
        run_processor_test("external/ProcessorTests/6502/v1/05.json".to_owned(), 0x05);
    }
    #[test]
    fn run_processor_test_06() {
        run_processor_test("external/ProcessorTests/6502/v1/06.json".to_owned(), 0x06);
    }
    #[test]
    fn run_processor_test_07() {
        run_processor_test("external/ProcessorTests/6502/v1/07.json".to_owned(), 0x07);
    }
    #[test]
    fn run_processor_test_08() {
        run_processor_test("external/ProcessorTests/6502/v1/08.json".to_owned(), 0x08);
    }
    #[test]
    fn run_processor_test_09() {
        run_processor_test("external/ProcessorTests/6502/v1/09.json".to_owned(), 0x09);
    }
    #[test]
    fn run_processor_test_0a() {
        run_processor_test("external/ProcessorTests/6502/v1/0a.json".to_owned(), 0x0a);
    }
    #[test]
    fn run_processor_test_0b() {
        run_processor_test("external/ProcessorTests/6502/v1/0b.json".to_owned(), 0x0b);
    }
    #[test]
    fn run_processor_test_0c() {
        run_processor_test("external/ProcessorTests/6502/v1/0c.json".to_owned(), 0x0c);
    }
    #[test]
    fn run_processor_test_0d() {
        run_processor_test("external/ProcessorTests/6502/v1/0d.json".to_owned(), 0x0d);
    }
    #[test]
    fn run_processor_test_0e() {
        run_processor_test("external/ProcessorTests/6502/v1/0e.json".to_owned(), 0x0e);
    }
    #[test]
    fn run_processor_test_0f() {
        run_processor_test("external/ProcessorTests/6502/v1/0f.json".to_owned(), 0x0f);
    }
    #[test]
    fn run_processor_test_10() {
        run_processor_test("external/ProcessorTests/6502/v1/10.json".to_owned(), 0x10);
    }
    #[test]
    fn run_processor_test_11() {
        run_processor_test("external/ProcessorTests/6502/v1/11.json".to_owned(), 0x11);
    }
    #[test]
    fn run_processor_test_12() {
        run_processor_test("external/ProcessorTests/6502/v1/12.json".to_owned(), 0x12);
    }
    #[test]
    fn run_processor_test_13() {
        run_processor_test("external/ProcessorTests/6502/v1/13.json".to_owned(), 0x13);
    }
    #[test]
    fn run_processor_test_14() {
        run_processor_test("external/ProcessorTests/6502/v1/14.json".to_owned(), 0x14);
    }
    #[test]
    fn run_processor_test_15() {
        run_processor_test("external/ProcessorTests/6502/v1/15.json".to_owned(), 0x15);
    }
    #[test]
    fn run_processor_test_16() {
        run_processor_test("external/ProcessorTests/6502/v1/16.json".to_owned(), 0x16);
    }
    #[test]
    fn run_processor_test_17() {
        run_processor_test("external/ProcessorTests/6502/v1/17.json".to_owned(), 0x17);
    }
    #[test]
    fn run_processor_test_18() {
        run_processor_test("external/ProcessorTests/6502/v1/18.json".to_owned(), 0x18);
    }
    #[test]
    fn run_processor_test_19() {
        run_processor_test("external/ProcessorTests/6502/v1/19.json".to_owned(), 0x19);
    }
    #[test]
    fn run_processor_test_1a() {
        run_processor_test("external/ProcessorTests/6502/v1/1a.json".to_owned(), 0x1a);
    }
    #[test]
    fn run_processor_test_1b() {
        run_processor_test("external/ProcessorTests/6502/v1/1b.json".to_owned(), 0x1b);
    }
    #[test]
    fn run_processor_test_1c() {
        run_processor_test("external/ProcessorTests/6502/v1/1c.json".to_owned(), 0x1c);
    }
    #[test]
    fn run_processor_test_1d() {
        run_processor_test("external/ProcessorTests/6502/v1/1d.json".to_owned(), 0x1d);
    }
    #[test]
    fn run_processor_test_1e() {
        run_processor_test("external/ProcessorTests/6502/v1/1e.json".to_owned(), 0x1e);
    }
    #[test]
    fn run_processor_test_1f() {
        run_processor_test("external/ProcessorTests/6502/v1/1f.json".to_owned(), 0x1f);
    }
    #[test]
    fn run_processor_test_20() {
        run_processor_test("external/ProcessorTests/6502/v1/20.json".to_owned(), 0x20);
    }
    #[test]
    fn run_processor_test_21() {
        run_processor_test("external/ProcessorTests/6502/v1/21.json".to_owned(), 0x21);
    }
    #[test]
    fn run_processor_test_22() {
        run_processor_test("external/ProcessorTests/6502/v1/22.json".to_owned(), 0x22);
    }
    #[test]
    fn run_processor_test_23() {
        run_processor_test("external/ProcessorTests/6502/v1/23.json".to_owned(), 0x23);
    }
    #[test]
    fn run_processor_test_24() {
        run_processor_test("external/ProcessorTests/6502/v1/24.json".to_owned(), 0x24);
    }
    #[test]
    fn run_processor_test_25() {
        run_processor_test("external/ProcessorTests/6502/v1/25.json".to_owned(), 0x25);
    }
    #[test]
    fn run_processor_test_26() {
        run_processor_test("external/ProcessorTests/6502/v1/26.json".to_owned(), 0x26);
    }
    #[test]
    fn run_processor_test_27() {
        run_processor_test("external/ProcessorTests/6502/v1/27.json".to_owned(), 0x27);
    }
    #[test]
    fn run_processor_test_28() {
        run_processor_test("external/ProcessorTests/6502/v1/28.json".to_owned(), 0x28);
    }
    #[test]
    fn run_processor_test_29() {
        run_processor_test("external/ProcessorTests/6502/v1/29.json".to_owned(), 0x29);
    }
    #[test]
    fn run_processor_test_2a() {
        run_processor_test("external/ProcessorTests/6502/v1/2a.json".to_owned(), 0x2a);
    }
    #[test]
    fn run_processor_test_2b() {
        run_processor_test("external/ProcessorTests/6502/v1/2b.json".to_owned(), 0x2b);
    }
    #[test]
    fn run_processor_test_2c() {
        run_processor_test("external/ProcessorTests/6502/v1/2c.json".to_owned(), 0x2c);
    }
    #[test]
    fn run_processor_test_2d() {
        run_processor_test("external/ProcessorTests/6502/v1/2d.json".to_owned(), 0x2d);
    }
    #[test]
    fn run_processor_test_2e() {
        run_processor_test("external/ProcessorTests/6502/v1/2e.json".to_owned(), 0x2e);
    }
    #[test]
    fn run_processor_test_2f() {
        run_processor_test("external/ProcessorTests/6502/v1/2f.json".to_owned(), 0x2f);
    }
    #[test]
    fn run_processor_test_30() {
        run_processor_test("external/ProcessorTests/6502/v1/30.json".to_owned(), 0x30);
    }
    #[test]
    fn run_processor_test_31() {
        run_processor_test("external/ProcessorTests/6502/v1/31.json".to_owned(), 0x31);
    }
    #[test]
    fn run_processor_test_32() {
        run_processor_test("external/ProcessorTests/6502/v1/32.json".to_owned(), 0x32);
    }
    #[test]
    fn run_processor_test_33() {
        run_processor_test("external/ProcessorTests/6502/v1/33.json".to_owned(), 0x33);
    }
    #[test]
    fn run_processor_test_34() {
        run_processor_test("external/ProcessorTests/6502/v1/34.json".to_owned(), 0x34);
    }
    #[test]
    fn run_processor_test_35() {
        run_processor_test("external/ProcessorTests/6502/v1/35.json".to_owned(), 0x35);
    }
    #[test]
    fn run_processor_test_36() {
        run_processor_test("external/ProcessorTests/6502/v1/36.json".to_owned(), 0x36);
    }
    #[test]
    fn run_processor_test_37() {
        run_processor_test("external/ProcessorTests/6502/v1/37.json".to_owned(), 0x37);
    }
    #[test]
    fn run_processor_test_38() {
        run_processor_test("external/ProcessorTests/6502/v1/38.json".to_owned(), 0x38);
    }
    #[test]
    fn run_processor_test_39() {
        run_processor_test("external/ProcessorTests/6502/v1/39.json".to_owned(), 0x39);
    }
    #[test]
    fn run_processor_test_3a() {
        run_processor_test("external/ProcessorTests/6502/v1/3a.json".to_owned(), 0x3a);
    }
    #[test]
    fn run_processor_test_3b() {
        run_processor_test("external/ProcessorTests/6502/v1/3b.json".to_owned(), 0x3b);
    }
    #[test]
    fn run_processor_test_3c() {
        run_processor_test("external/ProcessorTests/6502/v1/3c.json".to_owned(), 0x3c);
    }
    #[test]
    fn run_processor_test_3d() {
        run_processor_test("external/ProcessorTests/6502/v1/3d.json".to_owned(), 0x3d);
    }
    #[test]
    fn run_processor_test_3e() {
        run_processor_test("external/ProcessorTests/6502/v1/3e.json".to_owned(), 0x3e);
    }
    #[test]
    fn run_processor_test_3f() {
        run_processor_test("external/ProcessorTests/6502/v1/3f.json".to_owned(), 0x3f);
    }
    #[test]
    fn run_processor_test_40() {
        run_processor_test("external/ProcessorTests/6502/v1/40.json".to_owned(), 0x40);
    }
    #[test]
    fn run_processor_test_41() {
        run_processor_test("external/ProcessorTests/6502/v1/41.json".to_owned(), 0x41);
    }
    #[test]
    fn run_processor_test_42() {
        run_processor_test("external/ProcessorTests/6502/v1/42.json".to_owned(), 0x42);
    }
    #[test]
    fn run_processor_test_43() {
        run_processor_test("external/ProcessorTests/6502/v1/43.json".to_owned(), 0x43);
    }
    #[test]
    fn run_processor_test_44() {
        run_processor_test("external/ProcessorTests/6502/v1/44.json".to_owned(), 0x44);
    }
    #[test]
    fn run_processor_test_45() {
        run_processor_test("external/ProcessorTests/6502/v1/45.json".to_owned(), 0x45);
    }
    #[test]
    fn run_processor_test_46() {
        run_processor_test("external/ProcessorTests/6502/v1/46.json".to_owned(), 0x46);
    }
    #[test]
    fn run_processor_test_47() {
        run_processor_test("external/ProcessorTests/6502/v1/47.json".to_owned(), 0x47);
    }
    #[test]
    fn run_processor_test_48() {
        run_processor_test("external/ProcessorTests/6502/v1/48.json".to_owned(), 0x48);
    }
    #[test]
    fn run_processor_test_49() {
        run_processor_test("external/ProcessorTests/6502/v1/49.json".to_owned(), 0x49);
    }
    #[test]
    fn run_processor_test_4a() {
        run_processor_test("external/ProcessorTests/6502/v1/4a.json".to_owned(), 0x4a);
    }
    #[test]
    fn run_processor_test_4b() {
        run_processor_test("external/ProcessorTests/6502/v1/4b.json".to_owned(), 0x4b);
    }
    #[test]
    fn run_processor_test_4c() {
        run_processor_test("external/ProcessorTests/6502/v1/4c.json".to_owned(), 0x4c);
    }
    #[test]
    fn run_processor_test_4d() {
        run_processor_test("external/ProcessorTests/6502/v1/4d.json".to_owned(), 0x4d);
    }
    #[test]
    fn run_processor_test_4e() {
        run_processor_test("external/ProcessorTests/6502/v1/4e.json".to_owned(), 0x4e);
    }
    #[test]
    fn run_processor_test_4f() {
        run_processor_test("external/ProcessorTests/6502/v1/4f.json".to_owned(), 0x4f);
    }
    #[test]
    fn run_processor_test_50() {
        run_processor_test("external/ProcessorTests/6502/v1/50.json".to_owned(), 0x50);
    }
    #[test]
    fn run_processor_test_51() {
        run_processor_test("external/ProcessorTests/6502/v1/51.json".to_owned(), 0x51);
    }
    #[test]
    fn run_processor_test_52() {
        run_processor_test("external/ProcessorTests/6502/v1/52.json".to_owned(), 0x52);
    }
    #[test]
    fn run_processor_test_53() {
        run_processor_test("external/ProcessorTests/6502/v1/53.json".to_owned(), 0x53);
    }
    #[test]
    fn run_processor_test_54() {
        run_processor_test("external/ProcessorTests/6502/v1/54.json".to_owned(), 0x54);
    }
    #[test]
    fn run_processor_test_55() {
        run_processor_test("external/ProcessorTests/6502/v1/55.json".to_owned(), 0x55);
    }
    #[test]
    fn run_processor_test_56() {
        run_processor_test("external/ProcessorTests/6502/v1/56.json".to_owned(), 0x56);
    }
    #[test]
    fn run_processor_test_57() {
        run_processor_test("external/ProcessorTests/6502/v1/57.json".to_owned(), 0x57);
    }
    #[test]
    fn run_processor_test_58() {
        run_processor_test("external/ProcessorTests/6502/v1/58.json".to_owned(), 0x58);
    }
    #[test]
    fn run_processor_test_59() {
        run_processor_test("external/ProcessorTests/6502/v1/59.json".to_owned(), 0x59);
    }
    #[test]
    fn run_processor_test_5a() {
        run_processor_test("external/ProcessorTests/6502/v1/5a.json".to_owned(), 0x5a);
    }
    #[test]
    fn run_processor_test_5b() {
        run_processor_test("external/ProcessorTests/6502/v1/5b.json".to_owned(), 0x5b);
    }
    #[test]
    fn run_processor_test_5c() {
        run_processor_test("external/ProcessorTests/6502/v1/5c.json".to_owned(), 0x5c);
    }
    #[test]
    fn run_processor_test_5d() {
        run_processor_test("external/ProcessorTests/6502/v1/5d.json".to_owned(), 0x5d);
    }
    #[test]
    fn run_processor_test_5e() {
        run_processor_test("external/ProcessorTests/6502/v1/5e.json".to_owned(), 0x5e);
    }
    #[test]
    fn run_processor_test_5f() {
        run_processor_test("external/ProcessorTests/6502/v1/5f.json".to_owned(), 0x5f);
    }
    #[test]
    fn run_processor_test_60() {
        run_processor_test("external/ProcessorTests/6502/v1/60.json".to_owned(), 0x60);
    }
    #[test]
    fn run_processor_test_61() {
        run_processor_test("external/ProcessorTests/6502/v1/61.json".to_owned(), 0x61);
    }
    #[test]
    fn run_processor_test_62() {
        run_processor_test("external/ProcessorTests/6502/v1/62.json".to_owned(), 0x62);
    }
    #[test]
    fn run_processor_test_63() {
        run_processor_test("external/ProcessorTests/6502/v1/63.json".to_owned(), 0x63);
    }
    #[test]
    fn run_processor_test_64() {
        run_processor_test("external/ProcessorTests/6502/v1/64.json".to_owned(), 0x64);
    }
    #[test]
    fn run_processor_test_65() {
        run_processor_test("external/ProcessorTests/6502/v1/65.json".to_owned(), 0x65);
    }
    #[test]
    fn run_processor_test_66() {
        run_processor_test("external/ProcessorTests/6502/v1/66.json".to_owned(), 0x66);
    }
    #[test]
    fn run_processor_test_67() {
        run_processor_test("external/ProcessorTests/6502/v1/67.json".to_owned(), 0x67);
    }
    #[test]
    fn run_processor_test_68() {
        run_processor_test("external/ProcessorTests/6502/v1/68.json".to_owned(), 0x68);
    }
    #[test]
    fn run_processor_test_69() {
        run_processor_test("external/ProcessorTests/6502/v1/69.json".to_owned(), 0x69);
    }
    #[test]
    fn run_processor_test_6a() {
        run_processor_test("external/ProcessorTests/6502/v1/6a.json".to_owned(), 0x6a);
    }
    #[test]
    fn run_processor_test_6b() {
        run_processor_test("external/ProcessorTests/6502/v1/6b.json".to_owned(), 0x6b);
    }
    #[test]
    fn run_processor_test_6c() {
        run_processor_test("external/ProcessorTests/6502/v1/6c.json".to_owned(), 0x6c);
    }
    #[test]
    fn run_processor_test_6d() {
        run_processor_test("external/ProcessorTests/6502/v1/6d.json".to_owned(), 0x6d);
    }
    #[test]
    fn run_processor_test_6e() {
        run_processor_test("external/ProcessorTests/6502/v1/6e.json".to_owned(), 0x6e);
    }
    #[test]
    fn run_processor_test_6f() {
        run_processor_test("external/ProcessorTests/6502/v1/6f.json".to_owned(), 0x6f);
    }
    #[test]
    fn run_processor_test_70() {
        run_processor_test("external/ProcessorTests/6502/v1/70.json".to_owned(), 0x70);
    }
    #[test]
    fn run_processor_test_71() {
        run_processor_test("external/ProcessorTests/6502/v1/71.json".to_owned(), 0x71);
    }
    #[test]
    fn run_processor_test_72() {
        run_processor_test("external/ProcessorTests/6502/v1/72.json".to_owned(), 0x72);
    }
    #[test]
    fn run_processor_test_73() {
        run_processor_test("external/ProcessorTests/6502/v1/73.json".to_owned(), 0x73);
    }
    #[test]
    fn run_processor_test_74() {
        run_processor_test("external/ProcessorTests/6502/v1/74.json".to_owned(), 0x74);
    }
    #[test]
    fn run_processor_test_75() {
        run_processor_test("external/ProcessorTests/6502/v1/75.json".to_owned(), 0x75);
    }
    #[test]
    fn run_processor_test_76() {
        run_processor_test("external/ProcessorTests/6502/v1/76.json".to_owned(), 0x76);
    }
    #[test]
    fn run_processor_test_77() {
    
        run_processor_test("external/ProcessorTests/6502/v1/77.json".to_owned(), 0x77);
    }
    #[test]
    fn run_processor_test_78() {
    
        run_processor_test("external/ProcessorTests/6502/v1/78.json".to_owned(), 0x78);
    }
    #[test]
    fn run_processor_test_79() {
    
        run_processor_test("external/ProcessorTests/6502/v1/79.json".to_owned(), 0x79);
    }
    #[test]
    fn run_processor_test_7a() {
    
        run_processor_test("external/ProcessorTests/6502/v1/7a.json".to_owned(), 0x7a);
    }
    #[test]
    fn run_processor_test_7b() {
    
        run_processor_test("external/ProcessorTests/6502/v1/7b.json".to_owned(), 0x7b);
    }
    #[test]
    fn run_processor_test_7c() {
    
        run_processor_test("external/ProcessorTests/6502/v1/7c.json".to_owned(), 0x7c);
    }
    #[test]
    fn run_processor_test_7d() {
    
        run_processor_test("external/ProcessorTests/6502/v1/7d.json".to_owned(), 0x7d);
    }
    #[test]
    fn run_processor_test_7e() {
    
        run_processor_test("external/ProcessorTests/6502/v1/7e.json".to_owned(), 0x7e);
    }
    #[test]
    fn run_processor_test_7f() {
    
        run_processor_test("external/ProcessorTests/6502/v1/7f.json".to_owned(), 0x7f);
    }
    #[test]
    fn run_processor_test_80() {
    
        run_processor_test("external/ProcessorTests/6502/v1/80.json".to_owned(), 0x80);
    }
    #[test]
    fn run_processor_test_81() {
    
        run_processor_test("external/ProcessorTests/6502/v1/81.json".to_owned(), 0x81);
    }
    #[test]
    fn run_processor_test_82() {
    
        run_processor_test("external/ProcessorTests/6502/v1/82.json".to_owned(), 0x82);
    }
    #[test]
    fn run_processor_test_83() {
    
        run_processor_test("external/ProcessorTests/6502/v1/83.json".to_owned(), 0x83);
    }
    #[test]
    fn run_processor_test_84() {
    
        run_processor_test("external/ProcessorTests/6502/v1/84.json".to_owned(), 0x84);
    }
    #[test]
    fn run_processor_test_85() {
    
        run_processor_test("external/ProcessorTests/6502/v1/85.json".to_owned(), 0x85);
    }
    #[test]
    fn run_processor_test_86() {
    
        run_processor_test("external/ProcessorTests/6502/v1/86.json".to_owned(), 0x86);
    }
    #[test]
    fn run_processor_test_87() {
    
        run_processor_test("external/ProcessorTests/6502/v1/87.json".to_owned(), 0x87);
    }
    #[test]
    fn run_processor_test_88() {
        run_processor_test("external/ProcessorTests/6502/v1/88.json".to_owned(), 0x88);
    }
    #[test]
    fn run_processor_test_89() {
        run_processor_test("external/ProcessorTests/6502/v1/89.json".to_owned(), 0x89);
    }
    #[test]
    fn run_processor_test_8a() {
        run_processor_test("external/ProcessorTests/6502/v1/8a.json".to_owned(), 0x8a);
    }
    #[test]
    fn run_processor_test_8b() {
        run_processor_test("external/ProcessorTests/6502/v1/8b.json".to_owned(), 0x8b);
    }
    #[test]
    fn run_processor_test_8c() {
        run_processor_test("external/ProcessorTests/6502/v1/8c.json".to_owned(), 0x8c);
    }
    #[test]
    fn run_processor_test_8d() {
        run_processor_test("external/ProcessorTests/6502/v1/8d.json".to_owned(), 0x8d);
    }
    #[test]
    fn run_processor_test_8e() {
        run_processor_test("external/ProcessorTests/6502/v1/8e.json".to_owned(), 0x8e);
    }
    #[test]
    fn run_processor_test_8f() {
        run_processor_test("external/ProcessorTests/6502/v1/8f.json".to_owned(), 0x8f);
    }
    #[test]
    fn run_processor_test_90() {
        run_processor_test("external/ProcessorTests/6502/v1/90.json".to_owned(), 0x90);
    }
    #[test]
    fn run_processor_test_91() {
        run_processor_test("external/ProcessorTests/6502/v1/91.json".to_owned(), 0x91);
    }
    #[test]
    fn run_processor_test_92() {
        run_processor_test("external/ProcessorTests/6502/v1/92.json".to_owned(), 0x92);
    }
    #[test]
    fn run_processor_test_93() {
        run_processor_test("external/ProcessorTests/6502/v1/93.json".to_owned(), 0x93);
    }
    #[test]
    fn run_processor_test_94() {
        run_processor_test("external/ProcessorTests/6502/v1/94.json".to_owned(), 0x94);
    }
    #[test]
    fn run_processor_test_95() {
        run_processor_test("external/ProcessorTests/6502/v1/95.json".to_owned(), 0x95);
    }
    #[test]
    fn run_processor_test_96() {
        run_processor_test("external/ProcessorTests/6502/v1/96.json".to_owned(), 0x96);
    }
    #[test]
    fn run_processor_test_97() {
        run_processor_test("external/ProcessorTests/6502/v1/97.json".to_owned(), 0x97);
    }
    #[test]
    fn run_processor_test_98() {
        run_processor_test("external/ProcessorTests/6502/v1/98.json".to_owned(), 0x98);
    }
    #[test]
    fn run_processor_test_99() {
        run_processor_test("external/ProcessorTests/6502/v1/99.json".to_owned(), 0x99);
    }
    #[test]
    fn run_processor_test_9a() {
        run_processor_test("external/ProcessorTests/6502/v1/9a.json".to_owned(), 0x9a);
    }
    #[test]
    fn run_processor_test_9b() {
        run_processor_test("external/ProcessorTests/6502/v1/9b.json".to_owned(), 0x9b);
    }
    #[test]
    fn run_processor_test_9c() {
        run_processor_test("external/ProcessorTests/6502/v1/9c.json".to_owned(), 0x9c);
    }
    #[test]
    fn run_processor_test_9d() {
        run_processor_test("external/ProcessorTests/6502/v1/9d.json".to_owned(), 0x9d);
    }
    #[test]
    fn run_processor_test_9e() {
        run_processor_test("external/ProcessorTests/6502/v1/9e.json".to_owned(), 0x9e);
    }
    #[test]
    fn run_processor_test_9f() {
        run_processor_test("external/ProcessorTests/6502/v1/9f.json".to_owned(), 0x9f);
    }
    #[test]
    fn run_processor_test_a0() {
        run_processor_test("external/ProcessorTests/6502/v1/a0.json".to_owned(), 0xa0);
    }
    #[test]
    fn run_processor_test_a1() {
        run_processor_test("external/ProcessorTests/6502/v1/a1.json".to_owned(), 0xa1);
    }
    #[test]
    fn run_processor_test_a2() {
        run_processor_test("external/ProcessorTests/6502/v1/a2.json".to_owned(), 0xa2);
    }
    #[test]
    fn run_processor_test_a3() {
        run_processor_test("external/ProcessorTests/6502/v1/a3.json".to_owned(), 0xa3);
    }
    #[test]
    fn run_processor_test_a4() {
        run_processor_test("external/ProcessorTests/6502/v1/a4.json".to_owned(), 0xa4);
    }
    #[test]
    fn run_processor_test_a5() {
        run_processor_test("external/ProcessorTests/6502/v1/a5.json".to_owned(), 0xa5);
    }
    #[test]
    fn run_processor_test_a6() {
        run_processor_test("external/ProcessorTests/6502/v1/a6.json".to_owned(), 0xa6);
    }
    #[test]
    fn run_processor_test_a7() {
        run_processor_test("external/ProcessorTests/6502/v1/a7.json".to_owned(), 0xa7);
    }
    #[test]
    fn run_processor_test_a8() {
        run_processor_test("external/ProcessorTests/6502/v1/a8.json".to_owned(), 0xa8);
    }
    #[test]
    fn run_processor_test_a9() {
        run_processor_test("external/ProcessorTests/6502/v1/a9.json".to_owned(), 0xa9);
    }
    #[test]
    fn run_processor_test_aa() {
        run_processor_test("external/ProcessorTests/6502/v1/aa.json".to_owned(), 0xaa);
    }
    #[test]
    fn run_processor_test_ab() {
        run_processor_test("external/ProcessorTests/6502/v1/ab.json".to_owned(), 0xab);
    }
    #[test]
    fn run_processor_test_ac() {
        run_processor_test("external/ProcessorTests/6502/v1/ac.json".to_owned(), 0xac);
    }
    #[test]
    fn run_processor_test_ad() {
        run_processor_test("external/ProcessorTests/6502/v1/ad.json".to_owned(), 0xad);
    }
    #[test]
    fn run_processor_test_ae() {
        run_processor_test("external/ProcessorTests/6502/v1/ae.json".to_owned(), 0xae);
    }
    #[test]
    fn run_processor_test_af() {
        run_processor_test("external/ProcessorTests/6502/v1/af.json".to_owned(), 0xaf);
    }
    #[test]
    fn run_processor_test_b0() {
        run_processor_test("external/ProcessorTests/6502/v1/b0.json".to_owned(), 0xb0);
    }
    #[test]
    fn run_processor_test_b1() {
        run_processor_test("external/ProcessorTests/6502/v1/b1.json".to_owned(), 0xb1);
    }
    #[test]
    fn run_processor_test_b2() {
        run_processor_test("external/ProcessorTests/6502/v1/b2.json".to_owned(), 0xb2);
    }
    #[test]
    fn run_processor_test_b3() {
        run_processor_test("external/ProcessorTests/6502/v1/b3.json".to_owned(), 0xb3);
    }
    #[test]
    fn run_processor_test_b4() {
        run_processor_test("external/ProcessorTests/6502/v1/b4.json".to_owned(), 0xb4);
    }
    #[test]
    fn run_processor_test_b5() {
        run_processor_test("external/ProcessorTests/6502/v1/b5.json".to_owned(), 0xb5);
    }
    #[test]
    fn run_processor_test_b6() {
        run_processor_test("external/ProcessorTests/6502/v1/b6.json".to_owned(), 0xb6);
    }
    #[test]
    fn run_processor_test_b7() {
        run_processor_test("external/ProcessorTests/6502/v1/b7.json".to_owned(), 0xb7);
    }
    #[test]
    fn run_processor_test_b8() {
        run_processor_test("external/ProcessorTests/6502/v1/b8.json".to_owned(), 0xb8);
    }
    #[test]
    fn run_processor_test_b9() {
        run_processor_test("external/ProcessorTests/6502/v1/b9.json".to_owned(), 0xb9);
    }
    #[test]
    fn run_processor_test_ba() {
        run_processor_test("external/ProcessorTests/6502/v1/ba.json".to_owned(), 0xba);
    }
    #[test]
    fn run_processor_test_bb() {
        run_processor_test("external/ProcessorTests/6502/v1/bb.json".to_owned(), 0xbb);
    }
    #[test]
    fn run_processor_test_bc() {
        run_processor_test("external/ProcessorTests/6502/v1/bc.json".to_owned(), 0xbc);
    }
    #[test]
    fn run_processor_test_bd() {
        run_processor_test("external/ProcessorTests/6502/v1/bd.json".to_owned(), 0xbd);
    }
    #[test]
    fn run_processor_test_be() {
        run_processor_test("external/ProcessorTests/6502/v1/be.json".to_owned(), 0xbe);
    }
    #[test]
    fn run_processor_test_bf() {
        run_processor_test("external/ProcessorTests/6502/v1/bf.json".to_owned(), 0xbf);
    }
    #[test]
    fn run_processor_test_c0() {
        run_processor_test("external/ProcessorTests/6502/v1/c0.json".to_owned(), 0xc0);
    }
    #[test]
    fn run_processor_test_c1() {
        run_processor_test("external/ProcessorTests/6502/v1/c1.json".to_owned(), 0xc1);
    }
    #[test]
    fn run_processor_test_c2() {
        run_processor_test("external/ProcessorTests/6502/v1/c2.json".to_owned(), 0xc2);
    }
    #[test]
    fn run_processor_test_c3() {
    
        run_processor_test("external/ProcessorTests/6502/v1/c3.json".to_owned(), 0xc3);
    }
    #[test]
    fn run_processor_test_c4() {
    
        run_processor_test("external/ProcessorTests/6502/v1/c4.json".to_owned(), 0xc4);
    }
    #[test]
    fn run_processor_test_c5() {
    
        run_processor_test("external/ProcessorTests/6502/v1/c5.json".to_owned(), 0xc5);
    }
    #[test]
    fn run_processor_test_c6() {
    
        run_processor_test("external/ProcessorTests/6502/v1/c6.json".to_owned(), 0xc6);
    }
    #[test]
    fn run_processor_test_c7() {
    
        run_processor_test("external/ProcessorTests/6502/v1/c7.json".to_owned(), 0xc7);
    }
    #[test]
    fn run_processor_test_c8() {
        run_processor_test("external/ProcessorTests/6502/v1/c8.json".to_owned(), 0xc8);
    }
    #[test]
    fn run_processor_test_c9() {
        run_processor_test("external/ProcessorTests/6502/v1/c9.json".to_owned(), 0xc9);
    }
    #[test]
    fn run_processor_test_ca() {
        run_processor_test("external/ProcessorTests/6502/v1/ca.json".to_owned(), 0xca);
    }
    #[test]
    fn run_processor_test_cb() {
        run_processor_test("external/ProcessorTests/6502/v1/cb.json".to_owned(), 0xcb);
    }
    #[test]
    fn run_processor_test_cc() {
        run_processor_test("external/ProcessorTests/6502/v1/cc.json".to_owned(), 0xcc);
    }
    #[test]
    fn run_processor_test_cd() {
        run_processor_test("external/ProcessorTests/6502/v1/cd.json".to_owned(), 0xcd);
    }
    #[test]
    fn run_processor_test_ce() {
        run_processor_test("external/ProcessorTests/6502/v1/ce.json".to_owned(), 0xce);
    }
    #[test]
    fn run_processor_test_cf() {
        run_processor_test("external/ProcessorTests/6502/v1/cf.json".to_owned(), 0xcf);
    }
    #[test]
    fn run_processor_test_d0() {
        run_processor_test("external/ProcessorTests/6502/v1/d0.json".to_owned(), 0xd0);
    }
    #[test]
    fn run_processor_test_d1() {
        run_processor_test("external/ProcessorTests/6502/v1/d1.json".to_owned(), 0xd1);
    }
    #[test]
    fn run_processor_test_d2() {
        run_processor_test("external/ProcessorTests/6502/v1/d2.json".to_owned(), 0xd2);
    }
    #[test]
    fn run_processor_test_d3() {
        run_processor_test("external/ProcessorTests/6502/v1/d3.json".to_owned(), 0xd3);
    }
    #[test]
    fn run_processor_test_d4() {
        run_processor_test("external/ProcessorTests/6502/v1/d4.json".to_owned(), 0xd4);
    }
    #[test]
    fn run_processor_test_d5() {
        run_processor_test("external/ProcessorTests/6502/v1/d5.json".to_owned(), 0xd5);
    }
    #[test]
    fn run_processor_test_d6() {
        run_processor_test("external/ProcessorTests/6502/v1/d6.json".to_owned(), 0xd6);
    }
    #[test]
    fn run_processor_test_d7() {
        run_processor_test("external/ProcessorTests/6502/v1/d7.json".to_owned(), 0xd7);
    }
    #[test]
    fn run_processor_test_d8() {
        run_processor_test("external/ProcessorTests/6502/v1/d8.json".to_owned(), 0xd8);
    }
    #[test]
    fn run_processor_test_d9() {
        run_processor_test("external/ProcessorTests/6502/v1/d9.json".to_owned(), 0xd9);
    }
    #[test]
    fn run_processor_test_da() {
        run_processor_test("external/ProcessorTests/6502/v1/da.json".to_owned(), 0xda);
    }
    #[test]
    fn run_processor_test_db() {
        run_processor_test("external/ProcessorTests/6502/v1/db.json".to_owned(), 0xdb);
    }
    #[test]
    fn run_processor_test_dc() {
        run_processor_test("external/ProcessorTests/6502/v1/dc.json".to_owned(), 0xdc);
    }
    #[test]
    fn run_processor_test_dd() {
        run_processor_test("external/ProcessorTests/6502/v1/dd.json".to_owned(), 0xdd);
    }
    #[test]
    fn run_processor_test_de() {
        run_processor_test("external/ProcessorTests/6502/v1/de.json".to_owned(), 0xde);
    }
    #[test]
    fn run_processor_test_df() {
        run_processor_test("external/ProcessorTests/6502/v1/df.json".to_owned(), 0xdf);
    }
    #[test]
    fn run_processor_test_e0() {
        run_processor_test("external/ProcessorTests/6502/v1/e0.json".to_owned(), 0xe0);
    }
    #[test]
    fn run_processor_test_e1() {
        run_processor_test("external/ProcessorTests/6502/v1/e1.json".to_owned(), 0xe1);
    }
    #[test]
    fn run_processor_test_e2() {
        run_processor_test("external/ProcessorTests/6502/v1/e2.json".to_owned(), 0xe2);
    }
    #[test]
    fn run_processor_test_e3() {
        run_processor_test("external/ProcessorTests/6502/v1/e3.json".to_owned(), 0xe3);
    }
    #[test]
    fn run_processor_test_e4() {
        run_processor_test("external/ProcessorTests/6502/v1/e4.json".to_owned(), 0xe4);
    }
    #[test]
    fn run_processor_test_e5() {
        run_processor_test("external/ProcessorTests/6502/v1/e5.json".to_owned(), 0xe5);
    }
    #[test]
    fn run_processor_test_e6() {
        run_processor_test("external/ProcessorTests/6502/v1/e6.json".to_owned(), 0xe6);
    }
    #[test]
    fn run_processor_test_e7() {
    
        run_processor_test("external/ProcessorTests/6502/v1/e7.json".to_owned(), 0xe7);
    }
    #[test]
    fn run_processor_test_e8() {
    
        run_processor_test("external/ProcessorTests/6502/v1/e8.json".to_owned(), 0xe8);
    }
    #[test]
    fn run_processor_test_e9() {
    
        run_processor_test("external/ProcessorTests/6502/v1/e9.json".to_owned(), 0xe9);
    }
    #[test]
    fn run_processor_test_ea() {
    
        run_processor_test("external/ProcessorTests/6502/v1/ea.json".to_owned(), 0xea);
    }
    #[test]
    fn run_processor_test_eb() {
    
        run_processor_test("external/ProcessorTests/6502/v1/eb.json".to_owned(), 0xeb);
    }
    #[test]
    fn run_processor_test_ec() {
    
        run_processor_test("external/ProcessorTests/6502/v1/ec.json".to_owned(), 0xec);
    }
    #[test]
    fn run_processor_test_ed() {
    
        run_processor_test("external/ProcessorTests/6502/v1/ed.json".to_owned(), 0xed);
    }
    #[test]
    fn run_processor_test_ee() {
    
        run_processor_test("external/ProcessorTests/6502/v1/ee.json".to_owned(), 0xee);
    }
    #[test]
    fn run_processor_test_ef() {
    
        run_processor_test("external/ProcessorTests/6502/v1/ef.json".to_owned(), 0xef);
    }
    #[test]
    fn run_processor_test_f0() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f0.json".to_owned(), 0xf0);
    }
    #[test]
    fn run_processor_test_f1() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f1.json".to_owned(), 0xf1);
    }
    #[test]
    fn run_processor_test_f2() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f2.json".to_owned(), 0xf2);
    }
    #[test]
    fn run_processor_test_f3() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f3.json".to_owned(), 0xf3);
    }
    #[test]
    fn run_processor_test_f4() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f4.json".to_owned(), 0xf4);
    }
    #[test]
    fn run_processor_test_f5() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f5.json".to_owned(), 0xf5);
    }
    #[test]
    fn run_processor_test_f6() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f6.json".to_owned(), 0xf6);
    }
    #[test]
    fn run_processor_test_f7() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f7.json".to_owned(), 0xf7);
    }
    #[test]
    fn run_processor_test_f8() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f8.json".to_owned(), 0xf8);
    }
    #[test]
    fn run_processor_test_f9() {
    
        run_processor_test("external/ProcessorTests/6502/v1/f9.json".to_owned(), 0xf9);
    }
    #[test]
    fn run_processor_test_fa() {
    
        run_processor_test("external/ProcessorTests/6502/v1/fa.json".to_owned(), 0xfa);
    }
    #[test]
    fn run_processor_test_fb() {
    
        run_processor_test("external/ProcessorTests/6502/v1/fb.json".to_owned(), 0xfb);
    }
    #[test]
    fn run_processor_test_fc() {
    
        run_processor_test("external/ProcessorTests/6502/v1/fc.json".to_owned(), 0xfc);
    }
    #[test]
    fn run_processor_test_fd() {
    
        run_processor_test("external/ProcessorTests/6502/v1/fd.json".to_owned(), 0xfd);
    }
    #[test]
    fn run_processor_test_fe() {
    
        run_processor_test("external/ProcessorTests/6502/v1/fe.json".to_owned(), 0xfe);
    }
    #[test]
    fn run_processor_test_ff() {
    
        run_processor_test("external/ProcessorTests/6502/v1/ff.json".to_owned(), 0xff);
    }
}
