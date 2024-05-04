mod utils;

/// Code block
#[test]
fn code() {
    utils::assert_unchanged_after_formatting(
        "```js
console.log('Hello');
```
",
    );

    // Force append default code highlight when not set
    utils::assert_changed_after_formatting(
        "```
console.log('Hello');
```",
        "```text
console.log('Hello');
```
",
    );

    // Force convert indented style to the block quoted style
    // because when we want to preserve indents we cant distinguish
    // between tabs(indents) from code and from markdown
    utils::assert_changed_after_formatting(
        "    console.log('Hello');",
        "```text
console.log('Hello');
```
",
    );

    // Force convert one-line block-quote style to the inline
    utils::assert_changed_after_formatting(
        "```console.log('Hello');```",
        "`console.log('Hello');`
",
    );
}
