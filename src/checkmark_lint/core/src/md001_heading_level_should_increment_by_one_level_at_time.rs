use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement="Headings should increment one level at a time",
    rationale="Headings represent the structure of a document and can be confusing when skipped - especially for accessibility scenarios",
    documentation="https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md001.md",
    additional_links=[],
    is_fmt_fixable=false,
)]
fn md001(ast: &Node, _: &MarkDownFile, _: &Config) -> Vec<Violation> {
    let headings = common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .collect::<Vec<&Heading>>();

    headings.iter().zip(headings.iter().skip(1))
        .filter(|(prev, current)| current.depth > prev.depth + 1)
        .map(|(prev, current)| {
            let expected = "#".repeat(prev.depth as usize + 1);
            let expected_minus_one = "#".repeat(prev.depth as usize);
            let actual = "#".repeat(current.depth as usize);
            ViolationBuilder::default()
                .message("Heading level incremented by more then one at a time")
                .assertion(&format!("Expected {expected} or less, got {actual}"))
                .position(&current.position)
                .push_fix(&format!("Decrease the heading level so it will be {expected}, {expected_minus_one} or less"))
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# H1

## H2

#### H4

### H3

## H2

### H3")]
    fn detects_when_heading_level_increment_by_more_then_one(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(
            MD001.check(ast, file, config),
            vec![ViolationBuilder::default()
                .assertion("Expected ### or less, got ####")
                .message("Heading level incremented by more then one at a time")
                .position(&Some(Position::new(5, 1, 13, 5, 8, 20)))
                .set_fixes(vec![String::from(
                    "Decrease the heading level so it will be ###, ## or less"
                )])
                .build()]
        );
    }
}
