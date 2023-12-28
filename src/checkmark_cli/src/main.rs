mod cli;
mod config;
mod errors;
mod tui;

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
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug")
    }
    env_logger::init();

    // Read config
    let config = config::read_config(&cli);

    // Read all MD files
    let mut files = checkmark_ls::ls(&cli.project_root).await;

    // Create TUI
    let tui = tui::CheckProgressTUI::new_thread_safe(cli.ci);

    // Analyze
    match &cli.subcommands {
        cli::Subcommands::Fmt(fmt_cli) => match fmt_cli.check {
            true => {
                tui.lock().unwrap().start_spinner("Checking format...");
                for file in files.iter_mut() {
                    checkmark_fmt::check_md_format(file);
                    tui.lock().unwrap().print_file_check_status(file);
                }
            }
            false => {
                tui.lock().unwrap().start_spinner("Auto-formatting...");
                for file in files.iter_mut() {
                    std::fs::write(&file.path, &checkmark_fmt::fmt_markdown(&file).content)
                        .unwrap();
                }
            }
        },
        cli::Subcommands::Grammar(_) => {
            tui.lock().unwrap().start_spinner("Checking grammar...");
            for file in files.iter_mut() {
                checkmark_open_ai::check_grammar(file).await.unwrap();
            }
        }
        cli::Subcommands::Review(_) => {
            tui.lock().unwrap().start_spinner("Reviewing...");
            for file in files.iter_mut() {
                checkmark_open_ai::make_a_review(file, !config.review.no_suggestions)
                    .await
                    .unwrap();
            }
        }
        cli::Subcommands::Links(_) => {
            tui.lock().unwrap().start_spinner("Checking links...");
            for file in files.iter_mut() {
                checkmark_link_checker::check_links(file, &config.link_checker.ignore_wildcards)
                    .await;
            }
        }
        cli::Subcommands::Spelling(_) => {
            use rayon::prelude::*;

            tui.lock().unwrap().start_spinner("Checking spelling...");
            files.par_iter_mut().for_each(|file| {
                file.issues.append(&mut checkmark_spelling::spell_check(
                    file.clone(),
                    config.spelling.words_whitelist.clone(),
                ));
                tui.lock().unwrap().print_file_check_status(file);
            });
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
