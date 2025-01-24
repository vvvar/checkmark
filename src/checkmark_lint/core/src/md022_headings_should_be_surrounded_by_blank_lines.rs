use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement = "Heading should be surrounded with blank lines",
    rationale = "Aside from aesthetic reasons, some parsers, including kramdown, will not parse headings that don't have a blank line before, and will parse them as regular text",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md022.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md022(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .enumerate()
        .filter(|(i, h)| !surrounded_by_blank_lines(i, h, &file.content))
        .map(|(i, h)| to_violation(i, h))
        .collect::<Vec<Violation>>()
}

fn surrounded_by_blank_lines(line: &usize, h: &Heading, source: &str) -> bool {
    let h_line_end = h.position.as_ref().unwrap().end.line;
    // nth == end line of header because nth is 0-indexed while lines are not
    let text_after_heading = source.lines().nth(h_line_end).unwrap_or("");
    // When it is a first heading in document.
    if line.eq(&0) {
        // Only check if there is a blank line after
        text_after_heading.is_empty()
    } else {
        // Otherwise, check both before and after
        // nth == start line of header because nth is 0-indexed while lines are not
        let h_line_start = h.position.as_ref().unwrap().start.line;
        let text_before_heading = source.lines().nth(h_line_start - 2).unwrap_or("");
        text_before_heading.is_empty() && text_after_heading.is_empty()
    }
}

fn to_violation(i: usize, h: &Heading) -> Violation {
    let mut violation = ViolationBuilder::default().position(&h.position);
    if i.eq(&0) {
        violation = violation
            .message("Heading is not followed by blank line")
            .assertion("Expected a blank line after the heading, got none")
            .push_fix("Add a blank line after the the header");
    } else {
        violation = violation
            .message("Heading is not surrounded with blank lines")
            .assertion("Expected a blank line before and after the heading, got none")
            .push_fix("Ensure there is a blank line before and after the header");
    }
    violation.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(
        markdown = "# Heading 1\nSome text\n\nSome more text\n\n  ## Heading 2\n\nSome text\n## Heading 3"
    )]
    fn detects_heading_not_surrounded_with_blank_lines(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Heading is not followed by blank line")
                    .assertion("Expected a blank line after the heading, got none")
                    .push_fix("Add a blank line after the the header")
                    .position(&Some(Position::new(1, 1, 0, 1, 12, 11)))
                    .build(),
                ViolationBuilder::default()
                    .message("Heading is not surrounded with blank lines")
                    .assertion("Expected a blank line before and after the heading, got none")
                    .push_fix("Ensure there is a blank line before and after the header")
                    .position(&Some(Position::new(9, 1, 65, 9, 13, 77)))
                    .build(),
            ],
            MD022.check(ast, file, config)
        );
    }

    #[rule_test(markdown = "## H2\n\n# H1\n")]
    fn does_not_react_when_start_with_heading(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(Vec::<Violation>::new(), MD022.check(ast, file, config));
    }
}
