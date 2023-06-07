import assert from 'assert'
import { getFixture } from '../fixtures/index.js'

// The Ucan type is the same across browser and node environments
import type { Ucan } from '../../lib/browser/ucan_wasm.js'

export function runTokenTests(
  impl: {
    runner?: { describe, it },
    ucan: {
      decode: (token: string) => Promise<Ucan>
    }
  }) {

  // Use runner or fallback to implicit mocha implementations
  const describe = impl.runner?.describe ?? globalThis.describe
  const it = impl.runner?.it ?? globalThis.it

  const { decode } = impl.ucan

  describe('decode', async () => {
    it('should decode a token', async () => {
      const valid = getFixture('valid', 'UCAN is valid')
      const ucan = await decode(valid.token)

      // Check header
      assert.equal(ucan.header.alg, valid.assertions.header.alg)
      assert.equal(ucan.header.typ, valid.assertions.header.typ)
      assert.equal(ucan.header.ucv, valid.assertions.header.ucv)

      // Check payload
      assert.equal(ucan.payload.iss, valid.assertions.payload.iss)
      assert.equal(ucan.payload.aud, valid.assertions.payload.aud)
      assert.equal(ucan.payload.exp, valid.assertions.payload.exp)
      assert.equal(ucan.payload.nbf, valid.assertions.payload.nbf)
      assert.equal(ucan.payload.nnc, valid.assertions.payload.nnc)
      assert.deepEqual(ucan.payload.att, valid.assertions.payload.att)
      assert.deepEqual(ucan.payload.fct, valid.assertions.payload.fct)
      assert.deepEqual(ucan.payload.prf, valid.assertions.payload.prf)

      // Check signature
      assert.equal(ucan.signature, valid.assertions.signature)
    })
  })

}
