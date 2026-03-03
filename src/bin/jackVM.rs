use std::str::FromStr;

use jackrust::parser;
use code::Operation;

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
        let op: Operation = Operation::from_str(line)
            .unwrap();
        
        
    }
}


mod code {
    use std::str::FromStr;

    #[derive(Debug)]
    pub struct InvalidKeywordError;

    pub enum Operation {
        Arithmetic(ArithmeticOp),
        StackOperation(StackOp),
    }

    impl Operation {
        fn decode(full_code_line: &str) 
            -> Result<Operation, InvalidKeywordError> {

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

            match operation_string {
                // Arithmetics
                "add" => Ok(Operation::Arithmetic(ArithmeticOp::Add)),
                "sub" => Ok(Operation::Arithmetic(ArithmeticOp::Subtract)),
                "neg" => Ok(Operation::Arithmetic(ArithmeticOp::Negate)),
                "eq"  => Ok(Operation::Arithmetic(ArithmeticOp::Equate)),
                "gt"  => Ok(Operation::Arithmetic(ArithmeticOp::GreaterThan)),
                "lt"  => Ok(Operation::Arithmetic(ArithmeticOp::LessThan)),
                "and" => Ok(Operation::Arithmetic(ArithmeticOp::And)),
                "or"  => Ok(Operation::Arithmetic(ArithmeticOp::Or)),
                "not" => Ok(Operation::Arithmetic(ArithmeticOp::Not)),

                // Stack operations
                "push" => Ok(Operation::StackOperation(
                    StackOp::Push{
                        segment: arg1, 
                        index: arg2
                    }
                )),

                "pop" => Ok(Operation::StackOperation(
                    StackOp::Pop{
                        segment: arg1, 
                        index: arg2
                    }
                )),

                _ => Err(InvalidKeywordError)
                
            }
        }   
    }

    pub enum ArithmeticOp {
        Add,
        Subtract,
        Negate,
        Equate,
        GreaterThan,
        LessThan,
        And,
        Or,
        Not,
    }

    impl ArithmeticOp {
        fn decode(keyword_string: &str) -> Result<ArithmeticOp, InvalidKeywordError> {
            match keyword_string {
                "add" => Ok(ArithmeticOp::Add),
                "sub" => Ok(ArithmeticOp::Subtract),
                "neg" => Ok(ArithmeticOp::Negate),
                "eq"  => Ok(ArithmeticOp::Equate),
                "gt"  => Ok(ArithmeticOp::GreaterThan),
                "lt"  => Ok(ArithmeticOp::LessThan),
                "and" => Ok(ArithmeticOp::And),
                "or"  => Ok(ArithmeticOp::Or),
                "not" => Ok(ArithmeticOp::Not),

                _ => Err(InvalidKeywordError)
            }
        }

        fn emit_asm(&self, target: &mut String) -> Result<(), std::fmt::Error> {
            use std::fmt::Write;

            match self {
                // 1-parameter operation
                ArithmeticOp::Negate | 
                ArithmeticOp::Not 
                => writeln!(
                    target, 
                    "@SP\n
                    A=M-1\n
                    {}",
                    self.build_operation_asm()),

                // 2-parameter operations
                ArithmeticOp::Add | ArithmeticOp::Subtract |
                ArithmeticOp::Equate | ArithmeticOp::GreaterThan |
                ArithmeticOp::LessThan | ArithmeticOp::And |
                ArithmeticOp::Or => 
                    writeln!(target, 
                    "@SP\n
                    AM=M-1\n
                    D=M\n
                    A=A-1\n
                    {}",
                    self.build_operation_asm()),
            }
        }

        fn build_operation_asm(&self) -> &str {
            match self {
                ArithmeticOp::Add => "M=D+M",
                ArithmeticOp::Subtract => "M=M-D",
                ArithmeticOp::Negate => "M=-M",
                ArithmeticOp::Equate => 
                    "M=M-D
                    ",
                ArithmeticOp::GreaterThan => "",
                ArithmeticOp::LessThan => "",
                ArithmeticOp::And => "M=D&M",
                ArithmeticOp::Or => "M=D|M",
                ArithmeticOp::Not => "M=!M"
            }
        }

    }

    pub enum StackOp {
        Push {
            segment: String,
            index: String
        },

        Pop {
            segment: String,
            index: String
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
    }

}
