mod utils;

/// Check that valid single level list preserved
#[test]
fn single_level_list_preserved() {
    utils::assert_unchanged_after_formatting("- First\n- Second\n- Third\n");
}

/// Check that valid single level list preserved
#[test]
fn consequent_double_level_list_preserved() {
    utils::assert_unchanged_after_formatting(
        "- First
  - First_One
  - First_Two
- Second
  - Second_One
  - Second_Two
  - Second_Three
- Third
  - Third_One
",
    );
}

/// Check that when one nested list follows another with newline(called spread list)
/// then it's structure is not corrupted e.g. newline preserved
#[test]
fn consequent_spread_nested_lists_preserved() {
    utils::assert_unchanged_after_formatting(
        "- First
  - FirstOne
  - FirstTwo

- Second
  - SecondOne
  - SecondTwo
  - SecondThree

- Third
  - ThirdOne
",
    );
}

/// Simple ordered list rendered correctly
#[test]
fn simple_ordered_list_preserved() {
    utils::assert_unchanged_after_formatting(
        "1. One
2. Two
3. Three
",
    );
}

/// When incorrect order provided, we want to correct it
#[test]
fn ordered_tight_and_lose_lists_with_wrong_numbers() {
    // Here, first list is interpreted as loose
    // since it's at lease one el is split by newline
    utils::assert_changed_after_formatting(
        "Ordered loose list

1. Lorem ipsum dolor sit amet
2. Consectetur adipiscing elit
3. Integer molestie lorem at massa


1. You can use sequential numbers...
1. ...or keep all the numbers as `1.`

Ordered tight list:

57. foo
1. bar",
        "Ordered loose list

1. Lorem ipsum dolor sit amet

2. Consectetur adipiscing elit

3. Integer molestie lorem at massa

4. You can use sequential numbers...

5. ...or keep all the numbers as `1.`

Ordered tight list:

57. foo
58. bar
",
    );
}

/// Ordered lists can be mixed with unordered
#[test]
fn mix_ordered_list_with_unordered() {
    utils::assert_unchanged_after_formatting(
        "1. One
2. Two

- Three
- Four
",
    );
}

#[test]
fn list_with_multiple_paragraphs() {
    utils::assert_unchanged_after_formatting(
        "1. This is a list item with two paragraphs. Lorem ipsum dolor

   sit amet, consectetuer adipiscing elit. Aliquam hendrerit

   mi posuere lectus.
",
    );
}

#[test]
fn list_with_multiple_text_lines() {
    utils::assert_changed_after_formatting(
        "- __[pica](https://nodeca.github.io/pica/demo/)__ - high quality and fast image
  resize in browser.
- __[babelfish](https://github.com/nodeca/babelfish/)__ - developer friendly
  i18n with plurals support and easy syntax.
",
        "- **[pica](https://nodeca.github.io/pica/demo/)** - high quality and fast image
  resize in browser.
- **[babelfish](https://github.com/nodeca/babelfish/)** - developer friendly
  i18n with plurals support and easy syntax.
",
    );
}

#[test]
fn list_with_two_items_and_code_like_character() {
    utils::assert_unchanged_after_formatting(
        "- `one
- two`
",
    );
}

#[test]
fn code_in_list() {
    // Ordered
    utils::assert_unchanged_after_formatting(
        "1. List item with code block associated with it:

    ```javascript
    console.log('hello world');
    ```
",
    );

    utils::assert_unchanged_after_formatting(
        "1. List item with code block NOT associated with it:

```javascript
console.log('hello world');
```
",
    );
    utils::assert_unchanged_after_formatting(
        "1. One:

    ```javascript
    console.log('hello world');
    ```

2. Two:

    ```javascript
    console.log('hello world');
    ```
",
    );

    // Unordered
    utils::assert_unchanged_after_formatting(
        "- List item with code block associated with it:

   ```javascript
   console.log('hello world');
   ```
",
    );
    utils::assert_unchanged_after_formatting(
        "- List item with code block NOT associated with it:

```javascript
console.log('hello world');
```
",
    );
    utils::assert_unchanged_after_formatting(
        "- One:

   ```javascript
   console.log('hello world');
   ```

- Two:

   ```javascript
   console.log('hello world');
   ```
",
    );
}

#[test]
fn list_with_checkboxes() {
    utils::assert_unchanged_after_formatting(
        "# List with checkboxes

- [ ] Port remaining [markdownlint](https://github.com/DavidAnson/markdownlint) rules
- [ ] Provide a package via crates.io
- [ ] Provide pre-built packages via `brew`, `choco` and `apt`
",
    );
}
