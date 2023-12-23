#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub review: ReviewConfig,

    #[serde(default)]
    pub link_checker: LinkCheckerConfig,

    #[serde(default)]
    pub spelling: SpellingConfig
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            review: ReviewConfig::default(),
            link_checker: LinkCheckerConfig::default(),
            spelling: SpellingConfig::default(),
        }
    }
}

impl Config {
    /// Try to build config from TOML file
    pub fn from_file(path: &str) -> Option<Self> {
        log::debug!("Trying to build config from file: {}", &path);
        if let Ok(file) = std::fs::read_to_string(path) {
            match toml::from_str(&file) {
                Ok(cfg) => {
                    log::debug!("Config file found in {}: {:#?}", &path, &cfg);
                    return Some(cfg);
                }
                Err(err) => {
                    log::error!("Error while parsing config file: {}", err);
                    return None;
                }
            }
        } else {
            return None;
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ReviewConfig {
    #[serde(default)]
    pub no_suggestions: bool,
}

impl std::default::Default for ReviewConfig {
    fn default() -> Self {
        Self {
            no_suggestions: false,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct LinkCheckerConfig {
    #[serde(default)]
    pub ignore_wildcards: Vec<String>,
}

impl std::default::Default for LinkCheckerConfig {
    fn default() -> Self {
        Self {
            ignore_wildcards: vec![],
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SpellingConfig {
    #[serde(default)]
    pub words_whitelist: Vec<String>,
}

impl std::default::Default for SpellingConfig {
    fn default() -> Self {
        Self {
            words_whitelist: vec![],
        }
    }
}

/// First, create one with default values
/// Then, try reading from TOML file by path provided in CLI
/// if no CLI option provided - try reading from default locations(replace when found)
/// and then apply config from CLI because it has higher priority
pub fn read_config(cli: &crate::cli::Cli) -> Config {
    log::debug!("Building default config...");

    let mut config = Config::default();
    log::debug!("Default config built: {:#?}", &config);

    log::debug!("Trying to read config from file...");
    if let Some(cfg_path_from_cli) = &cli.config {
        log::debug!("Trying to read config from CLI arg {}...", &cfg_path_from_cli);
        if let Some(cfg) = Config::from_file(cfg_path_from_cli) {
            config = cfg; // Replace default config with config from file
        } else {
            log::warn!("Config file not found in {}", &cfg_path_from_cli);
        }
    } else {
        let default_locations = [
            "checkmark.toml",
            ".checkmark.toml",
            "config/checkmark.toml",
            "config/.checkmark.toml",
            "cfg/checkmark.toml",
            "cfg/.checkmark.toml",
            "conf/checkmark.toml",
            "conf/.checkmark.toml",
        ];
        log::debug!("Trying to read config from default locations {:#?}...", &default_locations);
        for file_path in default_locations.iter() {
            if let Some(cfg) = Config::from_file(file_path) {
                config = cfg; // Replace default config with config from file
                break;
            }
        }
    }
    log::debug!("Config after merging with file: {:#?}", &config);

    log::debug!("Merging config with CLI options...");
    match &cli.subcommands {
        crate::cli::Subcommands::Fmt(_) => {}
        crate::cli::Subcommands::Grammar(_) => {}
        crate::cli::Subcommands::Links(links) => {
            if !links.ignore_wildcards.is_empty() {
                config.link_checker.ignore_wildcards = links.ignore_wildcards.clone();
            }
        }
        crate::cli::Subcommands::Review(review) => {
            config.review.no_suggestions = review.no_suggestions;
        }
        crate::cli::Subcommands::Spelling(_) => {}
    }
    log::debug!("Config after merging with CLI: {:#?}", &config);

    return config;
}
