use r6502::emulator::instructions::{Instruction, OpCode};
use r6502::emulator::state::{SystemAction, SystemCycle, SystemFlags, SystemState};

use serde_json::Value;
use strum::IntoEnumIterator;
use tabled::builder::Builder;
use tabled::settings::themes::ColumnNames;
use tabled::settings::Style;
use tabled::Table;
use std::fs::File;
use std::io::Read;
use colored::Colorize;

fn json_to_state(state_map: &Value, key: &str, include_cycles: bool) -> SystemState {
    let mut state = SystemState {
        pc: state_map[key]["pc"].as_u64().unwrap() as u16,
        a:  state_map[key]["a"].as_u64().unwrap() as u8,
        x:  state_map[key]["x"].as_u64().unwrap() as u8,
        y:  state_map[key]["y"].as_u64().unwrap() as u8,
        s:  state_map[key]["s"].as_u64().unwrap() as u8,
        p: SystemFlags::from_bits_retain(state_map[key]["p"].as_u64().unwrap() as u8),
        m: vec![0; 0x10000],
        running: true,
        cycles: Default::default()
    };


    for memory in state_map[key]["ram"].as_array().unwrap().iter() {
        let memory = memory.as_array().unwrap();
        let address = memory.first().unwrap().as_u64().unwrap() as u16;
        let value = memory.get(1).unwrap().as_u64().unwrap() as u8;
        state.write(address, value).unwrap();
    }
    state.cycles.clear();

    if include_cycles {
        for cycle in state_map["cycles"].as_array().unwrap().iter() {
            let memory = cycle.as_array().unwrap();
            
            let address = memory.first().unwrap().as_u64().unwrap() as u16;
            let value = memory.get(1).unwrap().as_u64().unwrap() as u8;
            let operation = memory.get(2).unwrap().as_str().unwrap();
            match operation {
                "read" => {
                    state.cycles.push(SystemCycle {address, value, action: SystemAction::READ})
                },
                "write" => {
                    state.cycles.push(SystemCycle {address, value, action: SystemAction::WRITE})
                }
                unknown => {
                    panic!("Unknown rules for serializing cycle {}", unknown)
                }
            }
        }
    } 

    state
}

fn debug_state_comparison(
    initial_state: &SystemState,
    final_state: &SystemState,
    tested_state: &mut SystemState,
    strict: bool,
    print_me: bool,
) -> bool {
    let result = match strict {
        true => final_state == tested_state,
        false => {
            final_state.pc == tested_state.pc &&
            final_state.a == tested_state.a &&
            final_state.s == tested_state.s &&
            final_state.x == tested_state.x &&
            final_state.y == tested_state.y &&
            final_state.p == tested_state.p &&
            final_state.m == tested_state.m
        }
    };
    if !result && print_me {
        let mut table = Table::new(vec![("initial state", &*initial_state), ("tested state", &*tested_state), ("final state", &*final_state)]);
        table.with(Style::modern());
        println!("{}", table);

        let mvec: Vec<Vec<String>> = final_state
            .m
            .clone()
            .into_iter()
            .zip(tested_state.m.clone())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(
                |(addr, (exp, fin))| {
                    vec![addr.to_string(), exp.to_string(), fin.to_string()]
        })
            .collect();

        let mut table = Builder::from(mvec).build();
        table.with(Style::modern());
        table.with(ColumnNames::new(["Memory Location", "Expected", "Final"]));
        println!("{}", table);

        let mut cycle_comparison: Vec<Vec<String>> = vec![];
        let mut it_xs = tested_state.cycles.iter();
        let mut it_ys = final_state.cycles.iter();
        loop {
            match (it_xs.next(), it_ys.next()) {
                (Some(x), Some(y)) => {
                    cycle_comparison.push(vec![x.to_string(), y.to_string()])
                },
                (Some(x), None) => {
                    cycle_comparison.push(vec![x.to_string(), "None".to_owned()])
                },  
                (None, Some(y)) => {
                    cycle_comparison.push(vec!["None".to_owned(), y.to_string()])
                }, 
                (None, None) => break,
            }
        }
        
        let mut table = Builder::from(cycle_comparison).build();
        table.with(Style::modern());
        table.with(ColumnNames::new(["Final", "Expected"]));
        println!("{}", table);


        println!();
    }

    result
}

fn run_processor_test(filename: String, instruction: u8, failable: bool) -> bool {
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let v: Value = serde_json::from_str(&contents).unwrap();
    let mut tests_total = 0;
    let mut tests_passed = 0;
    let mut unknown_instructions: Vec<_> = Vec::new();
    let mut unfinished_instructions: Vec<_> = Vec::new();
    // TODO: Remove take, this is to speed up testing.
    for value in v.as_array().unwrap().iter().take(100) {
        tests_total += 1;
        let initial_state = json_to_state(value, "initial", false);
        let mut tested_state = json_to_state(value, "initial", false);
        let mut final_state = json_to_state(value, "final", true);
        final_state.running = false;
        // println!("Start state: {}", state.pc());

        match tested_state.execute_next_instruction() {
            Ok(_) => (),
            Err(Some(instruction)) => match instruction.opcode {
                OpCode::UnknownInstruction => {
                    if !unknown_instructions.contains(&instruction) {
                        unknown_instructions.push(instruction);
                    }
                }
                OpCode::BadInstruction => (),
                _ => {
                    if !unfinished_instructions.contains(&instruction) {
                        unfinished_instructions.push(instruction);
                    }
                }
            },
            Err(None) => {}
        }

        if debug_state_comparison(&initial_state, &final_state, &mut tested_state, false, failable) {
            tests_passed += 1;
        } else {
            break;
        }
    }
    for i in unknown_instructions.iter() {
        println!("Unknown Instruction {:?}", i);
    }
    for i in unfinished_instructions.iter() {
        println!("The following instruction isnt implemented: {:?}", i);
    }

    if failable {
        println!(
            "{:#04x} tests passed: {}/{}",
            instruction, tests_passed, tests_total
        );
        assert!(tests_passed == tests_total);
    }
    tests_passed == tests_total
}

#[test]
fn test_all_instructions_groupwise() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    let mut total = 0;
    let mut passed = 0;
    for opcode in OpCode::iter() {
        for (ibyte, instruction) in instructions.iter().enumerate() {
            if instruction.opcode == opcode {
                total += 1;
                print!("{}: ", instruction);
                let result = run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, false);
                if result == true {
                    passed += 1;
                    println!("{}", "Passed".green());
                }
                else {
                    println!("{}", "Failed".red());
                }
            }
        }
    }
    assert_eq!(total, passed);
}

#[test]
fn test_all_cmp() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::CMP {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_ldx() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::LDX {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_rol() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::ROL {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_ror() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::ROR {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_lsr() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::LSR {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_tya() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::TYA {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}



#[test]
fn test_all_tsx() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::TSX {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_asl() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::ASL {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_sty() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::STY {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_stx() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::STX {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_ldy() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::LDY {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_cpx() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::CPX {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_bit() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::BIT {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


// fixme

#[test]
fn test_all_php() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::PHP {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_plp() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::PLP {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_pla() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::PLA {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_dey() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::DEY {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_dex() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::DEX {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_inx() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::INX {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_txs() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::TXS {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_brk() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::BRK {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}