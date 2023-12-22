fn get_test_file_path() -> String {
    let mut test_file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_file_path.push("tests/data/basic.md");
    return test_file_path
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
}

/// Check links
#[tokio::test]
async fn link_check() {
    let mut markdown = common::MarkDownFile {
        path: get_test_file_path(),
        content: String::from(include_str!("data/basic.md")),
        issues: vec![],
    };

    checkmark_link_checker::check_links(&mut markdown, &vec![]).await;

    assert_eq!(&markdown.issues, &vec![
        common::CheckIssue {
            category: common::IssueCategory::LinkChecking,
            severity: common::IssueSeverity::Warning,
            file_path: "/Users/vvoinov/Documents/repos/md-checker/src/checkmark_link_checker/tests/data/basic.md".to_string(),
            row_num_start: 5,
            row_num_end: 5,
            col_num_start: 8,
            col_num_end: 1,
            offset_start: 0,
            offset_end: 104,
            message: "http://gffffffffffoooooogel.com: error sending request for url (http://gffffffffffoooooogel.com/): error trying to connect: dns error: no record found for Query { name: Name(\"gffffffffffoooooogel.com.\"), query_type: AAAA, query_class: IN }".to_string(),
            fixes: vec![
                "Can you open this link in a browser? If no then perhaps its broken".to_string(),
                "Is there internet connection?".to_string(),
                "Are you using proxy? Consider setting HTTP_PROXY and/or HTTPS_PROXY env variables".to_string(),
            ],
        }
    ]);
}
