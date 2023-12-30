mod utils;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestCase {
    pub markdown: String,
    pub example: i32,
    pub section: String,
}

/// Check tat CommonMark tests passes
/// https://spec.commonmark.org/0.30/spec.json
#[test]
fn common_mark_end_2_end() {
    let ignore_test_cases = [
        2, 3, // Duplicates of other test-cases, but with formatting that is stripped
    ];
    let ignore_sections = [
        "Tabs",
        "Backslash escapes",
        "Entity and numeric character references",
        "Precedence",
        "Thematic breaks",
        "ATX headings",
        "Setext headings",
        "Indented code blocks",
        "Fenced code blocks",
        "HTML blocks",
        "Link reference definitions",
        "Paragraphs",
        "Blank lines",
        "Block quotes",
        "List items",
        "Lists",
        "Code spans",
        "Emphasis and strong emphasis",
        "Links",
        "Images",
        "Autolinks",
        "Raw HTML",
        "Hard line breaks",
        "Soft line breaks",
    ];
    let mut ignored_test_cases = 0;
    for test_case in
        serde_json::from_str::<Vec<TestCase>>(include_str!("data/common_mark.json")).unwrap()
    {
        if !ignore_test_cases.contains(&test_case.example)
            && !ignore_sections.contains(&test_case.section.as_str())
        {
            let original = utils::create_dummy_md_file(&test_case.markdown);
            let formatted = checkmark_fmt::fmt_markdown(&original);
            assert_eq!(
                original.content, formatted.content,
                "Testing test case #{}, section: {}",
                &test_case.example, &test_case.section
            );
        } else {
            ignored_test_cases += 1;
        }
    }
    println!("Ignored test cases from CommonMark: {}", ignored_test_cases);
}
