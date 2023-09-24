use glob::glob;
use std::fs;
use std::fmt;

#[derive(Debug)]
struct ReadMdFilesError;

impl fmt::Display for ReadMdFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

pub fn list(root: &String) -> Result<Vec<String>, String> {
    match fs::canonicalize(root) {
        Ok(path) => {
            match path.into_os_string().into_string() {
                Ok(path_str) => {
                    let pattern = format!("{}{}", &path_str, "/**/*.md");
                    let mut result = Vec::new();
                    for entry in glob(&pattern).expect("Failed to read glob pattern") {
                        match entry {
                            Ok(path) => result.push(path.display().to_string()),
                            Err(_e) => return Err("Cannot read glob".to_string()),
                        }
                    }
                    return Ok(result);
                },
                Err(_e) => return Err("Cannot convert path to string".to_string())
            }
        },
        Err(_e) => return Err("Cant to parse path".to_string())
    }
}