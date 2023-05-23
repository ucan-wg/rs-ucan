import { beforeAll, expect, test } from 'vitest'

import init from '../pkg/ucan_wasm'

beforeAll(async () => {
  await init()
})

test("should work as expected", () => {
  expect(1 + 1).toEqual(2)

})
