import { describe, expect, test } from 'vitest'

import { getFixture } from '../fixtures'
import { checkSignature, isExpired, isTooEarly, validate } from '../../lib/node/ucan_wasm.js'

describe('validate', async () => {
  test('should resolve on a valid a UCAN', async () => {
    const validSignature = getFixture('valid', 'UCAN has a valid signature')
    const unexpired = getFixture('valid', 'UCAN has not expired')
    const active = getFixture('valid', 'UCAN is ready to be used')

    await expect(validate(validSignature.token)).resolves.toBe(undefined);
    await expect(validate(unexpired.token)).resolves.toBe(undefined);
    await expect(validate(active.token)).resolves.toBe(undefined);
  })

  test('should be true when a UCAN is expired', async () => {
    const invalidSignature = getFixture('invalid', 'UCAN has an invalid signature')
    const expired = getFixture('invalid', 'UCAN has expired')
    const early = getFixture('invalid', 'UCAN is not ready to be used')

    await expect(validate(invalidSignature.token)).rejects.toThrowError()
    await expect(validate(expired.token)).rejects.toThrowError()
    await expect(validate(early.token)).rejects.toThrowError()
  })
})

describe('checkSignature', async () => {
  test('should resolve on a valid signature', async () => {
    const valid = getFixture('valid', 'UCAN has a valid signature')
    await expect(checkSignature(valid.token)).resolves.toBe(undefined);
  })

  test('should throw on an invalid signature', async () => {
    const invalid = getFixture('invalid', 'UCAN has an invalid signature')
    await expect(checkSignature(invalid.token)).rejects.toThrowError()
  })
})

describe('isExpired', () => {
  test('should be false when a UCAN is active', () => {
    const valid = getFixture('valid', 'UCAN has not expired')
    expect(isExpired(valid.token)).toBe(false)
  })

  test('should be true when a UCAN is expired', () => {
    const invalid = getFixture('invalid', 'UCAN has expired')
    expect(isExpired(invalid.token)).toBe(true)
  })
})

describe('isTooEarly', () => {
  test('should be false when a UCAN is active', () => {
    const valid = getFixture('valid', 'UCAN is ready to be used')
    expect(isTooEarly(valid.token)).toBe(false)
  })

  test('should be true when a UCAN is early', () => {
    const invalid = getFixture('invalid', 'UCAN is not ready to be used')
    expect(isTooEarly(invalid.token)).toBe(true)
  })
})
