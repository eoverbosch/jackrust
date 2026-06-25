use jackrust::parser;
use code::*;
use std::fs;


fn main() {
    let input_file_path = parser::get_path_from_env();
    let filename = parser::get_filename_from_env();
    let file_line_buffer = parser::bufreader_from_path(&input_file_path)
        .expect("Unable to read .vm file. The following error was produced");

    // Run pre-processor
    let code_lines = file_line_buffer
        .map(Result::unwrap) 
        .map(parser::strip_comments)
        .filter(|line| !line.is_empty());

    // Parse code and push into output
    let mut output_code = String::new();
    let mut label_counter = LabelCounter::new();

    for (line_number, line) in code_lines.enumerate() {
        let (operation_string, rhs) = line
            .split_once(char::is_whitespace)
            .unwrap_or((&line, ""));

        let (segment_string, index_string) = rhs
            .split_once(char::is_whitespace)
            .unwrap_or((&rhs, ""));

        let operation = VMOperation::parse(operation_string)
            .unwrap_or_else(|_| panic!(
                "Syntax error on line {line_number}: \"{line}\"
                \n{operation_string} is not a valid operation."
            ));

        match operation {
            VMOperation::Arithmetic(arithmetic_operation) => {

                if !segment_string.is_empty() || !index_string.is_empty() {
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
                    code::generate_comp_label(&filename, label_counter.get())
                )
            },

            VMOperation::StackOperation(stack_operation) => {
                let segment: Segment 
                    = Segment::parse(segment_string).unwrap();

                let index = index_string.parse::<u16>().unwrap();

                stack_operation.emit_asm(&mut output_code, segment, index, &filename)

            }
        }
    }

    // Remove extra new line
    let output_code = output_code
        .strip_suffix('\n')
        .unwrap_or(&output_code);

    // Save to output file
    let output_file_path = parser::get_output_path_from_env(".vm", ".asm");

    fs::write(
        output_file_path,
        output_code
    )
    .expect("Unable to write to output file");
}


mod code {

    pub fn generate_comp_label(filename: &String, id: u16) -> String {
        filename.to_owned() + ".vmcomp." + &id.to_string()
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

        pub fn get_memlabel(&self, static_label: &String, index: &u16) -> Result<String, NoAvailableMemlabelError> {
            use Segment::*;



            let memlabel = match self {
                Local => "LCL".to_owned(),
                Argument => "ARG".to_owned(),
                This => "THIS".to_owned(),
                That => "THAT".to_owned(),
                Static => static_label.to_owned() + "." + &index.to_string(),
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
        StackOperation(StackOp)
    }

    impl VMOperation {
        pub fn parse(operation_string: &str)
        -> Result<VMOperation, InvalidKeywordError> {
            
            use ArithmeticOp::*;
            use CompareOp::*;
            use StackOp::*;
            let operation = match operation_string {
                "add"  => VMOperation::Arithmetic(Add),
                "sub"  => VMOperation::Arithmetic(Subtract),
                "neg"  => VMOperation::Arithmetic(Negate),
                "and"  => VMOperation::Arithmetic(And),
                "or"   => VMOperation::Arithmetic(Or),
                "not"  => VMOperation::Arithmetic(Not),

                "eq"   => VMOperation::Compare(Equate),
                "gt"   => VMOperation::Compare(GreaterThan),
                "lt"   => VMOperation::Compare(LessThan),

                "push" => VMOperation::StackOperation(Push),
                "pop"  => VMOperation::StackOperation(Pop),

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
        pub fn emit_asm(&self, target: &mut String, segment: Segment, index: u16, static_label: &String)-> () {

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
}
