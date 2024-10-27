use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::{List, Node};
use std::collections::VecDeque;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD029")
        .message("Ordered list item prefix should go in ordered")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md029.md")
        .rationale("Consistent formatting makes it easier to understand a document.")
        .push_fix("Fix the prefixes to be in numerical order.")
        .is_fmt_fixable(true)
}

fn counts_from_zero(l: &List, source: &str) -> bool {
    if let Some(li) = l.children.first() {
        let offset_start = li.position().as_ref().unwrap().start.offset;
        let offset_end = li.position().as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        text.starts_with("0.") || text.starts_with("00.")
    } else {
        false
    }
}

fn counts_from_one(l: &List, source: &str) -> bool {
    if let Some(li) = l.children.first() {
        let offset_start = li.position().as_ref().unwrap().start.offset;
        let offset_end = li.position().as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        text.starts_with("1.") || text.starts_with("01.")
    } else {
        false
    }
}

fn all_prefixes_are_zeros(l: &List, source: &str) -> bool {
    l.children.iter().all(|li: &Node| {
        let offset_start = li.position().as_ref().unwrap().start.offset;
        let offset_end = li.position().as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        text.starts_with("0.") || text.starts_with("00.")
    })
}

fn all_prefixes_are_ones(l: &List, source: &str) -> bool {
    l.children.iter().all(|li: &Node| {
        let offset_start = li.position().as_ref().unwrap().start.offset;
        let offset_end = li.position().as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        text.starts_with("1.") || text.starts_with("01.")
    })
}

fn increases_prefix_in_numerical_order(l: &List, source: &str) -> bool {
    let mut increasing_in_num_order = true;
    let mut current_num = if counts_from_zero(l, source) { 0 } else { 1 };
    for li in &l.children {
        let offset_start = li.position().as_ref().unwrap().start.offset;
        let offset_end = li.position().as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        let li_index = text.chars().take_while(|&ch| ch != '.').collect::<String>();
        if let Ok(parsed_li_index) = li_index.parse::<i32>() {
            if parsed_li_index != current_num {
                increasing_in_num_order = false;
                break;
            }
        }
        current_num += 1;
    }
    increasing_in_num_order
}

// Determine the case when we have for ex.:
//
// 1. One
//    ```text
//    Code block
//    ```
// 2. Two
//
// or:
//
// 1. One
//    > Quote
// 2. Two
fn two_lists_split_by_code_or_block_quote(
    root: &markdown::mdast::Node,
) -> Option<markdown::unist::Position> {
    let mut stack: VecDeque<&Node> = VecDeque::new();
    for el in root.children().unwrap() {
        if stack.len() >= 3 {
            stack.pop_front();
        }
        stack.push_back(el);

        let mut is_first_list = false;
        if let Some(Node::List(l)) = stack.front() {
            if l.ordered {
                is_first_list = true;
            }
        }
        let mut is_second_code_or_quote = false;
        if let Some(el) = stack.get(1) {
            if let Node::Code(_) = el {
                is_second_code_or_quote = true;
            }
            if let Node::Blockquote(_) = el {
                is_second_code_or_quote = true;
            }
        }
        let mut is_third_list = false;
        if let Some(Node::List(l)) = stack.get(2) {
            if l.ordered {
                is_third_list = true;
            }
        }
        if is_first_list && is_second_code_or_quote && is_third_list {
            return Some(stack.get(1).unwrap().position().unwrap().clone());
        }
    }
    None
}

pub fn md029_ordered_list_item_prefix(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD029] File: {:#?}", &file.path);

    let ast = common::ast::parse(&file.content).unwrap();
    let mut violations = common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_list(n))
        .filter(|l| l.ordered)
        .filter(|h| {
            let counts_from_zero = counts_from_zero(h, &file.content);
            let counts_from_one = counts_from_one(h, &file.content);
            let all_prefixes_are_zeros = all_prefixes_are_zeros(h, &file.content);
            let all_prefixes_are_ones = all_prefixes_are_ones(h, &file.content);
            let increases_prefix_in_numerical_order =
                increases_prefix_in_numerical_order(h, &file.content);
            // Everything that does not satisfy our criteria of a valid list is a violation
            !((all_prefixes_are_zeros || all_prefixes_are_ones)
                || (increases_prefix_in_numerical_order && (counts_from_zero || counts_from_one)))
        })
        .map(|h| violation_builder().position(&h.position).build())
        .collect::<Vec<Violation>>();
    if let Some(position) = two_lists_split_by_code_or_block_quote(&ast) {
        violations.push(
            violation_builder()
                .position(&Some(position.clone()))
                .message("Improperly-indented code block or quote appears between two list items and breaks the list in two")
                .rationale("Parsers could miss-interpret list and render them as two separate lists.")
                .set_fixes(vec![
                    "Indent the code block or quote so it becomes part of the preceding list item as intended".to_string()
                ])
                .build()
        );
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn to_list_ast(src: &str) -> List {
        let ast = common::ast::parse(src).unwrap();
        if let Some(node) = &ast.children().unwrap().first() {
            match node {
                Node::List(l) => l.clone(),
                _ => panic!("First item is not the list"),
            }
        } else {
            panic!("Incorrect list provided");
        }
    }

    #[test]
    fn md029_detects_when_counts_from_zero() {
        let starts_with_zero = "0. One\n1. Two\n2. Three";
        assert!(counts_from_zero(
            &to_list_ast(starts_with_zero),
            starts_with_zero
        ));

        let starts_with_zero_zero = "00. One\n01. Two\n02. Three";
        assert!(counts_from_zero(
            &to_list_ast(starts_with_zero_zero),
            starts_with_zero_zero
        ));

        let start_with_one = "1. One\n2. Two\n3. Three";
        assert!(!counts_from_zero(
            &to_list_ast(start_with_one),
            start_with_one
        ));

        let start_with_zero_one = "01. One\n02. Two\n03. Three";
        assert!(!counts_from_zero(
            &to_list_ast(start_with_zero_one),
            start_with_zero_one
        ));
    }

    #[test]
    fn md029_detects_when_counts_from_one() {
        // Happy paths
        let starts_with_one = "1. One\n2. Two\n3. Three";
        assert!(counts_from_one(
            &to_list_ast(starts_with_one),
            starts_with_one
        ));

        let starts_with_zero_one = "01. One\n02. Two\n03. Three";
        assert!(counts_from_one(
            &to_list_ast(starts_with_zero_one),
            starts_with_zero_one
        ));

        // Negative cases
        let start_with_two = "2. Two\n3. Three\n4. Four";
        assert!(!counts_from_one(
            &to_list_ast(start_with_two),
            start_with_two
        ));

        let start_with_eleven = "11. Eleven\n12. Twelve\n13. Thirteen";
        assert!(!counts_from_one(
            &to_list_ast(start_with_eleven),
            start_with_eleven
        ));
    }

    #[test]
    fn md029_increases_prefix_in_numerical_order() {
        // Happy paths
        let increase_in_mun_order_from_zero = "0. One\n1. Two\n2. Three";
        assert!(increases_prefix_in_numerical_order(
            &to_list_ast(increase_in_mun_order_from_zero),
            increase_in_mun_order_from_zero
        ));

        let increase_in_mun_order_from_one = "1. One\n2. Two\n3. Three";
        assert!(increases_prefix_in_numerical_order(
            &to_list_ast(increase_in_mun_order_from_one),
            increase_in_mun_order_from_one
        ));

        let increase_in_mun_order_from_zero_one = "01. One\n02. Two\n03. Three";
        assert!(increases_prefix_in_numerical_order(
            &to_list_ast(increase_in_mun_order_from_zero_one),
            increase_in_mun_order_from_zero_one
        ));

        // Negative cases
        let not_valid_increase = "1. Two.\n3. Three.\n";
        assert!(!increases_prefix_in_numerical_order(
            &to_list_ast(not_valid_increase),
            not_valid_increase
        ));

        let not_valid_increase_without_prefix = "0. One\n1. Two\n3. Three";
        assert!(!increases_prefix_in_numerical_order(
            &to_list_ast(not_valid_increase_without_prefix),
            not_valid_increase_without_prefix
        ));

        let not_valid_increase_with_prefix = "00. One\n01. Two\n03. Three";
        assert!(!increases_prefix_in_numerical_order(
            &to_list_ast(not_valid_increase_with_prefix),
            not_valid_increase_with_prefix
        ));
    }

    #[test]
    fn md029_detects_when_all_prefixes_are_zeros() {
        // Happy paths
        let all_zeros = "0. One\n0. Two\n0. Three";
        assert!(all_prefixes_are_zeros(&to_list_ast(all_zeros), all_zeros));

        let all_zeros_zeros = "00. One\n00. Two\n00. Three";
        assert!(all_prefixes_are_zeros(
            &to_list_ast(all_zeros_zeros),
            all_zeros_zeros
        ));

        let mix_zeros_and_zeros_zeros = "00. One\n0. Two\n00. Three";
        assert!(all_prefixes_are_zeros(
            &to_list_ast(mix_zeros_and_zeros_zeros),
            mix_zeros_and_zeros_zeros
        ));

        // Negative cases
        let start_zero_proceed_with_one = "0. One\n1. Two\n2. Three";
        assert!(!all_prefixes_are_zeros(
            &to_list_ast(start_zero_proceed_with_one),
            start_zero_proceed_with_one
        ));

        let start_zero_zero_proceed_with_one = "00. One\n1. Two\n2. Three";
        assert!(!all_prefixes_are_zeros(
            &to_list_ast(start_zero_zero_proceed_with_one),
            start_zero_zero_proceed_with_one
        ));

        let start_with_one = "1. Two\n2. Three";
        assert!(!all_prefixes_are_zeros(
            &to_list_ast(start_with_one),
            start_with_one
        ));
    }

    #[test]
    fn md029_detect_list_items_separated_by_element() {
        // Happy paths
        let two_lists_and_code = "1. First list\n\n```text\nCode block\n```\n\n1. Second list\n";
        assert!(two_lists_split_by_code_or_block_quote(
            &common::ast::parse(two_lists_and_code).unwrap()
        )
        .is_some());

        let two_lists_quote = "1. First list\n\n> Quote\n\n1. Second list\n";
        assert!(two_lists_split_by_code_or_block_quote(
            &common::ast::parse(two_lists_quote).unwrap()
        )
        .is_some());

        // Negative cases
        let one_list = "1. First list\n1. Second list\n";
        assert!(
            two_lists_split_by_code_or_block_quote(&common::ast::parse(one_list).unwrap())
                .is_none()
        );
    }

    #[test]
    fn md029_e2e_happy() {
        let valid_file = common::MarkDownFile {
            path: String::from("test.md"),
            content: String::from(
                "
# Valid. All one

1. Do this.
1. Do that.
1. Done.

# Valid. Ordered

1. Do this.
2. Do that.
3. Done.

# Valid. Counts from zero

0. Do this.
1. Do that.
2. Done.

# Valid. All zeros

0. Do this.
0. Do that.
0. Done.

# Valid. With break

1. First list

   ```text
   Code block
   ```

2. Still first list

# Valid. With break and quote

1. First list

   > Quote

2. Still first list
",
            ),
            issues: vec![],
        };
        assert_eq!(md029_ordered_list_item_prefix(&valid_file), vec![]);
    }

    #[test]
    fn md029_e2e_negative() {
        // Negative cases
        let invalid_file = common::MarkDownFile {
            path: String::from("test.md"),
            content: String::from(
                "
# Invalid. Not in order

1. Do this.
3. Done.

# Invalid. Break between items

1. First list

```text
Code block
```

1. Second list
",
            ),
            issues: vec![],
        };

        assert_eq!(md029_ordered_list_item_prefix(&invalid_file), vec![
            violation_builder()
                .position(&Some(markdown::unist::Position::new(4, 1, 26, 6, 1, 47)))
                .build(),
            violation_builder()
                .message("Improperly-indented code block or quote appears between two list items and breaks the list in two")
                .rationale("Parsers could miss-interpret list and render them as two separate lists.")
                .set_fixes(vec![
                    "Indent the code block or quote so it becomes part of the preceding list item as intended".to_string()
                ])
                .position(&Some(markdown::unist::Position::new(11, 1, 95, 13, 4, 117)))
                .build()
        ]);
    }
}
