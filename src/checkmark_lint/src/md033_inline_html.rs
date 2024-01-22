use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Html, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD033")
        .message("Non-whitelisted inline HTML used")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md033.md")
        .rationale("Raw HTML is allowed in Markdown, but this rule is included for those who want their documents to only include 'pure' Markdown, or for those who are rendering Markdown documents into something other than HTML")
}

pub fn md033_inline_html(file: &MarkDownFile, allowed_tags: &Vec<String>) -> Vec<Violation> {
    log::debug!(
        "[MD033] File: {:#?}, Allowed tags: {:#?}",
        &file.path,
        &allowed_tags
    );

    let ast = parse(&file.content).unwrap();
    let mut html_nodes: Vec<&Html> = vec![];
    for_each(&ast, |node| {
        if let Node::Html(t) = node {
            html_nodes.push(t);
        }
    });
    log::debug!("[MD033] HTML nodes: {:#?}", &html_nodes);

    let violations = html_nodes
        .iter()
        .filter(|node| {
            // Markdown parser parses closing tags(e.x. "</a>")
            // as separate nodes. We need to filter them out.
            let closing_tag_error =
                std::borrow::Cow::Owned("Found special tag while closing generic tag".to_string());
            let is_closing_tag = scraper::Html::parse_fragment(&node.value)
                .errors
                .contains(&closing_tag_error);
            !is_closing_tag
        })
        .filter(|node| {
            // Parse HTML into the node tree
            // and check if all elements in it are allowed
            !scraper::Html::parse_fragment(&node.value)
                .tree
                .into_iter()
                .all(|node| {
                    if let Some(el) = node.as_element() {
                        allowed_tags.contains(&el.name().to_string())
                            || el.name().eq("body")
                            || el.name().eq("html")
                    } else {
                        true
                    }
                })
        })
        .map(|node| {
            violation_builder()
                .position(&node.position)
                .push_fix("If your intention was show this HTML tag as a text, consider escaping it with \"\\\". For example: \"\\<br\\>\"")
                .push_fix("If it's not the case, consider using Markdown instead of HTML")
                .push_fix("If this HTML tag is needed, then consider adding a name of this element to the list of allowed tags. Use \"allowed_html_tags\" option from the \"[lint]\" section in the config file")
                .build()
        })
        .collect();
    log::debug!("[MD033] Violations: {:#?}", &violations);

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md003() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: r#"<h1 align="center">Header<\h1>\n\n"#.to_string(),
            issues: vec![],
        };

        // Plain case
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(1, 1, 0, 1, 35, 34)))
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
                    .position(&Some(Position::new(1, 1, 0, 1, 18, 17)))
                    .build(),
                violation_builder()
                    .position(&Some(Position::new(5, 1, 35, 5, 18, 52)))
                    .build(),
            ],
            md033_inline_html(
                &MarkDownFile {
                    path: String::from("this/is/a/dummy/path/to/a/file.md"),
                    content: "<h1>Header 1<\\h1>\n\nSome paragraph\n\n<h1>Header 2<\\h1>"
                        .to_string(),
                    issues: vec![],
                },
                &vec![]
            ),
        );

        // HTML code block can be completely ignored
        assert_eq!(
            Vec::<Violation>::new(),
            md033_inline_html(
                &common::MarkDownFile {
                    path: String::from("this/is/a/dummy/path/to/a/file.md"),
                    content: "<div><h1>Header 1<\\h1><img/></div>".to_string(),
                    issues: vec![],
                },
                &vec!["div".to_string(), "h1".to_string(), "img".to_string()]
            ),
        );

        // HTML code block is not ignored if it contains at least one non-whitelisted element
        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 35, 34)))
                .build(),],
            md033_inline_html(
                &common::MarkDownFile {
                    path: String::from("this/is/a/dummy/path/to/a/file.md"),
                    content: "<div><h1>Header 1<\\h1><img/></div>".to_string(),
                    issues: vec![],
                },
                &vec!["div".to_string(), "h1".to_string()]
            ),
        );
    }
}
