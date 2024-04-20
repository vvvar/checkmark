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

pub fn md024_multiple_headings_with_the_same_content(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD024] File: {:#?}", &file.path);
    let mut headings_content = std::collections::HashSet::new();
    let ast = common::ast::parse(&file.content).unwrap();
    common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| {
            let text = to_text(h, &file.content);
            if headings_content.contains(&text) {
                return true;
            } else {
                headings_content.insert(text);
                return false;
            }
        })
        .map(|h| violation_builder().position(&h.position).build())
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
            content: "# Some text\n\n## Some text\n\n## Some another text".to_string(),
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
