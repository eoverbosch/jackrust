use std::fs;
use std::env;
use std::io::{self, BufRead};
use std::path::PathBuf;

use clap::Parser;

#[derive(Default)]
enum Mode {
    #[default]
    None,
    Assembler,
    VmFile,
    VmDir,    
    Compiler,
}

#[derive(Parser)]
struct Config {
    #[arg(skip)]
    mode: Mode,

    /// Optional input. Required for 
    input: Option<PathBuf>

}

pub fn get_file_content_from_env() -> String {
    let input_file_path = get_path_from_env();

    let file_content: String = fs::read_to_string(input_file_path)
        .expect("Unable to open .asm file with the following error:");

    file_content
}

pub fn get_path_from_env() -> String {
    let env_args: Vec<String> = env::args().collect();
    env_args[1].clone()
}

pub fn get_filename_from_env() -> String {
    get_path_from_env()
    .split(std::path::MAIN_SEPARATOR)
    .last()
    .unwrap()
    .to_string()
}

pub fn get_output_path_from_env(
        file_extension_old: &str,
        file_extension_new: &str,
    ) -> String {

    let mut file_name = get_path_from_env()
    .strip_suffix(file_extension_old)
    .unwrap()
    .to_string();

    file_name.push_str(file_extension_new);
    file_name
}

pub fn bufreader_from_path(input_file_path: &String) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(input_file_path)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn strip_whitespace_and_comments(line: String) -> String {
    strip_whitespace(strip_comments(line))
}

pub fn strip_whitespace(line: String) -> String {
    line
    .split_whitespace()
    .collect::<String>()
}

pub fn strip_comments(line: String) -> String {
    line
    .splitn(2, "//")
    .next()
    .unwrap()
    .to_string()
}
