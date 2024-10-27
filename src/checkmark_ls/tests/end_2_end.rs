/// List files in folder
#[ignore = "Involves real HTTP req - unstable. Use manual invocation and verification."]
#[tokio::test]
async fn ls() {
    let tui = common::tui::CheckProgressTUI::new_thread_safe(true);
    let files = checkmark_ls::ls("https://github.com/google/googletest.git", &vec![], &tui).await;
    assert_eq!(files.len(), 27); // There are some files in the repo
}
