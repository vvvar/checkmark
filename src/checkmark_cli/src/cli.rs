use clap::Parser;

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct FmtCommand {
    /// Check fmt issues without fixing them
    #[arg(long, short, action)]
    pub check: bool,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct GrammarCommand {}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct ReviewCommand {
    /// Do not include suggestions in the report
    #[arg(long, short, action)]
    pub no_suggestions: bool,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct LinksCommand {
    /// List of wildcard URI patterns to ignore(both files and web links)
    #[arg(long, short)]
    pub ignore_wildcards: Vec<String>,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct SpellingCommand {
    /// List of words that should be ignored by spell checker
    #[arg(long, short)]
    pub words_whitelist: Vec<String>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Formatting tool.
    Fmt(FmtCommand),
    /// Grammar checker tool(requires internet and OPEN_AI_API_KEY env var set).
    Grammar(GrammarCommand),
    /// Make a review of the document(requires internet and OPEN_AI_API_KEY env var set).
    Review(ReviewCommand),
    /// Check links in the document
    Links(LinksCommand),
    /// Spell check document
    Spelling(SpellingCommand),
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Root of your project
    #[arg(global = true, value_hint=clap::ValueHint::DirPath, default_value=".")]
    pub project_root: String,

    /// Output report to a file in SARIF format
    #[arg(global = true, long, short, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath, default_missing_value="./report.sarif", num_args=0..=1)]
    pub sarif: Option<String>,

    /// Path to config file(files in default locations will be ignored when this is set)
    #[arg(global = true, long, short, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath)]
    pub config: Option<String>,

    /// Enable verbose logging
    #[arg(global = true, long, short, required = false, action)]
    pub verbose: bool,

    /// Individual tools
    #[command(subcommand)]
    pub subcommands: Subcommands,
}

pub fn init() -> Cli {
    return Cli::parse();
}
