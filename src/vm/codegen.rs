use std::fmt::Write;

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
    Pointer,
}

impl Segment {
    pub fn parse(segment_string: &str) -> Result<Segment, InvalidSegmentError> {
        let segment: Segment = match segment_string {
            "constant" => Segment::Constant,
            "local" => Segment::Local,
            "argument" => Segment::Argument,
            "static" => Segment::Static,
            "this" => Segment::This,
            "that" => Segment::That,
            "temp" => Segment::Temp,
            "pointer" => Segment::Pointer,

            _ => return Err(InvalidSegmentError),
        };

        Ok(segment)
    }

    pub fn get_memlabel(
        &self,
        filename: &str,
        index: &u16,
    ) -> Result<String, NoAvailableMemlabelError> {
        use Segment::*;

        let memlabel = match self {
            Local => "LCL".to_owned(),
            Argument => "ARG".to_owned(),
            This => "THIS".to_owned(),
            That => "THAT".to_owned(),
            Static => labels::generate_static_label(filename, index),
            Temp => {
                if index > &7 {
                    panic!("Temp index is outside segment bounds")
                }
                (5 + index).to_string().to_owned()
            }
            Pointer => {
                if index > &1 {
                    panic!("pointer index is outside segment bounds")
                }
                (3 + index).to_string().to_owned()
            }

            _ => return Err(NoAvailableMemlabelError),
        };

        Ok(memlabel)
    }
}

#[derive(Debug)]
pub struct InvalidKeywordError;

pub enum VMOperation {
    Initialisation(InitOp),
    Arithmetic(ArithmeticOp),
    Compare(CompareOp),
    StackOperation(StackOp),
    BranchOperation(BranchOp),
    FunctionOperation(FunctionOp),
}

impl VMOperation {
    pub fn parse(operation_string: &str) -> Result<VMOperation, InvalidKeywordError> {
        use VMOperation::*;

        let operation = match operation_string {
            "INIT" => Initialisation(InitOp),

            "add" => Arithmetic(ArithmeticOp::Add),
            "sub" => Arithmetic(ArithmeticOp::Subtract),
            "neg" => Arithmetic(ArithmeticOp::Negate),
            "and" => Arithmetic(ArithmeticOp::And),
            "or" => Arithmetic(ArithmeticOp::Or),
            "not" => Arithmetic(ArithmeticOp::Not),

            "eq" => Compare(CompareOp::Equate),
            "gt" => Compare(CompareOp::GreaterThan),
            "lt" => Compare(CompareOp::LessThan),

            "push" => StackOperation(StackOp::Push),
            "pop" => StackOperation(StackOp::Pop),

            "label" => BranchOperation(BranchOp::Label),
            "goto" => BranchOperation(BranchOp::Goto),
            "if-goto" => BranchOperation(BranchOp::IfGoto),

            "function" => FunctionOperation(FunctionOp::Function),
            "call" => FunctionOperation(FunctionOp::Call),
            "return" => FunctionOperation(FunctionOp::Return),

            _ => return Err(InvalidKeywordError),
        };

        Ok(operation)
    }
}

pub struct InitOp;

impl InitOp {
    pub fn emit_asm(&self, target: &mut String, address_pointer: AddressPointer, value: u16) {
        writeln!(
            target,
            "@{}\n\
            D=A\n\
            @{}\n\
            M=D",
            value,
            address_pointer.get_label()
        )
        .expect("Failed to write to target in InitOp::emit_asm")
    }
}

pub enum AddressPointer {
    StackPointer,
    Local,
    Argument,
    This,
    That,
}

impl AddressPointer {
    pub fn parse(address_string: &str) -> Result<AddressPointer, InvalidKeywordError> {
        let address = match address_string {
            "SP" => AddressPointer::StackPointer,
            "LCL" => AddressPointer::Local,
            "ARG" => AddressPointer::Argument,
            "THIS" => AddressPointer::This,
            "THAT" => AddressPointer::That,
            _ => return Err(InvalidKeywordError),
        };

        Ok(address)
    }

    pub fn get_label(&self) -> String {
        match self {
            AddressPointer::StackPointer => "SP".to_owned(),
            AddressPointer::Local => "LCL".to_owned(),
            AddressPointer::Argument => "ARG".to_owned(),
            AddressPointer::This => "THIS".to_owned(),
            AddressPointer::That => "THAT".to_owned(),
        }
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
        let _ = match self {
            // 2-parameter operations
            ArithmeticOp::Add | ArithmeticOp::Subtract | ArithmeticOp::And | ArithmeticOp::Or => {
                writeln!(
                    target,
                    "@SP\n\
                AM=M-1\n\
                D=M\n\
                A=A-1\n\
                {}",
                    self.insert_operation()
                )
            }

            // 1-parameter operation
            ArithmeticOp::Negate | ArithmeticOp::Not => writeln!(
                target,
                "@SP\n\
                A=M-1\n\
                {}",
                self.insert_operation()
            ),
        };
    }

    fn insert_operation(&self) -> &str {
        match self {
            ArithmeticOp::Add => "M=D+M",
            ArithmeticOp::Subtract => "M=M-D",
            ArithmeticOp::And => "M=D&M",
            ArithmeticOp::Or => "M=D|M",

            ArithmeticOp::Negate => "M=-M",
            ArithmeticOp::Not => "M=!M",
        }
    }
}

pub enum CompareOp {
    Equate,
    GreaterThan,
    LessThan,
}

impl CompareOp {
    pub fn emit_asm(&self, target: &mut String, label: String) {
        writeln!(
            target,
            "@SP\n\
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
    Pop,
}

impl StackOp {
    pub fn emit_asm(
        &self,
        target: &mut String,
        segment: Segment,
        index: u16,
        static_label: &str,
    ) -> () {
        use Segment::*;

        let _ = match (self, segment) {
            (StackOp::Push, Constant) => {
                writeln!(
                    target,
                    "@{}\n\
                D=A\n\
                @SP\n\
                M=M+1\n\
                A=M-1\n\
                M=D",
                    index
                )
            }

            (StackOp::Push, seg @ (Local | Argument | This | That)) => {
                writeln!(
                    target,
                    "@{}\n\
                D=A\n\
                @{}\n\
                A=D+M\n\
                D=M\n\
                @SP\n\
                M=M+1\n\
                A=M-1\n\
                M=D",
                    index,
                    seg.get_memlabel(static_label, &index).unwrap()
                )
            }

            (StackOp::Push, seg @ (Temp | Static | Pointer)) => {
                writeln!(
                    target,
                    "@{}\n\
                D=M\n\
                @SP\n\
                M=M+1\n\
                A=M-1\n\
                M=D",
                    seg.get_memlabel(static_label, &index).unwrap()
                )
            }

            (StackOp::Pop, Constant) => panic!("Cannot pop on virtual segment \"Constant\" "),

            (StackOp::Pop, seg @ (Local | Argument | This | That)) => {
                writeln!(
                    target,
                    "@{}\n\
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
                    seg.get_memlabel(static_label, &index).unwrap()
                )
            }

            (StackOp::Pop, seg @ (Temp | Static | Pointer)) => {
                writeln!(
                    target,
                    "@{}\n\
                D=A\n\
                @R13\n\
                M=D\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @R13\n\
                A=M\n\
                M=D",
                    seg.get_memlabel(static_label, &index).unwrap()
                )
            }
        };
    }
}

pub enum BranchOp {
    Label,
    Goto,
    IfGoto,
}

impl BranchOp {
    pub fn emit_asm(&self, target: &mut String, branch_label: &String) -> () {
        let _ = match self {
            BranchOp::Label => {
                writeln!(target, "({})", branch_label)
            }

            BranchOp::Goto => {
                writeln!(
                    target,
                    "@{}\n\
                0;JMP",
                    branch_label
                )
            }

            BranchOp::IfGoto => {
                writeln!(
                    target,
                    "@SP\n\
                AM=M-1\n\
                D=M\n\
                @{}\n\
                D;JNE",
                    branch_label
                )
            }
        };
    }
}

#[derive(Debug)]
pub enum FunctionOp {
    Call,
    Function,
    Return,
}

impl FunctionOp {
    pub fn emit_asm(
        &self,
        target: &mut String,
        function_label: &String,
        n: u16,
        id: &String,
    ) -> () {
        let _ = match self {
            FunctionOp::Function => {
                writeln!(
                    target,
                    "({})\n\
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

                writeln!(
                    target,
                    "@ARG\n\
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
                    n, return_label, function_label, return_label
                )
            }

            FunctionOp::Return => writeln!(
                target,
                "@LCL\n\
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
                0;JMP"
            ),
        };
    }
}

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
