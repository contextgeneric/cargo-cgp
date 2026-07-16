# Usage

`cargo-cgp` today is a single command, `cargo cgp check`, that stands in for `cargo check` and
re-presents CGP wiring errors with their root cause first. You run it in a cargo project the way you
run `cargo check`, and its output streams to your terminal in the same form — the difference is in
which errors it surfaces and how it words them. This document covers running the check, reading its
output, using it from an editor, and the switches that change its behavior. For installing the tool,
see [Installation](installation.md).

## Running a check

Run the command from anywhere inside a cargo package or workspace that uses `cgp`:

```sh
cargo cgp check
```

Every argument after `check` is forwarded verbatim to `cargo check`, so the flags you already use
work unchanged — `cargo cgp check --workspace`, `cargo cgp check -p my-crate`, `cargo cgp check -v`.
The command also runs directly as `cargo-cgp check` when you have not installed it as a cargo
subcommand.

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

## Further reading

- [Installation](installation.md) — installing and updating the tool.
- [CGP error codes](../error-code.md) — the catalog of the `[CGP-Exxx]` codes in the output.
- [Distribution](../implementation/distribution.md) — the design behind the check's toolchain
  forcing, the isolated target directory, and the Rust Analyzer integration.
