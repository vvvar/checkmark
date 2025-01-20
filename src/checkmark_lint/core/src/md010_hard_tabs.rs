use checkmark_lint_common::*;
use checkmark_lint_macro::*;
use common::find_offset_by_line_number;

#[rule(
    requirement = "Hard tabs should not be used",
    rationale = "Hard tabs are often rendered inconsistently by different editors and can be harder to work with than spaces",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md010.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md010(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
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
                line.contains('\t')
            }
        })
        .map(|(i, line)| {
            ViolationBuilder::default()
                .message("Found hard tab")
                .assertion("Expected space, got hard tab")
                .push_fix("Replace hard tab with space or remove them")
                .position(&Some(Position::new(
                    i,
                    1,
                    find_offset_by_line_number(&file.content, i),
                    i,
                    line.len(),
                    find_offset_by_line_number(&file.content, i) + line.len(),
                )))
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# H1\t
\t\t\t\t
## H2\t

```sh
\t\techo It's ok to have tabs in code blocks
```
")]
    fn detect_hard_tabs(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Found hard tab")
                    .assertion("Expected space, got hard tab")
                    .push_fix("Replace hard tab with space or remove them")
                    .position(&Some(Position::new(0, 1, 0, 0, 5, 5)))
                    .build(),
                ViolationBuilder::default()
                    .message("Found hard tab")
                    .assertion("Expected space, got hard tab")
                    .push_fix("Replace hard tab with space or remove them")
                    .position(&Some(Position::new(1, 1, 6, 1, 4, 10)))
                    .build(),
                ViolationBuilder::default()
                    .message("Found hard tab")
                    .assertion("Expected space, got hard tab")
                    .push_fix("Replace hard tab with space or remove them")
                    .position(&Some(Position::new(2, 1, 11, 2, 6, 17)))
                    .build()
            ],
            MD010.check(ast, file, config)
        );
    }
}
