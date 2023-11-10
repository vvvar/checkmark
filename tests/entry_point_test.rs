use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

mod utils;

#[test]
fn single_file_can_be_supplied() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = utils::get_cmd();

    let test_file_path = String::from(
        utils::get_data_root_path()
            .join("correct_files/correct_basic.md")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    cmd.arg(&test_file_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&test_file_path));

    Ok(())
}

#[test]
fn dir_can_be_supplied() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = utils::get_cmd();

    let test_file_dir_path = String::from(
        utils::get_data_root_path()
            .join("correct_files")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    cmd.arg(&test_file_dir_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("correct_files/correct_basic.md"))
        .stdout(predicate::str::contains(
            "correct_files/correct_basic_copy.md",
        ));

    Ok(())
}
