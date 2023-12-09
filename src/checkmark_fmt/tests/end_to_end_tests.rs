mod utils;

/// Check against markdown-test-file from: https://github.com/mxstbr/markdown-test-file
#[test]
fn test_valid_markdown_test_file_rendered_without_reformatting() {
    // let original = common::MarkDownFile {
    //     path: String::from("data/test_markdown_file.md"),
    //     content: String::from(include_str!("data/test_markdown_file.md")),
    // };
    // let formatted = checkmark_fmt::fmt_markdown(&original);
    // utils::print_diff(&original.content, &formatted.content);
    // std::fs::write("output.md", &formatted.content).unwrap();
    // assert_eq!(&original.content, &formatted.content);
}

/// Check with custom end-2-end file
#[test]
fn end_to_end_with_custom_md_file() {
    // let original = common::MarkDownFile {
    //     path: String::from("data/end_to_end.md"),
    //     content: String::from(include_str!("data/end_to_end.md")),
    // };
    // let formatted = checkmark_fmt::fmt_markdown(&original);
    // utils::print_diff(&original.content, &formatted.content);
    // std::fs::write("output.md", &formatted.content).unwrap();
    // assert_eq!(&original.content, &formatted.content);
}
