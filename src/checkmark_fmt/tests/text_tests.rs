mod utils;

/// Text
#[test]
fn text() {
    utils::assert_unchanged_after_formatting(
        r#"# Heading with special characters: \<char\>

| Flag                                                       | Description                       |
| ---------------------------------------------------------- | --------------------------------- |
| --gtest\_filter=\<pattern\>                                | Runs only subset of tests         |
| --gtest\_output=(xml\|json)\[:\<path\_to\_output\_file\>\] | Output result in a desired format |
"#,
    );
}
