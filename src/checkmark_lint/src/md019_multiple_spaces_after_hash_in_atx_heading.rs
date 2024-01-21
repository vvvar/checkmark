use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Heading, Node};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD019")
        .message("Multiple spaces after hash on atx style heading")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md019.md")
        .push_fix("Separate the heading text from the hash character by a single space")
        .is_fmt_fixable(true)
}

// Returns true if the line starts
// with atx heading with more then
// one space
// Example: "##   this_will_return_true"
fn start_with_atx_heading_without_space(h: &Heading, source: &str) -> bool {
    let line = h.position.as_ref().unwrap().start.line;
    let text = source.lines().nth(line - 1).unwrap_or("");
    // Pattern: start of the line followed by one or more hash
    //          characters followed by single space and one or
    //          more spaces followed by non-numeric letter
    Regex::new(r"^#+\s\s+\b").unwrap().is_match(text)
}

pub fn md019_multiple_spaces_after_hash_on_atx_style_heading(
    file: &MarkDownFile,
) -> Vec<Violation> {
    log::debug!("[MD019] File: {:#?}", &file.path);

    let ast = parse(&file.content).unwrap();
    let mut headings: Vec<&Heading> = vec![];
    for_each(&ast, |node| {
        if let Node::Heading(h) = node {
            headings.push(h);
        }
    });

    headings
        .iter()
        .filter(|h| start_with_atx_heading_without_space(&h, &file.content))
        .map(|h| violation_builder().position(&h.position).build())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md019() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "#   fff".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 8, 7)))
                .build(),],
            md019_multiple_spaces_after_hash_on_atx_style_heading(&file)
        );
    }
}
