mod cli;
mod errors;

use env_logger;

#[tokio::main]
async fn main() -> Result<(), errors::AppError> {
    env_logger::init();

    let tool_driver = serde_sarif::sarif::ToolComponentBuilder::default()
        .name("Markdown Checker")
        .build()
        .unwrap();

    let tool = serde_sarif::sarif::ToolBuilder::default()
        .driver(tool_driver)
        .build()
        .unwrap();

    let mut results: Vec<serde_sarif::sarif::Result> = vec![];
    match &cli::read().subcommands {
        cli::Subcommands::Fmt(fmt) => {
            for file in checkmark_ls::ls(&fmt.project_root) {
                if fmt.check {
                    results.append(
                        &mut checkmark_fmt::check_md_format(&file)
                            .iter()
                            .map(|issue| issue.to_sarif_result())
                            .collect(),
                    );
                } else {
                    std::fs::write(&file.path, &checkmark_fmt::fmt_markdown(&file).content).unwrap();
                }
            }
        }
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

    std::fs::write(
        "./report.sarif",
        format!("{}", serde_json::to_string(&sarif).unwrap()),
    )
    .unwrap();

    Ok(())
}
