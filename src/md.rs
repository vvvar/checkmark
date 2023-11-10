use glob;
use log::warn;

/// Creates a list of markdown files from provided path
/// Path could be:
///     1. path to a file - will just add this file to the list
///     2. path to a dir - will lookup all markdown files in this ir
pub fn list(path: &String) -> Vec<String> {
    let mut files = Vec::<String>::new();
    if let Ok(absolute_root_path) = std::path::PathBuf::from(&path).canonicalize() {
        if let Some(absolute_root_path_str) = absolute_root_path.to_str() {
            if absolute_root_path.is_file() {
                // Someone requested just a single file
                files.push(String::from(absolute_root_path_str));
            } else if absolute_root_path.is_dir() {
                // Someone provided just a plain path to dir
                match glob::glob(&format!("{}{}", &absolute_root_path_str, "/**/*.md")) {
                    Ok(search_results) => {
                        for search_result in search_results {
                            match search_result {
                                Ok(markdown_file_path) => match markdown_file_path.canonicalize() {
                                    Ok(markdown_file_abs_path) => {
                                        files.push(markdown_file_abs_path.display().to_string())
                                    }
                                    Err(error) => warn!(
                                        "Cannot obtain an absolute path to found file, error: {}",
                                        &error
                                    ),
                                },
                                Err(error) => {
                                    warn!("Found a Markdown file, but it had an error: {}", &error)
                                }
                            }
                        }
                    }
                    Err(error) => warn!("Unable to perform a glob search, error: {}", &error),
                }
            } else {
                warn!("Unable to collect markdown files: path is neither a file nor a dir");
            }
        } else {
            warn!("Unable to parse path to a file");
            warn!("Make sure the path to a file is correct");
        }
    } else {
        warn!("Unable to read root/file path. Make sure you are providing either a valid path(absolute/relative), glob or filename as a first argument");
    }
    return files;
}
