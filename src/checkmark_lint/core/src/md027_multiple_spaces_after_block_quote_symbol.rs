use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use regex::Regex;

#[rule(
    requirement = "Block quote symbol (>) should be followed by a single space",
    rationale = "Consistent formatting makes it easier to understand a document",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md027.md",
    additional_links = [],
    is_fmt_fixable = true,
)]
fn md027(ast: &Node, file: &MarkDownFile, _: &Config) -> Vec<Violation> {
    common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_block_quote(n))
        .filter(|bq| has_multiple_spaces_after_bq_symbol(bq, &file.content))
        .map(|bq| violation_builder().position(&bq.position).build())
        .collect::<Vec<Violation>>()
}

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .message("Multiple spaces after block quote symbol")
        .assertion("Expected a single space after the block quote symbol (>), got multiple")
        .push_fix("Remove any extraneous space after the \">\" symbol")
}

// Check that block quote has multiple spaces after bq symbol:
// Example: >  text
fn has_multiple_spaces_after_bq_symbol(bq: &Blockquote, source: &str) -> bool {
    let line_start = bq.position.as_ref().unwrap().start.line;
    let text = source.lines().nth(line_start - 1).unwrap_or("");
    // Pattern: ">" symbol followed by one space and one or more whitespace
    //          followed by any non-whitespace character
    Regex::new(r">\s\s+\S").unwrap().is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = ">  This is a block quote with bad indentation")]
    fn detect_multiple_spaces_after_block_quote_symbol(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(
            vec![violation_builder()
                .position(&Some(Position::new(1, 1, 0, 1, 46, 45)))
                .build()],
            MD027.check(ast, file, config)
        );
    }
}
