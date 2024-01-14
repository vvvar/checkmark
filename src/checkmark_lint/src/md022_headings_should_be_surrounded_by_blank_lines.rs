use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Heading, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD022")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md022.md")
        .is_fmt_fixable(true)
}

pub fn md022_headings_should_be_surrounded_by_blank_lines(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD022] File: {:#?}", &file.path);

    let ast = parse(&file.content).unwrap();

    // Get all block quotes
    let mut headings: Vec<&Heading> = vec![];
    for_each(&ast, |node| {
        if let Node::Heading(h) = node {
            headings.push(h);
        }
    });
    log::debug!("[MD022] Headings: {:#?}", &headings);

    headings
        .iter()
        .filter(|h| {
            let offset_start = h.position.as_ref().unwrap().start.offset;
            let offset_end = h.position.as_ref().unwrap().end.offset;
            let text_after_heading = file.content.get(offset_end..offset_end + 2).unwrap_or("");
            if h.depth == 1 {
                !text_after_heading.eq("\n\n")
            } else {
                let text_before_heading = if offset_start >= 1 {
                    file.content
                        .get(offset_start - 2..offset_start)
                        .unwrap_or("")
                } else {
                    file.content.get(offset_start..offset_start).unwrap_or("")
                };
                !text_before_heading.eq("\n\n") || !text_after_heading.eq("\n\n")
            }
        })
        .map(|h| {
            let mut violation = violation_builder().position(&h.position);
            if h.depth == 1 {
                violation = violation
                    .message("Heading is not followed by blank line")
                    .push_fix("Add blank line after the header");
            } else {
                violation = violation
                    .message("Heading is not surrounded by blank lines")
                    .push_fix("Add blank line before and after the header");
            }
            violation.build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md022() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1
Text directly after H1
## H2

Here all fine

## H2 - All good here as well"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .message("Heading is not followed by blank line")
                    .position(&Some(Position::new(1, 1, 0, 1, 5, 4)))
                    .build(),
                violation_builder()
                    .message("Heading is not surrounded by blank lines")
                    .position(&Some(Position::new(3, 1, 28, 3, 6, 33)))
                    .build(),
                violation_builder()
                    .message("Heading is not surrounded by blank lines")
                    .position(&Some(Position::new(7, 1, 50, 7, 30, 79)))
                    .build()
            ],
            md022_headings_should_be_surrounded_by_blank_lines(&file)
        );
    }
}
