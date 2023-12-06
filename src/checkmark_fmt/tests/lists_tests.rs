mod utils;

/// Check that valid single level list preserved
#[test]
fn single_level_list_preserved() {
    utils::assert_unchanged_after_formatting("+ First\n+ Second\n+ Third\n");
}

/// Check that valid single level list preserved
#[test]
fn consequent_double_level_list_preserved() {
    utils::assert_unchanged_after_formatting(
        "+ First
  + First_One
  + First_Two
+ Second
  + Second_One
  + Second_Two
  + Second_Three
+ Third
  + Third_One
",
    );
}

/// Check that when one nested list follows another with newline(called spread list)
/// then it's structure is not corrupted e.g. newline preserved
#[test]
fn consequent_spread_nested_lists_preserved() {
    utils::assert_unchanged_after_formatting(
        "+ First
  + FirstOne
  + FirstTwo

+ Second
  + SecondOne
  + SecondTwo
  + SecondThree

+ Third
  + ThirdOne",
    );
}

/// Simple ordered list rendered correctly
#[test]
fn simple_ordered_list_preserved() {
    utils::assert_unchanged_after_formatting(
        "1. One
2. Two
3. Three",
    );
}

/// Ordered lists can be mixed with unordered
#[test]
fn mix_ordered_list_with_unordered() {
    utils::assert_unchanged_after_formatting(
        "1. One
2. Two

+ Three
+ Four",
    );
}

#[test]
fn list_with_multiple_paragraphs() {
    utils::assert_unchanged_after_formatting(
        "1. This is a list item with two paragraphs. Lorem ipsum dolor

    sit amet, consectetuer adipiscing elit. Aliquam hendrerit

    mi posuere lectus.",
    );
}

#[test]
fn list_with_two_items_and_code_like_character() {
    utils::assert_unchanged_after_formatting(
        "+ `one
+ two`",
    );
}
