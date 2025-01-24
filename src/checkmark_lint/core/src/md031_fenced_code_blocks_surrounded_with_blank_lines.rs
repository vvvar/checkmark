use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use common::ast::{is_list_item, try_cast_to_code, BfsIterator};

#[rule(
    requirement = "Fenced code blocks should be surrounded by blank lines",
    rationale = "Aside from aesthetic reasons, some parsers, including kramdown, will not parse fenced code blocks that don't have blank lines before and after them",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/main/doc/md031.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md031(ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation> {
    BfsIterator::from(ast)
        .filter(|n| {
            if !config.linter.md031_list_items {
                // Filter-out list items only when code blocks from them has to be excluded
                is_list_item(n)
            } else {
                true
            }
        })
        .filter_map(|n| try_cast_to_code(n))
        .filter(|c| is_code_block_fenced(c, &file.content))
        .filter(|c| !is_code_block_surrounded_with_blank_lines(c, &file.content))
        .map(|c| violation_builder().position(&c.position).build())
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Fenced code blocks should be surrounded by blank lines")
        .assertion("Expected blank lines before/after code block, got none")
}

fn is_code_block_fenced(c: &Code, source: &str) -> bool {
    let offset_start = c.position.as_ref().unwrap().start.offset;
    let offset_end = c.position.as_ref().unwrap().end.offset;
    let text = source.get(offset_start..offset_end).unwrap_or("");
    text.starts_with("```")
}

fn is_code_block_surrounded_with_blank_lines(c: &Code, source: &str) -> bool {
    let start = c.position.as_ref().unwrap().start.line;
    let end = c.position.as_ref().unwrap().end.line;
    let line_before = source.lines().nth(start - 2).unwrap_or("MISSING_LINE");
    if end.eq(&source.lines().count()) {
        // Special case - code block is at the end of a file.
        // We need this special handling here since std::lines()
        // does not account trailing line break as a line.
        // Since we know that there's nothing beyond code block,
        // we can just check that whole file ends with newline.
        line_before.is_empty() && source.ends_with('\n')
    } else {
        // Normal case - code block placed between other blocks.
        // Here std::lines() can correctly see a following line.
        let line_after = source.lines().nth(end).unwrap_or("MISSING_LINE");
        line_before.is_empty() && line_after.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = r#"
```
echo Hello
```
"#)]
    fn code_block_at_the_end_should_not_cause_violation(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(Vec::<Violation>::new(), MD031.check(ast, file, config));
    }

    #[rule_test(markdown = r#"
```
var i = 1;
```

```
var i = 2;
```"#)]
    fn should_detect_missing_blank_line(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        // Plain case
        config.linter.md031_list_items = true;
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(6, 1, 21, 8, 4, 39)))
                .build()],
            MD031.check(ast, file, config),
        );
    }

    #[rule_test(markdown = r#"
1. List Item One
```
var i = 2;
```"#)]
    fn should_detect_missing_blank_line_in_list_item(
        ast: &Node,
        file: &MarkDownFile,
        config: &mut Config,
    ) {
        // Plain case
        config.linter.md031_list_items = true;
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 18, 5, 4, 36)))
                .build()],
            MD031.check(ast, file, config),
        );
    }

    #[rule_test(markdown = r#"
1. List Item One
```
var i = 2;
```"#)]
    fn should_ignore_missing_blank_line_in_list_when_disabled(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        // Plain case
        assert_eq!(Vec::<Violation>::new(), MD031.check(ast, file, config));
    }
}
