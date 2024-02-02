use crate::violation::{Violation, ViolationBuilder};
use common::{find_offset_by_line_number, MarkDownFile};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD020")
        .message("No space inside hashes on closed atx style heading")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md020.md")
        .rationale("Violations of this rule can lead to improperly rendered content")
        .push_fix("Separate the heading text from the hash character by a single space")
        .is_fmt_fixable(true)
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
    violation_builder()
        .position(&Some(markdown::unist::Position::new(
            line_number,
            1,
            offset_start,
            line_number,
            1,
            offset_end,
        )))
        .build()
}

pub fn md020_no_space_inside_hashes_on_closed_atx_heading(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD020] File: {:#?}", &file.path);
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| closed_atx_without_space_before_closing_hash(line))
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md020_detect_atx_heading_ends_with_without_space() {
        // Detects invalid headings
        assert!(closed_atx_without_space_before_closing_hash("## Heading##"));
        // Do not complains about valid headings
        assert!(!closed_atx_without_space_before_closing_hash(
            "#### Via Conan"
        ));
    }

    #[test]
    pub fn md020() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "##  Heading 2##".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(0, 1, 0, 0, 1, 15)))
                .build(),],
            md020_no_space_inside_hashes_on_closed_atx_heading(&file)
        );
    }
}
