#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

for crate in public-api rustdoc-json; do
    diff -u ${crate}/public-api.txt <(cargo run -p cargo-public-api -- --manifest-path ${crate}/Cargo.toml) ||
            (echo -e '\nFAIL: Public API changed! To bless, `git commit` the result of `./scripts/bless-public-apis.sh' && exit 1)
done
