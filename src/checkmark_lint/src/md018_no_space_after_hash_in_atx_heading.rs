use crate::violation::{Violation, ViolationBuilder};
use common::{find_offset_by_line_number, MarkDownFile};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD018")
        .message("No space after hash on atx style heading")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md018.md")
        .push_fix("Separate the heading text from the hash character by a single space")
        .is_fmt_fixable(true)
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

pub fn md018_no_space_after_hash_in_atx_heading(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD018] File: {:#?}", &file.path);
    file.content
        .lines()
        .enumerate()
        .filter(|(_, line)| start_with_atx_heading_without_space(line))
        .map(|(i, line)| to_issue(i, line, &file.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md018() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "#fff".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(0, 1, 0, 0, 1, 1)))
                .build(),],
            md018_no_space_after_hash_in_atx_heading(&file)
        );
    }
}
