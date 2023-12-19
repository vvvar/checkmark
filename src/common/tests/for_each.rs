#[ignore = "I don't want spend time implementing Eq for mdast. Use manual invocation and verification."]
#[test]
fn for_each() {
    let ast = markdown::to_mdast(
        "# This is a header

    And this is a paragraph.",
        &markdown::ParseOptions::gfm(),
    )
    .unwrap();

    let mut iterated_items: Vec<markdown::mdast::Node> = vec![];

    common::for_each(&ast, |el| {
        iterated_items.push(el.clone());
    });

    dbg!(&iterated_items);

    assert!(false);
}
