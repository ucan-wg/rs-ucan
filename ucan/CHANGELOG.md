# Changelog

## [0.4.0](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.3.2...ucan-v0.4.0) (2023-06-27)


### ⚠ BREAKING CHANGES

* Update capabilites in line with UCAN 0.9/0.10 specs ([#105](https://github.com/ucan-wg/rs-ucan/issues/105))
* Update `fct`/`ucv` layout for 0.10.0 spec ([#108](https://github.com/ucan-wg/rs-ucan/issues/108))
* Support generic hashers in `UcanBuilder` and `ProofChain`. ([#89](https://github.com/ucan-wg/rs-ucan/issues/89))

### Features

* Allow nullable expiry, per 0.9.0 spec. Fixes [#23](https://github.com/ucan-wg/rs-ucan/issues/23) ([#95](https://github.com/ucan-wg/rs-ucan/issues/95)) ([12d4756](https://github.com/ucan-wg/rs-ucan/commit/12d475606da940b64654f17807adf592551982d0))
* Support generic hashers in `UcanBuilder` and `ProofChain`. ([#89](https://github.com/ucan-wg/rs-ucan/issues/89)) ([e057f87](https://github.com/ucan-wg/rs-ucan/commit/e057f87c7b278d18e77b1d3d213656d18b1a2fee))
* Update `fct`/`ucv` layout for 0.10.0 spec ([#108](https://github.com/ucan-wg/rs-ucan/issues/108)) ([ae19741](https://github.com/ucan-wg/rs-ucan/commit/ae197415048da201f7d75bf08cdb010b4f657895))
* Update capabilites in line with UCAN 0.9/0.10 specs ([#105](https://github.com/ucan-wg/rs-ucan/issues/105)) ([0bdf98f](https://github.com/ucan-wg/rs-ucan/commit/0bdf98f9043e753026711fb19449ab0bc6d87fc7))

## [0.3.2](https://github.com/ucan-wg/rs-ucan/compare/ucan-v0.3.1...ucan-v0.3.2) (2023-05-25)


### Features

* `fct` and `prf` are now optional fields. Fixes [#98](https://github.com/ucan-wg/rs-ucan/issues/98) ([#99](https://github.com/ucan-wg/rs-ucan/issues/99)) ([6802b5c](https://github.com/ucan-wg/rs-ucan/commit/6802b5c85ce2b16680baa86342e6154896712041))

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
