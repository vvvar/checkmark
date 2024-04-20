use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{Heading, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD026")
        .message("Trailing punctuation in heading")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md026.md")
        .rationale("Headings are not meant to be full sentences")
        .push_additional_link(
            "https://cirosantilli.com/markdown-style-guide/#punctuation-at-the-end-of-headers",
        )
        .push_fix("Remove the trailing punctuation")
}

// Returns true when the heading text ends with "."
fn ends_with_trailing_punctuation(h: &Heading) -> bool {
    let mut buffer: String = String::new();
    let mut stack: Vec<&markdown::mdast::Node> = vec![];
    for child in &h.children {
        stack.push(child);
    }
    while let Some(current) = stack.pop() {
        if let Node::Text(t) = current {
            buffer.push_str(&t.value);
        }
        if let Some(children) = current.children() {
            for child in children.iter().rev() {
                stack.push(child);
            }
        }
    }
    buffer.ends_with('.')
}

pub fn md026_trailing_punctuation_in_heading(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD026] File: {:#?}", &file.path);
    let ast = common::ast::parse(&file.content).unwrap();
    common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| ends_with_trailing_punctuation(h))
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md026() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# This is a heading.\n\n## This is fine\n".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 21, 20)))
                .build(),],
            md026_trailing_punctuation_in_heading(&file)
        );
    }
}
