use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Root of your project. Recursively search for Markdown files in this folder
    pub root: String,

    /// Perform auto-formatting of files. 
    /// Note: All checker flags will be ignored in this mode.
    /// Note: This will modify your files!
    #[arg(short, long)]
    pub autoformat: bool
}

pub fn read() -> Args {
    return Args::parse();
}