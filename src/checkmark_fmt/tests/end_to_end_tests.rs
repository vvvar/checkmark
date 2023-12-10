mod utils;

/// Check against markdown-test-file from: https://github.com/mxstbr/markdown-test-file
#[test]
fn test_valid_markdown_test_file_rendered_without_reformatting() {
    // utils::assert_unchanged_after_formatting(include_str!("data/test_markdown_file.md"));
}

/// Check with custom end-2-end file
#[test]
fn end_to_end_with_custom_md_file() {
    // utils::assert_unchanged_after_formatting(include_str!("data/end_to_end.md"));
}

/// Check with full markdown example from:
/// https://gist.github.com/allysonsilva/85fff14a22bbdf55485be947566cc09e
#[test]
fn end_to_end_with_full_markdown_md_file() {
    // utils::assert_unchanged_after_formatting(include_str!("data/full_markdown_example.md"));
}
