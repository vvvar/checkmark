/// Calculates the diff between two strings
/// and returns it as a string ready for printing
pub fn get_diff(a: &str, b: &str) -> String {
    let mut result = "".to_string();
    let diff = similar::TextDiff::from_lines(a, b);
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let (sign, style) = match change.tag() {
                similar::ChangeTag::Delete => ("-", console::Style::new().red()),
                similar::ChangeTag::Insert => ("+", console::Style::new().green()),
                similar::ChangeTag::Equal => ("", console::Style::new()),
            };
            if similar::ChangeTag::Equal == change.tag() {
                if !result.ends_with("\n\n") {
                    result.push_str("\n\n");
                }
            } else {
                result.push_str(&format!(
                    "{}{}",
                    style.apply_to(sign).bold(),
                    style.apply_to(change)
                ));
            }
        }
    }
    result
}

/// Removes trailing new-line and spaces from the end of the string
pub fn remove_trailing_newline_and_space(s: &str) -> String {
    let mut result = String::from(s);
    while result.ends_with('\n') || result.ends_with(' ') {
        if let Some(s) = result.strip_suffix('\n') {
            result = s.to_string()
        }
        if let Some(s) = result.strip_suffix(' ') {
            result = s.to_string()
        }
    }
    result
}
