# Release Checklist

This checklist covers the Rust crates.io release flow for `0.1.0`.

## Package Names

The public Rust import names stay short:

- `rx`
- `rx_core`
- `rx_macros`

The crates.io package names are distinct because `rx` and `rx_core` are already
taken on crates.io:

- `rx-lang`
- `rx-lang-core`
- `rx-lang-macros`
- `rx-cli`

Users depend on the public library as:

```toml
[dependencies]
rx = { package = "rx-lang", version = "0.1.0" }
```

## Preflight

Run from the repository root:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo publish -p rx-lang-core --dry-run
```

`cargo package --workspace` cannot fully verify unpublished dependent crates
before `rx-lang-core` exists on crates.io because Cargo removes local `path`
dependencies during packaging. Verify and publish crates in dependency order.

## Publish Order

1. `cargo publish -p rx-lang-core`
2. `cargo publish -p rx-lang-macros`
3. `cargo publish -p rx-lang`
4. `cargo publish -p rx-cli`

After each crate is published and indexed by crates.io, run the next crate's
dry-run if desired:

```sh
cargo publish -p rx-lang-macros --dry-run
cargo publish -p rx-lang --dry-run
cargo publish -p rx-cli --dry-run
```

## Tag

After all crates are published:

```sh
git tag v0.1.0
git push origin v0.1.0
```
