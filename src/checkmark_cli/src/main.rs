mod cli;
mod config;
mod errors;
mod tui;

use colored::Colorize;
use rayon::prelude::*;

fn has_any_critical_issue(files: &Vec<common::MarkDownFile>) -> bool {
    let mut any_critical_issue = false;
    for file in files {
        any_critical_issue = !file.issues.is_empty()
            && file
                .issues
                .iter()
                .any(|issue| issue.severity == common::IssueSeverity::Error);
    }
    any_critical_issue
}

#[tokio::main]
async fn main() -> Result<(), errors::AppError> {
    // Parse CLI arguments
    let cli = cli::init();

    // When needed, force enable verbose logging
    let get_rust_log = |level: &str| {
        format!("none,checkmark_cli={level},checkmark_fmt={level},checkmark_link_checker={level},checkmark_lint={level},checkmark_ls={level},checkmark_open_ai={level},checkmark_spelling={level},common={level}")
    };
    if cli.verbose {
        std::env::set_var("RUST_LOG", get_rust_log("debug"))
    } else {
        std::env::set_var("RUST_LOG", get_rust_log("warn"))
    }
    env_logger::init();

    // Read config
    let config = config::read_config(&cli);

    // Read all MD files
    let mut files = checkmark_ls::ls(&cli.project_root, &config.global.exclude).await;

    // Create TUI
    let tui = tui::CheckProgressTUI::new_thread_safe(cli.ci);

    // Analyze
    match &cli.subcommands {
        cli::Subcommands::Fmt(_) => match config.fmt.check {
            true => {
                tui.lock().unwrap().start_spinner("Checking format...");
                files.par_iter_mut().for_each(|file| {
                    file.issues
                        .append(&mut checkmark_fmt::check_md_format(file, &config));
                    tui.lock().unwrap().print_file_check_status(file);
                });
            }
            false => {
                tui.lock().unwrap().start_spinner("Auto-formatting...");
                tui.lock().unwrap().set_custom_finish_message(
                    &"ʕっ•ᴥ•ʔっ All files has been auto-formatted"
                        .cyan()
                        .bold()
                        .to_string(),
                );
                files.par_iter_mut().for_each(|file| {
                    std::fs::write(
                        &file.path,
                        checkmark_fmt::fmt_markdown(file, &config).content,
                    )
                    .unwrap();
                    tui.lock().unwrap().print_file_check_status(file);
                });
            }
        },
        cli::Subcommands::Review(_) => {
            tui.lock().unwrap().start_spinner("Reviewing...");
            for file in files.iter_mut() {
                file.issues.append(
                    &mut checkmark_open_ai::make_a_review(file, &config)
                        .await
                        .unwrap(),
                );
                tui.lock().unwrap().print_file_check_status(file);
            }
        }
        cli::Subcommands::Compose(compose_cmd) => {
            tui.lock().unwrap().start_spinner("Composing...");
            let output_file = match &compose_cmd.output {
                Some(path) => path.clone(),
                None => "output.md".to_string(),
            };
            let context = match &compose_cmd.context {
                Some(path) => Some(std::fs::read_to_string(path).unwrap()),
                None => None,
            };
            tui.lock().unwrap().set_custom_finish_message(
                &"ʕっ•ᴥ•ʔっ Open a file to review the result"
                    .cyan()
                    .bold()
                    .to_string(),
            );
            let text = checkmark_open_ai::compose_markdown(&compose_cmd.prompt, &context, &config)
                .await
                .unwrap();
            std::fs::write(&output_file, &text).unwrap();
            tui.lock()
                .unwrap()
                .print_file_check_status(&common::MarkDownFile {
                    path: output_file,
                    content: text,
                    issues: vec![],
                });
        }
        cli::Subcommands::Linkcheck(_) => {
            tui.lock().unwrap().start_spinner("Checking links...");
            for file in files.iter_mut() {
                checkmark_link_checker::check_links(file, &config.link_checker.ignore_wildcards)
                    .await;
                tui.lock().unwrap().print_file_check_status(file);
            }
        }
        cli::Subcommands::Lint(_) => {
            tui.lock().unwrap().start_spinner("Linting...");
            files.par_iter_mut().for_each(|file| {
                file.issues.append(&mut checkmark_lint::lint(file, &config));
                tui.lock().unwrap().print_file_check_status(file);
            });
        }
        cli::Subcommands::Spellcheck(_) => {
            tui.lock().unwrap().start_spinner("Checking spelling...");
            files.par_iter_mut().for_each(|file| {
                file.issues
                    .append(&mut checkmark_spelling::spell_check(file, &config));
                tui.lock().unwrap().print_file_check_status(file);
            });
        }
        cli::Subcommands::GenerateConfig(generate_config) => {
            let path = dunce::canonicalize(&generate_config.path)
                .unwrap()
                .join("checkmark.toml");
            std::fs::write(path, include_str!("config_template.toml"))
                .expect("Unable to write a file");
        }
    }

    // Print all collected check issues
    tui.lock().unwrap().print_report(&files);

    // When requested - generate SARIF json
    if let Some(file_path) = &cli.sarif {
        let tool_driver = serde_sarif::sarif::ToolComponentBuilder::default()
            .name("Markdown Checker")
            .build()
            .unwrap();

        let tool = serde_sarif::sarif::ToolBuilder::default()
            .driver(tool_driver)
            .build()
            .unwrap();

        let mut results: Vec<serde_sarif::sarif::Result> = vec![];
        for analyzed_file in files.iter() {
            results.append(
                &mut analyzed_file
                    .issues
                    .iter()
                    .map(|issue| issue.to_sarif_result())
                    .collect(),
            );
        }

        let runs = serde_sarif::sarif::RunBuilder::default()
            .tool(tool)
            .results(results)
            .build()
            .unwrap();

        let sarif = serde_sarif::sarif::SarifBuilder::default()
            .version("2.1.0")
            .runs(vec![runs])
            .build()
            .unwrap();

        if let Some(file_path) = &cli.sarif {
            std::fs::write(file_path, serde_json::to_string(&sarif).unwrap()).unwrap();
        }
        std::fs::write(file_path, serde_json::to_string(&sarif).unwrap()).unwrap();
    }

    if has_any_critical_issue(&files) {
        return Err(errors::AppError {
            message: "Critical issues found during analysis. Check report for details.".to_string(),
        });
    } else {
        return Ok(());
    }
}
