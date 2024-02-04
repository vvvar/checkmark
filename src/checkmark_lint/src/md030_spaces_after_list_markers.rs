use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{ListItem, Node};
use regex::Regex;

pub const DEFAULT_NUM_SPACES_AFTER_MARKER: u8 = 1;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD030")
        .message("Spaces after list markers")
        .rationale("Violations of this rule can lead to improperly rendered content.")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md030.md")
        .is_fmt_fixable(true)
}

// Returns true when number of spaces after list marker matches expected value
fn assert_spaces_after_list_marker(l: &ListItem, source: &str, expected_num_spaces: u8) -> bool {
    let offset_start = l.position.as_ref().unwrap().start.offset;
    let offset_end = l.position.as_ref().unwrap().end.offset;
    let mut text = source.get(offset_start..offset_end).unwrap_or("");
    // Strip list marker. Pattern:
    // Either: numbered list,"*"-prefixed, "-"-prefixed or "+"-prefixed
    if let Some(matched) = Regex::new(r"(\b.\.|\*|\-|\+)").unwrap().find(text) {
        text = text.get(matched.end()..).unwrap_or("");
    }
    let mut ident: u8 = 0;
    for char in text.chars() {
        if char.eq(&' ') || char.eq(&'\t') {
            ident += 1;
        } else {
            break;
        }
    }
    ident.eq(&expected_num_spaces)
}

pub fn md030_spaces_after_list_markers(
    file: &MarkDownFile,
    expected_num_spaces: u8,
) -> Vec<Violation> {
    log::debug!("[MD030] File: {:#?}", &file.path);
    let ast = parse(&file.content).unwrap();
    let mut list_items: Vec<&ListItem> = vec![];
    for_each(&ast, |node| {
        if let Node::ListItem(li) = node {
            list_items.push(li);
        }
    });
    log::debug!("[MD030] List items: {:#?}", &list_items);
    list_items
        .iter()
        .filter(|li| !assert_spaces_after_list_marker(li, &file.content, expected_num_spaces))
        .map(|li| {
            violation_builder()
                .position(&li.position)
                .push_fix(&format!(
                    "Ensure {} spaces are used after the list marker.",
                    expected_num_spaces
                ))
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn to_list_items_ast(src: &str) -> Vec<ListItem> {
        let ast = parse(&src).unwrap();
        let mut list_items: Vec<ListItem> = vec![];
        for_each(&ast, |node| {
            if let Node::ListItem(li) = node {
                list_items.push(li.clone());
            }
        });
        list_items
    }

    #[test]
    fn md030_assert_spaces_after_list_marker() {
        let numbered = "0. One\n1. Two\n2. Three";
        for li in to_list_items_ast(&numbered) {
            assert!(assert_spaces_after_list_marker(
                &li,
                &numbered,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let asterisk = "* One\n* Two\n* Three";
        for li in to_list_items_ast(&asterisk) {
            assert!(assert_spaces_after_list_marker(
                &li,
                &asterisk,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let dash = "- One\n- Two\n- Three";
        for li in to_list_items_ast(&dash) {
            assert!(assert_spaces_after_list_marker(
                &li,
                &dash,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let plus = "+ One\n+ Two\n+ Three";
        for li in to_list_items_ast(&plus) {
            assert!(assert_spaces_after_list_marker(
                &li,
                &plus,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let nested_mix = "1. One\n   + Two\n   - Three";
        for li in to_list_items_ast(&nested_mix) {
            assert!(assert_spaces_after_list_marker(
                &li,
                &nested_mix,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }
    }

    #[test]
    fn md029_e2e_happy() {
        let valid_file = common::MarkDownFile {
            path: String::from("test.md"),
            content: String::from(
                "
# Valid. Unordered. Asterisk

* Do this
* Do that

# Valid. Unordered. Asterisk. Nested

* Do this
  * Do that
    * Do this also

# Valid. Unordered. Dash

- Do this
- Do that

# Valid. Unordered. Plus

+ Do this
+ Do that

# Valid. Ordered

1. Do this
2. Do that
",
            ),
            issues: vec![],
        };
        assert_eq!(
            md030_spaces_after_list_markers(&valid_file, DEFAULT_NUM_SPACES_AFTER_MARKER),
            vec![]
        );
    }

    #[test]
    fn md029_e2e_negative() {
        // Negative cases
        let invalid_file = common::MarkDownFile {
            path: String::from("test.md"),
            content: String::from(
                "
# Invalid. Unordered. Asterisk

*   Foo

    Second paragraph
            
*   Bar

# Invalid. Ordered

1.  Foo

    Second paragraph

1.  Bar
",
            ),
            issues: vec![],
        };

        assert_eq!(
            md030_spaces_after_list_markers(&invalid_file, DEFAULT_NUM_SPACES_AFTER_MARKER),
            vec![
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(4, 1, 33, 7, 13, 75)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(8, 1, 76, 9, 1, 84)))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(
                        12, 1, 105, 15, 1, 135
                    )))
                    .build(),
                violation_builder()
                    .position(&Some(markdown::unist::Position::new(
                        16, 1, 136, 16, 8, 143
                    )))
                    .build()
            ]
        );
    }
}
