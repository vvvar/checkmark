use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement="Unordered list elements style should be consistent",
    rationale="Consistent formatting makes it easier to understand a document",
    documentation= "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md004.md",
    is_fmt_fixable=true,
    additional_links=[],
)]
fn md004(ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation> {
    let unordered_list_items = common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_list(n))
        .filter(|l| !l.ordered) // We only care about unordered lists
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

    let style = UnorderedListStyle::from(config);

    let preferred_style = match &style {
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
            let expected_symbol = preferred_style.as_string();
            let actual_symbol = get_list_item_style(li, &file.content).as_string();

            let mut violation =
                ViolationBuilder::default()
                    .position(&li.position)
                    .push_fix(&format!(
                        "Consider replacing {:#?} with {:#?}",
                        &actual_symbol, &expected_symbol
                    ));

            if style.eq(&UnorderedListStyle::Consistent) {
                violation = violation
                    .message("Inconsistent unordered list item style")
                    .assertion(&format!(
                    "Expected {:#?} marker since first unordered list element uses it, got {:#?}",
                    &expected_symbol, &actual_symbol
                ));
            } else {
                violation = violation
                    .message("Wrong unordered list item style")
                    .assertion(&format!(
                        "Expected {:#?}, got {:#?}",
                        &expected_symbol, &actual_symbol
                    ));
            }

            violation.build()
        })
        .collect::<Vec<Violation>>()
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

impl From<&Config> for UnorderedListStyle {
    fn from(config: &Config) -> Self {
        match config.style.unordered_lists {
            common::UnorderedListStyle::Consistent => UnorderedListStyle::Consistent,
            common::UnorderedListStyle::Dash => UnorderedListStyle::Dash,
            common::UnorderedListStyle::Plus => UnorderedListStyle::Plus,
            common::UnorderedListStyle::Asterisk => UnorderedListStyle::Asterisk,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[rule_test(markdown = "# Inconsistent List Styles

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
- Item 3")]
    fn detect_inconsistent_unordered_list_element_style(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"-\""
                    )
                    .position(&Some(Position::new(4, 1, 37, 4, 9, 45)))
                    .set_fixes(vec![String::from("Consider replacing \"-\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"+\""
                    )
                    .position(&Some(Position::new(5, 1, 46, 6, 1, 55)))
                    .set_fixes(vec![String::from("Consider replacing \"+\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"-\""
                    )
                    .position(&Some(Position::new(7, 1, 56, 7, 9, 64)))
                    .set_fixes(vec![String::from("Consider replacing \"-\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"-\""
                    )
                    .position(&Some(Position::new(8, 1, 65, 8, 9, 73)))
                    .set_fixes(vec![String::from("Consider replacing \"-\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"-\""
                    )
                    .position(&Some(Position::new(9, 1, 74, 10, 1, 83)))
                    .set_fixes(vec![String::from("Consider replacing \"-\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"+\""
                    )
                    .position(&Some(Position::new(11, 1, 84, 11, 9, 92)))
                    .set_fixes(vec![String::from("Consider replacing \"+\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"+\""
                    )
                    .position(&Some(Position::new(12, 1, 93, 12, 9, 101)))
                    .set_fixes(vec![String::from("Consider replacing \"+\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"+\""
                    )
                    .position(&Some(Position::new(13, 1, 102, 14, 1, 111)))
                    .set_fixes(vec![String::from("Consider replacing \"+\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"-\""
                    )
                    .position(&Some(Position::new(19, 1, 140, 19, 9, 148)))
                    .set_fixes(vec![String::from("Consider replacing \"-\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"+\""
                    )
                    .position(&Some(Position::new(20, 1, 149, 20, 9, 157)))
                    .set_fixes(vec![String::from("Consider replacing \"+\" with \"*\"")])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent unordered list item style")
                    .assertion(
                        "Expected \"*\" marker since first unordered list element uses it, got \"-\""
                    )
                    .position(&Some(Position::new(21, 1, 158, 21, 9, 166)))
                    .set_fixes(vec![String::from("Consider replacing \"-\" with \"*\"")])
                    .build()
            ],
            MD004.check(ast, file, config)
        )
    }
}
