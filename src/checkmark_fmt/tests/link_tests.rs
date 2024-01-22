mod utils;

/// Normal link renders
#[test]
fn normal_link_rendered() {
    utils::assert_unchanged_after_formatting(
        "[Normal link](https://github.com \"Normal title\")\n",
    );

    utils::assert_unchanged_after_formatting(
        "[https://google.com](https://github.com \"Link in link\")\n",
    );
}

/// Auto-links
/// https://sgmljs.net/docs/markdown-autolink-examples.html
#[test]
fn auto_links() {
    utils::assert_changed_after_formatting("https://github.com", "<https://github.com>\n");
    utils::assert_changed_after_formatting("someone@some.where", "<mailto:someone@some.where>\n");
}

/// Footnotes
/// https://www.markdownguide.org/extended-syntax/#footnotes
#[test]
fn link_reference() {
    utils::assert_unchanged_after_formatting(
        "# Attribution

This Code of Conduct is adapted from the [Contributor Covenant][homepage],
version 2.1, available at
[https://www.contributor-covenant.org/version/2/1/code_of_conduct.html][v2.1].

[homepage]: https://www.contributor-covenant.org
[v2.1]: https://www.contributor-covenant.org/version/2/1/code_of_conduct.html
",
    );

    utils::assert_unchanged_after_formatting(
        "# Attribution

Community Impact Guidelines were inspired by
[Mozilla's code of conduct enforcement ladder][Mozilla CoC].

[Mozilla CoC]: https://github.com/mozilla/diversity
",
    );
}
