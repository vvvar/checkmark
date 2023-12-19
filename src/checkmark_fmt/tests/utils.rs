/// Used to print diff between two strings to looks pretty in test report
/// Use "cargo test -- --nocapture" to see it
#[allow(dead_code)]
pub fn print_diff(a: &String, b: &String) {
    let diff = similar::TextDiff::from_lines(a, b);
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let (sign, style) = match change.tag() {
                similar::ChangeTag::Delete => ("-", console::Style::new().red()),
                similar::ChangeTag::Insert => ("+", console::Style::new().green()),
                similar::ChangeTag::Equal => ("", console::Style::new()),
            };
            format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
            print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
        }
    }
}

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
    print_diff(&original.content, &formatted.content);
    // std::fs::write("output.md", &formatted.content).unwrap();
    assert_eq!(&original.content, &formatted.content);
}

/// Take markdown as an input, perform fmt and
/// check there's formatted matches expectation
#[allow(dead_code)]
pub fn assert_changed_after_formatting(source: &str, expected: &str) {
    let original = create_dummy_md_file(source);
    let formatted = checkmark_fmt::fmt_markdown(&original);
    print_diff(&expected.to_string(), &formatted.content);
    assert_eq!(&expected, &formatted.content);
}
