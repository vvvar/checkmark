use symspell::{AsciiStringStrategy, SymSpell, Verbosity};

/// To check spelling we need to provide pure word
/// without any special or punctuation characters
fn remove_all_special_characters(word: &str, lowercase: bool) -> String {
    let escaped = word
        .replace("?", "")
        .replace("!", "")
        .replace(".", "")
        .replace(",", "")
        .replace("'", "")
        .replace("`", "")
        .replace("\"", "")
        .replace("[", "")
        .replace("]", "")
        .replace("(", "")
        .replace(")", "")
        .replace(";", "")
        .replace(":", "")
        .replace("-", "")
        .replace(|c: char| !c.is_ascii(), "");
    if lowercase {
        return escaped.to_lowercase();
    } else {
        return escaped;
    }
}

/// Perform spell check of the file
/// and fill it with issues(if any)
pub fn spell_check(file: &mut common::MarkDownFile) {
    // Initialize SymSpell
    let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();
    for line in String::from(include_str!("dictionary/frequency_dictionary_en_82_765.txt")).lines() {
        symspell.load_dictionary_line(line, 0, 1, " ");
    }
    for line in String::from(include_str!("dictionary/frequency_bigramdictionary_en_243_342.txt")).lines() {
        symspell.load_bigram_dictionary_line(line, 0, 2, " ");
    }
    // Parse MD to AST
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    // Filter only Text nodes
    for text_node in common::filter_text_nodes(&ast) {
        // Split text into the words because spellcheck checks words, not sentences
        for word in text_node.value.split_ascii_whitespace() {
            // Remove special characters
            let escaped_word = remove_all_special_characters(word, true);
            // Do not proceed when this is not an actual word
            if !escaped_word.is_empty() {
                // Get suggestions
                let suggestions = symspell.lookup(&escaped_word, Verbosity::Top, 2);
                // Only when there are suggestions to change something
                // (SymSpell return same word when all fine) 
                if !suggestions.is_empty() && !suggestions.first().unwrap().term.eq(&escaped_word) {
                    let mut row_num_start = 0;
                    let mut row_num_end = 0;
                    let mut col_num_start = 0;
                    let mut col_num_end = 0;
                    let mut offset_start = 0;
                    let mut offset_end = 0;
                    if let Some(position) = &text_node.position {
                        row_num_start = position.start.line;
                        row_num_end = position.end.line;
                        col_num_start = position.start.column;
                        col_num_end = position.end.column;
                        // Calculate offset based on offset of text node + index of word
                        offset_start = position.start.offset + common::find_index(&text_node.value, word).start;
                        offset_end = offset_start + word.len();
                    }
                    let mut issue = common::CheckIssueBuilder::default()
                        .set_category(common::IssueCategory::Spelling)
                        .set_severity(common::IssueSeverity::Warning)
                        .set_file_path(file.path.clone())
                        .set_row_num_start(row_num_start)
                        .set_row_num_end(row_num_end)
                        .set_col_num_start(col_num_start)
                        .set_col_num_end(col_num_end)
                        .set_offset_start(offset_start)
                        .set_offset_end(offset_end)
                        .set_message(format!("Word {:#?} is unknown or miss-spelled", &remove_all_special_characters(word, false)));
                    for suggestion in suggestions {
                        issue = issue.push_fix(&format!("Consider changing {:#?} to {:#?}", &word, suggestion.term));
                    }
                    issue = issue.push_fix("If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)");
                    file.issues.push(issue.build());
                }
            }
        }  
    }
}
