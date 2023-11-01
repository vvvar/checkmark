use crate::checker::Issue;
use lychee_lib::Result;
use std::fs;
use colored::Colorize;

use symspell::{SymSpell, AsciiStringStrategy, Verbosity};

pub async fn check(path: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::<Issue>::new();

    let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();

    symspell.load_dictionary("/Users/vvoinov/Documents/repos/md-checker/src/spell_dictionary/frequency_dictionary_en_82_765.txt", 0, 1, " ");
    symspell.load_bigram_dictionary(
      "/Users/vvoinov/Documents/repos/md-checker/src/spell_dictionary/frequency_bigramdictionary_en_243_342.txt",
      0,
      2,
      " "
    );

    let file = fs::read_to_string(path)?;
    for (num_line, line) in file.lines().enumerate() {
      let text_only = markdown_to_text::convert(&line).to_lowercase()
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
          let mut issue_suggestions = vec![
            format!("Consider changing {} to {}", &word.red(), suggestions.first().unwrap().term.green())
          ];
          let compound_suggestions = symspell.lookup_compound(&text_only, 2);
          if !compound_suggestions.is_empty() && !compound_suggestions.first().unwrap().term.eq(&text_only) {
            issue_suggestions.push(
                format!("Also, consider changing the whole line:\n{}{}\n{}{}", "- ".red(), text_only.red(), "+ ".green(), compound_suggestions.first().unwrap().term.green())
            );
          }
          issue_suggestions.push(format!("If you're sure that this word is correct - add it to the spellcheck dictionary(TBD)"));
          issues.push(Issue {
            id: String::from("MD004"),
            file_path: format!("{}:{}", &path, &num_line + 1),
            category: String::from("Spelling"),
            description: format!("Unknown word: {:?}", &word),
            issue_in_code: None,
            suggestions: issue_suggestions
          });
        }
      }
    }

    return Ok(issues);
}