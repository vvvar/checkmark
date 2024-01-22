use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Heading, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD022")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md022.md")
        .rationale("side from aesthetic reasons, some parsers, including kramdown, will not parse headings that don't have a blank line before, and will parse them as regular text")
        .is_fmt_fixable(true)
}

fn surrounded_by_blank_lines(line: &usize, h: &Heading, source: &str) -> bool {
    let h_line_end = h.position.as_ref().unwrap().end.line;
    // nth == end line of header because nth is 0-indexed while lines are not
    let text_after_heading = source.lines().nth(h_line_end).unwrap_or("");
    // When it is a first heading in document.
    if line.eq(&0) {
        // Only check if there is a blank line after
        text_after_heading.is_empty()
    } else {
        // Otherwise, check both before and after
        // nth == start line of header because nth is 0-indexed while lines are not
        let h_line_start = h.position.as_ref().unwrap().start.line;
        let text_before_heading = source.lines().nth(h_line_start - 2).unwrap_or("");
        text_before_heading.is_empty() && text_after_heading.is_empty()
    }
}

fn to_violation(i: usize, h: &Heading) -> Violation {
    let mut violation = violation_builder().position(&h.position);
    if i.eq(&0) {
        violation = violation
            .message("Heading is not followed by blank line")
            .push_fix("Add a blank line after the the header");
    } else {
        violation = violation
            .message("Heading is not surrounded by blank lines")
            .push_fix("Ensure there is a blank line before and after the header");
    }
    violation.build()
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
        .enumerate()
        .filter(|(i, h)| !surrounded_by_blank_lines(i, h, &file.content))
        .map(|(i, h)| to_violation(i, h))
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    fn markdown_file(content: &str) -> MarkDownFile {
        MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: content.to_string(),
            issues: vec![],
        }
    }

    #[test]
    fn md022() {
        let headings_in_order = markdown_file(
            "# Heading 1\nSome text\n\nSome more text\n\n  ## Heading 2\n\nSome text\n## Heading 3",
        );
        assert_eq!(
            vec![
                violation_builder()
                    .message("Heading is not followed by blank line")
                    .position(&Some(Position::new(1, 1, 0, 1, 12, 11)))
                    .build(),
                violation_builder()
                    .message("Heading is not surrounded by blank lines")
                    .position(&Some(Position::new(9, 1, 65, 9, 13, 77)))
                    .build(),
            ],
            md022_headings_should_be_surrounded_by_blank_lines(&headings_in_order)
        );

        let headings_not_in_order = markdown_file("## H2\n\n# H1\n");
        assert_eq!(
            Vec::<Violation>::new(),
            md022_headings_should_be_surrounded_by_blank_lines(&headings_not_in_order)
        );
    }
}
