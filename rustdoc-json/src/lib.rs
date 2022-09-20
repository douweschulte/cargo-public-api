//! Utilities for working with rustdoc JSON.
//!
//! # Building
//!
//! Use [`build()`] to build rustdoc JSON. Like this:
//! ```
//!    use rustdoc_json::BuildOptions;
//!
//!    let json_path = rustdoc_json::build(
//!        BuildOptions::default()
//!            .toolchain("+nightly".to_owned())
//!            .manifest_path("Cargo.toml"),
//!    ).unwrap();
//!
//!    println!("Built and wrote rustdoc JSON to {:?}", &json_path);
//! ```
//!
//! A compilable example can be found
//! [here](https://github.com/Enselic/cargo-public-api/blob/main/rustdoc-json/examples/build-rustdoc-json.rs)

// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic, missing_docs)]

use std::path::PathBuf;

mod build;

/// Represents all errors that can occur when using [`crate::build()`].
#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    /// You tried to generate rustdoc JSON for a virtual manifest. That does not
    /// work. You need to point to the manifest of a real package.
    #[error("Manifest must be for an actual package. `{0:?}` is a virtual manifest")]
    VirtualManifest(PathBuf),

    /// A general error. Refer to the attached error message for more info.
    #[error("Failed to build rustdoc JSON. Stderr: {0}")]
    General(String),

    /// An error originating from `cargo_toml`.
    #[error(transparent)]
    CargoTomlError(#[from] cargo_toml::Error),

    /// An error originating from `cargo_metadata`.
    #[error(transparent)]
    CargoMetadataError(#[from] cargo_metadata::Error),

    /// Some kind of IO error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Contains all options for [`crate::build()`].
///
/// See [crate] for an example on how to use it.
#[derive(Debug)]
pub struct BuildOptions {
    toolchain: Option<String>,
    manifest_path: std::path::PathBuf,
    target: Option<String>,
    quiet: bool,
    no_default_features: bool,
    all_features: bool,
    features: Vec<String>,
    package: Option<String>,
}

/// Generate rustdoc JSON for a library crate. Returns the path to the freshly
/// built rustdoc JSON file.
///
/// See [crate] for an example on how to use it.
///
/// # Errors
///
/// E.g. if building the JSON fails or if the manifest path does not exist or is
/// invalid.
pub fn build(options: BuildOptions) -> Result<PathBuf, BuildError> {
    build::run_cargo_rustdoc(options)
}
