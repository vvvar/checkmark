use symspell::{AsciiStringStrategy, SymSpell, Verbosity};

pub fn spell_check(file: &mut common::MarkDownFile) {
    let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();

    for line in String::from(include_str!(
        "dictionary/frequency_dictionary_en_82_765.txt"
    ))
    .lines()
    {
        symspell.load_dictionary_line(line, 0, 1, " ");
    }
    for line in String::from(include_str!(
        "dictionary/frequency_bigramdictionary_en_243_342.txt"
    ))
    .lines()
    {
        symspell.load_bigram_dictionary_line(line, 0, 2, " ");
    }

    for (num_line, line) in file.content.lines().enumerate() {
        let text_only = markdown_to_text::convert(&line)
            .to_lowercase()
            .replace("!", "")
            .replace(".", "")
            .replace(",", "")
            .replace(";", "")
            .replace("'", "")
            .replace("\"", "")
            .replace(":", "");
        // 1. split line into words
        // 2. check every work for spelling error
        // 3. any spelling error? put it to suggestion list + lookup for compound suggestion for the whole sentence
        for word in text_only.split_ascii_whitespace() {
            let suggestions = symspell.lookup(&word, Verbosity::Top, 2);
            if !suggestions.is_empty() && !suggestions.first().unwrap().term.eq(&word) {
                let mut issue_suggestions = vec![format!(
                    "Consider changing {:#?} to {:#?}",
                    &word,
                    suggestions.first().unwrap().term
                )];
                let compound_suggestions = symspell.lookup_compound(&text_only, 2);
                if !compound_suggestions.is_empty()
                    && !compound_suggestions.first().unwrap().term.eq(&text_only)
                {
                    issue_suggestions.push(format!(
                        "Also, consider changing the whole line:\n{}{}\n{}{}",
                        "- ",
                        text_only,
                        "+ ",
                        compound_suggestions.first().unwrap().term
                    ));
                }
                issue_suggestions.push(format!("If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)"));
                file.issues.push(
                    common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Spelling)
                        .set_severity(common::IssueSeverity::Warning)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(&num_line + 1)
                        .set_row_num_end(&num_line + 1)
                        .set_col_num_start(1)
                        .set_col_num_end(1)
                        .set_offset_start(0)
                        .set_offset_end(file.content.len())
                        .set_message(format!("Word {:#?} is miss-spelled", &word))
                        .set_fixes(issue_suggestions)
                        .build(),
                );
            }
        }
    }
}
