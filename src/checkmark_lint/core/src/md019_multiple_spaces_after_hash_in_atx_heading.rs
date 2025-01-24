use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use regex::Regex;

#[rule(
    requirement = "Hash symbol in ATX-style heading should be followed with a single space",
    rationale = "Extra space has no purpose and does not affect the rendering of content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md019.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md019(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .filter(|h| start_with_atx_heading_without_space(h, &file.content))
        .map(|h| {
            ViolationBuilder::default()
                .message("Found multiple spaces after hash in atx style heading")
                .assertion("Expected single space, got multiple")
                .push_fix("Separate the heading text from the hash character by a single space")
                .position(&h.position)
                .build()
        })
        .collect()
}

// Returns true if the line starts
// with atx heading with more then
// one space
// Example: "##   this_will_return_true"
fn start_with_atx_heading_without_space(h: &Heading, source: &str) -> bool {
    let line = h.position.as_ref().unwrap().start.line;
    let text = source.lines().nth(line - 1).unwrap_or("");
    // Pattern: start of the line followed by one or more hash
    //          characters followed by single space and one or
    //          more spaces followed by non-numeric letter
    Regex::new(r"^#+\s\s+\b").unwrap().is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "#   fff")]
    fn detects_multiple_spaces_after_hash_symbol(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Found multiple spaces after hash in atx style heading")
                .assertion("Expected single space, got multiple")
                .push_fix("Separate the heading text from the hash character by a single space")
                .position(&Some(Position::new(1, 1, 0, 1, 8, 7)))
                .build()],
            MD019.check(ast, file, config)
        );
    }
}
