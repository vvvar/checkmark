use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("checkmark")?;

    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "A CLI tool that helps maintain high-quality Markdown documentation by checking for formatting, grammatical, and spelling errors, as well as broken links",
    ));

    Ok(())
}

#[test]
fn fmt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("checkmark")?;

    cmd.arg("fmt")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A tool for checking and correcting Markdown file formatting",
        ));

    Ok(())
}

#[test]
#[ignore = "Involves real HTTP req to OpenAI which costs money + unstable. Use manual invocation and verification."]
fn cli_grammar() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("checkmark")?;

    cmd.arg("grammar")
        .arg("./data/end_to_end.md")
        .assert()
        .success()
        .stdout(predicate::str::contains("Formatting tool"));

    Ok(())
}
