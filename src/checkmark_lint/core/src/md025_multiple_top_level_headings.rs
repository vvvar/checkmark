use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement = "Document should have a single top-level heading",
    rationale = "A top-level heading is an h1 on the first line of the file, and serves as the title for the document. If this convention is in use, then there can not be more than one title for the document, and the entire document should be contained within this heading",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md025.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md025(ast: &Node, _: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| h.depth == 1) // We only need 1st level headings
        .skip(1) // First heading is always legit
        .map(|h| violation_builder().position(&h.position).build()) // Others are violations
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Found multiple top-level headings")
        .assertion("Expected single top-level heading, got several")
        .push_fix("Structure your document so there is a single h1 heading that is the title for the document. Subsequent headings must be lower-level headings (h2, h3, etc.)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(
        markdown = "# Top level heading\n\n# Another top-level heading\n\n## Legit heading"
    )]
    fn detect_multiple_top_level_headings(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(3, 1, 21, 3, 28, 48)))
                .build(),],
            MD025.check(ast, file, config)
        );
    }
}
