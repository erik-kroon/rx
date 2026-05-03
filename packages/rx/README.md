# @rx-lang/rx

Readable regex for TypeScript, backed by Rust internals compiled to WASM.

Use `rx` to build, inspect, format, lint, and emit regular expressions without
hand-writing dense regex strings. The builder API is synchronous TypeScript, so
common authoring does not need WASM initialization. The Rust core powers raw
regex parsing, linting, formatting, explanation, migration, and validated
emission through WASM commands.

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
import { rx } from "@rx-lang/rx";

const pathPiece = rx.set(rx.ascii.alnum, "._/-").oneOrMore();

console.log(pathPiece.toReadable());
console.log(pathPiece.toRegex()); // [A-Za-z0-9._/-]+
```

The older functional builder style stays supported:

```ts
const pathPiece = rx.oneOrMore(
  rx.oneOf(rx.alphaNumeric(), rx.char("/"), rx.char("."), rx.char("-"), rx.char("_")),
);
```

## Node and Bun

Node and Bun get a dedicated build automatically through package export
conditions. They can also import `@rx-lang/rx/node` directly for sync command
APIs. This path loads WASM through the synchronous `wasm-pack` Node target
instead of experimental WASM module imports.

```ts
import { rx } from "@rx-lang/rx/node";

const pathPiece = rx.set(rx.ascii.alnum, "/").oneOrMore();

console.log(pathPiece.toRegex()); // [A-Za-z0-9/]+
console.log(rx.toRegexSync(pathPiece)); // [A-Za-z0-9/]+
```

## Commands

```ts
import { rx } from "@rx-lang/rx";

await rx.emitRx('one_or_more(set(ascii.alnum, chars("._/-")))');
await rx.explainRegex("[A-Za-z0-9._/-]+");
await rx.formatRx('one_or_more(set(ascii.alnum,chars("._/-")))');
await rx.lintRegex("[\\w\\._/-]+");
await rx.parseRegex("[A-Za-z0-9._/-]+");
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
  `rx.oneOf`, `rx.seq`, `rx.sequence`, `rx.either`, `rx.zeroOrMore`, `rx.oneOrMore`,
  `rx.optional`, `rx.repeat`, `rx.repeatBetween`, `rx.startText`, `rx.endText`,
  `rx.capture`, `rx.namedCapture`.
- Fluent pattern methods: `pattern.zeroOrMore`, `pattern.oneOrMore`,
  `pattern.optional`, `pattern.repeat`, `pattern.repeatBetween`,
  `pattern.toReadable`, `pattern.toRegex`, `pattern.toJson`,
  `pattern.toRegExp`.
- ASCII set helpers: `rx.asciiWord`, `rx.alphaNumeric`, `rx.asciiAlpha`,
  `rx.digit`, `rx.whitespace`, plus `rx.ascii.word`, `rx.ascii.alnum`,
  `rx.ascii.alpha`, `rx.ascii.digit`, and `rx.ascii.whitespace`.
- Async commands: `emitRx`, `explainRegex`, `formatRx`, `lintRegex`,
  `parseRegex`, `toRegex`, plus the same command methods on `rx`.
- Node/Bun sync commands: `emitRxSync`, `explainRegexSync`, `formatRxSync`,
  `lintRegexSync`, `parseRegexSync`, `toRegexSync`, plus the same sync command
  methods on `rx` from `@rx-lang/rx/node`.
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
