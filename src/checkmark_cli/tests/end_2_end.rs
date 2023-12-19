use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("checkmark_cli")?;

    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "CLI tool that help keep your Markdown documentation at hight quality.",
    ));

    Ok(())
}

#[test]
fn fmt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("checkmark_cli")?;

    cmd.arg("fmt")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Formatting tool"));

    Ok(())
}

#[test]
#[ignore = "Involves real HTTP req to OpenAI which costs money + unstable. Use manual invocation and verification."]
fn cli_grammar() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::cargo_bin("checkmark_cli")?;

    cmd.arg("grammar")
        .arg("./data/end_to_end.md")
        .assert()
        .success()
        .stdout(predicate::str::contains("Formatting tool"));

    Ok(())
}
