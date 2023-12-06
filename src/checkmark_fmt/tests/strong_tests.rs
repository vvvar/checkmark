mod utils;

/// Strong
/// https://github.com/markdown-it/markdown-it-sup
#[test]
fn string() {
    utils::assert_unchanged_after_formatting("__Underscored string :)__");
    utils::assert_unchanged_after_formatting("**Underscored string :)**");
}
