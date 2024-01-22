mod link_collector;

use colored::Colorize;
use common::{CheckIssue, CheckIssueBuilder, Config, IssueCategory, IssueSeverity, MarkDownFile};
use futures::future::join_all;
use link_collector::*;
use log::debug;
use lychee_lib::{ClientBuilder, ErrorKind, Status};
use reqwest::StatusCode;
use std::collections::HashSet;
use std::ops::Range;
use std::path::Path;
use std::time::Duration;

pub fn find_all_links_in_file(file: &MarkDownFile, uri: &str) -> Vec<Range<usize>> {
    file.content
        .match_indices(&uri)
        .into_iter()
        .map(|(offset, matched_str)| Range {
            start: offset,
            end: offset + matched_str.len(),
        })
        .collect()
}

pub async fn check_links(file: &MarkDownFile, config: &Config) -> Vec<CheckIssue> {
    debug!("Checking: {:#?}, config: {:#?}", &file.path, &config);
    let mut issues: Vec<CheckIssue> = vec![];
    let links = collect_links(&file.path, &config.link_checker.ignore_wildcards)
        .await
        .unwrap();
    let accepted_status_codes = config
        .link_checker
        .accept
        .iter()
        .map(|code| StatusCode::from_u16(*code).unwrap())
        .collect::<Vec<StatusCode>>();
    let timeout = config.link_checker.timeout.unwrap_or(30) as u64;
    let max_retries = config.link_checker.max_retries.unwrap_or(1) as u64;
    let requests = links.iter().map(|(uri, request)| {
        debug!("Checking {:#?}", &uri);
        let mut uri = uri.clone();
        if let Some(_uri) = uri.strip_suffix('/') {
            uri = _uri.to_string();
        };
        if let Some(_uri) = uri.strip_prefix("file://") {
            uri = _uri.to_string();
        };
        let request = request.clone();
        let github_token = match &config.link_checker.github_token {
            Some(token) => Some(secrecy::SecretString::from(token.clone())),
            None => None,
        };
        let client = ClientBuilder::builder()
            .timeout(Duration::from_secs(timeout))
            .accepted(HashSet::from_iter(
                accepted_status_codes.clone().into_iter(),
            ))
            .max_retries(max_retries)
            .github_token(github_token)
            .build()
            .client()
            .unwrap();
        async move { (uri, client.check(request).await.unwrap()) }
    });
    for (uri, response) in join_all(requests).await {
        match response.status() {
            // Request was successful
            Status::Ok(status_code) => {
                debug!("{uri} respond OK with code {status_code}");
            }
            // Failed request
            Status::Error(error_kind) => {
                match error_kind {
                    // Network error while handling request
                    ErrorKind::NetworkRequest(error) => {
                        debug!("{uri} respond with network error: {error}");

                        for offset in find_all_links_in_file(file, &uri) {
                            let mut issue = CheckIssueBuilder::default()
                                .set_category(IssueCategory::LinkChecking)
                                .set_severity(IssueSeverity::Warning)
                                .set_file_path(file.path.clone())
                                .set_row_num_start(1)
                                .set_row_num_end(file.content.lines().count())
                                .set_col_num_start(1)
                                .set_col_num_end(1)
                                .set_offset_start(offset.start)
                                .set_offset_end(offset.end)
                                .set_message(format!("{error}"));
                            issue = issue.push_fix(&format!(
                                "🧠 {}  {}",
                                "Rationale".cyan(),
                                "Having a broken hyperlink is a bad, confusing user experience"
                            ));
                            let fixes = vec![
                                format!("If your network requires a proxy, consider setting it via HTTP_PROXY/HTTPS_PROXY env variables"),
                                format!("Consider checking your internet connection"),
                                format!("Can you open this link in a browser? If no then perhaps its broken and shall be fixed"),
                            ];
                            for fix in fixes {
                                issue =
                                    issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
                            }
                            issue = issue.push_fix(&format!(
                                "📚 {}       https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/{}",
                                "Docs".cyan(),
                                error.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR).as_str()
                            ));
                            issues.push(issue.build());
                        }
                    }
                    // Cannot read the body of the received response
                    ErrorKind::ReadResponseBody(error) => {
                        debug!(
                            "{uri} respond OK, but unable to read response body, error: {error}"
                        );
                    }
                    // The network client required for making requests cannot be created
                    ErrorKind::BuildRequestClient(error) => {
                        debug!("{uri} request cant build client, error: {error}");
                    }
                    // Network error while using Github API
                    ErrorKind::GithubRequest(error) => {
                        debug!(
                            "{:#?} request error while using GitHub API, error:\n{:#?}",
                            &uri, &error
                        );
                    }
                    // Error while executing a future on the Tokio runtime
                    ErrorKind::RuntimeJoin(error) => {
                        debug!("{:#?} request error while executing a future on the Tokio runtime, error:\n{:#?}", &uri, &error);
                    }
                    // Error while converting a file to an input
                    ErrorKind::ReadFileInput(error, file) => {
                        debug!("{:#?} request error while converting a file {:#?} to an input, error:\n{:#?}", &uri, &file, &error);
                    }
                    // Error while reading stdin as input
                    ErrorKind::ReadStdinInput(error) => {
                        debug!(
                            "{:#?} request error while reading stdin as input, error:\n{:#?}",
                            &uri, &error
                        );
                    }
                    // Errors which can occur when attempting to interpret a sequence of u8 as a string
                    ErrorKind::Utf8(error) => {
                        debug!(
                            "{:#?} request error while interpret a sequence of u8, error:\n{:#?}",
                            &uri, &error
                        );
                    }
                    // The Github client required for making requests cannot be created
                    ErrorKind::BuildGithubClient(error) => {
                        debug!(
                            "{:#?} request error cant create github client, error:\n{:#?}",
                            &uri, &error
                        );
                    }
                    // Invalid Github URL
                    ErrorKind::InvalidGithubUrl(invalid_github_url) => {
                        debug!(
                            "{:#?} request error invalid GitHub URL: {:#?}",
                            &uri, &invalid_github_url
                        );
                    }
                    // The input is empty and not accepted as a valid URL
                    ErrorKind::EmptyUrl => {
                        debug!("{:#?} request error empty URL", &uri);
                    }
                    // The given string can not be parsed into a valid URL, e-mail address, or file path
                    ErrorKind::ParseUrl(error, url) => {
                        debug!(
                            "{:#?} request error cant parse URL, error:\n{:#?}",
                            &url, &error
                        );
                    }
                    // The given URI cannot be converted to a file path
                    ErrorKind::InvalidFilePath(_) => {
                        debug!("{:#?} request error invalid file path", &uri);

                        let broken_filename =
                            Path::new(&uri).file_name().unwrap().to_str().unwrap();
                        for offset in find_all_links_in_file(file, broken_filename) {
                            let mut issue = CheckIssueBuilder::default()
                                .set_category(IssueCategory::LinkChecking)
                                .set_severity(IssueSeverity::Warning)
                                .set_file_path(file.path.clone())
                                .set_row_num_start(1)
                                .set_row_num_end(file.content.lines().count())
                                .set_col_num_start(1)
                                .set_col_num_end(1)
                                .set_offset_start(offset.start)
                                .set_offset_end(offset.end)
                                .set_message(format!(
                                    "File \"{broken_filename}\" is not found in path \"{uri}\"",
                                ));
                            issue = issue.push_fix(&format!(
                                "🧠 {}  {}",
                                "Rationale".cyan(),
                                "Having a broken link to a file will lead to 404 error page and confuse users"
                            ));
                            let fixes = vec![
                                format!("Does this file really exist? Try opening {:#?} in your file explorer", &uri),
                                format!("Is this a symlink? If yes, then consider replacing it with a real path. Having symlinks in a project leads to dangling references and often considered a bad practice"),
                            ];
                            for fix in fixes {
                                issue =
                                    issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
                            }
                            issues.push(issue.build());
                        }
                    }
                    // The given path cannot be converted to a URI
                    ErrorKind::InvalidUrlFromPath(uri_path) => {
                        debug!("{:#?} request error given path cannot be converted to a URI, uri_path: {:#?}", &uri, &uri_path);
                    }
                    // The given mail address is unreachable
                    ErrorKind::UnreachableEmailAddress(uri, email) => {
                        debug!(
                            "{:#?} request error unreachable e-mail, email:{:#?}",
                            &uri, &email
                        );
                    }
                    // The given header could not be parsed.
                    ErrorKind::InvalidHeader(header) => {
                        debug!("{:#?} request error unable to parse HTTP header, problematic header: {:#?}", &uri, &header);
                    }
                    // The given string can not be parsed into a valid base URL or base directory
                    ErrorKind::InvalidBase(base_dir, url) => {
                        debug!("{:#?} request error given string can not be parsed into a valid base URL or base directory, base_dir: {:#?}", &url, &base_dir);
                    }
                    // The given input can not be parsed into a valid URI remapping
                    ErrorKind::InvalidUrlRemap(url) => {
                        debug!("{:#?} request error given input can not be parsed into a valid URI remapping, url: {:#?}", &uri, &url);
                    }
                    // Error while traversing an input directory
                    ErrorKind::DirTraversal(error) => {
                        debug!("{:#?} request error while traversing an input directory, error:\n{:#?}", &uri, &error);
                    }
                    // The given glob pattern is not valid
                    ErrorKind::InvalidGlobPattern(glob_error) => {
                        debug!(
                            "{:#?} request error given glob pattern is not valid, error:\n{:#?}",
                            &uri, &glob_error
                        );
                    }
                    // The Github API could not be called because of a missing Github token.
                    ErrorKind::MissingGitHubToken => {
                        debug!("{:#?} request error missing GitHub API token", &uri);
                    }
                    // Used an insecure URI where a secure variant was reachable
                    ErrorKind::InsecureURL(uri) => {
                        debug!("{:#?} request error used an insecure URI where a secure variant was reachable {:#?}", &uri, &uri);
                    }
                    // Error while sending/receiving messages from MPSC channel
                    ErrorKind::Channel(send_error) => {
                        debug!("{:#?} request error while sending/receiving messages from MPSC channel, error:\n{:#?}", &uri, &send_error);
                    }
                    // An URL with an invalid host was found
                    ErrorKind::InvalidUrlHost => {
                        debug!(
                            "{:#?} request error URL with an invalid host was found",
                            &uri
                        );
                    }
                    // Cannot parse the given URI
                    ErrorKind::InvalidURI(uri) => {
                        debug!("{:#?} request error cannot parse the given URI", &uri);
                    }
                    // The given status code is invalid (not in the range 100-1000)
                    ErrorKind::InvalidStatusCode(status_code) => {
                        debug!("{:#?} request error The given status code is invalid (not in the range 100-1000), status code: {:#?}", &uri, &status_code);
                    }
                    // Regex error
                    ErrorKind::Regex(regex_error) => {
                        debug!(
                            "{:#?} request error regex, error:\n{:#?}",
                            &uri, &regex_error
                        );
                    }
                    // Too many redirects (HTTP 3xx) were encountered (configurable)
                    ErrorKind::TooManyRedirects(error) => {
                        debug!(
                            "{:#?} request error to many redirects, error:\n{:#?}",
                            &uri, &error
                        );
                    }
                    // WTF???
                    _ => {
                        debug!("{:#?} unknown error", &uri);
                    }
                }
            }
            // Request timed out
            Status::Timeout(_) => {
                debug!("{uri} request timeout");

                for offset in find_all_links_in_file(file, &uri) {
                    let issue = CheckIssueBuilder::default()
                        .set_category(IssueCategory::LinkChecking)
                        .set_severity(IssueSeverity::Warning)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(1)
                        .set_row_num_end(file.content.lines().count())
                        .set_col_num_start(1)
                        .set_col_num_end(1)
                        .set_offset_start(offset.start)
                        .set_offset_end(offset.end)
                        .set_message(format!("Request timeout for url {uri}"))
                        .set_fixes(vec![
                            format!("Consider increasing timeout in config file, currently its set to {timeout} seconds"),
                            format!("Consider increasing maximum amount of retries in config file, currently its set to {max_retries} seconds"),
                            format!("If your network requires proxy, consider setting it via HTTP_PROXY/HTTPS_PROXY env variables or configure proxy in config file"),
                            format!("Consider checking your internet connection"),
                        ]);
                    issues.push(issue.build());
                }
            }
            // Got redirected to different resource
            Status::Redirected(status_code) => {
                debug!(
                    "{:#?} request redirected with status code :{:#?}",
                    &uri, &status_code
                );
            }
            // The given status code is not known by lychee
            Status::UnknownStatusCode(status_code) => {
                debug!(
                    "{:#?} request error unknown status code: {:#?}",
                    &uri, &status_code
                );
            }
            // Resource was excluded from checking
            Status::Excluded => {
                debug!("{:#?} request was excluded from checking", &uri);
            }
            // The request type is currently not supported
            Status::Unsupported(error_kind) => {
                debug!(
                    "{:#?} request type is currently not supported, error:\n{:#?}",
                    &uri, &error_kind
                );
            }
            // Cached request status from previous run
            Status::Cached(cache_status) => {
                debug!(
                    "{:#?} request cached, cache status: {:#?}",
                    &uri, &cache_status
                );
            }
        };
    }

    issues
}
