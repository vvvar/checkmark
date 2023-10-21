use std::fs;
use js_sandbox::{Script, AnyError};

use crate::checker::Issue;

pub fn format(md: &String) -> String {
    match Script::from_file("/Users/vvoinov/Documents/repos/md-checker/src/js/bundle.js") {
        Ok(mut script) => match script.call("format_markdown", (md,)) {
            Ok(formatted) => return formatted,
            Err(_e) => return String::from(md)
        }
        Err(_e) => return String::from(md)
    }
}

pub fn check_format(path: &String) -> Result<Vec<Issue>, AnyError> {
    let mut issues = Vec::<Issue>::new();
    let original = fs::read_to_string(path)?;
    let formatted = format(&original);
    if !original.eq(&formatted) {
        issues.push(Issue {
            id: String::from("MD001"),
            file_path: String::from(path),
            category: String::from("Format"),
            description: String::from("File has a wrong formatting"),
            suggestions: vec![
                String::from("Please autoformat the file")
            ]
        });
    }
    return Ok(issues);
}