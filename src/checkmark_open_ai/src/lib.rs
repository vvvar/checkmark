pub mod open_ai;

pub async fn make_a_review(
    file: &common::MarkDownFile,
    config: &common::Config,
) -> Result<Vec<common::CheckIssue>, open_ai::OpenAIError> {
    let mut issues: Vec<common::CheckIssue> = vec![];
    let creativity = match config.review.creativity {
        Some(value) => value,
        None => 10,
    };
    match open_ai::get_open_ai_review(file, &config.review.prompt, creativity).await {
        Ok(review) => {
            for suggestion in &review.suggestions {
                let offset = common::find_index(&file.content, &suggestion.original);
                let mut issue = common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::Review)
                    .set_severity(common::IssueSeverity::Note)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(1)
                    .set_row_num_end(file.content.len())
                    .set_col_num_start(1)
                    .set_col_num_end(1)
                    .set_offset_start(offset.start)
                    .set_offset_end(offset.end)
                    .set_message(suggestion.description.clone());
                if !config.review.no_suggestions {
                    issue = issue.push_fix(&format!(
                        "Consider following change: \n{}",
                        &suggestion.replacement
                    ));
                }
                issues.push(issue.build());
            }
            Ok(issues)
        }
        Err(err) => {
            log::warn!(
                "Error getting review from OpenAI for file {:#?}, error:\n{:#?}",
                &file.path,
                &err
            );
            // TODO: return error
            Ok(issues)
        }
    }
}

pub async fn compose_markdown(
    prompt: &str,
    context: &Option<String>,
) -> Result<String, open_ai::OpenAIError> {
    let base_role_prompt = "You will be provided with user prompt.
Your task is to compose a file in Markdown format based on it.
Reply only with content of the file.
Avoid additional commentary.";
    let role_prompt = match context {
        Some(text) => format!(
            "{} \n\nUse following content as a context: {}",
            base_role_prompt, text
        ),
        None => base_role_prompt.to_string(),
    };
    return match open_ai::open_ai_request(&role_prompt, &prompt, "text", 20).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                Ok(choice.message.content.clone())
            } else {
                Err(open_ai::OpenAIError {
                    message: "OpenAI failed to compose anything".to_string(),
                })
            }
        }
        Err(err) => Err(err),
    };
}
