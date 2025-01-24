use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use line_numbers::LinePositions;

#[rule(
    requirement = "Single blank line should be used to separate elements",
    rationale = "Except in a code block, blank lines serve no purpose and do not affect the rendering of content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md012.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md012(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    file.content
        .match_indices("\n\n\n")
        .map(|(i, _)| {
            let line_positions = LinePositions::from(file.content.as_str());
            ViolationBuilder::default()
                .message("Multiple consecutive blank lines")
                .assertion("Expected single blank line, got multiple")
                .push_fix("Remove unnecessary blank line")
                .position(&Some(Position::new(
                    line_positions.from_offset(i + 1).as_usize(),
                    1,
                    i + 1,
                    line_positions.from_offset(i + 2).as_usize(),
                    1,
                    i + 2,
                )))
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# H1


## H2")]
    fn detect_multiple_consecutive_blank_lines(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Multiple consecutive blank lines")
                .assertion("Expected single blank line, got multiple")
                .push_fix("Remove unnecessary blank line")
                .position(&Some(Position::new(1, 1, 5, 2, 1, 6)))
                .build()],
            MD012.check(ast, file, config)
        );
    }
}
