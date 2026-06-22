use std::str::FromStr;

use jackrust::parser;
use code::Operation;

use crate::code::ArithmeticOp;


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
    let file_line_buffer = parser::bufreader_from_path(&input_file_path)
        .expect("Unable to read .vm file. The following error was produced");

    // Run pre-processor
    let code_lines = file_line_buffer
        .map(Result::unwrap) 
        .map(parser::strip_comments)
        .filter(String::is_empty);

    let mut output_code: String = String::new();

    for (line_number, line) in code_lines.enumerate() {
        let (operation_string, rhs) = line
            .split_once(char::is_whitespace)
            .unwrap_or((&line, ""));

        let operation = Operation::parse(operation_string)
            .unwrap_or_else(|_| panic!(
                "Syntax error on line {line_number}: \"{line}\"
                \n{operation_string} is not a valid operation."
            ));

        let (segment_string, index_string) = rhs
            .split_once(char::is_whitespace)
            .unwrap_or((&rhs, ""));

        match operation {
            Operation::Arithmetic(arithmetic_operation) => 
                arithmetic_operation.emit_asm(&mut output_code),

            Operation::StackOperation(stack_operation) => {
                let segment = segment_string.parse();
                let index = index_string.parse::<u16>().unwrap();
                stack_operation.emit_asm(&mut output_code)
            }
        }


    }
}


mod code {
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

    pub enum Operation {
        Arithmetic(ArithmeticOp),
        StackOperation(StackOp),
    }

    impl Operation {
        pub fn parse(full_code_line: &str) 
            -> Result<Operation, InvalidKeywordError> {
            use ArithmeticOp::*;
            use StackOp::*;

            let (operation_string, rhs) = full_code_line
            .split_once(char::is_whitespace)
            .unwrap_or(
                (full_code_line, "")
            );

            let (arg1, arg2) = rhs
            .split_once(char::is_whitespace)
            .map(|(dest, i)| (dest.to_string(), i.to_string()))
            .unwrap_or(
                (rhs.to_string(), "".to_string())
            );

            let operation: Operation = match operation_string {
                // Arithmetics
                "add" => Operation::Arithmetic(Add),
                "sub" => Operation::Arithmetic(Subtract),
                "neg" => Operation::Arithmetic(Negate),
                "eq"  => Operation::Arithmetic(Equate),
                "gt"  => Operation::Arithmetic(GreaterThan),
                "lt"  => Operation::Arithmetic(LessThan),
                "and" => Operation::Arithmetic(And),
                "or"  => Operation::Arithmetic(Or),
                "not" => Operation::Arithmetic(Not),

                // Stack operations
                "push" => Operation::StackOperation(Push {
                    segment: arg1, 
                    index: arg2
                }),

                "pop" => Operation::StackOperation(Pop {
                    segment: arg1, 
                    index: arg2
                }),

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
        Equate,
        GreaterThan,
        LessThan,
    }

    impl ArithmeticOp {
        fn decode(keyword_string: &str) -> Result<ArithmeticOp, InvalidKeywordError> {
            match keyword_string {
                "add" => Ok(ArithmeticOp::Add),
                "sub" => Ok(ArithmeticOp::Subtract),
                "and" => Ok(ArithmeticOp::And),
                "or"  => Ok(ArithmeticOp::Or),

                "neg" => Ok(ArithmeticOp::Negate),
                "not" => Ok(ArithmeticOp::Not),

                "eq"  => Ok(ArithmeticOp::Equate),
                "gt"  => Ok(ArithmeticOp::GreaterThan),
                "lt"  => Ok(ArithmeticOp::LessThan),

                _ => Err(InvalidKeywordError)
            }
        }

        pub fn emit_asm(&self, target: &mut String) -> Result<(), std::fmt::Error> {
            use std::fmt::Write;
            let label = "PLACEHOLDER";

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
                ArithmeticOp::Negate 
                | ArithmeticOp::Not 
                => writeln!(target, 
                    "@SP\n
                    A=M-1\n
                    {}",
                    self.build_operation_asm()
                ),

                // Comparison operations
                ArithmeticOp::Equate
                | ArithmeticOp::GreaterThan | ArithmeticOp::LessThan
                => writeln!(target,
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
            }
        }

        fn build_operation_asm(&self) -> &str {
            match self {
                ArithmeticOp::Add => "M=D+M",
                ArithmeticOp::Subtract => "M=M-D",
                ArithmeticOp::And => "M=D&M",                
                ArithmeticOp::Or => "M=D|M",

                ArithmeticOp::Negate => "M=-M",
                ArithmeticOp::Not => "M=!M",

                ArithmeticOp::Equate => "JEQ",
                ArithmeticOp::GreaterThan => "JGT",
                ArithmeticOp::LessThan => "JLT",

            }
        }
    }

    pub enum StackOp {
        Push {
            segment: Segment,
            index: u16
        },

        Pop {
            segment: Segment,
            index: u16
        },
    }

    impl StackOp {
        fn decode(keyword_string: &str, segment_string: String, index_string: String) 
            -> Result<StackOp, InvalidKeywordError> {
            match keyword_string {
                "push" => Ok(
                    StackOp::Push{segment: segment_string, index: index_string}
                ),

                "pop" => Ok(
                    StackOp::Pop{segment: segment_string, index: index_string}
                ),

                _ => Err(InvalidKeywordError)
            }
        }

        pub fn emit_asm(&self, target: &mut String) 
            -> Result<(), std::fmt::Error> {
            match self {
                StackOp::Push => (),
                StackOp::Pop => ()
            }

        }
    }
}
