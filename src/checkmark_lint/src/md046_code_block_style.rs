use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Code, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD046")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md046.md")
        .is_fmt_fixable(true)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockStyle {
    Consistent,
    Fenced,
    Indented,
}

impl CodeBlockStyle {
    pub fn as_str(&self) -> &str {
        match self {
            CodeBlockStyle::Consistent => "consistent",
            CodeBlockStyle::Fenced => "fenced",
            CodeBlockStyle::Indented => "indented",
        }
    }
}

pub fn md046_code_block_style(file: &MarkDownFile, style: &CodeBlockStyle) -> Vec<Violation> {
    log::debug!("[MD046] File: {:#?}, style: {:#?}", &file.path, &style);

    let ast = parse(&file.content).unwrap();

    // Get all code blocks
    let mut code_blocks: Vec<&Code> = vec![];
    for_each(&ast, |node| {
        if let Node::Code(c) = node {
            code_blocks.push(c);
        }
    });
    log::debug!("[MD046] Code blocks: {:#?}", &code_blocks);

    // Take code node and original file and determine which style it is.
    let get_code_block_style = |c: &Code, source: &str| -> CodeBlockStyle {
        let offset_start = c.position.as_ref().unwrap().start.offset;
        let offset_end = c.position.as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        if text.starts_with("```") && text.ends_with("```") {
            CodeBlockStyle::Fenced
        } else {
            CodeBlockStyle::Indented
        }
    };

    let preferred_style = match style {
        CodeBlockStyle::Consistent => {
            if let Some(c) = code_blocks.first() {
                get_code_block_style(&c, &file.content)
            } else {
                CodeBlockStyle::Fenced
            }
        }
        CodeBlockStyle::Fenced => CodeBlockStyle::Fenced,
        CodeBlockStyle::Indented => CodeBlockStyle::Indented,
    };
    log::debug!(
        "[MD046] Document should have code block style: {:#?}",
        &preferred_style
    );

    code_blocks
        .iter()
        .filter(|c| get_code_block_style(&c, &file.content).ne(&preferred_style))
        .map(|c| violation_builder()
            .message(&format!("Wrong code block style. Expected {}, got {}", preferred_style.as_str(), get_code_block_style(&c, &file.content).as_str()))
            .push_fix(&format!("Use {} code block style", preferred_style.as_str()))
            .push_fix("See code block reference: https://www.markdownguide.org/extended-syntax/#fenced-code-blocks")
            .position(&c.position)
            .build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md046() {
        // Consistent code block checks
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# Test document

## Fenced block

```bash
echo 'Hello World'
```

## Indented block

    echo 'Hello World'

## Fenced block again

    echo 'Hello World'

"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .message("Wrong code block style. Expected fenced, got indented")
                    .position(&Some(Position::new(11, 1, 85, 11, 23, 107)))
                    .build(),
                violation_builder()
                    .message("Wrong code block style. Expected fenced, got indented")
                    .position(&Some(Position::new(15, 1, 132, 15, 23, 154)))
                    .build(),
            ],
            md046_code_block_style(&file, &CodeBlockStyle::Consistent)
        );

        // Check fenced style
        assert_eq!(
            vec![
                violation_builder()
                    .message("Wrong code block style. Expected fenced, got indented")
                    .position(&Some(Position::new(11, 1, 85, 11, 23, 107)))
                    .build(),
                violation_builder()
                    .message("Wrong code block style. Expected fenced, got indented")
                    .position(&Some(Position::new(15, 1, 132, 15, 23, 154)))
                    .build(),
            ],
            md046_code_block_style(&file, &CodeBlockStyle::Fenced)
        );

        // Check indented style
        assert_eq!(
            vec![violation_builder()
                .message("Wrong code block style. Expected indented, got fenced")
                .position(&Some(Position::new(5, 1, 34, 7, 4, 64)))
                .build()],
            md046_code_block_style(&file, &CodeBlockStyle::Indented)
        );
    }
}
