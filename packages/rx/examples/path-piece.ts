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
console.log(await toRegex(pathPiece));
