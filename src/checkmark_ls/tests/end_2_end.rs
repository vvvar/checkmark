#[cfg(test)]
#[allow(dead_code)]
fn activate_logging() {
    std::env::set_var("RUST_LOG", "debug");
    if let Ok(_) = env_logger::try_init() {}
}

/// List files in folder
#[ignore = "Involves real HTTP req - unstable. Use manual invocation and verification."]
#[tokio::test]
async fn ls() {
    let files = checkmark_ls::ls("https://github.com/google/googletest.git", &vec![]).await;
    assert_eq!(files.len(), 27); // There are some files in the repo
}
