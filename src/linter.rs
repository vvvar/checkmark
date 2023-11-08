use js_sandbox::{AnyError, Script};
use serde::Deserialize;
use std::fs;
use std::include_str;

use crate::checker::Issue;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarkdownLintFixInfo {
    edit_column: u32,
    delete_count: u32,
    insert_text: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarkdownLintIssue {
    line_number: u32,
    rule_names: Vec<String>,
    rule_description: String,
    error_detail: Option<String>,
    error_context: Option<String>,
    error_range: Vec<u32>,
    fix_info: Option<MarkdownLintFixInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MarkdownLintResponse {
    content: Vec<MarkdownLintIssue>,
}

/// Lint MarkDown file.
/// Returns vector of found issues with formatting.
pub fn lint(path: &String) -> Result<Vec<Issue>, AnyError> {
    let mut issues = Vec::<Issue>::new();
    let original = fs::read_to_string(path)?;
    let markdown_lint_script = format!(
        "{}\n{}\n{}\n{}\n{}",
        "
        class URL {
            constructor() {}
        }
        let document={
            createElement: () => {}
        };
        ",
        include_str!("js/markdown-it.min.js"),
        include_str!("js/micromark-browser.js"),
        include_str!("js/markdownlint-browser.js"),
        "
        async function lint(content) {
            return markdownlint.library.sync({ strings: { content } });
        }
        "
    );
    match Script::from_string(&markdown_lint_script) {
        Ok(mut script) => match script
            .call::<(&std::string::String,), MarkdownLintResponse>("lint", (&original,))
        {
            Ok(response) => {
                for markdown_lint_issue in  response.content {
                    let error_context = if markdown_lint_issue.error_context.is_some() { markdown_lint_issue.error_context.unwrap() } else { String::new() };
                    issues.push(Issue {
                        id: String::from("MD005"),
                        file_path: format!("{}:{}", &path, &markdown_lint_issue.line_number),
                        category: String::from("Lint"),
                        description: format!("{}: {:?}", &markdown_lint_issue.rule_description, error_context),
                        issue_in_code: None,
                        suggestions: vec![
                            format!("See issue description: https://github.com/DavidAnson/markdownlint/blob/main/doc/{}.md", &markdown_lint_issue.rule_names.first().unwrap().to_lowercase())
                        ],
                    });
                }
            }
            Err(e) => {
                dbg!(&e);
            }
        },
        Err(e) => {
            dbg!(&e);
        }
    }
    return Ok(issues);
}
