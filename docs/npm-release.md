# npm Release Checklist

This checklist covers publishing `@rx-lang/rx`.

## Package

- npm package: `@rx-lang/rx`
- current stable version: `0.1.0`
- Rust-backed package paths:
  - `@rx-lang/rx` for conditional browser, Node, and Bun resolution
  - `@rx-lang/rx/node` for explicit Node/Bun sync APIs
  - `@rx-lang/rx/builders` for builder-only imports

## Preflight

Run from `packages/rx`:

```sh
pnpm install --frozen-lockfile
pnpm exec tsc --noEmit
pnpm test
pnpm smoke:node
pnpm smoke:bun
pnpm pack --dry-run
pnpm publish --dry-run --access public --no-git-checks
```

Run `pnpm install --frozen-lockfile` before `pnpm build`, `pnpm pack`, or
`pnpm publish`; package lifecycle scripts do not install dev dependencies for
you. `pnpm test` builds both WASM targets before running Vitest. The package
requires `wasm-pack` and the Rust `wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Tarball Consumer Smoke

Before publishing, test the exact tarball in a clean project:

```sh
cd packages/rx
pnpm pack

tmpdir="$(mktemp -d)"
cd "$tmpdir"
pnpm init
pnpm add /path/to/rx/packages/rx/rx-lang-rx-0.1.0.tgz
node --input-type=module --eval 'import { rx, toRegexSync } from "@rx-lang/rx"; console.log(toRegexSync(rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")))));'
bun --eval 'import { rx, toRegexSync } from "@rx-lang/rx"; console.log(toRegexSync(rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")))));'
```

Both smoke commands should print:

```text
[A-Za-z0-9/]+
```

## Publish

Confirm npm authentication and package access:

```sh
npm whoami
npm access ls-packages
```

Publish the scoped public package:

```sh
cd packages/rx
pnpm publish --access public
```

## Post-Publish Smoke

After npm indexing completes:

```sh
npm view @rx-lang/rx version dist-tags --json

tmpdir="$(mktemp -d)"
cd "$tmpdir"
pnpm init
pnpm add @rx-lang/rx@0.1.0
node --input-type=module --eval 'import { rx, toRegexSync } from "@rx-lang/rx"; console.log(toRegexSync(rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")))));'
bun --eval 'import { rx, toRegexSync } from "@rx-lang/rx"; console.log(toRegexSync(rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")))));'
```
