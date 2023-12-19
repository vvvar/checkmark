mod cli;
mod errors;

use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use env_logger;

/// Perform an analysis according to the tool from subcommand
async fn analyze(cli: &cli::Cli, files: &mut Vec<common::MarkDownFile>) -> bool {
    let mut any_issue_found = false;
    match &cli.subcommands {
        cli::Subcommands::Fmt(fmt) => {
            for file in files {
                if fmt.check {
                    checkmark_fmt::check_md_format(file);
                    if !file.issues.is_empty() {
                        any_issue_found = true;
                    }
                } else {
                    std::fs::write(&file.path, &checkmark_fmt::fmt_markdown(&file).content)
                        .unwrap();
                }
            }
        }
        cli::Subcommands::Grammar(_) => {
            for file in files {
                checkmark_open_ai::check_grammar(file).await.unwrap();
                if !file.issues.is_empty() {
                    any_issue_found = true;
                }
            }
        }
    }
    return any_issue_found;
}

/// Produce an issue report
fn report(cli: &cli::Cli, analyzed_files: &mut Vec<common::MarkDownFile>) {
    // Always print codespan report
    let mut codespan_files = SimpleFiles::new();
    for analyzed_file in &mut *analyzed_files {
        let codespan_file_id = codespan_files.add(&analyzed_file.path, &analyzed_file.content);
        for issue in &analyzed_file.issues {
            let issue_code = match &issue.category {
                common::IssueCategory::Formatting => "Formatting",
                common::IssueCategory::Linting => "Linting",
                common::IssueCategory::LinkChecking => "LinkCheck",
                common::IssueCategory::Spelling => "Spelling",
                common::IssueCategory::Grammar => "Grammar",
                common::IssueCategory::Review => "Review",
            };
            let severity = match &issue.severity {
                common::IssueSeverity::Bug => Severity::Bug,
                common::IssueSeverity::Error => Severity::Error,
                common::IssueSeverity::Warning => Severity::Warning,
                common::IssueSeverity::Note => Severity::Note,
                common::IssueSeverity::Help => Severity::Help,
            };
            let codespan_diagnostic = Diagnostic::new(severity)
                .with_message(&issue.message)
                .with_code(issue_code)
                .with_labels(vec![Label::primary(
                    codespan_file_id,
                    issue.offset_start..issue.offset_end,
                )])
                .with_notes(issue.fixes.clone());
            let config = codespan_reporting::term::Config::default();
            codespan_reporting::term::emit(
                &mut StandardStream::stderr(ColorChoice::Always).lock(),
                &config,
                &codespan_files,
                &codespan_diagnostic,
            )
            .unwrap();
        }
    }

    // When requested - generate SARIF json
    if let Some(file_path) = &cli.sarif {
        let tool_driver = serde_sarif::sarif::ToolComponentBuilder::default()
            .name("Markdown Checker")
            .build()
            .unwrap();

        let tool = serde_sarif::sarif::ToolBuilder::default()
            .driver(tool_driver)
            .build()
            .unwrap();

        let mut results: Vec<serde_sarif::sarif::Result> = vec![];
        for analyzed_file in &mut *analyzed_files {
            results.append(
                &mut analyzed_file
                    .issues
                    .iter()
                    .map(|issue| issue.to_sarif_result())
                    .collect(),
            );
        }

        let runs = serde_sarif::sarif::RunBuilder::default()
            .tool(tool)
            .results(results)
            .build()
            .unwrap();

        let sarif = serde_sarif::sarif::SarifBuilder::default()
            .version("2.1.0")
            .runs(vec![runs])
            .build()
            .unwrap();

        if let Some(file_path) = &cli.sarif {
            std::fs::write(
                &file_path,
                format!("{}", serde_json::to_string(&sarif).unwrap()),
            )
            .unwrap();
        }
        std::fs::write(
            &file_path,
            format!("{}", serde_json::to_string(&sarif).unwrap()),
        )
        .unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), errors::AppError> {
    env_logger::init();
    let cli = cli::init();
    let mut files = checkmark_ls::ls(&cli.project_root);
    if analyze(&cli, &mut files).await {
        report(&cli, &mut files);
        return Err(errors::AppError {
            message: "Issues found during analysis. Check report for details.".to_string(),
        });
    } else {
        return Ok(());
    }
}
