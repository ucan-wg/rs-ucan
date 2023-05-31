import init, { checkSignature, isExpired, isTooEarly, validate } from '../lib/browser/ucan_wasm.js'
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
