use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use clap::CommandFactory;
use clap::Parser;
use clap::builder::OsStr;

use crate::ASM_FILE_EXT;
use crate::HACK_FILE_EXT;

#[derive(Parser)]
/// Hack assembler for the NAND2Tetris Hack platform. Assembles hack assembly code into binary,
/// using a simple command line tool.
pub struct Config {
    /// The path to a hack assembly input file.
    #[arg(value_parser = Config::parse_input_file)]
    pub input: PathBuf,

    /// Optional output file specifier. Defaults to input file name
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Optional flag that dumps intermediate results
    #[arg(short, long)]
    pub dump: bool,

    #[arg(skip)]
    pub path: PathBuf,
}

impl Config {
    pub fn init() -> Result<Self, clap::Error> {
        let mut config = Config::try_parse()?;
        let mut cmd = Config::command();

        config.path = std::env::current_dir().map_err(|e| {
            cmd.error(
                clap::error::ErrorKind::Io,
                format!("failed to fetch current path: {e}"),
            )
        })?;

        match &mut config.output {
            Some(output_file) => match output_file.extension() {
                None => {
                    output_file.set_extension(HACK_FILE_EXT);
                }
                Some(ext) if ext == HACK_FILE_EXT => (),
                Some(_) => {
                    return Err(cmd.error(
                        clap::error::ErrorKind::Io,
                        format!(
                            "specified output file \"{}\" is not of type .{}",
                            output_file.display(),
                            HACK_FILE_EXT
                        ),
                    ));
                }
            },

            None => {
                config.output = Some(
                    Path::new(config.input.file_name().unwrap_or(&OsStr::from("a")))
                        .with_extension(HACK_FILE_EXT),
                );
            }
        }

        Ok(config)
    }

    fn parse_input_file(s: &str) -> Result<PathBuf, String> {
        let mut input_file = PathBuf::from(s);

        match input_file.extension() {
            None => {
                input_file.set_extension(ASM_FILE_EXT);
            }
            Some(ext) if ext == ASM_FILE_EXT => (),
            Some(_) => {
                return Err(format!(
                    "Specified input file \"{}\" is not of type .{}",
                    s, ASM_FILE_EXT
                ));
            }
        };

        if !input_file.exists() {
            return Err(format!(
                "file {} does not exist in the current directory",
                s
            ));
        }

        Ok(input_file)
    }

    pub fn file_reader_from_input(&self) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
        let file = fs::File::open(&self.input)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn get_input_name(&self) -> &str {
        self.input.file_stem().unwrap().to_str().unwrap()
    }
}
