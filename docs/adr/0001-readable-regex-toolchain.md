# ADR 0001: Readable Regex Toolchain

Status: Accepted

## Context

Regex syntax is compact but hard to review, lint, explain, and migrate safely. The `rx` project should make regular expressions readable without requiring users to abandon existing regex engines.

## Decision

Build `rx` as a Rust-native readable regex compiler and toolchain, not as a replacement matcher.

The core representation should be a canonical AST that can be validated, linted, normalized, explained, pretty-printed, and emitted as standard regex for target dialects.

Initial implementation should prioritize the safe regular-language core:

- Literals.
- Character classes and ranges.
- Sequences and alternatives.
- Groups and captures.
- Quantifiers.
- Anchors and boundaries.
- Flags.

Compatibility-only features such as backreferences, lookbehind, recursive patterns, conditionals, and engine-specific escapes may be represented later, but they must be explicit and dialect-gated.

## Consequences

- Public APIs should favor typed Rust composition and macro syntax over stringly-typed construction.
- ASCII and Unicode semantics must be explicit in names and diagnostics.
- Emitters must produce compact standard regex.
- Dialect failures should be structured and actionable.
- The CLI, LSP, and playground should reuse the same core instead of reimplementing parsing or explanation logic.
- Core construction should follow [ADR 0002](0002-validated-core-interface.md): validated constructors and lowering paths, with raw representation mostly internal.
