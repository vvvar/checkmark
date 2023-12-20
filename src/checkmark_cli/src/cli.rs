use clap::Parser;

#[derive(clap::Parser)]
#[command(long_about = None)]
pub struct FmtCommand {
    /// Check fmt issues without fixing them
    #[arg(long, short, action)]
    pub check: bool,
}

#[derive(clap::Parser)]
#[command(long_about = None)]
pub struct GrammarCommand {}

#[derive(clap::Parser)]
#[command(long_about = None)]
pub struct ReviewCommand {}

#[derive(clap::Subcommand)]
pub enum Subcommands {
    /// Formatting tool.
    Fmt(FmtCommand),
    /// Grammar checker tool(requires internet and OPEN_AI_API_KEY env var set).
    Grammar(GrammarCommand),
    /// Make a review of the document(requires internet and OPEN_AI_API_KEY env var set).
    Review(ReviewCommand),
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Root of your project.
    #[arg(global = true, value_hint=clap::ValueHint::DirPath, default_value=".")]
    pub project_root: String,

    /// Output report to a file in SARIF format
    #[arg(global = true, long, short, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath, default_missing_value="./report.sarif", num_args=0..=1)]
    pub sarif: Option<String>,

    /// Individual tools
    #[command(subcommand)]
    pub subcommands: Subcommands,
}

pub fn init() -> Cli {
    return Cli::parse();
}
