use async_std::stream::StreamExt;
use common::Config;
use log::debug;
use lychee_lib::{Collector, Input, InputSource::*, Request, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::string::String;
use wildmatch::WildMatch;

/// Collect links from file
pub async fn collect(path: &str, config: &Config) -> Result<HashMap<String, Request>> {
    let input = vec![Input {
        source: FsPath(PathBuf::from(path)),
        file_type_hint: Some(lychee_lib::FileType::Markdown),
        excluded_paths: None,
    }];
    debug!("Lychee inputs: {:#?}", &input);

    let links = Collector::new(None) // base
        .skip_missing_inputs(true) // Valid pats are assumed
        .use_html5ever(false) // use html5gum, author claims it to be faster
        .include_verbatim(true) // verbatim is for ex. ```code``
        .collect_links(input)
        .collect::<Result<Vec<_>>>()
        .await?;
    debug!("Found links: {:#?}", &links);

    // Dedup them
    let mut links_map: HashMap<String, Request> = HashMap::new();
    for link in links {
        let uri = link.uri.as_str();
        let matches_any_ignored_uri_wildcard =
            config
                .link_checker
                .ignore_wildcards
                .iter()
                .any(|ignored_wildcard| {
                    if let Some(stripped_uri) = uri.strip_suffix('/') {
                        WildMatch::new(ignored_wildcard).matches(stripped_uri)
                    } else {
                        WildMatch::new(ignored_wildcard).matches(uri)
                    }
                });
        if !matches_any_ignored_uri_wildcard {
            links_map.insert(uri.to_string(), link.clone());
        }
    }
    debug!("De-duplicated links:\n{:#?}", &links_map);

    Ok(links_map)
}
