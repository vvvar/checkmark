use async_std::stream::StreamExt;
use common::{Config, MarkDownFile};
use lychee_lib::{Collector, Input, InputSource::*, Request, Result};
use markdown::{to_html_with_options, Options};
use regex::Regex;
use std::collections::HashSet;
use std::collections::HashMap;
use std::string::String;
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

pub async fn render(files: &Vec<MarkDownFile>, config: &Config) {
    // Pre-build themes
    let mut themes = HashMap::<String, String>::new();
    themes.insert(
        "black".to_string(),
        include_str!("themes/black.css").to_string(),
    );
    themes.insert(
        "book".to_string(),
        include_str!("themes/book.css").to_string(),
    );
    themes.insert(
        "classic".to_string(),
        include_str!("themes/classic.css").to_string(),
    );
    themes.insert(
        "funky".to_string(),
        include_str!("themes/funky.css").to_string(),
    );
    themes.insert(
        "gfm".to_string(),
        include_str!("themes/gfm.css").to_string(),
    );
    themes.insert(
        "grayscale".to_string(),
        include_str!("themes/grayscale.css").to_string(),
    );
    themes.insert(
        "newspaper".to_string(),
        include_str!("themes/newspaper.css").to_string(),
    );
    themes.insert(
        "paper".to_string(),
        include_str!("themes/paper.css").to_string(),
    );
    themes.insert(
        "publication".to_string(),
        include_str!("themes/publication.css").to_string(),
    );
    themes.insert(
        "tiny".to_string(),
        include_str!("themes/tiny.css").to_string(),
    );
    themes.insert(
        "typewriter".to_string(),
        include_str!("themes/typewriter.css").to_string(),
    );
    themes.insert(
        "whiteboard".to_string(),
        include_str!("themes/whiteboard.css").to_string(),
    );
    // 1. Ensure output dir exists and it's fresh
    let cwd = current_dir().unwrap();
    let output_dir = match &config.rendering.output {
        Some(output_dir) => PathBuf::from(output_dir),
        None => cwd.join("output"),
    };
    remove_dir_all(&output_dir).ok();
    create_dir_all(&output_dir).ok();
    // 2. Find all associated files(images, assets, etc) and copy them to output dir
    //    Preserve the directory structure
    copy_associated_files(&collect_associated_files(&files).await, &output_dir);
    // 3. Render markdown files to html
    //    Preserve the directory structure
    use rayon::prelude::*;
    files.par_iter().for_each(|file| {
        // 5. Calculate path to output file
        //    cwd + output_dir + file path relative to cwd
        //    Change ext from ".md" to ".html"
        let mut out_file_path =
            Path::new(&output_dir).join(Path::new(&file.path).strip_prefix(&cwd).unwrap());
        out_file_path.set_extension("html");
        // 6. Ensure dir tree exist and finally write the file
        create_dir_all(&out_file_path.parent().unwrap()).ok();
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
        .replace(".md", ".html");
        use html_editor::operation::*;
        let css = match &config.rendering.theme {
            Some(theme) => match themes.get(theme) {
                Some(css) => html_editor::Node::Text(css.to_string()),
                None => html_editor::Node::Text(include_str!("themes/typewriter.css").to_string()),
            },
            None => html_editor::Node::Text(include_str!("themes/typewriter.css").to_string()),
        };
        let style: html_editor::Node = html_editor::Node::new_element("style", vec![], vec![css]);
        let head: html_editor::Node = html_editor::Node::new_element("head", vec![], vec![style]);
        let content = html_editor::Node::Text(html);
        let body: html_editor::Node = html_editor::Node::new_element("body", vec![], vec![content]);
        let document = html_editor::Node::new_element("html", vec![], vec![head, body]);
        write(&out_file_path, &document.html()).unwrap();
    });
    if config.rendering.serve {
        println!("Serving files from {}", output_dir.display());
        println!("Open http://localhost:8000 in your browser. Press Ctrl+C to stop.");
        open::that("http://localhost:8000").ok();
        rouille::start_server("localhost:8000", move |request| {
            let response = rouille::match_assets(&request, &output_dir);
            if response.is_success() {
                return response;
            } else {
                let default_file = Path::new("/").join("README.html");
                return rouille::Response::redirect_302(default_file.display().to_string());
            }
        });
    }
}
