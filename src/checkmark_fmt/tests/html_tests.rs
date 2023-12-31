mod utils;

/// Block quote(strikethrough)
#[test]
fn html_preserves_newline() {
    utils::assert_unchanged_after_formatting(
        "# H1

<img src='ing.png' alt='alt' height='100'/>

Example paragraph that follows html.
",
    );

    utils::assert_unchanged_after_formatting(
        "# H1

<img src='one.png' alt='alt' height='100'/> <img src='two.png' alt='alt' height='100'/>

Example paragraph that follows html.
",
    );

    utils::assert_unchanged_after_formatting(
        "# H1

<custom-tag data-attr='ing.png'/>

Example paragraph that follows custom html.
",
    );

    utils::assert_unchanged_after_formatting(
        "# H1

<img src='ing.png' alt='alt' height='100'/>
Example paragraph that follows html.
",
    );

    utils::assert_changed_after_formatting(
        "# H1
<img src='ing.png' alt='alt' height='100'/>
Example paragraph that follows html.
",
        "# H1

<img src='ing.png' alt='alt' height='100'/>
Example paragraph that follows html.
",
    );
}

#[test]
fn html_format_when_several_newlines() {
    utils::assert_changed_after_formatting(
        "# H1
<img src='ing.png' alt='alt' height='100'/>


Example paragraph that follows html.
",
        "# H1

<img src='ing.png' alt='alt' height='100'/>

Example paragraph that follows html.
",
    );
}
