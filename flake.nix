{
  description = "cargo-cgp — a cargo subcommand that makes Context-Generic Programming compiler errors readable";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # oxalica's rust-overlay is the standard way to get a binary-distributed Rust
    # toolchain from a `rust-toolchain.toml`, including the unstable `rustc-dev`
    # component the driver links. It follows our nixpkgs so both resolve one set of
    # store paths.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # The exact nightly the driver is welded to, read straight from the workspace
        # `rust-toolchain.toml`. That file is the single source of truth: it names the
        # dated `channel` and the `rustc-dev` + `llvm-tools` components the
        # `rustc_private` driver needs, and the crates' `build.rs` scripts read the same
        # file, so the Nix toolchain can never drift from the pinned one. `rustc-dev`
        # carries `librustc_driver`, which the driver both links at build time and loads
        # at run time — the whole reason a plain nightly is not enough.
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Build both binaries with that one nightly, so the driver embeds the same
        # compiler whose sysroot and `librustc_driver` it will be handed at run time.
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        # The OS variable that lists directories searched for dynamic libraries, matching
        # the driver's own `dylib` module: macOS uses the DYLD fallback path, everywhere
        # else `LD_LIBRARY_PATH`. (Windows is not a Nix target, so it is not handled here.)
        dylibPathVar = if pkgs.stdenv.isDarwin then "DYLD_FALLBACK_LIBRARY_PATH" else "LD_LIBRARY_PATH";

        inherit (pkgs) lib;

        # Only the files the build actually reads, so the derivation's input hash — and
        # thus a rebuild — turns solely on the crate sources, the manifests, the lockfile,
        # and the pinned-toolchain file the `build.rs` scripts consume. Editing `docs/`, a
        # `tests/ui` fixture, `README.md`, or `AGENTS.md` leaves the source unchanged and
        # the built binaries cached. `crates/` is taken whole (it is small and all of it is
        # build input, including each workspace member's manifest that cargo must parse);
        # the churny, build-irrelevant directories sit at the repository root and are simply
        # not listed.
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./rust-toolchain.toml
            ./crates
          ];
        };

        cargo-cgp = rustPlatform.buildRustPackage {
          pname = "cargo-cgp";
          version = "0.1.0-alpha";
          inherit src;
          cargoLock.lockFile = ./Cargo.lock;

          # Build only the two shipped binaries. `cargo-cgp-error-processing` builds as a
          # dependency of the driver; `cargo-cgp-ui-tests` (the snapshot harness) is left
          # out, since it drives a live toolchain and expects a sibling `cgp` checkout.
          cargoBuildFlags = [ "-p" "cargo-cgp" "-p" "cargo-cgp-driver" ];

          # For the same reason the UI suite is out of scope, the package build does not
          # run the tests: they need the pinned toolchain at run time and the `../cgp`
          # checkout that a sandboxed build does not have.
          doCheck = false;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          # Two wrappers reconstruct, under Nix, what rustup provides on a managed machine.
          postInstall = ''
            # The driver links `librustc_driver` from the toolchain rather than a system
            # library, so pin that directory on its runtime search path. The front-end
            # already prepends the discovered sysroot's `lib` when it spawns the driver;
            # this makes the driver load that same library even when run directly, so it
            # is self-contained rather than dependent on the caller's environment.
            wrapProgram $out/bin/cargo-cgp-driver \
              --prefix ${dylibPathVar} : "${rustToolchain}/lib"

            # There is no rustup under Nix to install or force a toolchain, so tell the
            # front-end not to manage one (`CARGO_CGP_NO_MANAGE`) and hand it the pinned
            # nightly instead: `RUSTC` and the front-end PATH both point at this
            # toolchain, so the wrapped `cargo check` compiles under the same compiler the
            # driver embeds and `rustc --print sysroot` returns that toolchain's sysroot —
            # whose `lib` holds the matching `librustc_driver`. The driver stays reachable
            # as the front-end's sibling in this same `bin/` directory.
            #
            # The dylib prefix is the same protection the driver gets above, and the
            # front-end needs it for the toolchain binaries *it* spawns. Invoked as `cargo
            # cgp check`, the entry point is rustup's `cargo` shim, which exports
            # `${dylibPathVar}=<the project's active toolchain>/lib` to its children — so a
            # foreign toolchain's library directory is searched ahead of this one's own
            # RUNPATH. That is harmless while the two Rust versions differ, but a rustc
            # shared library is named for its Rust version rather than by content hash
            # (`libLLVM.so.<n>-rust-<version>`), so a project pinning the *same* version as
            # the toolchain below collides on the SONAME: the Nix `rustc` loads rustup's
            # copy, which then wants a system library the Nix loader cannot resolve, and
            # dies with a bare exit 127. Prefixing this toolchain's `lib` makes its own
            # libraries win that lookup while leaving the caller's entries in place.
            wrapProgram $out/bin/cargo-cgp \
              --set CARGO_CGP_NO_MANAGE 1 \
              --set RUSTC "${rustToolchain}/bin/rustc" \
              --prefix PATH : "${rustToolchain}/bin" \
              --prefix ${dylibPathVar} : "${rustToolchain}/lib"
          '';

          meta = {
            description = "A cargo subcommand that makes Context-Generic Programming compiler errors readable";
            homepage = "https://github.com/contextgeneric/cargo-cgp";
            license = pkgs.lib.licenses.mit;
            mainProgram = "cargo-cgp";
          };
        };
      in
      {
        packages = {
          default = cargo-cgp;
          cargo-cgp = cargo-cgp;
        };

        # `nix run .# -- check ...` runs the front-end, which then wraps `cargo check`.
        apps.default = flake-utils.lib.mkApp {
          drv = cargo-cgp;
          name = "cargo-cgp";
        };

        # A shell for developing cargo-cgp itself: the pinned nightly (with the same
        # `rustc-dev` linkage the driver needs) on PATH, plus rust-analyzer.
        # `CARGO_CGP_NO_MANAGE` matches how the UI suite runs the freshly built binaries
        # against the ambient toolchain.
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.rust-analyzer
          ];
          env.CARGO_CGP_NO_MANAGE = "1";
        };
      }
    );
}
