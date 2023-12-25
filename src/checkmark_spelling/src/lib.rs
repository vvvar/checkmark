use symspell::{AsciiStringStrategy, SymSpell, Verbosity};

/// We want to ignore spell-checking for certain exceptions
fn is_ignored_word(word: &str) -> bool {
    return word.chars().all(|c| c.is_numeric());
}

/// To check spelling we need to provide pure word
/// without any special or punctuation characters
fn remove_all_special_characters(word: &str, lowercase: bool) -> String {
    // Remove "'s" apostrophe because it can be added to any noun
    if word.contains("'s") {
        // return remove_all_special_characters(word.strip_suffix("'s").unwrap(), lowercase);
    }

    // These chars are generally considered unwanted
    let mut escaped = word
        .replace(
            [
                '?', '!', ',', '`', '\"', '{', '}', '[', ']', '(', ')', '#', '%', '|', '/', ';',
                ':',
            ],
            "",
        )
        .replace(|c: char| !c.is_ascii(), "");

    // Because we want to remove only prefix/suffix
    // and preserve words such as "don't", "isn't", etc.
    if let Some(stripped) = escaped.strip_prefix('\'') {
        escaped = stripped.to_string();
    }
    if let Some(stripped) = escaped.strip_suffix('\'') {
        escaped = stripped.to_string();
    }

    // Because we want to preserve words such as
    // un-intended and so on
    if let Some(stripped) = escaped.strip_prefix('-') {
        escaped = stripped.to_string();
    }
    if let Some(stripped) = escaped.strip_suffix('-') {
        escaped = stripped.to_string();
    }

    // Preserve period for abbreviations
    let abbreviations_with_period = [
        "a.k.a.", "e.g.", "etc.", "ex.", "al.", "i.e.", "p.s.", "u.s.", "vs.", "dr.", "mr.",
        "mrs.", "sun.", "mon.", "tues.", "wed.", "thurs.", "fri.", "sat.", "sun.", "jan.", "feb.",
        "aug.", "sept.", "oct.", "nov.", "dec.",
    ];
    if !abbreviations_with_period.contains(&escaped.to_lowercase().as_str()) {
        escaped = escaped.replace('.', "");
    }

    if lowercase {
        escaped.to_lowercase()
    } else {
        escaped
    }
}

/// Perform spell check of the file
/// and fill it with issues(if any).
/// For the details of library & algo see:
/// https://github.com/reneklacan/symspell
/// https://github.com/wolfgarbe/SymSpell
pub fn spell_check(file: &mut common::MarkDownFile, whitelist: &Vec<String>) {
    // Initialize SymSpell

    log::debug!("Initializing SymSpell...");
    let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();

    log::debug!("Loading default dictionary...");
    for line in String::from(include_str!(
        "dictionaries/frequency_dictionary_en_82_765.txt"
    ))
    .lines()
    {
        symspell.load_dictionary_line(line, 0, 1, " ");
    }
    for line in String::from(include_str!(
        "dictionaries/frequency_bigramdictionary_en_243_342.txt"
    ))
    .lines()
    {
        symspell.load_bigram_dictionary_line(line, 0, 2, " ");
    }

    log::debug!("Loading extended dictionary...");
    for line in String::from(include_str!(
        "dictionaries/extended_frequency_dictionary.txt"
    ))
    .lines()
    {
        log::debug!("Loading word from extended dictionary: {:#?}", &line);
        symspell.load_dictionary_line(line, 0, 1, " ");
        symspell.load_bigram_dictionary_line(line, 0, 2, " ");
    }

    log::debug!(
        "Loading words from the whitelist to the dictionary: {:#?}",
        &whitelist
    );
    for word in whitelist {
        log::debug!("Loading whitelisted word: {:#?}", &word);
        symspell.load_dictionary_line(&format!("{} 10956800", &word.to_lowercase()), 0, 1, " ");
        symspell.load_bigram_dictionary_line(
            &format!("{} 10956800", &word.to_lowercase()),
            0,
            2,
            " ",
        );
    }

    // Parse MD to AST
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    log::debug!("Parsed AST: {:#?}", &ast);
    // Filter only Text nodes
    for text_node in common::filter_text_nodes(&ast) {
        log::debug!("Spell checking text node: {:#?}", &text_node);
        // Split text into the words because spellcheck checks words, not sentences
        for word in text_node.value.split_ascii_whitespace() {
            log::debug!("Spell checking word: {:#?}", &word);
            // Remove special characters
            let escaped_word = remove_all_special_characters(word, true);
            log::debug!("Word after escaping: {:#?}", &word);
            // Do not proceed when this is not an actual word
            if !escaped_word.is_empty() && !is_ignored_word(&escaped_word) {
                // Get suggestions
                let suggestions = symspell.lookup(&escaped_word, Verbosity::Top, 2);
                log::debug!(
                    "Suggestions on word {:#?}: {:#?}",
                    &escaped_word,
                    &suggestions
                );
                // SymSpell suggest same word when all fine
                // Any suggestion - word is miss-spelled and SymSpell has ideas how to fix it
                // Empty suggestions means SymSpell has no clue what it is
                if !suggestions
                    .iter()
                    .any(|suggestion| suggestion.term.eq(&escaped_word))
                {
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
                        offset_start = position.start.offset
                            + common::find_index(&text_node.value, word).start;
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
                        .set_message(format!(
                            "Word {:#?} is unknown or miss-spelled",
                            &remove_all_special_characters(word, false)
                        ));
                    if suggestions.is_empty() {
                        issue = issue.push_fix(&format!(
                            "Cannot find any suggestion for word {:#?}",
                            &remove_all_special_characters(word, false)
                        ));
                    } else {
                        for suggestion in suggestions {
                            issue = issue.push_fix(&format!(
                                "Consider changing {:#?} to {:#?}",
                                &remove_all_special_characters(word, false),
                                suggestion.term
                            ));
                        }
                    }
                    issue = issue.push_fix("If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)");
                    file.issues.push(issue.build());
                }
            }
        }
    }
}
