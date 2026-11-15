# AGENTS.md: cargo-cgp

Follow these rules for every change to this repository. Read [README.md](README.md) for what the tool
does and its checks, and [sibling-projects.md](sibling-projects.md) for the related repositories.

## Universal rules

These rules hold in every CGP repository. The
[knowledge base's AGENTS.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/AGENTS.md)
holds their canonical wording: read it first, and change a rule there before updating this block:

- Learn CGP and its tools from the `/cgp` skill and the knowledge base; never restate them in this
  file.
- Prefer the local sibling checkout when reading another project.
- Write a cross-project link as a GitHub URL on `main`.
- Name official release versions only, never a pre-release.
- Commit only when the user explicitly asks, on the current branch.
- Verify a claim against the source before writing it.
- Keep every view of the same truth in sync in the same change.
- Ask the user when a decision is ambiguous.
- Apply point-first writing to internal prose, and natural writing to public-facing documents
  such as READMEs.

## Task guides

Each kind of task has a guide that holds its rules. Read the guide before the task:

- **Adding or changing a UI fixture:** [tests/README.md](tests/README.md#rules-for-the-fixtures).
- **Improving an error message against real-world CGP code:** the knowledge base's
  [improving-errors.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/improving-errors.md),
  as [Improving error messages](#improving-error-messages-against-real-world-cgp-code) says.
- **Writing a `cargo-cgp` document in the knowledge base:**
  [cargo-cgp/writing-guide.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/writing-guide.md).
- **Writing an error class or code entry:**
  [cgp/errors/writing-guide.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/writing-guide.md).

## Orient before any task

Read how the tool works and what CGP's errors are before changing either:

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

The tool reads `cgp` and the skill describes the tool, so each dependency runs one way:

- Keep the code dependency one-way: `cargo-cgp` reads `cgp`, never the reverse.
- Check whether the skill still describes the tool correctly when a command, flag, or reshaped class
  changes.

## Toolchain

The driver links the compiler, so it builds only on the nightly the repository pins:

- Build with the pinned nightly in [rust-toolchain.toml](rust-toolchain.toml), which carries
  `rustc-dev` and `llvm-tools`.
- Gate the driver's binary crate and its library with `#![feature(rustc_private)]`.
- Run the tool against a project using the same pinned nightly, through `RUSTUP_TOOLCHAIN` when needed.
- Bump the pin deliberately, and fix the driver against the new `rustc_driver` API.

## Code organization

Small modules, pure functions, and thin binaries keep most of the code testable without the compiler:

- Keep each module small and focused on one concept.
- Give a sub-directory a `mod.rs` that only re-exports its children.
- Keep the few well-known strings in `config` and pass them into the functions that use them.
- Do not put logic in a `bin` target; implement it as a library function and wrap it thinly.
- Split a function past about 80 lines, but keep a long `match` with one arm per variant whole.
- Keep the resolver's imports one-directional: `query`, `classify`, `walk`, then the anchors.
- Prefer pure functions that take their inputs as parameters and return data.
- Use a newtype, a small enum, or a function-pointer seam rather than dragging a compiler type into
  code that does not need it.
- Keep the front-end free of compiler linkage; only the driver links the compiler internals.
- Keep the two binaries in the same directory, since the front-end locates the driver as its sibling.
- Keep logic out of the `rustc_private` linkage in `cargo-cgp-error-processing`
  and `cargo-cgp-expand`.
- Add no environment variable, Cargo feature, `cfg`, or flag that exists only for a test.
- Keep tests out of `src/`; every test lives in the crate's `tests/` directory.
- Add a one-line `///` to every public item, and keep inline docs current.

## Tests

A UI fixture pins the output before the tool changes it, so every change to the output is a
reviewed snapshot diff:

- Add a UI fixture before changing the tool's output.

## Improving error messages against real-world CGP code

Follow the knowledge base's
[improving-errors.md](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/improving-errors.md)
when the user asks to improve the tool's output for a project; it holds the loop and its rules.

## Nix

The tool's local build runs where the host has Nix:

- Run the local build through Nix from the target project's directory when the host has Nix.
- Prefer the local flake over the published one; the knowledge base's installation and usage documents
  hold the full setup.

## Checks before committing

Every commit passes the checks CI runs:

- Run the build, format, clippy, docs, and test commands from [README.md](README.md#development)
  before committing.
