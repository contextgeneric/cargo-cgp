# Executable structure

`cargo-cgp` is two cooperating executables — a front-end that wraps `cargo` and a driver that wraps
`rustc` — so that it can watch a real compilation through the compiler's own `rustc_driver` API
while presenting an ordinary cargo subcommand to the user.

## Why two executables

The tool is split into two binaries because only one of them may link the compiler internals, and
keeping that linkage isolated keeps the other binary small and ordinary. The **`cargo-cgp` crate**
([`crates/cargo-cgp`](../../crates/cargo-cgp)) is the front-end: the cargo subcommand a user invokes,
a plain `std` + `anyhow` binary. The **`cargo-cgp-driver` crate**
([`crates/cargo-cgp-driver`](../../crates/cargo-cgp-driver)) is the driver: a `rustc` replacement
that links the compiler's unstable `rustc_driver` library under the `rustc_private` feature. If the
two lived in one binary, the front-end would drag the compiler dylib — and LLVM — behind every
invocation; splitting them means the front-end builds and runs as a normal tool, and the heavyweight
linkage is confined to the process that actually needs it.

This is the same split Clippy uses, `cargo-clippy` to `clippy-driver`, and for the same reason. A
third crate, the library-only [`cargo-cgp-error-processing`](../../crates/cargo-cgp-error-processing),
holds the diagnostic-processing stage; it links no compiler internals either, so the front-end
depends on it without compromising its plain-binary status (see [Error
processing](error-processing.md)). The mechanism that connects the two executables is cargo's wrapper
protocol, described next.

## Wrapping cargo: the front-end

The front-end's whole job is to run `cargo check` with the driver installed as the compiler cargo
uses for the user's own crates. It does this with the `RUSTC_WORKSPACE_WRAPPER` environment
variable, which tells cargo to invoke a wrapper in place of `rustc` for each *workspace* crate while
leaving dependencies to compile with the normal compiler. Scoping to the workspace is deliberate:
the point of the tool is the user's code, not their dependency tree.

The entrypoint is [`run::run`](../../crates/cargo-cgp/src/run.rs), which the thin
[`bin/cargo-cgp.rs`](../../crates/cargo-cgp/bin/cargo-cgp.rs) wrapper calls. It first normalizes the
process arguments, because the tool is reachable two ways that must reduce to the same thing:

```text
cargo cgp check --workspace   →  cargo-cgp  cgp  check --workspace   (cargo inserts "cgp")
cargo-cgp check --workspace   →  cargo-cgp       check --workspace   (invoked directly)
```

[`args::strip_subcommand`](../../crates/cargo-cgp/src/args.rs) drops the program name and a *leading*
`cgp` token if present, leaving `["check", ...]` in both cases, and
[`run::dispatch`](../../crates/cargo-cgp/src/run.rs) routes on the first remaining word. The only
subcommand today is `check`; anything after it is forwarded verbatim to `cargo check`, so
`cargo cgp check -v` and `cargo cgp check --workspace` behave as expected.

[`check::run_check`](../../crates/cargo-cgp/src/check/command.rs) then builds and runs the wrapped
command. It sets `RUSTC_WORKSPACE_WRAPPER` to the driver's path — located by
[`check::driver_path`](../../crates/cargo-cgp/src/check/driver_path.rs) as a sibling of the running
front-end executable, since cargo and rustup lay the two binaries down together — and hands the
driver the two further things it needs through the environment (the next section).

The front-end does not merely pass cargo's output through, though; it captures and reshapes it. It
appends `--message-format=json` (unless the caller chose their own format), captures cargo's stdout
and stderr, parses the JSON stream into
[`cargo_metadata::Diagnostic`](../../../external/cargo_metadata/src/diagnostic.rs) values in
[`check::diagnostics`](../../crates/cargo-cgp/src/check/diagnostics.rs), runs them through the
`cargo-cgp-error-processing` crate's `process_cgp_errors`, and re-emits the result — printing each
diagnostic's rendered text, then replaying cargo's own output. `process_cgp_errors` already rewrites
each diagnostic (stripping CGP path prefixes, resugaring `Symbol!`, turning unmet `HasField` bounds
into missing-field messages), so the printed diagnostics are cleaner than rustc's; the whole [error
pipeline](error-pipeline.md) documents this path and what it will grow into. Throughout, the exit code of the `cargo` process is propagated, so a failed check
fails the command.

## Wrapping rustc: the driver

Cargo invokes the driver the way `RUSTC_WORKSPACE_WRAPPER` prescribes — the wrapper name, then the
real compiler path, then the rustc arguments:

```text
cargo-cgp-driver  /path/to/rustc  --edition=2024  --crate-name foo  src/lib.rs  ...
```

The driver runs the real compiler in-process rather than shelling out, which is the whole reason for
its existence: only in-process, through [`rustc_driver`](../../crates/cargo-cgp-driver/src/run.rs),
can it read and rewrite the compilation's diagnostics. The entrypoint is
[`run::run`](../../crates/cargo-cgp-driver/src/run.rs), called by the thin
[`bin/cargo-cgp-driver.rs`](../../crates/cargo-cgp-driver/bin/cargo-cgp-driver.rs) wrapper; it
prepares the rustc argument vector (dropping the injected `rustc` path and injecting the sysroot and
the diagnostic flags), runs the compiler under `catch_with_exit_code`, and installs a custom emitter
that renames CGP wiring notes.

All of that — the argument preparation, the `rustc_private` compiler-API access, and the three
diagnostic transformations — is the subject of the [driver deep dive](driver.md); this document
covers only how the driver sits between cargo and the compiler, and the environment contract it needs
to do so.

## The environment contract

The front-end and the driver are separate processes, so what one must tell the other travels through
the environment. Two pieces of state cross that boundary, and both exist because the driver lives
*outside* any toolchain — in `target/debug`, not in the toolchain's `bin` directory — so the
compiler cannot infer from the driver's own location things it normally would.

The front-end passes the **sysroot** through `CARGO_CGP_SYSROOT`. It discovers the value by running
`rustc --print sysroot` ([`check::sysroot`](../../crates/cargo-cgp/src/check/sysroot.rs)) and the
driver reads it back to inject `--sysroot` ([`config::SYSROOT_ENV`](../../crates/cargo-cgp-driver/src/config.rs)),
because a `rustc_driver` binary that is not inside a toolchain has no other way to locate `std`. The
two crates declare the variable name independently; the shared string is the contract between them.

The front-end also prepends the sysroot's `lib` directory to the OS **dynamic-library search
path** — `LD_LIBRARY_PATH`, or its platform equivalent
([`check::command`](../../crates/cargo-cgp/src/check/command.rs)) — so the loader can find
`librustc_driver` when cargo spawns the driver. The driver links that library dynamically from the
sysroot, and nothing else would put it on the search path.

## Accessing the Rust compiler API

The driver links the compiler's internal crates from the sysroot through the `rustc_private` feature,
which is what its in-process access to the compiler rests on. How that linkage works — the
`extern crate` declarations, the feature gate needed on both the library and the binary, and the
pinned-nightly requirement — is part of the [driver deep dive](driver.md#accessing-the-rust-compiler-api).

## Comparison with Clippy

`cargo-cgp` is modeled closely on Clippy, and the shared skeleton is easiest to see first. Both are a
front-end plus a driver; both set `RUSTC_WORKSPACE_WRAPPER` to the driver and then run a cargo
subcommand; both locate the driver as a sibling of the front-end via `current_exe`; both detect
wrapper mode by testing whether the second argument's file stem is `rustc` and drop it; both inject
`--sysroot` only when one is absent; and both run the compiler with
`rustc_driver::run_compiler` inside `catch_with_exit_code`. Reading
[`external/rust-clippy/src/driver.rs`](../../../external/rust-clippy/src/driver.rs) and
[`external/rust-clippy/src/main.rs`](../../../external/rust-clippy/src/main.rs) alongside our two
crates, the correspondence is close enough to map function for function.

The differences fall into two groups: a few are structural, forced by how the tool is distributed,
and the rest are simplifications `cargo-cgp` has not yet needed to undo.

The structural difference is the sysroot. `clippy-driver` ships *inside* the toolchain, next to
`rustc`, so the compiler infers the sysroot from the driver's own location and Clippy injects
`--sysroot` only in the rare case its `SYSROOT` variable is set; it never puts anything on the
dynamic-library path. `cargo-cgp-driver` is an out-of-tree binary in `target/debug`, so it cannot
rely on either inference — hence the front-end proactively computes the sysroot with
`rustc --print sysroot`, passes it in `CARGO_CGP_SYSROOT`, and prepends the sysroot `lib` to the
loader path. This is the one place `cargo-cgp` must do materially more than Clippy, and it follows
directly from not being a rustup component.

The remaining differences are gaps, where `cargo-cgp` is deliberately simpler than Clippy today and
will likely grow toward it. The one that is a front-end concern lives here; the driver-side gaps —
argument reading, driver front-matter, info-query handling, and the `Callbacks` set — are catalogued
in the [driver deep dive](driver.md#comparison-with-clippy).

- **Front-end argument forwarding.** `cargo-cgp` forwards extra arguments straight to `cargo check`.
  Clippy packs its own arguments into a `CLIPPY_ARGS` variable with a separator hack and chooses
  between the `check` and `fix` cargo subcommands; `cargo-cgp` has no tool-specific arguments and
  only `check`, so it needs none of that.

## Further reading

The wrapper-and-driver approach is not unique to this tool, and the two mechanisms it rests on —
cargo's compiler-wrapper protocol and the `rustc_driver` API — are documented authoritatively
elsewhere in more depth than this document repeats. Read these when you need the full contract behind
a behavior described above.

- [Environment Variables — The Cargo Book](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
  defines `RUSTC_WORKSPACE_WRAPPER` and `RUSTC_WRAPPER`: cargo runs the wrapper with the real `rustc`
  path as its first argument, the workspace variant applies only to workspace members, and it affects
  the artifact hash so wrapped builds cache separately. This is the exact protocol the front-end
  drives and the driver decodes in wrapper mode.

The authoritative references for the compiler-side mechanisms — `rustc_driver`, the `Callbacks`
trait, custom emitters, and the `rustc_private` feature — are collected in the
[driver deep dive](driver.md#further-reading).

## Tests

The front-end's argument normalization is tested directly, and the end-to-end wrapping is verified by
hand. The full testing picture, including the example fixtures and the verification checklist, is its
own document: [Testing](testing.md); the driver's own argument and rewrite tests are listed in the
[driver deep dive](driver.md#tests).

- [`crates/cargo-cgp/tests/args.rs`](../../crates/cargo-cgp/tests/args.rs) — `strip_subcommand` across
  the invocation forms.

## Source

The front-end's modules are listed here; the driver's are in the
[driver deep dive](driver.md#source).

- [`crates/cargo-cgp/src/run.rs`](../../crates/cargo-cgp/src/run.rs) — front-end entrypoint and
  subcommand dispatch.
- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs) — process-argument
  normalization.
- [`crates/cargo-cgp/src/check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) — builds and
  runs the wrapped `cargo check`, sets the environment contract, captures its output, and re-emits the
  processed diagnostics.
- [`crates/cargo-cgp/src/check/diagnostics.rs`](../../crates/cargo-cgp/src/check/diagnostics.rs) —
  parses cargo's JSON diagnostics and re-renders the processed result.
- [`crates/cargo-cgp/src/check/driver_path.rs`](../../crates/cargo-cgp/src/check/driver_path.rs) —
  locates the sibling driver executable.
- [`crates/cargo-cgp/src/check/sysroot.rs`](../../crates/cargo-cgp/src/check/sysroot.rs) — discovers
  the toolchain sysroot.
- [`crates/cargo-cgp/src/config.rs`](../../crates/cargo-cgp/src/config.rs) — the front-end's shared
  names.
