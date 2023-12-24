#[cfg(test)]
use pretty_assertions::assert_eq;

/// Block quote(strikethrough)
#[test]
fn spell_check() {
    let file_path = String::from("this/is/a/dummy/path/to/a/file.md");

    let mut markdown = common::MarkDownFile {
        path: file_path.clone(),
        content: String::from(include_str!("data/basic.md")),
        issues: vec![],
    };
    checkmark_spelling::spell_check(&mut markdown, &vec![]);
    assert_eq!(&markdown.issues, &vec![
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: file_path.clone(),
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
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: file_path.clone(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 45,
            offset_start: 51,
            offset_end: 59,
            message: "Word \"additnal\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"additnal\" to \"additional\"".to_string(),
                "If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)".to_string(),
            ],
        },
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: file_path.clone(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 45,
            offset_start: 60,
            offset_end: 63,
            message: "Word \"txt\" is unknown or miss-spelled".to_string(),
            fixes: vec![
                "Consider changing \"txt\" to \"text\"".to_string(),
                "If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)".to_string(),
            ],
        },
    ]);
}
