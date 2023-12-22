/// Block quote(strikethrough)
#[test]
fn spell_check() {
    let mut markdown = common::MarkDownFile {
        path: String::from("this/is/a/dummy/path/to/a/file.md"),
        content: String::from(include_str!("data/basic.md")),
        issues: vec![],
    };
    checkmark_spelling::spell_check(&mut markdown);
    dbg!(&markdown);
    assert!(false);
}
