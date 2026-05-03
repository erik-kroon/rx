# Agent Guidance

This repo is the `rx` project: a Rust-native readable regex compiler and toolchain.

## Source Of Truth

- Product intent starts in [CONTEXT.md](CONTEXT.md).
- The full current PRD is in `.context/attachments/pasted_text_2026-05-03_11-28-52.txt`.
- Durable architecture decisions live in [docs/adr](docs/adr).
- Work and skill conventions live in [docs/agents](docs/agents).

## Working Rules

- Keep the safe core regular-language-first. Compatibility-only regex features must be explicit and dialect-gated.
- Prefer Rust-native typed APIs over stringly-typed pattern construction.
- Use explicit ASCII and Unicode namespaces in public API names.
- Keep generated regex output standard, compact, and usable by existing engines.
- Organize docs by user goals: unreadable regex, readable rx, generated regex, and plain-English explanation.
- For JavaScript/TypeScript distribution, keep Rust as the correctness boundary: use WASM-first command APIs plus a TypeScript-native facade, not a TS reimplementation or raw generated bindings as the public surface.

## Before Implementing

1. Check [CONTEXT.md](CONTEXT.md) for domain terms and product boundaries.
2. Check [docs/adr](docs/adr) for accepted decisions before changing architecture.
3. If adding a new major product surface, add or update an ADR.
4. If adding tracker or triage conventions, record them in [docs/agents/workflow.md](docs/agents/workflow.md).
