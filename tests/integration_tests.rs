use std::fs::File;
use std::io::Read;
use serde_json::{Result, Value};


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

#[test]
fn test_folder_a1() {
    let mut file = File::open("external/ProcessorTests/6502/v1/a0.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let v: Value = serde_json::from_str(&contents).unwrap();
    for value in v.as_array().unwrap().into_iter() {
        println!("Data {}", value);
    }
    
    assert_eq!(4, 2+1);
}



// {
// 	"name": "b1 28 b5",
// 	"initial": {
// 		"pc": 59082,
// 		"s": 39,
// 		"a": 57,
// 		"x": 33,
// 		"y": 174,
// 		"p": 96,
// 		"ram": [
// 			[59082, 177],
// 			[59083, 40],
// 			[59084, 181],
// 			[40, 160],
// 			[41, 233],
// 			[59982, 119]
// 		]
// 	},
// 	"final": {
// 		"pc": 59084,
// 		"s": 39,
// 		"a": 119,
// 		"x": 33,
// 		"y": 174,
// 		"p": 96,
// 		"ram": [
// 			[40, 160],
// 			[41, 233],
// 			[59082, 177],
// 			[59083, 40],
// 			[59084, 181],
// 			[59982, 119]
// 		]
// 	},
// 	"cycles": [
// 		[59082, 177, "read"],
// 		[59083, 40, "read"],
// 		[40, 160, "read"],
// 		[41, 233, "read"],
// 		[59083, 40, "read"],
// 		[59982, 119, "read"]
// 	]
// }