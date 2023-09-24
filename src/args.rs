use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Root where to search for md files
    #[arg(short, long)]
    pub root: String
}

pub fn read() -> Args {
    return Args::parse();
}