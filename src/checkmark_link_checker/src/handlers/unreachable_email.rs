use super::utils::find_all_links_in_file;
use colored::Colorize;
use common::{CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity, MarkDownFile};
use log::debug;

pub fn handle(file: &MarkDownFile, uri: &str, error_message: &str) -> Vec<CheckIssue> {
    debug!("{:#?} - handling unreachable E-Mail error", &uri);
    let mut issues: Vec<CheckIssue> = vec![];
    for offset in find_all_links_in_file(file, uri) {
        let mut issue = CheckIssueBuilder::default()
            .set_category(IssueCategory::LinkChecking)
            .set_severity(IssueSeverity::Warning)
            .set_file_path(file.path.clone())
            .set_row_num_start(1)
            .set_row_num_end(file.content.lines().count())
            .set_col_num_start(1)
            .set_col_num_end(1)
            .set_offset_start(offset.start)
            .set_offset_end(offset.end)
            .set_message(format!(
                "Unable to verify an e-mail \"{uri}\". Reason: {error_message}"
            ));
        issue = issue.push_fix(&format!(
            "🧠 {}  {}",
            "Rationale".cyan(),
            "Having a broken e-mails makes it hard for people to contact you"
        ));
        let fixes = vec![format!(
            "Is this e-mail available? Consider sending a test mail to verify it"
        )];
        for fix in fixes {
            issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
        }
        issues.push(issue.build());
    }
    issues
}
