use checkmark_lint_common::*;
use checkmark_lint_macro::*;
use common::ast::{try_cast_to_code, BfsIterator};

#[rule(
    requirement = "Code block style should be consistent",
    rationale = "Consistent formatting makes it easier to understand a document",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md046.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md046(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    let style = CodeBlockStyle::Consistent; // TODO: Make configurable.
    let code_blocks = BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_code(n))
        .collect::<Vec<&Code>>();

    let preferred_style = match style {
        CodeBlockStyle::Consistent => {
            if let Some(c) = code_blocks.first() {
                get_code_block_style(c, &file.content)
            } else {
                CodeBlockStyle::Fenced
            }
        }
        CodeBlockStyle::Fenced => CodeBlockStyle::Fenced,
        CodeBlockStyle::Indented => CodeBlockStyle::Indented,
    };

    code_blocks
        .iter()
        .filter(|c| get_code_block_style(c, &file.content).ne(&preferred_style))
        .map(|c| {
            let expected_style = preferred_style.as_str();
            let binding = get_code_block_style(c, &file.content);
            let actual_style = binding.as_str();
            let mut violation = ViolationBuilder::default()
                .message("Inconsistent code block style")
                .assertion(&format!("Expected {expected_style} code block style, got {actual_style}"));
            if style.eq(&CodeBlockStyle::Consistent) {
                // Give a hint that the first code block is the one that is used to determine the style
                violation = violation
                    .push_fix(&format!("Code block style is configured to be consistent across the document. First code block has a {expected_style} style, but this one is {actual_style}"));
            } else {
                violation = violation
                    .push_fix(&format!("Code block style is configured to be {expected_style}, but this one is {actual_style}"));
            }
            if preferred_style.eq(&CodeBlockStyle::Fenced) && code_block_is_fenced_and_indented(c, &file.content) {
                // When code blocks are expected to be fenced it is ok to have
                // a fenced block that with indentation. Most likely, the intent
                // of the user was to have a fenced code block inside a list item.
                // Thus give him a hint how to do it properly
                violation = violation.push_fix("It seems that you are indenting a fenced code block. If your intent is to have a fenced code block within the list item, then please make sure that the code block is aligned with a list item.
For example:

- List item

   ```sh
   echo Hello
   ```

Otherwise, remove the indentation from the code block.");
            } else {
                violation = violation.push_fix(&format!("Consider changing it to the {expected_style} code block style"));
            }
            violation = violation
                    .push_fix("See code block reference: https://www.markdownguide.org/extended-syntax/#fenced-code-blocks")
                    .position(&c.position);
            violation.build()
        })
        .collect::<Vec<Violation>>()
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

/// Take code node and original file and determine which style it is.
fn get_code_block_style(c: &Code, source: &str) -> CodeBlockStyle {
    let offset_start = c.position.as_ref().unwrap().start.offset;
    let offset_end = c.position.as_ref().unwrap().end.offset;
    let text = source.get(offset_start..offset_end).unwrap_or("");
    if text.starts_with("```") && text.ends_with("```") {
        CodeBlockStyle::Fenced
    } else {
        CodeBlockStyle::Indented
    }
}

// Return true when code block is fenced and indented at the same time. Example:
//
// # Document
//
//   ```sh
//   echo Hello
//   ```
//
fn code_block_is_fenced_and_indented(c: &Code, source: &str) -> bool {
    let offset_start = c.position.as_ref().unwrap().start.offset;
    let offset_end = c.position.as_ref().unwrap().end.offset;
    let text = source.get(offset_start..offset_end).unwrap_or("");
    !text.starts_with("```") && text.contains("```")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# Test document

## Fenced block

```bash
echo 'Hello World'
```

## Indented block

    echo 'Hello World'

## Fenced block again

    echo 'Hello World'

")]
    fn detect_inconsistent_code_blocks(ast: &Node, file: &MarkDownFile, config: &Config) {
        // Consistent code block checks
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Inconsistent code block style")
                    .assertion("Expected fenced code block style, got indented")
                    .set_fixes(vec![
                        String::from("Code block style is configured to be consistent across the document. First code block has a fenced style, but this one is indented"),
                        String::from("Consider changing it to the fenced code block style"),
                        String::from("See code block reference: https://www.markdownguide.org/extended-syntax/#fenced-code-blocks")
                    ])
                    .position(&Some(Position::new(11, 1, 85, 11, 23, 107)))
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent code block style")
                    .assertion("Expected fenced code block style, got indented")
                    .set_fixes(vec![
                        String::from("Code block style is configured to be consistent across the document. First code block has a fenced style, but this one is indented"),
                        String::from("Consider changing it to the fenced code block style"),
                        String::from("See code block reference: https://www.markdownguide.org/extended-syntax/#fenced-code-blocks")
                    ])
                    .position(&Some(Position::new(15, 1, 132, 15, 23, 154)))
                    .build(),
            ],
            MD046.check(ast, file, config)
        );

        // TODO: Make MD046 configurable, then uncomment & adjust.
        // // Check fenced style
        // assert_eq!(
        //     vec![
        //         ViolationBuilder::default()
        //             .message("Wrong code block style. Expected fenced, got indented")
        //             .position(&Some(Position::new(11, 1, 85, 11, 23, 107)))
        //             .build(),
        //         ViolationBuilder::default()
        //             .message("Wrong code block style. Expected fenced, got indented")
        //             .position(&Some(Position::new(15, 1, 132, 15, 23, 154)))
        //             .build(),
        //     ],
        //     MD046.check(ast, file, config)
        // );

        // // Check indented style
        // assert_eq!(
        //     vec![ViolationBuilder::default()
        //         .message("Wrong code block style. Expected indented, got fenced")
        //         .position(&Some(Position::new(5, 1, 34, 7, 4, 64)))
        //         .build()],
        //         MD046.check(ast, file, config)
        // );
    }
}
