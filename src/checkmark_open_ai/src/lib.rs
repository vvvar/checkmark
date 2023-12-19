pub mod open_ai;

pub async fn check_grammar(
    file: &common::MarkDownFile,
) -> Result<Vec<common::CheckIssue>, open_ai::OpenAIError> {
    let mut issues: Vec<common::CheckIssue> = vec![];
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    for text in common::filter_text_nodes(&ast) {
        match open_ai::get_open_ai_grammar_suggestion(&text.value).await? {
            open_ai::OpenAISuggestion::Suggestion(suggestion) => {
                let mut row_num_start = 0;
                let mut row_num_end = 0;
                let mut col_num_start = 0;
                let mut col_num_end = 0;
                if let Some(position) = &text.position {
                    row_num_start = position.start.line;
                    row_num_end = position.end.line;
                    col_num_start = position.start.column;
                    col_num_end = position.end.column;
                }
                issues.push(
                    common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Grammar)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(row_num_start)
                        .set_row_num_end(row_num_end)
                        .set_col_num_start(col_num_start)
                        .set_col_num_end(col_num_end)
                        .set_message(String::from("Statement/sentence does not look like standard English"))
                        .set_fixes(vec![suggestion])
                        .build(),
                );
            }
            open_ai::OpenAISuggestion::NoSuggestion => {}
        }
    }
    return Ok(issues);
}

pub fn make_a_review(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];

    if let Ok(review) = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(open_ai::get_open_ai_review(&file))
    {
        if !review.suggestions.is_empty() {
            issues.push(
                common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::Review)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(0)
                    .set_row_num_end(0)
                    .set_col_num_start(0)
                    .set_col_num_end(0)
                    .set_message(String::from(&review.summary))
                    .build(),
            );
            for suggestion in &review.suggestions {
                issues.push(
                    common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Review)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(suggestion.line_start)
                        .set_row_num_end(suggestion.line_end)
                        .set_col_num_start(0)
                        .set_col_num_end(0)
                        .set_message(suggestion.problem.clone())
                        .push_fix(&suggestion.fix.clone())
                        .build(),
                );
            }
        }
    }

    return issues;
}
