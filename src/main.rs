mod args;
mod checker;
mod grammar;
mod link_checker;
mod linter;
mod md;
mod prettier;
mod spell_checker;

use colored::Colorize;
use env_logger;
use log::warn;
use std::fmt;
use spinners::{Spinner, Spinners};

#[derive(Debug)]
struct AppError {
    message: String,
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Print GCC style error: https://www.gnu.org/prep/standards/html_node/Errors.html#Errors
fn print_gcc_style_error(issues: &Vec<checker::Issue>) {
    for issue in issues {
        eprintln!(
            "{}: {}: {}: {}",
            &issue.file_path.bold(),
            "error".bright_red().bold(),
            &issue.category.bold(),
            &issue.description
        );
        match &issue.issue_in_code {
            Some(issue_in_code) => {
                eprintln!("{}", issue_in_code);
            }
            None => {}
        }
        for suggestion in &issue.suggestions {
            eprintln!(
                "   {}{} {}",
                "Suggestion".blue(),
                ":".blue(),
                &suggestion.blue()
            );
        }
        eprintln!();
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
async fn main() -> Result<(), AppError> {
    env_logger::init();
    let arguments = args::read();
    let mut has_any_issue = false;
    for file in md::list(&arguments.root) {
        if arguments.autoformat {
            // let mut sp = Spinner::new(
            //     Spinners::Triangle,
            //     format!("{}: {}", "Auto-format".cyan().bold(), &file).into(),
            // );
            if prettier::auto_format(&file) {
                println!("✅{}: {}", "Auto-format".cyan().bold(), &file);
                // sp.stop_with_symbol("✅");
            } else {
                println!("❎{}: {}", "Auto-format".cyan().bold(), &file);
                // sp.stop_with_symbol("❎");
            }
        } else {
            let mut sp = Spinner::new(
                Spinners::Point,
                format!("{}: {}", "Check".cyan().bold(), &file).into(),
            );
            if let Ok(issues) = checker::check(&file).await {
                if issues.is_empty() {
                    sp.stop_with_symbol("✅");
                } else {
                    sp.stop_with_symbol("❌");
                    print_gcc_style_error(&issues);
                    has_any_issue = true;
                }
            } else {
                sp.stop_with_symbol("❌");
                has_any_issue = true;
                warn!("Unexpected error while checking a file");
            }
        }
    }
    if has_any_issue {
        Err(AppError {
            message: String::from("Some files failed a check"),
        })
    } else {
        Ok(())
    }
}
