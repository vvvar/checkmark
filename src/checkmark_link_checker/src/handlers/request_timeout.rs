use super::utils::find_all_links_in_file;
use common::{CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity, MarkDownFile};
use log::debug;

pub fn handle(file: &MarkDownFile, uri: &str, timeout: u64, max_retries: u64) -> Vec<CheckIssue> {
    debug!("{uri} - handling request timeout");
    let mut issues: Vec<CheckIssue> = vec![];
    for offset in find_all_links_in_file(file, &uri) {
        let issue = CheckIssueBuilder::default()
            .set_category(IssueCategory::LinkChecking)
            .set_severity(IssueSeverity::Warning)
            .set_file_path(file.path.clone())
            .set_row_num_start(1)
            .set_row_num_end(file.content.lines().count())
            .set_col_num_start(1)
            .set_col_num_end(1)
            .set_offset_start(offset.start)
            .set_offset_end(offset.end)
            .set_message(format!("Request timeout for url {uri}"))
            .set_fixes(vec![
                format!("Consider increasing timeout in config file, currently its set to {timeout} seconds"),
                format!("Consider increasing maximum amount of retries in config file, currently its set to {max_retries} seconds"),
                format!("If your network requires proxy, consider setting it via HTTP_PROXY/HTTPS_PROXY env variables or configure proxy in config file"),
                format!("Consider checking your internet connection"),
            ]);
        issues.push(issue.build());
    }
    issues
}
