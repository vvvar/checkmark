mod utils;

/// Tables is automatically aligned
#[test]
fn table_auto_aligned() {
    utils::assert_changed_after_formatting(
        "# Table

| Syntax | Description | Something Else | Opapa [Here](./Cargo.lock) |
| :-- | -----: | :--: | ---- |
| Header | Title | text | On ho! |
| Paragraph | Text | dfdfdfdfdfdfdfdfdfdfdfdfdfddsf | Lol |
",
        "# Table

| Syntax    | Description | Something Else                 | Opapa [Here](./Cargo.lock) |
| :-------- | ----------: | :----------------------------: | -------------------------- |
| Header    | Title       | text                           | On ho!                     |
| Paragraph | Text        | dfdfdfdfdfdfdfdfdfdfdfdfdfddsf | Lol                        |
",
    );
}

#[test]
fn table_with_special_symbols() {
    // Here second row looks like miss-aligned because two "\\"
    // are just to escape the first one
    utils::assert_unchanged_after_formatting(
        "#Table

| Syntax | Description |
| ------ | ----------: |
| Header | Title       |
| -\\>    | Text        |
",
    );
}
