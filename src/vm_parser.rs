use std::fs;
use std::env;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use clap::Parser;
use clap::builder::OsStr;
use clap::CommandFactory;

const VM_FILE_EXT: &str = "vm";
const ASM_FILE_EXT: &str = "asm";

#[derive(Default)]
enum Mode {
    #[default]
    None, 
    SingleFile,
    Directory
}

#[derive(Parser)]
/// VM Translator for the NAND2Tetris Hack platform. Translates hack virtual machine code to 
/// hack assembly. Can be run in directory mode or single-file mode. 
pub struct Config {
    /// Optional input file. If specified, runs translator in single-file mode.
    #[arg(value_parser = Config::parse_input_file)]
    input: Option<PathBuf>,
    
    /// Optional output file specifier. Defaults to folder name
    output: Option<PathBuf>,

    #[arg(skip)]
    path: PathBuf,

    #[arg(skip)]
    mode: Mode
}

impl Config {
    pub fn init() -> Result<Self, clap::Error> {
        let mut config = Config::try_parse()?;
        let mut cmd = Config::command();

        config.path = env::current_dir()
            .map_err(|e| cmd.error(
                clap::error::ErrorKind::Io,
                format!("failed to fetch current path: {e}"),
            ))?;

        config.mode = match config.input {
            Some(_) => Mode::SingleFile,
            None => Mode::Directory
        };

        match &mut config.output {
            Some(output_file) => {
                
                match output_file.extension() {
                    None => {output_file.set_extension(ASM_FILE_EXT);},
                    Some(ext) if ext == ASM_FILE_EXT => (),
                    Some(_) => return Err(cmd.error(
                        clap::error::ErrorKind::Io, 
                        format!(
                            "specified output file \"{}\" is not of type .{}", output_file.display(),
                             ASM_FILE_EXT
                            )
                    ))
                }
                todo!("match file extension: None: add .asm; .asm: do nothing; _: error")
            }

            None => {
                config.output = Some(Path::new(
                    config.path.file_name() // file_name actually returns a directory here
                    .unwrap_or(&OsStr::from("a")))
                    .with_extension(ASM_FILE_EXT));
            }
        }

        Ok(config)
    }

    fn parse_input_file(s: &str) -> Result<PathBuf, String> {
        let mut input_file = PathBuf::from(s);

        match input_file.extension() {
            None => {input_file.set_extension(VM_FILE_EXT);},
            Some(ext) if ext == VM_FILE_EXT => (),
            Some(_) => return Err(format!("Specified input file \"{}\" is not of type .{}", s, VM_FILE_EXT))
        };

        if !input_file.exists() {
            return Err(format!("file {} does not exist in the current directory", s))
        }

        Ok(input_file)
    }

    pub fn make_file_reader(self) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
        let file = fs::File::open(self.input.unwrap())?;
        Ok(io::BufReader::new(file).lines())
    }
}