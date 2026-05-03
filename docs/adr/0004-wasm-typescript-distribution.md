# ADR 0004: WASM-First TypeScript Distribution

Status: Accepted

## Context

`rx` needs a future JavaScript/TypeScript distribution for the web playground,
docs examples, VS Code webviews, Node usage, and edge runtimes. The Rust core is
the source of truth for parsing, validation, linting, diagnostics, emission,
pretty-printing, explanation, and migration behavior. A TypeScript surface
should improve ergonomics without becoming a second implementation.

The main packaging options are:

- Raw generated WASM bindings.
- A TypeScript reimplementation.
- A TypeScript facade backed by a Rust WASM engine.
- A Node-native NAPI package.

## Decision

Build the JavaScript/TypeScript track as:

```text
Rust core
  -> WASM wrapper
  -> TypeScript facade
  -> npm package
  -> optional Node-native NAPI package later
```

Start with WASM, not NAPI. WASM gives one Rust-backed engine for browser,
playground, docs site, VS Code webviews, Node, and edge-runtime contexts. NAPI
may be added later for Node-heavy workloads after the npm API stabilizes.

Do not expose the whole Rust object model directly to TypeScript first. Expose
stable command-shaped functions across the WASM boundary, backed by
JSON-compatible request and response structs.

Initial command-shaped functions should include:

- `explainRegex(input, options)`
- `lintRegex(input, options)`
- `parseRegex(input, options)`
- `emitRx(input, options)`
- `formatRx(input, options)`

Result shapes should be TypeScript-native and diagnostic-first:

- `readable`
- `regex`
- `explanation`
- `diagnostics`
- spans as byte offsets matching core diagnostics

The TypeScript package may expose ergonomic builders such as `rx.set()`,
`rx.seq()`, `rx.either()`, `rx.literal()`, `rx.char()`, `rx.chars()`, and
`rx.range()`. Those builders should produce a JSON-compatible intermediate form
and call Rust/WASM for validation, normalization, linting, emission,
pretty-printing, and explanation.

## Consequences

- Rust remains the correctness boundary.
- TypeScript owns package ergonomics, types, async initialization, and examples.
- The npm package should not depend on a separate TypeScript regex semantics
  implementation.
- The WASM wrapper should stay thin and command-oriented.
- The TypeScript facade should hide raw generated `wasm-bindgen` bindings.
- Browser and Node examples can share one package before any NAPI package
  exists.
- Performance-sensitive Node workflows can add a separate native package later,
  such as `@rx-lang/rx-node`, without changing the universal API shape.

## Initial Package Shape

```text
crates/
  rx-core/
  rx-wasm/
packages/
  rx/
```

The `rx-wasm` crate should depend on `rx-core` and expose serialized commands.
The `packages/rx` package should provide the TypeScript facade, type
definitions, tests, examples, and npm packaging.

## Follow-Up Work

The npm/TypeScript track should be split into independently reviewable slices:

1. Add `rx-wasm` crate.
2. Expose explain/lint/emit/parse WASM functions.
3. Create `@rx-lang/rx` TypeScript package.
4. Add TypeScript types for diagnostics, options, and results.
5. Add TypeScript facade around generated WASM bindings.
6. Add TypeScript builder API that emits JSON-compatible input.
7. Add Vitest snapshot tests against Rust golden outputs.
8. Add browser bundler example.
9. Add Node example.
10. Publish `@rx-lang/rx` 0.1.0.
