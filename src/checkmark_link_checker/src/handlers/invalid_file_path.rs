use super::utils::find_all_links_in_file;
use colored::Colorize;
use common::{CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity, MarkDownFile};
use log::debug;
use std::path::Path;

pub fn handle(file: &MarkDownFile, uri: &str) -> Vec<CheckIssue> {
    debug!("{:#?} - handling invalid file path error", &uri);
    let mut issues: Vec<CheckIssue> = vec![];
    let broken_filename = Path::new(&uri).file_name().unwrap().to_str().unwrap();
    for offset in find_all_links_in_file(file, broken_filename) {
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
                "File \"{broken_filename}\" is not found in path \"{uri}\"",
            ));
        issue = issue.push_fix(&format!(
            "🧠 {}  {}",
            "Rationale".cyan(),
            "Having a broken link to a file will lead to 404 error page and confuse users"
        ));
        let fixes = vec![
            format!("Does this file really exist? Try opening {:#?} in your file explorer", &uri),
            format!("Is this a symlink? If yes, then consider replacing it with a real path. Having symlinks in a project leads to dangling references and often considered a bad practice"),
        ];
        for fix in fixes {
            issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
        }
        issues.push(issue.build());
    }
    issues
}
