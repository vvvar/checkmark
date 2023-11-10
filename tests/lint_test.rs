use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

mod utils;

#[test]
fn file_does_not_start_with_h1_heading() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = utils::get_cmd();
    cmd.arg(utils::get_data_root_path().join("file_does_not_start_with_h1_heading.md"));
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "file_does_not_start_with_h1_heading.md:1",
        ))
        .stderr(predicate::str::contains(
            "Lint: First line in a file should be a top-level heading",
        ));

    Ok(())
}
