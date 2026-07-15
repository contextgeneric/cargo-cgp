# Distribution

`cargo-cgp` must run smoothly on a machine that never asked for a nightly toolchain, so distribution
is the problem of delivering two binaries and one exact nightly compiler, keeping them in lockstep,
and hiding all of it from a user whose own project builds on stable. This document describes how the
tool is packaged, installed, and provisioned, and why it is built that way.

The machinery is in place. A bare `cargo install cargo-cgp` installs the front-end; `cargo cgp setup`
provisions the pinned toolchain and the matching driver; `cargo cgp check` forces the pinned nightly
for the wrapped compilation and runs a read-only preflight that verifies a matching driver is present
before doing anything; and `cargo cgp update` upgrades the tool when a newer version is published. A
handful of environment variables ([`config`](../../crates/cargo-cgp/src/config.rs)) override the
management for local development. What is *not* yet automated is the release process itself — the two
crates must be published to crates.io together, at the same version, for the version handshake and
`setup`'s versioned driver install to hold (see [Open decisions and risks](#open-decisions-and-risks-to-resolve)).

## The problem

Distributing `cargo-cgp` is hard because the driver embeds the compiler, and the embedded compiler is
an exact, unstable nightly. The driver links the internal `rustc_driver` libraries under the
`rustc_private` feature, which ship only with a nightly toolchain carrying the `rustc-dev` component,
and the API those libraries expose changes between nightlies. So the driver is welded to one dated
nightly — the one pinned in [`rust-toolchain.toml`](../../rust-toolchain.toml) — and every part of
distribution has to respect that weld. Three consequences follow, and the plan is built around all
three at once.

The first consequence is that a user needs **two binaries, not one**. The front-end `cargo-cgp` and
the driver `cargo-cgp-driver` are separate crates for a good reason — only the driver links the
compiler and LLVM, and keeping that out of the front-end keeps the front-end a small, ordinary binary
(see [Executable structure](executable-structure.md#why-two-executables)). But a plain
`cargo install cargo-cgp` installs only the front-end, and the front-end cannot do anything without
its driver sibling. Getting the tool working therefore always takes a second step to bring the driver
in, and both must end up side by side.

The second consequence is that the pinned nightly, with its `rustc-dev` component, **has to be
present on the machine**. Building the driver needs `rustc-dev` (the compiler-internal `.rlib`s) and
`llvm-tools`; running it then needs the matching `librustc_driver` shared library and the
standard-library sysroot that only that toolchain supplies. A machine that only has stable Rust has
none of this.

The third consequence is that at check time the **sysroot and the dynamic `librustc_driver` must both
belong to the pinned nightly**, or the driver loads a compiler it was not compiled against and
crashes. The driver embeds the pinned nightly's `rustc_driver`; if it is handed a stable sysroot, or
if the loader finds a differently-versioned `librustc_driver` on its path, the ABI will not match.
This is the failure the current docs warn about, and the plan's job is to make the match automatic.

Two widespread misconceptions are worth clearing up before the design, because both make the problem
sound worse than it is. **Updating the host's nightly does not break the driver.** The pin is a
*dated* nightly (`nightly-2026-07-02`), which rustup treats as an immutable, distinct toolchain; a
`rustup update` refreshes the rolling `stable`/`beta`/`nightly` channels and leaves a dated nightly
untouched. The driver keeps working until *cargo-cgp itself* ships a new version that bumps the pin.
**And the project being checked does not need to use nightly at all.** A user's crate can pin stable,
or nothing; `cargo-cgp` supplies its own compiler for the check and leaves the project's real builds
alone. The design below is what makes that true.

## How similar tools solve it

`cargo-cgp` is not the first tool to link the compiler and ship it to strangers, and two reference
points bracket the design space. Clippy sits at one extreme and the `rustc_plugin` family at the
other, and `cargo-cgp` lands deliberately between them.

Clippy avoids the whole problem by **being part of the toolchain**. `clippy-driver` is built in-tree
by the same CI run that builds the `rustc` it links, distributed as the `clippy` rustup component, and
installed with `rustup component add clippy`. Because the driver ships *inside* the toolchain next to
`rustc`, it never has to find a sysroot, never risks a version mismatch, and never needs a separate
install of a compiler. This is the cleanest possible answer, and it is closed to `cargo-cgp`: an
out-of-tree project cannot inject a component into rustup's distribution.

The [`rustc_plugin`](https://github.com/cognitive-engineering-lab/rustc_plugin) framework — which
backs Flowistry, Aquascope, Paralegal, and Argus — solves the out-of-tree case, and it is the model
`cargo-cgp` follows. Three of its techniques carry over directly. It **bakes the pinned nightly into
a compile-time constant** (`pub const CHANNEL: &str = env!("RUSTC_CHANNEL")`), read from the
toolchain file at build time, so both the CLI and the driver agree on exactly one toolchain string.
It **treats a nightly bump as a semver-breaking change**, tagging each release with its nightly as a
prerelease label (`0.15.2-nightly-2026-05-01`), so a user's dependency pin names the nightly too. And
its build script **embeds the toolchain's library directory as an rpath** on the driver
(`rustc --print target-libdir` → `-Wl,-rpath,…`), so the driver finds `librustc_driver` at load time
without any environment variable.

The `rustc_plugin` tools also show what shapes the *install* step. Because
[`cargo install` ignores a crate's bundled `rust-toolchain.toml`](https://github.com/rust-lang/cargo/issues/11036),
a crate that needs a specific nightly to build cannot rely on the toolchain file being honored — the
nightly has to be selected explicitly with `cargo +<pinned> install`. And because compiling against
`rustc-dev` is slow, the mature tools lean on an installer that does the toolchain setup for the user:
Flowistry's editor extension runs
`rustup toolchain install nightly-… -c rustc-dev -c llvm-tools-preview` and then
`cargo +nightly install flowistry`. `cargo-cgp` adopts the baked-in constant and the installer-driven
setup — folding that installer into its own `cargo cgp setup` — and improves on the `rustc_plugin`
model in one place: how the check's toolchain is chosen (below).

## The pinned toolchain is an internal detail

The central design decision is that `cargo-cgp` **forces its own pinned nightly for the check and
never asks the project to supply it**. This is where the plan diverges from `rustc_plugin`, whose
tools require the *analyzed project* to pin the same nightly in its own `rust-toolchain.toml`. For a
diagnostic tool that mostly runs `check`, that requirement is an unnecessary imposition on the user's
repository. `cargo-cgp` instead carries the toolchain as an implementation detail and overrides the
project's toolchain for the duration of the check.

The mechanism is one embedded constant and one environment variable. The pinned nightly string lives
in a shared `config` constant — call it `PINNED_TOOLCHAIN` — generated at build time from
[`rust-toolchain.toml`](../../rust-toolchain.toml) by a build script, exactly as `rustc_plugin`
derives `CHANNEL` from `env!("RUSTC_CHANNEL")`, so the toolchain file stays the single source of
truth and the constant can never drift from it. When the front-end runs the wrapped `cargo check`, it
sets `RUSTUP_TOOLCHAIN=<PINNED_TOOLCHAIN>` in the child environment (equivalently, it could invoke
`cargo +<PINNED_TOOLCHAIN> check`). This one variable makes everything downstream consistent.

Forcing the toolchain resolves the sysroot-and-dylib coherence problem for free. With
`RUSTUP_TOOLCHAIN` set to the pinned nightly, the front-end's existing
`rustc --print sysroot` discovery ([`check::sysroot`](../../crates/cargo-cgp/src/check/sysroot.rs))
returns the *pinned* sysroot, so the `--sysroot` the driver injects and the `librustc_driver` it
loads both belong to the nightly the driver was compiled against. The three things that must agree —
the driver's embedded compiler, the injected sysroot, and the loaded shared library — now come from
one toolchain by construction, with no version-matching left to chance.

The override is safe because `RUSTUP_TOOLCHAIN` outranks `rust-toolchain.toml`. Rustup's override
precedence runs, highest first: a `+toolchain` on the command line, the `RUSTUP_TOOLCHAIN` variable, a
directory override, the `rust-toolchain.toml` file, then the default. Setting the variable therefore
wins over whatever the project pins, so `cargo cgp check` uses the pinned nightly even in a repository
that has committed its own `rust-toolchain.toml` — and it leaves that file in place, untouched, for
the project's ordinary builds.

Running the check under a nightly the project did not choose is an accepted trade-off, and it is the
same one the tool already makes. The driver already injects `-Znext-solver=globally` and `--verbose`,
so `cargo cgp check` never claimed to reproduce a plain `cargo check` — it is a diagnostic pass with
its own compiler settings, the way Clippy runs under its own toolchain. In rare cases a program could
compile under the project's stable toolchain but report an error under the pinned nightly, or the
reverse; this is documented as inherent to using a fixed diagnostic compiler
([The driver](driver.md#choosing-the-trait-solver) records the analogous next-solver caveat), not a
distribution bug.

## Installing the binaries

`cargo-cgp` is always installed with cargo — there is no prebuilt-binary channel — and the install
splits cleanly in two: a bare `cargo install cargo-cgp` puts the front-end in place, and
`cargo cgp setup` provisions everything heavyweight. That split is what lets the first command be one a
user types from memory, with no nightly date string in it, and it dissolves the bootstrapping problem
that a `rustc_private` tool would otherwise have.

The front-end installs on **any** toolchain, which is the key that makes the split work. Because the
front-end is a plain `std`/`anyhow` binary with no `rustc_private` linkage, `cargo install cargo-cgp`
builds it under whatever toolchain the user already has — stable, or anything else — and needs no
pinned nightly. So the cargo#11036 hazard above, that `cargo install` ignores a bundled
`rust-toolchain.toml`, never bites: the front-end does not care which toolchain builds it. The
freshly installed front-end embeds the `PINNED_TOOLCHAIN` constant and the whole setup routine, so it
carries everything needed to provision the rest of the tool itself.

`cargo cgp setup` then performs the two heavyweight steps under the pinned string it reads from that
constant, so the user never types the nightly date. It runs
`rustup toolchain install <PINNED_TOOLCHAIN> -c rustc-dev -c llvm-tools` for the build-time compiler
internals, then `cargo +<PINNED_TOOLCHAIN> install cargo-cgp-driver@<version>` to build the driver
under that exact nightly and drop it next to the front-end in `~/.cargo/bin`. The explicit
`+<PINNED_TOOLCHAIN>` is mandatory, not stylistic: without it `cargo install` would build the driver
against the active toolchain and fail on stable or mis-version on another nightly (cargo#11036 again).
Pinning the driver to the front-end's own version (`@<version>`) is what keeps the pair in lockstep,
and it is possible only because the two crates are released together (below).

Keeping the two crates separate is exactly what enables this. Folding the driver into the `cargo-cgp`
package as a second `[[bin]]` would force the whole package — front-end included — to build under the
pinned nightly, losing the bare-`cargo install cargo-cgp` bootstrap. Two crates let the front-end
install on any toolchain while the driver stays confined to the pinned one, and cargo places both in
`~/.cargo/bin`, so the front-end's sibling lookup
([`check::driver_path`](../../crates/cargo-cgp/src/check/driver_path.rs)) finds the driver with no
configuration. Lockstep is enforced by the version preflight below, not by shared packaging.

## Provisioning: `setup` does the work, `check` only verifies

All provisioning is delegated to one command, and `cargo cgp check` never installs or builds anything
itself. `cargo cgp setup` performs every mutating step; `cargo cgp check` runs a read-only preflight
that either passes silently or stops with an error telling the user to run `cargo cgp setup`. The
division rests on one rule: **`check` must never do slow, stateful, or surprising work** — no `rustup`
installs, no driver compilation, no toolchain changes — because a command whose job is to print
diagnostics should not, as a side effect, spend minutes compiling or alter the user's toolchains.
Everything of that kind lives in `setup`, which the user invokes knowingly.

`cargo cgp setup` performs the full provisioning: install the pinned toolchain
(`rustup toolchain install <PINNED_TOOLCHAIN> -c rustc-dev -c llvm-tools`) and build the driver under
it (`cargo +<PINNED_TOOLCHAIN> install cargo-cgp-driver@<version>`), landing it beside the front-end.
It is the command a user runs once after `cargo install cargo-cgp`, and the command the preflight
names whenever it finds something wrong. Making provisioning a single named subcommand gives the docs
one stable entry point instead of a shifting list of `rustup` and `cargo` incantations, and it is the
one place that reads `PINNED_TOOLCHAIN`, so the user never types the nightly date.

### What the preflight verifies

The preflight is a fast, read-only sequence at the top of `check`, run before any cargo is spawned,
and every failure produces the same actionable outcome: an error that names what is wrong and directs
the user to `cargo cgp setup`. It answers three questions in order — is a driver present, is the pinned
toolchain installed, and does the driver actually run and match — and stops at the first "no." The
ordering is deliberate: the cheap, disambiguating checks come first, so that when the driver itself
fails, the preflight already knows whether the toolchain underneath it is to blame and can say so.

**Is a driver present?** The preflight locates the driver as the `CARGO_CGP_DRIVER` override if set,
otherwise as the sibling of the running front-end (the existing
[`driver_path`](../../crates/cargo-cgp/src/check/driver_path.rs) lookup). No driver is the plainest
"run setup" case.

**Is the pinned toolchain installed?** The preflight runs `rustc +<PINNED_TOOLCHAIN> --version
--verbose` and reads back the toolchain's own rustc identity — its `commit-hash` and `release`. This
is cheap, and doing it first separates a *missing toolchain* from a *bad driver*: if this query fails,
the toolchain the driver needs is simply not installed, and that is the message the user gets. If it
succeeds, the preflight now holds the identity of the compiler the check will force, to compare the
driver against.

**Does the driver run and match?** The preflight then invokes `cargo-cgp-driver --version` under the
*exact* environment the real check will use — `RUSTUP_TOOLCHAIN` forced to the pinned nightly, the
discovered sysroot, and the dynamic-library search path set. This single step is both a load test and
a version handshake, because a driver built against the wrong nightly fails the first and a driver
from the wrong release fails the second:

- **Will it even load?** The driver binary dynamically links `librustc_driver-<hash>.so`, where the
  hash is fixed at build time to the exact nightly the driver was compiled against, and it is recorded
  in the binary's dynamic-dependency list. A driver built against a *different* nightly than the one
  installed therefore names a shared library the pinned toolchain's `lib` directory does not contain,
  and the operating system's loader aborts the process before `main` with a "cannot open shared object
  file" error. So a driver that will not run under the forced toolchain is, in the overwhelmingly
  common case, one compiled against the wrong nightly — and because the previous step already
  confirmed the toolchain *is* installed, the preflight attributes a launch failure to the driver and
  points at `setup` with that specific message, rather than passing a cryptic loader error to the user.
- **Is it the right build?** When the driver does load, it prints its `--version` output and exits.
  That output carries three fields baked in at build time, one per line so a value with spaces stays
  unambiguous — `cargo-cgp-driver <tool_version>`, `pinned-toolchain: <name>`, and
  `built-against-rustc: <rustc --version line>`, the last being the compiler that *actually* compiled
  the driver, which its build script captured by querying the compiling rustc (`$RUSTC --version`) so
  the value reflects reality rather than intent. The preflight parses these and makes two strict
  comparisons: `tool_version` must equal the front-end's own version exactly (a difference is a partial
  upgrade or a stale binary on `PATH`), and `built_against_rustc` must equal the installed toolchain's
  `rustc --version` read in the previous step (a difference is a driver built against a nightly other
  than the pinned one, which loaded only because its hash happened to resolve). Either mismatch sends
  the user to `setup`.

The flag is the ordinary `--version`, not a tool-specific name: `cargo-cgp` versions on its own track,
unrelated to `cgp`'s, so a `--cgp-version` would misleadingly imply otherwise. The driver already
distinguishes a direct invocation from cargo's wrapper invocation by testing whether its second
argument is a `rustc` path (see [The driver](driver.md#preparing-the-argument-vector)), so
`cargo-cgp-driver --version` prints the tool's version line while a wrapper-mode `--version` still
forwards to the real compiler for cargo's probing — the same split `clippy-driver` uses.

The two comparisons are strict equality rather than a compatibility range, and that is sound only
because the two crates are **released and version-bumped in lockstep** — which the workspace already
enforces. The crates share one version through `[workspace.package]` in the root
[`Cargo.toml`](../../Cargo.toml), so a release bumps both at once and a `cargo-cgp` is never published
against a `cargo-cgp-driver` of a different number. Every release ships the pair together, so a
correctly provisioned machine holds a matching pair and the preflight passes trivially, paying only
the cost of two quick subprocesses. The checks exist as the safety net for the ways an installed pair
drifts apart *after* a correct install: one crate upgraded but not the other, a leftover binary on
`PATH`, or the pinned toolchain removed from under a driver that needs it.

The alternative — having `check` itself run `rustup install`, `cargo install`, and rebuild the driver
on any incompatibility — is rejected deliberately. Automatic recompilation would make a routine
`cargo cgp check` occasionally block for minutes and mutate the user's toolchains as a side effect,
which is the wrong behavior for a diagnostic command. Keeping `check` a read-only verifier keeps the
fast path fast and makes every heavy, stateful action something the user triggers on purpose.

### Updating with `cargo cgp update`

A `cargo cgp update` subcommand upgrades the tool to its latest version, and its two-step shape —
update the front-end, then hand off to the *new* front-end's `setup` — is not merely convenient but
the only ordering that can work. A release may bump the pinned nightly, and the knowledge of the new
toolchain and the new provisioning logic lives *inside* the new `cargo-cgp` binary; the old process
cannot provision the new driver because it only knows the old pin. So `update` first reinstalls the
front-end with `cargo install cargo-cgp` (which, being plain `std`/`anyhow`, builds under any
toolchain and needs no pinned nightly), then execs the freshly installed `cargo-cgp setup`, which
brings the driver and toolchain up to the new version. The heavy `rustc-dev` work stays deferred to
the new `setup`, exactly where it belongs.

Before touching anything, `update` finds out whether there is a newer version to move to **in the
running version's channel** and skips out early when there is not. It reads the crates.io sparse index
for the front-end crate (`https://index.crates.io/ca/rg/cargo-cgp`, one JSON line per published
version), enumerates every non-yanked version, and picks the highest one whose pre-release-ness matches
the running version's — a stable install considers only stable candidates, a pre-release install only
pre-releases. If that highest in-channel version is **not strictly newer**, `update` prints "already up
to date (v<current>)" and exits without invoking `cargo install`, rustup, or `setup`; only a strictly
newer one triggers the reinstall.

Preserving the channel is why `update` enumerates all versions rather than asking cargo for the single
"latest". Both `cargo search` and `cargo info` report only a crate's *max version*, which **includes
pre-releases** — so if a pre-release higher than the latest stable is published, a stable install could
neither see the latest stable through them nor safely take the pre-release. Reading the index directly
gives every version, so the channel filter can pick the right one: `v0.1.0 → v0.1.1`, never
`v0.1.2-alpha`; and `v0.1.0-alpha → v0.1.1-alpha`. The comparison is `semver`, which orders a
pre-release below its release, so the "highest in channel" and "strictly newer" tests are both exact.

The index is read over HTTP with the widely-used `ureq` (rustls TLS) and `serde_json`, kept to the
front-end's `update` path alone — the check path and the driver link neither. The sparse-index access
follows the shape of the `crates-index` crate but is written directly over those two rather than taking
on that dependency. One limitation follows from reading `index.crates.io` directly: unlike the eventual
`cargo install`, it does not consult a user's configured registry mirror or private registry, so
automatic version discovery targets the default crates.io (see
[Open decisions and risks](#open-decisions-and-risks-to-resolve)).

Installation and update both go **through cargo** — there is no self-replace mechanism to maintain.
`update` shells out to `cargo install cargo-cgp` (the same tool that installed it), and where that
succeeds the exec into the new `setup` follows. This keeps `update` a thin orchestrator over cargo
rather than a re-implementation of what cargo already does.

The one case cargo cannot handle is the running binary on Windows, and there `update` fails cleanly
with instructions rather than trying to be clever. When a user types `cargo cgp update`, the running
process *is* `~/.cargo/bin/cargo-cgp`, the file `cargo install` overwrites. On Unix this is fine: cargo
writes the new binary alongside and moves it over the old name with an atomic rename, and the running
process keeps executing its original inode. On **Windows a running executable is locked**, so the
`cargo install` step fails with "Access is denied." `update` catches that failure and prints the exact
commands for the user to run by hand — `cargo install cargo-cgp` followed by `cargo cgp setup`, from a
shell where `cargo-cgp` is not currently running — which succeed precisely because they are not
self-referential. The tool does not embed a self-replacing installer to work around this; it delegates
to cargo and hands the user the manual path when cargo cannot finish.

Two smaller properties round out the design. **An interrupted update degrades safely**: between
reinstalling the front-end and finishing `setup`, the machine briefly holds a new front-end against an
old driver, and that is exactly the mismatch the [preflight](#what-the-preflight-verifies) already
catches, so the next `cargo cgp check` tells the user to run `cargo cgp setup` and recovers — which is
also why the Windows manual path above is safe to leave half-done. And **superseded toolchains
accumulate**: each nightly bump installs a new multi-gigabyte dated toolchain and leaves the previous
one in `~/.rustup/toolchains`, so `update` should *report* the now-unused toolchain and suggest
`rustup toolchain uninstall`, but never remove it automatically, since another tool may still depend
on that nightly.

## Escape hatches for local development

Developing `cargo-cgp` itself means running an unreleased front-end against a just-built driver, so
the provisioning above must be fully overridable, and a few environment variables provide the escape
hatches. They are what let the UI snapshot suite drive the freshly-compiled driver out of
`target/debug` instead of a provisioned one, and they are essential to the tool's own test loop.

- `CARGO_CGP_DRIVER` names an explicit driver executable, bypassing the sibling lookup. The UI test
  harness sets this to the `target/debug/cargo-cgp-driver` it just built, which is how the suite tests
  a driver that was never installed.
- `CARGO_CGP_NO_MANAGE` (or an equivalent skip flag) turns off the preflight and its version checks
  entirely, telling `cargo-cgp` to trust whatever driver and toolchain the environment already
  provides. A developer running inside the source checkout under the pinned toolchain wants exactly
  this — no provisioning, no handshake, just run.
- `CARGO_CGP_TOOLCHAIN` overrides `PINNED_TOOLCHAIN` at runtime, so the tool can be pointed at a
  different nightly for testing a toolchain bump before the constant is changed. Because a hand-picked
  toolchain will not match the driver's baked-in `built_against_rustc`, it is normally paired with
  `CARGO_CGP_NO_MANAGE` so the preflight does not reject the mismatch.

These variables belong in the shared `config` modules alongside the existing well-known names
(`CARGO_CGP_SYSROOT` and the rest), passed into the functions that use them rather than read
ad hoc, per the [code organization conventions](../../AGENTS.md#code-organization-conventions).

## Loading the compiler at runtime

At runtime the driver must load the pinned nightly's `librustc_driver`, and the plan keeps the
front-end's current sysroot-discovery approach. The front-end discovers the sysroot with
`rustc --print sysroot` (now returning the pinned sysroot, because the toolchain is forced), passes it
to the driver through `CARGO_CGP_SYSROOT`, and prepends the sysroot's `lib` directory to the OS
dynamic-library search path so the loader finds `librustc_driver` — the contract documented in
[Executable structure](executable-structure.md#the-environment-contract).

Runtime discovery is chosen over a build-time rpath, though both would work: because the driver is
always compiled on the machine it runs on, an embedded rpath — as `rustc_plugin`'s `build_main` bakes
in via `rustc --print target-libdir` → `-Wl,-rpath,…` — would point at a valid local path. Discovery
wins on two independent grounds. The front-end must run `rustc --print sysroot` anyway to inject the
`--sysroot` the out-of-tree driver needs, so the search path falls out of work already done; and it
needs no build-time linker configuration and is already implemented. An embedded rpath could be added
later as a belt-and-suspenders default, but it is not what the design depends on.

## Rust Analyzer integration

Rust Analyzer can run `cargo cgp check` as its on-save diagnostic backend, and the design already
accommodates it, but two of the plan's distribution choices — the JSON rendering and the forced
nightly — turn out to be exactly what makes the integration work or fail. The good news is that the
one interaction a reader would expect to break, Rust Analyzer's own compiler wrapper colliding with
the tool's, does not happen. The risk that is real is build-artifact contention, and it has a
concrete fix.

The tool is wired in through `rust-analyzer.check.overrideCommand`, not `check.command`. Rust
Analyzer's `check.command` names a single cargo subcommand and appends its own flags after it, so it
cannot express `cargo cgp check` (a two-word invocation), and multi-word or alias commands there are
[not fully supported](https://github.com/rust-lang/rust-analyzer/issues/8098). The `overrideCommand`
setting takes the full command as an argument array, is run from the workspace root, and **must emit
JSON**, so the user supplies the whole line including `--message-format=json`:

```jsonc
"rust-analyzer.check.overrideCommand": [
  "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
]
```

The JSON reaches Rust Analyzer through the pipeline the tool already has, with nothing new to build.
Rust Analyzer's flycheck parses cargo's `--message-format=json` stream — the `compiler-message`
records cargo wraps each rustc diagnostic in. In `cargo-cgp` the driver renders the *transformed*
diagnostic as rustc JSON through its `JsonEmitter`, cargo wraps that into its own message stream, and
the front-end forwards cargo's stdout untouched (see [The error pipeline](error-pipeline.md)). So the
front-end need only forward the `--message-format=json` argument to `cargo check` — which it already
does, since it forwards everything after `check` verbatim — and Rust Analyzer receives well-formed
cargo JSON with the CGP transforms already applied inside each diagnostic.

Rust Analyzer's own `RUSTC_WRAPPER` does not collide with the tool's `RUSTC_WORKSPACE_WRAPPER`,
because the two operate in different phases. Rust Analyzer sets `RUSTC_WRAPPER=rust-analyzer` only
when running build scripts and gathering proc-macro information
(`rust-analyzer.cargo.buildScripts.useRustcWrapper`), to skip work it does not need — *not* for the
flycheck check command. The check command `cargo-cgp` runs is therefore a clean cargo invocation with
only the tool's own `RUSTC_WORKSPACE_WRAPPER` set. Even in the hypothetical where both were present,
cargo nests them as `$RUSTC_WRAPPER $RUSTC_WORKSPACE_WRAPPER $RUSTC`, so the driver still receives the
real `rustc` path as its second argument and its wrapper-mode detection holds; the only realistic way
to actually chain a second wrapper is a user's global `RUSTC_WRAPPER` such as `sccache`, which is a
minor, user-created interaction rather than something Rust Analyzer imposes.

The genuine integration risk is build-artifact contention, and the fix is a dedicated target
directory that the tool uses by default. Because `cargo-cgp` forces the pinned nightly and injects
`-Znext-solver` and `--verbose`, its check produces artifacts with a different fingerprint from the
project's ordinary `cargo build` and from Rust Analyzer's own project-loading builds. Sharing one
`target/` directory then means the two fight over cargo's build lock and churn each other's caches —
the class of slowness and flapping diagnostics reported when clippy is used as the check command
([rust-analyzer#19336](https://github.com/rust-lang/rust-analyzer/issues/19336)).

So `cargo cgp check` always builds into `target/cgp`, not the project's `target/`, on every
invocation — whether Rust Analyzer is the caller or not — so a check never invalidates a normal build
and vice versa. This mirrors how `rustc_plugin` isolates its own `target/plugin-<channel>` directory,
and the isolation helps command-line use as much as the editor; the cost is a one-time rebuild of the
dependency graph in `target/cgp`, cached thereafter. A user who needs a different location passes
`--target-dir`, which overrides the default: the front-end injects its `target/cgp` default only when
the forwarded arguments do not already set one, the same inject-only-when-absent rule the driver
follows for its own flags. (A user's `CARGO_TARGET_DIR` is likewise respected in preference to the
default.)

One residual caveat has no fix, only awareness: Rust Analyzer's inline semantic analysis and the
`cargo-cgp` flycheck can disagree. Rust Analyzer's own type inference runs against the project's
toolchain and default solver, while the flycheck diagnostics come from the pinned nightly under
`-Znext-solver`, so occasionally `cargo-cgp` surfaces a diagnostic Rust Analyzer's inline analysis
does not, or the reverse. This is the same solver-and-toolchain divergence the tool already accepts
on the command line ([The driver](driver.md#choosing-the-trait-solver)), now visible in the editor;
it is a property of running a fixed diagnostic compiler, not a bug to resolve.

## Comparison with Clippy

Distribution is the one area where `cargo-cgp` cannot follow Clippy, and the divergence is
foundational rather than a gap to close later. Clippy is a first-party rustup component: `clippy-driver`
is compiled in-tree by the same CI that builds its `rustc`, shipped as part of the toolchain, and
installed with `rustup component add clippy`. Every distribution problem this document solves — two
binaries, a matching nightly, a coherent sysroot and `librustc_driver` — Clippy gets for free, because
its driver lives inside the toolchain and is versioned with it atomically.

`cargo-cgp` is out-of-tree, so it must reconstruct by hand what Clippy inherits from the distribution.
It pins a nightly instead of being built with one; it installs that nightly as a separate step instead
of being part of it; it discovers and injects a sysroot instead of being found next to `rustc`; and it
runs a version preflight instead of being versioned atomically with the compiler. The closest working
model for all of this is not Clippy but the `rustc_plugin` family, and the two places `cargo-cgp`
improves on that model are the check-time toolchain override (the analyzed project needs no nightly of
its own, where `rustc_plugin` requires it to pin the same one) and the bare-`cargo install cargo-cgp`
bootstrap that defers every heavyweight step to an explicit `cargo cgp setup` (so the user never types
a nightly date to get started).

## Open decisions and risks to resolve

One design choice is still open, one release-process requirement is not yet enforced, and a few
limitations are deliberate; they are collected here so they are handled knowingly rather than
discovered late.

- **Versioning scheme (open).** Whether to adopt `rustc_plugin`'s nightly-as-prerelease-label
  convention for crates.io releases, or keep a plain semver and carry the nightly only in
  `PINNED_TOOLCHAIN`. `update`'s `semver` comparison already orders either scheme correctly; the
  choice is about release ergonomics, not code.
- **Atomic two-crate publishing (not yet enforced).** The exact-version preflight and `setup`'s
  `cargo-cgp-driver@<version>` both assume both crates are published to crates.io at the same version
  on every release. `[workspace.package]` keeps the versions equal *in the source tree*, but nothing
  yet guarantees both are *published* together — a release that ships `cargo-cgp` without a matching
  `cargo-cgp-driver` leaves `setup` unable to find the driver version it asks for. The release
  automation must publish the pair atomically.
- **Driver co-location falls back when the front-end is not in a `bin` directory (limitation).**
  `setup` derives `--root` from the front-end's `current_exe` so the driver lands beside it (for the
  usual `~/.cargo/bin/cargo-cgp`, root `~/.cargo`). When the front-end sits somewhere that does not fit
  cargo's `<root>/bin` convention, `setup` warns and installs the driver to cargo's default location
  instead of guessing — which may not co-locate them. This is fine for the ordinary `cargo install`
  path and left as a rough edge for unusual layouts.
- **`setup` idempotency rests on cargo's "already installed" being success (assumption).** Re-running
  `setup` (as `update` does) relies on `rustup toolchain install` being a no-op when already satisfied
  and on `cargo install cargo-cgp-driver@<version>` exiting successfully when that version is already
  present. Both hold today; if a future cargo made "already installed" an error, `setup` would need a
  guard.
- **rustup is assumed throughout (limitation).** Toolchains are managed entirely through rustup —
  `RUSTUP_TOOLCHAIN`, `rustup toolchain install`. A machine whose Rust comes from a distro package,
  Nix, or another manager has no rustup to drive, so it is out of scope for automatic provisioning;
  `setup` reports a missing rustup plainly rather than failing obscurely.
- **`update` version discovery targets the default crates.io (limitation).** It reads
  `index.crates.io` directly, so unlike the `cargo install` that follows it, it does not consult a
  user's configured registry mirror or source replacement. A locked-down network that blocks
  `index.crates.io` will make `update`'s check fail (with a clear error); the manual
  `cargo install cargo-cgp` + `cargo cgp setup` path still works through the configured registry.

## Further reading

The mechanisms this plan rests on are documented authoritatively elsewhere; read these when
implementing the piece each one covers.

- [`rustc_plugin`](https://github.com/cognitive-engineering-lab/rustc_plugin) is the closest working
  model for out-of-tree distribution of a `rustc_private` tool — the `CHANNEL` constant, the
  rpath-embedding build script, and the nightly-as-prerelease versioning are all worth reading in its
  `src/lib.rs`, `src/build.rs`, `src/cli.rs`, and `src/driver.rs`.
- [`cargo install` ignores `rust-toolchain.toml` (cargo#11036)](https://github.com/rust-lang/cargo/issues/11036)
  is why `setup` must install the driver with an explicit `cargo +<pinned> install` — and why the
  front-end, needing no particular toolchain, installs with a bare `cargo install cargo-cgp`.
- [Overrides — The rustup book](https://rust-lang.github.io/rustup/overrides.html) defines the
  toolchain override precedence that lets `RUSTUP_TOOLCHAIN` win over a project's `rust-toolchain.toml`.
- [Flowistry's installation notes](https://github.com/willcrichton/flowistry) show a real editor tool
  driving `rustup toolchain install … -c rustc-dev -c llvm-tools-preview` and `cargo +nightly install`
  from an extension — the same toolchain-install-then-cargo-install shape `cargo cgp setup` automates.
- [Replacing a running executable on Windows (rustup#1186)](https://github.com/rust-lang/rustup/issues/1186)
  documents the file-lock that makes `cargo install` fail when it targets the running `cargo-cgp`,
  which is why `cargo cgp update` falls back to printing the manual commands there.
- [The crates.io sparse index](https://index.crates.io/) and the
  [`crates-index` crate](https://crates.io/crates/crates-index) — the index `update` reads to
  enumerate versions, and the reference implementation for the sparse-index path convention and the
  `vers`/`yanked` fields (used as a reference, not a dependency).

## Tests

The pieces are unit-tested over their pure logic, keeping the process-spawning at the edges, and the
UI suite exercises the managed/unmanaged wiring end to end.

- [`crates/cargo-cgp/tests/preflight.rs`](../../crates/cargo-cgp/tests/preflight.rs) — parses the
  driver's `--version` output (accepting the real shape, rejecting a foreign first line and missing
  fields); the [`evaluate`](../../crates/cargo-cgp/src/check/preflight.rs) verdict on a matching
  driver, a version mismatch, and a rustc mismatch; and a check that the baked-in `PINNED_TOOLCHAIN`
  equals the channel in [`rust-toolchain.toml`](../../rust-toolchain.toml).
- [`crates/cargo-cgp/tests/update.rs`](../../crates/cargo-cgp/tests/update.rs) — `sparse_index_path`
  over the registry's dir convention; `parse_versions` skipping yanked and unparseable lines; and the
  channel-preserving `select_update` — a stable install taking the highest stable and *never* a
  pre-release, a pre-release install taking the highest pre-release, and no-newer/downgrade yielding
  `None` — all without a network call.
- [`crates/cargo-cgp/tests/command.rs`](../../crates/cargo-cgp/tests/command.rs) — `forwards_target_dir`
  detects an explicit `--target-dir` (spaced or `=`) so the default is not injected over it.
- [`crates/cargo-cgp/tests/dispatch.rs`](../../crates/cargo-cgp/tests/dispatch.rs) — the
  side-effect-free dispatch error paths (unknown and missing subcommand).
- The [UI snapshot suite](testing.md) is the standing end-to-end proof that the driver runs as the
  compiler; it drives the built binaries with `CARGO_CGP_NO_MANAGE` set, so the check runs unmanaged
  against the toolchain `cargo test` already selected. The managed path (preflight + toolchain forcing)
  is not covered by the suite, since installing a provisioned driver is out of scope for the
  source-tree test loop.

## Source

The front-end holds the provisioning and management; the driver adds the version query; each crate
has a build script that bakes in the pinned toolchain.

- [`crates/cargo-cgp/src/config.rs`](../../crates/cargo-cgp/src/config.rs) — the well-known names:
  `PINNED_TOOLCHAIN` and `TOOL_VERSION` (baked in), the environment variables (`CARGO_CGP_DRIVER`,
  `CARGO_CGP_NO_MANAGE`, `CARGO_CGP_TOOLCHAIN`, `RUSTUP_TOOLCHAIN`), the `target/cgp` default, and the
  crate names.
- [`crates/cargo-cgp/build.rs`](../../crates/cargo-cgp/build.rs) — derives `PINNED_TOOLCHAIN` from
  [`rust-toolchain.toml`](../../rust-toolchain.toml), mirroring how `rustc_plugin` derives `CHANNEL`.
- [`crates/cargo-cgp/src/run.rs`](../../crates/cargo-cgp/src/run.rs) — dispatches `check`, `setup`, and
  `update`.
- [`crates/cargo-cgp/src/toolchain.rs`](../../crates/cargo-cgp/src/toolchain.rs) — resolves the
  effective pinned toolchain and queries its `rustc --version` through rustup.
- [`crates/cargo-cgp/src/check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) — runs the
  preflight (when managed), forces `RUSTUP_TOOLCHAIN`, wires the driver and sysroot, and injects the
  `target/cgp` default unless the caller set the target directory.
- [`crates/cargo-cgp/src/check/preflight.rs`](../../crates/cargo-cgp/src/check/preflight.rs) — verifies
  the toolchain is installed and the driver runs and matches (the pure `evaluate` plus the
  `--version`-parsing and IO), returning the discovered sysroot.
- [`crates/cargo-cgp/src/check/driver_path.rs`](../../crates/cargo-cgp/src/check/driver_path.rs) — the
  `CARGO_CGP_DRIVER` override and the sibling lookup.
- [`crates/cargo-cgp/src/check/dylib.rs`](../../crates/cargo-cgp/src/check/dylib.rs) — the OS
  dynamic-library search path, shared by the check and the preflight's load test.
- [`crates/cargo-cgp/src/setup.rs`](../../crates/cargo-cgp/src/setup.rs) — installs the pinned
  toolchain with `rustup` and the driver with `cargo install`, co-located via `--root`.
- [`crates/cargo-cgp/src/update.rs`](../../crates/cargo-cgp/src/update.rs) — reads the crates.io sparse
  index (`ureq` + `serde_json`), picks the highest in-channel version, skips when not newer, else
  reinstalls and hands off to the new `setup`.
- [`crates/cargo-cgp-driver/src/version.rs`](../../crates/cargo-cgp-driver/src/version.rs) and
  [`run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — the `--version` query (answered only in
  non-wrapper mode), and [`build.rs`](../../crates/cargo-cgp-driver/build.rs), which bakes in
  `PINNED_TOOLCHAIN` and the `built_against_rustc` identity from the compiling `$RUSTC --version`.
