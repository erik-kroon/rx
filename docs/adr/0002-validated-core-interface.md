# ADR 0002: Validated Core Interface

Status: Accepted

## Context

`rx-core` owns the canonical pattern representation used by builders, parsers, emitters, lints, macros, CLI, editor support, and the web playground. As the product grows beyond literals into character sets, ranges, repeats, captures, dialects, and compatibility decisions, raw construction of core representation across crates would spread invariants into callers.

The project needs one place to enforce domain rules such as valid character ranges, explicit ASCII and Unicode semantics, capture name validity, repeat bounds, safe-core restrictions, normalization, and dialect support.

## Decision

Keep raw core representation mostly internal. Expose validated constructors, builders, and parser lowering paths as the primary way to create canonical patterns.

`rx-core` should own:

- Canonical pattern representation.
- Validation of safe-core invariants.
- Normalization needed for deterministic compact emission.
- Structured diagnostic construction for invalid core states.
- Dialect support checks for canonical patterns.

Other crates should not assemble raw core internals directly. They should call the validated core interface or lower syntax-specific structures into that interface.

## Consequences

- The public `rx` crate remains a user-facing builder facade over validated core construction.
- Legacy regex parsing, `.rx` parsing, and macro parsing may keep their own syntax structures with source spans, then lower into `rx-core`.
- Tests should target public behavior and validated core interfaces, not private representation details.
- Future issue work for character sets, sequences, repeats, captures, dialects, lints, CLI, and parsers should avoid introducing public raw AST constructors just for convenience.
- If tests need to build invalid states, they should use dedicated test helpers inside the owning module rather than widening production interfaces.
