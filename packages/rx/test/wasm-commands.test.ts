import { describe, expect, it } from "vitest";

import {
  createRx,
  createRxSync,
  emitRx,
  emitRxSync,
  explainRegex,
  explainRegexSync,
  formatRx,
  formatRxSync,
  lintRegex,
  lintRegexSync,
  parseRegex,
  parseRegexSync,
  rx,
  toRegex,
  toRegexSync,
} from "../src/node";

function pathPiece() {
  return rx.oneOrMore(
    rx.oneOf(
      rx.alphaNumeric(),
      rx.char("/"),
      rx.char("."),
      rx.char("-"),
      rx.char("_"),
    ),
  );
}

describe("Node and Bun WASM commands", () => {
  it("creates a fluent rx API after WASM initialization", async () => {
    const { rx } = await createRx();
    const pathPiece = rx.set(rx.ascii.alnum, "._/-").oneOrMore();

    expect(pathPiece.toRegex()).toBe("[A-Za-z0-9._/-]+");
    expect(pathPiece.toReadable()).toContain("one_or_more");
  });

  it("creates a fluent rx API synchronously", () => {
    const { rx } = createRxSync();

    expect(rx.set(rx.ascii.alnum, "/").oneOrMore().toRegex()).toBe("[A-Za-z0-9/]+");
  });

  it("emits regex synchronously from TypeScript builders through Rust", () => {
    expect(toRegexSync(pathPiece())).toBe("[A-Za-z0-9/._-]+");
  });

  it("keeps async and sync APIs equivalent on the Node/Bun target", async () => {
    await expect(toRegex(pathPiece())).resolves.toBe(toRegexSync(pathPiece()));
  });

  it("returns structured diagnostics for readable rx errors", () => {
    const result = emitRxSync("one_or_more(set(ascii.nope))");

    expect(result.regex).toBeUndefined();
    expect(result.diagnostics).toHaveLength(1);
    expect(result.diagnostics[0]).toMatchObject({
      severity: "error",
      category: "compatibility",
      sourceFamily: "readable rx",
    });
  });

  it("keeps async command wrappers equivalent to sync command wrappers", async () => {
    await expect(emitRx("start_text")).resolves.toEqual(emitRxSync("start_text"));
    await expect(explainRegex("[A-Za-z0-9/._-]+")).resolves.toEqual(
      explainRegexSync("[A-Za-z0-9/._-]+"),
    );
    await expect(formatRx("start_text")).resolves.toEqual(formatRxSync("start_text"));
    await expect(lintRegex("[\\w\\._/-]+")).resolves.toEqual(lintRegexSync("[\\w\\._/-]+"));
    await expect(parseRegex("[A-Za-z0-9/._-]+")).resolves.toEqual(
      parseRegexSync("[A-Za-z0-9/._-]+"),
    );
  });

  it("explains and parses supported legacy regex", () => {
    const explained = explainRegexSync("[A-Za-z0-9/._-]+");
    const parsed = parseRegexSync("[A-Za-z0-9/._-]+");

    expect(explained.readable).toContain("one_or_more");
    expect(explained.explanation).toContain("one or more");
    expect(parsed.regex).toBe("[A-Za-z0-9/._-]+");
    expect(parsed.diagnostics).toEqual([]);
  });

  it("formats readable rx through Rust", () => {
    const result = formatRxSync('one_or_more(set(ascii.alnum,chars("/._-")))');

    expect(result.readable).toMatchInlineSnapshot(`
      "one_or_more(
          set(
              ascii.alnum,
              chars("/._-")
          )
      )"
    `);
  });

  it("returns diagnostics for invalid dialect options across commands", () => {
    const options = { dialect: "javascript" as never };

    for (const result of [
      emitRxSync("start_text", options),
      explainRegexSync("abc", options),
      formatRxSync("start_text", options),
      lintRegexSync("abc", options),
      parseRegexSync("abc", options),
    ]) {
      expect(result.diagnostics[0]).toMatchObject({
        severity: "error",
        category: "dialect",
        sourceFamily: "dialect",
        message: "unsupported dialect `javascript`",
      });
    }
  });

  it("reports legacy lint diagnostics", () => {
    const result = lintRegexSync("[\\w\\._/-]+");

    expect(result.diagnostics.map((diagnostic) => diagnostic.category)).toContain("lint");
  });

  it("exposes WASM commands on the rx namespace", async () => {
    await expect(rx.explainRegex("[A-Za-z0-9/]+")).resolves.toMatchObject({
      regex: "[A-Za-z0-9/]+",
      diagnostics: [],
    });
    expect(rx.toRegexSync(rx.set(rx.ascii.alnum, "/").oneOrMore())).toBe("[A-Za-z0-9/]+");
  });
});
