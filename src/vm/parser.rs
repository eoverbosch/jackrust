pub fn strip_comments(line: &str) -> String {
    line
    .splitn(2, "//")
    .next()
    .unwrap()
    .to_string()
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
