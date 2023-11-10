use assert_cmd::prelude::*;
use std::{path::Path, process::Command}; // Run program

pub fn get_cmd() -> Command {
    Command::cargo_bin("marklint").unwrap()
}

pub fn get_data_root_path() -> std::path::PathBuf {
    Path::new(".").canonicalize().unwrap().join("tests/data")
}
