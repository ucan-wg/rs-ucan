# Changelog

## [0.3.1](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.3.0...ucan-v0.3.1) (2023-05-23)


### Features

* add PartialEq, Eq traits to Ucan. Fixes [#90](https://github.com/ucan-wg/rs-ucan/issues/90) ([#91](https://github.com/ucan-wg/rs-ucan/issues/91)) ([27c3628](https://github.com/ucan-wg/rs-ucan/commit/27c36288fc47bd53ab6e8f4c3e8a596714dcc6ff))

## [0.3.0](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.2.0...ucan-v0.3.0) (2023-05-22)


### ⚠ BREAKING CHANGES

* Migrate default hashing from blake2b to blake3. ([#85](https://github.com/ucan-wg/rs-ucan/issues/85))
* Remove `stdweb` feature from instant crate to circumvent downstream issues with `stdweb/wasm-bindgen` ([#86](https://github.com/ucan-wg/rs-ucan/issues/86))

### Features

* Migrate default hashing from blake2b to blake3. ([#85](https://github.com/ucan-wg/rs-ucan/issues/85)) ([205cb96](https://github.com/ucan-wg/rs-ucan/commit/205cb962fcc99814caac8e1b9d4f8ffd956eb184))


### Bug Fixes

* Remove `stdweb` feature from instant crate to circumvent downstream issues with `stdweb/wasm-bindgen` ([#86](https://github.com/ucan-wg/rs-ucan/issues/86)) ([67ec64d](https://github.com/ucan-wg/rs-ucan/commit/67ec64db527b8bfadc4a219a65b580bdbc459640))

## [0.2.0](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.1.2...ucan-v0.2.0) (2023-05-04)


### ⚠ BREAKING CHANGES

* Custom 'now' for proof chain validation ([#83](https://github.com/ucan-wg/rs-ucan/issues/83))

### Features

* Custom 'now' for proof chain validation ([#83](https://github.com/ucan-wg/rs-ucan/issues/83)) ([1732a89](https://github.com/ucan-wg/rs-ucan/commit/1732a8911b67546f446126e4d469126f61769b44))

## [0.1.2](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.1.1...ucan-v0.1.2) (2023-04-22)


### Features

* Upgrade deps: `cid`, `libipld`, `base64`, `p256`, `rsa` ([#78](https://github.com/ucan-wg/rs-ucan/issues/78)) ([cfeed69](https://github.com/ucan-wg/rs-ucan/commit/cfeed6903d9a53d3728f35914d670e3b7920d88d))

## [0.1.1](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.1.0...ucan-v0.1.1) (2023-03-13)


### Features

* More derives for use in other libs ([#75](https://github.com/ucan-wg/rs-ucan/issues/75)) ([e60715f](https://github.com/ucan-wg/rs-ucan/commit/e60715f94f3b15b27ae7c1443cd4abae983d93ae))

## [0.1.0](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.1.0...ucan-v0.1.0) (2022-11-29)


### ⚠ BREAKING CHANGES

* New version requirements include `cid@0.9`, `libipld-core@0.15` and `libipld-json@0.15`

### Miscellaneous Chores

* Update IPLD-adjacent crates ([#55](https://github.com/ucan-wg/rs-ucan/issues/55)) ([bf55a3f](https://github.com/ucan-wg/rs-ucan/commit/bf55a3ffad0095d88c6b33b0cd6504e66918064a))
