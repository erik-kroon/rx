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

  it("supports fluent set and quantifier authoring", () => {
    const pathPiece = rx.set(rx.ascii.alnum, "._/-").oneOrMore();

    expect(pathPiece.toReadable()).toMatchInlineSnapshot(`
      "one_or_more(
          set(
              ascii.alnum,
              chars("._/-")
          )
      )"
    `);
  });

  it("emits common builder patterns synchronously without WASM", () => {
    expect(rx.set(rx.ascii.alnum, "._/-").oneOrMore().toRegex()).toBe("[A-Za-z0-9._/-]+");
    expect(rx.seq(rx.literal("GET"), rx.literal(" /")).toRegex()).toBe("GET /");
    expect(rx.either(rx.literal("cat"), rx.literal("dog")).toRegex()).toBe("cat|dog");
  });

  it("serializes patterns to JSON-compatible data", () => {
    expect(rx.set(rx.ascii.alnum, "/").oneOrMore().toJson()).toEqual({
      type: "one_or_more",
      pattern: {
        type: "set",
        items: [
          { type: "ascii", name: "alnum" },
          { type: "chars", value: "/" },
        ],
      },
    });
  });
});
