<div align="center">
  <a href="https://github.com/ucan-wg/rs-ucan" target="_blank">
    <img src="assets/logo.png" alt="rs-ucan Logo" height="200"></img>
  </a>

  <h1 align="center">rs-ucan</h1>

  <p>
    <a href="https://crates.io/crates/ucanskip_ratchet">
      <img src="https://img.shields.io/crates/v/ucan.svg?label=crates" alt="Crate Information">
    </a>
    <a href="https://codecov.io/gh/ucan-wg/rs-ucan">
      <img src="https://codecov.io/gh/ucan-wg/rs-ucan/branch/main/graph/badge.svg?token=UZ53MKNKJC" alt="Code Coverage"/>
    </a>
    <a href="https://github.com/ucan-wg/rs-ucan/actions?query=">
      <img src="https://github.com/ucan-wg/rs-ucan/actions/workflows/run_test_suite.yaml/badge.svg" alt="Build Status">
    </a>
    <a href="https://github.com/ucan-wg/rs-ucan/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/ucan">
      <img src="https://img.shields.io/static/v1?label=Docs&message=docs.rs&color=blue" alt="Docs">
    </a>
    <a href="https://discord.gg/JSyFG6XgVM">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

This is a Rust library to help the next generation of web applications make use
of UCANs in their authorization flows. To learn more about UCANs and how you
might use them in your application, visit [https://ucan.xyz][ucan website] or
read the [spec][spec].

## Outline

- [Crates](#crates)
- [Building the Project](#building-the-project)
- [Testing the Project](#testing-the-project)
- [Contributing](#contributing)
- [Getting Help](#getting-help)
- [License](#license)

## Crates

- [ucan](https://github.com/ucan-wg/rs-ucan/tree/main/ucan)
- [ucan-key-support](https://github.com/ucan-wg/rs-ucan/tree/main/ucan-key-support)

## Building the Project

- Clone the repository.

  ```bash
  git clone https://github.com/ucan-wg/rs-ucan.git
  ```

- Change directory

  ```bash
  cd rs-ucan
  ```

- Build the project

  ```bash
  cargo build
  ```

## Testing the Project

- Run tests

  ```bash
  cargo test
  ```

## Contributing

### Pre-commit Hook

This library recommends using [pre-commit][pre-commit] for running pre-commit
hooks. Please run this before every commit and/or push.

- Once installed, Run `pre-commit install` to setup the pre-commit hooks
  locally.  This will reduce failed CI builds.
- If you are doing interim commits locally, and for some reason if you _don't_
  want pre-commit hooks to fire, you can run
  `git commit -a -m "Your message here" --no-verify`.

## Getting Help

For usage questions, usecases, or issues reach out to us in our `rs-ucan`
[Discord channel](https://discord.gg/3EHEQ6M8BC).

We would be happy to try to answer your question or try opening a new issue on
Github.

## License

This project is licensed under the [Apache License 2.0](https://github.com/ucan-wg/rs-ucan/blob/main/LICENSE).

[pre-commit]: https://pre-commit.com/
[spec]: https://github.com/ucan-wg/spec
[ucan website]: https://ucan.xyz
