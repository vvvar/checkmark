use crate::violation::*;

use common::{Config, MarkDownFile};
use markdown::mdast::*;

use url::Url;

pub trait Rule
where
    Self: Send + Sync,
{
    fn metadata(&self) -> Metadata;

    fn check(&self, ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation>;

    fn is_enabled(&self, config: &Config) -> bool {
        config
            .linter
            .exclude
            .iter()
            .map(|rule_name| rule_name.to_lowercase())
            .any(|rule_name| rule_name.eq(&self.metadata().code.to_lowercase()))
    }
}

pub struct Metadata {
    pub additional_links: Vec<Url>,
    pub code: &'static str,
    pub documentation: Url,
    pub is_fmt_fixable: bool,
    pub rationale: &'static str,
    pub requirement: &'static str,
}
