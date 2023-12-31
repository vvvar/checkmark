use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, MarkDownFile};
use markdown::{
    mdast::{self},
    to_mdast, ParseOptions,
};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD004")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md004.md")
        .is_fmt_fixable(true)
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnorderedListStyle {
    /// Same as first list item in file
    Consistent,
    /// "*"
    Asterisk,
    /// "-"
    Dash,
    /// "+"
    Plus,
}

impl UnorderedListStyle {
    pub fn as_string(&self) -> String {
        match self {
            UnorderedListStyle::Consistent => "",
            UnorderedListStyle::Asterisk => "*",
            UnorderedListStyle::Dash => "-",
            UnorderedListStyle::Plus => "+",
        }
        .to_string()
    }
}

pub fn md004_unordered_list_style(
    file: &MarkDownFile,
    style: &UnorderedListStyle,
) -> Vec<Violation> {
    log::debug!("[MD004] File: {:#?}", &file.path);

    let ast = to_mdast(&file.content, &ParseOptions::gfm()).unwrap();

    // Get all unordered list items
    let mut unordered_list_items: Vec<&mdast::ListItem> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::List(l) = node {
            if !l.ordered {
                for child in &l.children {
                    if let mdast::Node::ListItem(li) = child {
                        unordered_list_items.push(li);
                    }
                }
            }
        }
    });
    log::debug!("[MD004] Unordered list items: {:#?}", &unordered_list_items);

    // Get style of unordered list item
    let get_list_item_style = |li: &mdast::ListItem, source: &str| -> UnorderedListStyle {
        let offset_start = li.position.as_ref().unwrap().start.offset;
        let offset_end = li.position.as_ref().unwrap().end.offset;
        let text = source
            .get(offset_start..offset_end)
            .unwrap_or("")
            .replace(" ", "");
        if text.starts_with("*") {
            UnorderedListStyle::Asterisk
        } else if text.starts_with("+") {
            UnorderedListStyle::Plus
        } else {
            UnorderedListStyle::Dash
        }
    };

    let preferred_style = match style {
        UnorderedListStyle::Consistent => {
            if let Some(li) = unordered_list_items.first() {
                get_list_item_style(&li, &file.content)
            } else {
                UnorderedListStyle::Dash
            }
        }
        UnorderedListStyle::Asterisk => UnorderedListStyle::Asterisk,
        UnorderedListStyle::Dash => UnorderedListStyle::Dash,
        UnorderedListStyle::Plus => UnorderedListStyle::Plus,
    };

    unordered_list_items
        .iter()
        .filter(|li| get_list_item_style(&li, &file.content).ne(&preferred_style))
        .map(|li| {
            violation_builder()
                .position(&li.position)
                .message(&format!(
                    "Wrong unordered list item style. Expected {:#?}, got {:#?}",
                    &preferred_style.as_string(),
                    get_list_item_style(&li, &file.content).as_string()
                ))
                .push_fix(&format!(
                    "Replace {:#?} with {:#?}",
                    get_list_item_style(&li, &file.content).as_string(),
                    &preferred_style.as_string()
                ))
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md001() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# Inconsistent List Styles

* Item 1
- Item 2
+ Item 3

- Item 1
- Item 2
- Item 3

+ Item 1
+ Item 2
+ Item 3

* Item 1
* Item 2
* Item 3

- Item 1
+ Item 2
- Item 3"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(markdown::unist::Position::new(4, 1, 37, 4, 9, 45)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(markdown::unist::Position::new(5, 1, 46, 6, 1, 55)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(markdown::unist::Position::new(7, 1, 56, 7, 9, 64)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(markdown::unist::Position::new(8, 1, 65, 8, 9, 73)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(markdown::unist::Position::new(9, 1, 74, 10, 1, 83)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(markdown::unist::Position::new(11, 1, 84, 11, 9, 92)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(markdown::unist::Position::new(12, 1, 93, 12, 9, 101)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(markdown::unist::Position::new(
                        13, 1, 102, 14, 1, 111
                    )))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(markdown::unist::Position::new(
                        19, 1, 140, 19, 9, 148
                    )))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(markdown::unist::Position::new(
                        20, 1, 149, 20, 9, 157
                    )))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(markdown::unist::Position::new(
                        21, 1, 158, 21, 9, 166
                    )))
                    .build()
            ],
            md004_unordered_list_style(&file, &UnorderedListStyle::Consistent)
        );
    }
}
