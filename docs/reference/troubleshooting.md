# Troubleshooting

`cargo-cgp` is two binaries plus an exact pinned nightly compiler, held together only by a
sibling-path lookup and a couple of environment variables, so when it misbehaves the fault can sit at
any of several seams. The front-end may fail to find the driver; the driver may fail to load the
compiler library it links; the driver, the installed toolchain, and the front-end may drift out of
step; or the check may simply be run in the wrong place. This document walks the common failures with
the **exact error each prints**, what it means, and how to fix it, so an agent can match a symptom to
a cause quickly. It builds on the driver-level debugging tools in
[Usage](usage.md#calling-the-driver-directly-debugging).

## The moving parts, and where they can break

Three things must line up for a check to run, and each is a place a failure can originate. The
**front-end** (`cargo-cgp`) is the cargo subcommand you invoke; it locates the **driver**
(`cargo-cgp-driver`) as a sibling in the same directory (or via the `CARGO_CGP_DRIVER` override) and
wires it in as cargo's compiler wrapper. The driver links the compiler's internal `librustc_driver`
dynamically, so that library — belonging to the **pinned nightly** the driver was built against — must
be on the loader's search path when the driver runs, and the sysroot handed to it must belong to that
same nightly. The full contract is in
[Executable structure](../implementation/executable-structure.md#the-environment-contract); the point
for troubleshooting is that a mismatch between *the nightly the driver embeds* and *the toolchain
present at run time* is the single most common root cause, and several different-looking errors trace
back to it.

## Isolating the failure

Two probes narrow almost any failure to the right seam before you read the sections below. First, run
the driver's own version query, which loads the compiler library before printing and so doubles as a
load test:

```sh
cargo-cgp-driver --version
```

If that fails, the problem is the driver or its library path, not the front-end — jump to
[The driver cannot load the compiler library](#the-driver-cannot-load-the-compiler-library). If it
succeeds, run the check verbosely to see exactly how the front-end invokes the driver and where it
stops:

```sh
cargo cgp check -v
```

The `Running` lines — each showing a `cargo-cgp-driver … rustc …` command — are the driver being
called; an error before them is a front-end or preflight problem, and an error from one of them is a
driver or compiler problem. [Usage](usage.md#calling-the-driver-directly-debugging) covers running that printed command
in isolation.

## The command itself fails

The simplest failures never reach the driver. If you invoke an unknown or missing subcommand, the
front-end says so and lists the three it accepts:

```text
cargo-cgp: missing subcommand (expected `cargo-cgp check`, `setup`, or `update`)
cargo-cgp: unknown cargo-cgp subcommand `frobnicate` (expected `check`, `setup`, or `update`)
```

The fix is to use `check`, `setup`, or `update`. Note that `cargo cgp check` reaches the same
entrypoint as a direct `cargo-cgp check` — cargo inserts the `cgp` token, which the front-end strips.

If the check runs outside a cargo package, the error comes from cargo, not `cargo-cgp`, because the
front-end forwards to `cargo check`:

```text
error: could not find `Cargo.toml` in `/some/dir` or any parent directory
```

Run the command from inside the package or workspace you mean to check. And if cargo itself is not on
`PATH`, the front-end reports that while trying to launch the wrapped build:

```text
failed to run `cargo check` (is cargo on PATH?)
```

## The front-end cannot find the driver

The front-end expects the driver beside it, so a driver that is missing, or a `CARGO_CGP_DRIVER`
override pointing at the wrong path, fails at the point the driver would be launched — but the message
differs by mode. In **managed** mode the preflight catches it first and names `cargo cgp setup`:

```text
cargo-cgp: failed to run the cargo-cgp-driver at /path/to/cargo-cgp-driver: No such file or directory (os error 2)

Run `cargo cgp setup`.
```

In **unmanaged** mode (`CARGO_CGP_NO_MANAGE` set — the from-source and Nix paths) there is no
preflight, so the bad path reaches cargo, which reports it as a wrapper it could not execute:

```text
error: could not execute process `/path/to/cargo-cgp-driver …/rustc -vV` (never executed)

Caused by:
  No such file or directory (os error 2)
```

Either way the fix is to make the driver reachable: run `cargo cgp setup` to reinstall it beside the
front-end (managed), point `CARGO_CGP_DRIVER` at the real driver binary (for a from-source build,
`target/debug/cargo-cgp-driver`), or run through the Nix flake, which places both binaries together.
The driver and front-end **must** live in the same directory unless `CARGO_CGP_DRIVER` says otherwise,
because the front-end finds the driver relative to its own executable.

## The driver cannot load the compiler library

This is the most common and most confusing failure, because the driver aborts *before* its own code
runs and the message comes from the operating system's loader:

```text
cargo-cgp-driver: error while loading shared libraries: librustc_driver-c29d28819724b6fa.so: cannot open shared object file: No such file or directory
```

The driver links `librustc_driver-<hash>.so` from the pinned nightly, and the `<hash>` is fixed at
build time to that exact nightly. The error means the loader cannot find *that* library, which has one
of two causes.

The first cause is that **the library path is simply not set** — you ran the driver directly without
the search-path setup the front-end normally provides. A Nix-built driver has the path baked into its
wrapper and does not hit this; a from-source driver run by hand does. Provide the pinned toolchain's
`lib` directory on the loader path (`DYLD_FALLBACK_LIBRARY_PATH` on macOS, `LD_LIBRARY_PATH`
elsewhere):

```sh
SYSROOT=$(rustc --print sysroot)                        # under the pinned toolchain
LD_LIBRARY_PATH=$SYSROOT/lib cargo-cgp-driver --version
```

The second cause is a **toolchain mismatch**: the library path is set, but to a *different* toolchain
than the one the driver was built against, so the exact `librustc_driver-<hash>.so` is absent from it.
This is what happens when an unmanaged check runs under a toolchain that is not the driver's pinned
nightly — the driver embeds, say, `nightly-2026-07-16`, but the ambient toolchain is `stable`, so cargo's
first probe of the wrapper aborts:

```text
error: process didn't exit successfully: `…/cargo-cgp-driver …/stable/…/rustc -vV` (exit status: 127)
--- stderr
…/cargo-cgp-driver: error while loading shared libraries: librustc_driver-c29d28819724b6fa.so: cannot open shared object file
```

The fix is to run under the pinned nightly. Managed mode forces this for you; in unmanaged mode make
the pinned nightly the active toolchain (run inside the `cargo-cgp` checkout, whose `rust-toolchain.toml`
selects it, or set `RUSTUP_TOOLCHAIN` to it), or use the Nix flake, which forces the matching nightly
regardless of the directory. The `--version` line the driver prints on success names both the
`pinned-toolchain:` it needs and the `built-against-rustc:` it was compiled with, which is the fastest
way to see which nightly a driver actually wants.

## A Nix install fails its sysroot probe under `cargo cgp`

A Nix-installed tool can fail before it compiles anything, on the query it makes to locate the
toolchain's libraries:

```text
cargo-cgp: `/nix/store/…-rust-minimal-1.99.0-nightly-…/bin/rustc --print sysroot` failed with status exit status: 127:

rustc: error while loading shared libraries: libz.so.1: cannot open shared object file: No such file or directory
```

The distinctive part is that the *same command run by hand succeeds*, and that it fails only in some
projects. Both follow from one cause: **a foreign toolchain's library directory reaching the Nix
toolchain's binaries.** Invoked as `cargo cgp check`, the entry point is rustup's `cargo` shim, which
exports the project's active toolchain's `lib` directory on the loader path for everything it
spawns — including the Nix `cargo-cgp`, and in turn the Nix `rustc` it probes. The loader searches
that directory ahead of the binary's own `RUNPATH`.

Whether that matters depends on the project, because a rustc shared library is named for its Rust
version rather than by a content hash. A project on a *different* version is harmless: its
`libLLVM.so.21.1-rust-1.91.1-stable` collides with nothing. A project pinning the **same** version as
the tool's own nightly is not: rustup's `libLLVM.so.22.1-rust-1.99.0-nightly` has exactly the name the
Nix `rustc` is looking for, so it is loaded instead — and being an FHS build, it then wants a system
`libz.so.1` the Nix loader cannot resolve. So the failure appears in whichever project happens to
track the same Rust version as the tool, which is the *opposite* of the intuition that a closely
matched toolchain is the safe case.

The fix ships in the flake, whose wrapper prefixes the pinned toolchain's `lib` so its own libraries
win that lookup; upgrade the installed tool (`nix profile upgrade cargo-cgp`, or remove and re-add it)
and the probe resolves the Nix copy again. To confirm the diagnosis on an un-upgraded install, run the
probe with the toolchain's own `lib` in front — it should succeed where the bare command failed:

```sh
NIX_RUSTC=/nix/store/…-rust-minimal-…/bin/rustc
LD_LIBRARY_PATH=$(dirname "$NIX_RUSTC")/../lib "$NIX_RUSTC" --print sysroot
```

A rustup-managed (non-Nix) install does not hit this. Its probe runs rustup's own `rustc` shim, which
sets the library path to the toolchain being queried, and the front-end then *prepends* the pinned
sysroot's `lib` for the driver — so the pinned toolchain wins every lookup by construction.

## The managed preflight rejects the setup

Before a managed check, the front-end runs a read-only preflight that verifies the toolchain and the
driver, and each failure names `cargo cgp setup` as the fix (the design is in
[Distribution](../implementation/distribution.md#what-the-preflight-verifies)). The messages tell the
cases apart, in the order the preflight checks them.

If the pinned toolchain is not installed, the preflight stops before even reaching the driver:

```text
cargo-cgp: toolchain `nightly-2026-07-16` is not available (exit status: 1)

The pinned toolchain is not installed. Run `cargo cgp setup`.
```

If the toolchain is present but the driver cannot run under it — almost always a driver built against
a *different* nightly than the one now installed — the preflight reports the load failure as a driver
problem, having already confirmed the toolchain exists:

```text
cargo-cgp: the cargo-cgp-driver could not run under toolchain `nightly-2026-07-16` (it was likely built against a different nightly). Run `cargo cgp setup`.
```

If the driver runs but reports a version or a build identity that does not match, the two are out of
lockstep — a partial upgrade, or a stale binary earlier on `PATH`:

```text
the installed cargo-cgp-driver is version 0.1.0, but this cargo-cgp is 0.2.0 (the two are out of lockstep)

Run `cargo cgp setup`.
```

```text
the cargo-cgp-driver was built against `rustc 1.98.0-nightly (…)`, but the pinned toolchain `nightly-2026-07-16` now provides `rustc 1.99.0-nightly (…)`

Run `cargo cgp setup`.
```

Every one of these is resolved by `cargo cgp setup`, which reinstalls the pinned toolchain and
rebuilds the driver against it at the front-end's own version. If you are deliberately running an
unprovisioned build (from source, or a hand-picked toolchain), set `CARGO_CGP_NO_MANAGE` to skip the
preflight and trust the environment instead — but then the toolchain-match responsibility is yours,
per the section above.

## Provisioning fails

`cargo cgp setup` manages toolchains through rustup, so on a machine without rustup it stops plainly
rather than failing obscurely:

```text
rustup was not found on PATH; cargo-cgp requires rustup to manage toolchains
```

On such a machine, install through the [Nix flake](installation.md#installing-with-nix) instead, which
provisions the toolchain at build time and needs no rustup.

## Symptom index

This table maps the distinctive fragment of each error to the section that covers it. Match the text
you see, then read the section for the fix.

| Error fragment | Cause | Section |
| --- | --- | --- |
| `missing subcommand` / `unknown cargo-cgp subcommand` | wrong invocation | [The command itself fails](#the-command-itself-fails) |
| `could not find Cargo.toml` | run outside a cargo package | [The command itself fails](#the-command-itself-fails) |
| `is cargo on PATH?` | cargo not installed / not on `PATH` | [The command itself fails](#the-command-itself-fails) |
| `failed to run the cargo-cgp-driver at …` | driver missing (managed) | [The front-end cannot find the driver](#the-front-end-cannot-find-the-driver) |
| `could not execute process` … `(never executed)` | driver path wrong (unmanaged) | [The front-end cannot find the driver](#the-front-end-cannot-find-the-driver) |
| `error while loading shared libraries: librustc_driver-…` | library path unset, or toolchain mismatch | [The driver cannot load the compiler library](#the-driver-cannot-load-the-compiler-library) |
| `--print sysroot` failed with status exit status: 127` | Nix install; a same-version rustup toolchain shadows its libraries | [A Nix install fails its sysroot probe](#a-nix-install-fails-its-sysroot-probe-under-cargo-cgp) |
| `the pinned toolchain is not installed` | pinned nightly absent | [The managed preflight rejects the setup](#the-managed-preflight-rejects-the-setup) |
| `could not run under toolchain …` | driver built against another nightly | [The managed preflight rejects the setup](#the-managed-preflight-rejects-the-setup) |
| `out of lockstep` / `now provides` | front-end and driver versions/builds differ | [The managed preflight rejects the setup](#the-managed-preflight-rejects-the-setup) |
| `rustup was not found on PATH` | no rustup for `setup` | [Provisioning fails](#provisioning-fails) |

## Further reading

- [Usage](usage.md#calling-the-driver-directly-debugging) — running the driver directly to isolate a
  problem, and reproducing cargo's exact driver invocation.
- [Installation](installation.md) — the install and update paths whose failures this document
  diagnoses.
- [Executable structure](../implementation/executable-structure.md#the-environment-contract) — the
  sibling lookup, the sysroot, and the dynamic-library path that the failures above trace to.
- [Distribution](../implementation/distribution.md#what-the-preflight-verifies) — what the managed
  preflight checks and why.
