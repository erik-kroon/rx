# rx Context

`rx` is a Rust-native readable regex system. It should let developers build, inspect, lint, convert, and compile regular expressions without writing dense regex strings by hand.

The product is a readable source language, compiler, linter, and explainer for regular expressions. It is not intended to be a replacement regex engine.

## Product Loop

The differentiating workflow is:

1. Read an existing regex.
2. Explain it.
3. Lint it.
4. Convert it to readable rx form.
5. Edit it readably.
6. Emit standard regex.
7. Generate Rust code.
8. Verify behavior.

## End-State Surfaces

- Rust builder API for day-to-day typed composition.
- Rust macro DSL for static readable patterns.
- `.rx` files for readable pattern definitions outside Rust source.
- CLI for explain, lint, emit, convert, test, and migrate workflows.
- TypeScript/npm facade backed by Rust WASM for JS/TS development.
- LSP/editor integration for hover explanations, diagnostics, and code actions.
- Web playground powered by the same core.

## Core Crate Boundaries

The PRD recommends this eventual layout:

- `rx`: public crate and prelude.
- `rx-core`: canonical AST, validation, normalization, linting, and emitters.
- `rx-macros`: `pattern!`, `regex!`, and related macros.
- `rx-parser`: legacy regex, `.rx`, and macro-style syntax parsing.
- `rx-cli`: command-line workflows.
- `rx-lsp`: editor integration.
- `rx-wasm`: WASM command wrapper for TypeScript and web surfaces.
- `rx-regex`: Rust regex engine integration.
- `packages/rx`: TypeScript/npm facade backed by `rx-wasm`.

## Domain Rules

- Generate standard regex strings that work with existing engines.
- Keep generated output compact, not mechanically expanded.
- Keep the default safe core regular: literals, character classes, ranges, sequences, alternatives, groups, captures, quantifiers, and anchors. Boundaries and flags are regular-language concepts, but they are post-0.1 work unless the roadmap says otherwise.
- Treat backreferences, lookbehind, recursive regex, conditionals, and engine-specific tricks as compatibility features, not safe-core features.
- Dialect-specific unsupported features should fail with structured, actionable errors.
- Builder overhead should happen before matching, not during matching.
- Keep raw core representation mostly internal. Callers should build patterns through validated constructors, builders, or parser lowering paths so invariants stay local to `rx-core`.

## Core Interface Direction

`rx-core` should expose a small validated interface around the canonical representation. It owns invariants such as valid ranges, capture names, repeat bounds, character-class normalization, safe-core restrictions, and dialect support checks.

Public and internal callers should prefer these construction paths:

- Public Rust builders in `rx`.
- Validated core constructors in `rx-core`.
- Parser lowering from legacy regex, `.rx` files, and macro syntax into the validated core.

Avoid spreading raw AST construction across crates. If a future parser needs source spans or syntax-specific recovery, keep that in parser-owned syntax structures and lower into the core only after validation.

## API Language

Prefer obvious, explicit API names:

- `one_or_more`
- `zero_or_more`
- `optional`
- `repeat`
- `repeat_between`
- `sequence`
- `either`
- `set`
- `not_set`
- `literal`
- `char`
- `chars`
- `range`
- `capture`
- `named_capture`
- `start_text`
- `end_text`

Avoid ambiguous public names that hide ASCII-vs-Unicode semantics. Prefer explicit namespaces and methods such as:

- `rx::ascii::word()`
- `rx::ascii::alnum()`
- `rx::unicode::word()`
- `rx::unicode::letter()`
- `.ascii_word()`
- `.ascii_alnum()`

## MVP Shape

The first useful product should include:

- Rust builder API.
- Canonical AST.
- Regex emission.
- ASCII classes.
- Sequence, either, set, repeat, captures, and anchors.
- Pretty-printer.
- Basic CLI explain and emit.
- Basic legacy regex parser.

## Current Baseline

The current `0.1.0` kernel baseline is tracked in
[docs/roadmap.md](docs/roadmap.md). Treat the original implementation slices
#2 through #13 as covered by that baseline when planning new work.

Remaining near-term work should harden the kernel before adding new product
surfaces:

- Boundaries and flags are post-0.1 unless they receive a dedicated implementation slice.
- Rust regex is the only fully supported `0.1.0` dialect. Other dialect selectors may exist only for explicit compatibility failures covered by tests.
- Tighten dialect support tests before documenting any additional fully supported target.
- Keep diagnostics centralized so CLI, macro, migration, playground, and editor
  surfaces render the same categories and source spans.
- Pull the TypeScript/npm package ahead of playground work when the goal is
  using `rx` in TypeScript development. The playground should consume that
  package boundary instead of growing its own WASM wrapper.
- For JavaScript/TypeScript distribution, follow
  [ADR 0004](docs/adr/0004-wasm-typescript-distribution.md): Rust core owns
  correctness, WASM exposes command-shaped functions, and TypeScript owns the
  ergonomic facade.
- The initial TypeScript facade may serialize builders through readable rx
  strings; move to JSON-compatible AST transport when TypeScript workflows need
  richer metadata or lower overhead across the WASM boundary.
- Treat Node and Bun as first-class TypeScript development targets. Keep a
  dedicated Node/Bun WASM build path available instead of forcing those runtimes
  through browser/bundler-only WASM loading.
