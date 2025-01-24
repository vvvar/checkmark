use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use common::{find_offset_by_line_number, MarkDownFile};

#[rule(
    requirement = "Hashes in closed ATX-style heading should be followed & preceeded by spaces",
    rationale = "Violations of this rule can lead to improperly rendered content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md020.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md020(_: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    let mut is_code_block = false;
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            if line.contains("```") {
                is_code_block = !is_code_block;
            }
            if is_code_block {
                false
            } else {
                closed_atx_without_space_before_closing_hash(line)
            }
        })
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

fn remove_trailing_suffix(s: &str, c: char) -> String {
    let mut result = String::from(s);
    while result.ends_with(c) {
        if let Some(s) = result.strip_suffix(c) {
            result = s.to_string()
        }
    }
    result
}

// Returns true if the line is an ATX heading
// without one space before a closing hash symbol
// Example: "## Heading##"
fn closed_atx_without_space_before_closing_hash(text: &str) -> bool {
    // Detect ATX headings
    if text.starts_with('#') {
        let mut heading = String::from(text);
        heading = remove_trailing_suffix(&heading, ' ');
        // Detect closing style ATX headings
        if heading.ends_with('#') {
            heading = remove_trailing_suffix(&heading, '#');
            !heading.ends_with(' ')
        } else {
            false
        }
    } else {
        false
    }
}

fn to_issue(line_number: usize, line: &str, file: &str) -> Violation {
    let offset_start = find_offset_by_line_number(file, line_number);
    let offset_end = find_offset_by_line_number(file, line_number) + line.len();
    ViolationBuilder::default()
        .message("Missing space inside hashes in closed atx style heading")
        .assertion("Expected hash symbols to be followes and preceeded with hashes, got none")
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

    #[test]
    fn detect_atx_heading_ends_with_without_space() {
        // Detects invalid headings
        assert!(closed_atx_without_space_before_closing_hash("## Heading##"));
        // Do not complains about valid headings
        assert!(!closed_atx_without_space_before_closing_hash(
            "#### Via Conan"
        ));
    }

    #[rule_test(markdown = "##  Heading 2##")]
    fn detect_missing_spaces_in_atx_heading(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Missing space inside hashes in closed atx style heading")
                .assertion(
                    "Expected hash symbols to be followes and preceeded with hashes, got none"
                )
                .push_fix("Separate the heading text from the hash character by a single space")
                .position(&Some(Position::new(0, 1, 0, 0, 1, 15)))
                .build(),],
            MD020.check(ast, file, config)
        );
    }

    #[rule_test(markdown = "
```txt
## Still Valid##
```")]
    fn should_skip_code_blocks(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(Vec::<Violation>::new(), MD020.check(ast, file, config));
    }
}
