# @rx-lang/rx

Readable regex for TypeScript, backed by Rust internals compiled to WASM.

Use `rx` to build, inspect, format, lint, and emit regular expressions without
hand-writing dense regex strings. The TypeScript API is ergonomic, while the
Rust core remains the source of truth for validation, diagnostics, formatting,
explanation, and regex emission.

## Install

```sh
pnpm add @rx-lang/rx
```

```sh
npm install @rx-lang/rx
```

```sh
bun add @rx-lang/rx
```

## Quick Start

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

console.log(pathPiece.toRx());
console.log(await toRegex(pathPiece)); // [A-Za-z0-9/._-]+
```

## Node and Bun

Node and Bun get a dedicated build automatically through package export
conditions. They can also import `@rx-lang/rx/node` directly for sync command
APIs. This path loads WASM through the synchronous `wasm-pack` Node target
instead of experimental WASM module imports.

```ts
import { rx, toRegexSync } from "@rx-lang/rx/node";

const pathPiece = rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")));

console.log(toRegexSync(pathPiece)); // [A-Za-z0-9/]+
```

## Commands

```ts
import { emitRx, explainRegex, formatRx, lintRegex, parseRegex, toRegex } from "@rx-lang/rx";

await emitRx('one_or_more(set(ascii.alnum, chars("._/-")))');
await explainRegex("[A-Za-z0-9._/-]+");
await formatRx('one_or_more(set(ascii.alnum,chars("._/-")))');
await lintRegex("[\\w\\._/-]+");
await parseRegex("[A-Za-z0-9._/-]+");
await toRegex(pathPiece);
```

All command results are diagnostic-first:

```ts
type CommandResult = {
  readable?: string;
  regex?: string;
  explanation?: string;
  diagnostics: RxDiagnostic[];
};
```

Node and Bun also expose sync command variants from `@rx-lang/rx/node`:

```ts
import {
  emitRxSync,
  explainRegexSync,
  formatRxSync,
  lintRegexSync,
  parseRegexSync,
  toRegexSync,
} from "@rx-lang/rx/node";
```

## Stable API

The `0.1.x` TypeScript API is:

- Builders: `rx.literal`, `rx.char`, `rx.chars`, `rx.range`, `rx.set`,
  `rx.oneOf`, `rx.sequence`, `rx.either`, `rx.zeroOrMore`, `rx.oneOrMore`,
  `rx.optional`, `rx.repeat`, `rx.repeatBetween`, `rx.startText`, `rx.endText`,
  `rx.capture`, `rx.namedCapture`.
- ASCII set helpers: `rx.asciiWord`, `rx.alphaNumeric`, `rx.asciiAlpha`,
  `rx.digit`, `rx.whitespace`.
- Async commands: `emitRx`, `explainRegex`, `formatRx`, `lintRegex`,
  `parseRegex`, `toRegex`.
- Node/Bun sync commands: `emitRxSync`, `explainRegexSync`, `formatRxSync`,
  `lintRegexSync`, `parseRegexSync`, `toRegexSync`.
- Types: `CommandResult`, `RxDiagnostic`, `EmitOptions`, `Dialect`,
  `RxPattern`, `RxError`, `SetItem`, `Span`.

## Build

Consumers do not need Rust or `wasm-pack`; the published package includes built
WASM for browser/bundler and Node/Bun targets.

Repository development expects package dependencies, `wasm-pack`, and the Rust
`wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
pnpm install --frozen-lockfile
pnpm build
```
