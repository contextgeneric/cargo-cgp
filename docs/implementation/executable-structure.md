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

This is the same split Clippy uses, `cargo-clippy` to `clippy-driver`, and for the same reason. The
mechanism that connects the two halves is cargo's wrapper protocol, described next.

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
driver the two further things it needs through the environment (the next section). Finally it runs
`cargo check`, forwards the extra arguments, and propagates cargo's exit code, so a failed check
fails the command.

## Wrapping rustc: the driver

Cargo invokes the driver the way `RUSTC_WORKSPACE_WRAPPER` prescribes — the wrapper name, then the
real compiler path, then the rustc arguments:

```text
cargo-cgp-driver  /path/to/rustc  --edition=2024  --crate-name foo  src/lib.rs  ...
```

The driver runs the real compiler in-process rather than shelling out, which is the whole reason for
its existence: running through [`rustc_driver`](../../crates/cargo-cgp-driver/src/run.rs) is what
will let a future version read the compilation's diagnostics. The entrypoint is
[`run::run`](../../crates/cargo-cgp-driver/src/run.rs), called by the thin
[`bin/cargo-cgp-driver.rs`](../../crates/cargo-cgp-driver/bin/cargo-cgp-driver.rs) wrapper.

Before handing control to the compiler, [`args::rustc_args`](../../crates/cargo-cgp-driver/src/args.rs)
turns the wrapper's argument vector into a rustc argument vector. It detects "wrapper mode" — the
second argument is a path whose file stem is `rustc` — and removes that injected compiler path,
because `rustc_driver::run_compiler` treats the vector's first element as the ignored program name
and everything after it as flags; leaving the `rustc` path in would make the compiler treat it as an
input file. It then injects flags, each unless the invocation already sets it: `--sysroot` unless a
sysroot is already present (see the environment contract below), plus the two *diagnostic* flags
`-Znext-solver=globally` and `--verbose`. The diagnostic pair are not structural concerns — they are
how the driver surfaces CGP errors the default solver hides and un-elides the types rustc would
compress — so they are documented in [The error pipeline](error-pipeline.md) rather than here,
along with why they are skipped for cargo's `-vV` and `--print` info queries. The prepared vector is run under
[`rustc_driver::catch_with_exit_code`](../../crates/cargo-cgp-driver/src/run.rs), which executes the
compiler and converts a compiler-signalled failure into the process `ExitCode`, matching what plain
`rustc` returns.

The compiler behavior itself is installed through
[`callbacks::CgpCallbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs), still an empty
`rustc_driver::Callbacks` implementation. The driver's current effect on diagnostics comes entirely
from the injected diagnostic flags, not from the callbacks; those remain the extension point for the
diagnostic capture to come, covered in [The error pipeline](error-pipeline.md).

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

The driver reaches the compiler through the `rustc_private` feature, which is what permits linking
the compiler's internal crates from the sysroot. Three facts about that access shape the crate.

First, the internal crates are pulled in by `extern crate`, not through Cargo. The library
[`crates/cargo-cgp-driver/src/lib.rs`](../../crates/cargo-cgp-driver/src/lib.rs) carries
`#![feature(rustc_private)]` and `extern crate rustc_driver;`, and a module that needs a further
compiler crate adds another `extern crate rustc_*;` line there. There is nothing to declare under
`[dependencies]` for these crates.

Second, the feature gate is needed on **both** the library and the binary crate. The binary
[`bin/cargo-cgp-driver.rs`](../../crates/cargo-cgp-driver/bin/cargo-cgp-driver.rs) repeats
`#![feature(rustc_private)]`, because the binary is what ultimately links the compiler dylib, and
that link is only permitted when the linking crate opts into the feature.

Third, the API is unstable and only ships with a nightly toolchain carrying the `rustc-dev`
component, so the toolchain is pinned in [`rust-toolchain.toml`](../../rust-toolchain.toml) to an
exact dated nightly and bumped deliberately. A consequence worth holding onto: the pinned nightly is
the compiler the driver *embeds*, so when `cargo cgp check` runs against a project, that nightly is
the compiler doing the checking, and the sysroot the front-end discovers must belong to the same
nightly — a sysroot from another toolchain would load a mismatched `librustc_driver`. In practice
the tool is run under the pinned toolchain.

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
will likely grow toward it:

- **Argument reading.** The driver reads `env::args()`, whereas Clippy uses
  `rustc_driver::args::raw_args`, which also expands `@argfile` arguments that cargo passes on some
  platforms (notably Windows, to dodge command-line length limits). Until this is adopted, an
  `@argfile` invocation would not be handled.
- **Front-end argument forwarding.** `cargo-cgp` forwards extra arguments straight to `cargo check`.
  Clippy packs its own arguments into a `CLIPPY_ARGS` variable with a separator hack and chooses
  between the `check` and `fix` cargo subcommands; `cargo-cgp` has no tool-specific arguments and
  only `check`, so it needs none of that.
- **Driver front-matter.** Clippy's driver installs a logger (`init_rustc_env_logger`) and an ICE
  hook (`install_ice_hook`) with a bug-report URL, and handles `--version`, `--help`, and a
  `--rustc` passthrough. `cargo-cgp` installs none of these yet; a panic in the driver therefore
  surfaces as a plain panic rather than a formatted ICE report.
- **Info-query handling.** Clippy detects cargo's info queries (`-vV`, `--print`) and its
  `--cap-lints=allow` / `--no-deps` cases to *skip* running its lints for them. `cargo-cgp` has no
  lints to skip, and `run_compiler` already answers those queries correctly as the real compiler, so
  the driver needs no such guard — but one will be needed once the callbacks do real work that
  should not run for an info query.
- **Callbacks.** Clippy carries three `Callbacks` implementations (default, rustc-only, and
  lint-registering) and selects among them per invocation. `cargo-cgp` has one empty `CgpCallbacks`;
  the differentiation will grow when the driver begins post-processing diagnostics.

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
- [rustc_driver and rustc_interface — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html)
  describes `rustc_driver::run_compiler` and the `Callbacks` trait — the entry point the driver calls
  and the hook `CgpCallbacks` implements.
- [Example: Getting diagnostics — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/rustc-driver/getting-diagnostics.html)
  walks a minimal `rustc_driver` program that captures the compiler's diagnostics through a callback,
  which is the concrete shape the tool's future diagnostics work will take.
- [Tracking issue for crates that are compiler dependencies (#27812) — rust-lang/rust](https://github.com/rust-lang/rust/issues/27812)
  is the `rustc_private` feature's tracking issue, the background for why linking the compiler crates
  needs a nightly toolchain with the `rustc-dev` component.

## Tests

The two argument transforms this document describes — `strip_subcommand` in the front-end and
`rustc_args` in the driver — are covered by tests, and the end-to-end wrapping is verified by hand.
The full testing picture, including the example fixtures and the verification checklist, is its own
document: [Testing](testing.md).

- [`crates/cargo-cgp/tests/args.rs`](../../crates/cargo-cgp/tests/args.rs) — `strip_subcommand` across
  the invocation forms.
- [`crates/cargo-cgp-driver/tests/args.rs`](../../crates/cargo-cgp-driver/tests/args.rs) — `rustc_args`
  wrapper-mode stripping, sysroot injection, and flag injection.

## Source

- [`crates/cargo-cgp/src/run.rs`](../../crates/cargo-cgp/src/run.rs) — front-end entrypoint and
  subcommand dispatch.
- [`crates/cargo-cgp/src/args.rs`](../../crates/cargo-cgp/src/args.rs) — process-argument
  normalization.
- [`crates/cargo-cgp/src/check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) — builds and
  runs the wrapped `cargo check`, sets the environment contract.
- [`crates/cargo-cgp/src/check/driver_path.rs`](../../crates/cargo-cgp/src/check/driver_path.rs) —
  locates the sibling driver executable.
- [`crates/cargo-cgp/src/check/sysroot.rs`](../../crates/cargo-cgp/src/check/sysroot.rs) — discovers
  the toolchain sysroot.
- [`crates/cargo-cgp/src/config.rs`](../../crates/cargo-cgp/src/config.rs) — the front-end's shared
  names.
- [`crates/cargo-cgp-driver/src/run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — driver
  entrypoint; runs the compiler through `rustc_driver`.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — builds the
  rustc argument vector (wrapper-mode stripping, sysroot injection).
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs) — the
  `Callbacks` implementation, the extension point for diagnostics work.
- [`crates/cargo-cgp-driver/src/lib.rs`](../../crates/cargo-cgp-driver/src/lib.rs) — the
  `rustc_private` feature gate and `extern crate` declarations.
