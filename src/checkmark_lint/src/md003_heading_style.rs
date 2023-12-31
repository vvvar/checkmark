use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, MarkDownFile};
use markdown::{
    mdast::{self},
    to_mdast, ParseOptions,
};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD003")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md001.md")
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

pub fn md003_heading_style(file: &MarkDownFile, style: &HeadingStyle) -> Vec<Violation> {
    log::debug!("[MD003] File: {:#?}, style: {:#?}", &file.path, &style);

    let ast = to_mdast(&file.content, &ParseOptions::gfm()).unwrap();

    // Get all headings
    let mut headings: Vec<&mdast::Heading> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::Heading(h) = node {
            headings.push(h);
        }
    });
    log::debug!("[MD003] Headings: {:#?}", &headings);

    let get_heading_style = |h: &mdast::Heading, source: &str| -> HeadingStyle {
        let offset_start = h.position.as_ref().unwrap().start.offset;
        let offset_end = h.position.as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        if text.starts_with("#") {
            HeadingStyle::Atx
        } else {
            HeadingStyle::SetExt
        }
    };

    let preferred_style = match style {
        HeadingStyle::Consistent => {
            if let Some(h) = headings.first() {
                get_heading_style(&h, &file.content)
            } else {
                HeadingStyle::Atx
            }
        }
        HeadingStyle::Atx => HeadingStyle::Atx,
        HeadingStyle::SetExt => HeadingStyle::SetExt,
    };
    log::debug!(
        "[MD003] Document should have heading style: {:#?}",
        &preferred_style
    );

    headings
        .iter()
        .filter(|h| get_heading_style(&h, &file.content).ne(&preferred_style))
        .map(|h| {
            violation_builder()
                .message(&format!(
                    "Wrong heading style. Expected {:#?}, got {:#?}",
                    preferred_style.as_str(),
                    get_heading_style(&h, &file.content).as_str()
                ))
                .push_fix(&format!("Change to {:#?} style", preferred_style.as_str()))
                .push_fix(
                    "See Markdown reference: https://www.markdownguide.org/basic-syntax/#headings",
                )
                .position(&h.position)
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md003() {
        let mut file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1
        
H2
-----"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"ATX\", got \"SetExt\"")
                .position(&Some(markdown::unist::Position::new(3, 1, 14, 4, 6, 22)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::Consistent),
        );

        file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"SetExt\", got \"ATX\"")
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 5, 4)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::SetExt),
        );

        file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "H1
==========="
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"ATX\", got \"SetExt\"")
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 2, 12, 14)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::Atx),
        );
    }
}
