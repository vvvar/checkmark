use glob::glob;
use clap::Parser;
use std::fs;
use std::fmt;

// pub mod deno;
// pub use deno::js_runtime;
use js_sandbox::{Script, AnyError};

#[derive(Debug)]
struct ReadMdFilesError;

impl fmt::Display for ReadMdFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Root where to search for md files
    #[arg(short, long)]
    root: String
}

fn read_all_md_files(root: &String) -> Result<Vec<String>, String> {
    match fs::canonicalize(root) {
        Ok(path) => {
            match path.into_os_string().into_string() {
                Ok(path_str) => {
                    let pattern = format!("{}{}", &path_str, "/**/*.md");
                    let mut result = Vec::new();
                    for entry in glob(&pattern).expect("Failed to read glob pattern") {
                        match entry {
                            Ok(path) => result.push(path.display().to_string()),
                            Err(_e) => return Err("Cannot read glob".to_string()),
                        }
                    }
                    return Ok(result);
                },
                Err(_e) => return Err("Cannot convert path to string".to_string())
            }
        },
        Err(_e) => return Err("Cant to parse path".to_string())
    }
}

fn main() -> Result<(), AnyError> {
    let args = Args::parse();
    match read_all_md_files(&args.root) {
        Ok(files) => {
            for file in files {
                println!("{:?}", file);
            }
        },
        Err(e) => println!("{:?}", e)
    }

    let mut script = Script::from_file("/Users/vvoinov/Documents/repos/md-checker/src/js/bundle.js")?;
    let result: String = script.call("format_markdown", ("test markdown",))?;
    println!("{:?}", result);
    Ok(())

    // let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    // if let Err(error) = runtime.block_on(js_runtime::format_markdown("fdfsdfs")) {
    //     eprintln!("error: {}", error);
    // }
     
    // let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    // if let Err(error) = runtime.block_on(js_runtime::run()) {
    //     eprintln!("error: {}", error);
    // }
}
