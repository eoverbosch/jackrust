use jackrust::assembler::cli;
use jackrust::assembler::codegen;
use jackrust::assembler::codegen::SymbolTable;
use jackrust::assembler::parser;

use std::fs;

fn main() {
    let config = cli::Config::init().unwrap_or_else(|e| e.exit());

    // FIRST PASS: String preprocessing
    let mut symbol_table = SymbolTable::build_default();

    let mut processed_lines: String = String::new();
    let mut line_number_offset: u16 = 0; // Compensates for removed lines

    for (line_number, line_result) in config.make_file_reader().unwrap().enumerate() {

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

        // else: normal instruction, add to processed lines buffer
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
            Some(stripped_line) => codegen::a_instruction(stripped_line, &mut symbol_table),
            None                => codegen::c_instruction(line)
        };

        final_lines.push_str(&instruction_string);
        final_lines.push_str("\n");
    }
    
    // Remove extra new line
    let final_lines = final_lines
        .strip_suffix('\n')
        .unwrap_or(&final_lines);

    // Write output to file    
    fs::write(
        config.output.as_ref().unwrap(),
        final_lines
    )
    .expect("Unable to write to output file");

    if config.dump {
        fs::write(
            &config.output.unwrap().with_extension("dump"),
            format!("{}\n\nSymbol table:\n{:#?}", processed_lines, symbol_table.inner)
        )
        .expect("Unable to dump assembler data to file");
    }
}
