use glob::{glob, GlobError};
use std::path::PathBuf;

pub fn list(root: &String) -> Result<Vec<String>, GlobError> {
    let absolute_root_path = PathBuf::from(&root)
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let pattern = format!("{}{}", &absolute_root_path, "/**/*.md");
    let mut results = Vec::<String>::new();
    for entry in glob(&pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => results.push(path.display().to_string()),
            Err(e) => println!("{:?}", e),
        }
    }
    return Ok(results);
}
