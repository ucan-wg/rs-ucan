import { beforeAll, describe, expect, test } from 'vitest'

import { getFixture } from './fixtures'
import init, { checkSignature, isExpired, isTooEarly } from '../pkg/ucan_wasm'

beforeAll(async () => {
  await init()
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

describe('checkSignature', async () => {
  test('should report a valid signature', async () => {
    const valid = getFixture('valid', 'UCAN has a valid signature')
    await expect(checkSignature(valid.token)).resolves.toBe(undefined);
  })

  test('should throw on an invalid signature', async () => {
    const invalid = getFixture('invalid', 'UCAN has an invalid signature')
    await expect(checkSignature(invalid.token)).rejects.toThrowError()
  })
})
