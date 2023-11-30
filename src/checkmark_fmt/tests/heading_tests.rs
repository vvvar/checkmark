// Internal - create dummy MarkdownFile in non-existing path
fn create_dummy_md_file(content: &str) -> common::MarkDownFile {
    common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(content)
    }
}

/// Valid ATX H1 is not corrupted
#[test]
fn test_h1_preserved() {
    let original = create_dummy_md_file("# This is an H1\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!(original.content, formatted.content);
}

/// Valid ATX H2 is not corrupted
#[test]
fn test_h2_preserved() {
    let original = create_dummy_md_file("## This is an H2\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!(original.content, formatted.content);
}

/// Valid Setext H1 is replaced with ATX
#[test]
fn test_h1_setext_converted() {
    let original = create_dummy_md_file("This is an H1\n=============\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!("# This is an H1\n", formatted.content);
}

/// Newline added to the H1
#[test]
fn test_h1_newline_appended() {
    let input_h1_without_newline = "# This is an H1";
    assert_eq!(format!("{}\n", &input_h1_without_newline), checkmark_fmt::fmt_markdown(&create_dummy_md_file(&input_h1_without_newline)).content);
}

/// Valid Setext H2 is replaced with ATX
#[test]
fn test_h2_setext_converted() {
    let original = create_dummy_md_file("This is an H2\n-------------\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!("## This is an H2\n", formatted.content);
}

/// Valid H3 is preserved after formatting
#[test]
fn test_h3_preserved() {
    let original = create_dummy_md_file("### This is an an H3\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!(original.content, formatted.content);
}

/// Valid H4 is preserved after formatting
#[test]
fn test_h4_preserved() {
    let original = create_dummy_md_file("#### This is an H4\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!(original.content, formatted.content);
}

/// Valid H5 is preserved after formatting
#[test]
fn test_h5_preserved() {
    let original = create_dummy_md_file("##### This is an H5\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!(original.content, formatted.content);
}

/// Valid mixed headings are recognized but converted to the ATX
#[test]
fn test_mixed_heading_styles_converted_to_atx() {
    let original = create_dummy_md_file("# This is an H1\n\nThis is an H2\n-------------\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!("# This is an H1\n\n## This is an H2\n", formatted.content);
}

/// Trailing spaces are removed with ATX-style headings
#[test]
fn test_atx_trailing_spaces_are_removed() {
    assert_eq!("# This is an H1\n", checkmark_fmt::fmt_markdown(&create_dummy_md_file("#        This is an H1\n")).content);
    assert_eq!("## This is an H2\n", checkmark_fmt::fmt_markdown(&create_dummy_md_file("##        This is an H2\n")).content);
}

/// Mixed heading order is ok
#[test]
fn test_mixed_headings_order() {
    let original = create_dummy_md_file("## This is an H2\n\n# But then comes an H1\n\n### And then H3!\n");
    let formatted = checkmark_fmt::fmt_markdown(&original);
    assert_eq!(original.content, formatted.content);
}