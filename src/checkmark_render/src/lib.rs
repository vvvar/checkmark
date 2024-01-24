use async_std::stream::StreamExt;
use common::MarkDownFile;
use log::debug;
use lychee_lib::{Collector, Input, InputSource::*, Request, Result};
use markdown::to_html;
use regex::Regex;
use std::collections::HashSet;
use std::env::current_dir;
use std::fs::{create_dir_all, remove_dir_all, write};
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

/// Collect links from file
pub async fn collect_associated_files(files: &Vec<MarkDownFile>) -> Vec<String> {
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
    debug!("De-duplicated links:\n{:#?}", &links_set);
    links_set
        .into_iter()
        .filter(|link| {
            let re = Regex::new(r".*#.+$").unwrap();
            let url = link.uri.as_str();
            link.uri.is_file() && !re.is_match(url)
        })
        .map(|link| {
            let mut uri = link.uri.as_str().to_string();
            debug!("Associated file uri: {:#?}", &uri);
            uri = match uri.strip_prefix("file://") {
                Some(stripped_uri) => stripped_uri.to_string(),
                None => uri,
            };
            debug!("Associated file uri stripped: {:#?}", &uri);
            dunce::canonicalize(&uri)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

fn copy_associated_files(files: &Vec<String>, output_dir: &str) {
    debug!("Copying associated files: {:#?}", &files);

    let cwd = current_dir().unwrap();
    debug!("Current working directory: {:#?}", &cwd);

    for file in files {
        debug!("Copying associated file: {:#?}", &file);

        let cwd_str = cwd.clone().into_os_string().to_str().unwrap().to_string();
        debug!("Current working directory srt: {:#?}", &cwd_str);

        let relative_file_path = file.strip_prefix(&cwd_str).unwrap();
        debug!("Relative file path: {:#?}", &relative_file_path);

        let output_file_path = format!("{}{}", &output_dir, &relative_file_path);
        debug!("Output file path: {:#?}", &output_file_path);

        let pb = PathBuf::from_str(&output_file_path).unwrap();
        let output_file_parent_dir_path = pb.parent().unwrap();
        debug!(
            "Output file parent dir path: {:#?}",
            &output_file_parent_dir_path
        );
        // if let Ok(_) = remove_dir_all(&output_file_parent_dir_path) {}
        if let Ok(_) = create_dir_all(&output_file_parent_dir_path) {}
        std::fs::copy(&file, &output_file_path).unwrap();
    }
}

pub async fn render(files: &Vec<MarkDownFile>) {
    let output_dir = PathBuf::from_str(".")
        .unwrap()
        .join("out")
        .to_str()
        .unwrap()
        .to_string();
    debug!("Rendering to {}", &output_dir);
    remove_dir_all(&output_dir).unwrap();
    create_dir_all(&output_dir).unwrap();

    let associated_files = collect_associated_files(&files).await;
    debug!("Associated files: {:#?}", &associated_files);
    copy_associated_files(&associated_files, &output_dir);

    for file in files {
        let html = to_html(&file.content).replace(".md", ".html");
        let cwd = current_dir().unwrap();
        let relative_file_path = file
            .path
            .strip_prefix(cwd.into_os_string().to_str().unwrap())
            .unwrap()
            .strip_suffix(".md")
            .unwrap();
        debug!("Relative file path: {:#?}", &relative_file_path);

        let output_file_path = format!("{}{}.html", &output_dir, &relative_file_path);
        debug!("Output file path: {:#?}", &output_file_path);

        create_dir_all(
            &PathBuf::from_str(&output_file_path)
                .unwrap()
                .parent()
                .unwrap(),
        )
        .unwrap();
        write(&output_file_path, &html).unwrap();
    }
}
