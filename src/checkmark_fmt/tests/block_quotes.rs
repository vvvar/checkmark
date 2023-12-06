mod utils;

/// Block quote(strikethrough)
#[test]
fn block_quote() {
    utils::assert_unchanged_after_formatting(
        "> In a few moments he was barefoot, his stockings folded in his pockets and his
> canvas shoes dangling by their knotted laces over his shoulders and, picking a
> pointed salt-eaten stick out of the jetsam among the rocks, he clambered down
> the slope of the breakwater.");

    utils::assert_unchanged_after_formatting(
        "> This is a main quote
> 
> > And this is a nested one
> > and this is a continuation of the nested one");

utils::assert_unchanged_after_formatting(
    "> This is a main quote
> And this is a list in it:
> 
> + One
> + Two");
}
