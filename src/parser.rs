
use std::fs;
use std::env;
use std::io::{self, BufRead};

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

pub fn bufreader_from_path(input_file_path: &String) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(input_file_path)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn strip_whitespace_and_comments(line: String) -> String {
    line
    .split_whitespace() // Split whitespace and put back together
    .collect::<String>()

    .splitn(2, "//") // Split once at first comment delimiter and unwrap on first slice
    .next()
    .unwrap() // (guaranteed)

    .to_string() // Return as string
}
