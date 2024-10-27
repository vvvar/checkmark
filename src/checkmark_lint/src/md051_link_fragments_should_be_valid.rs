use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{Heading, Link, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD051")
        .message("Link fragments should be valid")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md051.md")
        .rationale("GitHub section links are created automatically for every heading when Markdown content is displayed on GitHub. This makes it easy to link directly to different sections within a document. However, section links change if headings are renamed or removed. This rule helps identify broken section links within a document.\n\nSection links are not part of the CommonMark specification. This rule enforces the GitHub heading algorithm which is: convert heading to lowercase, remove punctuation, convert spaces to dashes, append an incrementing integer as needed for uniqueness")
        .push_additional_link("https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#section-links")
        .push_additional_link("https://github.com/gjtorikian/html-pipeline/blob/f13a1534cb650ba17af400d1acd3a22c28004c09/lib/html/pipeline/toc_filter.rb")
        .push_fix("Add missing anchor")
}

/// Get all Markdown links that points to a document fragment.
/// For example: [About](#about-us)
fn extract_links_with_fragments(ast: &Node) -> Vec<&Link> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_link(n))
        .filter(|l| l.url.starts_with('#'))
        .collect::<Vec<&Link>>()
}

fn extract_headings(ast: &Node) -> Vec<&Heading> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .collect::<Vec<&Heading>>()
}

/// Takes heading and returns fragment link of it.
/// Link element that want to jump to this header
/// should use this fragment.
fn heading_to_fragment(heading: &Heading) -> String {
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
            .replace([',', '.', '+', '&'], "")
            .replace(' ', "-")
    );
    text
}

/// Get all HTML links(<a/>).
/// At least one of them shall contain an anchor.
fn extract_html_elements(ast: &Node) -> Vec<scraper::Node> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_html(n))
        .flat_map(|html| {
            let fragment = scraper::Html::parse_fragment(&html.value);
            fragment.tree.clone().into_iter().collect::<Vec<_>>()
        })
        .collect::<Vec<scraper::Node>>()
}

/// Takes a list of links with fragment and for each of them tries to find whether it:
///   - has corresponding heading that is convertible to the same fragment
///   - has any HTML el with "id" attr or <a> el with "name" attribute that is == to the same fragment
///
/// For every link that does not satisfy any of these conditions, returns a violation.
fn find_violations(
    links: &[&Link],
    headings: &[&Heading],
    html_els: &[scraper::Node],
) -> Vec<Violation> {
    // Does link fragment point to a header?
    let does_fragment_points_to_header = |anchor: &Link| {
        headings
            .iter()
            .any(|heading| anchor.url.eq(&heading_to_fragment(heading)))
    };
    // Does anchor points to any other anchor in HTML <a id="#anchor"/>?
    let does_fragment_points_to_html = |link: &Link| {
        html_els.iter().any(|html_el| {
            if let Some(el) = html_el.as_element() {
                let id = el.attr("id").unwrap_or("");
                if el.name().eq("a") {
                    let name = el.attr("name").unwrap_or("");
                    link.url.eq(&format!("#{}", &id)) || link.url.eq(&format!("#{}", &name))
                } else {
                    link.url.eq(&format!("#{}", &id))
                }
            } else {
                false
            }
        })
    };
    let violations = links
        .iter()
        .filter(|link| !does_fragment_points_to_header(link) && !does_fragment_points_to_html(link))
        .map(|link| violation_builder().position(&link.position).build())
        .collect();
    log::debug!("[MD051] Violations: {:#?}", &violations);
    violations
}

pub fn md051_link_fragments_should_be_valid(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD051] File: {:#?}", &file.path);
    let ast = common::ast::parse(&file.content).unwrap();
    let links = extract_links_with_fragments(&ast);
    let headings = extract_headings(&ast);
    let html_elements = extract_html_elements(&ast);
    find_violations(&links, &headings, &html_elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    fn lint(content: &str) -> Vec<Violation> {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: content.to_string(),
            issues: vec![],
        };
        md051_link_fragments_should_be_valid(&file)
    }

    #[test]
    fn md051() {
        // Has invalid fragment
        assert_eq!(
            lint("# Heading Name\n\n[Link](#fragment)"),
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 16, 3, 18, 33)))
                .build()]
        );

        // Valid fragment from heading name
        assert_eq!(lint("# Section\n\n[Song](#section)"), vec![]);

        // More complex case when heading name is transformed
        assert_eq!(lint("# Seek & Destroy\n\n[Song](#seek--destroy)"), vec![]);

        // Random HTML el can be used as a fragment
        assert_eq!(
            lint("[Link](#anywhere)\n\n<span id='anywhere'>Hello<\\span>"),
            vec![]
        );

        // <a> tag with name attr can be used as a fragment
        assert_eq!(
            lint("[Link](#anywhere)\n\n<a name='anywhere'>Hello<\\a>"),
            vec![]
        );

        // <a> tag with id attr can be used as a fragment
        assert_eq!(
            lint("[Link](#anywhere)\n\n<a id='anywhere'>Hello<\\a>"),
            vec![]
        );

        // "+" symbol considered
        assert_eq!(lint("# C++ and C code\n\n[Code](#c-and-c-code)"), vec![]);
    }
}
