mod utils;

/// Block quote(strikethrough)
#[test]
fn block_quote() {
    utils::assert_unchanged_after_formatting(
        "> In a few moments he was barefoot, his stockings folded in his pockets and his
> canvas shoes dangling by their knotted laces over his shoulders and, picking a
> pointed salt-eaten stick out of the jetsam among the rocks, he clambered down
> the slope of the breakwater.
",
    );

    utils::assert_unchanged_after_formatting(
        "> This is a main quote
> 
> > And this is a nested one
> > and this is a continuation of the nested one
",
    );

    utils::assert_unchanged_after_formatting(
        "> This is a main quote
> And this is a list in it:
> 
> + One
> + Two
",
    );

    utils::assert_unchanged_after_formatting(
        "> This is a main quote
> And this is a list in it:
> 
> + One
> + Two
",
    );

    utils::assert_unchanged_after_formatting(
        "+ A list item with a blockquote:
  > This is a blockquote
  > inside a list item.
",
    );
}

/// Block quote with multiple paragraphs
#[test]
fn block_quote_multiple_paragraphs() {
    utils::assert_unchanged_after_formatting(
        "> This is the first paragraph in the block quote.
> 
> This is the second paragraph in the block quote.
",
    );
}

/// Block quote with a code block
#[test]
fn block_quote_with_code_block() {
    utils::assert_unchanged_after_formatting(
        "> This is a block quote with a code block:
> 
> ```rust
> fn main() {
>     println!(\"Hello, world!\");
> }
> ```
",
    );
}

/// Block quote with a nested block quote
#[test]
fn block_quote_with_nested_block_quote() {
    utils::assert_unchanged_after_formatting(
        "> This is a block quote.
> 
> > This is a nested block quote.
",
    );
}

/// Block quote with a list and a nested block quote
#[test]
fn block_quote_with_list_and_nested_block_quote() {
    utils::assert_unchanged_after_formatting(
        "> This is a block quote with a list:
> 
> + List item 1
> + List item 2
> 
> > And this is a nested block quote.
",
    );
}
