// This entry point is inserted into ./lib/workerd to support Cloudflare workers

import WASM from "./ucan_bg.wasm";
import { initSync } from "./ucan.js";
initSync(WASM);
export * from "./ucan.js";
