# PRD: End-State Readable Regex Toolchain

## Problem Statement

Developers rely on regular expressions because regex engines are powerful, portable, and widely available, but regex syntax itself is difficult to read, review, explain, lint, migrate, and maintain. This is especially painful in systems and backend codebases where validation and parsing logic needs to be reviewed carefully.

The user wants `rx` to solve the syntax and tooling problem without forcing teams to abandon existing regex engines. `rx` should become the layer developers use before emitting or compiling a regular expression: a readable, typed, lintable, portable representation that can still produce standard compact regex strings.

## Solution

Build `rx` as a Rust-native readable regex compiler and toolchain.

The product should provide a canonical readable regex representation that can be authored through a Rust builder API, a macro DSL, and readable pattern files. That representation should support validation, normalization, linting, explanation, pretty-printing, dialect-aware regex emission, CLI workflows, editor integration, and a web playground.

The default product should focus on regular-language features. Non-regular or engine-specific constructs may be represented later for migration and compatibility, but they must be explicit, clearly marked, and rejected by dialects that cannot support them.

## User Stories

1. As a Rust developer, I want to build regex patterns with readable typed calls, so that I do not need to hand-write dense regex strings.
2. As a Rust developer, I want generated output to be standard regex, so that I can keep using existing regex engines.
3. As a Rust developer, I want generated regex to be compact, so that using `rx` does not create slower or uglier patterns than I would write by hand.
4. As a Rust developer, I want ASCII and Unicode character classes to be explicit, so that code review does not depend on hidden engine semantics.
5. As a Rust developer, I want a first-class builder API, so that readable regex construction feels native in normal Rust code.
6. As a Rust developer, I want a macro DSL for static patterns, so that compile-time validation can catch obvious mistakes early.
7. As a Rust developer, I want macro errors to explain invalid ranges and invalid capture names clearly, so that failures are easy to fix.
8. As a Rust developer, I want runtime validation errors to be structured, so that applications and tools can report useful diagnostics.
9. As a Rust developer, I want named captures to emit correctly for supported dialects, so that readable patterns preserve useful match metadata.
10. As a Rust developer, I want unsupported named captures to fail clearly in incompatible dialects, so that portability problems do not slip through silently.
11. As a Rust developer, I want anchors, boundaries, flags, captures, groups, alternatives, sequences, character sets, ranges, and repeats in the safe core, so that common validation and parsing patterns are covered.
12. As a Rust developer, I want non-regular constructs to be outside the safe default, so that the core model remains portable and predictable.
13. As a library author, I want a canonical AST, so that validation, linting, explanation, and emission all share the same source of truth.
14. As a library author, I want normalization to happen before emission, so that generated regex output is minimal and deterministic.
15. As a library author, I want clear module boundaries between core representation, parsing, macros, CLI, editor support, web support, and engine integration, so that each area can evolve independently.
16. As a CLI user, I want to paste an unreadable regex and receive a readable explanation, so that I can understand inherited patterns quickly.
17. As a CLI user, I want to lint legacy regex strings, so that unnecessary escapes and ambiguous character classes are surfaced.
18. As a CLI user, I want to emit standard regex from readable pattern input, so that readable files can be used in existing systems.
19. As a CLI user, I want to convert regex into readable rx form, so that legacy patterns can be migrated incrementally.
20. As a CLI user, I want to convert readable rx form back into regex, so that `rx` can fit into existing build and release workflows.
21. As a CLI user, I want to test readable patterns against sample data, so that migration preserves behavior.
22. As a maintainer of a Rust codebase, I want tooling to find existing regex literals, so that migration work can be scoped.
23. As a maintainer of a Rust codebase, I want migration suggestions for common regex initialization patterns, so that replacements can be applied safely.
24. As a code reviewer, I want readable rx output next to generated regex, so that I can review validation logic without decoding dense syntax.
25. As a security-conscious reviewer, I want ambiguous escape and character class diagnostics, so that correctness-sensitive patterns are easier to audit.
26. As an editor user, I want hover explanations for regex literals, so that I can understand patterns without leaving my editor.
27. As an editor user, I want diagnostics for unnecessary escapes and target-dependent semantics, so that issues are visible while editing.
28. As an editor user, I want code actions to convert regex to builder form or macro form, so that migration is ergonomic.
29. As an editor user, I want generated tests or sample cases suggested from a pattern, so that regex behavior can be reviewed with examples.
30. As a developer using readable pattern files, I want named pattern definitions outside Rust source, so that complex patterns are maintainable as standalone assets.
31. As a developer using readable pattern files, I want checking, explaining, and emitting to work on those files, so that they are not second-class inputs.
32. As a developer targeting multiple languages, I want dialect-aware emission for Rust, JavaScript, Python, Go, PCRE2, and POSIX-style targets, so that one readable source can serve multiple environments where possible.
33. As a developer targeting multiple languages, I want unsupported features to produce actionable errors, so that I can simplify a pattern or choose another dialect.
34. As an educator, I want plain-English explanations of patterns, so that regex concepts can be taught through readable examples.
35. As a team lead, I want documentation organized by user goals, so that adoption does not require reading internal architecture notes.
36. As a new user, I want docs to repeatedly show unreadable regex, readable rx, generated regex, and plain-English explanation, so that the value of the tool is obvious.
37. As a web playground user, I want raw regex input, readable form, explanation, warnings, generated code, and generated regex visible together, so that I can explore conversions interactively.
38. As a web playground user, I want the playground to share the same core logic as the CLI and library, so that results are consistent across surfaces.
39. As a maintainer, I want golden tests for emission, so that output changes are intentional and reviewable.
40. As a maintainer, I want parser round-trip tests, so that readable syntax and legacy regex conversion remain stable.
41. As a maintainer, I want lint tests, so that diagnostics remain precise as the rule set grows.
42. As a maintainer, I want dialect compatibility tests, so that unsupported feature failures remain correct.
43. As a maintainer, I want compile-fail macro tests, so that macro diagnostics remain useful.
44. As a maintainer, I want property-based and fuzz tests around parsing and normalization, so that malformed or unusual inputs do not break the toolchain.
45. As a maintainer, I want cross-engine behavior tests where feasible, so that emitted regex behaves consistently with expectations.
46. As a performance-sensitive user, I want builder overhead to happen before matching, so that runtime matching is not meaningfully slower than hand-written regex.
47. As a performance-sensitive user, I want macro validation to avoid noticeable build-time penalties, so that static readable patterns remain practical.
48. As a performance-sensitive CLI user, I want large codebases handled incrementally, so that migration tooling scales beyond toy examples.
49. As an adopter, I want an MVP that proves the builder API, core AST, emission, ASCII classes, basic parser, pretty-printer, and CLI explain/emit workflows, so that the project delivers value before all end-state surfaces exist.
50. As an adopter, I want a strong v1 to add macro syntax, diagnostics, linting, migration suggestions, dialect support, and the playground, so that the product becomes useful for serious migration work.
51. As an adopter, I want the end state to include LSP support, full conversion, code generators, compatibility representation, WASM support, and corpus testing, so that `rx` can become a complete regex readability toolchain.

## Implementation Decisions

- Build around a canonical core representation that owns expression structure, character sets, repeats, groups, anchors, boundaries, flags, validation, normalization, linting, explanation, pretty-printing, and emission.
- Keep the default representation regular-language-first. Compatibility-only constructs are allowed only through an explicit compatibility layer.
- Use explicit ASCII and Unicode character class concepts in public APIs and diagnostics.
- Provide a fluent Rust builder API as the primary authoring interface.
- Provide a nested function-style API for users who prefer explicit composition.
- Provide macro-based authoring for static patterns with validation and useful compile-time errors.
- Provide readable pattern files for complex named patterns outside Rust source.
- Provide a dialect abstraction for regex emission, starting with Rust regex, ECMAScript, Python, Go, PCRE2, and POSIX ERE-style targets.
- Keep dialect unsupported-feature failures structured and actionable.
- Provide CLI workflows for explain, pretty-print, lint, emit, test, convert, and Rust migration discovery.
- Provide editor workflows for hover explanations, diagnostics, conversions, linting, and test generation.
- Provide a web playground that exposes raw regex input, readable form, explanation, warnings, generated Rust, and generated regex.
- Keep generated regex output standard and compact rather than mechanically expanded.
- Keep matching delegated to existing engines unless an optional matcher backend is explicitly added later.
- Organize documentation by user goals rather than internal architecture.
- Sequence delivery through MVP, strong v1, and end-state milestones rather than trying to ship every surface at once.

## Testing Decisions

- Tests should assert external behavior: emitted regex strings, diagnostics, parse results, dialect errors, explanations, and migration suggestions.
- Golden emission tests should cover canonical builder patterns, character sets, repeats, captures, anchors, and compact output expectations.
- Normalization tests should prove equivalent readable inputs produce deterministic canonical output.
- Parser round-trip tests should cover readable pattern syntax, legacy regex input, and macro-style syntax.
- Dialect compatibility tests should cover named captures, unsupported compatibility constructs, and dialect-specific escape behavior.
- Lint tests should cover unnecessary escapes, ambiguous ASCII-vs-Unicode classes, redundant set entries, invalid ranges, and invalid capture names.
- Macro compile-fail tests should cover invalid ranges, invalid capture names, unsupported constructs, and malformed syntax.
- Property-based tests should focus on parser stability, normalization invariants, and emission safety.
- Fuzz tests should target legacy regex parsing and readable pattern parsing.
- Cross-engine behavior tests should be added where practical for emitted regex in supported target engines.
- CLI tests should exercise explain, lint, emit, convert, and migration workflows through observable command output.
- Editor and playground tests should focus on shared core behavior first, then surface-specific integration.

## Out of Scope

- Replacing existing regex engines as the default execution backend.
- Making non-regular constructs part of the safe default language.
- Treating short aliases as the main documented API.
- Shipping every end-state surface in the MVP.
- Guaranteeing every readable pattern can emit to every dialect.
- Hiding ASCII-vs-Unicode semantics behind ambiguous names.
- Optimizing runtime matching beyond producing compact regex for existing engines.

## Further Notes

The central product promise is the full loop: read old regex, explain it, lint it, convert it, edit it readably, emit standard regex, generate Rust code, and verify behavior.

The adoption path should start with the smallest useful Rust-native core, then grow into CLI, macro, dialect, migration, editor, and playground surfaces without duplicating core logic.

