#[cfg(test)]
fn activate_logging() {
    std::env::set_var("RUST_LOG", "debug");
    if let Ok(_) = env_logger::try_init() {}
}

/// Check grammar check
// #[ignore = "Involves real HTTP req - unstable. Use manual invocation and verification."]
#[tokio::test]
async fn ls() {
    // activate_logging();
    let files = checkmark_ls::ls("https://github.com/google/googletest.git").await;
    assert_eq!(files.len(), 27); // There are some files in the repo
}
