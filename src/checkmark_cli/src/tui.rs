use colored::Colorize;

/// RAII Shows spinner + message while the check is running
/// Wen created - starts spinner(unless in CI mode)
/// When dropped - stops spinner
pub struct CheckProgressTUI {
    ci_mode: bool,
    had_any_issue: bool,
    spinner: Option<spinners::Spinner>,
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
        }
    }

    pub fn new_thread_safe(ci_mode: bool) -> std::sync::Arc<std::sync::Mutex<Self>> {
        std::sync::Arc::new(std::sync::Mutex::new(Self::new(ci_mode)))
    }

    pub fn start_spinner(&mut self, title: &str) {
        if let Some(spinner) = &mut self.spinner {
            spinner.stop_with_message("".to_owned());
        }
        self.spinner = Some(spinners::Spinner::new(
            spinners::Spinners::Line,
            format!("{}", &title.dimmed()),
        ));
    }

    pub fn finish_spinner(&mut self) {
        let message = match self.had_any_issue {
            true => format!(
                "{}: {}",
                "Check finished".cyan().bold(),
                "✗ Issues detected. See report above".red().bold()
            ),
            false => format!(
                "{}: {}",
                "Check finished".cyan().bold(),
                "✓ No issues detected".green().bold()
            ),
        };
        if let Some(spinner) = &mut self.spinner {
            spinner.stop_with_message(message);
        } else {
            println!("{}", message);
        }
    }

    pub fn print_file_check_status(&mut self, file: &common::MarkDownFile) {
        self.had_any_issue = file
            .issues
            .iter()
            .any(|issue| issue.severity != common::IssueSeverity::Help);
        let mut message = match self.had_any_issue {
            true => format!("{}: {}", "✗ Has issues".red().bold(), file.path),
            false => format!("{}: {}", "✓ Ok".green().bold(), file.path),
        };
        if self.spinner.is_some() {
            message = format!("\r{}", &message);
        }
        println!("{}", message);
    }

    pub fn print_report(&mut self, files: &Vec<common::MarkDownFile>) {
        use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
        use codespan_reporting::files::SimpleFiles;
        use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

        let mut codespan_files = SimpleFiles::new();
        for analyzed_file in files {
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
