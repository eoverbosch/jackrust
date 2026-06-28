use hack_assembler::SymbolTable;
use hack_assembler::code;
use std::fs;

use jackrust::parser;

fn main() {
    // Load file content
    let input_file_path = parser::get_path_from_env();
    let file_lines = parser::bufreader_from_path(&input_file_path)
        .expect("Unable to read .asm file. The following error was produced:");

    // Create output file to move file extension check upstream.
    use std::fmt::Write;
    let mut output_file_path = input_file_path
        .strip_suffix(".asm")
        .expect("Input file is not of type .asm")
        .to_string();

    // FIRST PASS: String preprocessing
    let mut symbol_table = SymbolTable::build_default();

    let mut processed_lines: String = String::new();
    let mut line_number_offset: u16 = 0; // Compensates for removed lines

    for (line_number, line_result) in file_lines.enumerate() {

        let line: String = parser::strip_whitespace_and_comments(
            line_result.unwrap()
        );
        
        // If L-instruction
        if let Some(unprocessed_statement) = line.strip_prefix("(") {
            let symbol_statement =
                unprocessed_statement
                .strip_suffix(")")
                .unwrap_or_else(
                    || panic!("Syntax error in symbol statement: Closing bracket missing or incorrectly placed")
                );

            symbol_table.insert_new(
                symbol_statement.to_string(),
                (line_number as u16) - line_number_offset
            );

            line_number_offset += 1;

        // else if empty
        } else if line.is_empty(){
            line_number_offset += 1;

        // else: normal instruction, add to processed lines buffer. Syntax errors in A/D instructions are treated in second pass
        } else {
            processed_lines.push_str(&line);
            processed_lines.push_str("\n");
        }
    }

    // SECOND PASS
    let mut final_lines: String = String::new();
    
    for line_str in processed_lines.lines() {
        let line = line_str;

        let instruction_string = match line.strip_prefix("@") {
            Some(stripped_line) => code::a_instruction(stripped_line, &mut symbol_table),
            None                => code::c_instruction(line)
        };

        final_lines.push_str(&instruction_string);
        final_lines.push_str("\n");
    }
    
    // Remove extra new line
    let final_lines = final_lines
        .strip_suffix('\n')
        .unwrap_or(&final_lines);

    // Write output to file
    output_file_path
    .write_str(".hack")
    .unwrap();

    fs::write(
        output_file_path,
        final_lines
    )
    .expect("Unable to write to output file");

    #[cfg(debug_assertions)]
    {
    println!("{}\n", processed_lines);
    println!("{:#?}\n", symbol_table.inner);
    println!("{}\n", final_lines);
    }
}


mod hack_assembler {
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;

    pub mod code {
        use super::SymbolTable;

        pub fn a_instruction(instruction: &str, symbol_table: &mut SymbolTable) -> String {
            let instruction_as_integer = match instruction.parse::<u16>() {
                Ok(value) => value, 
                Err(error) => match error.kind() {
                    std::num::IntErrorKind::PosOverflow => panic!("Invalid a-instruction due to 16-bit integer overflow"),

                    _ => symbol_table
                        .get_or_alloc(&instruction)
                        .clone()
                }
            };

            format!("{:016b}", instruction_as_integer) // Format as binary and return

        }

        pub fn c_instruction(line: &str) -> String{
            //Split 3 blocks by = and ;, set b[0..2] = 111
            //match first block: has A, has M, has D. If no =, 000
            //match second block: match to instruction table
            //match third block: match to jump table

            let (dest_instruction, rhs) = line
                .split_once("=")
                .unwrap_or(("", line));

            let (comp_instruction, jump_instruction) = rhs
                .split_once(";")
                .unwrap_or((rhs,""));

            let instruction_as_integer: u16
                = (0b1110_0000_0000_0000 as u16)
                + dest(&dest_instruction)
                + comp(&comp_instruction)
                + jump(&jump_instruction);

            format!("{:016b}", instruction_as_integer) // Format as binary and return
         
        }
        
        pub fn dest(code_string: &str) -> u16 {
            match code_string {
                ""    => 0b000_000,
                "M"   => 0b001_000,
                "D"   => 0b010_000,
                "MD"  => 0b011_000,
                "A"   => 0b100_000,
                "AM"  => 0b101_000,
                "AD"  => 0b110_000,
                "AMD" => 0b111_000,
                _ => panic!("Syntax error for dest mnemonic: {}", code_string)
            }
        }

        pub fn jump(code_string: &str) -> u16 {
            match code_string {
                ""    => 0b000,
                "JGT" => 0b001,
                "JEQ" => 0b010,
                "JGE" => 0b011,
                "JLT" => 0b100,
                "JNE" => 0b101,
                "JLE" => 0b110,
                "JMP" => 0b111,
                _ => panic!("Syntax error for jump mnemonic: {}", code_string),
            }
        }

        pub fn comp(code_string: &str) -> u16 {
            match code_string {
                // a = 0
                "0"   => 0b0000_101010_000000,
                "1"   => 0b0000_111111_000000,
                "-1"  => 0b0000_111010_000000,
                "D"   => 0b0000_001100_000000,
                "A"   => 0b0000_110000_000000,
                "!D"  => 0b0000_001101_000000,
                "!A"  => 0b0000_110001_000000,
                "-D"  => 0b0000_001111_000000,
                "-A"  => 0b0000_110011_000000,
                "D+1" => 0b0000_011111_000000,
                "A+1" => 0b0000_110111_000000,
                "D-1" => 0b0000_001110_000000,
                "A-1" => 0b0000_110010_000000,
                "D+A" => 0b0000_000010_000000,
                "D-A" => 0b0000_010011_000000,
                "A-D" => 0b0000_000111_000000,
                "D&A" => 0b0000_000000_000000,
                "D|A" => 0b0000_010101_000000,

                // a = 1
                "M"   => 0b0001_110000_000000,
                "!M"  => 0b0001_110001_000000,
                "-M"  => 0b0001_110011_000000,
                "M+1" => 0b0001_110111_000000,
                "M-1" => 0b0001_110010_000000,
                "D+M" => 0b0001_000010_000000,
                "D-M" => 0b0001_010011_000000,
                "M-D" => 0b0001_000111_000000,
                "D&M" => 0b0001_000000_000000,
                "D|M" => 0b0001_010101_000000,

                _ => panic!("Syntax error for comp mnemonic: {}", code_string),
            }
        }
    }

    pub struct SymbolTable {
        pub inner: HashMap<String, u16>,
        next_allocable_address: u16,
    }

    impl SymbolTable {
        #[allow(dead_code)]
        pub fn new() -> SymbolTable {
            let new_hashmap: HashMap<String, u16> = HashMap::new();

            SymbolTable {
                inner: new_hashmap,
                next_allocable_address: 0,
            }
        }

        pub fn build_default() -> SymbolTable {
            /*
            Implements addresses for default parameters:
            R0-16: 0-16
            SP:    0
            LCL:   1
            ARG:   2
            THIS:  3
            THAT:  4
            SCREEN: 16384
            KBD: 24576
             */

            let mut symbol_map: HashMap<String, u16> = HashMap::new();

            symbol_map.insert("R0".to_string(),   0);
            symbol_map.insert("R1".to_string(),   1);
            symbol_map.insert("R2".to_string(),   2);
            symbol_map.insert("R3".to_string(),   3);
            symbol_map.insert("R4".to_string(),   4);
            symbol_map.insert("R5".to_string(),   5);
            symbol_map.insert("R6".to_string(),   6);
            symbol_map.insert("R7".to_string(),   7);
            symbol_map.insert("R8".to_string(),   8);
            symbol_map.insert("R9".to_string(),   9);
            symbol_map.insert("R10".to_string(), 10);
            symbol_map.insert("R11".to_string(), 11);
            symbol_map.insert("R12".to_string(), 12);
            symbol_map.insert("R13".to_string(), 13);
            symbol_map.insert("R14".to_string(), 14);
            symbol_map.insert("R15".to_string(), 15);

            symbol_map.insert("SP".to_string(),   0);
            symbol_map.insert("LCL".to_string(),  1);
            symbol_map.insert("ARG".to_string(),  2);
            symbol_map.insert("THIS".to_string(), 3);
            symbol_map.insert("THAT".to_string(), 4);

            symbol_map.insert("SCREEN".to_string(), 16384);
            symbol_map.insert("KBD".to_string(), 24576);

            SymbolTable {
                inner: symbol_map,
                next_allocable_address: 16,
            }
        }

        pub fn get(&self, key: &str) -> Option<&u16> {
            self.inner.get(&key.to_string())
        }

        pub fn get_or_alloc(&mut self, key: &str) -> &u16 {
            let local_entry = self.inner.entry(key.to_string());
            let is_occupied: bool;

            match local_entry {
                // we match into a boolean because Rust's borrowing system is being annoying
                Entry::Occupied(_entry) => {
                    is_occupied = true;
                }
                Entry::Vacant(_entry) => {
                    is_occupied = false;
                }
            }

            if is_occupied {
                self.inner.get(key).unwrap()
            } 
            else {
                let addr = self.allocate_address();
                self.inner.insert(key.to_string(), addr);
                self.get(key).unwrap()
            }
        }

        fn allocate_address(&mut self) -> u16 {
            self.next_allocable_address += 1;
            self.next_allocable_address - 1
        }

        pub fn insert_new(&mut self, key: String, value: u16) {
            let key_copy: String = key.clone();

            match self.inner
            .insert(key, value) {
                None => (),
                Some(_) => panic!("The key \"{}\" is invalid: key already exists in HashMap", key_copy)
            }
            
        }

    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn symboltable_default_init() {
            let symbol_table = SymbolTable::build_default();

            assert_eq!(symbol_table.inner.get("R5").unwrap(), &5);
            assert_eq!(symbol_table.inner.get("THIS").unwrap(), &3);
        }

        #[test]
        fn symboltable_get() {
            let mut symbol_table = SymbolTable::new();

            symbol_table.inner.insert('a'.to_string(), 1);
            symbol_table.inner.insert('b'.to_string(), 10);

            assert_eq!(symbol_table.get("a").unwrap(), &1);
            assert_eq!(symbol_table.get("b").unwrap(), &10);
            assert!(symbol_table.get("c").is_none());
        }

        #[test]
        fn symboltable_get_or_alloc() {
            let mut symbol_table = SymbolTable::new();

            symbol_table.inner.insert('a'.to_string(), 1);

            assert_eq!(symbol_table.get_or_alloc("a"), &1);
            assert_eq!(symbol_table.get_or_alloc("b"), &0);
        }

        #[test]
        fn code_dest() {
            // exhaustive
            assert_eq!(code::dest(""),    0b0000000000_000_000u16);
            assert_eq!(code::dest("D"),   0b0000000000_001_000u16);
            assert_eq!(code::dest("M"),   0b0000000000_010_000u16);
            assert_eq!(code::dest("MD"),  0b0000000000_011_000u16);
            assert_eq!(code::dest("A"),   0b0000000000_100_000u16);
            assert_eq!(code::dest("AD"),  0b0000000000_101_000u16);
            assert_eq!(code::dest("AM"),  0b0000000000_110_000u16);
            assert_eq!(code::dest("AMD"), 0b0000000000_111_000u16);
        }

        #[test]
        fn code_jump() {
            assert_eq!(code::jump(""),    0b000);
            assert_eq!(code::jump("JGT"), 0b001);
            assert_eq!(code::jump("JEQ"), 0b010);
            assert_eq!(code::jump("JGE"), 0b011);
            assert_eq!(code::jump("JLT"), 0b100);
            assert_eq!(code::jump("JNE"), 0b101);
            assert_eq!(code::jump("JLE"), 0b110);
            assert_eq!(code::jump("JMP"), 0b111);
        }

        #[test]
         fn code_comp() {
            assert_eq!(code::comp("0"),   0b0000_101010_000000);
            assert_eq!(code::comp("1"),   0b0000_111111_000000);
            assert_eq!(code::comp("-1"),  0b0000_111010_000000);
            assert_eq!(code::comp("D"),   0b0000_001100_000000);
            assert_eq!(code::comp("A"),   0b0000_110000_000000);
            assert_eq!(code::comp("!D"),  0b0000_001101_000000);
            assert_eq!(code::comp("!A"),  0b0000_110001_000000);
            assert_eq!(code::comp("-D"),  0b0000_001111_000000);
            assert_eq!(code::comp("-A"),  0b0000_110011_000000);
            assert_eq!(code::comp("D+1"), 0b0000_011111_000000);
            assert_eq!(code::comp("A+1"), 0b0000_110111_000000);
            assert_eq!(code::comp("D-1"), 0b0000_001110_000000);
            assert_eq!(code::comp("A-1"), 0b0000_110010_000000);
            assert_eq!(code::comp("D+A"), 0b0000_000010_000000);
            assert_eq!(code::comp("D-A"), 0b0000_010011_000000);
            assert_eq!(code::comp("A-D"), 0b0000_000111_000000);
            assert_eq!(code::comp("D&A"), 0b0000_000000_000000);
            assert_eq!(code::comp("D|A"), 0b0000_010101_000000);
    
            assert_eq!(code::comp("M"),   0b0001_110000_000000);
            assert_eq!(code::comp("!M"),  0b0001_110001_000000);
            assert_eq!(code::comp("-M"),  0b0001_110011_000000);
            assert_eq!(code::comp("M+1"), 0b0001_110111_000000);
            assert_eq!(code::comp("M-1"), 0b0001_110010_000000);
            assert_eq!(code::comp("D+M"), 0b0001_000010_000000);
            assert_eq!(code::comp("D-M"), 0b0001_010011_000000);
            assert_eq!(code::comp("M-D"), 0b0001_000111_000000);
            assert_eq!(code::comp("D&M"), 0b0001_000000_000000);
            assert_eq!(code::comp("D|M"), 0b0001_010101_000000);
        }
    }
}
