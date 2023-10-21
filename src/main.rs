mod args;
mod checker;
mod link_checker;
mod md;
mod prettier;

use colored::Colorize;
use spinners::{Spinner, Spinners};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let arguments = args::read();
    for file in md::list(&arguments.root).expect("Failed to read Markdown files") {
        let mut sp = Spinner::new(Spinners::Triangle, format!("{} {}", "Check".cyan().bold(), &file).into());
        let issues = checker::check(&file).await?;
        if issues.is_empty() {
            sp.stop_with_symbol("✅");
        } else {
            sp.stop_with_symbol("❌");
            for issue in issues {
                println!("  └→❗️{}: {}: {}", "Error".bright_red().bold(), &issue.category.bold(), &issue.description);
            }
        }
    }
    return Ok(());
}
