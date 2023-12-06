mod utils;

/// Image as a definition
#[test]
fn image_definition() {
    utils::assert_unchanged_after_formatting(
        "[id]: https://octodex.github.com/images/dojocat.jpg \"The Cat\"",
    );
}
