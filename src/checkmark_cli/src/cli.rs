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
    /// Controls the creativity of generated text. Higher value means more temperature and randomness. Must be between 0 and 100
    #[arg(long)]
    pub creativity: Option<u8>,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct RenderCommand {
    /// Path where rendered content should be saved
    #[arg(long, value_hint=clap::ValueHint::AnyPath, default_value="./output")]
    pub output: Option<String>,
    /// CSS Theme to use
    #[arg(long)]
    pub theme: Option<String>,
    /// Start the server and open the rendered document in the browser
    #[arg(long, action)]
    pub serve: bool,
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
    /// Controls the creativity of generated text. Higher value means more temperature and randomness. Must be between 0 and 100
    #[arg(long)]
    pub creativity: Option<u8>,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct LinkcheckCommand {
    /// List of wildcard URI patterns to ignore(both files and web links)
    #[arg(long, short)]
    pub ignore_wildcards: Vec<String>,
    /// Request timeout in seconds
    #[arg(long)]
    pub timeout: Option<u8>,
    /// How many times to retry a request before giving up
    #[arg(long)]
    pub max_retries: Option<u8>,
    /// List of accepted HTTP status codes for valid links
    #[arg(long)]
    pub accept: Vec<u16>,
    /// Optional GitHub token used for GitHub links. This allows much more request before getting rate-limited
    #[arg(long)]
    pub github_token: Option<String>,
    /// Optional user agent to use for HTTP requests
    #[arg(long)]
    pub user_agent: Option<String>,
    /// Allow insecure SSL certificates. Use only as a last resort because it is insecure
    #[arg(long, short, action)]
    pub allow_insecure: bool,
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
pub struct SpellcheckCommand {
    /// List of words that should be ignored by spell checker
    #[arg(long, short)]
    pub words_whitelist: Vec<String>,
}

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
pub struct GenerateConfigCommand {
    /// Path where config file should be saved
    #[arg(value_hint=clap::ValueHint::AnyPath, default_value=".")]
    pub path: String,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Formats Markdown files. This will fix common formatting issues such as trailing whitespace, inconsistent line endings, and more
    Fmt(FmtCommand),
    /// Checks the Markdown document for broken links(both web and local)
    Linkcheck(LinkcheckCommand),
    /// Checks document for common Markdown linting issues
    Lint(LintCommand),
    /// Reviews the document using OpenAI's API. Requires internet connection and OPEN_AI_API_KEY environment variable(.dotenv file is supported)
    Review(ReviewCommand),
    /// Renders the document into desired format
    Render(RenderCommand),
    /// Compose a file in Markdown format from a prompt
    Compose(ComposeCommand),
    /// Checks the document for spelling errors(offline)
    Spellcheck(SpellcheckCommand),
    /// Generates default configuration file
    GenerateConfig(GenerateConfigCommand),
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
    /// Style: Type of heading style to enforce. Possible values are: "atx", "setext" or "consistent"
    #[arg(global = true, long, required = false)]
    pub style_headings: Option<String>,
    /// Style: Type of unordered list style to enforce. Possible values are: "dash", "asterisk", "plus" or "consistent"
    #[arg(global = true, long, required = false)]
    pub style_unordered_lists: Option<String>,
    /// Style: Amount of spaces to use after list markers. Defaults to 1
    #[arg(global = true, long, required = false)]
    pub style_num_spaces_after_list_marker: Option<u8>,
    /// Style: Type of bold element style to enforce. Possible values are: "asterisk", "underscore" or "consistent"
    #[arg(global = true, long, required = false)]
    pub style_bold: Option<String>,
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
