# Installation

`cargo-cgp` is two binaries plus an exact pinned nightly compiler, and installing it means getting
all three in place and matched. The `cargo-cgp` front-end is the cargo subcommand you invoke; the
`cargo-cgp-driver` it calls links the compiler's internals and so must be built against the nightly
pinned in the project's `rust-toolchain.toml`. This document covers the ways to install that pair and
how to keep it up to date. It is the usage-level summary for an agent; the design behind it — why the
toolchain is pinned and how the binaries stay in lockstep — is in
[Distribution](../implementation/distribution.md).

If your goal is only to *run* the current tool — the usual case for an agent testing or demonstrating
it — you do not install a release at all: use a local checkout through Nix or from source, as
[Usage](usage.md#running-on-a-project-outside-this-repository) describes. The install paths below are
for provisioning the tool as a durable command; prefer the local build over a published release
whenever a checkout is available, since it reflects the current code.

## Which path to use

There are two supported ways to install, and which one fits depends on how the machine manages Rust.
Use the **cargo path** if rustup is present: you install the small front-end with cargo, then let it
provision the pinned nightly and the driver for you. Use the **Nix path** if Nix is present: a flake
builds both binaries against the pinned nightly and needs no rustup at all. A third option, building
**from source**, is for developing the tool itself or running a checkout directly.

The tool supplies its own compiler for the check either way, so your own project is never asked to
adopt the pinned nightly — it keeps whatever toolchain it already uses for its ordinary builds.

## Current availability

Only the Nix and from-source paths work today. The current two-binary design is not yet published to
crates.io: `cargo-cgp-driver` is unpublished, and the `cargo-cgp` on crates.io is an old `0.0.1`
prototype unrelated to the current tool. Until both crates are published together at a matching
version — one of the [open release items](../implementation/distribution.md#open-decisions-and-risks-to-resolve) —
`cargo install cargo-cgp` and `cargo cgp setup` will not produce a working install. Install from the
Nix flake or from a source checkout in the meantime. The cargo path is documented below because it is
the intended primary distribution and the machinery for it is already in the tool.

## Installing with Nix

The flake at the repository root builds both binaries against the pinned nightly and wraps them so
they run without rustup, which makes it the simplest way to install the current tool. It provisions
the compiler as a build-time fact, so a machine that has only Nix — no rustup, no nightly — can run
the tool, and the project you check needs no toolchain of its own. The details of how the flake does
this are in [Installing with Nix](../implementation/distribution.md#installing-with-nix).

To install the tool onto your `PATH` so `cargo cgp check` works like any other cargo subcommand:

```sh
nix profile install github:contextgeneric/cargo-cgp
```

This puts both `cargo-cgp` and `cargo-cgp-driver` in your Nix profile, side by side as the front-end
requires. To run the tool once without installing it — for example in CI or to try it on a project —
run the flake's default app from the project directory instead:

```sh
cd /path/to/your/project     # a cargo package or workspace that uses `cgp`
nix run github:contextgeneric/cargo-cgp -- check
```

To pin the tool in another project's own flake, add it as an input and take its `packages.default`:

```nix
inputs.cargo-cgp.url = "github:contextgeneric/cargo-cgp";
# then, in a devShell or CI derivation:
#   packages = [ cargo-cgp.packages.${system}.default ];
```

## Installing with cargo

The cargo path installs the small front-end first and then provisions everything heavyweight in a
second step, so the first command is one you can type from memory with no nightly date in it. (See
[Current availability](#current-availability): this path needs the crates published, which has not
happened yet.)

```sh
cargo install cargo-cgp      # installs the front-end; builds on any toolchain
cargo cgp setup              # provisions the pinned nightly + builds the matching driver
```

The two commands divide the work cleanly. `cargo install cargo-cgp` builds the front-end, which is a
plain binary with no compiler linkage, so it installs under whatever toolchain you already have.
`cargo cgp setup` then does the heavyweight, stateful work: it installs the pinned nightly with its
`rustc-dev` and `llvm-tools` components through rustup, builds `cargo-cgp-driver` against that exact
nightly at the front-end's own version, and places the driver next to the front-end in
`~/.cargo/bin`. Because `setup` reads the pinned toolchain from the front-end's baked-in constant,
you never type a nightly date. `setup` requires rustup to manage the toolchain.

You run `cargo cgp setup` once after installing, and again only when told to: `cargo cgp check` runs
a fast, read-only preflight before each check and, if it finds the driver missing, the toolchain
absent, or the two out of step, stops with an error naming `cargo cgp setup` as the fix rather than
doing any slow or stateful work itself.

## Installing from source

To run the current tool from a checkout — the practical path today alongside Nix — clone the
repository and build both binaries, which the pinned `rust-toolchain.toml` compiles under the correct
nightly automatically:

```sh
git clone https://github.com/contextgeneric/cargo-cgp
cd cargo-cgp
cargo build
```

Then run the freshly built front-end against another project, pointing it at the built driver and
skipping the management preflight, so it uses the pinned toolchain the checkout already selects:

```sh
CARGO_CGP_NO_MANAGE=1 CARGO_CGP_DRIVER=$PWD/target/debug/cargo-cgp-driver \
  /path/to/cargo-cgp/target/debug/cargo-cgp check
```

For most from-source use the [Nix flake](#installing-with-nix) is easier, since it wires these
environment overrides for you.

## Updating

How you update matches how you installed. On the cargo path, `cargo cgp update` upgrades the tool:

```sh
cargo cgp update
```

It reads the crates.io index for the front-end, picks the highest published version **in your current
release channel** — a stable install never jumps to a pre-release, and a pre-release install stays on
pre-releases — and does nothing if you already have the newest. When there is a newer version it
reinstalls the front-end and re-runs the new `setup` to bring the driver and toolchain up to match.
On Windows the running binary is locked and cannot replace itself, so `update` prints the two
commands (`cargo install cargo-cgp` then `cargo cgp setup`) to run by hand from a shell where the
tool is not running.

On the Nix path there is no `cargo cgp update`; you upgrade by refreshing the flake instead. For an
installed profile, upgrade it (`nix profile upgrade`); for the tool pinned as an input in another
flake, run `nix flake update cargo-cgp` there to pull the newer release. Each nightly bump ships as a
new release of the flake, so updating the input is all that is needed.

## Further reading

- [Distribution](../implementation/distribution.md) — the design behind installation: why the
  toolchain is pinned, how `setup`/`update`/the preflight work, how the two crates stay in lockstep,
  and the Nix packaging.
- [Usage](usage.md) — how to run the tool once it is installed.
