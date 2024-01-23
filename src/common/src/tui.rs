use crate::*;
use colored::Colorize;
use spinners::{Spinner, Spinners::Point};
use std::sync::{Arc, Mutex};

/// RAII Shows spinner + message while the check is running
/// Wen created - starts spinner(unless in CI mode)
/// When dropped - stops spinner
pub struct CheckProgressTUI {
    ci_mode: bool,
    had_any_issue: bool,
    spinner: Option<Spinner>,
    custom_finish_message: Option<String>,
}

impl Drop for CheckProgressTUI {
    fn drop(&mut self) {
        self.finish_spinner();
    }
}

impl CheckProgressTUI {
    pub fn new(ci_mode: bool) -> Self {
        Self {
            ci_mode,
            had_any_issue: false,
            spinner: None,
            custom_finish_message: None,
        }
    }

    pub fn set_custom_finish_message(&mut self, message: &str) {
        self.custom_finish_message = Some(message.to_string());
    }

    pub fn new_thread_safe(ci_mode: bool) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(ci_mode)))
    }

    pub fn start_spinner(&mut self, title: &str) {
        if let Some(spinner) = &mut self.spinner {
            spinner.stop_with_newline();
        }
        self.spinner = Some(Spinner::new(Point, format!("{}", &title.dimmed())));
    }

    pub fn finish_spinner(&mut self) {
        let message = if self.custom_finish_message.is_some() {
            self.custom_finish_message.take().unwrap()
        } else if self.had_any_issue {
            format!(
                "{}: {}",
                "Check finished".cyan().bold(),
                "✗ Issues detected. See report above".red().bold()
            )
        } else {
            format!(
                "{}: {}",
                "Check finished".cyan().bold(),
                "✓ No issues detected".green().bold()
            )
        };
        if let Some(spinner) = &mut self.spinner {
            spinner.stop_with_message(message);
        } else {
            println!("{}", message);
        }
    }

    pub fn print_file_check_status(&mut self, file: &MarkDownFile) {
        self.had_any_issue = file
            .issues
            .iter()
            .any(|issue| issue.severity != IssueSeverity::Help);
        let mut message = match self.had_any_issue {
            true => format!("{}: {}", "✗ Has issues".red().bold(), file.path),
            false => format!("{}: {}", "✓ Ok".green().bold(), file.path),
        };
        if self.spinner.is_some() {
            message = format!("\r{}", &message);
        }
        println!("{}", message);
    }

    pub fn print_report(&mut self, files: &Vec<MarkDownFile>) {
        use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
        use codespan_reporting::files::SimpleFiles;
        use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

        let mut codespan_files = SimpleFiles::new();
        for analyzed_file in files {
            let codespan_file_id = codespan_files.add(&analyzed_file.path, &analyzed_file.content);
            for issue in &analyzed_file.issues {
                let issue_code = match &issue.category {
                    IssueCategory::Formatting => "Formatting",
                    IssueCategory::Linting => "Linting",
                    IssueCategory::LinkChecking => "LinkCheck",
                    IssueCategory::Spelling => "Spelling",
                    IssueCategory::Grammar => "Grammar",
                    IssueCategory::Review => "Review",
                };
                let severity = match &issue.severity {
                    IssueSeverity::Bug => Severity::Bug,
                    IssueSeverity::Error => Severity::Error,
                    IssueSeverity::Warning => Severity::Warning,
                    IssueSeverity::Note => Severity::Note,
                    IssueSeverity::Help => Severity::Help,
                };
                let mut codespan_diagnostic = Diagnostic::new(severity)
                    .with_message(&issue.message)
                    .with_code(issue_code)
                    .with_notes(issue.fixes.clone());
                if severity == Severity::Help {
                    codespan_diagnostic = codespan_diagnostic.with_labels(vec![Label::primary(
                        codespan_file_id,
                        0..analyzed_file.content.len(),
                    )]);
                } else {
                    codespan_diagnostic = codespan_diagnostic.with_labels(vec![Label::primary(
                        codespan_file_id,
                        issue.offset_start..issue.offset_end,
                    )]);
                }
                let config = codespan_reporting::term::Config::default();
                let writer = if self.ci_mode {
                    StandardStream::stderr(ColorChoice::Never)
                } else {
                    StandardStream::stderr(ColorChoice::Auto)
                };
                codespan_reporting::term::emit(
                    &mut writer.lock(),
                    &config,
                    &codespan_files,
                    &codespan_diagnostic,
                )
                .unwrap();
            }
        }
    }
}
