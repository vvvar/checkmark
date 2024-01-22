pub mod open_ai;

use colored::Colorize;
use common::{
    find_index, CheckIssue, CheckIssueBuilder, Config, IssueCategory, IssueSeverity, MarkDownFile,
};
use open_ai::{open_ai_request, OpenAIError, OpenAIReview, OpenAiRequestParameters};

pub async fn make_a_review(
    file: &MarkDownFile,
    config: &Config,
) -> Result<Vec<CheckIssue>, OpenAIError> {
    let default_role_explanation = include_str!("prompts/review_default_role.txt");
    let response_format_explanation = include_str!("prompts/review_response_format.txt");
    let role_prompt = match &config.review.prompt {
        Some(prompt) => format!("{prompt}{response_format_explanation}"),
        None => format!("{default_role_explanation}{response_format_explanation}"),
    };
    let params = OpenAiRequestParameters::new(
        &role_prompt,
        &file.content,
        "json_object",
        &config.review.creativity,
        &config.open_ai.api_key,
    );
    let response = open_ai_request(&params).await?;

    let review = response
        .choices
        .iter()
        // Parse Open AI response into the JSON
        .map(|choice| serde_json::from_str::<OpenAIReview>(&choice.message.content))
        // Filter out corrupted responses
        .take_while(|e| e.is_ok())
        // Unwrap JSON responses(safe because of the filter above)
        .map(|e| e.unwrap())
        // Aggregate all responses into the single review
        .reduce(|acc, r| OpenAIReview {
            summary: r.summary,
            suggestions: vec![r.suggestions.clone(), acc.suggestions].concat(),
        })
        // If not possible - return default review with empty suggestions
        .unwrap_or(OpenAIReview::default());

    let check_issues = review
        .suggestions
        .iter()
        .map(|suggestion| {
            let offset = find_index(&file.content, &suggestion.original);
            let mut issue = CheckIssueBuilder::default()
                .set_category(IssueCategory::Review)
                .set_severity(IssueSeverity::Note)
                .set_file_path(file.path.clone())
                .set_row_num_start(1)
                .set_row_num_end(file.content.len())
                .set_col_num_start(1)
                .set_col_num_end(1)
                .set_offset_start(offset.start)
                .set_offset_end(offset.end)
                .set_message(suggestion.description.clone());
            if !config.review.no_suggestions {
                let suggestion =
                    &format!("Consider replacing with: {:#?}", &suggestion.replacement);
                issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), suggestion));
            }
            issue.build()
        })
        .collect::<Vec<CheckIssue>>();

    Ok(check_issues)
}

pub async fn compose_markdown(
    prompt: &str,
    context: &Option<String>,
    config: &Config,
) -> Result<String, OpenAIError> {
    let base_role_prompt = include_str!("prompts/compose_default_role.txt");
    let role_prompt = match context {
        Some(text) => format!("{base_role_prompt}\n\nUse following content as a context:\n{text}"),
        None => base_role_prompt.to_string(),
    };
    let params = OpenAiRequestParameters::new(
        &role_prompt,
        prompt,
        "text",
        &config.compose.creativity,
        &config.open_ai.api_key,
    );
    let response = open_ai_request(&params).await?;
    if let Some(choice) = response.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err(OpenAIError {
            message: "OpenAI failed to compose anything".to_string(),
        })
    }
}
