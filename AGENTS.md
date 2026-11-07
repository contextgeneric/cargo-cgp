# AGENTS.md

Read this file before working in this repository. The
[knowledge base's AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md)
carries the canonical wording of the universal rules. Change a universal rule there first, then
update this block to match.

## Universal rules

- Prefer the local sibling checkout when reading another project.
- Write a cross-project link as a GitHub URL on `main`.
- Name official release versions only, never a pre-release.
- Commit only when the user explicitly asks, on the current branch.
- Verify a claim against the source before writing it.
- Keep every view of the same truth in sync in the same change.
- Ask the user when a decision is ambiguous.
- Apply point-first writing to internal prose.

## Versions

- Document the tool as v0.1.0.
- Quote output that names the version with `0.1.0`.

## Before any task

- Invoke the `/cgp` skill when a task touches CGP constructs or their diagnostics.
- Read the [error catalog](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md)
  and the relevant reference and implementation documents, starting with
  [usage](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/reference/usage.md),
  [typed-resolution](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/typed-resolution/overview.md),
  the [driver](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/driver/program.md),
  and [testing](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/testing.md),
  plus troubleshooting, the error-code catalog, diagnostic reshapes, error processing, architecture, and
  rustc diagnostic internals when the task reaches them.
- Treat the Rust compiler and Clippy checkouts under `../external/` as read-only ground truth; cite
  them, never edit them, and never depend on them.
- Read the `cgp` source to understand CGP behavior; never modify it.

## Sibling projects

- Read [sibling-projects.md](sibling-projects.md) to find a sibling and the revision to read.
- Use the local checkout first; fetch the file from the repository when it is absent.
- Write a link to another project as a GitHub URL on `main`; keep a bare checkout path relative.
- Keep the code dependency one-way: `cargo-cgp` reads `cgp`, never the reverse.
- Check whether the skill still describes the tool correctly when a command, flag, or reshaped class
  changes.

## Toolchain

- Build with the pinned nightly in [rust-toolchain.toml](rust-toolchain.toml), which carries
  `rustc-dev` and `llvm-tools`.
- Gate the driver's binary crate and its library with `#![feature(rustc_private)]`.
- Run the tool against a project using the same pinned nightly, through `RUSTUP_TOOLCHAIN` when needed.
- Bump the pin deliberately, and fix the driver against the new `rustc_driver` API.

## Code organization

- Keep each module small and focused on one concept.
- Give a sub-directory a `mod.rs` that only re-exports its children.
- Keep the few well-known strings in `config` and pass them into the functions that use them.
- Do not put logic in a `bin` target; implement it as a library function and wrap it thinly.
- Split a function past about 80 lines, but keep a long `match` with one arm per variant whole.
- Keep the resolver's imports one-directional: `query`, `classify`, `walk`, then the anchors.
- Prefer pure functions that take their inputs as parameters and return data.
- Reach for a newtype, a small enum, or a function-pointer seam rather than dragging a compiler type
  into code that does not need it.
- Keep the front-end free of compiler linkage; only the driver links the compiler internals.
- Keep the two binaries in the same directory, since the front-end locates the driver as its sibling.
- Keep logic out of the `rustc_private` linkage in `cargo-cgp-error-processing`
  and `cargo-cgp-expand`.
- Add no environment variable, Cargo feature, `cfg`, or flag that exists only for a test.
- Keep tests out of `src/`; every test lives in the crate's `tests/` directory.
- Add a one-line `///` to every public item, and keep inline docs current.

## Tests and UI snapshots

- Add a UI fixture before changing the tool's output.
- Run the three passes: the tool run, the plain `cargo check` baseline, and the expand run.
- File a fixture under its error class, mirroring the knowledge base's error catalog.
- Record a fixture's output quality in its `//@quality:` header.
- Bless a new or changed fixture with the pinned toolchain, and review each diff.

## Improving error messages against real-world CGP code

Apply the detailed loop in
[cgp-knowledge-base/cargo-cgp/implementation/improving-errors.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/improving-errors.md).

- Reproduce the failure in the target project and read its uncommitted diff to learn the root cause.
- Treat the target project as read-only.
- Add a distilled, self-contained UI fixture before changing the tool.
- Verify the fixture improves and every other snapshot still passes before re-checking the project.
- Confirm the fix the message names in a `git worktree` placed beside the project, never in the
  project itself.
- Never hard-code a name, type, or module from an example project; generalize the resolver and the
  wording.
- Frame every explanation around core CGP, not around one project.
- Raise a fixture's quality header when its output clears the usability bar.

## Nix

- Run the local build through Nix from the target project's directory when the host has Nix.
- Prefer the local flake over the published one; the knowledge base's installation and usage documents
  hold the full setup.

## Checks before committing

- Run the build, format, clippy, docs, and test commands from [README.md](README.md) before committing.
- Format the UI suite's auxiliary sources directly, because their manifests are templates.
- Run the UI suite through `cargo test -p cargo-cgp-ui-tests` when only the fixtures change.

## Committing

- Never commit as a side effect, suggest a commit unprompted, or treat a request as standing
  authorization.
