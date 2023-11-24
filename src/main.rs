mod args;
mod file_explorer;
mod format_checker;
mod formatter;

use env_logger;

use spinners::{Spinner, Spinners};
use std::fmt;
use std::fs;

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
    let mut has_any_issue = false;
    for file in file_explorer::list_markdown_files(&arguments.root) {
        // let sarif: serde_sarif::sarif::Sarif = serde_json::from_str(
        // r#"{ "version": "2.1.0", "runs": [
        //     {
        //         "tool": {
        //             "driver": {
        //               "name": "Tool Name",
        //               "rules": [
        //                 {
        //                   "id": "R01",
        //                   "properties" : {
        //                      "id" : "java/unsafe-deserialization",
        //                      "kind" : "path-problem",
        //                      "name" : "...",
        //                      "problem.severity" : "error",
        //                      "security-severity" : "9.8"
        //                    }
        //                 }
        //               ]
        //             }
        //         },
        //         "results": []
        //     }
        // ] }"#
        //   ).unwrap();
        // dbg!(&sarif);
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
            .results(format_checker::check_md_format(&file))
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
    if has_any_issue {
        Err(AppError {
            message: String::from("Some files failed a check"),
        })
    } else {
        Ok(())
    }
}
