use crate::violation::{Violation, ViolationBuilder};
use common::{find_offset_by_line_number, MarkDownFile};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD021")
        .message("Multiple spaces inside hashes on closed atx style heading")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md021.md")
        .push_fix("Separate the heading text from the hash character by a single space")
        .is_fmt_fixable(true)
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

pub fn md021_multiple_spaces_inside_hashes_on_closed_atx_heading(
    file: &MarkDownFile,
) -> Vec<Violation> {
    log::debug!("[MD021] File: {:#?}", &file.path);
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| ends_with_atx_heading_without_more_then_one_space(line))
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md021() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "##  Heading 2  ##".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(0, 1, 14, 0, 1, 17)))
                .build(),],
            md021_multiple_spaces_inside_hashes_on_closed_atx_heading(&file)
        );
    }
}
