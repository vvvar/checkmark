use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD009")
        .message("Trailing space found")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md009.md")
        .rationale("Except when being used to create a line break, trailing whitespace has no purpose and does not affect the rendering of content")
        .push_fix("Remove trailing space")
        .is_fmt_fixable(true)
}

pub fn md009_trailing_spaces(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD009] File: {:#?}", &file.path);
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
            log::debug!("[MD009] Problematic line {:#?}: {:#?}", i + 1, &line);
            violation_builder()
                .position(&Some(markdown::unist::Position::new(
                    i,
                    1,
                    common::find_offset_by_line_number(&file.content, i),
                    i,
                    line.len(),
                    common::find_offset_by_line_number(&file.content, i) + line.len(),
                )))
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md009() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1
   
## H2 

```text
This is a fenced code block    
```

## Trailing space after code block 

    ```text
    This is an indented code block    
    ```

## Trailing space after indented block 
"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(1, 1, 5, 1, 3, 8)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(2, 1, 9, 2, 6, 15)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(8, 1, 62, 8, 35, 97)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(
                        14, 1, 159, 14, 39, 198
                    )))
                    .build(),
            ],
            md009_trailing_spaces(&file)
        );
    }
}
