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

fn json_to_state(state_map: &Value, key: &str, include_cycles: bool) -> SystemState {
    let mut state = SystemState {
        pc: state_map[key]["pc"].as_u64().unwrap() as u16,
        a:  state_map[key]["a"].as_u64().unwrap() as u8,
        x:  state_map[key]["x"].as_u64().unwrap() as u8,
        y:  state_map[key]["y"].as_u64().unwrap() as u8,
        s:  state_map[key]["s"].as_u64().unwrap() as u8,
        p: SystemFlags::from_bits_retain(state_map[key]["p"].as_u64().unwrap() as u8),
        m: vec![0; 0x10000],
        running: false,
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
    state_expected: &mut SystemState,
    state: &mut SystemState,
    strict: bool,
    print_me: bool,
) -> bool {
    let result = match strict {
        true => state_expected == state,
        false => {
            state_expected.pc == state.pc &&
            state_expected.a == state.a &&
            state_expected.s == state.s &&
            state_expected.x == state.x &&
            state_expected.y == state.y &&
            state_expected.p == state.p &&
            state_expected.m == state.m
        }
    };
    if !result && print_me {
        let mut table = Table::new(vec![("final state", &*state), ("expected state", &*state_expected)]);
        table.with(Style::modern());
        println!("{}", table);

        let mvec: Vec<Vec<String>> = state_expected
            .m
            .clone()
            .into_iter()
            .zip(state.m.clone())
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
        let mut it_xs = state.cycles.iter();
        let mut it_ys = state_expected.cycles.iter();
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

fn run_processor_test(filename: String, instruction: u8) {
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
        let mut state = json_to_state(value, "initial", false);
        let mut final_state = json_to_state(value, "final", true);
        // println!("Start state: {}", state.pc());

        match state.execute_next_instruction() {
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

        if debug_state_comparison(&mut final_state, &mut state, false, true) {
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

    println!(
        "{:#04x} tests passed: {}/{}",
        instruction, tests_passed, tests_total
    );
    assert!(tests_passed == tests_total);
}

#[test]
fn test_all_instructions_groupwise() {
    let mut instructions = vec![];
    for ibyte in 0..255u8 {
        let instruction = Instruction::from(ibyte);
        instructions.push(instruction);
    }
    
    for opcode in OpCode::iter() {
        for (ibyte, instruction) in instructions.iter().enumerate() {
            if instruction.opcode == opcode {
                println!("{}", instruction);
                run_processor_test(format!("external/ProcessorTests/6502/v1/{:02x}.json", ibyte), 0x0)
            }
        }
    }
}

#[test]
fn instruction_0x0_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/00.json".to_owned(), 0x0)
}

#[test]
fn instruction_0x1_ora_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/01.json".to_owned(), 0x1)
}
#[test]
fn instruction_0x2_asl_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/02.json".to_owned(), 0x2)
}
#[test]
fn instruction_0x3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/03.json".to_owned(), 0x3)
}
#[test]
fn instruction_0x4_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/04.json".to_owned(), 0x4)
}
#[test]
fn instruction_0x5_ora_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/05.json".to_owned(), 0x5)
}
#[test]
fn instruction_0x6_asl_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/06.json".to_owned(), 0x6)
}
#[test]
fn instruction_0x7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/07.json".to_owned(), 0x7)
}
#[test]
fn instruction_0x8_php_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/08.json".to_owned(), 0x8)
}
#[test]
fn instruction_0x9_ora_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/09.json".to_owned(), 0x9)
}
#[test]
fn instruction_0xa_asl_in_accumulator_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/0a.json".to_owned(), 0xa)
}
#[test]
fn instruction_0xb_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/0b.json".to_owned(), 0xb)
}
#[test]
fn instruction_0xc_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/0c.json".to_owned(), 0xc)
}
#[test]
fn instruction_0xd_ora_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/0d.json".to_owned(), 0xd)
}
#[test]
fn instruction_0xe_asl_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/0e.json".to_owned(), 0xe)
}
#[test]
fn instruction_0xf_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/0f.json".to_owned(), 0xf)
}
#[test]
fn instruction_0x10_bpl_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/10.json".to_owned(), 0x10)
}
#[test]
fn instruction_0x11_ora_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/11.json".to_owned(), 0x11)
}
#[test]
fn instruction_0x12_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/12.json".to_owned(), 0x12)
}
#[test]
fn instruction_0x13_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/13.json".to_owned(), 0x13)
}
#[test]
fn instruction_0x14_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/14.json".to_owned(), 0x14)
}
#[test]
fn instruction_0x15_ora_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/15.json".to_owned(), 0x15)
}
#[test]
fn instruction_0x16_asl_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/16.json".to_owned(), 0x16)
}
#[test]
fn instruction_0x17_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/17.json".to_owned(), 0x17)
}
#[test]
fn instruction_0x18_clc_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/18.json".to_owned(), 0x18)
}
#[test]
fn instruction_0x19_ora_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/19.json".to_owned(), 0x19)
}
#[test]
fn instruction_0x1a_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/1a.json".to_owned(), 0x1a)
}
#[test]
fn instruction_0x1b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/1b.json".to_owned(), 0x1b)
}
#[test]
fn instruction_0x1c_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/1c.json".to_owned(), 0x1c)
}
#[test]
fn instruction_0x1d_ora_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/1d.json".to_owned(), 0x1d)
}
#[test]
fn instruction_0x1e_asl_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/1e.json".to_owned(), 0x1e)
}
#[test]
fn instruction_0x1f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/1f.json".to_owned(), 0x1f)
}
#[test]
fn instruction_0x20_bit_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/20.json".to_owned(), 0x20)
}
#[test]
fn instruction_0x21_and_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/21.json".to_owned(), 0x21)
}
#[test]
fn instruction_0x22_rol_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/22.json".to_owned(), 0x22)
}
#[test]
fn instruction_0x23_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/23.json".to_owned(), 0x23)
}
#[test]
fn instruction_0x24_bit_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/24.json".to_owned(), 0x24)
}
#[test]
fn instruction_0x25_and_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/25.json".to_owned(), 0x25)
}
#[test]
fn instruction_0x26_rol_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/26.json".to_owned(), 0x26)
}
#[test]
fn instruction_0x27_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/27.json".to_owned(), 0x27)
}
#[test]
fn instruction_0x28_plp_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/28.json".to_owned(), 0x28)
}
#[test]
fn instruction_0x29_and_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/29.json".to_owned(), 0x29)
}
#[test]
fn instruction_0x2a_rol_in_accumulator_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/2a.json".to_owned(), 0x2a)
}
#[test]
fn instruction_0x2b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/2b.json".to_owned(), 0x2b)
}
#[test]
fn instruction_0x2c_bit_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/2c.json".to_owned(), 0x2c)
}
#[test]
fn instruction_0x2d_and_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/2d.json".to_owned(), 0x2d)
}
#[test]
fn instruction_0x2e_rol_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/2e.json".to_owned(), 0x2e)
}
#[test]
fn instruction_0x2f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/2f.json".to_owned(), 0x2f)
}
#[test]
fn instruction_0x30_bmi_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/30.json".to_owned(), 0x30)
}
#[test]
fn instruction_0x31_and_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/31.json".to_owned(), 0x31)
}
#[test]
fn instruction_0x32_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/32.json".to_owned(), 0x32)
}
#[test]
fn instruction_0x33_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/33.json".to_owned(), 0x33)
}
#[test]
fn instruction_0x34_bit_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/34.json".to_owned(), 0x34)
}
#[test]
fn instruction_0x35_and_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/35.json".to_owned(), 0x35)
}
#[test]
fn instruction_0x36_rol_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/36.json".to_owned(), 0x36)
}
#[test]
fn instruction_0x37_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/37.json".to_owned(), 0x37)
}
#[test]
fn instruction_0x38_sec_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/38.json".to_owned(), 0x38)
}
#[test]
fn instruction_0x39_and_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/39.json".to_owned(), 0x39)
}
#[test]
fn instruction_0x3a_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/3a.json".to_owned(), 0x3a)
}
#[test]
fn instruction_0x3b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/3b.json".to_owned(), 0x3b)
}
#[test]
fn instruction_0x3c_bit_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/3c.json".to_owned(), 0x3c)
}
#[test]
fn instruction_0x3d_and_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/3d.json".to_owned(), 0x3d)
}
#[test]
fn instruction_0x3e_rol_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/3e.json".to_owned(), 0x3e)
}
#[test]
fn instruction_0x3f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/3f.json".to_owned(), 0x3f)
}
#[test]
fn instruction_0x40_jmp_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/40.json".to_owned(), 0x40)
}
#[test]
fn instruction_0x41_eor_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/41.json".to_owned(), 0x41)
}
#[test]
fn instruction_0x42_lsr_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/42.json".to_owned(), 0x42)
}
#[test]
fn instruction_0x43_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/43.json".to_owned(), 0x43)
}
#[test]
fn instruction_0x44_jmp_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/44.json".to_owned(), 0x44)
}
#[test]
fn instruction_0x45_eor_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/45.json".to_owned(), 0x45)
}
#[test]
fn instruction_0x46_lsr_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/46.json".to_owned(), 0x46)
}
#[test]
fn instruction_0x47_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/47.json".to_owned(), 0x47)
}
#[test]
fn instruction_0x48_pha_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/48.json".to_owned(), 0x48)
}
#[test]
fn instruction_0x49_eor_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/49.json".to_owned(), 0x49)
}
#[test]
fn instruction_0x4a_lsr_in_accumulator_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/4a.json".to_owned(), 0x4a)
}
#[test]
fn instruction_0x4b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/4b.json".to_owned(), 0x4b)
}
#[test]
fn instruction_0x4c_jmp_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/4c.json".to_owned(), 0x4c)
}
#[test]
fn instruction_0x4d_eor_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/4d.json".to_owned(), 0x4d)
}
#[test]
fn instruction_0x4e_lsr_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/4e.json".to_owned(), 0x4e)
}
#[test]
fn instruction_0x4f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/4f.json".to_owned(), 0x4f)
}
#[test]
fn instruction_0x50_bvc_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/50.json".to_owned(), 0x50)
}
#[test]
fn instruction_0x51_eor_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/51.json".to_owned(), 0x51)
}
#[test]
fn instruction_0x52_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/52.json".to_owned(), 0x52)
}
#[test]
fn instruction_0x53_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/53.json".to_owned(), 0x53)
}
#[test]
fn instruction_0x54_jmp_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/54.json".to_owned(), 0x54)
}
#[test]
fn instruction_0x55_eor_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/55.json".to_owned(), 0x55)
}
#[test]
fn instruction_0x56_lsr_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/56.json".to_owned(), 0x56)
}
#[test]
fn instruction_0x57_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/57.json".to_owned(), 0x57)
}
#[test]
fn instruction_0x58_cli_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/58.json".to_owned(), 0x58)
}
#[test]
fn instruction_0x59_eor_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/59.json".to_owned(), 0x59)
}
#[test]
fn instruction_0x5a_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/5a.json".to_owned(), 0x5a)
}
#[test]
fn instruction_0x5b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/5b.json".to_owned(), 0x5b)
}
#[test]
fn instruction_0x5c_jmp_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/5c.json".to_owned(), 0x5c)
}
#[test]
fn instruction_0x5d_eor_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/5d.json".to_owned(), 0x5d)
}
#[test]
fn instruction_0x5e_lsr_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/5e.json".to_owned(), 0x5e)
}
#[test]
fn instruction_0x5f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/5f.json".to_owned(), 0x5f)
}
#[test]
fn instruction_0x60_jmpabsolute_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/60.json".to_owned(), 0x60)
}
#[test]
fn instruction_0x61_adc_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/61.json".to_owned(), 0x61)
}
#[test]
fn instruction_0x62_ror_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/62.json".to_owned(), 0x62)
}
#[test]
fn instruction_0x63_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/63.json".to_owned(), 0x63)
}
#[test]
fn instruction_0x64_jmpabsolute_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/64.json".to_owned(), 0x64)
}
#[test]
fn instruction_0x65_adc_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/65.json".to_owned(), 0x65)
}
#[test]
fn instruction_0x66_ror_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/66.json".to_owned(), 0x66)
}
#[test]
fn instruction_0x67_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/67.json".to_owned(), 0x67)
}
#[test]
fn instruction_0x68_pla_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/68.json".to_owned(), 0x68)
}
#[test]
fn instruction_0x69_adc_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/69.json".to_owned(), 0x69)
}
#[test]
fn instruction_0x6a_ror_in_accumulator_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/6a.json".to_owned(), 0x6a)
}
#[test]
fn instruction_0x6b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/6b.json".to_owned(), 0x6b)
}
#[test]
fn instruction_0x6c_jmpabsolute_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/6c.json".to_owned(), 0x6c)
}
#[test]
fn instruction_0x6d_adc_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/6d.json".to_owned(), 0x6d)
}
#[test]
fn instruction_0x6e_ror_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/6e.json".to_owned(), 0x6e)
}
#[test]
fn instruction_0x6f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/6f.json".to_owned(), 0x6f)
}
#[test]
fn instruction_0x70_bvs_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/70.json".to_owned(), 0x70)
}
#[test]
fn instruction_0x71_adc_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/71.json".to_owned(), 0x71)
}
#[test]
fn instruction_0x72_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/72.json".to_owned(), 0x72)
}
#[test]
fn instruction_0x73_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/73.json".to_owned(), 0x73)
}
#[test]
fn instruction_0x74_jmpabsolute_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/74.json".to_owned(), 0x74)
}
#[test]
fn instruction_0x75_adc_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/75.json".to_owned(), 0x75)
}
#[test]
fn instruction_0x76_ror_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/76.json".to_owned(), 0x76)
}
#[test]
fn instruction_0x77_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/77.json".to_owned(), 0x77)
}
#[test]
fn instruction_0x78_sei_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/78.json".to_owned(), 0x78)
}
#[test]
fn instruction_0x79_adc_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/79.json".to_owned(), 0x79)
}
#[test]
fn instruction_0x7a_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/7a.json".to_owned(), 0x7a)
}
#[test]
fn instruction_0x7b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/7b.json".to_owned(), 0x7b)
}
#[test]
fn instruction_0x7c_jmpabsolute_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/7c.json".to_owned(), 0x7c)
}
#[test]
fn instruction_0x7d_adc_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/7d.json".to_owned(), 0x7d)
}
#[test]
fn instruction_0x7e_ror_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/7e.json".to_owned(), 0x7e)
}
#[test]
fn instruction_0x7f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/7f.json".to_owned(), 0x7f)
}
#[test]
fn instruction_0x80_sty_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/80.json".to_owned(), 0x80)
}
#[test]
fn instruction_0x81_sta_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/81.json".to_owned(), 0x81)
}
#[test]
fn instruction_0x82_stx_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/82.json".to_owned(), 0x82)
}
#[test]
fn instruction_0x83_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/83.json".to_owned(), 0x83)
}
#[test]
fn instruction_0x84_sty_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/84.json".to_owned(), 0x84)
}
#[test]
fn instruction_0x85_sta_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/85.json".to_owned(), 0x85)
}
#[test]
fn instruction_0x86_stx_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/86.json".to_owned(), 0x86)
}
#[test]
fn instruction_0x87_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/87.json".to_owned(), 0x87)
}
#[test]
fn instruction_0x88_dey_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/88.json".to_owned(), 0x88)
}
#[test]
fn instruction_0x89_sta_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/89.json".to_owned(), 0x89)
}
#[test]
fn instruction_0x8a_txa_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/8a.json".to_owned(), 0x8a)
}
#[test]
fn instruction_0x8b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/8b.json".to_owned(), 0x8b)
}
#[test]
fn instruction_0x8c_sty_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/8c.json".to_owned(), 0x8c)
}
#[test]
fn instruction_0x8d_sta_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/8d.json".to_owned(), 0x8d)
}
#[test]
fn instruction_0x8e_stx_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/8e.json".to_owned(), 0x8e)
}
#[test]
fn instruction_0x8f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/8f.json".to_owned(), 0x8f)
}
#[test]
fn instruction_0x90_bcc_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/90.json".to_owned(), 0x90)
}
#[test]
fn instruction_0x91_sta_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/91.json".to_owned(), 0x91)
}
#[test]
fn instruction_0x92_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/92.json".to_owned(), 0x92)
}
#[test]
fn instruction_0x93_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/93.json".to_owned(), 0x93)
}
#[test]
fn instruction_0x94_sty_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/94.json".to_owned(), 0x94)
}
#[test]
fn instruction_0x95_sta_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/95.json".to_owned(), 0x95)
}
#[test]
fn instruction_0x96_stx_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/96.json".to_owned(), 0x96)
}
#[test]
fn instruction_0x97_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/97.json".to_owned(), 0x97)
}
#[test]
fn instruction_0x98_tya_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/98.json".to_owned(), 0x98)
}
#[test]
fn instruction_0x99_sta_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/99.json".to_owned(), 0x99)
}
#[test]
fn instruction_0x9a_txs_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/9a.json".to_owned(), 0x9a)
}
#[test]
fn instruction_0x9b_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/9b.json".to_owned(), 0x9b)
}
#[test]
fn instruction_0x9c_sty_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/9c.json".to_owned(), 0x9c)
}
#[test]
fn instruction_0x9d_sta_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/9d.json".to_owned(), 0x9d)
}
#[test]
fn instruction_0x9e_stx_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/9e.json".to_owned(), 0x9e)
}
#[test]
fn instruction_0x9f_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/9f.json".to_owned(), 0x9f)
}
#[test]
fn instruction_0xa0_ldy_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a0.json".to_owned(), 0xa0)
}
#[test]
fn instruction_0xa1_lda_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a1.json".to_owned(), 0xa1)
}
#[test]
fn instruction_0xa2_ldx_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a2.json".to_owned(), 0xa2)
}
#[test]
fn instruction_0xa3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/a3.json".to_owned(), 0xa3)
}
#[test]
fn instruction_0xa4_ldy_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a4.json".to_owned(), 0xa4)
}
#[test]
fn instruction_0xa5_lda_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a5.json".to_owned(), 0xa5)
}
#[test]
fn instruction_0xa6_ldx_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a6.json".to_owned(), 0xa6)
}
#[test]
fn instruction_0xa7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/a7.json".to_owned(), 0xa7)
}
#[test]
fn instruction_0xa8_tay_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a8.json".to_owned(), 0xa8)
}
#[test]
fn instruction_0xa9_lda_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/a9.json".to_owned(), 0xa9)
}
#[test]
fn instruction_0xaa_tax_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/aa.json".to_owned(), 0xaa)
}
#[test]
fn instruction_0xab_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/ab.json".to_owned(), 0xab)
}
#[test]
fn instruction_0xac_ldy_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ac.json".to_owned(), 0xac)
}
#[test]
fn instruction_0xad_lda_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ad.json".to_owned(), 0xad)
}
#[test]
fn instruction_0xae_ldx_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ae.json".to_owned(), 0xae)
}
#[test]
fn instruction_0xaf_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/af.json".to_owned(), 0xaf)
}
#[test]
fn instruction_0xb0_bcs_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b0.json".to_owned(), 0xb0)
}
#[test]
fn instruction_0xb1_lda_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b1.json".to_owned(), 0xb1)
}
#[test]
fn instruction_0xb2_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/b2.json".to_owned(), 0xb2)
}
#[test]
fn instruction_0xb3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/b3.json".to_owned(), 0xb3)
}
#[test]
fn instruction_0xb4_ldy_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b4.json".to_owned(), 0xb4)
}
#[test]
fn instruction_0xb5_lda_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b5.json".to_owned(), 0xb5)
}
#[test]
fn instruction_0xb6_ldx_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b6.json".to_owned(), 0xb6)
}
#[test]
fn instruction_0xb7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/b7.json".to_owned(), 0xb7)
}
#[test]
fn instruction_0xb8_clv_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b8.json".to_owned(), 0xb8)
}
#[test]
fn instruction_0xb9_lda_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/b9.json".to_owned(), 0xb9)
}
#[test]
fn instruction_0xba_tsx_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ba.json".to_owned(), 0xba)
}
#[test]
fn instruction_0xbb_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/bb.json".to_owned(), 0xbb)
}
#[test]
fn instruction_0xbc_ldy_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/bc.json".to_owned(), 0xbc)
}
#[test]
fn instruction_0xbd_lda_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/bd.json".to_owned(), 0xbd)
}
#[test]
fn instruction_0xbe_ldx_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/be.json".to_owned(), 0xbe)
}
#[test]
fn instruction_0xbf_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/bf.json".to_owned(), 0xbf)
}
#[test]
fn instruction_0xc0_cpy_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c0.json".to_owned(), 0xc0)
}
#[test]
fn instruction_0xc1_cmp_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c1.json".to_owned(), 0xc1)
}
#[test]
fn instruction_0xc2_dec_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c2.json".to_owned(), 0xc2)
}
#[test]
fn instruction_0xc3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/c3.json".to_owned(), 0xc3)
}
#[test]
fn instruction_0xc4_cpy_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c4.json".to_owned(), 0xc4)
}
#[test]
fn instruction_0xc5_cmp_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c5.json".to_owned(), 0xc5)
}
#[test]
fn instruction_0xc6_dec_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c6.json".to_owned(), 0xc6)
}
#[test]
fn instruction_0xc7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/c7.json".to_owned(), 0xc7)
}
#[test]
fn instruction_0xc8_iny_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c8.json".to_owned(), 0xc8)
}
#[test]
fn instruction_0xc9_cmp_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/c9.json".to_owned(), 0xc9)
}
#[test]
fn instruction_0xca_dex_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ca.json".to_owned(), 0xca)
}
#[test]
fn instruction_0xcb_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/cb.json".to_owned(), 0xcb)
}
#[test]
fn instruction_0xcc_cpy_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/cc.json".to_owned(), 0xcc)
}
#[test]
fn instruction_0xcd_cmp_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/cd.json".to_owned(), 0xcd)
}
#[test]
fn instruction_0xce_dec_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ce.json".to_owned(), 0xce)
}
#[test]
fn instruction_0xcf_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/cf.json".to_owned(), 0xcf)
}
#[test]
fn instruction_0xd0_bne_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d0.json".to_owned(), 0xd0)
}
#[test]
fn instruction_0xd1_cmp_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d1.json".to_owned(), 0xd1)
}
#[test]
fn instruction_0xd2_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/d2.json".to_owned(), 0xd2)
}
#[test]
fn instruction_0xd3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/d3.json".to_owned(), 0xd3)
}
#[test]
fn instruction_0xd4_cpy_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d4.json".to_owned(), 0xd4)
}
#[test]
fn instruction_0xd5_cmp_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d5.json".to_owned(), 0xd5)
}
#[test]
fn instruction_0xd6_dec_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d6.json".to_owned(), 0xd6)
}
#[test]
fn instruction_0xd7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/d7.json".to_owned(), 0xd7)
}
#[test]
fn instruction_0xd8_cld_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d8.json".to_owned(), 0xd8)
}
#[test]
fn instruction_0xd9_cmp_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/d9.json".to_owned(), 0xd9)
}
#[test]
fn instruction_0xda_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/da.json".to_owned(), 0xda)
}
#[test]
fn instruction_0xdb_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/db.json".to_owned(), 0xdb)
}
#[test]
fn instruction_0xdc_cpy_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/dc.json".to_owned(), 0xdc)
}
#[test]
fn instruction_0xdd_cmp_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/dd.json".to_owned(), 0xdd)
}
#[test]
fn instruction_0xde_dec_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/de.json".to_owned(), 0xde)
}
#[test]
fn instruction_0xdf_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/df.json".to_owned(), 0xdf)
}
#[test]
fn instruction_0xe0_cpx_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e0.json".to_owned(), 0xe0)
}
#[test]
fn instruction_0xe1_sbc_in_indirectzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e1.json".to_owned(), 0xe1)
}
#[test]
fn instruction_0xe2_inc_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e2.json".to_owned(), 0xe2)
}
#[test]
fn instruction_0xe3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/e3.json".to_owned(), 0xe3)
}
#[test]
fn instruction_0xe4_cpx_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e4.json".to_owned(), 0xe4)
}
#[test]
fn instruction_0xe5_sbc_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e5.json".to_owned(), 0xe5)
}
#[test]
fn instruction_0xe6_inc_in_directzeropage_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e6.json".to_owned(), 0xe6)
}
#[test]
fn instruction_0xe7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/e7.json".to_owned(), 0xe7)
}
#[test]
fn instruction_0xe8_inx_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e8.json".to_owned(), 0xe8)
}
#[test]
fn instruction_0xe9_sbc_in_immediate_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/e9.json".to_owned(), 0xe9)
}
#[test]
fn instruction_0xea_nop_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ea.json".to_owned(), 0xea)
}
#[test]
fn instruction_0xeb_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/eb.json".to_owned(), 0xeb)
}
#[test]
fn instruction_0xec_cpx_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ec.json".to_owned(), 0xec)
}
#[test]
fn instruction_0xed_sbc_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ed.json".to_owned(), 0xed)
}
#[test]
fn instruction_0xee_inc_in_directabsolute_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/ee.json".to_owned(), 0xee)
}
#[test]
fn instruction_0xef_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/ef.json".to_owned(), 0xef)
}
#[test]
fn instruction_0xf0_beq_in_relative_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f0.json".to_owned(), 0xf0)
}
#[test]
fn instruction_0xf1_sbc_in_indirectzeropagey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f1.json".to_owned(), 0xf1)
}
#[test]
fn instruction_0xf2_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/f2.json".to_owned(), 0xf2)
}
#[test]
fn instruction_0xf3_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/f3.json".to_owned(), 0xf3)
}
#[test]
fn instruction_0xf4_cpx_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f4.json".to_owned(), 0xf4)
}
#[test]
fn instruction_0xf5_sbc_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f5.json".to_owned(), 0xf5)
}
#[test]
fn instruction_0xf6_inc_in_directzeropagex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f6.json".to_owned(), 0xf6)
}
#[test]
fn instruction_0xf7_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/f7.json".to_owned(), 0xf7)
}
#[test]
fn instruction_0xf8_sed_with_no_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f8.json".to_owned(), 0xf8)
}
#[test]
fn instruction_0xf9_sbc_in_directabsolutey_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/f9.json".to_owned(), 0xf9)
}
#[test]
fn instruction_0xfa_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/fa.json".to_owned(), 0xfa)
}
#[test]
fn instruction_0xfb_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/fb.json".to_owned(), 0xfb)
}
#[test]
fn instruction_0xfc_cpx_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/fc.json".to_owned(), 0xfc)
}
#[test]
fn instruction_0xfd_sbc_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/fd.json".to_owned(), 0xfd)
}
#[test]
fn instruction_0xfe_inc_in_directabsolutex_mode() {
    run_processor_test("external/ProcessorTests/6502/v1/fe.json".to_owned(), 0xfe)
}
#[test]
fn instruction_0xff_not_valid() {
    run_processor_test("external/ProcessorTests/6502/v1/ff.json".to_owned(), 0xff)
}
