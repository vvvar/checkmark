use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Heading, Node};
use std::collections::VecDeque;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD025")
        .message("Multiple top-level headings in the same document")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md025.md")
        .rationale("A top-level heading is an h1 on the first line of the file, and serves as the title for the document. If this convention is in use, then there can not be more than one title for the document, and the entire document should be contained within this heading")
        .push_fix("Structure your document so there is a single h1 heading that is the title for the document. Subsequent headings must be lower-level headings (h2, h3, etc.)")
}

pub fn md025_multiple_top_level_headings(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD025] File: {:#?}", &file.path);

    let ast = parse(&file.content).unwrap();

    let mut h1_headings = VecDeque::<&Heading>::new();
    for_each(&ast, |node| {
        if let Node::Heading(h) = node {
            if h.depth == 1 {
                h1_headings.push_back(h);
            }
        }
    });
    log::debug!("[MD025] H1 Headings: {:#?}", &h1_headings);
    // First element is legit
    h1_headings.pop_front();
    // Everything else - violation
    h1_headings
        .iter()
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md025() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# Top level heading\n\n# Another top-level heading".to_string(),
            issues: vec![],
        };
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 21, 3, 28, 48)))
                .build(),],
            md025_multiple_top_level_headings(&file)
        );
    }
}
