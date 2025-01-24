use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use common::{find_offset_by_line_number, MarkDownFile};
use regex::Regex;

#[rule(
    requirement = "Hash symbol in ATX-style heading should be followed with a space symbol",
    rationale = "Violations of this rule can lead to improperly rendered content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md018.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md018(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| start_with_atx_heading_without_space(line))
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

// Returns true if the line starts
// with atx heading without space
// Example: "##this_will_return_true"
fn start_with_atx_heading_without_space(text: &str) -> bool {
    // Pattern: start of the line followed by one or more hash
    //          characters followed by any non-numeric letter
    Regex::new(r"^#+\b").unwrap().is_match(text)
}

fn to_issue(line_number: usize, line: &str, file: &str) -> Violation {
    let offset_start = find_offset_by_line_number(file, line_number);
    let offset_end = offset_start + line.find(' ').unwrap_or(1);
    ViolationBuilder::default()
        .message("Missing a space after a hash in ATX-style heading")
        .assertion("Expected a space after the hash symbol, got none")
        .push_fix("Separate the heading text from the hash character by a single space")
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

    #[rule_test(markdown = "#fff")]
    fn detect_missing_space_after_atx_heading_hash(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Missing a space after a hash in ATX-style heading")
                .assertion("Expected a space after the hash symbol, got none")
                .push_fix("Separate the heading text from the hash character by a single space")
                .position(&Some(Position::new(0, 1, 0, 0, 1, 1)))
                .build(),],
            MD018.check(ast, file, config)
        );
    }
}
