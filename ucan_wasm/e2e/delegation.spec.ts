import { test, expect } from '@playwright/test';
import { URL } from './config';

test.beforeEach(async ({ page }) => {
  await page.goto(URL)
  await page.waitForFunction(() => !!window.ucan)
});

test.describe("Document", async () => {
  test('constructor', async ({ page }) => {
    const out = await page.evaluate(async () => {
      // const { .. } = window.ucan
      // return { doc, docId }
    })

    expect(out.doc).toBeDefined()
    expect(out.docId).toBeDefined()
  })
})
