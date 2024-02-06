{
  description = "ucan";

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
        overlays = [
          devshell.overlays.default
          (import rust-overlay)
        ];

        pkgs = import nixpkgs {inherit system overlays;};
        unstable = import nixos-unstable {inherit system overlays;};

        rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
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

        cargo = "${pkgs.cargo}/bin/cargo";
        node = "${unstable.nodejs_20}/bin/node";
        wasm-pack = "${pkgs.wasm-pack}/bin/wasm-pack";
        wasm-opt = "${pkgs.binaryen}/bin/wasm-opt";
      in rec {
        devShells.default = pkgs.devshell.mkShell {
          name = "ucan";

          imports = [./pre-commit.nix];

          packages = with pkgs;
            [
              direnv
              rust-toolchain
              self.packages.${system}.irust
              (pkgs.hiPrio pkgs.rust-bin.nightly.latest.rustfmt)

              chromedriver
              protobuf
              unstable.nodejs_20
              unstable.nodePackages.pnpm
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
              command = "${cargo} build --release";
            }
            {
              name = "release:wasm:web";
              help = "Release for current host target";
              category = "release";
              command = "${wasm-pack} build --release --target=web";
            }
            {
              name = "release:wasm:nodejs";
              help = "Release for current host target";
              category = "release";
              command = "${wasm-pack} build --release --target=nodejs";
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
              command = "${cargo} build";
            }
            {
              name = "build:wasm:web";
              help = "Build for wasm32-unknown-unknown with web bindings";
              category = "build";
              command = "${wasm-pack} build --dev --target=web";
            }
            {
              name = "build:wasm:nodejs";
              help = "Build for wasm32-unknown-unknown with Node.js bindgings";
              category = "build";
              command = "${wasm-pack} build --dev --target=nodejs";
            }
            {
              name = "build:node";
              help = "Build JS-wrapped Wasm library";
              category = "build";
              command = "${pkgs.nodePackages.pnpm}/bin/pnpm install && ${node} run build";
            }
            {
              name = "build:wasi";
              help = "Build for WASI";
              category = "build";
              command = "${cargo} build --target wasm32-wasi";
            }
            # Bench
            {
              name = "bench";
              help = "Run benchmarks, including test utils";
              category = "dev";
              command = "${cargo} bench --features test_utils";
            }
            # FIXME align with `bench`?
            {
              name = "bench:host";
              help = "Run host Criterion benchmarks";
              category = "dev";
              command = "${cargo} criterion";
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
              command = "${cargo} clippy";
            }
            {
              name = "lint:pedantic";
              help = "Run Clippy pedantically";
              category = "dev";
              command = "${cargo} clippy -- -W clippy::pedantic";
            }
            {
              name = "lint:fix";
              help = "Apply non-pendantic Clippy suggestions";
              category = "dev";
              command = "${cargo} clippy --fix";
            }
            # Watch
            {
              name = "watch:build:host";
              help = "Rebuild host target on save";
              category = "watch";
              command = "${cargo} watch --clear";
            }
            {
              name = "watch:build:wasm";
              help = "Rebuild host target on save";
              category = "watch";
              command = "${cargo} watch --clear --features=serde -- cargo build --target=wasm32-unknown-unknown";
            }
            {
              name = "watch:lint";
              help = "Lint on save";
              category = "watch";
              command = "${cargo} watch --clear --exec clippy";
            }
            {
              name = "watch:lint:pedantic";
              help = "Pedantic lint on save";
              category = "watch";
              command = "${cargo} watch --clear --exec 'clippy -- -W clippy::pedantic'";
            }
            {
              name = "watch:test:host";
              help = "Run all tests on save";
              category = "watch";
              command = "${cargo} watch --clear --exec test";
            }
            {
              name = "watch:test:docs:host";
              help = "Run all tests on save";
              category = "watch";
              command = "${cargo} watch --clear --exec test";
            }
            {
              name = "watch:test:wasm";
              help = "Run all tests on save";
              category = "watch";
              command = "${cargo} watch --clear --exec 'test --target=wasm32-unknown-unknown'";
            }
            {
              name = "watch:test:docs:wasm";
              help = "Run all tests on save";
              category = "watch";
              command = "${cargo} watch --clear --exec 'test --target=wasm32-unknown-unknown'";
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
              command = "${cargo} test";
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
              command = "${wasm-pack} test --node";
            }
            {
              name = "test:wasm:chrome";
              help = "Run wasm-pack tests in headless Chrome";
              category = "test";
              command = "${wasm-pack} test --headless --chrome";
            }
            {
              name = "test:docs";
              help = "Run Cargo doctests";
              category = "test";
              command = "${cargo} test --doc --features=mermaid_docs";
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
              command = "${cargo} doc --features=mermaid_docs";
            }
            {
              name = "docs:wasm:build";
              help = "Refresh the docs with the wasm32-unknown-unknown target";
              category = "dev";
              command = "${cargo} doc --features=mermaid_docs --target=wasm32-unknown-unknown";
            }
            {
              name = "docs:open";
              help = "Open refreshed docs";
              category = "dev";
              command = "${cargo} doc --features=mermaid_docs --open";
            }
            {
              name = "docs:wasm:open";
              help = "Open refreshed docs for wasm32-unknown-unknown";
              category = "dev";
              command = "${cargo} doc --features=mermaid_docs --target=wasm32-unknown-unknown --open";
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
