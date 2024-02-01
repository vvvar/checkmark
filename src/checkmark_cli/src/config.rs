/// First, create one with default values
/// Then, try reading from TOML file by path provided in CLI
/// if no CLI option provided - try reading from default locations(replace when found)
/// and then apply config from CLI because it has higher priority
pub fn read_config(cli: &crate::cli::Cli) -> common::Config {
    log::debug!("Building default config...");

    let mut config = common::Config::default();
    log::debug!("Default config built: {:#?}", &config);

    log::debug!("Trying to read config from file...");
    if let Some(cfg_path_from_cli) = &cli.config {
        log::debug!(
            "Trying to read config from CLI arg {}...",
            &cfg_path_from_cli
        );
        if let Some(cfg) = common::Config::from_file(cfg_path_from_cli) {
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
            "conf/checkmark.toml",
            "conf/.checkmark.toml",
            "cfg/checkmark.toml",
            "cfg/.checkmark.toml",
            ".github/checkmark.toml",
            ".github/.checkmark.toml",
        ];
        log::debug!(
            "Trying to read config from default locations {:#?}...",
            &default_locations
        );
        for file_path in default_locations.iter() {
            if let Some(cfg) = common::Config::from_file(file_path) {
                config = cfg; // Replace default config with config from file
                config.location = Some(
                    dunce::canonicalize(file_path.to_string())
                        .unwrap()
                        .display()
                        .to_string(),
                ); // Remember where we found it
                break;
            }
        }
    }
    log::debug!("Config after merging with file: {:#?}", &config);

    log::debug!("Merging config with CLI options...");
    match &cli.subcommands {
        crate::cli::Subcommands::Fmt(fmt) => {
            // When someone enabled these options via CLI - consider it as a force enablement.
            // Otherwise - keep one from the config. Doing this because of ambiguity of bool in CLI args
            // e.g. we cant normally distinguish when user provide's --check or just didn't set it at all
            if fmt.check && !config.fmt.check {
                config.fmt.check = true;
            }
            if fmt.show_diff && !config.fmt.show_diff {
                config.fmt.show_diff = true;
            }
        }
        crate::cli::Subcommands::Linkcheck(links) => {
            if !links.ignore_wildcards.is_empty() {
                config.link_checker.ignore_wildcards = links.ignore_wildcards.clone();
            }
            if let Some(timeout) = links.timeout {
                config.link_checker.timeout = Some(timeout);
            }
            if let Some(max_retries) = links.max_retries {
                config.link_checker.max_retries = Some(max_retries);
            }
            if !links.accept.is_empty() {
                config.link_checker.accept = links.accept.clone();
            }
            if let Some(github_token) = &links.github_token {
                config.link_checker.github_token = Some(github_token.clone());
            }
        }
        crate::cli::Subcommands::Lint(lint) => {
            if !lint.allowed_html_tags.is_empty() {
                config.linter.allowed_html_tags = lint.allowed_html_tags.clone();
            }
        }
        crate::cli::Subcommands::Review(review) => {
            config.review.no_suggestions = review.no_suggestions;
            if let Some(prompt) = &review.prompt {
                config.review.prompt = Some(prompt.clone());
            }
            if let Some(creativity) = review.creativity {
                if creativity > 100 {
                    log::warn!("Creativity value must be between 0 and 100! Ignoring this setting and using default value");
                } else {
                    config.review.creativity = Some(creativity);
                }
            }
        }
        crate::cli::Subcommands::Render(render) => {
            if let Some(output) = &render.output {
                config.rendering.output = Some(output.clone());
            }
            if let Some(theme) = &render.theme {
                config.rendering.theme = Some(theme.clone());
            }
            config.rendering.serve = render.serve;
        }
        crate::cli::Subcommands::Compose(compose) => {
            if let Some(creativity) = compose.creativity {
                if creativity > 100 {
                    log::warn!("Creativity value must be between 0 and 100! Ignoring this setting and using default value");
                } else {
                    config.compose.creativity = Some(creativity);
                }
            }
        }
        crate::cli::Subcommands::Spellcheck(_) => {}
        crate::cli::Subcommands::GenerateConfig(_) => {}
    }
    if !cli.exclude.is_empty() {
        config.global.exclude = cli.exclude.clone();
    }
    if let Some(style_headings) = &cli.style_headings {
        if style_headings.eq("consistent") {
            config.style.headings = common::HeadingStyle::Consistent;
        } else if style_headings.eq("atx") {
            config.style.headings = common::HeadingStyle::Atx;
        } else if style_headings.eq("setext") {
            config.style.headings = common::HeadingStyle::Setext;
        } else {
            log::warn!("Unknown heading style: {}", &style_headings);
        }
    }
    if let Some(style_unordered_lists) = &cli.style_unordered_lists {
        if style_unordered_lists.eq("consistent") {
            config.style.unordered_lists = common::UnorderedListStyle::Consistent;
        } else if style_unordered_lists.eq("dash") {
            config.style.unordered_lists = common::UnorderedListStyle::Dash;
        } else if style_unordered_lists.eq("asterisk") {
            config.style.unordered_lists = common::UnorderedListStyle::Asterisk;
        } else if style_unordered_lists.eq("plus") {
            config.style.unordered_lists = common::UnorderedListStyle::Plus;
        } else {
            log::warn!("Unknown unordered list style: {}", &style_unordered_lists);
        }
    }
    if let Some(num_spaces_after_list_marker) = cli.style_num_spaces_after_list_marker {
        config.style.num_spaces_after_list_marker = Some(num_spaces_after_list_marker);
    }
    if let Some(style_bold) = &cli.style_bold {
        if style_bold.eq("consistent") {
            config.style.bold = common::BoldStyle::Consistent;
        } else if style_bold.eq("asterisk") {
            config.style.bold = common::BoldStyle::Asterisk;
        } else if style_bold.eq("underscore") {
            config.style.bold = common::BoldStyle::Underscore;
        } else {
            log::warn!("Unknown bold style: {}", &style_bold);
        }
    }
    log::debug!("Config after merging with CLI: {:#?}", &config);

    config
}
