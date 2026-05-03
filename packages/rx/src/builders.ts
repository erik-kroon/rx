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

export class RxPattern {
  readonly kind: PatternKind;

  constructor(kind: PatternKind) {
    this.kind = kind;
  }

  toRx(): string {
    return printPattern(this, 0);
  }

  toRegex(emit: EmitCommand, options?: EmitOptions): string {
    const result = emit(this.toRx(), options);
    if (result.regex !== undefined) {
      return result.regex;
    }
    throw new RxError(result.diagnostics);
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

export const rx = {
  literal(value: string): RxPattern {
    return new RxPattern({ type: "literal", value });
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
    return { type: "range", start, end };
  },
  asciiWord(): SetItem {
    return { type: "ascii", name: "word" };
  },
  alphaNumeric(): SetItem {
    return { type: "ascii", name: "alnum" };
  },
  asciiAlpha(): SetItem {
    return { type: "ascii", name: "alpha" };
  },
  digit(): SetItem {
    return { type: "ascii", name: "digit" };
  },
  whitespace(): SetItem {
    return { type: "ascii", name: "whitespace" };
  },
  set(...items: SetItem[]): RxPattern {
    return new RxPattern({ type: "set", items });
  },
  oneOf(...items: SetItem[]): RxPattern {
    return this.set(...items);
  },
  sequence(...patterns: RxPattern[]): RxPattern {
    return new RxPattern({ type: "sequence", patterns });
  },
  either(...patterns: RxPattern[]): RxPattern {
    return new RxPattern({ type: "either", patterns });
  },
  zeroOrMore(pattern: RxPattern): RxPattern {
    return new RxPattern({ type: "repeat", name: "zero_or_more", pattern });
  },
  oneOrMore(pattern: RxPattern): RxPattern {
    return new RxPattern({ type: "repeat", name: "one_or_more", pattern });
  },
  optional(pattern: RxPattern): RxPattern {
    return new RxPattern({ type: "repeat", name: "optional", pattern });
  },
  repeat(pattern: RxPattern, count: number): RxPattern {
    assertNonNegativeInteger(count, "repeat count");
    return new RxPattern({ type: "repeat_exactly", pattern, count });
  },
  repeatBetween(pattern: RxPattern, min: number, max: number): RxPattern {
    assertNonNegativeInteger(min, "repeat min");
    assertNonNegativeInteger(max, "repeat max");
    return new RxPattern({ type: "repeat_between", pattern, min, max });
  },
  startText(): RxPattern {
    return new RxPattern({ type: "start_text" });
  },
  endText(): RxPattern {
    return new RxPattern({ type: "end_text" });
  },
  capture(pattern: RxPattern): RxPattern {
    return new RxPattern({ type: "capture", pattern });
  },
  namedCapture(name: string, pattern: RxPattern): RxPattern {
    return new RxPattern({ type: "capture", name, pattern });
  },
};

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
