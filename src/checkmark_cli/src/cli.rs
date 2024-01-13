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
    /// Check mode: Reviews formatting issues without fixing them. Returns 0 if no issues, 1 otherwise
    #[arg(long, action)]
    pub check: bool,

    /// Display a detailed comparison if formatting issues are detected
    #[arg(long, action, requires = "check")]
    pub show_diff: bool,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct ReviewCommand {
    /// Do not include suggestions in the report
    #[arg(long, short, action)]
    pub no_suggestions: bool,
    /// Provide custom prompt for OpenAI's API(will replace the default prompt)
    #[arg(long, action, required = false)]
    pub prompt: Option<String>,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct ComposeCommand {
    /// Describe what you want to write about
    #[arg(long, action, required = true)]
    pub prompt: String,
    /// Where to save the file. Defaults to the current directory
    #[arg(long, short, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath)]
    pub output: Option<String>,
    /// Use this file as a context to generate the text
    #[arg(long, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath)]
    pub context: Option<String>,
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
pub struct LintCommand {
    /// MD033: List of HTML tags to allow in the document
    #[arg(long, short)]
    pub allowed_html_tags: Vec<String>,
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
    /// Checks the document for broken links(both web and local)
    Links(LinksCommand),
    /// Run linter
    Lint(LintCommand),
    /// Reviews the document using OpenAI's API. Requires internet connection and OPEN_AI_API_KEY environment variable(.dotenv file is supported)
    Review(ReviewCommand),
    /// Compose a file in Markdown format from a prompt
    Compose(ComposeCommand),
    /// Checks the document for spelling errors(offline)
    Spelling(SpellingCommand),
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None, styles = styles())]
#[command(propagate_version = true)]
pub struct Cli {
    /// Sets the project root, file, or web URL for scanning Markdown files.
    /// Can also accept a Git repository.
    /// Defaults to the current directory if not specified
    #[arg(global = true, value_hint=clap::ValueHint::AnyPath, default_value=".")]
    pub project_root: String,

    /// List of files(wildcards) to exclude from scanning
    #[arg(global = true, long, required = false, num_args = 1.., value_delimiter = ' ', value_hint=clap::ValueHint::AnyPath)]
    pub exclude: Vec<String>,

    /// Style: Type of heading style to enforce
    #[arg(long, action, required = false)]
    pub style_heading: Option<String>,

    /// Sets the configuration file path. Overrides default files if set
    #[arg(global = true, long, short, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath)]
    pub config: Option<String>,

    /// CI Mode: Turns off interactive prompts and outputs report in a format suitable for CI/CD pipelines
    #[arg(global = true, long, required = false, action)]
    pub ci: bool,

    /// Saves the report in SARIF format to a given file or defaults to './report.sarif' if no file is specified
    #[arg(global = true, long, action, required = false, value_name = "FILE_PATH", value_hint=clap::ValueHint::FilePath, default_missing_value="./report.sarif", num_args=0..=1)]
    pub sarif: Option<String>,

    /// Verbose logging: Provides detailed tool activity, useful for debugging
    #[arg(global = true, long, required = false, action)]
    pub verbose: bool,

    /// Specifies the individual tools or commands to run.
    /// This is required and allows you to run specific checks or operations
    #[command(subcommand)]
    pub subcommands: Subcommands,
}

pub fn init() -> Cli {
    Cli::parse()
}
