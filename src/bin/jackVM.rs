use jackrust::parser;
use code::*;
use std::fs;

// Big thoughts time: 
// Parse line and count line number
// Split first part and parse as Operation
// Split second, parse from string to Register (enum)
// Third: parse as integer u16
// Translate operation to binary and add to buffer

// if arithmetic op but the rest is not empty, syntax error
// match against different arithmetic operations and emit assembly
// 

fn main() {
    let input_file_path = parser::get_path_from_env();
    let filename = parser::get_filename_from_env();
    let file_line_buffer = parser::bufreader_from_path(&input_file_path)
        .expect("Unable to read .vm file. The following error was produced");

    // Run pre-processor
    let code_lines = file_line_buffer
        .map(Result::unwrap) 
        .map(parser::strip_comments)
        .filter(String::is_empty);

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
            code::Arithmetic(arithmetic_operation) => {

                if !segment_string.is_empty() || !index_string.is_empty() {
                    panic!("Syntax error on line {}", line_number)
                }

                arithmetic_operation.emit_asm(&mut output_code)
            },

            code::Compare(compare_operation) => {
                compare_operation.emit_asm(
                    &mut output_code,
                    generate_label(filename, label_counter.get())
                )
            },

            code::StackOperation(stack_operation) => {
                let segment: Result<Segment, InvalidKeywordError> = segment_string.parse();
                let index = index_string.parse::<u16>().unwrap();

                stack_operation.emit_asm(&mut output_code)
            }
        }
    }

    // Remove extra new line
    let output_code = output_code
        .strip_suffix('\n')
        .unwrap_or(&output_code);

    // Save to output file
    let output_file_path = parser::get_output_path_from_env(".vm", ".hack");

    fs::write(
        output_file_path,
        output_code
    )
    .expect("Unable to write to output file");
}


mod code {
    fn generate_label(filename: String, id: u16) -> String {
        filename + ".comp." + &id.to_string()
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
    }
    
    #[derive(Debug)]
    pub struct InvalidKeywordError;

    pub enum VMOperation {
        Arithmetic(ArithmeticOp),
        Compare(CompareOp),
        StackOperation(StackOp)
    }

    pub struct Arithmetic(ArithmeticOp);

    impl Arithmetic {
        fn parse(operation_string: &str) 
            -> Result<Arithmetic, InvalidKeywordError> {
            use ArithmeticOp::*;

            let operation = match operation_string {
                "add" => Arithmetic(Add),
                "sub" => Arithmetic(Subtract),
                "neg" => Arithmetic(Negate),
                "and" => Arithmetic(And),
                "or"  => Arithmetic(Or),
                "not" => Arithmetic(Not),

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

            match self {
                // 2-parameter operations
                ArithmeticOp::Add | ArithmeticOp::Subtract 
                | ArithmeticOp::And | ArithmeticOp::Or 
                => writeln!(target, 
                    "@SP\n
                    AM=M-1\n
                    D=M\n
                    A=A-1\n
                    {}",
                    self.build_operation_asm()
                ),

                // 1-parameter operation
                ArithmeticOp::Negate | ArithmeticOp::Not 
                => writeln!(target, 
                    "@SP\n
                    A=M-1\n
                    {}",
                    self.build_operation_asm()
                )
            };
        }

        fn build_operation_asm(&self) -> &str {
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
    
    pub struct Compare(CompareOp);
    
    impl Compare {
        fn parse(operation_string: &str) -> Result<Compare, InvalidKeywordError> {
            use CompareOp::*;
            
            let operation = match operation_string {
                "eq"  => Compare(Equate),
                "gt"  => Compare(GreaterThan),
                "lt"  => Compare(LessThan),

                _ => return Err(InvalidKeywordError)
            };

            Ok(operation)
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
            writeln!(target,
                "@SP\n
                AM=M-1\n
                D=M\n
                A=A-1\n
                D=M-D\n
                M=-1\n
                @{}\n
                D; {}\n
                @SP\n
                A=M-1\n
                M=0\n
                ({})
                ",
                label,
                self.build_operation_asm(),
                label
            )
            .expect("Failed to write to target in CompareOp::emit_asm")
        }

        fn build_operation_asm(&self) -> &str {
            match self {
                CompareOp::Equate => "JEQ",
                CompareOp::GreaterThan => "JGT",
                CompareOp::LessThan => "JLT",
            }
        }
    }

    pub struct StackOperation(StackOp);
    
    impl StackOperation {
        fn parse(operation_string: &str) 
        -> Result<Self, InvalidKeywordError> {
            use StackOp::*;

            let operation = match operation_string {
                "push" => StackOperation(Push),
                "pop" => StackOperation(Pop),
                
                _ => return Err(InvalidKeywordError)
            };

            Ok(operation)
        }   
    }

    pub enum StackOp {
        Push,
        Pop
    }

    impl StackOp {
        pub fn emit_asm(&self, target: &mut String, ) {
            match self {
                _ => ()
            };
        }
    }
}
