# @rx-lang/rx

TypeScript facade for `rx`, backed by the Rust core compiled to WASM.

The package keeps the TypeScript surface ergonomic while Rust remains the source
of truth for validation, diagnostics, formatting, explanation, and regex
emission.

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

Node and Bun get a dedicated build automatically through package export
conditions. It loads WASM through the synchronous `wasm-pack` Node target
instead of experimental WASM module imports:

```ts
import { rx, toRegexSync } from "@rx-lang/rx/node";

const pathPiece = rx.oneOrMore(rx.oneOf(rx.alphaNumeric(), rx.char("/")));

console.log(toRegexSync(pathPiece)); // [A-Za-z0-9/]+
```

## Commands

```ts
import { emitRx, explainRegex, lintRegex, parseRegex } from "@rx-lang/rx";

await emitRx('one_or_more(set(ascii.alnum, chars("._/-")))');
await explainRegex("[A-Za-z0-9._/-]+");
await lintRegex("[\\w\\._/-]+");
await parseRegex("[A-Za-z0-9._/-]+");
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

## Build

This package expects `wasm-pack` and the Rust `wasm32-unknown-unknown` target:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
pnpm install
pnpm build
```
