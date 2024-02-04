mod invalid_file_path;
mod network_request_error;
mod request_timeout;
mod unreachable_email;
mod utils;

use crate::client_config::ClientConfig;
use common::{CheckIssue, MarkDownFile};
use log::debug;
use lychee_lib::{ErrorKind, Response, Status};

pub fn handle_response(
    file: &MarkDownFile,
    uri: &String,
    response: &Response,
    client_config: &ClientConfig,
) -> Vec<CheckIssue> {
    debug!("Handling response for {uri}");
    match response.status() {
        // Request was successful
        Status::Ok(status_code) => {
            debug!("{uri} respond OK with code {status_code}");
            vec![]
        }
        // Failed request
        Status::Error(error_kind) => {
            match error_kind {
                // Network error while handling request
                ErrorKind::NetworkRequest(error) => network_request_error::handle(file, uri, error),
                // Cannot read the body of the received response
                ErrorKind::ReadResponseBody(error) => {
                    debug!("{uri} respond OK, but unable to read response body, error: {error}");
                    vec![]
                }
                // The network client required for making requests cannot be created
                ErrorKind::BuildRequestClient(error) => {
                    debug!("{uri} request cant build client, error: {error}");
                    vec![]
                }
                // Network error while using Github API
                ErrorKind::GithubRequest(error) => {
                    debug!(
                        "{:#?} request error while using GitHub API, error:\n{:#?}",
                        &uri, &error
                    );
                    vec![]
                }
                // Error while executing a future on the Tokio runtime
                ErrorKind::RuntimeJoin(error) => {
                    debug!("{:#?} request error while executing a future on the Tokio runtime, error:\n{:#?}", &uri, &error);
                    vec![]
                }
                // Error while converting a file to an input
                ErrorKind::ReadFileInput(error, file) => {
                    debug!("{:#?} request error while converting a file {:#?} to an input, error:\n{:#?}", &uri, &file, &error);
                    vec![]
                }
                // Error while reading stdin as input
                ErrorKind::ReadStdinInput(error) => {
                    debug!(
                        "{:#?} request error while reading stdin as input, error:\n{:#?}",
                        &uri, &error
                    );
                    vec![]
                }
                // Errors which can occur when attempting to interpret a sequence of u8 as a string
                ErrorKind::Utf8(error) => {
                    debug!(
                        "{:#?} request error while interpret a sequence of u8, error:\n{:#?}",
                        &uri, &error
                    );
                    vec![]
                }
                // The Github client required for making requests cannot be created
                ErrorKind::BuildGithubClient(error) => {
                    debug!(
                        "{:#?} request error cant create github client, error:\n{:#?}",
                        &uri, &error
                    );
                    vec![]
                }
                // Invalid Github URL
                ErrorKind::InvalidGithubUrl(invalid_github_url) => {
                    debug!(
                        "{:#?} request error invalid GitHub URL: {:#?}",
                        &uri, &invalid_github_url
                    );
                    vec![]
                }
                // The input is empty and not accepted as a valid URL
                ErrorKind::EmptyUrl => {
                    debug!("{:#?} request error empty URL", &uri);
                    vec![]
                }
                // The given string can not be parsed into a valid URL, e-mail address, or file path
                ErrorKind::ParseUrl(error, url) => {
                    debug!(
                        "{:#?} request error cant parse URL, error:\n{:#?}",
                        &url, &error
                    );
                    vec![]
                }
                // The given URI cannot be converted to a file path
                ErrorKind::InvalidFilePath(_) => invalid_file_path::handle(file, uri),
                // The given path cannot be converted to a URI
                ErrorKind::InvalidUrlFromPath(uri_path) => {
                    debug!("{:#?} request error given path cannot be converted to a URI, uri_path: {:#?}", &uri, &uri_path);
                    vec![]
                }
                // The given mail address is unreachable
                ErrorKind::UnreachableEmailAddress(_, email) => {
                    unreachable_email::handle(file, uri, email)
                }
                // The given header could not be parsed.
                ErrorKind::InvalidHeader(header) => {
                    debug!("{:#?} request error unable to parse HTTP header, problematic header: {:#?}", &uri, &header);
                    vec![]
                }
                // The given string can not be parsed into a valid base URL or base directory
                ErrorKind::InvalidBase(base_dir, url) => {
                    debug!("{:#?} request error given string can not be parsed into a valid base URL or base directory, base_dir: {:#?}", &url, &base_dir);
                    vec![]
                }
                // The given input can not be parsed into a valid URI remapping
                ErrorKind::InvalidUrlRemap(url) => {
                    debug!("{:#?} request error given input can not be parsed into a valid URI remapping, url: {:#?}", &uri, &url);
                    vec![]
                }
                // Error while traversing an input directory
                ErrorKind::DirTraversal(error) => {
                    debug!(
                        "{:#?} request error while traversing an input directory, error:\n{:#?}",
                        &uri, &error
                    );
                    vec![]
                }
                // The given glob pattern is not valid
                ErrorKind::InvalidGlobPattern(glob_error) => {
                    debug!(
                        "{:#?} request error given glob pattern is not valid, error:\n{:#?}",
                        &uri, &glob_error
                    );
                    vec![]
                }
                // The Github API could not be called because of a missing Github token.
                ErrorKind::MissingGitHubToken => {
                    debug!("{:#?} request error missing GitHub API token", &uri);
                    vec![]
                }
                // Used an insecure URI where a secure variant was reachable
                ErrorKind::InsecureURL(uri) => {
                    debug!("{:#?} request error used an insecure URI where a secure variant was reachable {:#?}", &uri, &uri);
                    vec![]
                }
                // Error while sending/receiving messages from MPSC channel
                ErrorKind::Channel(send_error) => {
                    debug!("{:#?} request error while sending/receiving messages from MPSC channel, error:\n{:#?}", &uri, &send_error);
                    vec![]
                }
                // An URL with an invalid host was found
                ErrorKind::InvalidUrlHost => {
                    debug!(
                        "{:#?} request error URL with an invalid host was found",
                        &uri
                    );
                    vec![]
                }
                // Cannot parse the given URI
                ErrorKind::InvalidURI(uri) => {
                    debug!("{:#?} request error cannot parse the given URI", &uri);
                    vec![]
                }
                // The given status code is invalid (not in the range 100-1000)
                ErrorKind::InvalidStatusCode(status_code) => {
                    debug!("{:#?} request error The given status code is invalid (not in the range 100-1000), status code: {:#?}", &uri, &status_code);
                    vec![]
                }
                // Regex error
                ErrorKind::Regex(regex_error) => {
                    debug!(
                        "{:#?} request error regex, error:\n{:#?}",
                        &uri, &regex_error
                    );
                    vec![]
                }
                // Too many redirects (HTTP 3xx) were encountered (configurable)
                ErrorKind::TooManyRedirects(error) => {
                    debug!(
                        "{:#?} request error to many redirects, error:\n{:#?}",
                        &uri, &error
                    );
                    vec![]
                }
                // WTF???
                _ => {
                    debug!("{:#?} unknown error", &uri);
                    vec![]
                }
            }
        }
        // Request timed out
        Status::Timeout(_) => {
            request_timeout::handle(file, uri, client_config.timeout, client_config.max_retries)
        }
        // Got redirected to different resource
        Status::Redirected(status_code) => {
            debug!(
                "{:#?} request redirected with status code :{:#?}",
                &uri, &status_code
            );
            vec![]
        }
        // The given status code is not known by lychee
        Status::UnknownStatusCode(status_code) => {
            debug!(
                "{:#?} request error unknown status code: {:#?}",
                &uri, &status_code
            );
            vec![]
        }
        // Resource was excluded from checking
        Status::Excluded => {
            debug!("{:#?} request was excluded from checking", &uri);
            vec![]
        }
        // The request type is currently not supported
        Status::Unsupported(error_kind) => {
            debug!(
                "{:#?} request type is currently not supported, error:\n{:#?}",
                &uri, &error_kind
            );
            vec![]
        }
        // Cached request status from previous run
        Status::Cached(cache_status) => {
            debug!(
                "{:#?} request cached, cache status: {:#?}",
                &uri, &cache_status
            );
            vec![]
        }
    }
}
