mod utils;

/// Text
#[test]
fn text_with_special_chars() {
    utils::assert_unchanged_after_formatting(
        r#"# Heading with special characters: \<char\>

| Flag                                                   | Description                       |
| ------------------------------------------------------ | --------------------------------- |
| --gtest_filter=\<pattern\>                             | Runs only subset of tests         |
| --gtest_output=(xml\|json)\[:\<path_to_output_file\>\] | Output result in a desired format |
"#,
    );
}
