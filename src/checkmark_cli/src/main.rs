mod cli;
mod errors;

use env_logger;

/// Perform an analysis according to the tool from subcommand
fn analyze(cli: &cli::Cli, issues: &mut Vec<common::CheckIssue>) {
    match &cli.subcommands {
        cli::Subcommands::Fmt(fmt) => {
            for file in checkmark_ls::ls(&cli.project_root) {
                if fmt.check {
                    issues.append(&mut checkmark_fmt::check_md_format(&file));
                } else {
                    std::fs::write(&file.path, &checkmark_fmt::fmt_markdown(&file).content)
                        .unwrap();
                }
            }
        }
    }
}

/// Produce an issue report
fn report(cli: &cli::Cli, issues: &Vec<common::CheckIssue>) {
    let tool_driver = serde_sarif::sarif::ToolComponentBuilder::default()
        .name("Markdown Checker")
        .build()
        .unwrap();

    let tool = serde_sarif::sarif::ToolBuilder::default()
        .driver(tool_driver)
        .build()
        .unwrap();

    let results: Vec<serde_sarif::sarif::Result> =
        issues.iter().map(|issue| issue.to_sarif_result()).collect();

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
        std::fs::write(
            &file_path,
            format!("{}", serde_json::to_string(&sarif).unwrap()),
        )
        .unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), errors::AppError> {
    env_logger::init();
    let cli = cli::init();
    let mut issues: Vec<common::CheckIssue> = vec![];
    analyze(&cli, &mut issues);
    report(&cli, &issues);
    if issues.is_empty() {
        return Ok(());
    } else {
        return Err(errors::AppError {
            message: "Issues found during analysis. Check report for details.".to_string(),
        });
    }
}
