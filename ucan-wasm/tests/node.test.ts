import { describe, it } from 'vitest'

import { checkSignature, decode, isExpired, isTooEarly, validate } from '../lib/node/ucan_wasm.js'
import { runTokenTests } from "./ucan/token.test.js"
import { runVerifyTests } from "./ucan/verify.test.js"

runVerifyTests({
  runner: { describe, it },
  ucan: {
      isExpired,
      isTooEarly,
      checkSignature,
      validate
  }
})

runTokenTests({
  runner: { describe, it },
  ucan: {
    decode
  }
})
