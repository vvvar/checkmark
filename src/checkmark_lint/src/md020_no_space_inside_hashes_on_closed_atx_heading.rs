use crate::violation::{Violation, ViolationBuilder};
use common::{find_offset_by_line_number, MarkDownFile};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD020")
        .message("No space inside hashes on closed atx style heading")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md020.md")
        .push_fix("Separate the heading text from the hash character by a single space")
        .is_fmt_fixable(true)
}

// Returns true if the line ends
// with atx heading without one
// space before hash symbol
// Example: "## this_will_return_true##"
fn ends_with_atx_heading_without_space(text: &str) -> bool {
    // Pattern: start of the line followed by one or more hash
    //          characters followed by on or more of any characters
    //          that ens with non-whitespace character followed by
    //          one or more hash characters
    Regex::new(r"^#+.+\S#+").unwrap().is_match(text)
}

fn to_issue(line_number: usize, line: &str, file: &str) -> Violation {
    let offset_start =
        find_offset_by_line_number(file, line_number) + line.rfind(' ').unwrap_or(0) + 1;
    let offset_end = find_offset_by_line_number(file, line_number + 1) - 1;
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
        .filter(|(_, line)| ends_with_atx_heading_without_space(line))
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md020() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "##  Heading 2##".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(0, 1, 12, 0, 1, 15)))
                .build(),],
            md020_no_space_inside_hashes_on_closed_atx_heading(&file)
        );
    }
}
