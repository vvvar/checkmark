use async_std::stream::StreamExt;
use lychee_lib::Result;

// Find all network links in file which failed with network request and convert them to the list of issues
fn network_error_to_issues(file: &mut common::MarkDownFile, error: &reqwest::Error) {
    log::debug!("Handling network error:\n{:#?}", &error);
    let url = match error.url() {
        Some(url) => match url.as_str().strip_suffix('/') {
            Some(stripped_url) => stripped_url,
            None => url.as_str(),
        },
        None => "",
    };
    log::debug!("Failed URL: {:#?}", &url);
    for (offset, matched_str) in file.content.match_indices(&url) {
        log::debug!("Found match by index: {:#?}", &offset);
        file.issues.push(
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::LinkChecking)
                .set_severity(common::IssueSeverity::Warning)
                .set_file_path(file.path.clone())
                .set_row_num_start(1)
                .set_row_num_end(file.content.lines().count())
                .set_col_num_start(1)
                .set_col_num_end(1)
                .set_offset_start(offset)
                .set_offset_end(offset + matched_str.len())
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

// Find all places in source_file_path where file_path is mentioned and convert to list of issues
fn invalid_file_error_to_issues(file: &mut common::MarkDownFile, unreachable_file_path: &str) {
    log::debug!(
        "Handling unreachable file error: {:#?}",
        &unreachable_file_path
    );
    let mut document_folder_path_buf = std::path::PathBuf::from(&file.path);
    document_folder_path_buf.pop();
    let document_folder_path = document_folder_path_buf.to_str().unwrap().to_string();
    log::debug!("Problematic document path: {:#?}", &document_folder_path);
    let file_path_in_document = match unreachable_file_path
        .replace(&document_folder_path, "")
        .strip_prefix('/')
    {
        Some(stripped) => stripped.to_string(),
        None => unreachable_file_path.replace(&document_folder_path, ""),
    };
    log::debug!(
        "Path as it is specified in document: {:#?}",
        &file_path_in_document
    );
    for (offset, matched_str) in file.content.match_indices(&file_path_in_document) {
        log::debug!("Found match by index: {:#?}", &offset);
        file.issues.push(
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::LinkChecking)
                .set_severity(common::IssueSeverity::Warning)
                .set_file_path(file.path.clone())
                .set_row_num_start(1)
                .set_row_num_end(file.content.lines().count())
                .set_col_num_start(1)
                .set_col_num_end(1)
                .set_offset_start(offset)
                .set_offset_end(offset + matched_str.len())
                .set_message(format!("File is unreachable: {}", &file_path_in_document))
                .set_fixes(vec![
                    "Does this file really exist?".to_string(),
                    "Does it referenced correctly? Often such issues appear when relative path is used, for ex. '../file.md' will expect file one directory above".to_string()
                ])
                .build()
        );
    }
}

/// Collect links from file
async fn collect_links(
    path: &str,
    ignored_uri_wildcards: &Vec<String>,
) -> lychee_lib::Result<std::collections::HashMap<String, lychee_lib::Request>> {
    log::debug!(
        "Collecting links in file: {:#?}, ignoring these links: {:#?}",
        &path,
        &ignored_uri_wildcards
    );
    let input = vec![lychee_lib::Input {
        source: lychee_lib::InputSource::FsPath(std::path::PathBuf::from(path)),
        file_type_hint: None,
        excluded_paths: None,
    }];
    log::debug!("Lychee inputs:\n{:#?}", &input);
    let links = lychee_lib::Collector::new(None) // base
        .skip_missing_inputs(false) // don't skip missing inputs? (default=false)
        .use_html5ever(false) // use html5ever for parsing? (default=false)
        .include_verbatim(true)
        .collect_links(input)
        .await // base url or directory
        .collect::<Result<Vec<_>>>()
        .await?;
    log::debug!("Found links:\n{:#?}", &links);
    // Dedup them
    let mut links_map: std::collections::HashMap<String, lychee_lib::Request> =
        std::collections::HashMap::new();
    for link in links {
        let uri = link.uri.as_str();
        let matches_any_ignored_uri_wildcard =
            ignored_uri_wildcards.iter().any(|ignored_wildcard| {
                if let Some(stripped_uri) = uri.strip_suffix('/') {
                    wildmatch::WildMatch::new(ignored_wildcard).matches(stripped_uri)
                } else {
                    wildmatch::WildMatch::new(ignored_wildcard).matches(uri)
                }
            });
        if !matches_any_ignored_uri_wildcard {
            links_map.insert(uri.to_string(), link.clone());
        }
    }
    log::debug!("De-duplicated links:\n{:#?}", &links_map);
    Ok(links_map)
}

pub async fn check_links(file: &mut common::MarkDownFile, ignored_uri_wildcards: &Vec<String>) {
    for (uri, request) in collect_links(&file.path, ignored_uri_wildcards)
        .await
        .unwrap()
    {
        match lychee_lib::check(request).await.unwrap().status() {
            // Request was successful
            lychee_lib::Status::Ok(status_code) => {
                log::debug!("{:#?} request OK with code {:#?}", &uri, &status_code);
            }
            // Failed request
            lychee_lib::Status::Error(error_kind) => {
                match error_kind {
                    // Network error while handling request
                    lychee_lib::ErrorKind::NetworkRequest(error) => {
                        log::debug!("{:#?} request network error:\n{:#?}", &uri, &error);
                        network_error_to_issues(file, error);
                    }
                    // Cannot read the body of the received response
                    lychee_lib::ErrorKind::ReadResponseBody(error) => {
                        log::debug!(
                            "{:#?} request OK, but unable to read response body, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // The network client required for making requests cannot be created
                    lychee_lib::ErrorKind::BuildRequestClient(error) => {
                        log::debug!(
                            "{:#?} request cant build client, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // Network error while using Github API
                    lychee_lib::ErrorKind::GithubRequest(error) => {
                        log::debug!(
                            "{:#?} request error while using GitHub API, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // Error while executing a future on the Tokio runtime
                    lychee_lib::ErrorKind::RuntimeJoin(error) => {
                        log::debug!("{:#?} request error while executing a future on the Tokio runtime, error:\n{:#?}", &uri, &error);
                    }
                    // Error while converting a file to an input
                    lychee_lib::ErrorKind::ReadFileInput(error, file) => {
                        log::debug!("{:#?} request error while converting a file {:#?} to an input, error:\n{:#?}", &uri, &file, &error);
                    }
                    // Error while reading stdin as input
                    lychee_lib::ErrorKind::ReadStdinInput(error) => {
                        log::debug!(
                            "{:#?} request error while reading stdin as input, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // Errors which can occur when attempting to interpret a sequence of u8 as a string
                    lychee_lib::ErrorKind::Utf8(error) => {
                        log::debug!(
                            "{:#?} request error while interpret a sequence of u8, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // The Github client required for making requests cannot be created
                    lychee_lib::ErrorKind::BuildGithubClient(error) => {
                        log::debug!(
                            "{:#?} request error cant create github client, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // Invalid Github URL
                    lychee_lib::ErrorKind::InvalidGithubUrl(invalid_github_url) => {
                        log::debug!(
                            "{:#?} request error invalid GitHub URL: {:#?}",
                            &uri,
                            &invalid_github_url
                        );
                    }
                    // The input is empty and not accepted as a valid URL
                    lychee_lib::ErrorKind::EmptyUrl => {
                        log::debug!("{:#?} request error empty URL", &uri);
                    }
                    // The given string can not be parsed into a valid URL, e-mail address, or file path
                    lychee_lib::ErrorKind::ParseUrl(error, url) => {
                        log::debug!(
                            "{:#?} request error cant parse URL, error:\n{:#?}",
                            &url,
                            &error
                        );
                    }
                    // The given URI cannot be converted to a file path
                    lychee_lib::ErrorKind::InvalidFilePath(uri) => {
                        log::debug!("{:#?} request error invalid file path", &uri);
                        invalid_file_error_to_issues(file, uri.path());
                    }
                    // The given path cannot be converted to a URI
                    lychee_lib::ErrorKind::InvalidUrlFromPath(uri_path) => {
                        log::debug!("{:#?} request error given path cannot be converted to a URI, uri_path: {:#?}", &uri, &uri_path);
                    }
                    // The given mail address is unreachable
                    lychee_lib::ErrorKind::UnreachableEmailAddress(uri, email) => {
                        log::debug!(
                            "{:#?} request error unreachable e-mail, email:{:#?}",
                            &uri,
                            &email
                        );
                    }
                    // The given header could not be parsed.
                    lychee_lib::ErrorKind::InvalidHeader(header) => {
                        log::debug!("{:#?} request error unable to parse HTTP header, problematic header: {:#?}", &uri, &header);
                    }
                    // The given string can not be parsed into a valid base URL or base directory
                    lychee_lib::ErrorKind::InvalidBase(base_dir, url) => {
                        log::debug!("{:#?} request error given string can not be parsed into a valid base URL or base directory, base_dir: {:#?}", &url, &base_dir);
                    }
                    // The given input can not be parsed into a valid URI remapping
                    lychee_lib::ErrorKind::InvalidUrlRemap(url) => {
                        log::debug!("{:#?} request error given input can not be parsed into a valid URI remapping, url: {:#?}", &uri, &url);
                    }
                    // The given path does not resolve to a valid file
                    lychee_lib::ErrorKind::FileNotFound(file_path) => {
                        log::debug!("{:#?} request error given path does not resolve to a valid file, file_path: {:#?}", &uri, &file_path);
                    }
                    // Error while traversing an input directory
                    lychee_lib::ErrorKind::DirTraversal(error) => {
                        log::debug!("{:#?} request error while traversing an input directory, error:\n{:#?}", &uri, &error);
                    }
                    // The given glob pattern is not valid
                    lychee_lib::ErrorKind::InvalidGlobPattern(glob_error) => {
                        log::debug!(
                            "{:#?} request error given glob pattern is not valid, error:\n{:#?}",
                            &uri,
                            &glob_error
                        );
                    }
                    // The Github API could not be called because of a missing Github token.
                    lychee_lib::ErrorKind::MissingGitHubToken => {
                        log::debug!("{:#?} request error missing GitHub API token", &uri);
                    }
                    // Used an insecure URI where a secure variant was reachable
                    lychee_lib::ErrorKind::InsecureURL(uri) => {
                        log::debug!("{:#?} request error used an insecure URI where a secure variant was reachable {:#?}", &uri, &uri);
                    }
                    // Error while sending/receiving messages from MPSC channel
                    lychee_lib::ErrorKind::Channel(send_error) => {
                        log::debug!("{:#?} request error while sending/receiving messages from MPSC channel, error:\n{:#?}", &uri, &send_error);
                    }
                    // An URL with an invalid host was found
                    lychee_lib::ErrorKind::InvalidUrlHost => {
                        log::debug!(
                            "{:#?} request error URL with an invalid host was found",
                            &uri
                        );
                    }
                    // Cannot parse the given URI
                    lychee_lib::ErrorKind::InvalidURI(uri) => {
                        log::debug!("{:#?} request error cannot parse the given URI", &uri);
                    }
                    // The given status code is invalid (not in the range 100-1000)
                    lychee_lib::ErrorKind::InvalidStatusCode(status_code) => {
                        log::debug!("{:#?} request error The given status code is invalid (not in the range 100-1000), status code: {:#?}", &uri, &status_code);
                    }
                    // Regex error
                    lychee_lib::ErrorKind::Regex(regex_error) => {
                        log::debug!(
                            "{:#?} request error regex, error:\n{:#?}",
                            &uri,
                            &regex_error
                        );
                    }
                    // Too many redirects (HTTP 3xx) were encountered (configurable)
                    lychee_lib::ErrorKind::TooManyRedirects(error) => {
                        log::debug!(
                            "{:#?} request error to many redirects, error:\n{:#?}",
                            &uri,
                            &error
                        );
                    }
                    // WTF???
                    &_ => {
                        log::debug!("{:#?} unknown error", &uri);
                    }
                }
            }
            // Request timed out
            lychee_lib::Status::Timeout(status_code) => {
                log::debug!(
                    "{:#?} request timeout, status code: {:#?}",
                    &uri,
                    &status_code
                );
            }
            // Got redirected to different resource
            lychee_lib::Status::Redirected(status_code) => {
                log::debug!(
                    "{:#?} request redirected with status code :{:#?}",
                    &uri,
                    &status_code
                );
            }
            // The given status code is not known by lychee
            lychee_lib::Status::UnknownStatusCode(status_code) => {
                log::debug!(
                    "{:#?} request error unknown status code: {:#?}",
                    &uri,
                    &status_code
                );
            }
            // Resource was excluded from checking
            lychee_lib::Status::Excluded => {
                log::debug!("{:#?} request was excluded from checking", &uri);
            }
            // The request type is currently not supported
            lychee_lib::Status::Unsupported(error_kind) => {
                log::debug!(
                    "{:#?} request type is currently not supported, error:\n{:#?}",
                    &uri,
                    &error_kind
                );
            }
            // Cached request status from previous run
            lychee_lib::Status::Cached(cache_status) => {
                log::debug!(
                    "{:#?} request cached, cache status: {:#?}",
                    &uri,
                    &cache_status
                );
            }
        };
    }
}
