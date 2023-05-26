// This entry point is inserted into ./lib/workerd to support Cloudflare workers

import WASM from "./ucan_wasm_bg.wasm";
import { initSync } from "./ucan_wasm.js";
initSync(WASM);
export * from "./ucan_wasm.js";
