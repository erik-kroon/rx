import { describe, expect, it } from "vitest";

import { rx } from "../src/builders";

describe("TypeScript builders", () => {
  it("serializes the path segment example into readable rx", () => {
    const pathPiece = rx.oneOrMore(
      rx.oneOf(
        rx.alphaNumeric(),
        rx.char("/"),
        rx.char("."),
        rx.char("-"),
        rx.char("_"),
      ),
    );

    expect(pathPiece.toRx()).toMatchInlineSnapshot(`
      "one_or_more(
          set(
              ascii.alnum,
              char("/"),
              char("."),
              char("-"),
              char("_")
          )
      )"
    `);
  });

  it("throws before invalid single-character set items reach Rust", () => {
    expect(() => rx.char("ab")).toThrow("char must contain exactly one character");
  });
});
