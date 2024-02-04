mod spell_checker;
mod text_to_words;

use spell_checker::{check_spelling, create_spell_checker};
use text_to_words::{text_to_words, Word};

use colored::Colorize;
use common::{CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity, MarkDownFile};
use rayon::prelude::*;

fn to_check_issue(
    word: &Word,
    source_file_path: &str,
    config_file_location: &Option<String>,
    suggestions: &Vec<symspell::Suggestion>,
) -> CheckIssue {
    let mut issue: CheckIssueBuilder = CheckIssueBuilder::default()
        .set_category(IssueCategory::Spelling)
        .set_severity(IssueSeverity::Warning)
        .set_file_path(source_file_path.to_string())
        .set_row_num_start(word.source.line.start)
        .set_row_num_end(word.source.line.end)
        .set_col_num_start(word.source.column.start)
        .set_col_num_end(word.source.column.end)
        .set_offset_start(word.source.offset.start)
        .set_offset_end(word.source.offset.end)
        .set_message(format!("{:#?}: Unknown word", &word.source.value))
        .set_fixes(vec![format!(
            "🧠 {}  {}",
            "Rationale".cyan(),
            "Accurate spelling ensures clear, professional, and credible communication"
        )]);
    if !suggestions.is_empty() {
        for suggestion in suggestions {
            let fix = &format!(
                "Consider changing {:#?} to {:#?}",
                &word.source.value, suggestion.term
            );
            issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
        }
    }
    if let Some(location) = &config_file_location {
        let suggestion = format!(
            "Consider white-listing this word by adding it to your config file: {:#?}",
            &location
        );
        issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), suggestion));
        issue = issue.push_fix(&format!("🔗 {}        {}", "See".cyan(), "https://github.com/vvvar/checkmark/blob/main/src/checkmark_cli/src/config_template.toml"));
    } else {
        let suggestion = "Consider white-listing this word by adding it to the \"words_whitelist\" property in the config file";
        issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), suggestion));
        issue = issue.push_fix(&format!(
            "🔗 {}        {}",
            "See".cyan(),
            "https://github.com/vvvar/checkmark/tree/main#generate-config"
        ));
    }
    issue.build()
}

/// Perform spell check of the file
/// and fill it with issues(if any).
/// For the details of library & algo see:
/// https://github.com/reneklacan/symspell
/// https://github.com/wolfgarbe/SymSpell
pub fn spell_check(file: &MarkDownFile, config: &common::Config) -> Vec<CheckIssue> {
    log::debug!("Checking spelling for file {:#?}", &file);
    let spell_checker = create_spell_checker(&config.spelling.words_whitelist);
    text_to_words(&file.content)
        .par_iter()
        .map(|word| (word, check_spelling(&spell_checker, &word.value)))
        .filter(|(_, result)| result.is_err())
        .map(|(word, result)| (word, result.unwrap_err()))
        .map(|(word, suggestions)| to_check_issue(word, &file.path, &config.location, &suggestions))
        .collect::<Vec<CheckIssue>>()
}
