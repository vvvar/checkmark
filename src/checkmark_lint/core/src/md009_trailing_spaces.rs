use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement = "Trailing spaces should not be used in a document",
    rationale = "Except when being used to create a line break, trailing whitespace has no purpose and does not affect the rendering of content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md009.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md009(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    let mut is_code_block = false;
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            if line.contains("```") {
                is_code_block = !is_code_block;
            }
            if is_code_block {
                false
            } else {
                line.ends_with(' ')
            }
        })
        .map(|(i, line)| {
            ViolationBuilder::default()
                .message("Found trailing space")
                .assertion("Expected no trailing space, found one")
                .position(&Some(Position::new(
                    i,
                    1,
                    common::find_offset_by_line_number(&file.content, i),
                    i,
                    line.len(),
                    common::find_offset_by_line_number(&file.content, i) + line.len(),
                )))
                .push_fix("Remove trailing space")
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# H1
Trailing space #1 

## H2

```text
This trailing space does not count
```

## Trailing space #2 

    ```text
    This trailing space does not count, but one line below does
    ``` 

## Trailing space #3 
")]
    fn detect_trailing_space(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Found trailing space")
                    .assertion("Expected no trailing space, found one")
                    .set_fixes(vec![String::from("Remove trailing space")])
                    .position(&Some(Position::new(1, 1, 5, 1, 18, 23)))
                    .build(),
                ViolationBuilder::default()
                    .message("Found trailing space")
                    .assertion("Expected no trailing space, found one")
                    .set_fixes(vec![String::from("Remove trailing space")])
                    .position(&Some(Position::new(9, 1, 80, 9, 21, 101)))
                    .build(),
                ViolationBuilder::default()
                    .message("Found trailing space")
                    .assertion("Expected no trailing space, found one")
                    .set_fixes(vec![String::from("Remove trailing space")])
                    .position(&Some(Position::new(13, 1, 179, 13, 8, 187)))
                    .build(),
                ViolationBuilder::default()
                    .message("Found trailing space")
                    .assertion("Expected no trailing space, found one")
                    .set_fixes(vec![String::from("Remove trailing space")])
                    .position(&Some(Position::new(15, 1, 189, 15, 21, 210)))
                    .build(),
            ],
            MD009.check(ast, file, config)
        );
    }
}
