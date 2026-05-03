# rx

Readable regex for serious software.

`rx` is a Rust-native readable regex compiler and toolchain. It helps developers build, inspect, lint, convert, and emit regular expressions without hand-writing dense regex strings.

## Context

- [CONTEXT.md](CONTEXT.md) summarizes product intent, domain rules, and MVP boundaries.
- [AGENTS.md](AGENTS.md) contains repo-local guidance for coding agents.
- [docs/prd/end-state-readable-regex-toolchain.md](docs/prd/end-state-readable-regex-toolchain.md) is the current end-state PRD.
- [docs/adr](docs/adr) stores durable architecture decisions.
- [docs/agents/workflow.md](docs/agents/workflow.md) records agent workflow and work tracking conventions.

## Development

Run the standard Rust checks from the repository root:

```sh
cargo fmt --all --check
cargo test --workspace
```
