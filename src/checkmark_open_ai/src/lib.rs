pub mod open_ai;

pub async fn check_grammar(
    file: &common::MarkDownFile,
) -> Result<Vec<common::CheckIssue>, open_ai::OpenAIError> {
    let mut issues: Vec<common::CheckIssue> = vec![];
    if let Ok(review) = open_ai::get_open_ai_grammar_suggestion(&file.content).await {
        for suggestion in review.suggestions {
            let row_num_start = 0;
            let row_num_end = 0;
            let col_num_start = 0;
            let col_num_end = 0;
            let offset = common::find_index(&file.content, &suggestion.original);
            issues.push(
                common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::Grammar)
                    .set_severity(common::IssueSeverity::Warning)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(row_num_start)
                    .set_row_num_end(row_num_end)
                    .set_col_num_start(col_num_start)
                    .set_col_num_end(col_num_end)
                    .set_offset_start(offset.start)
                    .set_offset_end(offset.end)
                    .set_message(suggestion.description)
                    .push_fix(&format!(
                        "Consider changing {:?} to: \n{:?}",
                        suggestion.original, suggestion.replacement
                    ))
                    .build(),
            );
        }
    }
    Ok(issues)
}

pub async fn make_a_review(
    file: &common::MarkDownFile,
    suggestions: bool,
) -> Result<Vec<common::CheckIssue>, open_ai::OpenAIError> {
    let mut issues: Vec<common::CheckIssue> = vec![];

    match open_ai::get_open_ai_review(file).await {
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
                if suggestions {
                    issue = issue.push_fix(&format!(
                        "Consider following change: \n{}",
                        &suggestion.replacement
                    ));
                }
                issues.push(issue.build());
            }
            // issues.push(
            //     common::CheckIssueBuilder::default()
            //         .set_category(common::IssueCategory::Review)
            //         .set_severity(common::IssueSeverity::Help)
            //         .set_file_path(file.path.clone())
            //         .set_row_num_start(0)
            //         .set_row_num_end(0)
            //         .set_col_num_start(0)
            //         .set_col_num_end(0)
            //         .set_offset_start(0)
            //         .set_offset_end(file.content.len())
            //         .set_message(review.summary)
            //         .build(),
            // );
            Ok(issues)
        }
        Err(err) => {
            log::error!(
                "Error getting review from OpenAI for file {:#?}, error:\n{:#?}",
                &file,
                &err
            );
            // TODO: return error
            Ok(issues)
        }
    }
}
