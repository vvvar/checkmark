mod utils;

/// Delete(strikethrough)
#[test]
fn delete() {
    utils::assert_unchanged_after_formatting("~~Deleted~~\n");
}
