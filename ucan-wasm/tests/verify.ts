import { beforeAll, expect, test } from 'vitest'

import { getFixture } from './fixtures'
import init, { isExpired, isTooEarly } from '../pkg/ucan_wasm'

beforeAll(async () => {
  await init()
})

test('isExpired should report active and expired UCANs', async () => {
  const valid = getFixture('valid', 'UCAN has not expired')
  expect(isExpired(valid.token)).toBe(false)

  const invalid = getFixture('invalid', 'UCAN has expired')
  expect(isExpired(invalid.token)).toBe(true)
})

test('isTooEarly should report active and early UCANs', async () => {
  const valid = getFixture('valid', 'UCAN is ready to be used')
  expect(isTooEarly(valid.token)).toBe(false)

  const invalid = getFixture('invalid', 'UCAN is not ready to be used')
  expect(isTooEarly(invalid.token)).toBe(true)
})
