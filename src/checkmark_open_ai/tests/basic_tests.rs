/// Block quote(strikethrough)
#[ignore]
#[test]
fn smoke_test() {
    let markdown = common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(include_str!("data/basic.md")),
    };
    let issues = checkmark_open_ai::check_md_open_ai(&markdown);

    assert_eq!(&issues, &vec![
        common::CheckIssueBuilder::default()
            .set_category(common::IssueCategory::Grammar)
            .set_file_path("this/is/a/dummy/path/to/a/file.md".to_string())
            .set_row_num_start(1)
            .set_row_num_end(1)
            .set_col_num_start(3)
            .set_col_num_end(19)
            .set_message("Consider provided grammar suggestions".to_string())
            .set_fixes(vec![
                "This is a heading".to_string()
            ])
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
    ]);
}
