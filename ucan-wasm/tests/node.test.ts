import * as ucan from '../lib/node/ucan_wasm.js'
import { runCIDTests } from "./ucan/cid.test.js"
import { runTokenTests } from "./ucan/token.test.js"
import { runVerifyTests } from "./ucan/verify.test.js"


const { checkSignature, decode, isExpired, isTooEarly, toCID, validate } = ucan

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
  ucan: {
    toCID
  }
})
