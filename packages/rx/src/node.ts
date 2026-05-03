import { createRequire } from "node:module";

import type { CommandResult, EmitOptions } from "./builders";
import { createRxBuilder, RxError, RxPattern } from "./builders";

export type {
  CommandResult,
  Dialect,
  EmitCommand,
  EmitOptions,
  RxDiagnostic,
  SetItem,
  Span,
} from "./builders";
export { createRxBuilder, RxError, RxPattern };

interface NodeWasmModule {
  emitRx: (input: string, options?: EmitOptions) => CommandResult;
  explainRegex: (input: string, options?: EmitOptions) => CommandResult;
  formatRx: (input: string, options?: EmitOptions) => CommandResult;
  lintRegex: (input: string, options?: EmitOptions) => { diagnostics: CommandResult["diagnostics"] };
  parseRegex: (input: string, options?: EmitOptions) => CommandResult;
}

const require = createRequire(import.meta.url);
const wasm = require("../wasm-node/rx_wasm.js") as NodeWasmModule;

export async function initRx(): Promise<void> {}

export const rx = Object.assign(createRxBuilder(), {
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
  toRegex,
  toRegexSync,
});

export interface CreatedRx {
  rx: typeof rx;
  emitRx: typeof emitRx;
  emitRxSync: typeof emitRxSync;
  explainRegex: typeof explainRegex;
  explainRegexSync: typeof explainRegexSync;
  formatRx: typeof formatRx;
  formatRxSync: typeof formatRxSync;
  lintRegex: typeof lintRegex;
  lintRegexSync: typeof lintRegexSync;
  parseRegex: typeof parseRegex;
  parseRegexSync: typeof parseRegexSync;
  toRegex: typeof toRegex;
  toRegexSync: typeof toRegexSync;
}

export function createRxSync(): CreatedRx {
  return {
    rx,
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
    toRegex,
    toRegexSync,
  };
}

export async function createRx(): Promise<CreatedRx> {
  return createRxSync();
}

export function emitRxSync(input: string | RxPattern, options?: EmitOptions): CommandResult {
  return wasm.emitRx(typeof input === "string" ? input : input.toRx(), options);
}

export async function emitRx(input: string | RxPattern, options?: EmitOptions): Promise<CommandResult> {
  return emitRxSync(input, options);
}

export function explainRegexSync(input: string, options?: EmitOptions): CommandResult {
  return wasm.explainRegex(input, options);
}

export async function explainRegex(input: string, options?: EmitOptions): Promise<CommandResult> {
  return explainRegexSync(input, options);
}

export function formatRxSync(input: string | RxPattern, options?: EmitOptions): CommandResult {
  return wasm.formatRx(typeof input === "string" ? input : input.toRx(), options);
}

export async function formatRx(input: string | RxPattern, options?: EmitOptions): Promise<CommandResult> {
  return formatRxSync(input, options);
}

export function lintRegexSync(
  input: string,
  options?: EmitOptions,
): { diagnostics: CommandResult["diagnostics"] } {
  return wasm.lintRegex(input, options);
}

export async function lintRegex(
  input: string,
  options?: EmitOptions,
): Promise<{ diagnostics: CommandResult["diagnostics"] }> {
  return lintRegexSync(input, options);
}

export function parseRegexSync(input: string, options?: EmitOptions): CommandResult {
  return wasm.parseRegex(input, options);
}

export async function parseRegex(input: string, options?: EmitOptions): Promise<CommandResult> {
  return parseRegexSync(input, options);
}

export function toRegexSync(input: string | RxPattern, options?: EmitOptions): string {
  const result = emitRxSync(input, options);
  if (result.regex !== undefined) {
    return result.regex;
  }
  throw new RxError(result.diagnostics);
}

export async function toRegex(input: string | RxPattern, options?: EmitOptions): Promise<string> {
  return toRegexSync(input, options);
}
