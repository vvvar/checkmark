use common::find_index;
use markdown::mdast::Text;
use markdown::unist::Position;
use rayon::prelude::*;
use std::fmt::{Display, Formatter, Result};
use std::ops::Range;

/// Return true when the word is an ordinal number.
/// Will return false when the word is not a valid ordinal number like 5st, 5nd, 5rd.
///
/// Most ordinal numbers end in "th" except when the final word is:
///     one → first (1st)
///     two → second (2nd)
///     three → third (3rd)
fn is_valid_ordinal_number(src: &str) -> bool {
    let is_ordinal = src.chars().nth(0).unwrap_or(' ').is_numeric()
        && (src.ends_with("st")
            || src.ends_with("nd")
            || src.ends_with("rd")
            || src.ends_with("th"));
    if is_ordinal {
        let mut last_num_char = ' ';
        let mut ending = String::from("");
        for char in src.chars() {
            if char.is_numeric() {
                last_num_char = char;
            } else {
                ending.push(char);
            }
        }
        return match last_num_char {
            '1' => ending == "st",
            '2' => ending == "nd",
            '3' => ending == "rd",
            _ => ending == "th",
        };
    } else {
        return false;
    }
}

/// Return true when all characters in word are not alphabetic.
/// or example "123" or "1234"
fn is_whole_word_non_alphabetic(word: &str) -> bool {
    word.chars().all(|c| !c.is_alphabetic())
}

/// Return true when the word contains a number within the string
/// bun not at the beginning. For example "hello1" or "hello123
fn is_word_with_number_within(word: &str) -> bool {
    word.chars().skip(1).any(|c| c.is_numeric())
}

/// We want to ignore spell-checking for certain exceptions
fn is_ignored_word(word: &str) -> bool {
    is_whole_word_non_alphabetic(word)
        || is_word_with_number_within(word)
        || is_valid_ordinal_number(word)
}

/// Struct to hold information about
/// the source of the original word
#[derive(Debug, Clone)]
pub struct WordSource {
    pub value: String,
    pub line: Range<usize>,
    pub column: Range<usize>,
    pub offset: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Word {
    pub value: String,
    pub source: WordSource,
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.value.as_ref())
    }
}

impl PartialEq<&str> for Word {
    fn eq(&self, other: &&str) -> bool {
        self.value.eq(other)
    }
}

// Takes a Text node and splits it into words that is ready for spell checking
// while preserving meta information about original word and its position
fn extract(node: &markdown::mdast::Text) -> Vec<Word> {
    node.value
        .split_ascii_whitespace()
        .filter(|word| !is_url::is_url(word))
        .flat_map(|w| w.split('-').collect::<Vec<_>>())
        .flat_map(|w| {
            if w.to_lowercase().contains("n't") {
                vec![w]
            } else {
                w.split('\'').collect::<Vec<_>>()
            }
        })
        .flat_map(|w| w.split('/').collect::<Vec<_>>())
        .flat_map(|w| w.split('(').collect::<Vec<_>>())
        .flat_map(|w| w.split(')').collect::<Vec<_>>())
        .flat_map(|w| w.split('{').collect::<Vec<_>>())
        .flat_map(|w| w.split('}').collect::<Vec<_>>())
        .flat_map(|w| w.split('[').collect::<Vec<_>>())
        .flat_map(|w| w.split(']').collect::<Vec<_>>())
        .flat_map(|w| w.split(',').collect::<Vec<_>>())
        .flat_map(|w| w.split('.').collect::<Vec<_>>())
        .flat_map(|w| w.split('!').collect::<Vec<_>>())
        .flat_map(|w| w.split('?').collect::<Vec<_>>())
        .flat_map(|w| w.split(':').collect::<Vec<_>>())
        .flat_map(|w| w.split('_').collect::<Vec<_>>())
        .flat_map(|w| w.split('"').collect::<Vec<_>>())
        .flat_map(|w| w.split('+').collect::<Vec<_>>())
        .map(|w| (w, w.to_lowercase()))
        .filter(|(_, escaped)| !escaped.is_empty())
        .filter(|(_, escaped)| !is_ignored_word(escaped))
        .map(|(original, escaped)| {
            let fallback_position = Position::new(0, 0, 0, 0, 0, 0);
            let text_node_offset = Range {
                start: node
                    .position
                    .as_ref()
                    .unwrap_or(&fallback_position)
                    .start
                    .offset,
                end: node
                    .position
                    .as_ref()
                    .unwrap_or(&fallback_position)
                    .end
                    .offset,
            };
            let word_offset = find_index(&node.value, original);
            Word {
                value: escaped.to_string(),
                source: WordSource {
                    value: original.to_string(),
                    line: node
                        .position
                        .as_ref()
                        .map(|p| p.start.line..p.end.line)
                        .unwrap_or(0..0),
                    column: node
                        .position
                        .as_ref()
                        .map(|p| p.start.column..p.end.column)
                        .unwrap_or(0..0),
                    offset: Range {
                        start: text_node_offset.start + word_offset.start,
                        end: text_node_offset.start + word_offset.end,
                    },
                },
            }
        })
        .collect::<Vec<_>>()
}

pub fn text_to_words(text: &str) -> Vec<Word> {
    let ast = common::ast::parse(text).unwrap();
    common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_text(n))
        .collect::<Vec<&Text>>() // Need to collect because .par_iter() is not available for iterators
        .par_iter()
        .flat_map(|t| extract(t))
        .collect::<Vec<Word>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn text_node(value: &str) -> markdown::mdast::Text {
        markdown::mdast::Text {
            value: value.to_string(),
            position: None,
        }
    }

    #[test]
    fn detect_ordinal_number() {
        assert!(is_valid_ordinal_number("1st"));
        assert!(is_valid_ordinal_number("2nd"));
        assert!(is_valid_ordinal_number("3rd"));
        assert!(is_valid_ordinal_number("4th"));
        assert!(is_valid_ordinal_number("5th"));
        assert!(is_valid_ordinal_number("21st"));
        assert!(is_valid_ordinal_number("22nd"));
        assert!(is_valid_ordinal_number("23rd"));
        assert!(is_valid_ordinal_number("23rd"));
        assert!(is_valid_ordinal_number("24th"));
        assert!(!is_valid_ordinal_number("5"));
        assert!(!is_valid_ordinal_number("5st"));
        assert!(!is_valid_ordinal_number("5nd"));
        assert!(!is_valid_ordinal_number("5rd"));
    }

    #[test]
    fn detect_words_with_num_within() {
        assert!(is_word_with_number_within("hello1"));
        assert!(is_word_with_number_within("hello123"));
        assert!(!is_word_with_number_within("hello"));
    }

    #[test]
    fn detect_whole_word_non_alphabetic() {
        assert!(is_whole_word_non_alphabetic("123"));
        assert!(is_whole_word_non_alphabetic("1234"));
        assert!(!is_whole_word_non_alphabetic("hello"));
    }

    #[test]
    fn extracting_words_from_text_node() {
        assert_eq!(
            extract(&text_node("This/is a & {test} 111+")),
            vec!["this", "is", "a", "test"]
        );
        assert_eq!(
            extract(&text_node(
                "Get, https://totalbs.com your [double-edged]: sword."
            )),
            vec!["get", "your", "double", "edged", "sword"]
        );
        assert_eq!(
            extract(&text_node("Hello(there)World fr1end 4real!?")),
            vec!["hello", "there", "world", "4real"]
        );
    }
}
