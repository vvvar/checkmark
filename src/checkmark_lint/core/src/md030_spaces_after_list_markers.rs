use checkmark_lint_common::*;
use checkmark_lint_macro::*;
use common::ast::{try_cast_to_list_item, BfsIterator};

use regex::Regex;

pub const DEFAULT_NUM_SPACES_AFTER_MARKER: u8 = 1;

#[rule(
    requirement = "List marker should be followed by configured number of spaces",
    rationale = "Violations of this rule can lead to improperly rendered content",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md030.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md030(ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation> {
    let expected_num_spaces = match config.style.num_spaces_after_list_marker {
        Some(n) => n,
        None => DEFAULT_NUM_SPACES_AFTER_MARKER,
    };
    BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_list_item(n))
        .filter(|li| !assert_spaces_after_list_marker(li, &file.content, expected_num_spaces))
        .map(|li| {
            violation_builder()
                .assertion(&format!(
                    "Expected {expected_num_spaces} spaces, got other amount"
                ))
                .push_fix(&format!(
                    "Ensure {expected_num_spaces} spaces are used after the list marker"
                ))
                .position(&li.position)
                .build()
        })
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default().message("Wrong number of spaces after the list marker")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn to_list_items_ast(src: &str) -> Vec<ListItem> {
        let ast = common::ast::parse(src).unwrap();
        BfsIterator::from(&ast)
            .filter_map(|n| try_cast_to_list_item(n))
            .cloned()
            .collect::<Vec<ListItem>>()
    }

    #[test]
    fn md030_assert_spaces_after_list_marker() {
        let numbered = "0. One\n1. Two\n2. Three";
        for li in to_list_items_ast(numbered) {
            assert!(assert_spaces_after_list_marker(
                &li,
                numbered,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let asterisk = "* One\n* Two\n* Three";
        for li in to_list_items_ast(asterisk) {
            assert!(assert_spaces_after_list_marker(
                &li,
                asterisk,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let dash = "- One\n- Two\n- Three";
        for li in to_list_items_ast(dash) {
            assert!(assert_spaces_after_list_marker(
                &li,
                dash,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let plus = "+ One\n+ Two\n+ Three";
        for li in to_list_items_ast(plus) {
            assert!(assert_spaces_after_list_marker(
                &li,
                plus,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }

        let nested_mix = "1. One\n   + Two\n   - Three";
        for li in to_list_items_ast(nested_mix) {
            assert!(assert_spaces_after_list_marker(
                &li,
                nested_mix,
                DEFAULT_NUM_SPACES_AFTER_MARKER
            ));
        }
    }

    #[rule_test(markdown = "
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
")]
    fn e2e_happy(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(MD030.check(ast, file, config), vec![]);
    }

    #[rule_test(markdown = "
# Invalid. Unordered. Asterisk

*   Foo

    Second paragraph
            
*   Bar

# Invalid. Ordered

1.  Foo

    Second paragraph

1.  Bar
")]
    fn e2e_negative(ast: &Node, file: &MarkDownFile, config: &Config) {
        // Negative cases
        assert_eq!(
            MD030.check(ast, file, config),
            vec![
                violation_builder()
                    .assertion("Expected 1 spaces, got other amount")
                    .push_fix("Ensure 1 spaces are used after the list marker")
                    .position(&Some(Position::new(4, 1, 33, 7, 13, 75)))
                    .build(),
                violation_builder()
                    .assertion("Expected 1 spaces, got other amount")
                    .push_fix("Ensure 1 spaces are used after the list marker")
                    .position(&Some(Position::new(8, 1, 76, 9, 1, 84)))
                    .build(),
                violation_builder()
                    .assertion("Expected 1 spaces, got other amount")
                    .push_fix("Ensure 1 spaces are used after the list marker")
                    .position(&Some(Position::new(12, 1, 105, 15, 1, 135)))
                    .build(),
                violation_builder()
                    .assertion("Expected 1 spaces, got other amount")
                    .push_fix("Ensure 1 spaces are used after the list marker")
                    .position(&Some(Position::new(16, 1, 136, 16, 8, 143)))
                    .build()
            ]
        );
    }
}
