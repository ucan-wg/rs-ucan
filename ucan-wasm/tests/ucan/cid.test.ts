import assert from 'assert'
import { getCIDFixture } from '../fixtures/index.js'

export function runCIDTests(
  impl: {
    runner?: { describe, it },
    ucan: {
      toCID: (token: string, hasher?: string) => Promise<string>
    }
  }) {

  // Use runner or fallback to implicit mocha implementations
  const describe = impl.runner?.describe ?? globalThis.describe
  const it = impl.runner?.it ?? globalThis.it

  const { toCID } = impl.ucan

  describe('toCID', async () => {
    it('should compute CID for a UCAN using a SHA2-256 hasher', async () => {
      const fixture = getCIDFixture('SHA2-256')
      const cid = await toCID(fixture.token, 'SHA2-256')
      assert.equal(cid, fixture.cid)
    })

    it('should compute CID for a UCAN using a BLAKE3-256 hasher', async () => {
      const fixture = getCIDFixture('BLAKE3-256')
      const cid = await toCID(fixture.token, 'BLAKE3-256')
      assert.equal(cid, fixture.cid)
    })

    it('should compute CID for a UCAN deafulting to the BLAKE3-256 hasher', async () => {
      const fixture = getCIDFixture('BLAKE3-256')
      const cid = await toCID(fixture.token)
      assert.equal(cid, fixture.cid)
    })
  })
}
