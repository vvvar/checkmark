use common::Config;
use reqwest::StatusCode;
use secrecy::SecretString;

pub struct ClientConfig {
    pub accepted_status_codes: Vec<StatusCode>,
    pub timeout: u64,
    pub max_retries: u64,
    pub github_token: Option<SecretString>,
    pub check_emails: bool,
}

impl ClientConfig {
    pub fn from_checkmark_config(config: &Config) -> Self {
        Self {
            accepted_status_codes: ClientConfig::accept_status_codes(config),
            timeout: ClientConfig::timeout(config),
            max_retries: ClientConfig::max_retries(config),
            github_token: ClientConfig::github_token(config),
            check_emails: config.link_checker.check_emails,
        }
    }

    // Calculate accepted status codes from config
    fn accept_status_codes(config: &Config) -> Vec<StatusCode> {
        config
            .link_checker
            .accept
            .iter()
            .map(|code| StatusCode::from_u16(*code).unwrap())
            .collect()
    }

    // Calculate request timeout from config
    fn timeout(config: &Config) -> u64 {
        config.link_checker.timeout.unwrap_or(10) as u64
    }

    // Calculate maximum amount of HTTP retries from config
    fn max_retries(config: &Config) -> u64 {
        config.link_checker.max_retries.unwrap_or(1) as u64
    }

    fn github_token(config: &Config) -> Option<SecretString> {
        config
            .link_checker
            .github_token
            .as_ref()
            .map(|token| SecretString::from(token.clone()))
    }
}
