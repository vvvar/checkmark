use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD012")
        .message("Multiple consecutive blank lines")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md012.md")
        .push_fix("Remove unnecessary blank line")
        .is_fmt_fixable(true)
}

pub fn md012_multiple_blank_lines(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD012] File: {:#?}", &file.path);
    file.content
        .match_indices("\n\n\n")
        .map(|(i, _)| {
            log::debug!("[MD012] Problem offset {:#?}", i);
            let line_positions = line_numbers::LinePositions::from(file.content.as_str());
            violation_builder()
                .position(&Some(markdown::unist::Position::new(
                    line_positions.from_offset(i + 1).as_usize(),
                    1,
                    i + 1,
                    line_positions.from_offset(i + 2).as_usize(),
                    1,
                    i + 2,
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
    pub fn md012() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1


## H2"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(1, 1, 5, 2, 1, 6)))
                .build(),],
            md012_multiple_blank_lines(&file)
        );
    }
}
