/// Check grammar check
#[ignore]
#[test]
fn grammar() {
    let markdown = common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(include_str!("data/basic.md")),
    };
    let issues = checkmark_open_ai::check_grammar(&markdown);

    assert_eq!(
        &issues,
        &vec![
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Grammar)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(1)
                .set_row_num_end(1)
                .set_col_num_start(3)
                .set_col_num_end(19)
                .set_message("Consider provided grammar suggestions".to_string())
                .set_fixes(vec!["This is a heading".to_string()])
                .build(),
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Grammar)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(3)
                .set_row_num_end(3)
                .set_col_num_start(1)
                .set_col_num_end(45)
                .set_message("Consider provided grammar suggestions".to_string())
                .set_fixes(vec![
                    "And this is a text. Here is some additional text.".to_string()
                ])
                .build(),
        ]
    );
}

/// Check review generation(not consistent) 
#[ignore]
#[test]
fn review() {
    let markdown = common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(include_str!("data/basic.md")),
    };

    let issues = checkmark_open_ai::make_a_review(&markdown);

    assert_eq!(
        &issues,
        &vec![
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Review)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(0)
                .set_row_num_end(0)
                .set_col_num_start(0)
                .set_col_num_end(0)
                .set_message("The project documentation needs improvement in terms of grammar, punctuation, formatting, and clarity.".to_string())
                .build(),
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Review)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(1)
                .set_row_num_end(1)
                .set_col_num_start(3)
                .set_col_num_end(3)
                .set_message("Header is not properly formatted.".to_string())
                .set_fixes(vec!["Add a # symbol before the header text.".to_string()])
                .build(),
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Review)
                .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
                .set_row_num_start(3)
                .set_row_num_end(3)
                .set_col_num_start(0)
                .set_col_num_end(0)
                .set_message("Extra word 'txt'.".to_string())
                .set_fixes(vec!["Remove the word 'txt'.".to_string()])
                .build(),
        ]
    );
}
