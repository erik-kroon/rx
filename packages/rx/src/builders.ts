export type Dialect = "rust-regex" | "rust" | "rust_regex" | "pcre2" | "posix-ere" | "posix_ere";

export interface EmitOptions {
  dialect?: Dialect;
}

export interface Span {
  start: number;
  end: number;
}

export interface RxDiagnostic {
  severity: "error" | "warning" | string;
  category: string;
  sourceFamily: string;
  message: string;
  suggestion?: string;
  span: Span;
}

export interface CommandResult {
  readable?: string;
  regex?: string;
  explanation?: string;
  diagnostics: RxDiagnostic[];
}

export type EmitCommand = (readable: string, options?: EmitOptions) => CommandResult;

export interface RxApi {
  readonly ascii: {
    readonly word: SetItem;
    readonly alnum: SetItem;
    readonly alpha: SetItem;
    readonly digit: SetItem;
    readonly whitespace: SetItem;
  };
  literal(value: string): RxPattern;
  char(value: string): SetItem;
  chars(value: string): SetItem;
  range(start: string, end: string): SetItem;
  asciiWord(): SetItem;
  alphaNumeric(): SetItem;
  asciiAlpha(): SetItem;
  digit(): SetItem;
  whitespace(): SetItem;
  set(...items: SetInput[]): RxPattern;
  oneOf(...items: SetInput[]): RxPattern;
  seq(...patterns: RxPattern[]): RxPattern;
  sequence(...patterns: RxPattern[]): RxPattern;
  either(...patterns: RxPattern[]): RxPattern;
  zeroOrMore(pattern: RxPattern): RxPattern;
  oneOrMore(pattern: RxPattern): RxPattern;
  optional(pattern: RxPattern): RxPattern;
  repeat(pattern: RxPattern, count: number): RxPattern;
  repeatBetween(pattern: RxPattern, min: number, max: number): RxPattern;
  startText(): RxPattern;
  endText(): RxPattern;
  capture(pattern: RxPattern): RxPattern;
  namedCapture(name: string, pattern: RxPattern): RxPattern;
}

type PatternKind =
  | { type: "literal"; value: string }
  | { type: "set"; items: SetItem[] }
  | { type: "sequence"; patterns: RxPattern[] }
  | { type: "either"; patterns: RxPattern[] }
  | { type: "repeat"; name: "zero_or_more" | "one_or_more" | "optional"; pattern: RxPattern }
  | { type: "repeat_exactly"; pattern: RxPattern; count: number }
  | { type: "repeat_between"; pattern: RxPattern; min: number; max: number }
  | { type: "start_text" }
  | { type: "end_text" }
  | { type: "capture"; pattern: RxPattern; name?: string };

export type SetItem =
  | { type: "char"; value: string }
  | { type: "chars"; value: string }
  | { type: "range"; start: string; end: string }
  | { type: "ascii"; name: "word" | "alnum" | "alpha" | "digit" | "whitespace" };

export type SetInput = SetItem | string;

export class RxPattern {
  readonly kind: PatternKind;

  constructor(kind: PatternKind) {
    this.kind = kind;
  }

  toRx(): string {
    return printPattern(this, 0);
  }

  toReadable(): string {
    return this.toRx();
  }

  toRegex(options?: EmitOptions): string;
  toRegex(emit: EmitCommand, options?: EmitOptions): string;
  toRegex(first?: EmitOptions | EmitCommand, second?: EmitOptions): string {
    const emit = typeof first === "function" ? first : undefined;
    const options = typeof first === "function" ? second : first;
    if (emit !== undefined) {
      const result = emit(this.toRx(), options);
      if (result.regex !== undefined) {
        return result.regex;
      }
      throw new RxError(result.diagnostics);
    }
    ensurePureEmitDialect(options);
    return emitPattern(this);
  }

  toJson(): unknown {
    return patternToJson(this);
  }

  toRegExp(options?: EmitOptions): RegExp {
    return new RegExp(this.toRegex(options));
  }

  zeroOrMore(): RxPattern {
    return new RxPattern({ type: "repeat", name: "zero_or_more", pattern: this });
  }

  oneOrMore(): RxPattern {
    return new RxPattern({ type: "repeat", name: "one_or_more", pattern: this });
  }

  optional(): RxPattern {
    return new RxPattern({ type: "repeat", name: "optional", pattern: this });
  }

  repeat(count: number): RxPattern {
    assertNonNegativeInteger(count, "repeat count");
    return new RxPattern({ type: "repeat_exactly", pattern: this, count });
  }

  repeatBetween(min: number, max: number): RxPattern {
    assertNonNegativeInteger(min, "repeat min");
    assertNonNegativeInteger(max, "repeat max");
    assertRepeatBounds(min, max);
    return new RxPattern({ type: "repeat_between", pattern: this, min, max });
  }
}

export class RxError extends Error {
  readonly diagnostics: RxDiagnostic[];

  constructor(diagnostics: RxDiagnostic[]) {
    super(diagnostics[0]?.message ?? "rx command failed");
    this.name = "RxError";
    this.diagnostics = diagnostics;
  }
}

const ascii = {
  word: { type: "ascii", name: "word" },
  alnum: { type: "ascii", name: "alnum" },
  alpha: { type: "ascii", name: "alpha" },
  digit: { type: "ascii", name: "digit" },
  whitespace: { type: "ascii", name: "whitespace" },
} as const satisfies RxApi["ascii"];

export function createRxBuilder(): RxApi {
  const build = (kind: PatternKind) => new RxPattern(kind);

  return {
    ascii,
    literal(value: string): RxPattern {
      return build({ type: "literal", value });
    },
    char(value: string): SetItem {
      assertOneCharacter(value, "char");
      return { type: "char", value };
    },
    chars(value: string): SetItem {
      return { type: "chars", value };
    },
    range(start: string, end: string): SetItem {
      assertOneCharacter(start, "range start");
      assertOneCharacter(end, "range end");
      assertRangeBounds(start, end);
      return { type: "range", start, end };
    },
    asciiWord(): SetItem {
      return ascii.word;
    },
    alphaNumeric(): SetItem {
      return ascii.alnum;
    },
    asciiAlpha(): SetItem {
      return ascii.alpha;
    },
    digit(): SetItem {
      return ascii.digit;
    },
    whitespace(): SetItem {
      return ascii.whitespace;
    },
    set(...items: SetInput[]): RxPattern {
      return build({ type: "set", items: items.map(normalizeSetInput) });
    },
    oneOf(...items: SetInput[]): RxPattern {
      return this.set(...items);
    },
    seq(...patterns: RxPattern[]): RxPattern {
      return this.sequence(...patterns);
    },
    sequence(...patterns: RxPattern[]): RxPattern {
      return build({ type: "sequence", patterns });
    },
    either(...patterns: RxPattern[]): RxPattern {
      return build({ type: "either", patterns });
    },
    zeroOrMore(pattern: RxPattern): RxPattern {
      return build({ type: "repeat", name: "zero_or_more", pattern });
    },
    oneOrMore(pattern: RxPattern): RxPattern {
      return build({ type: "repeat", name: "one_or_more", pattern });
    },
    optional(pattern: RxPattern): RxPattern {
      return build({ type: "repeat", name: "optional", pattern });
    },
    repeat(pattern: RxPattern, count: number): RxPattern {
      assertNonNegativeInteger(count, "repeat count");
      return build({ type: "repeat_exactly", pattern, count });
    },
    repeatBetween(pattern: RxPattern, min: number, max: number): RxPattern {
      assertNonNegativeInteger(min, "repeat min");
      assertNonNegativeInteger(max, "repeat max");
      assertRepeatBounds(min, max);
      return build({ type: "repeat_between", pattern, min, max });
    },
    startText(): RxPattern {
      return build({ type: "start_text" });
    },
    endText(): RxPattern {
      return build({ type: "end_text" });
    },
    capture(pattern: RxPattern): RxPattern {
      return build({ type: "capture", pattern });
    },
    namedCapture(name: string, pattern: RxPattern): RxPattern {
      assertCaptureName(name);
      return build({ type: "capture", name, pattern });
    },
  };
}

export const rx = createRxBuilder();

function normalizeSetInput(input: SetInput): SetItem {
  return typeof input === "string" ? { type: "chars", value: input } : input;
}

function ensurePureEmitDialect(options: EmitOptions | undefined): void {
  if (
    options?.dialect !== undefined &&
    !["rust", "rust-regex", "rust_regex"].includes(options.dialect)
  ) {
    throw new TypeError("Pure TypeScript builder emission currently supports only rust-regex dialect output.");
  }
}

function emitPattern(pattern: RxPattern): string {
  switch (pattern.kind.type) {
    case "literal":
      return escapeRegexLiteral(pattern.kind.value);
    case "set":
      return emitSet(pattern.kind.items);
    case "sequence":
      return pattern.kind.patterns.map(emitSequenceItem).join("");
    case "either":
      return pattern.kind.patterns.map(emitPattern).join("|");
    case "repeat":
      return `${emitRepeatAtom(pattern.kind.pattern)}${repeatSuffix(pattern.kind.name)}`;
    case "repeat_exactly":
      return `${emitRepeatAtom(pattern.kind.pattern)}{${pattern.kind.count}}`;
    case "repeat_between":
      return `${emitRepeatAtom(pattern.kind.pattern)}{${pattern.kind.min},${pattern.kind.max}}`;
    case "start_text":
      return "^";
    case "end_text":
      return "$";
    case "capture":
      if (pattern.kind.name !== undefined) {
        return `(?<${pattern.kind.name}>${emitPattern(pattern.kind.pattern)})`;
      }
      return `(${emitPattern(pattern.kind.pattern)})`;
  }
}

function emitSequenceItem(pattern: RxPattern): string {
  return pattern.kind.type === "either" ? `(?:${emitPattern(pattern)})` : emitPattern(pattern);
}

function emitRepeatAtom(pattern: RxPattern): string {
  if (pattern.kind.type === "literal" && [...pattern.kind.value].length === 1) {
    return escapeRegexLiteral(pattern.kind.value);
  }
  if (pattern.kind.type === "set" || pattern.kind.type === "capture") {
    return emitPattern(pattern);
  }
  return `(?:${emitPattern(pattern)})`;
}

function repeatSuffix(name: "zero_or_more" | "one_or_more" | "optional"): string {
  switch (name) {
    case "zero_or_more":
      return "*";
    case "one_or_more":
      return "+";
    case "optional":
      return "?";
  }
}

function emitSet(items: SetItem[]): string {
  let output = "[";
  let hasHyphen = false;
  for (const item of items) {
    switch (item.type) {
      case "char":
        if (item.value === "-") {
          hasHyphen = true;
        } else {
          output += escapeSetLiteral(item.value);
        }
        break;
      case "chars":
        for (const char of item.value) {
          if (char === "-") {
            hasHyphen = true;
          } else {
            output += escapeSetLiteral(char);
          }
        }
        break;
      case "range":
        output += `${escapeSetLiteral(item.start)}-${escapeSetLiteral(item.end)}`;
        break;
      case "ascii":
        output += asciiRegexFragment(item.name);
        break;
    }
  }
  if (hasHyphen) {
    output += "-";
  }
  return `${output}]`;
}

function asciiRegexFragment(name: Extract<SetItem, { type: "ascii" }>["name"]): string {
  switch (name) {
    case "word":
      return "A-Za-z0-9_";
    case "alnum":
      return "A-Za-z0-9";
    case "alpha":
      return "A-Za-z";
    case "digit":
      return "0-9";
    case "whitespace":
      return "\\t\\n\\f\\r ";
  }
}

function escapeRegexLiteral(value: string): string {
  return value.replace(/[\\.+*?()[\]{}^$|]/g, "\\$&");
}

function escapeSetLiteral(value: string): string {
  return value.replace(/[\\\]\-^]/g, "\\$&");
}

function patternToJson(pattern: RxPattern): unknown {
  switch (pattern.kind.type) {
    case "sequence":
    case "either":
      return {
        type: pattern.kind.type,
        patterns: pattern.kind.patterns.map(patternToJson),
      };
    case "repeat":
      return {
        type: pattern.kind.name,
        pattern: patternToJson(pattern.kind.pattern),
      };
    case "repeat_exactly":
      return {
        type: "repeat",
        pattern: patternToJson(pattern.kind.pattern),
        count: pattern.kind.count,
      };
    case "repeat_between":
      return {
        type: "repeat_between",
        pattern: patternToJson(pattern.kind.pattern),
        min: pattern.kind.min,
        max: pattern.kind.max,
      };
    case "capture":
      return {
        type: pattern.kind.name === undefined ? "capture" : "named_capture",
        name: pattern.kind.name,
        pattern: patternToJson(pattern.kind.pattern),
      };
    default:
      return pattern.kind;
  }
}

function printPattern(pattern: RxPattern, indent: number): string {
  switch (pattern.kind.type) {
    case "literal":
      return `literal("${escapeRxString(pattern.kind.value)}")`;
    case "set":
      return printCall("set", pattern.kind.items.map(printSetItem), indent);
    case "sequence":
      return printCall(
        "sequence",
        pattern.kind.patterns.map((item) => printPattern(item, indent + 4)),
        indent,
      );
    case "either":
      return printCall(
        "either",
        pattern.kind.patterns.map((item) => printPattern(item, indent + 4)),
        indent,
      );
    case "repeat":
      return printUnaryCall(pattern.kind.name, printPattern(pattern.kind.pattern, indent + 4), indent);
    case "repeat_exactly":
      return printCall(
        "repeat",
        [printPattern(pattern.kind.pattern, indent + 4), String(pattern.kind.count)],
        indent,
      );
    case "repeat_between":
      return printCall(
        "repeat_between",
        [
          printPattern(pattern.kind.pattern, indent + 4),
          String(pattern.kind.min),
          String(pattern.kind.max),
        ],
        indent,
      );
    case "start_text":
      return "start_text";
    case "end_text":
      return "end_text";
    case "capture":
      if (pattern.kind.name !== undefined) {
        return printCall(
          "named_capture",
          [`"${escapeRxString(pattern.kind.name)}"`, printPattern(pattern.kind.pattern, indent + 4)],
          indent,
        );
      }
      return printUnaryCall("capture", printPattern(pattern.kind.pattern, indent + 4), indent);
  }
}

function printSetItem(item: SetItem): string {
  switch (item.type) {
    case "char":
      return `char("${escapeRxString(item.value)}")`;
    case "chars":
      return `chars("${escapeRxString(item.value)}")`;
    case "range":
      return `range("${escapeRxString(item.start)}", "${escapeRxString(item.end)}")`;
    case "ascii":
      return `ascii.${item.name}`;
  }
}

function printUnaryCall(name: string, value: string, indent: number): string {
  return `${name}(\n${" ".repeat(indent + 4)}${value}\n${" ".repeat(indent)})`;
}

function printCall(name: string, values: string[], indent: number): string {
  if (values.length === 0) {
    return `${name}()`;
  }
  return `${name}(\n${values.map((value) => `${" ".repeat(indent + 4)}${value}`).join(",\n")}\n${" ".repeat(indent)})`;
}

function escapeRxString(value: string): string {
  return value
    .replaceAll("\\", "\\\\")
    .replaceAll('"', '\\"')
    .replaceAll("\n", "\\n")
    .replaceAll("\r", "\\r")
    .replaceAll("\t", "\\t");
}

function assertOneCharacter(value: string, context: string): void {
  if ([...value].length !== 1) {
    throw new TypeError(`${context} must contain exactly one character`);
  }
}

function assertNonNegativeInteger(value: number, context: string): void {
  if (!Number.isInteger(value) || value < 0) {
    throw new TypeError(`${context} must be a non-negative integer`);
  }
}

function assertRepeatBounds(min: number, max: number): void {
  if (min > max) {
    throw new TypeError("repeat min must be less than or equal to repeat max");
  }
}

function assertRangeBounds(start: string, end: string): void {
  if (start > end) {
    throw new TypeError("range start must be less than or equal to range end");
  }
}

function assertCaptureName(name: string): void {
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) {
    throw new TypeError(
      "capture name must start with a letter or underscore and contain only letters, digits, or underscore",
    );
  }
}
