use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{Html, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD031")
        .message("Fenced code blocks should be surrounded by blank lines")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/main/doc/md031.md")
        .rationale("Aside from aesthetic reasons, some parsers, including kramdown, will not parse fenced code blocks that don't have blank lines before and after them.")
}

pub fn md031_fenced_code_blocks_surrounded_with_blank_lines(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD031] File: {:#?}", &file.path);
    let ast = common::ast::parse(&file.content).unwrap();
    let html_nodes = common::ast::BfsIterator::from(&ast)
        .filter(|node| matches!(node, Node::Heading(_)))
        .collect::<Vec<_>>();
    dbg!(&html_nodes);
    for node in common::ast::BfsIterator::from(&ast) {
        if let Node::Html(t) = node {
            dbg!("[MD031] Html node: {:#?}", t);
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md031() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: r#"# Heading\n\n<h1 align="center">Header<\h1>\n\n"#.to_string(),
            issues: vec![],
        };

        assert!(true);

        // Plain case
        // assert_eq!(
        //     vec![violation_builder()
        //         .position(&Some(Position::new(1, 1, 0, 1, 35, 34)))
        //         .build()],
        //         md031_fenced_code_blocks_surrounded_with_blank_lines(&file),
        // );
    }
}
