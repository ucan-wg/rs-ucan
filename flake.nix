{
  description = "rs-ucan";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    nixos-unstable.url = "nixpkgs/nixos-unstable-small";

    flake-utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    devshell,
    flake-utils,
    nixos-unstable,
    nixpkgs,
    rust-overlay,
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            devshell.overlays.default
            (import rust-overlay)
            (final: prev: {
              rustfmt = prev.rust-bin.nightly.latest.rustfmt;
            })
          ];
        };

        unstable = import nixos-unstable {
          inherit system;
        };

        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "cargo"
            "clippy"
            "llvm-tools-preview"
            "rust-src"
            "rust-std"
            "rustfmt"
          ];

          targets = [
            "aarch64-apple-darwin"
            "x86_64-apple-darwin"

            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-musl"

            "wasm32-unknown-unknown"
            "wasm32-wasi"
          ];
        };

        format-pkgs = with pkgs; [
          nixpkgs-fmt
          alejandra
          taplo
        ];

        darwin-installs = with pkgs.darwin.apple_sdk.frameworks; [
          Security
          CoreFoundation
          Foundation
        ];

        cargo-installs = with pkgs; [
          cargo-criterion
          cargo-deny
          cargo-expand
          cargo-nextest
          cargo-outdated
          cargo-sort
          cargo-udeps
          cargo-watch
          llvmPackages.bintools
          twiggy
          unstable.cargo-component
          wasm-bindgen-cli
          wasm-tools
        ];
      in rec {
        devShells.default = pkgs.devshell.mkShell {
          name = "rs-ucan";

          imports = [./pre-commit.nix];

          packages = with pkgs;
            [
              # NOTE: The ordering of these two items is important. For nightly rustfmt to be used
              # instead of the rustfmt provided by `rust-toolchain`, it must appear first in the list.
              # This is because native build inputs are added to $PATH in the order they're listed here.
              #
              # nightly-rustfmt
              rust-toolchain

              direnv
              self.packages.${system}.irust

              chromedriver
              nodePackages.pnpm
              protobuf
              unstable.nodejs_20
              unstable.wasmtime
            ]
            ++ format-pkgs
            ++ cargo-installs
            ++ lib.optionals stdenv.isDarwin darwin-installs;

          env = [
            {
              name = "RUSTC_WRAPPER";
              value = "${pkgs.sccache}/bin/sccache";
            }
          ];

          commands = [
            # Release
            {
              name = "release";
              help = "[DEFAULT] Release (optimized build) for current host target";
              category = "release";
              command = "release:host";
            }
            {
              name = "release:host";
              help = "Release for current host target";
              category = "release";
              command = "${pkgs.cargo}/bin/cargo build --release";
            }
            {
              name = "release:wasm";
              help = "Release for current host target";
              category = "release";
              command = "${pkgs.cargo}/bin/cargo build --release --target=wasm32-unknown-unknown";
            }
            # Build
            {
              name = "build";
              help = "[DEFAULT] Build for current host target";
              category = "build";
              command = "build:host";
            }
            {
              name = "build:host";
              help = "Build for current host target";
              category = "build";
              command = "${pkgs.cargo}/bin/cargo build";
            }
            {
              name = "build:wasm";
              help = "Build for wasm32-unknown-unknown";
              category = "build";
              command = "${pkgs.cargo}/bin/cargo build --target=wasm32-unknown-unknown";
            }
            {
              name = "build:wasi";
              help = "Build for WASI";
              category = "build";
              command = "${pkgs.cargo}/bin/cargo build --target wasm32-wasi";
            }
            # Bench
            {
              name = "bench:host";
              help = "Run host Criterion benchmarks";
              category = "dev";
              command = "${pkgs.cargo}/bin/cargo criterion";
            }
            {
              name = "bench:host:open";
              help = "Open host Criterion benchmarks in browser";
              category = "dev";
              command = "${pkgs.xdg-utils}/bin/xdg-open ./target/criterion/report/index.html";
            }
            # Lint
            {
              name = "lint";
              help = "Run Clippy";
              category = "dev";
              command = "${pkgs.cargo}/bin/cargo clippy";
            }
            {
              name = "lint:pedantic";
              help = "Run Clippy pedantically";
              category = "dev";
              command = "${pkgs.cargo}/bin/cargo clippy -- -W clippy::pedantic";
            }
            {
              name = "lint:fix";
              help = "Apply non-pendantic Clippy suggestions";
              category = "dev";
              command = "${pkgs.cargo}/bin/cargo clippy --fix";
            }
            # Watch
            {
              name = "watch:build:host";
              help = "Rebuild host target on save";
              category = "watch";
              command = "${pkgs.cargo}/bin/cargo watch --clear";
            }
            {
              name = "watch:build:wasm";
              help = "Rebuild host target on save";
              category = "watch";
              command = "${pkgs.cargo}/bin/cargo watch --clear --features=serde -- cargo build --target=wasm32-unknown-unknown";
            }
            {
              name = "watch:lint";
              help = "Lint on save";
              category = "watch";
              command = "${pkgs.cargo}/bin/cargo watch --clear --exec clippy";
            }
            {
              name = "watch:lint:pedantic";
              help = "Pedantic lint on save";
              category = "watch";
              command = "${pkgs.cargo}/bin/cargo watch --clear --exec 'clippy -- -W clippy::pedantic'";
            }
            {
              name = "watch:test:host";
              help = "Run all tests on save";
              category = "watch";
              command = "${pkgs.cargo}/bin/cargo watch --clear --exec test";
            }
            {
              name = "watch:test:docs:host";
              help = "Run all tests on save";
              category = "watch";
              command = "${pkgs.cargo}/bin/cargo watch --clear --exec test";
            }
            # Test
            {
              name = "test:all";
              help = "Run Cargo tests";
              category = "test";
              command = "test:host && test:docs && test:wasm";
            }
            {
              name = "test:host";
              help = "Run Cargo tests for host target";
              category = "test";
              command = "${pkgs.cargo}/bin/cargo test";
            }
            {
              name = "test:wasm";
              help = "Run wasm-pack tests on all targets";
              category = "test";
              command = "test:wasm:node && test:wasm:chrome";
            }
            {
              name = "test:wasm:nodejs";
              help = "Run wasm-pack tests in Node.js";
              category = "test";
              command = "${pkgs.wasm-pack}/bin/wasm-pack test --node";
            }
            {
              name = "test:wasm:chrome";
              help = "Run wasm-pack tests in headless Chrome";
              category = "test";
              command = "${pkgs.wasm-pack}/bin/wasm-pack test --headless --chrome";
            }
            {
              name = "test:docs";
              help = "Run Cargo doctests";
              category = "test";
              command = "${pkgs.cargo}/bin/cargo test --doc";
            }
            # Docs
            {
              name = "docs";
              help = "[DEFAULT]: Open refreshed docs";
              category = "dev";
              command = "docs:open";
            }
            {
              name = "docs:build";
              help = "Refresh the docs";
              category = "dev";
              command = "${pkgs.cargo}/bin/cargo doc";
            }
            {
              name = "docs:open";
              help = "Open refreshed docs";
              category = "dev";
              command = "${pkgs.cargo}/bin/cargo doc --open";
            }
          ];
        };

        packages.irust = pkgs.rustPlatform.buildRustPackage rec {
          pname = "irust";
          version = "1.71.19";
          src = pkgs.fetchFromGitHub {
            owner = "sigmaSd";
            repo = "IRust";
            rev = "irust@${version}";
            sha256 = "sha256-R3EAovCI5xDCQ5R69nMeE6v0cGVcY00O3kV8qHf0akc=";
          };

          doCheck = false;
          cargoSha256 = "sha256-2aVCNz/Lw7364B5dgGaloVPcQHm2E+b/BOxF6Qlc8Hs=";
        };

        formatter = pkgs.alejandra;

        # NOTE: blst requires --target=wasm32 support in Clang, which MacOS system clang doesn't provide
        stdenv = pkgs.clangStdenv;
      }
    );
}
