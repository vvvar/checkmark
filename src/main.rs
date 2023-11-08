mod args;
mod checker;
mod grammar;
mod link_checker;
mod md;
mod prettier;
mod spell_checker;

use colored::Colorize;
use pathdiff;
use spinners::{Spinner, Spinners};
use std::path::PathBuf;

/// Print GCC style error: https://www.gnu.org/prep/standards/html_node/Errors.html#Errors
fn print_gcc_style_error(issues: &Vec<checker::Issue>) {
    for issue in issues {
        let check_root_abs = PathBuf::from(&args::read().root)
            .canonicalize()
            .unwrap()
            .display()
            .to_string();
        let file_path_rel = String::from(
            pathdiff::diff_paths(&issue.file_path, check_root_abs)
                .expect("Unable to determine problematic file path")
                .to_string_lossy(),
        );
        println!(
            "{}: {}: {}: {}",
            &file_path_rel.bold(),
            "error".bright_red().bold(),
            &issue.category.bold(),
            &issue.description
        );
        match &issue.issue_in_code {
            Some(issue_in_code) => {
                println!("{}", issue_in_code);
            }
            None => {}
        }
        for suggestion in &issue.suggestions {
            println!(
                "   {}{} {}",
                "Suggestion".blue(),
                ":".blue(),
                &suggestion.blue()
            );
        }
        println!("");
    }
}

/// Print custom style error
#[allow(dead_code)]
fn print_custom_style_error(issues: &Vec<checker::Issue>) {
    for issue in issues {
        println!(
            "❗️{}: {}: {}: {}",
            &issue.file_path.bold(),
            "Error".bright_red().bold(),
            &issue.category.bold(),
            &issue.description
        );
        for suggestion in &issue.suggestions {
            println!("└→💡{}: {}", "Suggestion".yellow(), &suggestion);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = args::read();
    for file in md::list(&arguments.root).expect("Failed to read Markdown files") {
        if arguments.autoformat {
            let mut sp = Spinner::new(
                Spinners::Triangle,
                format!("{}: {}", "Auto-format".cyan().bold(), &file).into(),
            );
            if prettier::auto_format(&file) {
                sp.stop_with_symbol("✅");
            } else {
                sp.stop_with_symbol("❎");
            }
        } else {
            let mut sp = Spinner::new(
                Spinners::Triangle,
                format!("{}: {}", "Check".cyan().bold(), &file).into(),
            );
            let issues = checker::check(&file).await?;
            if issues.is_empty() {
                sp.stop_with_symbol("✅");
            } else {
                sp.stop_with_symbol("❌");
                print_gcc_style_error(&issues);
            }
        }
    }
    return Ok(());
}
