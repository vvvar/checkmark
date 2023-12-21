pub mod open_ai;

/// Find index of substring in source string
fn find_index(source: &str, sub_str: &str) -> std::ops::Range<usize> {
    let mut index_start = 0;
    let mut index_end = source.len();
    log::debug!("Searching {:#?}", &sub_str);
    if let Some(index) = source.find(&sub_str) {
        log::debug!("Found exact index: {:#?}", &index);
        index_start = index;
        index_end = index_start + &sub_str.len();
    } else {
        log::debug!("Unable to find exact index, trying to guess");
        for line in source.lines() {
            if strsim::sorensen_dice(&sub_str, &line) > 0.5 {
                index_start = source.find(&line).unwrap();
                index_end = source.len();
                log::debug!(
                    "Found the best guess line on index {:#?}:\n{:#?}",
                    &index_start,
                    &line
                );
            }
        }
    }
    return std::ops::Range {
        start: index_start,
        end: index_end,
    };
}

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

pub async fn make_a_review(
    file: &mut common::MarkDownFile,
    include_suggestions: bool,
) -> Result<(), open_ai::OpenAIError> {
    match open_ai::get_open_ai_review(&file).await {
        Ok(review) => {
            if review.suggestions.is_empty() {
                log::warn!("OpenAI haven't provided any suggestions:\n{:#?}", &review);
                return Ok(());
            } else {
                log::debug!("Got OpenAI review:\n{:#?}", &review);
            }
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
                let offset = find_index(&file.content, &suggestion.original);
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
                if include_suggestions {
                    issue = issue.push_fix(&format!(
                        "Consider following change: \n{}",
                        &suggestion.replacement
                    ));
                }
                file.issues.push(issue.build());
            }
            return Ok(());
        }
        Err(err) => {
            log::error!("Error getting review from OpenAI:\n{:#?}", &err);
            return Err(err);
        }
    }
}
