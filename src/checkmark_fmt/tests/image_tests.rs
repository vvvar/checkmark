mod utils;

/// Normal image with alt and title
#[test]
fn image() {
    utils::assert_unchanged_after_formatting(
        "![Cat](https://octodex.github.com/images/stormtroopocat.jpg \"The Cat\")",
    );
}
