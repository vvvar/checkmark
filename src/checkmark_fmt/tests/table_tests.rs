mod utils;

/// Tables
#[test]
fn table() {
    utils::assert_changed_after_formatting(
        "# Table

| Syntax | Description | Something Else | Opapa [Here](./Cargo.lock) |
| :-- | -----: | :--: | ---- |
| Header | Title | fdfdsf | On ho! |
| Paragraph | Text | dfdfdfdfdfdfdfdfdfdfdfdfdfddsf | Lol |
",
        "# Table

| Syntax    | Description | Something Else                 | Opapa [Here](./Cargo.lock) |
| :-------- | ----------: | :----------------------------: | -------------------------- |
| Header    | Title       | fdfdsf                         | On ho!                     |
| Paragraph | Text        | dfdfdfdfdfdfdfdfdfdfdfdfdfddsf | Lol                        |
",
    );
}
