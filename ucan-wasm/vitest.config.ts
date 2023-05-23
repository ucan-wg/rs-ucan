import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: [ 'tests/*.ts' ],
    browser: {
      enabled: true,
      name: 'chrome',
      headless: true
    },
  },
})
