use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, MarkDownFile};
use markdown::{
    mdast::{self},
    to_mdast, ParseOptions,
};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD051")
        .message("Link fragments should be valid")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md051.md")
        .push_fix("Add missing anchor")
}

pub fn md051_link_fragments_should_be_valid(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD051] File: {:#?}", &file.path);

    let ast = to_mdast(&file.content, &ParseOptions::gfm()).unwrap();

    // Get all Markdown links. All of them shall point to valid anchors.
    let mut link_nodes: Vec<&mdast::Link> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::Link(l) = node {
            if l.url.starts_with("#") {
                link_nodes.push(l);
            }
        }
    });
    log::debug!("[MD051] Link nodes: {:#?}", &link_nodes);

    // Get all headings. At lease one of them shall be referenced by a link.
    let mut heading_nodes: Vec<&mdast::Heading> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::Heading(h) = node {
            heading_nodes.push(h);
        }
    });
    log::debug!("[MD051] Heading nodes: {:#?}", &heading_nodes);

    // Get all HTML links(<a/>). At least one of them shall contain an anchor.
    let mut html_links: Vec<&mdast::Html> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::Html(h) = node {
            let fragment = scraper::Html::parse_fragment(&h.value);
            let a_selector = scraper::Selector::parse("a").unwrap();
            if fragment.select(&a_selector).next().is_some() {
                html_links.push(h);
            }
        }
    });
    log::debug!("[MD051] HTML links: {:#?}", &heading_nodes);

    let heading_to_anchor = |heading: &mdast::Heading| -> String {
        let mut text = "".to_string();
        for node in &heading.children {
            if let mdast::Node::Text(t) = node {
                text = format!("{}{}", &text, &t.value);
            }
        }
        text = format!(
            "#{}",
            &text
                .to_lowercase()
                .replace(",", "")
                .replace(".", "")
                .replace("&", "")
                .replace(" ", "-")
        );
        text
    };

    let violations = link_nodes
        .iter()
        .filter(|link| {
            // Does this link point to a heading?
            !heading_nodes
                .iter()
                .any(|heading| link.url.eq(&heading_to_anchor(&heading))) &&
            // Or to any anchor in HTML <a id="#anchor"/>?
            !html_links.iter()
                .any(|html_link| {
                    let fragment = scraper::Html::parse_fragment(&html_link.value);
                    let a_selector = scraper::Selector::parse("a").unwrap();
                    if let Some(a) = fragment.select(&a_selector).next() {
                        let id = a.value().attr("id").unwrap_or("");
                        link.url.eq(&format!("#{}", &id))
                    } else {
                        false
                    }
                })
        })
        .map(|node| violation_builder().position(&node.position).build())
        .collect();
    log::debug!("[MD051] Violations: {:#?}", &violations);

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md051() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "
# Title

- [About](#about--us)
- [Help](#help)
- [Contribute](#contribute)
- [Normal Link](https://google.com)

## About & Us

This is about

## Not-help

This is not help

## <a id='contribute' /> But this is a contribution

This is a contribution

## <a href='#not-contribute' /> And this is a something else

Something else text with <a>random HTML</a>"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(5, 3, 34, 5, 16, 47)))
                .build()],
            md051_link_fragments_should_be_valid(&file)
        );
    }
}
