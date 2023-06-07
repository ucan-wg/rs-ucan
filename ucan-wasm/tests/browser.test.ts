import init, { checkSignature, decode, isExpired, isTooEarly, validate } from '../lib/browser/ucan_wasm.js'
import { runVerifyTests } from "./ucan/verify.test.js"
import { runTokenTests } from "./ucan/token.test.js"

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
