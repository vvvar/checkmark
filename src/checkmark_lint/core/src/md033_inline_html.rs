use checkmark_lint_common::*;
use checkmark_lint_macro::*;
use common::ast::{try_cast_to_html, BfsIterator};

use std::borrow::Cow::Owned;

use scraper::Html;

#[rule(
    requirement = "Only whitelisted HTML should be used",
    rationale = "Raw HTML is allowed in Markdown, but this rule is included for those who want their documents to only include 'pure' Markdown, or for those who are rendering Markdown documents into something other than HTML",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md033.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md033(ast: &Node, _: &MarkDownFile, config: &Config) -> Vec<Violation> {
    let allowed_tags = &config.linter.md033_allowed_html_tags;
    BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_html(n))
        .filter(|node| {
            // Markdown parser parses closing tags(e.x. "</a>")
            // as separate nodes. We need to filter them out.
            let closing_tag_error = Owned("Found special tag while closing generic tag".to_string());
            let is_closing_tag = Html::parse_fragment(&node.value)
                .errors
                .contains(&closing_tag_error);
            !is_closing_tag
        })
        .filter(|node| {
            // Parse HTML into the node tree
            // and check if all elements in it are allowed
            !Html::parse_fragment(&node.value)
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
            let assertion = if allowed_tags.is_empty() {
                "Expected no inline HTML, got some"
            } else {
                &format!("Expected no inline HTML except [{}], got some", allowed_tags.join(", "))
            };
            violation_builder()
                .position(&node.position)
                .assertion(assertion)
                .push_fix("If your intention was show this HTML tag as a text, consider escaping it with \"\\\". For example: \"\\<br\\>\"")
                .push_fix("If it's not the case, consider using Markdown instead of HTML")
                .push_fix("If this HTML tag is needed, then consider adding a name of this element to the list of allowed tags. Use \"allowed_html_tags\" option from the \"[lint]\" section in the config file")
                .build()
        })
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default().message("Non-whitelisted inline HTML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = r#"<h1 align="center">Header<\h1>\n\n"#)]
    fn respect_white_lsit(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        // Plain case.
        assert_eq!(
            vec![violation_builder()
                .assertion("Expected no inline HTML, got some")
                .set_fixes(vec![
                    String::from("If your intention was show this HTML tag as a text, consider escaping it with \"\\\". For example: \"\\<br\\>\""),
                    String::from("If it's not the case, consider using Markdown instead of HTML"),
                    String::from("If this HTML tag is needed, then consider adding a name of this element to the list of allowed tags. Use \"allowed_html_tags\" option from the \"[lint]\" section in the config file"),
                ])
                .position(&Some(Position::new(1, 1, 0, 1, 35, 34)))
                .build()],
            MD033.check(ast, file, config)
        );

        // White-listing tags.
        config.linter.md033_allowed_html_tags = vec![String::from("h1")];
        assert_eq!(Vec::<Violation>::new(), MD033.check(ast, file, config),);
    }

    #[rule_test(markdown = "<h1>Header 1<\\h1>\n\nSome paragraph\n\n<h1>Header 2<\\h1>")]
    fn report_multiple_elements(ast: &Node, file: &MarkDownFile, config: &Config) {
        // Several elements reported.
        assert_eq!(
            vec![
                violation_builder()
                    .assertion("Expected no inline HTML, got some")
                    .set_fixes(vec![
                        String::from("If your intention was show this HTML tag as a text, consider escaping it with \"\\\". For example: \"\\<br\\>\""),
                        String::from("If it's not the case, consider using Markdown instead of HTML"),
                        String::from("If this HTML tag is needed, then consider adding a name of this element to the list of allowed tags. Use \"allowed_html_tags\" option from the \"[lint]\" section in the config file"),
                    ])
                    .position(&Some(Position::new(1, 1, 0, 1, 18, 17)))
                    .build(),
                violation_builder()
                    .assertion("Expected no inline HTML, got some")
                    .set_fixes(vec![
                        String::from("If your intention was show this HTML tag as a text, consider escaping it with \"\\\". For example: \"\\<br\\>\""),
                        String::from("If it's not the case, consider using Markdown instead of HTML"),
                        String::from("If this HTML tag is needed, then consider adding a name of this element to the list of allowed tags. Use \"allowed_html_tags\" option from the \"[lint]\" section in the config file"),
                    ])
                    .position(&Some(Position::new(5, 1, 35, 5, 18, 52)))
                    .build(),
            ],
            MD033.check(ast, file, config)
        );
    }

    #[rule_test(markdown = "<div><h1>Header 1<\\h1><img/></div>")]
    fn can_ignore_entire_html_code_block(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        // HTML code block can be completely ignored.
        config.linter.md033_allowed_html_tags =
            vec![String::from("div"), String::from("h1"), String::from("img")];
        assert_eq!(Vec::<Violation>::new(), MD033.check(ast, file, config));
    }

    #[rule_test(markdown = "<div><h1>Header 1<\\h1><img/></div>")]
    fn detects_unclosed_elements(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        // HTML code block is not ignored if it contains at least one non-whitelisted element.
        config.linter.md033_allowed_html_tags = vec![String::from("div"), String::from("h1")];
        assert_eq!(
            vec![violation_builder()
                .assertion("Expected no inline HTML except [div, h1], got some")
                .set_fixes(vec![
                    String::from("If your intention was show this HTML tag as a text, consider escaping it with \"\\\". For example: \"\\<br\\>\""),
                    String::from("If it's not the case, consider using Markdown instead of HTML"),
                    String::from("If this HTML tag is needed, then consider adding a name of this element to the list of allowed tags. Use \"allowed_html_tags\" option from the \"[lint]\" section in the config file"),
                ])
                .position(&Some(Position::new(1, 1, 0, 1, 35, 34)))
                .build(),],
            MD033.check(ast, file, config)
        );
    }
}
