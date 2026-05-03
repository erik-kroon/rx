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

interface WasmModule {
  default?: () => Promise<void>;
  emitRx: (input: string, options?: EmitOptions) => CommandResult;
  explainRegex: (input: string, options?: EmitOptions) => CommandResult;
  formatRx: (input: string, options?: EmitOptions) => CommandResult;
  lintRegex: (input: string, options?: EmitOptions) => { diagnostics: CommandResult["diagnostics"] };
  parseRegex: (input: string, options?: EmitOptions) => CommandResult;
}

let wasmModule: Promise<WasmModule> | undefined;

export const rx = Object.assign(createRxBuilder(), {
  emitRx,
  explainRegex,
  formatRx,
  lintRegex,
  parseRegex,
  toRegex,
});

export async function initRx(): Promise<void> {
  await loadWasm();
}

export interface CreatedRx {
  rx: typeof rx;
  emitRx: typeof emitRx;
  explainRegex: typeof explainRegex;
  formatRx: typeof formatRx;
  lintRegex: typeof lintRegex;
  parseRegex: typeof parseRegex;
  toRegex: typeof toRegex;
}

export async function createRx(): Promise<CreatedRx> {
  const wasm = await loadWasm();
  return {
    rx,
    emitRx,
    explainRegex,
    formatRx,
    lintRegex,
    parseRegex,
    toRegex,
  };
}

export async function emitRx(input: string | RxPattern, options?: EmitOptions): Promise<CommandResult> {
  const wasm = await loadWasm();
  return wasm.emitRx(typeof input === "string" ? input : input.toRx(), options);
}

export async function explainRegex(input: string, options?: EmitOptions): Promise<CommandResult> {
  return (await loadWasm()).explainRegex(input, options);
}

export async function formatRx(input: string | RxPattern, options?: EmitOptions): Promise<CommandResult> {
  const wasm = await loadWasm();
  return wasm.formatRx(typeof input === "string" ? input : input.toRx(), options);
}

export async function lintRegex(
  input: string,
  options?: EmitOptions,
): Promise<{ diagnostics: CommandResult["diagnostics"] }> {
  return (await loadWasm()).lintRegex(input, options);
}

export async function parseRegex(input: string, options?: EmitOptions): Promise<CommandResult> {
  return (await loadWasm()).parseRegex(input, options);
}

export async function toRegex(input: string | RxPattern, options?: EmitOptions): Promise<string> {
  const result = await emitRx(input, options);
  if (result.regex !== undefined) {
    return result.regex;
  }
  throw new RxError(result.diagnostics);
}

async function loadWasm(): Promise<WasmModule> {
  const moduleUrl = new URL("../wasm/rx_wasm.js", import.meta.url).href;
  wasmModule ??= import(/* @vite-ignore */ moduleUrl).then(async (module) => {
    const wasm = module as WasmModule;
    await wasm.default?.();
    return wasm;
  });
  return wasmModule;
}
