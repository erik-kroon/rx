# rx

Readable regex for Rust.

`rx` is a Rust-native readable regex compiler and toolchain. It lets you build,
inspect, lint, convert, and emit regular expressions without hand-writing dense
regex strings.

`rx` does not replace regex engines. It emits compact standard regex strings for
existing engines, with Rust's `regex` syntax as the fully supported `0.1.0`
target.

## Install

Use the library crate for builders, macros, parsing, diagnostics, and emission:

```toml
[dependencies]
rx = { package = "rx-lang", version = "0.1.0" }
```

Install the CLI when you want command-line explain, check, convert, and emit
workflows:

```sh
cargo install rx-cli
```

The CLI binary is named `rx`.

## Rust Builder

```rust
let pattern = rx::sequence([
    rx::start_text(),
    rx::set([rx::ascii::alpha(), rx::char('_')]),
    rx::set([rx::ascii::alnum(), rx::char('_')]).zero_or_more(),
    rx::end_text(),
]);

assert_eq!(pattern.to_regex(), "^[A-Za-z_][A-Za-z0-9_]*$");
```

Character class semantics are explicit:

```rust
let path_piece = rx::Set::new()
    .ascii_word()
    .chars("./-")
    .one_or_more();

assert_eq!(path_piece.to_regex(), r#"[A-Za-z0-9_./-]+"#);
```

Alternation is available through `either`:

```rust
let method = rx::either([rx::literal("GET"), rx::literal("POST")]);
let pattern = rx::sequence([rx::start_text(), method, rx::literal(" /"), rx::end_text()]);

assert_eq!(pattern.to_regex(), r#"^(?:GET|POST) /$"#);
```

## Macros

Use `pattern!` for a validated static pattern and `regex!` for a generated regex
string literal:

```rust
let pattern = rx::pattern! {
    one_or_more(
        set(
            ascii::alnum,
            chars("._/-")
        )
    )
};

assert_eq!(pattern.to_regex(), "[A-Za-z0-9._/-]+");

const IDENTIFIER: &str = rx::regex! {
    sequence(
        start_text,
        set(ascii::alpha, char("_")),
        zero_or_more(set(ascii.alnum, char("_"))),
        end_text
    )
};
```

## Readable Pattern Files

Readable rx files define named patterns:

```rx
pattern path_piece = one_or_more(set(ascii.alnum, chars("._/-")))
pattern identifier = sequence(
    start_text,
    set(ascii.alpha, char("_")),
    zero_or_more(set(ascii.alnum, char("_"))),
    end_text
)
```

They can be checked or emitted through the CLI:

```sh
rx check patterns.rx
rx emit patterns.rx
```

## CLI

Explain a legacy regex:

```sh
rx explain '[\w\._/-]+'
```

Convert a supported legacy regex into readable rx:

```sh
rx convert '[\w\._/-]+'
```

Emit regex from readable rx:

```sh
rx emit 'one_or_more(set(ascii.alnum, chars("._/-")))'
```

Select a dialect when needed:

```sh
rx emit 'named_capture("id", one_or_more(set(ascii.digit)))' --dialect rust-regex
```

`rust-regex` is the fully supported `0.1.0` dialect. `pcre2` and `posix-ere`
exist as limited compatibility targets, primarily to exercise explicit
unsupported-feature diagnostics.

## TypeScript

The `packages/rx` workspace package provides the TypeScript facade backed
by the Rust core compiled to WASM:

```ts
import { rx, toRegex } from "@rx-lang/rx";

const pathPiece = rx.oneOrMore(
  rx.oneOf(
    rx.alphaNumeric(),
    rx.char("/"),
    rx.char("."),
    rx.char("-"),
    rx.char("_"),
  ),
);

console.log(await toRegex(pathPiece)); // [A-Za-z0-9/._-]+
```

Node and Bun use a dedicated package export that loads the Rust WASM through the
non-experimental `wasm-pack` Node target:

```ts
import { rx, toRegexSync } from "@rx-lang/rx/node";

const pathPiece = rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")));

console.log(toRegexSync(pathPiece)); // [A-Za-z0-9/]+
```

Build it locally with:

```sh
cd packages/rx
pnpm install
pnpm build
```

`pnpm build` generates both browser/bundler and Node/Bun WASM bindings. It
requires `wasm-pack` and a Rust toolchain with `wasm32-unknown-unknown`
installed.

## Diagnostics

Runtime parsing and emission errors are structured. Diagnostics include source
spans, categories, severities, messages, suggestions, and source families where
available.

```rust
let error = rx::parse_legacy_regex(r#"(\w+)\1"#).unwrap_err();

assert_eq!(
    error.kind,
    rx::ParseErrorKind::UnsupportedFeature(rx::UnsupportedFeature::Backreference)
);
```

## `0.1.0` Scope

Included in the `0.1.0` kernel:

- Rust builder facade.
- `pattern!` and `regex!` macros.
- Readable rx parser and `.rx` files.
- Legacy regex analysis for a focused safe subset.
- Lint diagnostics for common legacy regex issues.
- Conversion suggestions for supported legacy forms.
- Compact Rust-regex emission.
- Explicit ASCII class semantics and explicit Unicode rejection.
- Literals, sets, ranges, sequences, alternation, repeats, captures, named
  captures, and start/end anchors.
- CLI `explain`, `convert`, `check`, and `emit`.

Deferred beyond `0.1.0`:

- Boundaries and flags.
- Full dialect support beyond Rust regex.
- Full legacy regex compatibility representation.
- LSP/editor integration.
- Stable npm publishing, Node-native acceleration, and playground surfaces.

## Crates

- `rx`: public Rust API, builders, macros, parsing helpers, and diagnostics.
- `rx-core`: canonical representation, parsing, diagnostics, emission, linting,
  explanation, behavior checks, and migration suggestions.
- `rx-macros`: compile-time readable regex macros.
- `rx-cli`: command-line workflows.
- `rx-wasm`: WASM command wrapper over `rx-core` for TypeScript/web surfaces.
- `packages/rx`: TypeScript facade and npm package scaffold backed by
  `rx-wasm`.

The crates.io package names for the library crates are `rx-lang`,
`rx-lang-core`, and `rx-lang-macros`; the Rust crate names remain `rx`,
`rx_core`, and `rx_macros`.

## Project Docs

- [CONTEXT.md](CONTEXT.md) summarizes product intent, domain rules, and MVP boundaries.
- [docs/roadmap.md](docs/roadmap.md) records the milestone baseline and next delivery sequence.
- [docs/adr](docs/adr) stores durable architecture decisions.
- [docs/adr/0004-wasm-typescript-distribution.md](docs/adr/0004-wasm-typescript-distribution.md) records the WASM-first TypeScript/npm distribution direction.
- [docs/agents/workflow.md](docs/agents/workflow.md) records agent workflow and work tracking conventions.
- [docs/release.md](docs/release.md) records the Rust crates.io release checklist.
- [docs/npm-release.md](docs/npm-release.md) records the `@rx-lang/rx` npm release checklist.

## Development

Run the standard checks from the repository root:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Packaging checks:

```sh
cargo package --workspace --allow-dirty
```

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
