import init, { checkSignature, decode, isExpired, isTooEarly, toCID, validate } from '../lib/browser/ucan_wasm.js'
import { runCIDTests } from "./ucan/cid.test.js"
import { runTokenTests } from "./ucan/token.test.js"
import { runVerifyTests } from "./ucan/verify.test.js"

before(async () => {
  await init()
})

runVerifyTests({
  ucan: {
    isExpired,
    isTooEarly,
    checkSignature,
    validate
  }
})


runTokenTests({
  ucan: {
    decode
  }
})

runCIDTests({
  runner: { describe, it },
  ucan: {
    toCID
  }
})
