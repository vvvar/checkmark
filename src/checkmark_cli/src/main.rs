use env_logger;

use std::fmt;
use std::fs;

mod args;

#[derive(Debug)]
struct AppError {
    message: String,
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();
    let arguments = args::read();
    for file in checkmark_ls::ls(&arguments.root) {
        let mut results: Vec<serde_sarif::sarif::Result> = vec![];
        results.append(
            &mut checkmark_fmt::check_md_format(&file)
                .iter()
                .map(|issue| issue.to_sarif_result())
                .collect(),
        );
        results.append(
            &mut checkmark_lint::lint(&file)
                .iter()
                .map(|issue| issue.to_sarif_result())
                .collect(),
        );

        let tool_driver = serde_sarif::sarif::ToolComponentBuilder::default()
            .name("Markdown Checker")
            .build()
            .unwrap();
        let tool = serde_sarif::sarif::ToolBuilder::default()
            .driver(tool_driver)
            .build()
            .unwrap();
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
        fs::write(
            "./report.sarif",
            format!("{}", serde_json::to_string(&sarif).unwrap()),
        )
        .unwrap();
    }
    Ok(())
}
