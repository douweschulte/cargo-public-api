use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serial_test::serial;

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

// We must serially run tests that touch the test crate git repo to prevent
// ".git/index.lock: File exists"-errors.
#[serial]
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

#[serial]
#[test]
fn diff_public_items_with_color() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-items").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--color=always");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.6.0");
    cmd.arg("v0.7.1");
    cmd.assert()
        .stdout(
            "Removed items from the public API\n\
             =================================\n\
             \x1b[31mpub fn public_items::PublicItem::hash<__H: $crate::hash::Hasher>(&self, state: &mut __H) -> ()\x1b[0m\n\
             \x1b[31mpub fn public_items::diff::PublicItemsDiff::print_with_headers(&self, w: &mut impl std::io::Write, header_removed: &str, header_changed: &str, header_added: &str) -> std::io::Result<()>\x1b[0m\n\
             \n\
             Changed items in the public API\n\
             ===============================\n\
             \x1b[31mpub fn public_items::PublicItem::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result\x1b[0m\n\
             \x1b[32mpub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result\x1b[0m\n\
             \x1b[31mpub fn public_items::diff::PublicItemsDiff::between(old: Vec<PublicItem>, new: Vec<PublicItem>) -> Self\x1b[0m\n\
             \x1b[32mpub fn public_items::diff::PublicItemsDiff::between(old_items: Vec<PublicItem>, new_items: Vec<PublicItem>) -> Self\x1b[0m\n\
             \n\
             Added items to the public API\n\
             =============================\n\
             \x1b[32mpub fn public_items::diff::ChangedPublicItem::cmp(&self, other: &ChangedPublicItem) -> $crate::cmp::Ordering\x1b[0m\n\
             \x1b[32mpub fn public_items::diff::ChangedPublicItem::eq(&self, other: &ChangedPublicItem) -> bool\x1b[0m\n\
             \x1b[32mpub fn public_items::diff::ChangedPublicItem::ne(&self, other: &ChangedPublicItem) -> bool\x1b[0m\n\
             \x1b[32mpub fn public_items::diff::ChangedPublicItem::partial_cmp(&self, other: &ChangedPublicItem) -> $crate::option::Option<$crate::cmp::Ordering>\x1b[0m\n\
             \x1b[32mpub fn public_items::diff::PublicItemsDiff::eq(&self, other: &PublicItemsDiff) -> bool\x1b[0m\n\
             \x1b[32mpub fn public_items::diff::PublicItemsDiff::ne(&self, other: &PublicItemsDiff) -> bool\x1b[0m\n\
            \n\
            ",
        )
        .success();
}

#[serial]
#[test]
fn diff_public_items_markdown() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-items").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--output-format=markdown");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.6.0");
    cmd.arg("v0.7.1");
    cmd.assert()
        .stdout(
            "Removed items from the public API\n\
             =================================\n\
             -pub fn public_items::PublicItem::hash<__H: $crate::hash::Hasher>(&self, state: &mut __H) -> ()\n\
             -pub fn public_items::diff::PublicItemsDiff::print_with_headers(&self, w: &mut impl std::io::Write, header_removed: &str, header_changed: &str, header_added: &str) -> std::io::Result<()>\n\
             \n\
             Changed items in the public API\n\
             ===============================\n\
             -pub fn public_items::PublicItem::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result\n\
             +pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result\n\
             -pub fn public_items::diff::PublicItemsDiff::between(old: Vec<PublicItem>, new: Vec<PublicItem>) -> Self\n\
             +pub fn public_items::diff::PublicItemsDiff::between(old_items: Vec<PublicItem>, new_items: Vec<PublicItem>) -> Self\n\
             \n\
             Added items to the public API\n\
             =============================\n\
             +pub fn public_items::diff::ChangedPublicItem::cmp(&self, other: &ChangedPublicItem) -> $crate::cmp::Ordering\n\
             +pub fn public_items::diff::ChangedPublicItem::eq(&self, other: &ChangedPublicItem) -> bool\n\
             +pub fn public_items::diff::ChangedPublicItem::ne(&self, other: &ChangedPublicItem) -> bool\n\
             +pub fn public_items::diff::ChangedPublicItem::partial_cmp(&self, other: &ChangedPublicItem) -> $crate::option::Option<$crate::cmp::Ordering>\n\
             +pub fn public_items::diff::PublicItemsDiff::eq(&self, other: &PublicItemsDiff) -> bool\n\
             +pub fn public_items::diff::PublicItemsDiff::ne(&self, other: &PublicItemsDiff) -> bool\n\
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

/// Helper to get the absolute path to a given path, relative to the current
/// path
fn current_dir_and<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut cur_dir = std::env::current_dir().unwrap();
    cur_dir.push(path);
    cur_dir
}

/// Helper to clone the test crate git repo to the proper place
fn clone_test_crate() {
    let mut git = std::process::Command::new("git");
    git.arg("clone");
    git.arg("https://github.com/Enselic/public_items.git");
    git.arg(test_crate_path());
    assert!(git.spawn().unwrap().wait().unwrap().success());
}

/// Path to the git cloned test crate we use to test the diffing functionality
fn test_crate_path() -> PathBuf {
    let mut path = get_cache_dir();
    path.push("cargo-public-items-test-repo");
    path
}

/// Where to put things that survives across tests runs. For example a git
/// cloned test crate. We don't want to clone it every time we run tests. We
/// want to clone it just once.
fn get_cache_dir() -> PathBuf {
    option_env!("CARGO_TARGET_DIR")
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| std::env::temp_dir())
}
