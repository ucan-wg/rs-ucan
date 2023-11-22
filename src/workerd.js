// This entry point is inserted into ./lib/workerd to support Cloudflare workers

import WASM from "./rs_ucan_bg.wasm";
import { initSync } from "./rs_ucan.js";
initSync(WASM);
export * from "./rs_ucan.js";
