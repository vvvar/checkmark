use checkmark_lint_common::*;
use checkmark_lint_macro::*;

#[rule(
    requirement = "Block quote should not have blank lines",
    rationale = "Some parsers will treat two block quotes separated by one or more blank lines as the same block quote, while others will treat them as separate block quotes",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md028.md",
    additional_links = [],
    is_fmt_fixable = false,
)]
fn md028(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    // Get all block quotes
    let block_quotes = common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_block_quote(n))
        .collect::<Vec<&Blockquote>>();

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

    violations
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Found a blank line inside block quote")
        .assertion("Expected code block to be a single one, got a separation with a blank line")
        .push_fix("If you want to have a single block quote - remove blank line")
        .push_fix("If you want to have block quotes split - add any text between them, for example an empty comment \"<!--  -->\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "
> Block quote 1
> > Block quote 1.2
    
> Block quote 2


> Block quote 3

Here some text

> Block quote 4")]
    fn detect_block_quotes_with_blank_lines(ast: &Node, file: &MarkDownFile, config: &Config) {
        assert_eq!(
            vec![
                violation_builder()
                    .position(&Some(Position::new(4, 1, 37, 4, 1, 41)))
                    .build(),
                violation_builder()
                    .position(&Some(Position::new(6, 1, 58, 7, 1, 59)))
                    .build()
            ],
            MD028.check(ast, file, config)
        );
    }
}
