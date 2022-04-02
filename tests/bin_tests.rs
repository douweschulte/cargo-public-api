use std::path::{Path, PathBuf};

use assert_cmd::Command;

#[test]
fn list_public_items() {
    let cmd = Command::cargo_bin("cargo-public-items").unwrap();
    assert_presence_of_own_library_items(cmd);
}

#[test]
fn list_public_items_explicit_manifest_path() {
    let mut cmd = Command::cargo_bin("cargo-public-items").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(current_dir_and("Cargo.toml"));
    assert_presence_of_own_library_items(cmd);
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("cargo-public-items").unwrap();
    cmd.arg("--help");
    assert_presence_of_args_in_help(cmd);
}

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("cargo-public-items").unwrap();
    cmd.arg("-h");
    assert_presence_of_args_in_help(cmd);
}

fn assert_presence_of_own_library_items(mut cmd: Command) {
    cmd.assert()
        .stdout(
            "pub fn cargo_public_items::for_self_testing_purposes_please_ignore()\n\
             pub mod cargo_public_items\n\
             ",
        )
        .success();
}

fn assert_presence_of_args_in_help(mut cmd: Command) {
    cmd.assert()
        .stdout(predicates::str::contains("--with-blanket-implementations"))
        .stdout(predicates::str::contains("--manifest-path"))
        .stdout(predicates::str::contains("--diff-git-checkouts"))
        .success();
}

fn current_dir_and<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut cur_dir = std::env::current_dir().unwrap();
    cur_dir.push(path);
    cur_dir
}

fn clone_test_crate() {
    let mut git = std::process::Command::new("git");
    git.arg("clone");
    git.arg("https://github.com/Enselic/public_items.git");
    git.arg(test_crate_path());
    git.spawn().unwrap().wait().unwrap().success()
}

fn test_crate_path() -> PathBuf {
    let path = PathBuf::from(option_env!("CARGO_TARGET_DIR").unwrap_or("/tmp"));
    path.push("cargo-public-items-test-repo");
    path
}
