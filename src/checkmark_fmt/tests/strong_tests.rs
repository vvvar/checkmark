mod utils;

/// Strong
/// https://github.com/markdown-it/markdown-it-sup
#[test]
fn strong() {
    utils::assert_changed_after_formatting("__Hello__\n", "**Hello**\n");
    utils::assert_unchanged_after_formatting("**Underscored string :)**\n");
    utils::assert_unchanged_after_formatting("**Bold\nwith new-line**\n");
}
