use clap::Parser;
use clap::ValueHint;

/// MarkDown files checker
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Check MarkDown files for validity, formatting, grammar and links availability(web URLs as well as file links)"
)]
pub struct Args {
    /// Root of your project. Recursively search for Markdown files in this folder
    #[arg(value_name="ROOT_PATH", value_hint=ValueHint::DirPath, default_value=".")]
    pub root: String,

    /// Comma-separated list of URI globs(web links or files) that will be ignored by link checker.
    /// You can also provide this arguments multiple times
    #[arg(short, long, value_name="URI", value_hint=ValueHint::Url, num_args = 1.., value_delimiter=',')]
    pub ignore_url: Vec<String>,

    /// Perform auto-formatting of files.
    /// Note: All checker flags will be ignored in this mode.
    /// Note: This will modify your files!
    #[arg(short, long)]
    pub autoformat: bool,
}

pub fn read() -> Args {
    return Args::parse();
}
