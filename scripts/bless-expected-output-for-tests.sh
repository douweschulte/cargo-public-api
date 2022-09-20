#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# `:-+nightly` means "if unset, use +nightly"
toolchain=${RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK:-+nightly}

test_git_dir="/tmp/cargo-public-api-test-repo"
rm -rf "${test_git_dir}"
cargo run -p test-utils -- "${test_git_dir}" ./test-apis

build_for="
    comprehensive_api
    comprehensive_api_proc_macro
    example_api-v0.2.0
"

output_for="
    comprehensive_api
    comprehensive_api_proc_macro
"

for crate in $build_for; do
    cargo ${toolchain} rustdoc --lib --manifest-path "./test-apis/${crate}/Cargo.toml" -- -Z unstable-options --output-format json
done

for crate in $output_for; do
    cargo run -p public-api -- "./test-apis/${crate}/target/doc/${crate}.json" > "public-api/tests/expected-output/${crate}.txt"
done

BLESS=1 RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo test -- cargo_public_api_with_features

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p public-api -- \
      --with-blanket-implementations "./test-apis/example_api-v0.2.0/target/doc/example_api.json" > \
      "public-api/tests/expected-output/example_api-v0.2.0-with-blanket-implementations.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" > \
      "public-api/tests/expected-output/example_api-v0.3.0.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=never --diff-git-checkouts "v0.2.0" "v0.3.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "public-api/Cargo.toml" \
      --color=never > \
      "cargo-public-api/tests/expected-output/public_api_list.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --package rustdoc-json \
      --color=never > \
      "cargo-public-api/tests/expected-output/rustdoc_json_list.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=never > \
      "cargo-public-api/tests/expected-output/test_repo_api_latest.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=always --diff-git-checkouts "v0.1.0" "v0.2.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0_colored.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=always > \
      "cargo-public-api/tests/expected-output/example_api_v0.3.0_colored.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --diff-git-checkouts "v0.1.0" "v0.2.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0.txt"

echo 'FIXME: Do not hardcode to `--cap-lints warn` in `rustdoc-json` crate'
RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "./test-apis/lint_error/Cargo.toml" > \
      "cargo-public-api/tests/expected-output/lint_error_list.txt"
