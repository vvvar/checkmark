use async_std::stream::StreamExt;
use common::MarkDownFile;
use lychee_lib::{Collector, Input, InputSource::*, Request, Result};
use markdown::{to_html_with_options, Options};
use regex::Regex;
use std::collections::HashSet;
use std::env::current_dir;
use std::fs::{create_dir_all, remove_dir_all, write};
use std::path::{Path, PathBuf};

/// Collect links from file
pub async fn collect_associated_files(files: &Vec<MarkDownFile>) -> Vec<PathBuf> {
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
                uri = match uri.strip_prefix("/") {
                    Some(stripped_uri) => stripped_uri.to_string(),
                    None => uri,
                };
            }
            uri
        })
        .filter(|link| dunce::canonicalize(&link).is_ok())
        .map(|link| dunce::canonicalize(&link).unwrap())
        .collect()
}

fn copy_associated_files(files: &Vec<PathBuf>, output_dir: &PathBuf) {
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

pub async fn render(files: &Vec<MarkDownFile>) {
    // 1. Ensure output dir exists and it's fresh
    let cwd = current_dir().unwrap();
    let output_dir = Path::new(&cwd).join("output");
    remove_dir_all(&output_dir).ok();
    create_dir_all(&output_dir).ok();
    // 2. Find all associated files(images, assets, etc) and copy them to output dir
    //    Preserve the directory structure
    copy_associated_files(&collect_associated_files(&files).await, &output_dir);
    // 3. Render markdown files to html
    //    Preserve the directory structure
    for file in files {
        let html = to_html_with_options(
            &file.content,
            &Options {
                compile: markdown::CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..markdown::CompileOptions::gfm()
                },
                ..markdown::Options::gfm()
            },
        )
        .expect("Unable to parse Markdown file")
        // 4. Replace .md with .html in links
        //    We are going to save them as .html
        .replace(".md", ".html");
        // 5. Calculate path to output file
        //    cwd + output_dir + file path relative to cwd
        //    Change ext from ".md" to ".html"
        let mut out_file_path =
            Path::new(&output_dir).join(Path::new(&file.path).strip_prefix(&cwd).unwrap());
        out_file_path.set_extension("html");
        // 6. Ensure dir tree exist and finally write the file
        create_dir_all(&out_file_path.parent().unwrap()).ok();
        write(&out_file_path, &html).unwrap();
    }
}
