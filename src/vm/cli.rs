use std::path::{PathBuf, Path};

use clap::builder::OsStr;
use clap::CommandFactory;
use clap::Parser;

const ASM_FILE_EXT: &str = "asm";
pub const ENTRYPOINT: &str = "Sys.init";


#[derive(Parser)]
/// VM Translator for the NAND2Tetris Hack platform. Translates hack virtual machine code to 
/// hack assembly. Can be run in directory mode or single-file mode. 
pub struct Config {    
    /// Optional output file specifier. Defaults to folder name
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

        config.path = std::env::current_dir()
            .map_err(|e| cmd.error(
                clap::error::ErrorKind::Io,
                format!("failed to fetch current path: {e}"),
            ))?;

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

    pub fn get_input_name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("a")
    }
}