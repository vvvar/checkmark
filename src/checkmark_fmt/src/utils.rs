/// Calculates the diff between two strings
/// and returns it as a string ready for printing
pub fn get_diff(a: &str, b: &str) -> String {
    prettydiff::diff_lines(a, b).to_string()
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
