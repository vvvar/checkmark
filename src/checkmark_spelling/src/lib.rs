mod spell_checker;
mod text_to_words;

use spell_checker::{check_spelling, create_spell_checker};
use symspell::{AsciiStringStrategy, SymSpell};
use text_to_words::{text_to_words, Word};

use colored::Colorize;
use common::tui::CheckProgressTUI;
use common::{CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity, MarkDownFile};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

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

/// Perform spell check of a single file.
/// Returns a list of issues found in the file.
/// Spell checker is injected due to performance reasons
/// since it's expensive to create it for each file it is
/// assumed that those who use this function will have
/// to create it once and use it for multiple files.
/// For the details of library & algo see:
/// https://github.com/reneklacan/symspell
/// https://github.com/wolfgarbe/SymSpell
pub fn spell_check(
    spell_checker: &SymSpell<AsciiStringStrategy>,
    file: &MarkDownFile,
    config: &common::Config,
) -> Vec<CheckIssue> {
    log::debug!("Checking spelling for file {:#?}", &file.path);
    text_to_words(&file.content)
        .par_iter()
        .map(|word| (word, check_spelling(spell_checker, &word.value)))
        .filter(|(_, result)| result.is_err())
        .map(|(word, result)| (word, result.unwrap_err()))
        .map(|(word, suggestions)| to_check_issue(word, &file.path, &config.location, &suggestions))
        .collect::<Vec<CheckIssue>>()
}

/// Perform spell check of a list of files and fill them with issues found.
pub fn spell_check_bulk(
    files: &mut Vec<MarkDownFile>,
    config: &common::Config,
    tui: &Arc<Mutex<CheckProgressTUI>>,
) {
    tui.lock().unwrap().start_spinner("Checking spelling...");
    log::debug!("Initializing spell checker...");
    let spell_checker = create_spell_checker(&config.spelling.words_whitelist);
    files.par_iter_mut().for_each(|file| {
        file.issues
            .append(&mut spell_check(&spell_checker, file, config));
        tui.lock().unwrap().print_file_check_status(file);
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[cfg(test)]
    const DUMMY_FILE_PATH: &str = "this/is/a/dummy/path/to/a/file.md";

    /// Share spellchecker across tests
    /// to avoid re-creating it for each test
    #[cfg(test)]
    static SPELL_CHECKER: once_cell::sync::Lazy<SymSpell<AsciiStringStrategy>> =
        once_cell::sync::Lazy::new(|| create_spell_checker(&vec![]));

    #[cfg(test)]
    fn assert_has_issues(
        content: &str,
        whitelist: &Vec<String>,
        expected_issues: &Vec<common::CheckIssue>,
    ) {
        let markdown = common::MarkDownFile {
            path: DUMMY_FILE_PATH.to_owned(),
            content: content.to_owned(),
            issues: vec![],
        };
        let config = common::Config {
            spelling: common::SpellingConfig {
                words_whitelist: whitelist.clone(),
                ..common::SpellingConfig::default()
            },
            ..common::Config::default()
        };
        let actual_issues = spell_check(&SPELL_CHECKER, &markdown, &config);
        // Custom assertion because we won't check "fixes" field
        for (index, issue) in expected_issues.iter().enumerate() {
            assert_eq!(issue.category, actual_issues.get(index).unwrap().category);
            assert_eq!(issue.severity, actual_issues.get(index).unwrap().severity);
            assert_eq!(issue.file_path, actual_issues.get(index).unwrap().file_path);
            assert_eq!(
                issue.row_num_start,
                actual_issues.get(index).unwrap().row_num_start
            );
            assert_eq!(
                issue.row_num_end,
                actual_issues.get(index).unwrap().row_num_end
            );
            assert_eq!(
                issue.col_num_start,
                actual_issues.get(index).unwrap().col_num_start
            );
            assert_eq!(
                issue.col_num_end,
                actual_issues.get(index).unwrap().col_num_end
            );
            assert_eq!(
                issue.offset_start,
                actual_issues.get(index).unwrap().offset_start
            );
            assert_eq!(
                issue.offset_end,
                actual_issues.get(index).unwrap().offset_end
            );
            assert_eq!(issue.message, actual_issues.get(index).unwrap().message);
        }
    }

    #[cfg(test)]
    fn assert_has_no_issues(content: &str, whitelist: &Vec<String>) {
        assert_has_issues(&content, whitelist, &vec![]);
    }

    /// Basic spell checking tests
    #[test]
    fn spelling_plain_misspelled_word() {
        assert_has_issues("# This is a headr\n", &vec![], &vec![
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 1,
            row_num_end: 1,
            col_num_start: 3,
            col_num_end: 18,
            offset_start: 12,
            offset_end: 17,
            message: "\"headr\": Unknown word".to_string(),
            fixes: vec![
                "🧠 Accurate spelling ensures clear, professional, and credible communication".to_string(),
                "💡 Consider changing \"headr\" to \"head\"".to_string(),
                "💡 Consider white-listing this word by adding it to the \"words_whitelist\" property in the config file".to_string(),
                "🔗 https://github.com/vvvar/checkmark/tree/main#generate-config".to_string()
            ],
        },
    ]);
    }

    #[test]
    fn spelling_several_misspelled_words() {
        assert_has_issues("\n\nHere is sommm additnal txt\n", &vec![], &vec![
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 27,
            offset_start: 10,
            offset_end: 15,
            message: "\"sommm\": Unknown word".to_string(),
            fixes: vec![
                "🧠 Accurate spelling ensures clear, professional, and credible communication".to_string(),
                "💡 Consider changing \"sommm\" to \"somme\"".to_string(),
                "💡 Consider white-listing this word by adding it to the \"words_whitelist\" property in the config file".to_string(),
                "🔗 https://github.com/vvvar/checkmark/tree/main#generate-config".to_string()
            ],
        },
        common::CheckIssue {
            category: common::IssueCategory::Spelling,
            severity: common::IssueSeverity::Warning,
            file_path: DUMMY_FILE_PATH.to_owned(),
            row_num_start: 3,
            row_num_end: 3,
            col_num_start: 1,
            col_num_end: 27,
            offset_start: 16,
            offset_end: 24,
            message: "\"additnal\": Unknown word".to_string(),
            fixes: vec![
                "🧠 Accurate spelling ensures clear, professional, and credible communication".to_string(),
                "💡 Consider changing \"additnal\" to \"additional\"".to_string(),
                "💡 Consider white-listing this word by adding it to the \"words_whitelist\" property in the config file".to_string(),
                "🔗 https://github.com/vvvar/checkmark/tree/main#generate-config".to_string()
            ],
        }
    ]);
    }

    #[test]
    fn spelling_apostrophe_supported() {
        assert_has_no_issues("# Don't", &vec![]);
        assert_has_no_issues("# Couldn't", &vec![]);
        assert_has_no_issues("# Won't", &vec![]);
    }

    #[test]
    fn spelling_respect_owned_form() {
        assert_has_no_issues("# Project's", &vec![]);
    }

    #[test]
    fn spelling_skip_numbers() {
        assert_has_no_issues("# Number here 42", &vec![]);
    }

    #[test]
    fn spelling_gibberish_handled() {
        assert_has_issues("# fdssryyukiuu's ", &vec![], &vec![common::CheckIssue {
        category: common::IssueCategory::Spelling,
        severity: common::IssueSeverity::Warning,
        file_path: DUMMY_FILE_PATH.to_owned(),
        row_num_start: 1,
        row_num_end: 1,
        col_num_start: 3,
        col_num_end: 17,
        offset_start: 2,
        offset_end: 14,
        message: "\"fdssryyukiuu\": Unknown word".to_string(),
        fixes: vec![
            "🧠 Accurate spelling ensures clear, professional, and credible communication".to_string(),
            "💡 Consider white-listing this word by adding it to the \"words_whitelist\" property in the config file".to_string(),
            "🔗 https://github.com/vvvar/checkmark/tree/main#generate-config".to_string()
        ],
    },]);
    }

    #[test]
    fn spelling_consider_abbreviation() {
        assert_has_no_issues(
            "# p.s. this is an example a.k.a. Example e.g. yeah, and etc.",
            &vec![],
        );
    }
}
