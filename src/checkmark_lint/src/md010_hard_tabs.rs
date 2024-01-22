use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD010")
        .message("Use of hard tabs")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md010.md")
        .rationale("Hard tabs are often rendered inconsistently by different editors and can be harder to work with than spaces")
        .push_fix("Remove hard tabs")
        .is_fmt_fixable(true)
}

pub fn md010_hard_tabs(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD010] File: {:#?}", &file.path);
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("\t"))
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
    pub fn md010() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1\t
\t\t\t\t
## H2\t"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(0, 1, 0, 0, 5, 5)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(1, 1, 6, 1, 4, 10)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(2, 1, 11, 2, 6, 17)))
                    .build()
            ],
            md010_hard_tabs(&file)
        );
    }
}
