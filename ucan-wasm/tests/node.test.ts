import { describe, it } from 'vitest'

import { checkSignature, isExpired, isTooEarly, validate } from '../lib/node/ucan_wasm.js'
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
