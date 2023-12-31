use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use once_cell::sync::Lazy;
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD011")
        .message("Reversed link syntax")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md011.md")
        .push_fix("Reversed links are not rendered as usable links. Swap the [] and () around")
        .is_fmt_fixable(true)
}

pub fn md011_reversed_link_syntax(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD011] File: {:#?}", &file.path);

    /// Example of lint that shall match - "(link)[https://www.example.com/]"
    static REGEX_CORRECTIONS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(.*\)\[.*\]").unwrap());

    let vector_match: Vec<Vec<Violation>> = REGEX_CORRECTIONS
        .captures_iter(&file.content)
        .map(|c| {
            c.iter()
                .map(|m| {
                    let offset = m.unwrap().range();
                    let line_positions = line_numbers::LinePositions::from(file.content.as_str());
                    violation_builder()
                        .position(&Some(markdown::unist::Position::new(
                            line_positions.from_offset(offset.start).as_usize(),
                            1,
                            offset.start,
                            line_positions.from_offset(offset.end).as_usize(),
                            1,
                            offset.end,
                        )))
                        .build()
                })
                .collect()
        })
        .collect();

    vector_match.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md011() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1

(Incorrect link one)[https://www.example.com/]

(Incorrect link two)[https://www.example.com/]

"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(2, 1, 6, 2, 1, 52)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(4, 1, 54, 4, 1, 100)))
                    .build()
            ],
            md011_reversed_link_syntax(&file)
        );
    }
}
