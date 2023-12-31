use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, MarkDownFile};
use markdown::{
    mdast::{self},
    to_mdast, ParseOptions,
};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD005")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md005.md")
        .is_fmt_fixable(true)
}

pub fn md005_consistent_list_items_indentation(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD005] File: {:#?}", &file.path);

    let ast = to_mdast(&file.content, &ParseOptions::gfm()).unwrap();

    let mut lists: Vec<&mdast::List> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::List(l) = node {
            lists.push(l);
        }
    });

    let get_list_item_alignment = |li: &mdast::ListItem, source: &str| -> usize {
        let mut padding: usize = 0;
        let num_line = li.position.as_ref().unwrap().start.line;
        if let Some(line) = source.lines().nth(num_line - 1) {
            for char in line.chars() {
                if char.eq(&' ') {
                    padding += 1;
                } else {
                    break;
                }
            }
        }
        padding
    };

    // Take List, find first List Item and return
    // with huw many space symbols it is aligned
    let first_list_item_alignment = |l: &mdast::List, source: &str| -> usize {
        let mut padding: usize = 0;
        for child in &l.children {
            if let mdast::Node::ListItem(li) = child {
                padding = get_list_item_alignment(&li, source);
                break;
            }
        }
        padding
    };

    let get_max_num_item = |l: &mdast::List| -> usize {
        let mut num_item: usize = l.start.unwrap_or(1) as usize;
        for child in &l.children {
            if let mdast::Node::ListItem(_) = child {
                num_item += 1;
            }
        }
        num_item
    };

    let get_miss_aligned_items = |l: &mdast::List| -> Vec<mdast::ListItem> {
        let expected_alignment = &first_list_item_alignment(&l, &file.content);
        let mut miss_indented_items: Vec<mdast::ListItem> = vec![];
        let max_num_item = get_max_num_item(&l);
        let mut num_item: usize = l.start.unwrap_or(1) as usize;
        for child in &l.children {
            if let mdast::Node::ListItem(li) = child {
                let actual_alignment = get_list_item_alignment(&li, &file.content);
                if l.ordered && first_list_item_alignment(&l, &file.content) > 0 {
                    let digits_in_max_item = max_num_item.checked_ilog10().unwrap_or(0) as usize;
                    let digits_in_current_item = num_item.checked_ilog10().unwrap_or(0) as usize;
                    let num_item_alignment = digits_in_max_item - digits_in_current_item;
                    if actual_alignment.ne(&num_item_alignment) {
                        miss_indented_items.push(li.clone());
                    }
                } else {
                    if actual_alignment.ne(expected_alignment) {
                        miss_indented_items.push(li.clone());
                    }
                }
                num_item += 1;
            }
        }
        miss_indented_items
    };

    lists.iter()
        .filter(|l| !get_miss_aligned_items(l).is_empty())
        .map(|l| {
            let expected_alignment = &first_list_item_alignment(&l, &file.content);
            get_miss_aligned_items(l).iter().map(|miss_indented_item|
                violation_builder()
                    .position(&miss_indented_item.position)
                    .message(&format!(
                        "Inconsistent indentation for list items at the same level. Expected {} spaces, got {} spaces",
                        &expected_alignment,
                        get_list_item_alignment(&miss_indented_item, &file.content)
                    ))
                    .push_fix(&format!(
                        "Align list item to be indented with {:#?} spaces",
                        &expected_alignment
                    ))
                    .build())
            .collect::<Vec<Violation>>()
        })
        .flatten()
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md005() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# Inconsistent List Item Indentation

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
"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .message("Inconsistent indentation for list items at the same level. Expected 2 spaces, got 3 spaces")
                    .position(&Some(markdown::unist::Position::new(8, 3, 109, 9, 1, 130)))
                    .build(),
                violation_builder()
                    .message("Inconsistent indentation for list items at the same level. Expected 1 spaces, got 0 spaces")
                    .position(&Some(markdown::unist::Position::new(30, 1, 328, 30, 21, 348)))
                    .build(),
                violation_builder()
                    .message("Inconsistent indentation for list items at the same level. Expected 1 spaces, got 0 spaces")
                    .position(&Some(markdown::unist::Position::new(33, 1, 367, 33, 27, 393)))
                    .build()
            ],
            md005_consistent_list_items_indentation(&file)
        );
    }
}
