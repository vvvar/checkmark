use crate::violation::{Violation, ViolationBuilder};
use common::{parse, MarkDownFile};
use markdown::mdast::Node;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD007")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md007.md")
        .is_fmt_fixable(true)
}

/// Takes a lin and calculates number of spaces before the first non-space character
/// while stripping block quote chars
fn calculate_ident(line: &str) -> usize {
    let mut line_without_prefix = line;
    while line_without_prefix.strip_prefix("> ").is_some() {
        line_without_prefix = line_without_prefix.strip_prefix("> ").unwrap();
    }

    let mut ident: usize = 0;
    for char in line_without_prefix.chars() {
        if char.eq(&' ') {
            ident += 1;
        } else {
            break;
        }
    }

    ident
}

/// Recursively analyze a list and all its children
/// considering depth and indentation
fn analyze_list(
    violations: &mut Vec<Violation>,
    node: &Node,
    nesting_level: usize,
    file: &MarkDownFile,
    expected_indent_per_level: usize,
) {
    match node {
        Node::ListItem(li) => {
            let num_line = li.position.as_ref().unwrap().start.line;
            if let Some(line) = file.content.lines().nth(num_line - 1) {
                let expected_ident = (nesting_level - 1) * expected_indent_per_level;
                let actual_ident = calculate_ident(line);
                if actual_ident.ne(&expected_ident) {
                    violations.push(
                        violation_builder()
                            .message(&format!(
                                "Wrong indentation of unordered list item. Expected {} spaces, got {} spaces",
                                expected_ident,
                                actual_ident
                            ))
                            .position(&li.position)
                            .build(),
                    );
                }
                for child in &li.children {
                    analyze_list(
                        violations,
                        child,
                        nesting_level,
                        file,
                        expected_indent_per_level,
                    );
                }
            }
        }
        Node::List(l) => {
            for child in &l.children {
                analyze_list(
                    violations,
                    child,
                    nesting_level + 1,
                    file,
                    expected_indent_per_level,
                );
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    analyze_list(
                        violations,
                        child,
                        nesting_level,
                        file,
                        expected_indent_per_level,
                    );
                }
            }
        }
    }
}

pub fn md007_unordered_list_indentation(file: &MarkDownFile, indent: usize) -> Vec<Violation> {
    log::debug!("[MD007] File: {:#?}", &file.path);

    let ast = parse(&file.content).unwrap();

    // Extract all root-level lists
    let mut top_level_lists: Vec<&Node> = vec![];
    let mut stack: Vec<&Node> = vec![];
    stack.push(&ast);
    while let Some(current) = stack.pop() {
        if let Node::List(_) = current {
            top_level_lists.push(current);
        } else if let Some(children) = current.children() {
            for child in children.iter().rev() {
                stack.push(child);
            }
        }
    }

    let mut violations: Vec<Violation> = vec![];
    for list in top_level_lists {
        analyze_list(&mut violations, list, 0, file, indent);
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md007() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# Wrong List Item Indentation

## Plain nested list

- One
- Two
  - Two-One
  - Two-Two
     - Two-Two-One
    - Two-Two-Two
- Three

## Nested list in block quote

> - One
> - Two
>   - Two-One
> - Three
> - Four
>  - Four-One

## Nested list in nested block quote

> - One
> - Two
>  - Two-One
> - Three
> > - One
> > - Two
> >  - Two-One
> > - Three

"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .message(
                        "Wrong indentation of unordered list item. Expected 4 spaces, got 5 spaces"
                    )
                    .position(&Some(Position::new(9, 5, 93, 9, 19, 107)))
                    .build(),
                violation_builder()
                    .message(
                        "Wrong indentation of unordered list item. Expected 0 spaces, got 1 spaces"
                    )
                    .position(&Some(Position::new(20, 3, 217, 20, 14, 228)))
                    .build(),
                violation_builder()
                    .message(
                        "Wrong indentation of unordered list item. Expected 0 spaces, got 1 spaces"
                    )
                    .position(&Some(Position::new(26, 3, 286, 26, 13, 296)))
                    .build(),
                violation_builder()
                    .message(
                        "Wrong indentation of unordered list item. Expected 0 spaces, got 1 spaces"
                    )
                    .position(&Some(Position::new(30, 5, 331, 30, 15, 341)))
                    .build()
            ],
            md007_unordered_list_indentation(&file, 2)
        );
    }
}
