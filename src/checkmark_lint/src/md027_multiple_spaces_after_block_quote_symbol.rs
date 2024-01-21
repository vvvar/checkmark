use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{BlockQuote, Node};
use regex::Regex;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD027")
        .message("Multiple spaces after block quote symbol")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md027.md")
        .is_fmt_fixable(true)
}

// Check that block quote has multiple spaces after bq symbol:
// Example: >  text
fn has_multiple_spaces_after_bq_symbol(bq: &BlockQuote, source: &str) -> bool {
    let line_start = bq.position.as_ref().unwrap().start.line;
    let text = source.lines().nth(line_start - 1).unwrap_or("");
    // Pattern: ">" symbol followed by one space and one or more whitespace
    //          followed by any non-whitespace character
    Regex::new(r">\s\s+\S").unwrap().is_match(text)
}

pub fn md027_multiple_spaces_after_block_quote_symbol(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD027] File: {:#?}", &file.path);

    let ast = parse(&file.content).unwrap();

    // Get all block quotes
    let mut block_quotes: Vec<&BlockQuote> = vec![];
    for_each(&ast, |node| {
        if let Node::BlockQuote(bq) = node {
            block_quotes.push(bq);
        }
    });
    log::debug!("[MD027] Block quotes: {:#?}", &block_quotes);

    block_quotes
        .iter()
        .filter(|bq| has_multiple_spaces_after_bq_symbol(bq, &file.content))
        .map(|bq| {
            violation_builder()
                .position(&bq.position)
                .push_fix("Remove any extraneous space after the \">\" symbol")
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md027() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: ">  This is a block quote with bad indentation".to_string(),
            issues: vec![],
        };
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(1, 1, 0, 1, 46, 45)))
                .build()],
            md027_multiple_spaces_after_block_quote_symbol(&file)
        );
    }
}
