use clap::Parser;

fn styles() -> clap::builder::styling::Styles {
    clap::builder::styling::Styles::styled()
        .header(
            clap::builder::styling::AnsiColor::Green.on_default()
                | clap::builder::styling::Effects::BOLD,
        )
        .usage(
            clap::builder::styling::AnsiColor::Green.on_default()
                | clap::builder::styling::Effects::BOLD,
        )
        .literal(
            clap::builder::styling::AnsiColor::Cyan.on_default()
                | clap::builder::styling::Effects::BOLD,
        )
        .placeholder(clap::builder::styling::AnsiColor::Cyan.on_default())
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct FmtCommand {
    /// Check fmt issues without fixing them
    #[arg(long, action)]
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
    /// Formats all Markdown files in the project. This will fix common formatting issues such as trailing whitespace, inconsistent line endings, and more
    Fmt(FmtCommand),
    /// Checks the document for grammatical errors. Requires internet connection and OPEN_AI_API_KEY environment variable(.dotenv file is supported)
    Grammar(GrammarCommand),
    /// Reviews the document using OpenAI's API. Requires internet connection and OPEN_AI_API_KEY environment variable(.dotenv file is supported)
    Review(ReviewCommand),
    /// Checks the document for broken links(both web and local)
    Links(LinksCommand),
    /// Checks the document for spelling errors(offline)
    Spelling(SpellingCommand),
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None, styles = styles())]
#[command(propagate_version = true)]
pub struct Cli {
    /// Specifies the root directory of your project, a single file, or a web URL. This is where the tool will start scanning for Markdown files. If a single file or a web URL is specified, only that will be scanned. Defaults to the current directory
    #[arg(global = true, value_hint=clap::ValueHint::DirPath, default_value=".")]
    pub project_root: String,

    /// Outputs the report to a specified file in SARIF format. If no file is specified, the report will be saved to './report.sarif'.
    #[arg(global = true, long, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath, default_missing_value="./report.sarif", num_args=0..=1)]
    pub sarif: Option<String>,

    /// Specifies the path to the configuration file. If this is set, files in default locations will be ignored
    #[arg(global = true, long, short, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath)]
    pub config: Option<String>,

    /// Enable verbose logging. This will output more detailed information about what the tool is doing, which can be helpful for debugging
    #[arg(global = true, long, required = false, action)]
    pub verbose: bool,

    /// Enable CI Mode - disables all interactive prompts and outputs the report in a machine-readable format. This is useful for running the tool in a CI/CD pipeline
    #[arg(global = true, long, required = false, action)]
    pub ci: bool,

    /// Specifies the individual tools or commands to run. This is required and allows you to run specific checks or operations
    #[command(subcommand)]
    pub subcommands: Subcommands,
}

pub fn init() -> Cli {
    Cli::parse()
}
