<div align="center">
  <a href="https://github.com/ucan-wg/ucan" target="_blank">
    <img src="https://raw.githubusercontent.com/ucan-wg/ucan/main/assets/logo.png" alt="ucan Logo" width="100"></img>
  </a>

  <h1 align="center">ucan</h1>

  <p>
    <a href="https://crates.io/crates/ucan">
      <img src="https://img.shields.io/crates/v/ucan?label=crates" alt="Crate">
    </a>
    <a href="https://codecov.io/gh/ucan-wg/ucan">
      <img src="https://codecov.io/gh/ucan-wg/ucan/branch/main/graph/badge.svg?token=SOMETOKEN" alt="Code Coverage"/>
    </a>
    <a href="https://github.com/ucan-wg/ucan/actions?query=">
      <img src="https://github.com/ucan-wg/ucan/actions/workflows/tests_and_checks.yml/badge.svg" alt="Build Status">
    </a>
    <a href="https://github.com/ucan-wg/ucan/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/ucan">
      <img src="https://img.shields.io/static/v1?label=Docs&message=docs.rs&color=blue" alt="Docs">
    </a>
    <a href="https://discord.gg/4UdeQhw7fv">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

<div align="center"><sub>:warning: Work in progress :warning:</sub></div>

## Usage

Add the following to the `[dependencies]` section of your `Cargo.toml` file:

```toml
ucan = "1.0.0-rc.1"
```

## Testing the Project

Run tests

| Nix        | Cargo        |
|------------|--------------|
| `test:all` | `cargo test` |

## Benchmarking the Project

For benchmarking and measuring performance, this project leverages
[Criterion] and a `test_utils` feature flag
for integrating [proptest] within the the suite for working with
[strategies] and sampling from randomly generated values.

## Benchmarks

| Nix     | Cargo                               |
|---------|-------------------------------------|
| `bench` | `cargo bench --features test_utils` |

## Contributing

:balloon: We're thankful for any feedback and help in improving our project!
We have a [contributing guide][CONTRIBUTING] to help you get involved. We
also adhere to our [Code of Conduct].

### Nix

This repository contains a [Nix flake] that initiates both the Rust
toolchain set in [`rust-toolchain.toml`] and a [pre-commit hook]. It also
installs helpful cargo binaries for development.

Please install [Nix] to get started. We also recommend installing [direnv].

Run `nix develop` or `direnv allow` to load the `devShell` flake output,
according to your preference.

The Nix shell also includes several helpful shortcut commands.
You can see a complete list of commands via the `menu` command.

### Formatting

For formatting Rust in particular, we automatically format on `nightly`, as it
uses specific nightly features we recommend by default.

### Pre-commit Hook

This project recommends using [pre-commit] for running pre-commit
hooks. Please run this before every commit and/or push.

- If you are doing interim commits locally, and for some reason if you _don't_
  want pre-commit hooks to fire, you can run
  `git commit -a -m "Your message here" --no-verify`.

### Recommended Development Flow

- We recommend leveraging [cargo-watch][cargo-watch],
  [`cargo-expand`] and [IRust] for Rust development.
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

For usage questions, usecases, or issues reach out to us in the [UCAN Discord].

We would be happy to try to answer your question or try opening a new issue on GitHub.

## External Resources

These are references to specifications, talks and presentations, etc.

## License

This project is [licensed under the Apache License 2.0][LICENSE], or
[http://www.apache.org/licenses/LICENSE-2.0][Apache].

<!-- Internal Links -->

[Benchmarking the Project]: #benchmarking-the-project
[Contributing]: #contributing
[External Resources]: #external-resources
[Getting Help]: #getting-help
[License]: #license
[Testing the Project]: #testing-the-project
[Usage]: #usage
[pre-commit hook]: #pre-commit-hook

<!-- Local Links -->

[CONTRIBUTING]: ./CONTRIBUTING.md
[LICENSE]: ./LICENSE
[Code of Conduct]: ./CODE_OF_CONDUCT.md
[`rust-toolchain.toml`]: ./rust-toolchain.toml

<!-- External Links -->

[Apache]: https://www.apache.org/licenses/LICENSE-2.0
[`cargo-expand`]: https://github.com/dtolnay/cargo-expand
[`cargo-udeps`]: https://github.com/est31/cargo-udeps
[`cargo-watch`]: https://github.com/watchexec/cargo-watch
[commit-spec]: https://www.conventionalcommits.org/en/v1.0.0/#specification
[commit-spec-site]: https://www.conventionalcommits.org/
[Criterion]: https://github.com/bheisler/criterion.rs
[direnv]:https://direnv.net/
[IRust]: https://github.com/sigmaSd/IRust
[Nix]:https://nixos.org/download.html
[Nix flake]: https://nixos.wiki/wiki/Flakes
[pre-commit]: https://pre-commit.com/
[proptest]: https://github.com/proptest-rs/proptest
[strategies]: https://docs.rs/proptest/latest/proptest/strategy/trait.Strategy.html
[UCAN Discord]: https://discord.gg/4UdeQhw7fv
