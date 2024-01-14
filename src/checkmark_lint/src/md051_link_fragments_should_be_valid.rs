use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Heading, Html, Link, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD051")
        .message("Link fragments should be valid")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md051.md")
        .push_fix("Add missing anchor")
}

/// Get all Markdown links.
/// All of them shall point to valid anchors.
fn extract_anchor_links(ast: &Node) -> Vec<&Link> {
    let mut link_nodes: Vec<&Link> = vec![];
    for_each(&ast, |node| {
        if let Node::Link(l) = node {
            if l.url.starts_with("#") {
                link_nodes.push(l);
            }
        }
    });
    log::debug!("[MD051] Link nodes: {:#?}", &link_nodes);
    link_nodes
}

/// Get all headings.
/// At lease one of them shall be referenced by a link.
fn extract_headings(ast: &Node) -> Vec<&Heading> {
    let mut heading_nodes: Vec<&Heading> = vec![];
    for_each(&ast, |node| {
        if let Node::Heading(h) = node {
            heading_nodes.push(h);
        }
    });
    log::debug!("[MD051] Heading nodes: {:#?}", &heading_nodes);
    heading_nodes
}

/// Takes heading and returns anchor link to it.
/// Link element that want to jump to this header
/// should have this anchor.
fn heading_to_anchor(heading: &Heading) -> String {
    let mut text = "".to_string();
    for node in &heading.children {
        if let Node::Text(t) = node {
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
}

/// Get all HTML links(<a/>).
/// At least one of them shall contain an anchor.
fn extract_html_links(ast: &Node) -> Vec<&Html> {
    let mut html_links: Vec<&Html> = vec![];
    for_each(&ast, |node| {
        if let Node::Html(h) = node {
            let fragment = scraper::Html::parse_fragment(&h.value);
            let a_selector = scraper::Selector::parse("a").unwrap();
            if fragment.select(&a_selector).next().is_some() {
                html_links.push(h);
            }
        }
    });
    log::debug!("[MD051] HTML links: {:#?}", &html_links);
    html_links
}

fn find_violations(
    anchor_links: &Vec<&Link>,
    headings: &Vec<&Heading>,
    html_links: &Vec<&Html>,
) -> Vec<Violation> {
    // Does anchor link point to a heading?
    let does_anchor_points_to_header = |anchor: &Link| {
        headings
            .iter()
            .any(|heading| anchor.url.eq(&heading_to_anchor(&heading)))
    };
    // Does anchor points to any other anchor in HTML <a id="#anchor"/>?
    let does_anchor_points_to_link = |anchor: &Link| {
        html_links.iter().any(|html_link| {
            let fragment = scraper::Html::parse_fragment(&html_link.value);
            let a_selector = scraper::Selector::parse("a").unwrap();
            if let Some(a) = fragment.select(&a_selector).next() {
                let id = a.value().attr("id").unwrap_or("");
                anchor.url.eq(&format!("#{}", &id))
            } else {
                false
            }
        })
    };
    let link_to_violation = |link: &Link| violation_builder().position(&link.position).build();
    let violations = anchor_links
        .iter()
        .filter(|link| !does_anchor_points_to_header(&link) && !does_anchor_points_to_link(&link))
        .map(|link| link_to_violation(&link))
        .collect();
    log::debug!("[MD051] Violations: {:#?}", &violations);
    violations
}

pub fn md051_link_fragments_should_be_valid(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD051] File: {:#?}", &file.path);
    let ast = parse(&file.content).unwrap();
    let anchor_links = extract_anchor_links(&ast);
    let headings = extract_headings(&ast);
    let html_links = extract_html_links(&ast);
    find_violations(&anchor_links, &headings, &html_links)
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md051() {
        let file = MarkDownFile {
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
                .position(&Some(Position::new(5, 3, 34, 5, 16, 47)))
                .build()],
            md051_link_fragments_should_be_valid(&file)
        );
    }
}
