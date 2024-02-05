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
        let client = ClientBuilder::builder()
            .allow_insecure(client_config.allow_insecure)
            .timeout(client_config.timeout)
            .include_mail(client_config.check_emails)
            .accepted(client_config.accepted_status_codes.clone())
            .max_retries(client_config.max_retries)
            .github_token(client_config.github_token.clone())
            .user_agent(client_config.user_agent.clone())
            .build()
            .client()
            .unwrap();
        let uri = sanitize_uri(uri);
        debug!("Checking {:#?}", &uri);
        async move { (uri, client.check(request.clone()).await.unwrap()) }
    });
    join_all(requests)
        .await
        .iter()
        .flat_map(|(uri, response)| handle_response(file, uri, response, &client_config))
        .collect::<Vec<CheckIssue>>()
}

pub struct BulkCheckResult {
    pub path: String,
    pub issues: Vec<CheckIssue>,
}

pub async fn bulk_check(files: &[MarkDownFile], config: &Config) -> Vec<BulkCheckResult> {
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
