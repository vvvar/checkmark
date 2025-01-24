use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement = "Heading should not have a trailing punctuation",
    rationale = "Headings are not meant to be full sentences",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md026.md",
    additional_links = ["https://cirosantilli.com/markdown-style-guide/#punctuation-at-the-end-of-headers"],
    is_fmt_fixable = false,
)]
fn md026(ast: &Node, _: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| ends_with_trailing_punctuation(h))
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Found trailing punctuation in the heading")
        .assertion("Expected heading to end without trailing punctuation, got one")
        .push_fix("Remove trailing punctuation")
}

// Returns true when the heading text ends with "."
fn ends_with_trailing_punctuation(h: &Heading) -> bool {
    let mut buffer: String = String::new();
    let mut stack: Vec<&Node> = vec![];
    for child in &h.children {
        stack.push(child);
    }
    while let Some(current) = stack.pop() {
        if let Node::Text(t) = current {
            buffer.push_str(&t.value);
        }
        if let Some(children) = current.children() {
            for child in children.iter().rev() {
                stack.push(child);
            }
        }
    }
    buffer.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# This is a heading.\n\n## This is fine\n")]
    fn detects_trailing_punctuation_in_heading(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(1, 1, 0, 1, 21, 20)))
                .build(),],
            MD026.check(ast, file, config)
        );
    }
}
