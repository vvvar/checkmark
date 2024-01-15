#[cfg(test)]
use pretty_assertions::assert_eq;

const DUMMY_FILE_PATH: &str = "this/is/a/dummy/path/to/a/file.md";

#[cfg(test)]
fn assert_has_issues(content: &str, whitelist: &Vec<String>, issues: &Vec<common::CheckIssue>) {
    let markdown = common::MarkDownFile {
        path: DUMMY_FILE_PATH.to_owned(),
        content: content.to_owned(),
        issues: vec![],
    };
    assert_eq!(
        &checkmark_spelling::spell_check(
            &markdown,
            &common::Config {
                spelling: common::SpellingConfig {
                    words_whitelist: whitelist.clone(),
                    ..common::SpellingConfig::default()
                },
                ..common::Config::default()
            }
        ),
        issues
    );
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
                "You can white list this word by adding it to the \"words_whitelist\" property in the config file or by passing it with the --words-whitelist argument".to_string(),
            ],
        },
    ]);
}

#[test]
fn spelling_several_misspelled_words() {
    assert_has_issues("\n\nHere is sommm additnal txt\n", &vec![], &vec![
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 27,
            offset_start: 10,
            offset_end: 15,
            message: "Word \"sommm\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"sommm\" to \"somme\"".to_string(),
                "You can white list this word by adding it to the \"words_whitelist\" property in the config file or by passing it with the --words-whitelist argument".to_string(),
            ],
        },
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 27,
            offset_start: 16,
            offset_end: 24,
            message: "Word \"additnal\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"additnal\" to \"additional\"".to_string(),
                "You can white list this word by adding it to the \"words_whitelist\" property in the config file or by passing it with the --words-whitelist argument".to_string(),
            ],
        }
    ]);
}

#[test]
fn spelling_apostrophe_supported() {
    assert_has_no_issues("# Don't", &vec![]);
    assert_has_no_issues("# Couldn't", &vec![]);
    assert_has_no_issues("# Won't", &vec![]);
}

#[test]
fn spelling_respect_owned_form() {
    assert_has_no_issues("# Project's", &vec![]);
}

#[test]
fn spelling_skip_quoted() {
    assert_has_no_issues("# Single quoted './report.sarif'", &vec![]);
    assert_has_no_issues("# Double quoted \"./report.sarif\"", &vec![]);
}

#[test]
fn spelling_skip_numbers() {
    assert_has_no_issues("# Number here 42", &vec![]);
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
            "Cannot find any suggestion for word \"fdssryyukiuu\"".to_string(),
            "You can white list this word by adding it to the \"words_whitelist\" property in the config file or by passing it with the --words-whitelist argument".to_string(),
        ],
    },]);
}

#[test]
fn spelling_consider_abbreviation() {
    assert_has_no_issues(
        "# p.s. this is an example a.k.a. Example e.g. yeah, and etc.",
        &vec![],
    );
}
