use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD009")
        .message("Trailing space found")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md009.md")
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
            if is_code_block {
                false
            } else if line.starts_with("```") {
                is_code_block = !is_code_block;
                false
            } else {
                line.ends_with(" ")
            }
        })
        .map(|(i, line)| {
            log::debug!("[MD009] Problematic line {:#?}: {:#?}", i + 1, &line);
            violation_builder()
                .position(&Some(markdown::unist::Position::new(
                    i,
                    line.len() - 1,
                    common::find_offset_by_line_number(&file.content, i) + line.len() - 1,
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
This is a code block    
```
"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(1, 2, 7, 1, 3, 8)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(2, 5, 14, 2, 6, 15)))
                    .build(),
            ],
            md009_trailing_spaces(&file)
        );
    }
}
