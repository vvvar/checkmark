use super::utils::find_all_links_in_file;
use colored::Colorize;
use common::{CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity, MarkDownFile};
use log::debug;
use reqwest::StatusCode;

pub fn handle(file: &MarkDownFile, uri: &str, error: &reqwest::Error) -> Vec<CheckIssue> {
    debug!("{uri} - handling network error: {error}");
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
            .set_message(format!("{error}"));
        issue = issue.push_fix(&format!(
            "🧠 {}  {}",
            "Rationale".cyan(),
            "Having a broken hyperlink is a bad, confusing user experience"
        ));
        let fixes = vec![
            format!("If your network requires a proxy, consider setting it via HTTP_PROXY/HTTPS_PROXY env variables"),
            format!("Consider checking your internet connection"),
            format!("Can you open this link in a browser? If no then perhaps its broken and shall be fixed"),
        ];
        for fix in fixes {
            issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
        }
        issue = issue.push_fix(&format!(
            "📚 {}       https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/{}",
            "Docs".cyan(),
            error
                .status()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                .as_str()
        ));
        issues.push(issue.build());
    }
    issues
}
