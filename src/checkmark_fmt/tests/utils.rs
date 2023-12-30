/// Internal - create dummy MarkdownFile in non-existing path
#[allow(dead_code)]
pub fn create_dummy_md_file(content: &str) -> common::MarkDownFile {
    common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(content),
        issues: vec![],
    }
}

/// Take markdown as an input, perform fmt and
/// check there's no diff between input and fmt
#[allow(dead_code)]
pub fn assert_unchanged_after_formatting(markdown: &str) {
    let original = create_dummy_md_file(markdown);
    let formatted = checkmark_fmt::fmt_markdown(&original);
    println!(
        "{}",
        prettydiff::diff_lines(&original.content, &formatted.content).to_string()
    );
    // std::fs::write("output.md", &formatted.content).unwrap();
    assert_eq!(&original.content, &formatted.content);
}

/// Take markdown as an input, perform fmt and
/// check there's formatted matches expectation
#[allow(dead_code)]
pub fn assert_changed_after_formatting(source: &str, expected: &str) {
    let original = create_dummy_md_file(source);
    let formatted = checkmark_fmt::fmt_markdown(&original);
    println!(
        "{}",
        prettydiff::diff_lines(&original.content, &formatted.content).to_string()
    );
    assert_eq!(&expected, &formatted.content);
}
