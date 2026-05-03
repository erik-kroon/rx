# rx Roadmap

This roadmap records the current delivery posture after the initial architecture
deepening work. It is intentionally milestone-oriented: keep the kernel small,
make each surface reuse the same core contracts, and avoid widening into editor
or playground work before the language and diagnostics are stable.

## Current Baseline: 0.1.0 Kernel

Treat the following as the current implemented baseline for future issue work:

- Public Rust builder facade in `rx`.
- Canonical validated pattern representation in `rx-core`.
- Explicit ASCII class semantics and explicit Unicode rejection for the MVP.
- Compact regex emission with dialect-aware named-capture rejection.
- Alternation through `either` across core, public builders, readable syntax,
  macro expansion, legacy parsing, emission, explanation, and golden tests.
- Pretty-printing to readable rx and plain-English explanations.
- Readable rx parser, `.rx` pattern files, and validated readable artifacts.
- `pattern!` and `regex!` macro DSL paths backed by readable artifacts.
- CLI `explain`, `check`, and `emit` workflows.
- Legacy regex analysis facade for parsing, lint diagnostics, unsupported
  compatibility diagnostics, and migration replacement suggestions.
- Shared diagnostic categories, severities, source families, and render helpers.
- A lightweight performance smoke harness for parser and emission hot paths.

This baseline covers the original implementation slices #2 through #13. Those
issues should not be used as fresh starting points unless they are reopened with
a narrower follow-up scope.

## 0.1.0 Release Hardening

Before tagging a first release, focus on consistency and trust rather than new
surfaces:

- Make the documented supported language match the actual kernel. If
  boundaries or flags are not implemented, document them as post-0.1 work.
- Keep `either` / alternation in 0.1.0 and require future changes to preserve
  support across core, public builders, readable syntax, macro expansion,
  legacy parsing where supported, emission, explanation, and golden tests.
- Treat Rust regex as the only fully supported 0.1 target. Other dialect
  selectors are compatibility probes until they have dialect-specific positive
  and negative tests for every advertised feature.
- Route CLI and macro formatting through the shared diagnostic contract where
  practical. Surface-specific wording can stay in adapters, but categories and
  source locations should come from core diagnostics.
- Keep the performance harness lightweight, but run it before and after parser
  or emission changes that claim speed improvements.

## 0.2 Migration Depth

The next product milestone should deepen the legacy-to-readable workflow:

- Replace parallel legacy parse/lint scans with a recoverable shared fact model
  when diagnostics need to continue after unsupported constructs.
- Expand migration discovery beyond the first Rust `regex` constructors.
- Add CLI conversion output for supported legacy regex inputs.
- Add behavior-preservation helpers for sample input checks.
- Add fuzz/property coverage around legacy parsing, readable parsing, and
  emission safety.

## 0.3 Playground

Start the web playground only after the kernel contracts are stable enough to
reuse directly:

- Compile the shared core to a playground-friendly boundary.
- Follow [ADR 0004](adr/0004-wasm-typescript-distribution.md): add a thin
  `rx-wasm` command wrapper and a TypeScript-native npm facade instead of raw
  generated bindings or a TypeScript reimplementation.
- Show raw regex, readable rx, explanation, warnings, generated Rust, and
  generated regex from the same analysis path used by CLI/library code.
- Keep unsupported compatibility constructs explicit rather than approximated.

## 0.4 TypeScript/npm Package

After the playground-facing WASM boundary is stable, publish a reusable npm
package:

- Create `packages/rx` as the TypeScript facade over `rx-wasm`.
- Expose command-shaped APIs first: `explainRegex`, `lintRegex`, `parseRegex`,
  `emitRx`, and `formatRx`.
- Add TypeScript result and diagnostic types that mirror the shared core
  diagnostic contract.
- Add a TypeScript builder that produces JSON-compatible input and delegates
  validation, normalization, linting, emission, pretty-printing, and explanation
  to Rust/WASM.
- Add browser bundler and Node examples.
- Defer a NAPI package until Node-heavy usage justifies it and the universal API
  shape is stable.

## Later: Editor Integration

Editor work should follow the same contracts as CLI and playground:

- Hover explanations from core analysis.
- Diagnostics from the shared diagnostic taxonomy.
- Code actions backed by migration suggestions and readable artifacts.
- No editor-only parser or formatter behavior unless it is a thin adapter.
