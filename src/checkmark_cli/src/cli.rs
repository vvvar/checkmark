use clap::Parser;

#[derive(clap::Parser)]
#[command(about = "Formatting tool.", long_about = None)]
pub struct FmtCommand {
    /// Root of your project.
    #[arg(value_hint=clap::ValueHint::DirPath, default_value=".")]
    pub project_root: String,

    /// Check fmt issues without fixing them
    #[arg(long, short, action)]
    pub check: bool,
}

#[derive(clap::Subcommand)]
pub enum Subcommands {
    Fmt(FmtCommand),
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommands: Subcommands,
}

pub fn read() -> Cli {
    return Cli::parse();
}
