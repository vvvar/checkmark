use lychee_lib::Result;
use lychee_lib::Collector;
use lychee_lib::Input;
use crate::checker::Issue;
use std::path::PathBuf;
use async_std::stream::StreamExt;
use lychee_lib::Status;
use std::fs;
use lychee_lib::ErrorKind;
use std::collections::HashMap;
use std::path::Path;

async fn collect_links(path: &str) -> Result<HashMap<String, lychee_lib::Request>> {
    // Collect links from file
    let input = Input{
        source: lychee_lib::InputSource::FsPath(PathBuf::from(path)),
        file_type_hint: None,
        excluded_paths: None
    };
    let links = Collector::new(None) // base
        .skip_missing_inputs(false) // don't skip missing inputs? (default=false)
        .use_html5ever(false) // use html5ever for parsing? (default=false)
        .include_verbatim(true)
        .collect_links(Vec::<Input>::from([input])) // base url or directory
        .await
        .collect::<Result<Vec<_>>>()
        .await?;
    // Dedup them
    let mut links_map: HashMap<String, lychee_lib::Request> = HashMap::new();
    for link in links {
        links_map.insert(link.uri.to_string(), link);
    }
    // Return result
    Ok(links_map)
}

// Find all network links in file which failed with network request and convert them to the list of issues
fn network_error_to_issues(source_file_path: &str, error: &reqwest::Error) -> Vec<Issue> {
    let mut issues = Vec::<Issue>::new();
    let source_file_content = fs::read_to_string(source_file_path)
                                            .expect("Unable to read a source file to figure out which link failed exactly");
    let url = error.url()
                        .expect("Unable to get which URL failed with Network Error")
                        .as_str()
                        .strip_suffix("/")
                        .expect("Unable to strip a strip the suffix from URL");
    for (line_number, line_content) in source_file_content.lines().enumerate() {
        if line_content.contains(&url) {
            issues.push(Issue {
                id: String::from("MD002"),
                file_path: format!("{}:{}.{}", &source_file_path, &line_number + 1, line_content.find(&url).unwrap() + 1),
                category: String::from("Link/URL"),
                description: format!("{}: {}", &url, &error),
                issue_in_code: None,
                suggestions: vec![
                    String::from("Can you open this link in a browser? If no then perhaps its broken"),
                    String::from("Is there internet connection?"),
                    String::from("Are you using proxy? Consider setting HTTP_PROXY and/or HTTPS_PROXY env variables")
                ]
            });
        }
    }
    return issues;
}

// Find all places in source_file_path where file_path is mentioned and convert to list of issues
fn invalid_file_error_to_issues(source_file_path: &str, unreachable_file_path: &str) -> Vec<Issue> {
    let mut issues = Vec::<Issue>::new();
    let source_file_content = fs::read_to_string(source_file_path)
                                            .expect("Unable to read a source file to figure out which link failed exactly");
    let unreachable_filename = Path::new(unreachable_file_path).file_name().unwrap().to_str().unwrap();
    for (line_number, line_content) in source_file_content.lines().enumerate() {
        if line_content.contains(&unreachable_filename) {
            issues.push(Issue {
                id: String::from("MD002"),
                file_path: format!("{}:{}.{}", &source_file_path, &line_number + 1, line_content.find(&unreachable_filename).unwrap() + 1),
                category: String::from("Link/URL"),
                description: format!("File is unreachable: {}", &unreachable_filename),
                issue_in_code: None,
                suggestions: vec![
                    String::from("Does this file really exist?"),
                    String::from("Does it referenced correctly? Often such issues appear when relative path is used, for ex. '../file.md' will expect file one directory above")
                ]
            });
        }
    }
    return issues;
}

#[allow(unused_variables, unused_mut)]
pub async fn check(path: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::<Issue>::new();
    for (uri, request) in collect_links(&path).await? {
        match lychee_lib::check(request).await?.status() {
            // Request was successful
            Status::Ok(status_code) => {},
            // Failed request
            Status::Error(error_kind) => match error_kind {
                    // Network error while handling request
                    ErrorKind::NetworkRequest(error) => issues.append(&mut network_error_to_issues(&path, &error)),
                    // Cannot read the body of the received response
                    ErrorKind::ReadResponseBody(error) => {},
                    // The network client required for making requests cannot be created
                    ErrorKind::BuildRequestClient(error) => {},
                    // Network error while using Github API
                    ErrorKind::GithubRequest(error) => {},
                    // Error while executing a future on the Tokio runtime
                    ErrorKind::RuntimeJoin(error) => {},
                    // Error while converting a file to an input
                    ErrorKind::ReadFileInput(error, file) => {},
                    // Error while reading stdin as input
                    ErrorKind::ReadStdinInput(error) => {},
                    // Errors which can occur when attempting to interpret a sequence of u8 as a string
                    ErrorKind::Utf8(error) => {},
                    // The Github client required for making requests cannot be created
                    ErrorKind::BuildGithubClient(error) => {},
                    // Invalid Github URL
                    ErrorKind::InvalidGithubUrl(invalid_github_url) => {},
                    // The input is empty and not accepted as a valid URL
                    ErrorKind::EmptyUrl => {},
                    // The given string can not be parsed into a valid URL, e-mail address, or file path
                    ErrorKind::ParseUrl(error, url) => {},
                    // The given URI cannot be converted to a file path
                    ErrorKind::InvalidFilePath(uri) => issues.append(&mut invalid_file_error_to_issues(&path, &uri.path())),
                    // The given path cannot be converted to a URI
                    ErrorKind::InvalidUrlFromPath(uri_path) => {},
                    // The given mail address is unreachable
                    ErrorKind::UnreachableEmailAddress(uri, email) => {},
                    // The given header could not be parsed.
                    ErrorKind::InvalidHeader(header) => {},
                    // The given string can not be parsed into a valid base URL or base directory
                    ErrorKind::InvalidBase(base_dir, url) => {},
                    // The given input can not be parsed into a valid URI remapping
                    ErrorKind::InvalidUrlRemap(url) => {},
                    // The given path does not resolve to a valid file
                    ErrorKind::FileNotFound(file_path) => {},
                    // Error while traversing an input directory
                    ErrorKind::DirTraversal(error) => {},
                    // The given glob pattern is not valid
                    ErrorKind::InvalidGlobPattern(glob_error) => {},
                    // The Github API could not be called because of a missing Github token.
                    ErrorKind::MissingGitHubToken => {},
                    // Used an insecure URI where a secure variant was reachable
                    ErrorKind::InsecureURL(uri) => {},
                    // Error while sending/receiving messages from MPSC channel
                    ErrorKind::Channel(send_error) => {},
                    // An URL with an invalid host was found
                    ErrorKind::InvalidUrlHost => {},
                    // Cannot parse the given URI
                    ErrorKind::InvalidURI(uri) => {},
                    // The given status code is invalid (not in the range 100-1000)
                    ErrorKind::InvalidStatusCode(status_code) => {},
                    // Regex error
                    ErrorKind::Regex(regex_error) => {},
                    // Too many redirects (HTTP 3xx) were encountered (configurable)
                    ErrorKind::TooManyRedirects(error) => {},
                    // WTF???
                    &_ => {}
            },
            // Request timed out
            Status::Timeout(status_code) => {},
            // Got redirected to different resource
            Status::Redirected(status_code) => {},
            // The given status code is not known by lychee
            Status::UnknownStatusCode(status_code) => {},
            // Resource was excluded from checking
            Status::Excluded => {},
            // The request type is currently not supported
            Status::Unsupported(error_kind) => {},
            // Cached request status from previous run
            Status::Cached(cache_status) => {}
        };
    }
    return Ok(issues);
}