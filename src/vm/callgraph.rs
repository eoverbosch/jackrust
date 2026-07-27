use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Clone)]
    pub struct FunctionRegistration {
        identifier: String,
        line_bounds: (usize, usize),
    }

pub enum FunctionNode {
    Called(String),
    Defined(FunctionRegistration)
}

#[derive(Debug)]
pub struct PlaceholderError;

impl std::fmt::Display for PlaceholderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "placeholder error").unwrap();
        Ok(())
    }
}

impl std::error::Error for PlaceholderError {}

    pub struct CallGraph {
        graph: DiGraph<FunctionNode, ()>,
        nodes: HashMap<String, NodeIndex>
    }

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

    pub fn build_function_reader<'a>(
        &self,
        entrypoint_label: &str,
        files: &'a [CachedFile],
    ) -> Result<Vec<std::iter::Take<std::iter::Skip<std::str::Lines<'a>>>>, Box<dyn std::error::Error>> {

        let entrypoint_index = self.nodes.get(entrypoint_label)
            .ok_or(PlaceholderError)?;

        let mut readers = Vec::new();
        let mut dfs = petgraph::visit::Dfs::new(&self.graph, *entrypoint_index);

        while let Some(node_index) = dfs.next(&self.graph) {
            // Every reachable node must be Defined at this point
            let FunctionNode::Defined(function_reg) = &self.graph[node_index]
                else { return Err(Box::new(PlaceholderError)) };

            // Resolve file from identifier prefix
            let (file_label, _) = function_reg.identifier.split_once(".")
                .ok_or(PlaceholderError)?;

            let file = files.iter()
                .find(|file| file.filepath.file_stem() == Some(OsStr::new(file_label)))
                .ok_or(PlaceholderError)?;

            // File must already be cached; error otherwise
            let content = file.content.as_ref()
                .ok_or(PlaceholderError)?;

            // line_bounds inclusive on both ends
            let (start, end) = function_reg.line_bounds;
            let reader = content.lines().skip(start).take(end - start + 1);

            readers.push(reader);
        }

        Ok(readers)
    }

    pub fn from_entrypoint(entrypoint_label: String, files: &mut Vec<CachedFile>)
        -> Result<CallGraph, Box<dyn std::error::Error>> {

        let mut callgraph = CallGraph::new();
        let mut worklist: Vec<usize> = Vec::new(); // usize is an index into files

        let (entrypoint_filename, _) = entrypoint_label.split_once(".")
            .ok_or(PlaceholderError)?;

        let entrypoint_file_index = files.iter()
            .position(|file| file.filepath.file_stem() == Some(OsStr::new(entrypoint_filename)))
            .ok_or(PlaceholderError)?;

        worklist.push(entrypoint_file_index);

        while let Some(vm_file_index) = worklist.pop() {
            let mut active_function_index: Option<NodeIndex> = None;

            let vm_file = &mut files[vm_file_index];

            // If file has been opened already, continue
            if let Some(_) = vm_file.content {continue}

            // If not, scan file 
            vm_file.read()?;
            
            let content = files[vm_file_index].content.as_ref().unwrap();
            let vm_file_length = content.lines().count();

            for (line_number, line) in content.lines().enumerate() {
                let mut line_arguments = line.split_whitespace();

                match line_arguments.next() {
                    Some("function") => {
                        // Store final version of current active function in callgraph
                        if let Some(fn_index) = active_function_index {

                            // Access active function in callgraph
                            let FunctionNode::Defined(active_function) 
                                = callgraph.graph.node_weight_mut(fn_index).unwrap()
                                    else {unreachable!()};

                            // Replace line end
                            active_function.line_bounds.1 = line_number - 1;
                        }

                        // Make new function registration with default line_end = eof
                        let function_label = line_arguments.next().unwrap();

                        let function_reg = FunctionRegistration {
                            identifier: function_label.to_owned(),
                            line_bounds: (line_number, vm_file_length - 1)
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

                        // Find file relating to function in files and add to worklist
                        let Some((file_label, _)) = function_label.split_once(".") 
                            else {return Err(Box::new(PlaceholderError))};

                        let file_index = files.iter()
                            .position(|file| file.filepath.file_stem() == Some(OsStr::new(file_label)))
                            .ok_or(PlaceholderError)?;
                        
                        worklist.push(file_index)
                    },

                    _ => continue
                }
            }
        }
        Ok(callgraph)
    }
}

pub struct CachedFile {
    pub filepath: PathBuf,
    pub content: Option<String>
}

impl CachedFile {
    pub fn read_from_path(path_in: impl Into<PathBuf>) -> Result<CachedFile, std::io::Error> {
        let mut cached_file = CachedFile::from_path(path_in);
        cached_file.read()?;
        Ok(cached_file)
    }

    pub fn from_path(path_in: impl Into<PathBuf>) -> CachedFile {
        CachedFile { filepath: path_in.into(), content: None }
    }

    pub fn read(&mut self) -> Result<(), std::io::Error> {
        self.content = Some(std::fs::read_to_string(&self.filepath)?);
        Ok(())
    }

    pub fn uncache(&mut self) {
        self.content = None
    }
}