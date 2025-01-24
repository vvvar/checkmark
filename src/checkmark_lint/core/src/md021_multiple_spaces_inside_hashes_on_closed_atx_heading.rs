use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use common::{find_offset_by_line_number, MarkDownFile};
use regex::Regex;

#[rule(
    requirement = "Single space should be used after/before hashes in closed ATX-style headings",
    rationale = "Extra space has no purpose and does not affect the rendering of content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md021.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md021(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| ends_with_atx_heading_without_more_then_one_space(line))
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Multiple spaces inside hashes on closed atx style heading")
        .assertion("Expected single space after/before hash in closed ATX-style heading, got more")
        .push_fix("Separate the heading text from the hash character by a single space")
}

// Returns true if the line ends
// with atx heading with more then
// one whitespace before hash symbol
// Example: "## Heading   ##"
fn ends_with_atx_heading_without_more_then_one_space(text: &str) -> bool {
    // Pattern: start of the line followed by one or more hash
    //          characters followed by any amount of any characters
    //          that ens with whitespace followed by one or more
    //          whitespace followed by one or more hash character
    Regex::new(r"^#+.*\s\s+#+").unwrap().is_match(text)
}

fn to_issue(line_number: usize, line: &str, file: &str) -> Violation {
    let offset_start = find_offset_by_line_number(file, line_number) + line.rfind(' ').unwrap_or(0);
    let offset_end = find_offset_by_line_number(file, line_number + 1) - 1; // - 1 whitespace
    violation_builder()
        .position(&Some(Position::new(
            line_number,
            1,
            offset_start,
            line_number,
            1,
            offset_end,
        )))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "##  Heading 2  ##")]
    fn detects_multiples_spaces_in_atx_headeing(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(0, 1, 14, 0, 1, 17)))
                .build(),],
            MD021.check(ast, file, config)
        );
    }
}
