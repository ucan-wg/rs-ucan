{
  description = "ucan";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.11";
    nixos-unstable.url = "nixpkgs/nixos-unstable-small";

    command-utils.url = "git+https://codeberg.org/expede/nix-command-utils";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    flake-utils,
    nixos-unstable,
    nixpkgs,
    rust-overlay,
    command-utils,
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          (import rust-overlay)
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        unstable = import nixos-unstable {
          inherit system overlays;
          config.allowUnfree = true;
        };

        rustVersion = "1.90.0";

        rust-toolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "cargo"
            "clippy"
            "llvm-tools-preview"
            "rust-src"
            "rust-std"
          ];

          targets = [
            "aarch64-apple-darwin"
            "x86_64-apple-darwin"

            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-musl"

            "wasm32-unknown-unknown"
            "thumbv6m-none-eabi"
          ];
        };

        # Nightly rustfmt for unstable formatting options (imports_granularity, etc.)
        # We need a combined nightly toolchain (rustc + rustfmt) because rustfmt
        # links against librustc_driver, which lives in the rustc component.
        # On macOS, symlinks break @rpath resolution, so we wrap the binary
        # with DYLD_LIBRARY_PATH pointing to the combined toolchain's lib/.
        nightly-rustfmt-unwrapped = pkgs.rust-bin.nightly.latest.minimal.override {
          extensions = ["rustfmt"];
        };

        nightly-rustfmt = pkgs.writeShellScriptBin "rustfmt" ''
          export DYLD_LIBRARY_PATH="${nightly-rustfmt-unwrapped}/lib''${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
          export LD_LIBRARY_PATH="${nightly-rustfmt-unwrapped}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          exec "${nightly-rustfmt-unwrapped}/bin/rustfmt" "$@"
        '';

        format-pkgs = with pkgs; [
          nixpkgs-fmt
          alejandra
          taplo
        ];

        cargo-installs = with pkgs; [
          cargo-component
          cargo-criterion
          cargo-deny
          cargo-expand
          cargo-nextest
          cargo-outdated
          cargo-sort
          cargo-udeps
          cargo-watch
          twiggy
          wasm-bindgen-cli
          wasm-tools
        ];

        # Built-in command modules from nix-command-utils
        rust = command-utils.rust.${system};
        wasm = command-utils.wasm.${system};
        cmd = command-utils.cmd.${system};

        command_menu = command-utils.commands.${system} [
          # Rust commands
          (rust.build {cargo = pkgs.cargo;})
          (rust.test {
            cargo = pkgs.cargo;
            cargo-watch = pkgs.cargo-watch;
          })
          (rust.lint {cargo = pkgs.cargo;})
          (rust.fmt {cargo = pkgs.cargo;})
          (rust.doc {cargo = pkgs.cargo;})
          (rust.bench {
            cargo = pkgs.cargo;
            cargo-criterion = pkgs.cargo-criterion;
            xdg-open = pkgs.xdg-utils;
          })
          (rust.watch {cargo-watch = pkgs.cargo-watch;})

          # Wasm commands
          (wasm.build {wasm-pack = pkgs.wasm-pack;})
          (wasm.release {
            wasm-pack = pkgs.wasm-pack;
            gzip = pkgs.gzip;
          })
          (wasm.test {
            wasm-pack = pkgs.wasm-pack;
            features = "browser_test";
          })
          (wasm.doc {
            cargo = pkgs.cargo;
            xdg-open = pkgs.xdg-utils;
          })
        ];
      in rec {
        devShells.default = pkgs.mkShell {
          name = "ucan";

          nativeBuildInputs =
            [
              command_menu
              rust-toolchain
              nightly-rustfmt

              pkgs.rust-analyzer
              pkgs.wasm-pack
            ]
            ++ format-pkgs
            ++ cargo-installs;

          shellHook = ''
            unset SOURCE_DATE_EPOCH
            export RUSTFMT="${nightly-rustfmt}/bin/rustfmt"
            menu
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}
