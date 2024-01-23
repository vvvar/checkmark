mod client_config;
mod collector;
mod handlers;

use client_config::*;
use collector::*;
use common::{CheckIssue, Config, MarkDownFile};
use futures::future::join_all;
use handlers::*;
use log::debug;
use lychee_lib::ClientBuilder;
use std::collections::HashSet;
use std::time::Duration;

fn sanitize_uri(uri: &str) -> String {
    let mut uri = uri.to_string();
    if let Some(stripped) = uri.strip_suffix('/') {
        uri = stripped.to_string();
    };
    if let Some(stripped) = uri.strip_prefix("file://") {
        uri = stripped.to_string();
    };
    uri
}

pub async fn check(file: &MarkDownFile, config: &Config) -> Vec<CheckIssue> {
    debug!("Checking: {:#?}, config: {:#?}", &file.path, &config);
    let links = collect(&file.path, config).await.unwrap();
    let client_config = ClientConfig::from_checkmark_config(config);
    let requests = links.iter().map(|(uri, request)| {
        let uri = sanitize_uri(uri);
        let request = request.clone();
        let timeout = Duration::from_secs(client_config.timeout);
        let accepted = HashSet::from_iter(client_config.accepted_status_codes.clone().into_iter());
        let client = ClientBuilder::builder()
            .timeout(timeout)
            .include_mail(client_config.check_emails)
            .accepted(accepted)
            .max_retries(client_config.max_retries)
            .github_token(client_config.github_token.clone())
            .build()
            .client()
            .unwrap();
        debug!("Checking {:#?}", &uri);
        async move { (uri, client.check(request).await.unwrap()) }
    });
    join_all(requests)
        .await
        .iter()
        .map(|(uri, response)| handle_response(file, uri, response, &client_config))
        .flatten()
        .collect::<Vec<CheckIssue>>()
}

pub struct BulkCheckResult {
    pub path: String,
    pub issues: Vec<CheckIssue>,
}

pub async fn bulk_check(files: &Vec<MarkDownFile>, config: &Config) -> Vec<BulkCheckResult> {
    let checks = files.iter().map(|file| {
        let file_path = file.path.clone();
        let config_clone = config.clone();
        async move {
            BulkCheckResult {
                path: file_path.clone(),
                issues: check(file, &config_clone).await,
            }
        }
    });
    join_all(checks).await
}
