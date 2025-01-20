use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use line_numbers::LinePositions;
use once_cell::sync::Lazy;
use regex::Regex;

#[rule(
    requirement = "Link syntax should not be reversed",
    rationale = "Reversed links are not rendered as usable links",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md011.md",
    additional_links = ["https://www.markdownguide.org/basic-syntax/#links"],
    is_fmt_fixable = true,
)]
fn md011(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    /// Example of lint that shall match - "(link)[https://www.example.com/]"
    static REGEX_CORRECTIONS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\(.*\)\[.*\]").unwrap());

    let vector_match: Vec<Vec<Violation>> = REGEX_CORRECTIONS
        .captures_iter(&file.content)
        .map(|c| {
            c.iter()
                .map(|m| {
                    let offset = m.unwrap().range();
                    let line_positions = LinePositions::from(file.content.as_str());
                    ViolationBuilder::default()
                        .message("Found reversed link syntax")
                        .assertion("Expected normal link syntax, got reversed one")
                        .position(&Some(Position::new(
                            line_positions.from_offset(offset.start).as_usize(),
                            1,
                            offset.start,
                            line_positions.from_offset(offset.end).as_usize(),
                            1,
                            offset.end,
                        )))
                        .build()
                })
                .collect()
        })
        .collect();

    vector_match.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# H1

(Incorrect link one)[https://www.example.com/]

(Incorrect link two)[https://www.example.com/]

")]
    fn detect_reversed_link_syntax(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Found reversed link syntax")
                    .assertion("Expected normal link syntax, got reversed one")
                    .position(&Some(Position::new(2, 1, 6, 2, 1, 52)))
                    .build(),
                ViolationBuilder::default()
                    .message("Found reversed link syntax")
                    .assertion("Expected normal link syntax, got reversed one")
                    .position(&Some(Position::new(4, 1, 54, 4, 1, 100)))
                    .build()
            ],
            MD011.check(ast, file, config)
        );
    }
}
