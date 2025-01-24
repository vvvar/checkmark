use checkmark_lint_common::*;
use checkmark_lint_macro::*;
use common::ast::{try_cast_to_heading, try_cast_to_html, try_cast_to_link, BfsIterator};

use scraper::{Html, Node as HtmlNode};

#[rule(
    requirement = "Link fragments should be valid",
    rationale = "GitHub section links are created automatically for every heading when Markdown content is displayed on GitHub. This makes it easy to link directly to different sections within a document. However, section links change if headings are renamed or removed. This rule helps identify broken section links within a document.\n\nSection links are not part of the CommonMark specification. This rule enforces the GitHub heading algorithm which is: convert heading to lowercase, remove punctuation, convert spaces to dashes, append an incrementing integer as needed for uniqueness",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md051.md",
    additional_links = [
        "https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#section-links",
        "https://github.com/gjtorikian/html-pipeline/blob/f13a1534cb650ba17af400d1acd3a22c28004c09/lib/html/pipeline/toc_filter.rb"
    ],
    is_fmt_fixable = false,
)]
fn md051(ast: &Node, _: &MarkDownFile, _: &Config) -> Vec<Violation> {
    let links = extract_links_with_fragments(ast);
    let headings = extract_headings(ast);
    let html_elements = extract_html_elements(ast);
    find_violations(&links, &headings, &html_elements)
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Invalid link fragments")
        .assertion("Expected link fragment to reference an existing heading's generated name, got link fragment that matches to none")
        .push_fix("Add missing anchor")
}

/// Get all Markdown links that points to a document fragment.
/// For example: [About](#about-us)
fn extract_links_with_fragments(ast: &Node) -> Vec<&Link> {
    BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_link(n))
        .filter(|l| l.url.starts_with('#'))
        .collect::<Vec<&Link>>()
}

fn extract_headings(ast: &Node) -> Vec<&Heading> {
    BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_heading(n))
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
fn extract_html_elements(ast: &Node) -> Vec<HtmlNode> {
    BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_html(n))
        .flat_map(|html| {
            let fragment = Html::parse_fragment(&html.value);
            fragment.tree.clone().into_iter().collect::<Vec<_>>()
        })
        .collect::<Vec<HtmlNode>>()
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
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# Heading Name\n\n[Link](#fragment)")]
    fn detect_invalid_fragment(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 16, 3, 18, 33)))
                .build()],
            MD051.check(ast, file, config)
        );
    }

    #[rule_test(markdown = "# Section\n\n[Song](#section)")]
    fn do_not_complain_about_valid_fragment_from_heading_name(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(Vec::<Violation>::new(), MD051.check(ast, file, config));
    }

    #[rule_test(markdown = "# Seek & Destroy\n\n[Song](#seek--destroy)")]
    fn handle_more_complex_case_when_heading_name_is_transformed(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(Vec::<Violation>::new(), MD051.check(ast, file, config));
    }

    #[rule_test(markdown = "[Link](#anywhere)\n\n<span id='anywhere'>Hello<\\span>")]
    fn handle_random_html_element_as_a_fragment(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(Vec::<Violation>::new(), MD051.check(ast, file, config));
    }

    #[rule_test(markdown = "[Link](#anywhere)\n\n<a name='anywhere'>Hello<\\a>")]
    fn do_nor_complain_when_a_tag_with_name_attr_is_used_as_fragment(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(Vec::<Violation>::new(), MD051.check(ast, file, config));
    }

    #[rule_test(markdown = "[Link](#anywhere)\n\n<a id='anywhere'>Hello<\\a>")]
    fn do_nor_complain_when_a_tag_with_id_attr_is_used_as_fragment(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(Vec::<Violation>::new(), MD051.check(ast, file, config));
    }

    #[rule_test(markdown = "# C++ and C code\n\n[Code](#c-and-c-code)")]
    fn consider_plus_symbol(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(Vec::<Violation>::new(), MD051.check(ast, file, config));
    }
}
