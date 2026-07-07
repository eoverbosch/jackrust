use jackrust::parser;
use jackrust::vm_parser;
use code::*;
use code::labels;
use std::fs;



fn main() {
    let config = vm_parser::Config::init().unwrap_or_else(|e| e.exit());
    let line_reader = config.make_file_reader()
    .unwrap_or_else(|e|
        panic!("Failed to read {} failed with error: {e}", &config.input.as_ref().unwrap().display())
    );

    // Run pre-processor
    let code_lines = line_reader
        .map(Result::unwrap) 
        .map(parser::strip_comments)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty());

    let mut output_code = String::new();
    let mut label_counter = LabelCounter::new();
    let mut function_def_tracker = FunctionDefTracker::new();

    for (line_number, line) in code_lines.enumerate() {
        let (operation_string, rhs) = line
            .split_once(char::is_whitespace)
            .unwrap_or((&line, ""));

        let (argument_string, index_string) = rhs
            .split_once(char::is_whitespace)
            .unwrap_or((&rhs, ""));

        let operation = VMOperation::parse(operation_string)
            .unwrap_or_else(|_| panic!(
                "Syntax error on line {line_number}: \"{line}\"\n{operation_string} is not a valid operation."
            ));

        match operation {
            VMOperation::Arithmetic(arithmetic_operation) => {

                if !argument_string.is_empty() || !index_string.is_empty() {
                    panic!(
                        "Syntax error on line {}: arithmetic operation takes no arguments", 
                        line_number
                    )
                }

                arithmetic_operation.emit_asm(&mut output_code)
            },

            VMOperation::Compare(compare_operation) => {
                compare_operation.emit_asm(
                    &mut output_code,
                    labels::generate_compare_label(config.get_input_name(), label_counter.get())
                )
            },

            VMOperation::StackOperation(stack_operation) => {
                let segment: Segment 
                    = Segment::parse(argument_string).unwrap();

                let index = index_string.parse::<u16>().unwrap();

                stack_operation.emit_asm(&mut output_code, segment, index, config.get_input_name())
            }

            VMOperation::BranchOperation(branch_operation) => {
                if !index_string.is_empty() {
                    panic!("Syntax error on line {}: branch operation takes no index arguments", 
                        line_number)
                }

                let branch_label_prefix: String
                    = function_def_tracker
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
                                line_number
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
                                line_number,
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
                            |_| panic!("Syntax error on line {}: 
                                Index string parsing failed for function operation {:?} {}",
                                line_number,
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
}


mod code {
    #[allow(dead_code)]
    pub mod labels {
        pub fn generate_static_label(filename: &str, index: &u16) -> String {
            filename.to_owned() + "." + &index.to_string()
        }

        pub fn generate_compare_label(filename: &str, id: u16) -> String {
            filename.to_owned() + "$cmp." + &id.to_string()
        }

        pub fn generate_return_label(function_label: &str, id: &str) -> String {
            function_label.to_owned() + "$ret." + id
        }

        pub fn generate_goto_label(function_label: String, id: &str) -> String {
            function_label.to_owned() + "$" + id
        }
    }


    /// FunctionDef represent a vm function definition (i.e. function function_label n_vars). 
    /// The inner fields map to function_label: String and n_vars: u16.
    #[derive(Debug)]

    pub struct FunctionDef(String);

    impl FunctionDef {
        pub fn new(function_label: &String) -> FunctionDef {
            FunctionDef(function_label.to_owned())
        }
    }

    #[derive(Debug)]

    pub struct FunctionDefTracker(Option<FunctionDef>);

    impl FunctionDefTracker {
        pub fn new() -> FunctionDefTracker {
            FunctionDefTracker(None)
        }

        pub fn process_define_statement(&mut self, function_label: &String) {
            self.0 = Some(FunctionDef::new(function_label));
        }        

        pub fn get_current_label(&self) -> Option<String> {
            self.0.as_ref().map(|def| def.0.to_owned())
        }
    }

    pub struct LabelCounter(u16);

    impl LabelCounter {
        pub fn new() -> LabelCounter {
            LabelCounter(0)
        }

        pub fn get(&mut self) -> u16 {
            self.0 += 1;
            self.0.clone() - 1
        }
    }

    #[derive(Debug)]
    pub struct InvalidSegmentError;

    #[derive(Debug)]
    pub struct NoAvailableMemlabelError;

    pub enum Segment {
        Constant,
        Local,
        Argument,
        Static,
        This,
        That,
        Temp,
        Pointer
    }

    impl Segment {
        pub fn parse(segment_string: &str)
        -> Result<Segment, InvalidSegmentError> {
            use Segment::*;

            let segment: Segment = match segment_string {
                "constant" => Constant,
                "local"    => Local,
                "argument" => Argument,
                "static"   => Static,
                "this"     => This,
                "that"     => That,
                "temp"     => Temp,
                "pointer"  => Pointer,

                _ => return Err(InvalidSegmentError)
            };

            Ok(segment)
        }

        pub fn get_memlabel(&self, filename: &str, index: &u16) -> Result<String, NoAvailableMemlabelError> {
            use Segment::*;

            let memlabel = match self {
                Local => "LCL".to_owned(),
                Argument => "ARG".to_owned(),
                This => "THIS".to_owned(),
                That => "THAT".to_owned(),
                Static => labels::generate_static_label(filename, index),
                Temp => {
                    if index > &8 {panic!("Temp index is outside segment bounds")}
                    (5 + index).to_string().to_owned()
                }, 
                Pointer => {
                    if index > &1 {panic!("pointer index is outside segment bounds")}
                    (3 + index).to_string().to_owned()
                },

                _ => return Err(NoAvailableMemlabelError)
            };

            Ok(memlabel)
        }
    }
    
    #[derive(Debug)]
    pub struct InvalidKeywordError;

    pub enum VMOperation {
        Arithmetic(ArithmeticOp),
        Compare(CompareOp),
        StackOperation(StackOp),
        BranchOperation(BranchOp),
        FunctionOperation(FunctionOp)
    }

    impl VMOperation {
        pub fn parse(operation_string: &str)
        -> Result<VMOperation, InvalidKeywordError> {
            
            use VMOperation::*;

            let operation = match operation_string {
                "add"  => Arithmetic(ArithmeticOp::Add),
                "sub"  => Arithmetic(ArithmeticOp::Subtract),
                "neg"  => Arithmetic(ArithmeticOp::Negate),
                "and"  => Arithmetic(ArithmeticOp::And),
                "or"   => Arithmetic(ArithmeticOp::Or),
                "not"  => Arithmetic(ArithmeticOp::Not),

                "eq"   => Compare(CompareOp::Equate),
                "gt"   => Compare(CompareOp::GreaterThan),
                "lt"   => Compare(CompareOp::LessThan),

                "push" => StackOperation(StackOp::Push),
                "pop"  => StackOperation(StackOp::Pop),

                "label"   => BranchOperation(BranchOp::Label),
                "goto"    => BranchOperation(BranchOp::Goto),
                "if-goto" => BranchOperation(BranchOp::IfGoto),

                "function" => FunctionOperation(FunctionOp::Function),
                "call"     => FunctionOperation(FunctionOp::Call),
                "return"   => FunctionOperation(FunctionOp::Return),

                _ => return Err(InvalidKeywordError)
            };

            Ok(operation)
        }
    }

    pub enum ArithmeticOp {
        Add,
        Subtract,
        And,
        Or,
        Negate,
        Not,
    }

    impl ArithmeticOp {
        pub fn emit_asm(&self, target: &mut String) {

            use std::fmt::Write;

            let _ = match self {
                // 2-parameter operations
                ArithmeticOp::Add | ArithmeticOp::Subtract 
                | ArithmeticOp::And | ArithmeticOp::Or 
                => writeln!(target, "@SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    {}",
                    self.insert_operation()
                ),

                // 1-parameter operation
                ArithmeticOp::Negate | ArithmeticOp::Not 
                => writeln!(target, "@SP\n\
                    A=M-1\n\
                    {}",
                    self.insert_operation()
                )
            };
        }

        fn insert_operation(&self) -> &str {
            match self {
                ArithmeticOp::Add => "M=D+M",
                ArithmeticOp::Subtract => "M=M-D",
                ArithmeticOp::And => "M=D&M",                
                ArithmeticOp::Or => "M=D|M",

                ArithmeticOp::Negate => "M=-M",
                ArithmeticOp::Not => "M=!M"
            }
        }
    }
    
    pub enum CompareOp {
        Equate,
        GreaterThan,
        LessThan
    }

    impl CompareOp {
        pub fn emit_asm(&self, target: &mut String, label: String) {

            use std::fmt::Write;
            writeln!(target, "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                D=M-D\n\
                M=-1\n\
                @{}\n\
                D;{}\n\
                @SP\n\
                A=M-1\n\
                M=0\n\
                ({})",
                label,
                self.insert_operation(),
                label
            )
            .expect("Failed to write to target in CompareOp::emit_asm")

        }

        fn insert_operation(&self) -> &str {
            match self {
                CompareOp::Equate => "JEQ",
                CompareOp::GreaterThan => "JGT",
                CompareOp::LessThan => "JLT",
            }
        }
    }

    pub enum StackOp {
        Push,
        Pop
    }

    impl StackOp {
        pub fn emit_asm(&self, target: &mut String, segment: Segment, index: u16, static_label: &str)-> () {

            use Segment::*;
            use std::fmt::Write;

            let _ = match (self, segment) {
                (StackOp::Push, Constant) => {
                    writeln!(target,"@{}\n\
                    D=A\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D",
                    index)
                }

                (StackOp::Push, seg @ ( Local | Argument | This | That )) => {
                    writeln!(target,"@{}\n\
                    D=A\n\
                    @{}\n\
                    A=D+M\n\
                    D=M\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D",
                    index,
                    seg.get_memlabel(static_label, &index).unwrap())
                },

                (StackOp::Push, seg @ ( Temp | Static | Pointer )) => {
                    writeln!(target,"@{}\n\
                    D=M\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D",
                    seg.get_memlabel(static_label, &index).unwrap())
                },

                (StackOp::Pop, Constant) => 
                    panic!("Cannot pop on virtual segment \"Constant\" "),

                (StackOp::Pop, seg @ ( Local | Argument | This | That )) => {
                    writeln!(target,"@{}\n\
                    D=A\n\
                    @{}\n\
                    D=D+M\n\
                    @R13\n\
                    M=D\n\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    @R13\n\
                    A=M\n\
                    M=D",
                    index,
                    seg.get_memlabel(static_label, &index).unwrap())
                },

                (StackOp::Pop, seg @ ( Temp | Static | Pointer )) => {
                    writeln!(target,"@{}\n\
                    D=A\n\
                    @R13\n\
                    M=D\n\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    @R13\n\
                    A=M\n\
                    M=D",
                    seg.get_memlabel(static_label, &index).unwrap())
                },
            };
        }
    }

    pub enum BranchOp {
        Label,
        Goto,
        IfGoto
    }

    impl BranchOp {
        pub fn emit_asm(&self, target: &mut String, branch_label: &String)-> () {
            use std::fmt::Write;

            let _ = match self {
                BranchOp::Label => {
                    writeln!(target,"({})",
                    branch_label)
                }

                BranchOp::Goto => {
                    writeln!(target,"@{}\n\
                    0;JMP",
                    branch_label)
                },

                BranchOp::IfGoto => {
                    writeln!(target,"@SP\n\
                    AM=M-1\n\
                    D=M\n\
                    @{}\n\
                    D;JNE", 
                    branch_label)
                }
            };
        }
    }

    #[derive(Debug)]
    pub enum FunctionOp {
        Call,
        Function,
        Return
    }

    impl FunctionOp {
        pub fn emit_asm(&self, target: &mut String, function_label: &String, n: u16, id: &String)-> () {
            use std::fmt::Write;

            let _ = match self {
                FunctionOp::Function => {
                    writeln!(target,"({})\n\
                    @SP\n\
                    D=M\n\
                    @LCL\n\
                    AM=D\n\
                    {}\
                    D=A\n\
                    @SP\n\
                    M=D",
                    function_label,
                    "M=0\nA=A+1\n".repeat(n as usize),
                    )
                }
        
                FunctionOp::Call => {
                    let return_label = labels::generate_return_label(function_label, id);

                    writeln!(target,"@ARG\n\
                    D=M\n\
                    @R14\n\
                    M=D\n\
                    @SP\n\
                    D=M\n\
                    @{}\n\
                    D=D-A\n\
                    @ARG\n\
                    M=D\n\
                    @{}\n\
                    D=A\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D\n\
                    @LCL\n\
                    D=M\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D\n\
                    @R14\n\
                    D=M\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D\n\
                    @THIS\n\
                    D=M\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D\n\
                    @THAT\n\
                    D=M\n\
                    @SP\n\
                    M=M+1\n\
                    A=M-1\n\
                    M=D\n\
                    @{}\n\
                    0;JMP\n\
                    ({})",
                    n,
                    return_label,
                    function_label,
                    return_label
                    )
                },

                FunctionOp::Return => 
                    writeln!(target,"@LCL\n\
                    D=M\n\
                    @5\n\
                    D=D-A\n\
                    @R14\n\
                    AM=D\n\
                    D=M\n\
                    @R13\n\
                    M=D\n\
                    @SP\n\
                    A=M-1\n\
                    D=M\n\
                    @ARG\n\
                    A=M\n\
                    M=D\n\
                    D=A+1\n\
                    @SP\n\
                    M=D\n\
                    @R14\n\
                    AM=M+1\n\
                    D=M\n\
                    @LCL\n\
                    M=D\n\
                    @R14\n\
                    AM=M+1\n\
                    D=M\n\
                    @ARG\n\
                    M=D\n\
                    @R14\n\
                    AM=M+1\n\
                    D=M\n\
                    @THIS\n\
                    M=D\n\
                    @R14\n\
                    AM=M+1\n\
                    D=M\n\
                    @THAT\n\
                    M=D\n\
                    @R13\n\
                    A=M\n\
                    0;JMP")
            };
        }
    }
}
