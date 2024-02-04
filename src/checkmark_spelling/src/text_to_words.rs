use common::{filter_text_nodes, find_index, parse};
use markdown::unist::Position;
use rayon::prelude::*;
use std::fmt::{Display, Formatter, Result};
use std::ops::Range;

/// We want to ignore spell-checking for certain exceptions
fn is_ignored_word(word: &str) -> bool {
    let is_number = |w: &str| w.chars().all(|c| c.is_numeric());
    is_number(word)
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
        f.write_str(&self.value.as_ref())
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
        .filter(|word| !is_url::is_url(&word))
        .map(|w| w.split('-').collect::<Vec<_>>())
        .flatten()
        .map(|w| {
            if w.to_lowercase().contains("n't") {
                vec![w]
            } else {
                w.split('\'').collect::<Vec<_>>()
            }
        })
        .flatten()
        .map(|w| w.split('/').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('(').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split(')').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('{').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('}').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('[').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split(']').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split(',').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('.').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('!').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('?').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split(':').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('_').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('"').collect::<Vec<_>>())
        .flatten()
        .map(|w| w.split('+').collect::<Vec<_>>())
        .flatten()
        // .map(|w| (w, remove_all_special_characters(w, true)))
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
    let ast = parse(&text).unwrap();
    filter_text_nodes(&ast)
        .par_iter()
        .map(|text_node| extract(&text_node))
        .flatten()
        .collect()
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
    fn extracting_words_from_text_node() {
        assert_eq!(
            extract(&text_node("This/is a {test} 111+")),
            vec!["this", "is", "a", "test"]
        );
        assert_eq!(
            extract(&text_node(
                "Get, https://totalbs.com your [double-edged]: sword."
            )),
            vec!["get", "your", "double", "edged", "sword"]
        );
        assert_eq!(
            extract(&text_node("Hello(there)World!?")),
            vec!["hello", "there", "world"]
        );
    }
}
