use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement = "Heading content should be unique",
    rationale = "Some Markdown parsers generate anchors for headings based on the heading name; headings with the same content can cause problems with that",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md024.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md024(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    let mut headings_content = std::collections::HashSet::new();
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| {
            let text = to_text(h, &file.content);
            if headings_content.contains(&text) {
                true
            } else {
                headings_content.insert(text);
                false
            }
        })
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Multiple headings with the same content")
        .assertion("Expected heading to have unique content, got a duplicate")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# Some text\n\n## Some text\n\n## Some another text")]
    fn detects_hedings_with_same_content(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 13, 3, 13, 25)))
                .build(),],
            MD024.check(ast, file, config)
        );
    }
}
