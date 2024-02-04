use async_std::stream::StreamExt;
use common::MarkDownFile;
use lychee_lib::{Collector, Input, InputSource::*, Request, Result};
use regex::Regex;
use std::collections::HashSet;
use std::env::current_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

/// Collect all links from files
/// that are pointing to other files or directories
pub async fn collect(files: &[MarkDownFile]) -> Vec<PathBuf> {
    let input = files
        .iter()
        .map(|file| Input {
            source: FsPath(PathBuf::from(&file.path)),
            file_type_hint: Some(lychee_lib::FileType::Markdown),
            excluded_paths: None,
        })
        .collect::<Vec<_>>();
    let links = Collector::new(None) // base
        .skip_missing_inputs(true) // Valid pats are assumed
        .use_html5ever(false) // use html5gum, author claims it to be faster
        .include_verbatim(true) // verbatim is for ex. ```code``
        .collect_links(input)
        .collect::<Result<Vec<_>>>()
        .await
        .unwrap();
    // Dedup them
    let mut links_set = HashSet::<Request>::new();
    for link in links {
        links_set.insert(link.clone());
    }
    links_set
        .into_iter()
        .filter(|link| {
            let re = Regex::new(r".*#.+$").unwrap();
            let url = link.uri.as_str();
            link.uri.is_file() && !re.is_match(url)
        })
        .map(|link| {
            let mut uri = link.uri.as_str().to_string();
            uri = match uri.strip_prefix("file://") {
                Some(stripped_uri) => stripped_uri.to_string(),
                None => uri,
            };
            if os_info::get().os_type().eq(&os_info::Type::Windows) {
                uri = match uri.strip_prefix('/') {
                    Some(stripped_uri) => stripped_uri.to_string(),
                    None => uri,
                };
            }
            uri
        })
        .filter(|link| dunce::canonicalize(link).is_ok())
        .map(|link| dunce::canonicalize(link).unwrap())
        .collect()
}

// Copy files from one dir to another
// while preserving the directory structure
// based on diff between two paths
// Example:
//    source "checkmark/src/associated_files.rs"
//    dest   "checkmark/output/"
// This will be copied to "checkmark/output/src/associated_files.rs"
pub fn copy(files: &Vec<PathBuf>, output_dir: &PathBuf) {
    let cwd = current_dir().unwrap();
    for file in files {
        let out_file_path = Path::new(output_dir).join(file.strip_prefix(&cwd).unwrap());
        if file.is_file() {
            create_dir_all(out_file_path.parent().unwrap()).ok();
            std::fs::copy(file, out_file_path).ok();
        } else {
            fs_extra::dir::copy(file, out_file_path, &fs_extra::dir::CopyOptions::new()).ok();
        }
    }
}

// Convenience function
pub async fn collect_and_copy(files: &[MarkDownFile], dest: &PathBuf) {
    copy(&collect(files).await, dest);
}
