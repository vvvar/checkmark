#[cfg(test)]
use pretty_assertions::assert_eq;

const DUMMY_FILE_PATH: &str = "this/is/a/dummy/path/to/a/file.md";

#[cfg(test)]
fn assert_has_issues(content: &str, whitelist: &Vec<String>, issues: &Vec<common::CheckIssue>) {
    // if let Ok(_) = env_logger::try_init() {}
    let mut markdown = common::MarkDownFile {
        path: DUMMY_FILE_PATH.to_owned(),
        content: content.to_owned(),
        issues: vec![],
    };
    checkmark_spelling::spell_check(&mut markdown, whitelist);
    assert_eq!(&markdown.issues, issues);
}

#[cfg(test)]
fn assert_has_no_issues(content: &str, whitelist: &Vec<String>) {
    assert_has_issues(&content, whitelist, &vec![]);
}

/// Basic spell checking tests
#[test]
fn spelling_plain_misspelled_word() {
    assert_has_issues("# This is a headr\n", &vec![], &vec![
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 1,
            row_num_end: 1,
            col_num_start: 3,
            col_num_end: 18,
            offset_start: 12,
            offset_end: 17,
            message: "Word \"headr\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"headr\" to \"head\"".to_string(),
                "If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)".to_string(),
            ],
        },
    ]);
}

#[test]
fn spelling_several_misspelled_words() {
    assert_has_issues("\n\nHere is som additnal txt\n", &vec![], &vec![
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 25,
            offset_start: 14,
            offset_end: 22,
            message: "Word \"additnal\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"additnal\" to \"additional\"".to_string(),
                "If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)".to_string(),
            ],
        },
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 25,
            offset_start: 23,
            offset_end: 26,
            message: "Word \"txt\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"txt\" to \"text\"".to_string(),
                "If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)".to_string(),
            ],
        },
    ]);
}

#[test]
fn spelling_apostrophe_supported() {
    assert_has_no_issues("# Don't", &vec![]);
    assert_has_no_issues("# Couldn't", &vec![]);
    assert_has_no_issues("# Won't", &vec![]);
}

#[test]
fn spelling_gibberish_handled() {
    assert_has_issues("# fdssryyukiuu's ", &vec![], &vec![common::CheckIssue {
        category: common::IssueCategory::Spelling,
        severity: common::IssueSeverity::Warning,
        file_path: DUMMY_FILE_PATH.to_owned(),
        row_num_start: 1,
        row_num_end: 1,
        col_num_start: 3,
        col_num_end: 17,
        offset_start: 2,
        offset_end: 16,
        message: "Word \"fdssryyukiuu's\" is unknown or miss-spelled".to_string(),
        fixes: vec![
            "Cannot find any suggestion for word \"fdssryyukiuu's\"".to_string(),
            "If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)".to_string(),
        ],
    },]);
}
