mod utils;

/// Normal link renders
#[test]
fn normal_link_rendered() {
    utils::assert_unchanged_after_formatting(
        "[Normal link](https://github.com/markdown-it/markdown-it-ins \"Normal title\")",
    );
}
