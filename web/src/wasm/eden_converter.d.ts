/* tslint:disable */
/* eslint-disable */

export function convert(eden_bytes: Uint8Array, mapping_json?: string | null): Uint8Array;

/**
 * Return the default block mapping as JSON (for the web UI to display).
 */
export function default_mapping_json(): string;

/**
 * Generate a procedural Eden world and return raw .eden file bytes.
 * `params_json` must be a JSON object with fields:
 *   width (u32), depth (u32), seed (u32),
 *   base_height (i32, optional, default 30),
 *   water_amnt (u32 1-5, optional, default 3)
 * Returns JSON: { eden: <base64 eden bytes>, stats: { ... } }
 */
export function generate_world(params_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly convert: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly default_mapping_json: () => [number, number];
    readonly generate_world: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
