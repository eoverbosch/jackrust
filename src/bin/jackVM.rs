use jackrust::parser;

fn main() {
    let input_file_path = parser::get_path_from_env();
    let file_lines = parser::bufreader_from_path(&input_file_path)
        .expect("Unable to read .vm file. The following error was produced");

    // Run pre-processor
    let mut processed_lines: String = String::new();

    for line_result in file_lines {

        let line: String = parser::strip_comments(
            line_result
            .unwrap()
        );

        if !line.is_empty() {
            processed_lines.push_str(&line);
            processed_lines.push_str("\n")
        }
    }

    // Run code gen    
    let mut final_lines: String = String::new();

    for line in processed_lines.lines() {
        let (operation, rhs) =
        line
        .split_once(
            char::is_whitespace
        )
        .unwrap_or(
            (line,"")
        );

        match operation

    }

    // For every line in processed lines
        // (Parse line)
        // Parse first keyword
        // if arth: print code directly
        // if push/pop:
        // Match second statement to a specific codestring
        // Pass codestring and argument to write!("code{}etc{}", ..., ...)

}


pub mod code {
    enum Operation {
        Arithmetic,
        StackOp
    }

    impl Operation {
        fn from_string(operation_string: &String) {
            
            operation_string
        }
    }

}