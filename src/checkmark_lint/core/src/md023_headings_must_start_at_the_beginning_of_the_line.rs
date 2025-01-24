use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use regex::Regex;

#[rule(
    requirement = "Headings should start at the beginning of the line",
    rationale = "Headings that don't start at the beginning of the line will not be parsed as headings, and will instead appear as regular text",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md023.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md023(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| heading_is_indented(h, &file.content))
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Heading is shifted")
        .assertion("Expected heading to start at the begining of the line, got shifted heading")
        .push_fix("Ensure that all headings start at the beginning of the line(heading inside block quote is an exception)")
}

/// Check is heading matches on these patters:
/// "    # Heading" or ">   # Heading"
fn heading_is_indented(h: &Heading, source: &str) -> bool {
    let line = h.position.as_ref().unwrap().start.line;
    let heading = source.lines().nth(line - 1).unwrap_or("");
    // Pattern #1: String starts with one or more whitespace followed by one or more hash characters
    // Pattern #2: String starts with block quote followed by one space followed by one or more another whitespace
    //             and followed by one or more hash characters
    Regex::new(r"^\s+#+").unwrap().is_match(heading)
        || Regex::new(r"^>\s\s++#+").unwrap().is_match(heading)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "Some text

  # Indented heading

>  # Indented heading")]
    fn detects_left_pad_heading(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(Position::new(3, 1, 11, 3, 21, 31)))
                    .build(),
                violation_builder()
                    .position(&Some(Position::new(5, 3, 35, 5, 22, 54)))
                    .build(),
            ],
            MD023.check(ast, file, config)
        );
    }
}
