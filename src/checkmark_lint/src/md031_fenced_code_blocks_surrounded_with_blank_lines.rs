use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::Code;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD031")
        .message("Fenced code blocks should be surrounded by blank lines")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/main/doc/md031.md")
        .rationale("Aside from aesthetic reasons, some parsers, including kramdown, will not parse fenced code blocks that don't have blank lines before and after them.")
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
    let line_after = source.lines().nth(end).unwrap_or("MISSING_LINE");
    line_before.is_empty() && line_after.is_empty()
}

pub fn md031_fenced_code_blocks_surrounded_with_blank_lines(
    file: &MarkDownFile,
    list_items: bool,
) -> Vec<Violation> {
    let ast = common::ast::parse(&file.content).unwrap();
    common::ast::BfsIterator::from(&ast)
        .filter(|n| {
            if !list_items {
                // Filter-out list items only when code blocks from them has to be excluded
                common::ast::is_list_item(n)
            } else {
                true
            }
        })
        .filter_map(|n| common::ast::try_cast_to_code(n))
        .filter(|c| is_code_block_fenced(c, &file.content))
        .filter(|c| !is_code_block_surrounded_with_blank_lines(c, &file.content))
        .map(|c| violation_builder().position(&c.position).build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md031_detect_missing_blank_line() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: r#"
```
var i = 1;
```

```
var i = 2;
```"#
                .to_string(),
            issues: vec![],
        };

        // Plain case
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(6, 1, 21, 8, 4, 39)))
                .build()],
            md031_fenced_code_blocks_surrounded_with_blank_lines(&file, true),
        );
    }

    #[test]
    fn md031_detect_missing_blank_line_in_list_item() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: r#"
1. List Item One
```
var i = 2;
```"#
                .to_string(),
            issues: vec![],
        };

        // Plain case
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 18, 5, 4, 36)))
                .build()],
            md031_fenced_code_blocks_surrounded_with_blank_lines(&file, true),
        );
    }

    #[test]
    fn md031_ignore_missing_blank_line_in_list_when_disabled() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: r#"
1. List Item One
```
var i = 2;
```"#
                .to_string(),
            issues: vec![],
        };

        // Plain case
        assert_eq!(
            Vec::<Violation>::new(),
            md031_fenced_code_blocks_surrounded_with_blank_lines(&file, false),
        );
    }
}
