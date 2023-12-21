use lychee_lib::{Result};
use async_std::stream::StreamExt;

// Find all network links in file which failed with network request and convert them to the list of issues
fn network_error_to_issues(file: &mut common::MarkDownFile, error: &reqwest::Error) {
    let url = error
        .url()
        .expect("Unable to get which URL failed with Network Error")
        .as_str()
        .strip_suffix("/")
        .expect("Unable to strip a strip the suffix from URL");
    for (line_number, line_content) in file.content.lines().enumerate() {
        if line_content.contains(&url) {
            file.issues.push(
                common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::LinkChecking)
                    .set_severity(common::IssueSeverity::Warning)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(&line_number + 1)
                    .set_row_num_end(file.content.lines().count())
                    .set_col_num_start(line_content.find(&url).unwrap() + 1)
                    .set_col_num_end(1)
                    .set_offset_start(0)
                    .set_offset_end(file.content.len())
                    .set_message(format!("{}: {}", &url, &error))
                    .set_fixes(vec![
                        "Can you open this link in a browser? If no then perhaps its broken".to_string(),
                        "Is there internet connection?".to_string(),
                        "Are you using proxy? Consider setting HTTP_PROXY and/or HTTPS_PROXY env variables".to_string()
                    ])
                    .build()
            );
        }
    }
}

// Find all places in source_file_path where file_path is mentioned and convert to list of issues
fn invalid_file_error_to_issues(file: &mut common::MarkDownFile, unreachable_file_path: &str) {
    let unreachable_filename = std::path::Path::new(unreachable_file_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    for (line_number, line_content) in file.content.lines().enumerate() {
        if line_content.contains(&unreachable_filename) {
            file.issues.push(
                common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::LinkChecking)
                    .set_severity(common::IssueSeverity::Warning)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(&line_number + 1)
                    .set_row_num_end(file.content.lines().count())
                    .set_col_num_start(line_content.find(&unreachable_filename).unwrap() + 1)
                    .set_col_num_end(1)
                    .set_offset_start(0)
                    .set_offset_end(file.content.len())
                    .set_message(format!("File is unreachable: {}", &unreachable_filename))
                    .set_fixes(vec![
                        "Does this file really exist?".to_string(),
                        "Does it referenced correctly? Often such issues appear when relative path is used, for ex. '../file.md' will expect file one directory above".to_string()
                    ])
                    .build()
            );
        }
    }
}

/// Collect links from file
async fn collect_links(path: &str, ignored_uri_wildcards: &Vec<String>) -> lychee_lib::Result<std::collections::HashMap<String, lychee_lib::Request>> {
    let input = vec![lychee_lib::Input {
        source: lychee_lib::InputSource::FsPath(std::path::PathBuf::from(path)),
        file_type_hint: None,
        excluded_paths: None,
    }];
    let links = lychee_lib::Collector::new(None) // base
        .skip_missing_inputs(false) // don't skip missing inputs? (default=false)
        .use_html5ever(false) // use html5ever for parsing? (default=false)
        .include_verbatim(true)
        .collect_links(input)
        .await // base url or directory
        .collect::<Result<Vec<_>>>()
        .await?;
    // Dedup them
    let mut links_map: std::collections::HashMap<String, lychee_lib::Request> = std::collections::HashMap::new();
    for link in links {
        let uri = link.uri.as_str();
        let matches_any_ignored_uri_wildcard =
            ignored_uri_wildcards.iter().any(|ignored_wildcard| {
                if let Some(stripped_uri) = uri.strip_suffix("/") {
                    wildmatch::WildMatch::new(&ignored_wildcard).matches(&stripped_uri)
                } else {
                    wildmatch::WildMatch::new(&ignored_wildcard).matches(&uri)
                }
            });
        if !matches_any_ignored_uri_wildcard {
            links_map.insert(uri.to_string(), link.clone());
        }
    }
    // Return result
    Ok(links_map)
}

pub async fn check_links(file: &mut common::MarkDownFile, ignored_uri_wildcards: &Vec<String>) {
    for (uri, request) in collect_links(&file.path, &ignored_uri_wildcards).await.unwrap() {
        match lychee_lib::check(request).await.unwrap().status() {
            // Request was successful
            lychee_lib::Status::Ok(status_code) => {}
            // Failed request
            lychee_lib::Status::Error(error_kind) => match error_kind {
                // Network error while handling request
                lychee_lib::ErrorKind::NetworkRequest(error) => network_error_to_issues(file, &error),
                // Cannot read the body of the received response
                lychee_lib::ErrorKind::ReadResponseBody(error) => {}
                // The network client required for making requests cannot be created
                lychee_lib::ErrorKind::BuildRequestClient(error) => {}
                // Network error while using Github API
                lychee_lib::ErrorKind::GithubRequest(error) => {}
                // Error while executing a future on the Tokio runtime
                lychee_lib::ErrorKind::RuntimeJoin(error) => {}
                // Error while converting a file to an input
                lychee_lib::ErrorKind::ReadFileInput(error, file) => {}
                // Error while reading stdin as input
                lychee_lib::ErrorKind::ReadStdinInput(error) => {}
                // Errors which can occur when attempting to interpret a sequence of u8 as a string
                lychee_lib::ErrorKind::Utf8(error) => {}
                // The Github client required for making requests cannot be created
                lychee_lib::ErrorKind::BuildGithubClient(error) => {}
                // Invalid Github URL
                lychee_lib::ErrorKind::InvalidGithubUrl(invalid_github_url) => {}
                // The input is empty and not accepted as a valid URL
                lychee_lib::ErrorKind::EmptyUrl => {}
                // The given string can not be parsed into a valid URL, e-mail address, or file path
                lychee_lib::ErrorKind::ParseUrl(error, url) => {}
                // The given URI cannot be converted to a file path
                lychee_lib::ErrorKind::InvalidFilePath(uri) => invalid_file_error_to_issues(file, &uri.path()),
                // The given path cannot be converted to a URI
                lychee_lib::ErrorKind::InvalidUrlFromPath(uri_path) => {}
                // The given mail address is unreachable
                lychee_lib::ErrorKind::UnreachableEmailAddress(uri, email) => {}
                // The given header could not be parsed.
                lychee_lib::ErrorKind::InvalidHeader(header) => {}
                // The given string can not be parsed into a valid base URL or base directory
                lychee_lib::ErrorKind::InvalidBase(base_dir, url) => {}
                // The given input can not be parsed into a valid URI remapping
                lychee_lib::ErrorKind::InvalidUrlRemap(url) => {}
                // The given path does not resolve to a valid file
                lychee_lib::ErrorKind::FileNotFound(file_path) => {}
                // Error while traversing an input directory
                lychee_lib::ErrorKind::DirTraversal(error) => {}
                // The given glob pattern is not valid
                lychee_lib::ErrorKind::InvalidGlobPattern(glob_error) => {}
                // The Github API could not be called because of a missing Github token.
                lychee_lib::ErrorKind::MissingGitHubToken => {}
                // Used an insecure URI where a secure variant was reachable
                lychee_lib::ErrorKind::InsecureURL(uri) => {}
                // Error while sending/receiving messages from MPSC channel
                lychee_lib::ErrorKind::Channel(send_error) => {}
                // An URL with an invalid host was found
                lychee_lib::ErrorKind::InvalidUrlHost => {}
                // Cannot parse the given URI
                lychee_lib::ErrorKind::InvalidURI(uri) => {}
                // The given status code is invalid (not in the range 100-1000)
                lychee_lib::ErrorKind::InvalidStatusCode(status_code) => {}
                // Regex error
                lychee_lib::ErrorKind::Regex(regex_error) => {}
                // Too many redirects (HTTP 3xx) were encountered (configurable)
                lychee_lib::ErrorKind::TooManyRedirects(error) => {}
                // WTF???
                &_ => {}
            },
            // Request timed out
            lychee_lib::Status::Timeout(status_code) => {}
            // Got redirected to different resource
            lychee_lib::Status::Redirected(status_code) => {}
            // The given status code is not known by lychee
            lychee_lib::Status::UnknownStatusCode(status_code) => {}
            // Resource was excluded from checking
            lychee_lib::Status::Excluded => {}
            // The request type is currently not supported
            lychee_lib::Status::Unsupported(error_kind) => {}
            // Cached request status from previous run
            lychee_lib::Status::Cached(cache_status) => {}
        };
    }
}