use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{Heading, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD024")
        .message("Multiple headings with the same content")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md024.md")
        .rationale("Some Markdown parsers generate anchors for headings based on the heading name; headings with the same content can cause problems with that")
        .push_fix("Ensure that the content of each heading is different")
}

fn to_text(h: &Heading, source: &str) -> String {
    let offset_start = h.position.as_ref().unwrap().start.offset;
    let offset_end = h.position.as_ref().unwrap().end.offset;
    let heading = source
        .get(offset_start..offset_end)
        .unwrap_or("")
        .trim_start_matches(' ')
        .trim_start_matches('>')
        .trim_start_matches('#');
    heading.to_string()
}

fn is_in(lhs: &Heading, slice: &[&Heading], source: &str) -> bool {
    slice
        .iter()
        .any(|rhs| to_text(lhs, source).eq(&to_text(rhs, source)))
}

pub fn md024_multiple_headings_with_the_same_content(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD024] File: {:#?}", &file.path);

    let ast = common::ast::parse(&file.content).unwrap();

    // Get all block quotes
    let mut headings: Vec<&Heading> = vec![];
    common::ast::for_each(&ast, |node| {
        if let Node::Heading(h) = node {
            headings.push(h);
        }
    });
    log::debug!("[MD024] Headings: {:#?}", &headings);

    headings
        .iter()
        .enumerate()
        .filter(|(i, h)| is_in(h, &headings[0..*i], &file.content))
        .map(|(_, h)| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md024() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# Some text\n\n## Some text".to_string(),
            issues: vec![],
        };
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 13, 3, 13, 25)))
                .build(),],
            md024_multiple_headings_with_the_same_content(&file)
        );
    }
}
