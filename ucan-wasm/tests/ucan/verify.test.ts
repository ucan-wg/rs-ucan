import assert from 'assert'
import { getFixture } from '../fixtures/index.js'

export function runVerifyTests(
  impl: {
    runner?: { describe, it },
    ucan: {
      isExpired: (token: string) => boolean
      isTooEarly: (token: string) => boolean
      checkSignature: (token: string) => Promise<void>
      validate: (token: string) => Promise<void>
    }
  }) {

  // Use runner or fallback to implicit mocha implementations
  const describe = impl.runner?.describe ?? globalThis.describe
  const it = impl.runner?.it ?? globalThis.it

  const { checkSignature, isExpired, isTooEarly, validate } = impl.ucan

  describe('validate', async () => {
    it('should resolve on a valid a UCAN', async () => {
      const validSignature = getFixture('valid', 'UCAN has a valid signature')
      const unexpired = getFixture('valid', 'UCAN has not expired')
      const active = getFixture('valid', 'UCAN is ready to be used')

      await assert.doesNotReject(validate(validSignature.token))
      await assert.doesNotReject(validate(unexpired.token))
      await assert.doesNotReject(validate(active.token))
    })

    it('should be true when a UCAN is expired', async () => {
      const invalidSignature = getFixture('invalid', 'UCAN has an invalid signature')
      const expired = getFixture('invalid', 'UCAN has expired')
      const early = getFixture('invalid', 'UCAN is not ready to be used')

      await assert.rejects(validate(invalidSignature.token))
      await assert.rejects(validate(expired.token))
      await assert.rejects(validate(early.token))
    })
  })

  describe('checkSignature', async () => {
    it('should resolve on a valid signature', async () => {
      const valid = getFixture('valid', 'UCAN has a valid signature')
      await assert.doesNotReject(checkSignature(valid.token))
    })

    it('should throw on an invalid signature', async () => {
      const invalid = getFixture('invalid', 'UCAN has an invalid signature')
      await assert.rejects(checkSignature(invalid.token))
    })
  })

  describe('isExpired', () => {
    it('should be false when a UCAN is active', () => {
      const valid = getFixture('valid', 'UCAN has not expired')
      assert.equal(isExpired(valid.token), false)
    })

    it('should be true when a UCAN is expired', () => {
      const invalid = getFixture('invalid', 'UCAN has expired')
      assert(isExpired(invalid.token))
    })
  })

  describe('isTooEarly', () => {
    it('should be false when a UCAN is active', () => {
      const valid = getFixture('valid', 'UCAN is ready to be used')
      assert.equal(isTooEarly(valid.token), false)
    })

    it('should be true when a UCAN is early', () => {
      const invalid = getFixture('invalid', 'UCAN is not ready to be used')
      assert(isTooEarly(invalid.token))
    })
  })
}
