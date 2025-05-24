mod parser;

use clap::Parser;
use log::info;

use parser::parse_lua_bytecode;

/// Command-line arguments parser
#[derive(Parser, Debug)]
#[clap(
    author = "bytexenon",
    version = "1.0.0",
    about = "Decompile .luac files and convert them back to Lua source code"
)]
struct Arguments {
    /// Paths to the Lua bytecode files to decompile
    #[clap(
        required = true,
        help = "One or more Lua bytecode file to decompile.",
        value_name = "FILE",
        value_hint = clap::ValueHint::FilePath
    )]
    files: Vec<String>,
}

/// Reads a Lua bytecode file and returns its contents as a byte vector
fn read_file(file_path: &str) -> std::io::Result<Vec<u8>> {
    let data = std::fs::read(file_path)?;
    Ok(data)
}

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let args = Arguments::parse();
    let file_paths = args.files;

    for file_path in file_paths {
        info!("Parsing file: {}", file_path);

        // Read the Lua bytecode file
        let bytecode = read_file(file_path.as_str()).unwrap_or_else(|err| {
            eprintln!("Error reading file: {}", err);
            std::process::exit(1);
        });

        // Parse the Lua bytecode
        match parse_lua_bytecode(&bytecode) {
            Ok((header, prototype)) => {
                info!("Parsed Lua bytecode successfully.");

                println!("Header: {:#?}", header);
                println!("Function Prototype: {:#?}", prototype);
                for instr in prototype.code {
                    println!("{}", instr);
                }
            }
            Err(err) => {
                eprintln!("Error parsing Lua bytecode: {:?}", err);
            }
        }
    }
}
