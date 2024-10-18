use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::Blockquote;
use markdown::unist::Position;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD028")
        .message("Blank line inside block quote")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md028.md")
        .rationale("Some Markdown parsers will treat two block quotes separated by one or more blank lines as the same block quote, while others will treat them as separate block quotes")
        .push_fix("If you want to have a single block quote - remove blank line")
        .push_fix("If you want to have block quotes split - add any text between them, for example an empty comment \"<!--  -->\"")
}

pub fn md028_blank_line_inside_block_quote(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD028] File: {:#?}", &file.path);

    let ast = common::ast::parse(&file.content).unwrap();

    // Get all block quotes
    let block_quotes = common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_block_quote(n))
        .collect::<Vec<&Blockquote>>();
    log::debug!("[MD028] Block quotes(in sequence): {:#?}", &block_quotes);

    let mut violations: Vec<Violation> = vec![];
    for (i, current_block_quote) in block_quotes.iter().enumerate() {
        if let Some(next_block_quote) = block_quotes.get(i + 1) {
            let current_block_quote_end_offset =
                current_block_quote.position.as_ref().unwrap().end.offset;
            let next_block_quote_start_offset =
                next_block_quote.position.as_ref().unwrap().start.offset;
            let text_between_block_quotes = file
                .content
                .get(current_block_quote_end_offset..next_block_quote_start_offset)
                .unwrap_or("Blank");
            if text_between_block_quotes
                .replace(['\n', ' '], "")
                .is_empty()
            {
                violations.push(
                    violation_builder()
                        .position(&Some(Position::new(
                            current_block_quote.position.as_ref().unwrap().end.line + 1,
                            1,
                            current_block_quote_end_offset + 1,
                            next_block_quote.position.as_ref().unwrap().start.line - 1,
                            1,
                            next_block_quote_start_offset - 1,
                        )))
                        .build(),
                );
            }
        }
    }
    log::debug!("[MD051] Violations: {:#?}", &violations);

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md028() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "
> Block quote 1
> > Block quote 1.2
    
> Block quote 2


> Block quote 3

Here some text

> Block quote 4"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(Position::new(4, 1, 37, 4, 1, 41)))
                    .build(),
                violation_builder()
                    .position(&Some(Position::new(6, 1, 58, 7, 1, 59)))
                    .build()
            ],
            md028_blank_line_inside_block_quote(&file)
        );
    }
}
