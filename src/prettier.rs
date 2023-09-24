use std::fs;
use js_sandbox::{Script, AnyError};

pub fn format(md: &String) -> String {
    match Script::from_file("/Users/vvoinov/Documents/repos/md-checker/src/js/bundle.js") {
        Ok(mut script) => match script.call("format_markdown", (md,)) {
            Ok(formatted) => return formatted,
            Err(_e) => return String::from(md)
        }
        Err(_e) => return String::from(md)
    }
}

pub fn check_format(path: &String) -> Result<bool, AnyError> {
    println!("Checking format of {:?}...", path);
    let original = fs::read_to_string(path)?;
    let formatted = format(&original);
    return Ok(original.eq(&formatted));
}