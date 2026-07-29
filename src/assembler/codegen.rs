use std::collections::HashMap;
use std::collections::hash_map::Entry;

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

    let out = format!("{:016b}", instruction_as_integer); // Format as binary and return
    out
}

/// Splits 3 blocks by = and ; according to the format dest=comp;jump e.g. AM=M-D;JGT.
/// Parses each block accordingly and converts to binary number.
/// Each block represents a different slice of bits of the binary instruction, so we can just add
/// as numbers to binary number b to concatenate. Parse as binary string afterwards
/// 
/// 111 (convention) maps to b[0..2]: constant for C-instructions.
/// jump instruction maps to b[2..9]
/// oper instruction maps to b[10..12]
/// dest instruction maps to b[13..15]
pub fn c_instruction(line: &str) -> String {

    let (dest_instruction, rhs) = line
        .split_once("=")
        .unwrap_or(("", line));

    let (oper_instruction, jump_instruction) = rhs
        .split_once(";")
        .unwrap_or((rhs,""));

    let instruction_as_integer: u16
        = (0b1110_0000_0000_0000 as u16)
        + dest(&dest_instruction)
        + oper(&oper_instruction)
        + jump(&jump_instruction);

    let out = format!("{:016b}", instruction_as_integer); // Format as binary and return

    out
}

fn dest(code_string: &str) -> u16 {
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

fn jump(code_string: &str) -> u16 {
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

fn oper(code_string: &str) -> u16 {
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


pub struct SymbolTable {
    pub inner: HashMap<String, u16>,
    next_allocable_address: u16,
}

impl SymbolTable {
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

        match self.inner.insert(key, value) {
            None => (),
            Some(_) => panic!("The key \"{}\" is invalid: key already exists in HashMap", key_copy)
        }  
    }
}