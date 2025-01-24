use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement= "List items on the same level should have a consistent indentation",
    rationale= "Violations of this rule can lead to improperly rendered content",
    documentation="https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md005.md",
    is_fmt_fixable=true,
    additional_links=[],
)]
fn md005(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_list(n))
        .filter(|l| !get_miss_aligned_items(l, file).is_empty())
        .flat_map(|l| {
            let num_spaces_expected = &first_list_item_alignment(l, &file.content);
            get_miss_aligned_items(l, file)
                .iter()
                .map(|miss_indented_item| {
                    let num_spaces_actual =
                        get_list_item_alignment(miss_indented_item, &file.content);
                    ViolationBuilder::default()
                        .position(&miss_indented_item.position)
                        .message("Inconsistent indentation for list items at the same level")
                        .assertion(&format!(
                            "Expected {num_spaces_expected} spaces, got {num_spaces_actual} spaces"
                        ))
                        .push_fix(&format!(
                            "Align list item to be indented with {num_spaces_expected} spaces"
                        ))
                        .build()
                })
                .collect::<Vec<Violation>>()
        })
        .collect::<Vec<Violation>>()
}

// Returns number of spaces used to align this list item.
fn get_list_item_alignment(li: &ListItem, source: &str) -> usize {
    let mut padding: usize = 0;
    let num_line = li.position.as_ref().unwrap().start.line;
    if let Some(line) = source.lines().nth(num_line - 1) {
        for char in line.chars() {
            if char.eq(&' ') || char.eq(&'\t') {
                padding += 1;
            } else {
                break;
            }
        }
    }
    padding
}

// Take List, find first item and returns number of spaces it is aligned with.
fn first_list_item_alignment(l: &List, source: &str) -> usize {
    let mut padding: usize = 0;
    for child in &l.children {
        if let Node::ListItem(li) = child {
            padding = get_list_item_alignment(li, source);
            break;
        }
    }
    padding
}

fn get_miss_aligned_items(l: &List, file: &MarkDownFile) -> Vec<ListItem> {
    let expected_alignment = &first_list_item_alignment(l, &file.content);
    let mut miss_indented_items: Vec<ListItem> = vec![];
    let mut num_item: usize = l.start.unwrap_or(1) as usize;
    for child in &l.children {
        if let Node::ListItem(li) = child {
            let actual_alignment = get_list_item_alignment(li, &file.content);
            if l.ordered && first_list_item_alignment(l, &file.content) > 0 {
                // When list is ordered and first item is indented then we assume that
                // there could be two possible cases:
                // 1. All items are indented the same, normal case
                // 2. All items are indented with respect to the max element, e.g.:
                //      1. One
                //      2. Two
                //    ...
                //    100. Hundred
                // So, get max number in this list, calculate how many digits it has,
                // subs from current digit and get possible additional alignment
                let max_num_item = get_max_num_item(l);
                let digits_in_max_item = max_num_item.checked_ilog10().unwrap_or(0) as usize + 1;
                let digits_in_current_item = num_item.checked_ilog10().unwrap_or(0) as usize + 1;
                let additional_num_item_alignment = digits_in_max_item - digits_in_current_item;
                let expected_num_item_alignment =
                    expected_alignment + additional_num_item_alignment;
                // If none of two cases satisfied then it is miss-aligned
                if actual_alignment.ne(&expected_num_item_alignment)
                    && actual_alignment.ne(&additional_num_item_alignment)
                {
                    miss_indented_items.push(li.clone());
                }
            } else if actual_alignment.ne(expected_alignment) {
                miss_indented_items.push(li.clone());
            }
            num_item += 1;
        }
    }
    miss_indented_items
}

// Returns maximum number of item in a list.
fn get_max_num_item(l: &List) -> usize {
    let mut num_item: usize = l.start.unwrap_or(1) as usize;
    for child in &l.children {
        if let Node::ListItem(_) = child {
            num_item += 1;
        }
    }
    num_item
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# Inconsistent List Item Indentation

## Bad un-aligned list

* Item 1
  * Nested Item 1
  * Nested Item 2
   * A misaligned item

## Good aligned numbered list

 1. Item
 2. Item
 3. Item
 4. Item
 5. Item
 6. Item
 7. Item
 8. Item
 9. Item
10. Item
11. Item

## Bad aligned numbered list

 1. Item
 2. Item
 3. Item
 4. Item
5. A misaligned item
 6. Item
 7. Item
8. Another misaligned item
 9. Item
10. Item
11. Item

## Normal aligned numbered list

1. Item
   1. Item
   2. Item

")]
    fn detect_inconsistent_lit_item_indent(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![
                ViolationBuilder::default()
                    .message("Inconsistent indentation for list items at the same level")
                    .assertion("Expected 2 spaces, got 3 spaces")
                    .position(&Some(Position::new(8, 3, 109, 9, 1, 130)))
                    .set_fixes(vec![String::from(
                        "Align list item to be indented with 2 spaces"
                    )])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent indentation for list items at the same level")
                    .assertion("Expected 1 spaces, got 0 spaces")
                    .position(&Some(Position::new(30, 1, 328, 30, 21, 348)))
                    .set_fixes(vec![String::from(
                        "Align list item to be indented with 1 spaces"
                    )])
                    .build(),
                ViolationBuilder::default()
                    .message("Inconsistent indentation for list items at the same level")
                    .assertion("Expected 1 spaces, got 0 spaces")
                    .position(&Some(Position::new(33, 1, 367, 33, 27, 393)))
                    .set_fixes(vec![String::from(
                        "Align list item to be indented with 1 spaces"
                    )])
                    .build()
            ],
            MD005.check(ast, file, config)
        );
    }

    //     make_rule_test! {
    //         MD005,
    //         should_detect_inconsistent_lit_item_indent,
    //         [
    //             ViolationBuilder::default()
    //                 .message("Inconsistent indentation for list items at the same level")
    //                 .assertion("Expected 2 spaces, got 3 spaces")
    //                 .position(&Some(Position::new(8, 3, 109, 9, 1, 130)))
    //                 .set_fixes(vec![String::from(
    //                     "Align list item to be indented with 2 spaces"
    //                 )])
    //                 .build(),
    //             ViolationBuilder::default()
    //                 .message("Inconsistent indentation for list items at the same level")
    //                 .assertion("Expected 1 spaces, got 0 spaces")
    //                 .position(&Some(Position::new(30, 1, 328, 30, 21, 348)))
    //                 .set_fixes(vec![String::from(
    //                     "Align list item to be indented with 1 spaces"
    //                 )])
    //                 .build(),
    //             ViolationBuilder::default()
    //                 .message("Inconsistent indentation for list items at the same level")
    //                 .assertion("Expected 1 spaces, got 0 spaces")
    //                 .position(&Some(Position::new(33, 1, 367, 33, 27, 393)))
    //                 .set_fixes(vec![String::from(
    //                     "Align list item to be indented with 1 spaces"
    //                 )])
    //                 .build()
    //         ],
    // "# Inconsistent List Item Indentation

    // ## Bad un-aligned list

    // * Item 1
    //   * Nested Item 1
    //   * Nested Item 2
    //    * A misaligned item

    // ## Good aligned numbered list

    //  1. Item
    //  2. Item
    //  3. Item
    //  4. Item
    //  5. Item
    //  6. Item
    //  7. Item
    //  8. Item
    //  9. Item
    // 10. Item
    // 11. Item

    // ## Bad aligned numbered list

    //  1. Item
    //  2. Item
    //  3. Item
    //  4. Item
    // 5. A misaligned item
    //  6. Item
    //  7. Item
    // 8. Another misaligned item
    //  9. Item
    // 10. Item
    // 11. Item

    // ## Normal aligned numbered list

    // 1. Item
    //    1. Item
    //    2. Item

    // "
    //     }
}
