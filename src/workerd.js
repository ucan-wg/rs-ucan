// This entry point is inserted into ./lib/workerd to support Cloudflare workers

import WASM from "./rs-ucan_bg.wasm";
import { initSync } from "./rs-ucan.js";
initSync(WASM);
export * from "./rs-ucan.js";
