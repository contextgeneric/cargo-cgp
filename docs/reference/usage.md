# Usage

`cargo-cgp` has two commands that read your code. `cargo cgp check` stands in for `cargo check` and
re-presents CGP wiring errors with their root cause first; `cargo cgp expand` shows the ordinary Rust
your CGP macros generate, with CGP's type-level constructs spelled the way you wrote them. You run
either in a cargo project the way you run `cargo check`. This document covers running the check,
reading its output, expanding a target, using the tool from an editor, and the switches that change
its behavior. For installing it, see [Installation](installation.md).

cargo-cgp is **optional**, and its job is developer-time readability. Its two reading commands compile
your code only to inspect it — `check` re-checks it under the pinned nightly solely to reshape the
diagnostics, and `expand` stops as soon as the macros are expanded — and there is no
`cargo cgp build`, `run`, or `test` (`setup` and `update` only provision the tool).
CGP is an ordinary library that builds on any **stable Rust ≥ 1.89**, so plain `cargo check`,
`cargo build`, `cargo run`, and `cargo test` all work on a CGP project unchanged. Use `cargo cgp check`
when you hit or expect a wiring error and want it readable; use plain `cargo check` when you do not;
and always build, run, and test with ordinary cargo.

## Running a check

Run the command from anywhere inside a cargo package or workspace that uses `cgp`:

```sh
cargo cgp check
```

Every argument after `check` is forwarded verbatim to `cargo check`, so the flags you already use
work unchanged — `cargo cgp check --workspace`, `cargo cgp check -p my-crate`, `cargo cgp check -v`.
The command also runs directly as `cargo-cgp check` when you have not installed it as a cargo
subcommand. For a summary of the available commands, run `cargo cgp --help` (or `cargo cgp` with no
subcommand at all, which prints the same overview).

cargo-cgp does not interpret those forwarded flags itself — it appends them to `cargo check` and lets
cargo own them. Three consequences follow. Every `cargo check` flag works exactly as it does under a
plain `cargo check`, since that is literally what runs. **cargo**, not cargo-cgp, validates them, so an
unknown flag produces cargo's own error (`error: unexpected argument '--nope' found`), not a cargo-cgp
message. And `cargo cgp check --help` prints `cargo check`'s own help and exits *without* running a
check — the flag is forwarded like any other, so what you see is cargo's flag list under a
`Usage: cargo check [OPTIONS]` banner, and the driver never runs. The one flag cargo-cgp inspects is
`--target-dir`, which it looks for only to decide whether to inject its own default (below); every
other flag it passes through untouched. (The pinned toolchain and the injected diagnostic flags in the
next paragraph are applied *around* this forwarding, not by altering the flags you pass.)

A check differs from a plain `cargo check` in three deliberate ways, all handled for you. It compiles
your workspace under the tool's own **pinned nightly** rather than your project's toolchain, so the
diagnostics are reproducible and the embedded compiler matches the driver; your project's own
toolchain is left untouched for its ordinary builds. It turns on the **next-generation trait solver**
(`-Znext-solver`), which is what surfaces the CGP dependency errors the default solver hides —
reporting the real missing bound, such as `HasField<Symbol!("name")>`, instead of stopping at a
generic "trait bounds were not satisfied". And it builds into an **isolated `target/cgp` directory**
rather than your project's `target/`, so a check never invalidates your normal build cache and vice
versa. Because these are diagnostic settings the tool chooses, `cargo cgp check` is a diagnostic pass
rather than a reproduction of your project's own `cargo check`, much as Clippy runs under its own
settings.

To send the check's artifacts somewhere other than `target/cgp`, pass `--target-dir` (or set
`CARGO_TARGET_DIR`); either takes precedence over the default.

## Expanding a target

`cargo cgp expand` prints the crate as the compiler sees it after macro expansion, which is how you
answer "what did that macro actually generate?" — for a wiring table you are unsure about, for a
provider whose bound you want to see, or to confirm what an error is telling you:

```sh
cargo cgp expand --lib          # expand the library target
cargo cgp expand --bin my-app   # expand one binary
cargo cgp expand -p my-crate --lib
```

Arguments are forwarded to `cargo rustc`, so target selection is cargo's own — and because it expands
exactly one target, cargo asks you to choose when a package has several. That is worth knowing before
your first run: a package with both a library and a binary needs `--lib` or `--bin <NAME>`, and without
one cargo declines with *"extra arguments to `rustc` can only be passed to one target"*. Expanding a
module of a library crate is therefore `cargo cgp expand --lib --item <path>`. The output goes to stdout, so
it pipes and redirects like any other program's:

```sh
cargo cgp expand --lib > expanded.rs
cargo cgp expand --lib | rg 'Symbol!'
```

**A whole crate's expansion is long, so `--item <path>` narrows it to one part.** The path is
`::`-separated, and what it selects depends on what it names:

```sh
cargo cgp expand --lib --item shapes             # a module: its contents
cargo cgp expand --lib --item shapes::Rectangle  # a type: its declaration and every impl for it
cargo cgp expand --lib --item AreaCalculator     # a trait: its definition and every impl of it
```

The path names a module or item **inside the crate being expanded**, and a leading `crate::` is
accepted, so `--item crate::contexts::app` and `--item contexts::app` are the same request. (`self::`
and a bare leading `::` work too.)

The trait form is usually what you want on CGP code, because a component's generated items *are*
impls: `--item AreaCalculator` gives the provider trait together with the blanket impls, the
`UseContext` impl, and each provider's impl of it. A type's form is the companion — `--item Rectangle`
shows the struct with its `HasField` impls and its wiring. If the path matches nothing you get an
error saying so, not a silent whole-crate expansion.

The filter is the one argument `expand` does not forward to cargo. A bare positional path — the way
`cargo-expand` takes it — is not accepted, because with everything else passed through untouched a bare
word cannot be told from the value of a cargo flag (`--bin my_module`).

Two things about the output are worth knowing. **Every macro is expanded**, not only CGP's, so
`#[derive(Debug)]` and `println!` appear in their generated form too — the CGP-specific part is that
CGP's own type-level constructs are resugared, so a field name reads `Symbol!("height")` rather than a
six-level `Chars` spine. And the `cgp::macro_prelude::` qualifier the macros emit is stripped for
readability, which means the output is meant to be *read* rather than compiled.

`expand` is not a check: the compilation stops once the crate is expanded, so no type analysis runs and
no CGP diagnostic is produced. A malformed macro invocation still fails — that happens during
expansion — but a wiring mistake does not. Use `check` for that.

## Reading the output

The output is ordinary rustc/cargo diagnostics, with the errors `cargo-cgp` recognizes rewritten into
a clearer form. When the tool rewrites an error into a known CGP class, it stamps the message with a
short code in square brackets:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
```

The `[CGP-Exxx]` code names one class of CGP mistake — what it means and how to fix it — and is
looked up in the [CGP error-code catalog](../error-code.md). The diagnostic's own Rust code
(`E0277` here) is always kept, so `rustc --explain` still works and nothing is reclassified away from
rustc; the CGP code rides inside the message as a tag on the sentence it classifies. Errors the tool
does not recognize pass through as the compiler wrote them.

## Editor integration (Rust Analyzer)

Rust Analyzer can run `cargo cgp check` as its on-save check backend. Because the command is two
words and must emit JSON, wire it through `check.overrideCommand` (not `check.command`) with
`--message-format=json`:

```jsonc
"rust-analyzer.check.overrideCommand": [
  "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
]
```

The tool renders the transformed diagnostics as rustc JSON, which cargo wraps and the editor parses,
so the CGP transforms appear inline in the editor's diagnostics. The isolated `target/cgp` directory
matters here too: it keeps the editor's check from contending with your normal builds. The full
integration notes, including why Rust Analyzer's own wrapper does not collide with the tool's, are in
[Rust Analyzer integration](../implementation/distribution.md#rust-analyzer-integration).

## Running on a project outside this repository

To exercise the tool on CGP source in another location, run it through Nix, and **prefer a local
`cargo-cgp` checkout whenever one is available** — you are usually working inside this repository or
beside it, and the local build reflects the current code, including any uncommitted changes, which a
freshly fetched release would not. Point the flake reference at the local checkout and run its
default app from the target project's directory:

```sh
cd /path/to/the/target/project                 # a cargo package/workspace that uses `cgp`
nix run /path/to/the/local/cargo-cgp -- check   # local checkout — reflects current code
```

Only when no local checkout is available, fall back to the published flake:

```sh
nix run github:contextgeneric/cargo-cgp -- check
```

Either way this builds (or reuses a cached) `cargo-cgp` and `cargo-cgp-driver` under the pinned
nightly and runs `cargo cgp check` in the current directory, with everything after `--` forwarded to
the check as usual. Running through the flake is what makes it work from *any* directory: the flake
wraps the front-end to force the pinned nightly and run unmanaged, so it needs no rustup and leaves
the target project's own toolchain and `target/` untouched.

When the local binaries are already built (`cargo build` in the checkout) *and* the pinned nightly is
the active toolchain — as it is inside the `cargo-cgp` workspace itself — the built binaries can be
driven directly with the environment overrides described under
[Installing from source](installation.md#installing-from-source), skipping the flake build. That form
relies on the ambient toolchain matching the driver's embedded compiler, which the flake otherwise
guarantees, so reach for it only when that match holds. See
[Installation](installation.md#installing-with-nix) for the other Nix entry points.

## Environment overrides

A few environment variables change how the front-end behaves, for local development and unusual
setups. They are summarized here; the full contract is in
[Distribution](../implementation/distribution.md#escape-hatches-for-local-development).

- `CARGO_CGP_NO_MANAGE` — when set, skip the preflight and the toolchain forcing, and trust whatever
  driver and toolchain the environment already provides. Used when running a source or Nix build that
  is not provisioned through rustup.
- `CARGO_CGP_DRIVER` — an explicit path to the driver executable, bypassing the sibling lookup. Point
  it at a freshly built `target/debug/cargo-cgp-driver`.
- `CARGO_CGP_TOOLCHAIN` — override the pinned nightly at runtime, for testing a toolchain bump;
  normally paired with `CARGO_CGP_NO_MANAGE`.
- `CARGO_TARGET_DIR` / `--target-dir` — choose the check's target directory instead of the default
  `target/cgp`.

## Calling the driver directly (debugging)

You normally never invoke `cargo-cgp-driver` yourself — the front-end wires it in as cargo's rustc
wrapper, and cargo calls it once per workspace crate. But when `cargo cgp check` misbehaves, calling
the driver directly is how you tell a front-end wiring problem apart from a driver or compiler one,
because it takes cargo and the front-end out of the loop.

Start with the version query, which doubles as a load test:

```sh
cargo-cgp-driver --version   # or -V
```

The driver links `librustc_driver` dynamically and loads it before printing, so this is the quickest
confirmation that the binary can run at all. (`cargo-cgp-driver --help`, `-h`, or a bare invocation
with no arguments prints a short description of the driver and these same flags instead.) On success
`--version` prints three lines — its own version, the `pinned-toolchain:` it targets, and the
`built-against-rustc:` compiler it was actually built with:

```text
cargo-cgp-driver 0.1.0-alpha
pinned-toolchain: nightly-2026-07-16
built-against-rustc: rustc 1.99.0-nightly (d0babd8b6 2026-07-15)
```

A failure *before* that output — typically `error while loading shared libraries:
librustc_driver-<hash>.so: cannot open shared object file` — means the loader cannot find the
compiler library, either because the dynamic-library path is not set or because the driver was built
against a different nightly than the one installed. A Nix-built driver has that path baked into its
wrapper, so `--version` works as-is; a from-source driver needs the pinned toolchain's `lib` directory
on the loader path, which is what the front-end normally sets for it (`DYLD_FALLBACK_LIBRARY_PATH` on
macOS, `LD_LIBRARY_PATH` elsewhere):

```sh
SYSROOT=$(rustc --print sysroot)                        # run under the pinned toolchain
LD_LIBRARY_PATH=$SYSROOT/lib cargo-cgp-driver --version
```

When the driver comes from the [Nix flake](installation.md#installing-with-nix), run it through the
flake instead, which needs no library-path setup at all — the flake bakes that path into the driver's
wrapper. Bring the tool onto `PATH` in a throwaway shell (preferring a local checkout, as everywhere)
and call the driver there:

```sh
nix shell /path/to/the/local/cargo-cgp -c cargo-cgp-driver --version
```

or build the package once and run the binary out of the result:

```sh
nix build /path/to/the/local/cargo-cgp   # then:
./result/bin/cargo-cgp-driver --version
```

There is no `nix run` app for the driver — it is a second binary of the same package as the front-end,
so it is reached through `nix shell` or the built `result/bin`, not `nix run …#cargo-cgp-driver`. The
baked-in library path covers the load test above; to replay a full *compilation* with the Nix driver
you still supply the sysroot (`CARGO_CGP_SYSROOT`) as in the reproduction below, and for that case
running the whole check through the flake (`nix run … -- check -v`, above) is usually easier.

To debug an actual compilation, reproduce the exact command cargo hands the driver rather than
building one by hand. Run the failing check verbosely and cargo prints each driver invocation in full:

```sh
cargo cgp check -v
```

Each `Running …` line — showing a full `cargo-cgp-driver … rustc --crate-name …` command — is a
complete, replayable invocation. (If none prints, the crate was cached; touch a source file or clean `target/cgp` to force a
recompile.) Copy one and run it directly, with the two environment values the front-end passes
reconstructed — the sysroot, which the driver reads from `CARGO_CGP_SYSROOT` to inject `--sysroot`,
and the library path above:

```sh
SYSROOT=$(rustc --print sysroot)
LD_LIBRARY_PATH=$SYSROOT/lib CARGO_CGP_SYSROOT=$SYSROOT \
  cargo-cgp-driver /path/to/rustc --crate-name … <the rest of the printed args>
```

Run this way the driver behaves exactly as it does under cargo — it drops the leading `rustc` path
(that leading path is what puts it in "wrapper mode"), injects `--sysroot`, `-Znext-solver=globally`,
and `--verbose`, then runs the real compiler in-process — but now in isolation, where you can add
`RUST_BACKTRACE=1`, extra `-Z` flags, or a debugger to watch the transform. Dropping the leading
`rustc` path instead runs the driver in its *direct* mode, the mode `--version` uses; the
wrapper-mode form above is the one to copy from `-v` output. How the driver prepares its argument
vector and reaches the compiler is documented in
[The driver](../implementation/driver.md#preparing-the-argument-vector).

## Further reading

- [Installation](installation.md) — installing and updating the tool.
- [CGP error codes](../error-code.md) — the catalog of the `[CGP-Exxx]` codes in the output.
- [Distribution](../implementation/distribution.md) — the design behind the check's toolchain
  forcing, the isolated target directory, and the Rust Analyzer integration.
- [The driver](../implementation/driver.md) — how the driver wraps rustc and transforms diagnostics,
  for when direct invocation surfaces a driver-side problem.
