## Tips to work on this tool

This project has kinship with [`public_items`](https://github.com/Enselic/public_items). Here follows some tips on how to make it easier to work with both projects. This guides assumes you have cloned `public_items` to `~/src/public_items` and [`cargo-public-api`](https://github.com/Enselic/cargo-public-api) to `~/src/cargo-public-api`.

### Make `cargo public-api` use local changes of `public_items`

Uncomment
```toml
# path = "/Users/martin/src/public_items"
```
in `~/src/cargo-public-api/Cargo.toml` and update the path so it fits your system.

### Run local copy of `cargo-public-api` on an arbitrary crate

There are two ways. You can either do:
```
% cd ~/src/arbitrary-crate
% cargo run --manifest-path ~/src/cargo-public-api/Cargo.toml
```
or you can do
```
% cd ~/src/cargo-public-api
% cargo run -- --manifest-path ~/src/arbitrary-crate/Cargo.toml
```
In the first case `--manifest-path` is interpreted by `cargo` itself, and in the second case `--manifest-path` is interpreted by `cargo-public-api`.

NOTE: The second way does not work with `--diff-git-checkouts` yet.

You can also combine both ways:
```
% cd /does/not/matter
% cargo run --manifest-path ~/src/cargo-public-api/Cargo.toml -- --manifest-path ~/src/arbitrary-crate/Cargo.toml
```
