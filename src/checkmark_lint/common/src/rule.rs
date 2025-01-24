use crate::violation::*;

use common::{Config, MarkDownFile};
use markdown::mdast::*;

use url::Url;

pub trait Rule
where
    Self: Default,
{
    fn metadata(&self) -> Metadata;

    fn check(&self, ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation>;
}

pub struct Metadata {
    pub additional_links: Vec<Url>,
    pub code: &'static str,
    pub documentation: Url,
    pub is_fmt_fixable: bool,
    pub rationale: &'static str,
    pub requirement: &'static str,
}
