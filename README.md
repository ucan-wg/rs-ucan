<div align="center">
  <a href="https://github.com/ucan-wg/rs-ucan" target="_blank">
    <img src="https://raw.githubusercontent.com/ucan-wg/rs-ucan/main/assets/a_logo.png" alt="rs-ucan Logo" width="100"></img>
  </a>

  <h1 align="center">rs-ucan</h1>

  <p>
    <a href="https://crates.io/crates/rs-ucan">
      <img src="https://img.shields.io/crates/v/rs-ucan?label=crates" alt="Crate">
    </a>
    <a href="https://codecov.io/gh/ucan-wg/rs-ucan">
      <img src="https://codecov.io/gh/ucan-wg/rs-ucan/branch/main/graph/badge.svg?token=SOMETOKEN" alt="Code Coverage"/>
    </a>
    <a href="https://github.com/ucan-wg/rs-ucan/actions?query=">
      <img src="https://github.com/ucan-wg/rs-ucan/actions/workflows/tests_and_checks.yml/badge.svg" alt="Build Status">
    </a>
    <a href="https://github.com/ucan-wg/rs-ucan/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/rs-ucan">
      <img src="https://img.shields.io/static/v1?label=Docs&message=docs.rs&color=blue" alt="Docs">
    </a>
    <a href="https://discord.gg/4UdeQhw7fv">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

<div align="center"><sub>:warning: Work in progress :warning:</sub></div>

## Outline

- [Usage](#usage)
- [Testing the Project](#testing-the-project)
- [Benchmarking the Project](#benchmarking-the-project)
- [Contributing](#contributing)
- [Getting Help](#getting-help)
- [External Resources](#external-resources)
- [License](#license)

## Usage

Add the following to the `[dependencies]` section of your `Cargo.toml` file:

```toml
rs-ucan = "1.0.0-rc.1"
```

## Testing the Project

- Run tests

  ```console
  cargo test
  ```

## Benchmarking the Project

For benchmarking and measuring performance, this project leverages
[criterion][criterion] and a `test_utils` feature flag
for integrating [proptest][proptest] within the the suite for working with
[strategies][strategies] and sampling from randomly generated values.

- Run benchmarks

  ```console
  cargo bench --features test_utils
  ```

## Contributing

:balloon: We're thankful for any feedback and help in improving our project!
We have a [contributing guide](./CONTRIBUTING.md) to help you get involved. We
also adhere to our [Code of Conduct](./CODE_OF_CONDUCT.md).

### Nix

This repository contains a [Nix flake][nix-flake] that initiates both the Rust
toolchain set in [rust-toolchain.toml](./rust-toolchain.toml) and a
[pre-commit hook](#pre-commit-hook). It also installs helpful cargo binaries for
development. Please install [nix][nix] and [direnv][direnv] to get started.

Run `nix develop` or `direnv allow` to load the `devShell` flake output,
according to your preference.

### Formatting

For formatting Rust in particular, we automatically format on `nightly`, as it
uses specific nightly features we recommend by default.

### Pre-commit Hook

This project recommends using [pre-commit][pre-commit] for running pre-commit
hooks. Please run this before every commit and/or push.

- If you are doing interim commits locally, and for some reason if you _don't_
  want pre-commit hooks to fire, you can run
  `git commit -a -m "Your message here" --no-verify`.

### Recommended Development Flow

- We recommend leveraging [cargo-watch][cargo-watch],
  [cargo-expand][cargo-expand] and [irust][irust] for Rust development.
- We recommend using [cargo-udeps][cargo-udeps] for removing unused dependencies
  before commits and pull-requests.

### Conventional Commits

This project *lightly* follows the [Conventional Commits
convention][commit-spec-site] to help explain
commit history and tie in with our release process. The full specification
can be found [here][commit-spec]. We recommend prefixing your commits with
a type of `fix`, `feat`, `docs`, `ci`, `refactor`, etc..., structured like so:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Getting Help

For usage questions, usecases, or issues reach out to us in our [Discord channel](https://discord.gg/4UdeQhw7fv).

We would be happy to try to answer your question or try opening a new issue on Github.

## External Resources

These are references to specifications, talks and presentations, etc.

## License

This project is licensed under the [Apache License 2.0](./LICENSE), or
[http://www.apache.org/licenses/LICENSE-2.0][apache].


[apache]: https://www.apache.org/licenses/LICENSE-2.0
[cargo-expand]: https://github.com/dtolnay/cargo-expand
[cargo-udeps]: https://github.com/est31/cargo-udeps
[cargo-watch]: https://github.com/watchexec/cargo-watch
[commit-spec]: https://www.conventionalcommits.org/en/v1.0.0/#specification
[commit-spec-site]: https://www.conventionalcommits.org/
[criterion]: https://github.com/bheisler/criterion.rs
[direnv]:https://direnv.net/
[irust]: https://github.com/sigmaSd/IRust
[nix]:https://nixos.org/download.html
[nix-flake]: https://nixos.wiki/wiki/Flakes
[pre-commit]: https://pre-commit.com/
[proptest]: https://github.com/proptest-rs/proptest
[strategies]: https://docs.rs/proptest/latest/proptest/strategy/trait.Strategy.html
