use auth_git2::GitAuthenticator;
use log::warn;
use path_slash::PathExt as _;
use std::borrow::Cow;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wildmatch::WildMatch;

/// Returns a path to a tmp dir based on input URI
fn tmp_dir(uri: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir()
        .join("checkmark")
        .join(std::path::Path::new(uri).file_stem().unwrap());
    if path.exists() {
        log::debug!("Directory {:#?} already exists, removing", &path);
        std::fs::remove_dir_all(&path).unwrap();
    }
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Creates a list of markdown files from provided path
/// Path could be:
///     1. path to a file - will just add this file to the list
///     2. path to a dir - will lookup all markdown files in this ir
///     3. remote URL
pub async fn ls(
    path: &str,
    exclude: &Vec<String>,
    tui: &Arc<Mutex<common::tui::CheckProgressTUI>>,
) -> Vec<common::MarkDownFile> {
    log::debug!("Listing Markdown files in: {:#?}", &path);

    let mut input_path = path.to_owned();

    if input_path.ends_with(".git") {
        log::debug!("Path is a git repo, cloning into tmp dir");
        tui.lock()
            .unwrap()
            .start_spinner(&format!("Cloning git repo {path}"));

        let auth = GitAuthenticator::default();
        let git_config = git2::Config::open_default().unwrap();

        let mut cb = git2::RemoteCallbacks::new();
        cb.transfer_progress(|stats| {
            log::trace!(
                "transfer_progress callback, stats.indexed_deltas(): {}",
                &stats.indexed_deltas()
            );
            log::trace!(
                "transfer_progress callback, stats.indexed_objects(): {}",
                &stats.indexed_objects()
            );
            log::trace!(
                "transfer_progress callback, stats.received_bytes(): {}",
                &stats.received_bytes()
            );
            log::trace!(
                "transfer_progress callback, stats.received_objects(): {}",
                &stats.received_objects()
            );
            true
        });
        cb.credentials(auth.credentials(&git_config));

        let mut co = git2::build::CheckoutBuilder::new();
        co.progress(|path, cur, total| {
            if let Some(path) = path {
                log::trace!("progress callback, path: {}", &path.display());
            }
            log::trace!("progress callback, cur: {}", &cur);
            log::trace!("progress callback, total: {}", &total);
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(cb);

        let tmp_dir = tmp_dir(&input_path);

        log::debug!("Cloning into the {:#?}", &tmp_dir);
        git2::build::RepoBuilder::new()
            .fetch_options(fetch_options)
            .with_checkout(co)
            .clone(&input_path, std::path::Path::new(&tmp_dir))
            .unwrap();
        log::debug!("Cloned {:#?} into the {:#?}", &input_path, &tmp_dir);
        input_path = tmp_dir.to_str().unwrap().to_owned();
    } else if is_url::is_url(&input_path) {
        log::debug!("Path is a plain URL, downloading as single file into tmp dir");

        let response = reqwest::get(&input_path).await.unwrap();

        let tmp_file_path = tmp_dir(&input_path).join(format!(
            "{}.md",
            std::path::Path::new(&input_path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
        ));
        log::debug!("Will download into a file: {:#?}", &tmp_file_path);

        let mut file = std::fs::File::create(&tmp_file_path).unwrap();
        let mut content = std::io::Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).unwrap();

        input_path = tmp_file_path.to_str().unwrap().to_owned();
    }

    let mut files = Vec::<String>::new();
    if let Ok(absolute_root_path) = dunce::canonicalize(&input_path) {
        log::debug!("Absolute path: {:#?}", &absolute_root_path);
        if let Some(absolute_root_path_str) = absolute_root_path.to_str() {
            if absolute_root_path.is_file() {
                // Someone requested just a single file
                log::debug!("Path is a single file");
                files.push(String::from(absolute_root_path_str));
            } else if absolute_root_path.is_dir() {
                // Someone provided just a plain path to dir
                log::debug!("Path is a dir");

                let glob_pattern = std::path::Path::new(absolute_root_path_str)
                    .join("**")
                    .join("*.md")
                    .to_str()
                    .unwrap()
                    .to_owned();
                log::debug!("Searching files by glob pattern: {:#?}", &glob_pattern);

                match glob::glob(&glob_pattern) {
                    Ok(search_results) => {
                        for search_result in search_results {
                            match search_result {
                                Ok(markdown_file_path) => {
                                    match dunce::canonicalize(markdown_file_path) {
                                        Ok(markdown_file_abs_path) => {
                                            files.push(markdown_file_abs_path.display().to_string())
                                        }
                                        Err(error) => warn!(
                                        "Cannot obtain an absolute path to found file, error: {}",
                                        &error
                                    ),
                                    }
                                }
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
    log::debug!("Collected list of files {:#?}", &files);

    let mut markdown_files: Vec<common::MarkDownFile> = vec![];
    for file_path in &files {
        match std::fs::read_to_string(file_path) {
            Ok(content) => markdown_files.push(common::MarkDownFile {
                path: file_path.clone(),
                content,
                issues: vec![],
            }),
            Err(_) => warn!("Unable to read file content. Make sure file has correct permissions"),
        }
    }

    // Filter files by exclude patterns
    markdown_files.retain(|markdown_file| {
        let unix_style_file_path = Path::new(&markdown_file.path)
            .to_slash()
            .unwrap_or(Cow::from(markdown_file.path.clone()));
        for exclude_pattern in exclude {
            if WildMatch::new(exclude_pattern).matches(&unix_style_file_path) {
                log::debug!("Ignoring {:#?}", &markdown_file.path);
                return false;
            }
        }
        true
    });

    markdown_files
}
