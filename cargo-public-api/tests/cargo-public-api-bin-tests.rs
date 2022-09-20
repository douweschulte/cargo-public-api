// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

//! To update expected output it is in many cases sufficient to run
//! ```bash
//! ./scripts/bless-expected-output-for-tests.sh
//! ```

use std::ffi::OsStr;
use std::io::Write;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use assert_cmd::assert::Assert;
use assert_cmd::Command;
use predicates::str::contains;

// rust-analyzer bug: https://github.com/rust-lang/rust-analyzer/issues/9173
#[path = "../../test-utils/src/lib.rs"]
mod test_utils;
use test_utils::rustdoc_json_path_for_crate;

#[path = "../src/git_utils.rs"] // Say NO to copy-paste!
mod git_utils;

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn list_public_items() {
    let mut cmd = TestCmd::new();
    cmd.assert()
        .stdout(include_str!(
            "../../public-api/tests/expected-output/example_api-v0.3.0.txt"
        ))
        .success();
}

#[test]
fn list_public_items_with_lint_error() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.args(["--manifest-path", "../test-apis/lint_error/Cargo.toml"]);
    cmd.assert()
        .stdout(include_str!("./expected-output/lint_error_list.txt"))
        .success();
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn custom_toolchain() {
    let mut cmd = TestCmd::new();
    cmd.arg("--toolchain");
    cmd.arg("+nightly");
    cmd.assert()
        .stdout(include_str!(
            "../../public-api/tests/expected-output/example_api-v0.3.0.txt"
        ))
        .success();
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn list_public_items_explicit_manifest_path() {
    let test_repo = TestRepo::new();
    let mut test_repo_manifest = PathBuf::from(test_repo.path());
    test_repo_manifest.push("Cargo.toml");

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(&test_repo_manifest);
    cmd.assert()
        .stdout(include_str!(
            "../../public-api/tests/expected-output/example_api-v0.3.0.txt"
        ))
        .success();
}

/// Make sure we can run the tool with a specified package from a virtual
/// manifest. Use the smallest crate in our workspace to make tests run fast
#[test]
fn list_public_items_via_package_spec() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--package");
    cmd.arg("rustdoc-json");
    cmd.assert()
        .stdout(include_str!("./expected-output/rustdoc_json_list.txt"))
        .success();
}

#[test]
fn target_arg() {
    // A bit of a hack but similar to how rustc bootstrap script does it:
    // https://github.com/rust-lang/rust/blob/1ce51982b8550c782ded466c1abff0d2b2e21c4e/src/bootstrap/bootstrap.py#L207-L219
    fn get_host_target_triple() -> String {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c");
        cmd.arg("rustc -vV | sed -n 's/host: \\(.*\\)/\\1/gp'");
        let stdout = cmd.output().unwrap().stdout;
        String::from_utf8_lossy(&stdout)
            .to_string()
            .trim()
            .to_owned()
    }

    // Make sure to use a separate and temporary repo so that this test does not
    // accidentally pass due to files from other tests lying around
    let mut cmd = TestCmd::new();
    cmd.arg("--target");
    cmd.arg(get_host_target_triple());
    cmd.assert()
        .stdout(include_str!("./expected-output/test_repo_api_latest.txt"))
        .success();
}

#[test]
fn virtual_manifest_error() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(current_dir_and("tests/virtual-manifest/Cargo.toml"));
    cmd.assert()
        .stdout("")
        .stderr(contains(
            "Listing or diffing the public API of an entire workspace is not supported.",
        ))
        .failure();
}

#[test]
fn diff_public_items() {
    let mut cmd = TestCmd::new();
    let test_repo_path = cmd.test_repo_path().to_owned();
    let branch_before = git_utils::current_branch(&test_repo_path).unwrap().unwrap();
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"
        ))
        .success();
    let branch_after = git_utils::current_branch(&test_repo_path).unwrap().unwrap();

    // Diffing does a git checkout of the commits to diff. Afterwards the
    // original branch shall be restored to minimize user disturbance.
    assert_eq!(branch_before, branch_after);
}

/// Test that the mechanism to restore the original git branch works even if
/// there is no current branch
#[test]
fn diff_public_items_detached_head() {
    let test_repo = TestRepo::new();

    // Detach HEAD
    let path = test_repo.path();
    git_utils::git_checkout("v0.1.1", path, true).unwrap();
    assert_eq!(None, git_utils::current_branch(path).unwrap());
    let before = git_utils::current_commit(path).unwrap();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(path);
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"
        ))
        .success();

    let after = git_utils::current_commit(path).unwrap();
    assert_eq!(before, after);
}

/// Test that diffing fails if the git tree is dirty
#[test]
#[cfg_attr(target_family = "windows", ignore)]
fn diff_public_items_with_dirty_tree_fails() {
    let test_repo = TestRepo::new();

    // Make the tree dirty by appending a comment to src/lib.rs
    let mut lib_rs_path = test_repo.path.path().to_owned();
    lib_rs_path.push("src/lib.rs");

    let mut lib_rs = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&lib_rs_path)
        .unwrap();

    writeln!(lib_rs, "// Make git tree dirty").unwrap();

    // Make sure diffing does not destroy uncommitted data!
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stderr(contains(
            "Your local changes to the following files would be overwritten by checkout",
        ))
        .failure();
}

#[test]
fn deny_when_not_diffing() {
    let mut cmd = TestCmd::new();
    cmd.arg("--deny=all");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_added_when_not_diffing() {
    let mut cmd = TestCmd::new();
    cmd.arg("--deny=added");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_changed_when_not_diffing() {
    let mut cmd = TestCmd::new();
    cmd.arg("--deny=changed");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_removed_when_not_diffing() {
    let mut cmd = TestCmd::new();
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_combination_when_not_diffing() {
    let mut cmd = TestCmd::new();
    cmd.arg("--deny=added");
    cmd.arg("--deny=changed");
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_without_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.1.1");
    cmd.arg("--deny=all");
    cmd.assert().success();
}

#[test]
fn deny_with_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=all");
    cmd.assert()
        .stderr(contains("The API diff is not allowed as per --deny"))
        .failure();
}

#[test]
fn deny_added_with_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=added");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.1.0_to_v0.2.0.txt"
        ))
        .failure();
}

#[test]
fn deny_changed_with_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=changed");
    cmd.assert().failure();
}

#[test]
fn deny_removed_with_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains(
            "The API diff is not allowed as per --deny: Removed items not allowed: [pub fn example_api::function(v1_param: Struct, v2_param: usize)]",
        ))
        .failure();
}

#[test]
fn deny_with_invalid_arg() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--deny=invalid");
    cmd.assert()
        .stderr(contains("\"invalid\" isn't a valid value"))
        .failure();
}

#[test]
fn diff_public_items_with_manifest_path() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(format!(
        "{}/Cargo.toml",
        &test_repo.path.path().to_string_lossy()
    ));
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"
        ))
        .success();
}

#[test]
fn diff_public_items_without_git_root() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg("/does/not/exist/Cargo.toml");
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stderr(predicates::str::starts_with(
            "Error: No `.git` dir when starting from `",
        ))
        .failure();
}

#[test]
fn diff_public_items_with_color() {
    let mut cmd = TestCmd::new();
    cmd.arg("--color=always");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.1.0_to_v0.2.0_colored.txt"
        ))
        .success();
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn list_public_items_with_color() {
    let mut cmd = TestCmd::new();
    cmd.arg("--color=always");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_v0.3.0_colored.txt"
        ))
        .success();
}

#[test]
fn diff_public_items_from_files() {
    let old = rustdoc_json_path_for_crate("../test-apis/example_api-v0.1.0");
    let new = rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0");
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--diff-rustdoc-json");
    cmd.arg(old);
    cmd.arg(new);
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.1.0_to_v0.2.0.txt"
        ))
        .success();
}

#[test]
fn diff_public_items_missing_one_arg() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stderr(contains(
            "requires at least 2 values but only 1 was provided",
        ))
        .failure();
}

#[test]
fn verbose() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg("../test-apis/lint_error/Cargo.toml");
    cmd.arg("--verbose");
    cmd.assert()
        .stdout(contains("Processing \""))
        .stdout(contains("rustdoc JSON missing referenced item"))
        .success();
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--help");
    assert_presence_of_args_in_help(cmd);
}

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("-h");
    assert_presence_of_args_in_help(cmd);
}

fn assert_presence_of_args_in_help(mut cmd: Command) {
    cmd.assert()
        .stdout(contains("--with-blanket-implementations"))
        .stdout(contains("--manifest-path"))
        .stdout(contains("--diff-git-checkouts"))
        .success();
}

/// Helper to get the absolute path to a given path, relative to the current
/// path
fn current_dir_and<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut cur_dir = std::env::current_dir().unwrap();
    cur_dir.push(path);
    cur_dir
}

/// Helper to initialize a test crate git repo. Each test gets its own git repo
/// to use so that tests can run in parallel.
fn initialize_test_repo(dest: &Path) {
    test_utils::create_test_git_repo(dest, "../test-apis");
}

#[test]
fn cargo_public_api_with_features() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug)]
    struct F<'a> {
        all: bool,
        none: bool,
        features: &'a [&'a str],
    }

    impl<'a> F<'a> {
        fn none(mut self) -> Self {
            self.none = true;
            self
        }
        fn all(mut self) -> Self {
            self.all = true;
            self
        }
        fn new(features: &'a [&'a str]) -> Self {
            F {
                all: false,
                none: false,
                features,
            }
        }
    }

    impl std::fmt::Display for F<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.all {
                write!(f, "all")?;
            }
            if self.none {
                write!(f, "none")?;
            }
            for feat in self.features {
                write!(f, "{feat}")?;
            }
            Ok(())
        }
    }

    let root = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()?
        .workspace_root;

    for features in [
        F::new(&[]).all(),
        F::new(&[]).none(),
        F::new(&["feature_a", "feature_b", "feature_c"]).none(),
        F::new(&["feature_b"]).none(),
        F::new(&["feature_c"]).none(), // includes `feature_b`
    ] {
        let expected_file = root.join(format!(
            "cargo-public-api/tests/expected-output/features-feat{features}.txt"
        ));

        let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
        cmd.current_dir(root.join("test-apis/features"));

        if features.none {
            cmd.arg("--no-default-features");
        }

        if features.all {
            cmd.arg("--all-features");
        }

        for feature in features.features {
            cmd.args(["--features", feature]);
        }

        if std::env::var("BLESS").is_ok() {
            let out = cmd.output().unwrap();
            std::fs::write(expected_file, out.stdout).unwrap();
        } else {
            // Make into a string to show diff
            let expected = String::from_utf8(
                std::fs::read(&expected_file)
                    .unwrap_or_else(|_| panic!("couldn't read file: {expected_file:?}")),
            )
            .unwrap();
            cmd.assert().stdout(expected).success();
        }
    }
    Ok(())
}

/// A git repository that lives during the duration of a test. Having each test
/// have its own git repository to test with makes tests runnable concurrently.
struct TestRepo {
    path: tempfile::TempDir,
}

impl TestRepo {
    fn new() -> Self {
        let tempdir = tempfile::tempdir().unwrap();
        initialize_test_repo(tempdir.path());

        Self { path: tempdir }
    }

    fn path(&self) -> &Path {
        self.path.path()
    }
}

/// Frequently a test needs to create a test repo and then run
/// `cargo-public-api` on that repo. This helper constructs such a pair and
/// pre-configures it, so that tests becomes shorter and more to-the-point.
///
/// It comes with a bunch of convenience methods ([`Self::arg()`], etc) to make
/// test code simpler.
struct TestCmd {
    /// `cargo-public-api`
    cmd: Command,

    /// A short-lived temporary git repo used for tests. Each test typically has
    /// its own repo so that tests can run in parallel.
    test_repo: TestRepo,
}

impl TestCmd {
    fn new() -> Self {
        let test_repo = TestRepo::new();

        let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
        cmd.current_dir(&test_repo.path);

        Self { cmd, test_repo }
    }

    pub fn test_repo_path(&self) -> &Path {
        self.test_repo.path()
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    pub fn assert(&mut self) -> Assert {
        self.cmd.assert()
    }
}
