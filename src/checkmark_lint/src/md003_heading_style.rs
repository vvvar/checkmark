use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{Heading, Node};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD003")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md003.md")
        .rationale("Consistent formatting makes it easier to understand a document")
        .push_additional_link("https://www.markdownguide.org/basic-syntax/#headings")
        .is_fmt_fixable(true)
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeadingStyle {
    Consistent,
    Atx,
    SetExt,
}

impl HeadingStyle {
    pub fn as_str(&self) -> &str {
        match self {
            HeadingStyle::Consistent => "consistent",
            HeadingStyle::Atx => "ATX",
            HeadingStyle::SetExt => "SetExt",
        }
    }
}

// Detect whether heading is ATX or SetExt style
fn get_heading_style(h: &Heading, source: &str) -> HeadingStyle {
    let offset_start = h.position.as_ref().unwrap().start.offset;
    let offset_end = h.position.as_ref().unwrap().end.offset;
    let heading = source.get(offset_start..offset_end).unwrap_or("");
    // Pattern: starts with zero or more whitespace followed by one or more hash characters
    // This shall capture ATX headings on root and nested level
    if Regex::new(r"\s*#+").unwrap().is_match(heading) {
        HeadingStyle::Atx
    } else {
        HeadingStyle::SetExt
    }
}

// When first heading exist, detects it's style
// Otherwise - fallback to Atx
fn get_first_heading_style(headings: &Vec<&Heading>, source: &str) -> HeadingStyle {
    if let Some(h) = headings.first() {
        get_heading_style(h, source)
    } else {
        HeadingStyle::Atx
    }
}

pub fn md003_heading_style(file: &MarkDownFile, style: &HeadingStyle) -> Vec<Violation> {
    log::debug!("[MD003] File: {:#?}, style: {:#?}", &file.path, &style);

    let ast = common::ast::parse(&file.content).unwrap();
    let headings = common::ast::BfsIterator::from(&ast)
        .filter_map(|node| match node {
            Node::Heading(e) => Some(e),
            _ => None,
        })
        .collect::<Vec<&Heading>>();
    log::debug!("[MD003] Headings: {:#?}", &headings);

    let preferred_style = match style {
        HeadingStyle::Consistent => get_first_heading_style(&headings, &file.content),
        HeadingStyle::Atx => HeadingStyle::Atx,
        HeadingStyle::SetExt => HeadingStyle::SetExt,
    };
    log::debug!(
        "[MD003] Document should have heading style: {:#?}",
        &preferred_style
    );

    headings
        .iter()
        .filter_map(|h| {
            if get_heading_style(h, &file.content).ne(&preferred_style) {
                let mut violation = violation_builder();
                // To have more descriptive messages, we're giving
                // more precise messages based on heading style
                if style.eq(&HeadingStyle::Consistent) {
                    violation = violation.message(&format!(
                        "Inconsistent headings style. First heading in this file is {:#?}, but this one is {:#?}",
                        preferred_style.as_str(),
                        get_heading_style(h, &file.content).as_str()
                    ))
                } else {
                    violation = violation.message(&format!(
                        "Wrong heading style. Expected {:#?}, got {:#?}",
                        style.as_str(),
                        get_heading_style(h, &file.content).as_str()
                    ))
                }
                Some(violation
                    .push_fix(&format!("Change heading style to {:#?}", preferred_style.as_str()))
                    .push_fix("Alternatively, you can enforce specific heading style via either \"headings\" option from the \"[style]\" section in config file or via \"--style-headings\" CLI option")
                    .position(&h.position)
                    .build())
            } else {
                None
            }
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md003() {
        let mut file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1
        
H2
-----"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Inconsistent headings style. First heading in this file is \"ATX\", but this one is \"SetExt\"")
                .position(&Some(Position::new(3, 1, 14, 4, 6, 22)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::Consistent),
        );

        file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"SetExt\", got \"ATX\"")
                .position(&Some(Position::new(1, 1, 0, 1, 5, 4)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::SetExt),
        );

        file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "H1
==========="
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"ATX\", got \"SetExt\"")
                .position(&Some(Position::new(1, 1, 0, 2, 12, 14)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::Atx),
        );
    }
}
