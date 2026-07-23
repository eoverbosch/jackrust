use jackrust::vm::callgraph::*;
use jackrust::vm::cli;
use jackrust::vm::codegen::*;
use jackrust::vm::codegen::labels;
use jackrust::vm::parser;
use jackrust::vm::parser::{FunctionDefTracker, LabelCounter};

use std::fs;  
use std::ffi::OsStr;


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let config = cli::Config::init().unwrap_or_else(|e| e.exit());

    // Discovery
    let mut vm_files: Vec<CachedFile> = config.path
        .read_dir()?
        .filter_map(Result::ok)
        .map(|entry| CachedFile::from_path(entry.path()))
        .filter(|cached_file| cached_file.filepath.extension() == Some(OsStr::new("vm")))
        .collect();

    vm_files.iter().find(
        |file| file.filepath.file_name() == Some(OsStr::new("Sys.vm"))
    ).ok_or("Sys.vm not found in current directory")?;

    vm_files.iter().find(
        |file| file.filepath.file_name() == Some(OsStr::new("Main.vm"))
    ).ok_or("Main.vm not found in current directory")?;

    let callgraph = CallGraph::from_entrypoint(cli::ENTRYPOINT.to_owned(), &mut vm_files)?;
    let line_reader = callgraph.build_function_reader(cli::ENTRYPOINT, &vm_files)?;
    
    const BOOTSTRAP_CODE: &[&str] = &[
        "INIT SP 256",
        "call Sys.init 0"
    ];
        
    // Run pre-processor
    let code_lines = BOOTSTRAP_CODE.iter().copied()
    .chain(line_reader.into_iter().flatten())
    .map(parser::strip_comments)
    .map(|line| line.trim().to_string())
    .enumerate()
    .filter(|(_, line)| !line.is_empty());
    
    if config.dump {
        fs::write(
            config.output
                .clone()
                .expect("Unable to clone config.output for debug")
                .with_extension("vm"),
            code_lines
                .clone()
                .map(|(_, line)| line)
                .collect::<Vec<_>>()
                .join("\n")
        )
        .expect("Unable to write debug dump");
    }

    let mut output_code = String::new();
    let mut label_counter = LabelCounter::new();
    let mut function_def_tracker = FunctionDefTracker::new();

    for (line_number, line) in code_lines {
        let (operation_string, rhs) = line
            .split_once(char::is_whitespace)
            .unwrap_or((&line, ""));

        let (argument_string, index_string) = rhs
            .split_once(char::is_whitespace)
            .unwrap_or((&rhs, ""));

        let operation = VMOperation::parse(operation_string)
            .unwrap_or_else(|_| panic!(
                "Syntax error on line {}: \"{line}\"\n{operation_string} is not a valid operation.",
                line_number + 1,
            ));

        match operation {
            VMOperation::Initialisation(init_operation) => {
                let address_pointer = AddressPointer::parse(argument_string)
                    .expect("Invalid address pointer label in INIT call");

                let index: u16 = index_string.parse()
                    .expect("Invalid value in INIT call");

                init_operation.emit_asm(&mut output_code, address_pointer, index);
            }

            VMOperation::Arithmetic(arithmetic_operation) => {

                if !argument_string.is_empty() || !index_string.is_empty() {
                    panic!(
                        "Syntax error on line {}: arithmetic operation takes no arguments", 
                        line_number+1
                    )
                }

                arithmetic_operation.emit_asm(&mut output_code)
            },

            VMOperation::Compare(compare_operation) => {
                let label = function_def_tracker
                    .get_current_label()
                    .unwrap_or(config.get_input_name().to_owned());

                compare_operation.emit_asm(
                    &mut output_code,
                    labels::generate_compare_label(&label, label_counter.get())
                )
            },

            VMOperation::StackOperation(stack_operation) => {
                let segment: Segment = Segment::parse(argument_string).unwrap();
                let index = index_string.parse::<u16>().unwrap();

                let static_prefix = function_def_tracker
                    .get_current_label()
                    .and_then(|label| label.split_once('.').map(|(file, _)| file.to_owned()))
                    .expect("stack operation outside function body");

                stack_operation.emit_asm(&mut output_code, segment, index, &static_prefix)
            }

            VMOperation::BranchOperation(branch_operation) => {
                if !index_string.is_empty() {
                    panic!("Syntax error on line {}: branch operation takes no index arguments", 
                        line_number+1)
                }

                let branch_label_prefix: String = function_def_tracker
                    .get_current_label()
                    .unwrap_or(config.get_input_name().to_owned());

                let branch_label = labels::generate_goto_label(
                    branch_label_prefix, 
                    &argument_string.to_owned()
                );

                branch_operation.emit_asm(&mut output_code, &branch_label); 
            }

            VMOperation::FunctionOperation(function_operation) => {
                match function_operation {
                    FunctionOp::Return => {
                        if !argument_string.is_empty() || !index_string.is_empty() {
                            panic!("Syntax error on line {}: function operation \"return\" takes no arguments", 
                                line_number+1
                            );
                        }

                        function_operation.emit_asm(
                            &mut output_code, 
                            &argument_string.to_owned(),
                            0,
                            &"".to_string()
                        )
                    },

                    FunctionOp::Function => {
                        let index = index_string.parse::<u16>().unwrap_or_else(
                            |_| panic!("Syntax error on line {}: 
                                Index string parsing failed for function operation {:?} {}",
                                line_number+1,
                                function_operation,
                                argument_string
                            )
                        );

                        function_def_tracker.process_define_statement(&argument_string.to_owned());
                            
                        function_operation.emit_asm(
                            &mut output_code, 
                            &argument_string.to_owned(),
                            index,
                            &"".to_string()
                        )
                    },

                    FunctionOp::Call => {
                        let index = index_string.parse::<u16>().unwrap_or_else(
                            |_| panic!("Syntax error on line {}: \
                                Index string parsing failed for function operation {:?} {}",
                                line_number+1,
                                function_operation,
                                argument_string
                            )
                        );

                        function_operation.emit_asm(
                        &mut output_code, 
                        &argument_string.to_owned(),
                        index,
                        &label_counter.get().to_string()
                    )}    
                }
            }
        }
    }

    // Remove extra new line
    let output_code = output_code
        .strip_suffix('\n')
        .unwrap_or(&output_code);
 

    fs::write(
        config.output.unwrap(),
        output_code
    )
    .expect("Unable to write to output file");

    Ok(())
}
