use symspell::{AsciiStringStrategy, Suggestion, SymSpell, Verbosity};

pub fn create_spell_checker(whitelisted_words: &Vec<String>) -> SymSpell<AsciiStringStrategy> {
    let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();
    // Initial dictionary
    include_str!("dictionaries/frequency_dictionary_en_82_765.txt")
        .lines()
        .for_each(|line| {
            symspell.load_dictionary_line(line, 0, 1, " ");
        });
    include_str!("dictionaries/frequency_bigramdictionary_en_243_342.txt")
        .lines()
        .for_each(|line| {
            symspell.load_bigram_dictionary_line(line, 0, 2, " ");
        });
    // Extended dictionary
    include_str!("dictionaries/extended_frequency_dictionary.txt")
        .lines()
        .for_each(|line| {
            let word = format!("{} 10956800", &line);
            symspell.load_dictionary_line(&word, 0, 1, " ");
            symspell.load_bigram_dictionary_line(&word, 0, 2, " ");
        });
    // User-defined white list
    whitelisted_words.iter().for_each(|word| {
        symspell.load_dictionary_line(&format!("{} 10956800", &word.to_lowercase()), 0, 1, " ");
        symspell.load_bigram_dictionary_line(
            &format!("{} 10956800", &word.to_lowercase()),
            0,
            2,
            " ",
        );
    });
    symspell
}

// Takes a word and checks if it is miss-spelled
// When all fine Ok is returned, otherwise Err with suggestions provided
pub fn check_spelling(
    spell_checker: &SymSpell<AsciiStringStrategy>,
    word: &str,
) -> Result<(), Vec<Suggestion>> {
    let suggestions = spell_checker.lookup(&word, Verbosity::Top, 2);
    // SymSpell suggest same word when all fine
    // Any suggestion - word is miss-spelled and SymSpell has ideas how to fix it
    // Empty suggestions means SymSpell has no clue what it is
    let is_miss_spelled = !suggestions
        .iter()
        .any(|suggestion| suggestion.term.eq(&word));
    if is_miss_spelled {
        Err(suggestions)
    } else {
        Ok(())
    }
}
