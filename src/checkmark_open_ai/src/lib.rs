pub mod open_ai;

pub async fn check_grammar(file: &mut common::MarkDownFile) -> Result<(), open_ai::OpenAIError> {
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    for text in common::filter_text_nodes(&ast) {
        match open_ai::get_open_ai_grammar_suggestion(&text.value).await? {
            open_ai::OpenAISuggestion::Suggestion(suggestion) => {
                let mut row_num_start = 0;
                let mut row_num_end = 0;
                let mut col_num_start = 0;
                let mut col_num_end = 0;
                let mut offset_start = 0;
                let mut offset_end = 0;
                if let Some(position) = &text.position {
                    row_num_start = position.start.line;
                    row_num_end = position.end.line;
                    col_num_start = position.start.column;
                    col_num_end = position.end.column;
                    offset_start = position.start.offset;
                    offset_end = position.end.offset;
                }
                file.issues.push(
                    common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Grammar)
                        .set_severity(common::IssueSeverity::Warning)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(row_num_start)
                        .set_row_num_end(row_num_end)
                        .set_col_num_start(col_num_start)
                        .set_col_num_end(col_num_end)
                        .set_offset_start(offset_start)
                        .set_offset_end(offset_end)
                        .set_message(String::from(
                            "Statement/sentence does not look like standard English",
                        ))
                        .push_fix(&format!("Consider changing to: \n{}", suggestion))
                        .build(),
                );
            }
            open_ai::OpenAISuggestion::NoSuggestion => {}
        }
    }
    Ok(())
}

pub async fn make_a_review(file: &mut common::MarkDownFile) -> Result<(), open_ai::OpenAIError> {
    if let Ok(review) = open_ai::get_open_ai_review(&file).await {
        if !review.suggestions.is_empty() {
            file.issues.push(
                common::CheckIssueBuilder::default()
                    .set_category(common::IssueCategory::Review)
                    .set_severity(common::IssueSeverity::Help)
                    .set_file_path(file.path.clone())
                    .set_row_num_start(0)
                    .set_row_num_end(0)
                    .set_col_num_start(0)
                    .set_col_num_end(0)
                    .set_offset_start(0)
                    .set_offset_end(file.content.len())
                    .set_message("Consider review of your document".to_string())
                    .push_fix(&review.summary)
                    .build(),
            );
            for suggestion in &review.suggestions {
                let mut index_start = 0;
                let mut index_end = file.content.len();
                if let Some(index) = file.content.find(&suggestion.original) {
                    index_start = index;
                    index_end = index_start + &suggestion.original.len();
                } else {
                    for line in file.content.lines() {
                        if strsim::sorensen_dice(&suggestion.original, &line) > 0.5 {
                            index_start = file.content.find(&line).unwrap();
                            index_end = file.content.len();
                        }
                    }
                }
                file.issues.push(
                    common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Review)
                        .set_severity(common::IssueSeverity::Note)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(1)
                        .set_row_num_end(file.content.len())
                        .set_col_num_start(1)
                        .set_col_num_end(1)
                        .set_offset_start(index_start)
                        .set_offset_end(index_end)
                        .set_message(suggestion.description.clone())
                        .push_fix(&format!(
                            "Consider changing to: \n{}",
                            &suggestion.replacement
                        ))
                        .build(),
                );
            }
        }
    }
    return Ok(());
}
