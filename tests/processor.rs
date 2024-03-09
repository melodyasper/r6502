use r6502::emulator::{DefaultVirtualMemory, Emulator, EmulatorBuilder, VirtualMemory};
use r6502::instructions::{Instruction, OpCode};
use r6502::state::{SystemAction, SystemCycle, SystemFlags, SystemState};

use serde_json::Value;
use strum::IntoEnumIterator;
use tabled::builder::Builder;
use tabled::settings::themes::ColumnNames;
use tabled::settings::Style;
use tabled::Table;
use std::fs::File;
use std::io::Read;
use colored::Colorize;



fn json_to_state(state_map: &Value, key: &str, include_cycles: bool) -> Emulator<DefaultVirtualMemory>  {
    let state = SystemState {
        pc: state_map[key]["pc"].as_u64().unwrap() as u16,
        a:  state_map[key]["a"].as_u64().unwrap() as u8,
        x:  state_map[key]["x"].as_u64().unwrap() as u8,
        y:  state_map[key]["y"].as_u64().unwrap() as u8,
        s:  state_map[key]["s"].as_u64().unwrap() as u8,
        p: SystemFlags::from_bits_retain(state_map[key]["p"].as_u64().unwrap() as u8),
        running: true,
        cycles: Default::default()
    };


    let mut emulator = EmulatorBuilder::default().memory(DefaultVirtualMemory::default()).state(state).build().unwrap();

    for memory in state_map[key]["ram"].as_array().unwrap().iter() {
        let memory = memory.as_array().unwrap();
        let address = memory.first().unwrap().as_u64().unwrap() as u16;
        let value = memory.get(1).unwrap().as_u64().unwrap() as u8;
        emulator.write(address, value);
    }
    emulator.state.cycles.clear();

    if include_cycles {
        for cycle in state_map["cycles"].as_array().unwrap().iter() {
            let memory = cycle.as_array().unwrap();
            
            let address = memory.first().unwrap().as_u64().unwrap() as u16;
            let value = memory.get(1).unwrap().as_u64().unwrap() as u8;
            let operation = memory.get(2).unwrap().as_str().unwrap();
            match operation {
                "read" => {
                    emulator.state.cycles.push(SystemCycle {address, value, action: SystemAction::READ})
                },
                "write" => {
                    emulator.state.cycles.push(SystemCycle {address, value, action: SystemAction::WRITE})
                }
                unknown => {
                    panic!("Unknown rules for serializing cycle {}", unknown)
                }
            }
        }
    } 

    emulator
}

fn debug_state_comparison(
    initial_state: &mut Emulator<DefaultVirtualMemory>,
    final_state: &mut Emulator<DefaultVirtualMemory>,
    tested_state: &mut Emulator<DefaultVirtualMemory>,
    strict: bool,
    print_me: bool,
) -> bool {
    
    // let (final_vec, tested_vec) = {
    //     let mut final_vec = vec![0u8; 0x10000];
    //     let mut tested_vec = vec![0u8; 0x10000];
    //     for i in 0..0x10000 {
    //         final_vec[i] = final_state.read(i as u16);
    //         tested_vec[i] = tested_state.read(i as u16);
    //     }
    //     (final_vec, tested_vec)
    // };
    let (final_vec, tested_vec) = (final_state.iter_memory(), tested_state.iter_memory());

    let result = {
            final_state.state.pc == tested_state.state.pc &&
            final_state.state.a == tested_state.state.a &&
            final_state.state.s == tested_state.state.s &&
            final_state.state.x == tested_state.state.x &&
            final_state.state.y == tested_state.state.y &&
            final_state.state.p == tested_state.state.p &&
            final_vec.clone().zip(tested_vec.clone()).filter(|&(a, b)| a == b).count() != 0
    };
    if !result && print_me {
        let mut table = Table::new(vec![("initial state", (&*initial_state).state.clone()), ("tested state", (&*tested_state).state.clone()), ("final state", (&*final_state).state.clone())]);
        table.with(Style::modern());
        println!("{}", table);

        let mvec: Vec<Vec<String>> = final_vec
            .clone()
            .into_iter()
            .zip(tested_vec.clone())
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
        let mut it_xs = tested_state.state.cycles.iter();
        let mut it_ys = final_state.state.cycles.iter();
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
        let mut initial_state = json_to_state(value, "initial", false);
        let mut tested_state = json_to_state(value, "initial", false);
        let mut final_state = json_to_state(value, "final", true);
        final_state.state.running = false;
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

        if debug_state_comparison(&mut initial_state, &mut final_state, &mut tested_state, false, failable) {
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
                let result = run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, false);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_iny() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::INY {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
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
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}


#[test]
fn test_all_cpy() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::CPY {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_jmp() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::JMP {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_rti() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::RTI {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_rts() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::RTS {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_bcs() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::BCS {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_jsr() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::JSR {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}

#[test]
fn test_all_sbc() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for (ibyte, instruction) in instructions.iter().enumerate() {
        if instruction.opcode == OpCode::SBC {
            println!("{}", instruction);
            run_processor_test(format!("external/ProcessorTests/nes6502/v1/{:02x}.json", ibyte), ibyte as u8, true);
        }
    }
}
/* Needed still
SBC
*/

