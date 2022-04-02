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
fn diff_public_items() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-items").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.0.4");
    cmd.arg("v0.0.5");
    cmd.assert()
        .stdout(
            "Removed items from the public API\n\
             =================================\n\
             -pub fn public_items::from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<HashSet<String>>\n\
             \n\
             Changed items in the public API\n\
             ===============================\n\
             (none)\n\
             \n\
             Added items to the public API\n\
             =============================\n\
             +pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<String>>\n\
             \n\
            ",
        )
        .success();
}

fn ensure_test_crate_is_cloned() {
    if !test_crate_path().exists() {
        clone_test_crate();
    }
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
    assert!(git.spawn().unwrap().wait().unwrap().success());
}

fn test_crate_path() -> PathBuf {
    let mut path = PathBuf::from(option_env!("CARGO_TARGET_DIR").unwrap_or("/tmp"));
    path.push("cargo-public-items-test-repo");
    path
}
