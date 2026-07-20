use jackrust::parser;
use jackrust::vm_parser::{Config, CachedFile};
use code::*;
use code::labels;
use std::fs;  
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let config = Config::init().unwrap_or_else(|e| e.exit());

    // Discovery
    let vm_files: Vec<CachedFile> = config.path
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


    // Run pre-processor
    let code_lines = line_reader
        .map(Result::unwrap) 
        .map(parser::strip_comments)
        .map(|line| line.trim().to_string())
        .enumerate()
        .filter(|(_, line)| !line.is_empty());

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
                        line_number+1)
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


mod code {
    use std::fmt::Error;
use std::{ffi::OsStr, path::PathBuf};
    use jackrust::vm_parser::{CachedFile};
    use petgraph::graph::{DiGraph, NodeIndex};
    use petgraph::visit;
    use std::collections::HashMap;

use crate::code::FunctionNode::Called;

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

    pub enum FunctionNode {
        Called(String),
        Defined(FunctionRegistration)
    }

    pub struct CallGraph {
        graph: DiGraph<FunctionNode, ()>,
        nodes: HashMap<String, NodeIndex>
    }

    #[derive(Debug)]
    pub struct PlaceholderError;

    impl std::fmt::Display for PlaceholderError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "placeholder error")
        }
    }

    impl std::error::Error for PlaceholderError {}

    impl CallGraph {
        pub fn new() -> CallGraph {
            CallGraph {
                graph: DiGraph::new(), 
                nodes: HashMap::new()
            }
        }

        pub fn get(&self, key: &str) -> Option<&FunctionNode>{
            self.nodes.get(key).and_then(|&i| self.graph.node_weight(i))
        }

        pub fn add_node(&mut self, node_weight: FunctionNode) -> Option<NodeIndex> {
            let node_key = match &node_weight {
                FunctionNode::Called(identifier) => identifier.clone(),
                FunctionNode::Defined(function_reg) => function_reg.identifier.clone()
            };

            if !self.nodes.contains_key(&node_key) {

                let node_index = self.graph.add_node(node_weight);
                self.nodes.insert(node_key.to_owned(), node_index);

                Some(node_index)

            } else {
                None
            }
        }

        fn add_defined(&mut self, function_reg: FunctionRegistration) -> NodeIndex {
            let Some(idx) = self.nodes.get(&function_reg.identifier) else {
                let idx = self.graph.add_node(FunctionNode::Defined(function_reg.clone()));
                self.nodes.insert(function_reg.identifier.to_owned(), idx);
                return idx
            };

            idx.to_owned()
        }

        pub fn add_called(&mut self, identifier: &str) -> NodeIndex {
            let Some(idx) = self.nodes.get(identifier) else {
                let idx = self.graph.add_node(FunctionNode::Called(identifier.to_string()));
                self.nodes.insert(identifier.to_owned(), idx);
                return idx
            };

            idx.to_owned()
        }

        fn upgrade_node(&mut self, function_reg: FunctionRegistration) -> Result<NodeIndex,Box<dyn std::error::Error>> {
            match self.get(&function_reg.identifier).unwrap() {
                FunctionNode::Defined(_) => Err(Box::new(PlaceholderError)),
                FunctionNode::Called(identifier) => {
                    let idx = self.nodes.get(identifier).unwrap().to_owned();
                    self.graph[idx] = FunctionNode::Defined(function_reg);
                    Ok(idx)
                }
            }
        }

        /// Either adds a non-existing node based on identifier, or upgrades a function call (Called)
        /// to a function definition (Defined). Errors when caller tries to upgrade an already existing Defined node
        pub fn try_add_or_upgrade(&mut self, function_reg: FunctionRegistration) -> Result<NodeIndex,Box<dyn std::error::Error>> {
            match self.get(&function_reg.identifier) {
                Some(FunctionNode::Defined(_)) => return Err(Box::new(PlaceholderError)),
                Some(FunctionNode::Called(_)) => self.upgrade_node(function_reg),
                None => Ok(self.add_defined(function_reg))
            }
        }

        pub fn from_entrypoint(entrypoint_label: String, files: &mut Vec<CachedFile>)
            -> Result<(), Box<dyn std::error::Error>> {

            let mut active_function_index: Option<NodeIndex> = None;
            let mut callgraph = CallGraph::new();
            let mut worklist: Vec<&CachedFile> = Vec::new();

            let (entrypoint_filename, _) = entrypoint_label.split_once(".")
                .ok_or(PlaceholderError)?;

            let entrypoint_file = files.iter()
                .find(|file| file.filepath.file_stem() == Some(OsStr::new(entrypoint_filename)))
                .ok_or(PlaceholderError)?;

            worklist.push(entrypoint_file);

            while let Some(vm_file) = worklist.pop() {
                // If file has been opened already, continue
                if let Some(_) = vm_file.content {continue}

                // If not, scan file 
                vm_file.read()?;
                
                let vm_file_length = vm_file.content.iter().len();
                let mut vm_file_lines = vm_file.content.unwrap().lines().enumerate();

                while let Some((line_number, line)) = vm_file_lines.next() {
                    let mut line_arguments = line.split_whitespace();

                    match line_arguments.next() {
                        Some("function") => {
                            // Store final version of current active function in callgraph
                            if let Some(fn_index) = active_function_index {
                                // Access active function in callgraph
                                let FunctionNode::Defined(mut active_function) 
                                    = callgraph.graph.node_weight_mut(fn_index).unwrap() 
                                        else {unreachable!()}; // active_function_index is by default Defined

                                // Replace line end
                                active_function.line_bounds.1 = line_number - 1;
                                
                            }

                            // Make new function registration with default line_end = eof
                            let function_label = line_arguments.next().unwrap();

                            let function_reg = FunctionRegistration {
                                identifier: function_label.to_owned(),
                                line_bounds: (line_number, vm_file_length)
                            };
                            
                            // If identifier doesn't exist in callgraph or is Called, insert
                            // new function registration as Defined. If exists but is defined, error
                            let new_index = callgraph.try_add_or_upgrade(function_reg)?;

                            // Set active function to NodeIndex
                            active_function_index = Some(new_index);

                        },

                        Some("call") => {
                            // If identifier doesn't exist in callgraph: add as Called
                            let function_label = line_arguments.next().unwrap();

                            let node_index = callgraph.add_called(function_label);

                            // If active function is not None:
                            // Add edge to graph from active function to called function
                            if let Some(idx) = active_function_index {
                                callgraph.graph.add_edge(idx, node_index, ());
                            }
                        },

                        _ => continue
                    }
                }
            }

            Ok(())
        }

    }

    #[derive(Clone)]
    pub struct FunctionRegistration {
        identifier: String,
        line_bounds: (usize, usize),
    }

    impl FunctionRegistration {
        pub fn from_function_label(fn_label: &str, files: Vec<PathBuf>) 
            -> Result<FunctionRegistration, Box<dyn std::error::Error>> {

            let (file_str, _) = fn_label
                .split_once(".")
                .ok_or_else(|| format!("Invalid fn_label formatting: {fn_label}"))?;

            let fn_file = files.iter()
                .find(|file| file.file_stem() == Some(OsStr::new(file_str)))
                .ok_or_else(|| format!("File {file_str}.vm not found in current file context"))?;
            
            let file_content = std::fs::read_to_string(fn_file)?;
            let mut enumerated_file_content = file_content
                .lines().enumerate();
            
            let (line_start, _) = enumerated_file_content
                .find( |(_, line)| {
                    let mut args = line.split_whitespace();
                    args.next() == Some("function") && args.next() == Some(fn_label)
                    }
                )
                .ok_or_else(|| format!("function token {fn_label} not found in {file_str}.vm"))?;
            
            let mut last = line_start;

            let line_end = enumerated_file_content
                .find_map( |(line_number, line)| {
                    last = line_number;
                    (line.split_whitespace().next() == Some("function")).then_some(line_number)
                })
                .unwrap_or(last + 1);

            Ok(FunctionRegistration {
                identifier: fn_label.to_owned(),
                line_bounds: (line_start, line_end)
            })  
        }
    }

    /// FunctionDef represent a vm function definition during translation (i.e. function function_label n_vars). 
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
