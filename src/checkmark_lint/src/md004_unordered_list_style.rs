use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{ListItem, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD004")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md004.md")
        .rationale("Consistent formatting makes it easier to understand a document")
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

    let ast = common::ast::parse(&file.content).unwrap();

    let unordered_list_items = common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_list(n))
        .filter(|l| l.ordered == false) // We only care about unordered lists
        .flat_map(|l| {
            // Get all list items from them
            let mut items = vec![];
            for child in &l.children {
                if let Node::ListItem(li) = child {
                    items.push(li);
                }
            }
            items
        })
        .collect::<Vec<_>>();

    // Get style of unordered list item
    let get_list_item_style = |li: &ListItem, source: &str| -> UnorderedListStyle {
        let offset_start = li.position.as_ref().unwrap().start.offset;
        let offset_end = li.position.as_ref().unwrap().end.offset;
        let text = source
            .get(offset_start..offset_end)
            .unwrap_or("")
            .replace(' ', "");
        if text.starts_with('*') {
            UnorderedListStyle::Asterisk
        } else if text.starts_with('+') {
            UnorderedListStyle::Plus
        } else {
            UnorderedListStyle::Dash
        }
    };

    let preferred_style = match style {
        UnorderedListStyle::Consistent => {
            if let Some(li) = unordered_list_items.first() {
                get_list_item_style(li, &file.content)
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
        .filter(|li| get_list_item_style(li, &file.content).ne(&preferred_style))
        .map(|li| {
            let mut violation = violation_builder()
                .position(&li.position)
                .message(&format!(
                    "Wrong unordered list item style. Expected {:#?}, got {:#?}",
                    &preferred_style.as_string(),
                    get_list_item_style(li, &file.content).as_string()
                ));
            if style.eq(&UnorderedListStyle::Consistent) {
                violation = violation.push_fix(&format!(
                    "Unordered list item style is configured to be consistent across the document. First list item in document uses {:#?} symbol, but this one uses {:#?}. Consider replacing {:#?} with {:#?}",
                    &preferred_style.as_string(),
                    get_list_item_style(li, &file.content).as_string(),
                    get_list_item_style(li, &file.content).as_string(),
                    &preferred_style.as_string(),
                ));
            } else {
                violation = violation.push_fix(&format!(
                    "Unordered list item style is configured to use {:#?} symbol. Consider replacing {:#?} with {:#?}",
                    &preferred_style.as_string(),
                    get_list_item_style(li, &file.content).as_string(),
                    &preferred_style.as_string()
                ));
            }
            violation.build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md001() {
        let file = MarkDownFile {
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
                    .position(&Some(Position::new(4, 1, 37, 4, 9, 45)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(Position::new(5, 1, 46, 6, 1, 55)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(Position::new(7, 1, 56, 7, 9, 64)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(Position::new(8, 1, 65, 8, 9, 73)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(Position::new(9, 1, 74, 10, 1, 83)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(Position::new(11, 1, 84, 11, 9, 92)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(Position::new(12, 1, 93, 12, 9, 101)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(Position::new(13, 1, 102, 14, 1, 111)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(Position::new(19, 1, 140, 19, 9, 148)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"+\"")
                    .position(&Some(Position::new(20, 1, 149, 20, 9, 157)))
                    .build(),
                violation_builder()
                    .message("Wrong unordered list item style. Expected \"*\", got \"-\"")
                    .position(&Some(Position::new(21, 1, 158, 21, 9, 166)))
                    .build()
            ],
            md004_unordered_list_style(&file, &UnorderedListStyle::Consistent)
        );
    }
}
