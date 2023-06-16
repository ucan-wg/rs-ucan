### Build

The build command output for `browser`, `node`, `deno`, and `workerd` targets to `ucan-wasm/lib`.

```
npm run build
```

### Test

The test command tests `node` and browser environments including `chromium`, `firefox`, and `webkit`
using headless browsers.

```
npm run test
```

Testing can also be run for `node` only.

```
npm run test:node
```

The test commands runs `build` to ensure all targets are available for testing.

### Reporting

Test runs output test results and coverage as JSON artifacts to `ucan-wasm/tests/report`.
