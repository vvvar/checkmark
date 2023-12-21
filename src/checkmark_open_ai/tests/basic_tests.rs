/// Check grammar check
#[ignore = "Involves real HTTP req to OpenAI which costs money + unstable. Use manual invocation and verification."]
#[tokio::test]
async fn open_ai_grammar() {
    let mut markdown = common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(include_str!("data/basic.md")),
        issues: vec![],
    };

    checkmark_open_ai::check_grammar(&mut markdown)
        .await
        .unwrap();

    assert_eq!(
        &markdown.issues,
        &vec![
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Grammar)
                .set_severity(common::IssueSeverity::Warning)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(1)
                .set_row_num_end(1)
                .set_col_num_start(3)
                .set_col_num_end(18)
                .set_offset_start(2)
                .set_offset_end(17)
                .set_message("Statement/sentence does not look like standard English".to_string())
                .set_fixes(vec!["Consider changing to: \nThis is a header".to_string()])
                .build(),
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Grammar)
                .set_severity(common::IssueSeverity::Warning)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(3)
                .set_row_num_end(3)
                .set_col_num_start(1)
                .set_col_num_end(45)
                .set_offset_start(19)
                .set_offset_end(63)
                .set_message("Statement/sentence does not look like standard English".to_string())
                .set_fixes(vec![
                    "Consider changing to: \nAnd this is a text. Here is some additional text"
                        .to_string()
                ])
                .build(),
        ]
    );
}

/// Check review generation(not consistent)
#[ignore = "Involves real HTTP req to OpenAI which costs money + unstable. Use manual invocation and verification."]
#[tokio::test]
async fn review() {
    let mut markdown = common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(include_str!("data/basic.md")),
        issues: vec![],
    };

    checkmark_open_ai::make_a_review(&mut markdown)
        .await
        .unwrap();

    assert_eq!(
        &markdown.issues,
        &vec![
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Review)
                .set_severity(common::IssueSeverity::Help)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(0)
                .set_row_num_end(0)
                .set_col_num_start(0)
                .set_col_num_end(0)
                .set_offset_start(0)
                .set_offset_end(64)
                .set_message("Consider review of your document".to_string())
                .push_fix(
                    "The document has several issues with grammar, punctuation, and formatting."
                )
                .build(),
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Review)
                .set_severity(common::IssueSeverity::Note)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(1)
                .set_row_num_end(64)
                .set_col_num_start(1)
                .set_col_num_end(1)
                .set_offset_start(2)
                .set_offset_end(17)
                .set_message("Typo: 'headr' should be 'header'".to_string())
                .set_fixes(vec!["Consider changing to: \nThis is a header".to_string()])
                .build(),
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Review)
                .set_severity(common::IssueSeverity::Note)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(1)
                .set_row_num_end(64)
                .set_col_num_start(1)
                .set_col_num_end(1)
                .set_offset_start(39)
                .set_offset_end(63)
                .set_message("Typo: 'txt' should be 'text'".to_string())
                .set_fixes(vec![
                    "Consider changing to: \nHere is some additional text".to_string()
                ])
                .build(),
        ]
    );
}
