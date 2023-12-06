mod utils;

/// Subscript
/// https://github.com/markdown-it/markdown-it-sub
#[test]
fn subscript() {
    utils::assert_unchanged_after_formatting("19^th^");
}

/// Superscript
/// https://github.com/markdown-it/markdown-it-sup
#[test]
fn superscript() {
    utils::assert_unchanged_after_formatting("H~2~O");
}
