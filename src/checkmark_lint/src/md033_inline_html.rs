use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, MarkDownFile};
use markdown::{
    mdast::{self},
    to_mdast, ParseOptions,
};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD033")
        .message("Non-whitelisted inline HTML used")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md033.md")
}

pub fn md033_inline_html(file: &MarkDownFile, allowed_elements: &Vec<String>) -> Vec<Violation> {
    log::debug!(
        "[MD033] File: {:#?}, Allowed elements: {:#?}",
        &file.path,
        &allowed_elements
    );

    let ast = to_mdast(&file.content, &ParseOptions::gfm()).unwrap();
    let mut html_nodes: Vec<&mdast::Html> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::Html(t) = node {
            html_nodes.push(t);
        }
    });
    log::debug!("[MD033] HTML nodes: {:#?}", &html_nodes);

    let violations = html_nodes
        .iter()
        .filter(|node| {
            let mut is_allowed = false;
            for allowed_element in allowed_elements {
                if node.value.contains(&format!("<{}", allowed_element)) {
                    is_allowed = true;
                    break;
                }
            }
            !is_allowed
        })
        .map(|node| {
            violation_builder()
                .position(&node.position)
                .push_fix("Replace inline HTML with Markdown")
                .push_fix("Remove inline HTML")
                .build()
        })
        .collect();
    log::debug!("[MD033] Violations: {:#?}", &violations);

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md003() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: r#"<h1 align="center">Header<\h1>\n\n"#.to_string(),
            issues: vec![],
        };

        // Plain case
        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 35, 34)))
                .build()],
            md033_inline_html(&file, &vec![]),
        );

        // White-listing tags
        assert_eq!(
            Vec::<Violation>::new(),
            md033_inline_html(&file, &vec!["h1".to_string()]),
        );

        // Several elements reported
        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 18, 17)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(5, 1, 35, 5, 18, 52)))
                    .build(),
            ],
            md033_inline_html(
                &common::MarkDownFile {
                    path: String::from("this/is/a/dummy/path/to/a/file.md"),
                    content: "<h1>Header 1<\\h1>\n\nSome paragraph\n\n<h1>Header 2<\\h1>"
                        .to_string(),
                    issues: vec![],
                },
                &vec![]
            ),
        );
    }
}
