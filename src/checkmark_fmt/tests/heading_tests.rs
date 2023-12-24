mod utils;

/// Valid ATX heading is preserved
#[test]
fn heading_atx() {
    utils::assert_unchanged_after_formatting("# This is an H1\n");
    utils::assert_unchanged_after_formatting("## This is an H2\n");
    utils::assert_unchanged_after_formatting("### This is an H3\n");
    utils::assert_unchanged_after_formatting("#### This is an H4\n");
    utils::assert_unchanged_after_formatting("##### This is an H5\n");
    utils::assert_unchanged_after_formatting("###### This is an H6\n");
}

/// Trailing spaces are removed with ATX-style headings
#[test]
fn heading_atx_trailing_space_removed() {
    assert_eq!(
        "# This is an H1\n",
        checkmark_fmt::fmt_markdown(&utils::create_dummy_md_file("#        This is an H1\n"))
            .content
    );
    assert_eq!(
        "## This is an H2\n",
        checkmark_fmt::fmt_markdown(&utils::create_dummy_md_file("##        This is an H2\n"))
            .content
    );
}

/// Valid SetExt heading is preserved
#[test]
fn heading_set_ext() {
    utils::assert_unchanged_after_formatting("This is an H1\n=============\n");
    utils::assert_unchanged_after_formatting("This is an H2\n-------------\n");
}

/// Valid mixed headings are recognized and preserved
#[test]
fn heading_mixed_atx_set_ext() {
    utils::assert_unchanged_after_formatting("# This is an H1\n\nThis is an H2\n-------------\n");
}

/// Mixed heading order is ok
#[test]
fn heading_mixed_depth_ok() {
    utils::assert_unchanged_after_formatting(
        "## This is an H2\n\n# But then comes an H1\n\n### And then H3!\n",
    );
}

/// Markdown heading supports maximum depth level 6
/// Nevertheless, level 7 shall still be rendered but as a plain text
#[test]
fn heading_invalid_level_not_ignored() {
    utils::assert_unchanged_after_formatting("####### This is not an H7\n");
}
