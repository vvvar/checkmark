use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Heading, Node};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD023")
        .message("Headings must start at the beginning of the line")
        .push_fix("Ensure that all headings start at the beginning of the line(heading inside block quote is an exception)")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md023.md")
        .is_fmt_fixable(true)
}

/// Check is heading matches on these patters:
/// "    # Heading" or ">   # Heading"
fn heading_is_indented(h: &Heading, source: &str) -> bool {
    let line = h.position.as_ref().unwrap().start.line;
    let heading = source.lines().nth(line - 1).unwrap_or("");
    // Pattern #1: String starts with one or more whitespace followed by one or more hash characters
    // Pattern #2: String starts with block quote followed by one space followed by one or more another whitespace
    //             and followed by one or more hash characters
    Regex::new(r"^\s+#+").unwrap().is_match(heading)
        || Regex::new(r"^>\s\s++#+").unwrap().is_match(heading)
}

pub fn md023_headings_must_start_at_the_beginning_of_the_line(
    file: &MarkDownFile,
) -> Vec<Violation> {
    log::debug!("[MD023] File: {:#?}", &file.path);

    let ast = parse(&file.content).unwrap();

    // Get all block quotes
    let mut headings: Vec<&Heading> = vec![];
    for_each(&ast, |node| {
        if let Node::Heading(h) = node {
            headings.push(h);
        }
    });
    log::debug!("[MD023] Headings: {:#?}", &headings);

    headings
        .iter()
        .filter(|h| heading_is_indented(h, &file.content))
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md023() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "Some text

  # Indented heading

>  # Indented heading"
                .to_string(),
            issues: vec![],
        };
        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(Position::new(3, 1, 11, 3, 21, 31)))
                    .build(),
                violation_builder()
                    .position(&Some(Position::new(5, 3, 35, 5, 22, 54)))
                    .build(),
            ],
            md023_headings_must_start_at_the_beginning_of_the_line(&file)
        );
    }
}
