#[test]
fn test_my_fn() {
    assert_eq!(
        common::find_offset_by_line_number(
            "1234
1234
1234
1234
1234

Here all fine

## H2 - All good here as well",
            5
        ),
        25
    );
}
