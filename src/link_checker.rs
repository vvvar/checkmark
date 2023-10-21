use lychee_lib::Result;
use lychee_lib::Collector;
use lychee_lib::Input;
use crate::checker::Issue;
use std::path::PathBuf;
use async_std::stream::StreamExt;
use lychee_lib::Status;


pub async fn check(path: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::<Issue>::new();
    let input = Input{
        source: lychee_lib::InputSource::FsPath(PathBuf::from(path)),
        file_type_hint: None,
        excluded_paths: None
    };
    let inputs = Vec::<Input>::from([input]);
    let links = Collector::new(None) // base
        .skip_missing_inputs(false) // don't skip missing inputs? (default=false)
        .use_html5ever(false) // use html5ever for parsing? (default=false)
        .collect_links(inputs) // base url or directory
        .await
        .collect::<Result<Vec<_>>>()
        .await?;
    for link in links {
        let url = link.uri.clone().to_string();
        match lychee_lib::check(link).await {
            Ok(response) => {
                match response.status() {
                    Status::Ok(_status) => {},
                    Status::Error(err) => issues.push(Issue {
                        id: String::from("MD002"),
                        file_path: String::from(path),
                        category: String::from("Link/URL"),
                        description: format!("{}: {}", url, err.details().unwrap()),
                        suggestions: vec![
                            String::from("Please check following link is reachable or ignore it")
                        ]
                    }),
                    Status::Timeout(_option) => issues.push(Issue {
                        id: String::from("MD002"),
                        file_path: String::from(path),
                        category: String::from("Link/URL"),
                        description: format!("{}: {}", url, "Request timeout"),
                        suggestions: vec![
                            String::from("Please check following link is reachable or ignore it")
                        ]
                    }),
                    Status::Redirected(_status) => issues.push(Issue {
                        id: String::from("MD002"),
                        file_path: String::from(path),
                        category: String::from("Link/URL"),
                        description: format!("{}: {}", url, "Request redirected"),
                        suggestions: vec![
                            String::from("Please check following link is reachable or ignore it")
                        ]
                    }),
                    Status::UnknownStatusCode(status_code) => issues.push(Issue {
                        id: String::from("MD002"),
                        file_path: String::from(path),
                        category: String::from("Link/URL"),
                        description: format!("{}: {}: {}", url, "Request replied with unknown stats code", status_code),
                        suggestions: vec![
                            String::from("Please check following link is reachable or ignore it")
                        ]
                    }),
                    Status::Excluded => {},
                    Status::Unsupported(_err) => {},
                    Status::Cached(_cache_status) => {}
                }
            },
            Err(error) => issues.push(Issue {
                id: String::from("MD002"),
                file_path: String::from(path),
                category: String::from("Link/URL"),
                description: format!("{}", error),
                suggestions: vec![
                    String::from("Please check following link is reachable or ignore it")
                ]
            })
        }
    }
    return Ok(issues);
}