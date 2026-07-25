# AGENTS.md

This file guides LLM agents working in the `cargo-cgp` repository. Read it before any task here,
then read the sub-crate `AGENTS.md` for whichever crate you are about to touch.

## What this project is

`cargo-cgp` is a cargo subcommand that makes Context-Generic Programming (CGP) compiler errors
readable. A CGP macro expands to ordinary Rust, so many mistakes are caught not by the macro but by
the compiler type-checking the generated code, where a single error can cascade across generated
types the programmer never wrote and the root cause is often buried or suppressed entirely.
`cargo-cgp` post-processes those diagnostics into a compact, root-cause-first form, the way Clippy
layers analysis on top of `rustc`.

The main diagnostic command is `cargo cgp check` (`expand` is the second reading command, showing the
Rust a target's CGP macros generate; `setup` and `update` only provision the tool). It compiles the workspace through a `rustc_driver`-based wrapper that does two
things. First, it injects `-Znext-solver=globally`, turning on the next-generation trait solver,
which surfaces the CGP dependency errors the default solver hides (it descends to the real missing
bound — e.g. `HasField<Symbol!("name")>` — instead of stopping at the provider trait). Second, its
in-process emitter recognizes the CGP error classes and rewrites them: it leads with the root cause,
renders the transitive dependency chain as a `cargo tree`-style tree, and stamps each rewritten
message with a `[CGP-Exxx]` code (keeping rustc's own code alongside it). The project is an early
pre-release: it reshapes the core wiring-error classes well and still passes some classes through
largely as `rustc` wrote them — a shrinking gap tracked in [docs/issues/](docs/issues/README.md).
When reasoning about behaviour, remember the tool diverges from plain `cargo check` both by its choice
of trait solver and by rewriting the diagnostics it recognizes.

## The two projects: cgp and cargo-cgp

`cargo-cgp` and `cgp` are developed together but live in separate Git repositories, so a task in one
routinely reads — and sometimes updates — the other. `cargo-cgp` is CGP's **first-class toolchain**:
it rewrites the compiler's diagnostics into readable, root-cause-first CGP errors, and it is the
recommended way to build and check any CGP code. `cgp`'s own docs and skill point users at it, so
treat it as the default tool a CGP developer reaches for.

**Finding `cgp`.** Look first in the parent directory, at `../cgp`. When that sibling checkout is
present, use it — it is the fastest reference and reflects any uncommitted work. When it is absent,
fall back to the official repository at
[github.com/contextgeneric/cgp](https://github.com/contextgeneric/cgp), fetching the file you need
from there. (The mirror image applies to an agent working in `cgp` and looking for `cargo-cgp` at
`../cargo-cgp`, else [github.com/contextgeneric/cargo-cgp](https://github.com/contextgeneric/cargo-cgp).)

**Writing cross-project links.** A *link* from a committed file here to a document in `cgp` — a
Markdown link or a URL — is **always written as a GitHub URL**
(`https://github.com/contextgeneric/cgp/blob/main/<path>`), never as a relative `../cgp/...` link, so
the document resolves for a reader who has only this repository checked out. When *reading* such a
link yourself, prefer the local `../cgp` checkout if it is available and fetch the GitHub URL only
when it is not. (A bare mention of the checkout's location, like the path `../cgp`, is a filesystem
reference rather than a link and stays relative.)

**Keeping the two in sync.** When a change here affects `cgp`'s documentation — its error catalog,
the `cargo-cgp` reference doc, the CGP skill, or a fixture the catalog cites — and `cgp` is checked
out at `../cgp`, update it in the same change, following `cgp`'s own conventions (its `AGENTS.md` and
doc-authoring rules). When `../cgp` is not checked out, state plainly what in `cgp` needs updating so
it is not silently left to drift.

**The code dependency stays one-way.** No `cgp` *crate* may depend on a `cargo-cgp` crate:
`cargo-cgp` reads `cgp` (its source, its error catalog, its UI-fixture inputs), never the reverse.
Documentation is the deliberate exception — `cgp`'s prose references `cargo-cgp` as the recommended
toolchain — so it is the *code* graph that stays acyclic, not the documentation.

## Orient before any task

CGP is the subject matter, so load the CGP mental model before reasoning about what this tool
should do to an error. **Invoke the `/cgp` skill** whenever a task touches CGP constructs or the
diagnostics they produce; the skill is the ground truth for what each macro expands to and why its
errors look the way they do.

The `cgp` source is a sibling of this repository (see
[The two projects](#the-two-projects-cgp-and-cargo-cgp) above for finding it, linking to it, and the
sync rule). Read anything under `../cgp` to understand CGP behaviour, and in particular the
**[CGP error catalog](https://github.com/contextgeneric/cgp/blob/main/docs/errors/README.md)** is the
map of every error class this tool recognizes or should: which classes hide the root cause, which
surface it, and where the cause sits when it is present. Each class is backed by a UI fixture in
*this* repository's [`tests/ui/`](tests/README.md) — the post-codegen compile-fail cases were migrated
here out of `cgp`, so a fixture is a concrete input you run through `cargo cgp check` to see a class
in the raw, and each `cgp` class doc links the fixture that pins it. The **code** dependency stays
one-way — `cargo-cgp` reads `cgp`, never the reverse — but `cgp`'s *documentation* now references this
tool as the recommended toolchain, so editing `cgp`'s prose (never a `cgp` crate) to match a change
here is expected under the sync rule.

Two more read-only references sit alongside this repository, and you should use them whenever a task
turns on how the compiler actually behaves rather than on how it is documented to behave. The **Rust
compiler source is at [`../external/rust`](../external/rust)** — consult it to confirm the real
signature and behaviour of a `rustc_driver`/`rustc_interface` API before relying on it, since these
internals are unstable and change between nightlies; grep `compiler/rustc_driver_impl/src` for the
entrypoints this project calls. **Clippy's source is at
[`../external/rust-clippy`](../external/rust-clippy)** — it is the closest working example of the
integration this project performs, so read its `src/main.rs` (the `cargo-clippy` front-end) and
`src/driver.rs` (the `clippy-driver` wrapper) when working out how to wire the driver into cargo,
inject the sysroot, or install callbacks. Prefer verifying against these sources over guessing, but
do not edit them and do not create a dependency on them.

This repository keeps its own knowledge base under [`docs/`](docs/README.md), written by and for
agents. Read it before a task and keep it in sync after one. The
[implementation documentation](docs/implementation/README.md) explains how the tool is built and why
— start with [Executable structure](docs/implementation/executable-structure.md) for the
two-executable design and the cargo wrapping, and [The driver](docs/implementation/driver.md) for the
rustc wrapping, the compiler-API access, and the driver-side diagnostic transformations. The knowledge base
is bound by a **synchronization rule** ([docs/AGENTS.md](docs/AGENTS.md)): the source is the single
source of truth, and when you change how the tool is structured or behaves, you revise the matching
document in the same change. A document describing a design the code no longer has is worse than
none.

When your task involves editing markdown documentation or inline doc comments, **load the
`/dual-reader-prose` skill** and follow its convention for the prose you write. The knowledge base's
own authoring conventions — including showing the example code behind any error message a document
discusses, and keeping markdown backticks well formed (inline spans opened and closed on one line,
fenced blocks delimited by blank-line-separated triple-backtick lines, re-checked after every edit) —
live in [docs/AGENTS.md](docs/AGENTS.md) and apply to inline doc comments too.

## Architecture: two binaries, like Clippy

`cargo-cgp` mirrors Clippy's split into a front-end and a driver, and understanding that split is
the key to the whole codebase. The **`cargo-cgp` crate** (`crates/cargo-cgp`) is the front-end: the
cargo subcommand a user invokes, a plain `std` + `anyhow` binary that runs `cargo check` with
`RUSTC_WORKSPACE_WRAPPER` set to the driver and lets cargo's output stream through untouched. The
**`cargo-cgp-driver` crate** (`crates/cargo-cgp-driver`) is the driver: the `rustc` replacement cargo
then calls for each workspace crate, running the real compiler in-process through `rustc_driver`, and
it is where every diagnostic transform now happens. They are separate crates for one concrete
reason — only the driver links the compiler's internal libraries, and keeping that linkage out of the
front-end keeps it a small, ordinary binary that builds without loading LLVM. A third, library-only
crate, **`cargo-cgp-error-processing`** (`crates/cargo-cgp-error-processing`), holds the rustc-free
string-level diagnostic helpers the driver drives — the wiring-message rewrite, the fallback
post-processing text transforms, the rustc-free root-cause model and the diagnostic-plan wording
that turns it into text, the dependency-tree renderer, the cross-diagnostic de-duplication ledger,
and the text signals the emitter's candidate checks key on; it links no compiler internals
either, so it builds and tests on any toolchain (see
[Error processing](docs/implementation/error-processing.md)). A fourth, library-only crate,
**`cargo-cgp-expand`** (`crates/cargo-cgp-expand`), is the rustc-free half of `cargo cgp expand`: it
takes the source the compiler printed for an expanded crate and folds CGP's type-level spines back to
the surface macros (`Symbol!`, `Path!`, `Product!`/`Sum!`), and it links no compiler internals either
(see [Resugaring](docs/implementation/resugaring.md) and
[The expand command](docs/implementation/expand-command.md)).

How the two cooperate — the argument normalization, the `CARGO_CGP_SYSROOT` and
dynamic-library-path contract, wrapper-mode detection, and the front-end's plain forwarding of
cargo's output — is documented in
[Executable structure](docs/implementation/executable-structure.md); the driver's own internals — the
argument preparation, the `rustc_private` compiler-API access, and the diagnostic transformations (the
`-Znext-solver=globally` and `--verbose` flag injections, the generic `CgpEmitter` that renders text
or JSON like vanilla `rustc`, its typed root-cause resolution, and its wiring-note rename plus
post-processing fallback) — are in [The driver](docs/implementation/driver.md). Read the relevant one
before changing how the executables interact or what the driver does, and keep it in sync when you do.

## Toolchain and `rustc_private`

The driver links the compiler's unstable internal crates, so it can only be built with a nightly
toolchain that carries the `rustc-dev` component. That toolchain is pinned in
[`rust-toolchain.toml`](rust-toolchain.toml) to an exact dated nightly, because the `rustc_private`
API changes between nightlies; the `rustc-dev` and `llvm-tools` components install automatically on
first build. Bump the pin deliberately, and expect to fix driver code against `rustc_driver` API
changes when you do.

Two `rustc_private` consequences are easy to trip over: the `#![feature(rustc_private)]` gate is
needed on the driver's **binary** crate as well as its library (the binary is what links the compiler
dylib), and the pinned nightly is the compiler the driver *embeds*, so the tool must be run against a
project using that same nightly — for example with `RUSTUP_TOOLCHAIN` set — or it loads a mismatched
`librustc_driver` and fails.
[The driver](docs/implementation/driver.md#accessing-the-rust-compiler-api)
explains both in full.

## Code organization conventions

The code organization follows the same conventions as the `cgp` workspace, especially
`../cgp/crates/macros/cgp-macro-core`. Hold to them when adding code.

Keep each module small and focused on one concept, and give sub-directories a `mod.rs` that contains
nothing but re-exports of its child modules (`mod child; pub use child::*;`). A top-level `lib.rs`
declares its modules with `pub mod`. Keep code loosely coupled by passing values as parameters
rather than reaching for hardcoded constants: the few unavoidable well-known strings (the cargo
subcommand name, the driver executable name, the sysroot environment variable) live in a `config`
module and are passed into the functions that use them, so call sites stay independent of the
literals. Do not put logic in a `bin` target; implement everything as library functions and let the
`bin` file be a lightweight wrapper around the entrypoint (the one exception is the driver binary's
`#![feature(rustc_private)]` gate, which must be on the binary crate for linking).

Give each module one construct — a single type with its inherent impls, or one function — or a small
group of closely-related utility functions, and no more. A module that accumulates several constructs
is a directory waiting to happen: split it into one file per construct under a sub-directory whose
`mod.rs` re-exports them, rather than letting one file grow. The driver's `emitter/` and `resolve/`,
and the helper library's `rewrite/`, `postprocess/`, and `diagnosis/`, are the worked examples — each
was one large file before it earned a directory.

Prefer plain, side-effect-free functions that take their inputs as parameters and return data. A
function that *computes* a value — a rewritten message, a diagnostic plan, a dependency tree — is
reachable and pinnable by a unit test through the crate's public API, so keep the side effects
(launching a process, reading the environment, mutating rustc's `DiagInner`) at the thin edges that
call those functions. The diagnostic wording is the pattern to follow: the driver's emitter mutates a
`DiagInner`, but only from the strings a pure `plan_resolved` returns, and that planning is tested on
hand-built inputs with no compiler in the loop.

Keep as much logic as possible out of the `rustc_private` linkage, in the rustc-free
`cargo-cgp-error-processing` crate, so it builds and its tests run on any toolchain. When logic is
entangled with the compiler, look for a plain-data boundary that lets the rustc-free half move
across: the driver reads compiler state into an owned, `String`-only model (`Resolved`,
`DependencyTree`), and everything downstream of that model — the wording, the plan, the tree
rendering — lives in the helper crate. Reach for a newtype, a small enum, or a `fn`-pointer seam (as
`ComponentNameMap` and `DiagKind` do) rather than dragging a compiler type into code that does not
truly need it. When you move code across the boundary, add the unit test that its new rustc-free home
makes possible.

Keep tests out of `src/`. Do not write inline `#[cfg(test)] mod tests` blocks; every test lives in
the crate's `tests/` directory alongside `src/`, as an integration test exercising the crate's
public API. This keeps source files focused on the code, and it means a function worth testing is
reached through the crate's public surface rather than through module-private access — expose it (or
test it via a public entry point) instead of reaching inside. The custom UI harness is consistent
with this: its `harness = false` target and its plain tests both live under
`crates/cargo-cgp-ui-tests/tests/`.

Keep inline docs brief and current as you write. Add a one-line `///` to any public item that lacks
one, prefer naming the *why* or a corner case over restating the signature, and delete a comment
that only repeats the code.

### Module map

The front-end (`crates/cargo-cgp/src`) is organized around dispatch and four subcommands.
`run.rs` is the entrypoint that normalizes arguments and dispatches on the subcommand (`check`,
`expand`, `setup`, `update`), printing `help.rs`'s help text for a leading `--help`/`-h` or no
subcommand; `args.rs` strips the cargo-inserted `cgp` token so the same entrypoint serves both
`cargo cgp check` and a direct `cargo-cgp check`; `config.rs` holds the shared well-known names,
including the build-time-baked `PINNED_TOOLCHAIN` (from `build.rs`), the management environment
variables, and the `EXPAND_FLAG` marker; `toolchain.rs` resolves the effective pinned toolchain and
queries its `rustc`. The `launch/` directory holds what the two reading commands share:
`command.rs` runs the [preflight](docs/implementation/distribution.md) and forces `RUSTUP_TOOLCHAIN` to
the pinned nightly (unless `CARGO_CGP_NO_MANAGE` is set), then builds the wrapped cargo command with
the driver wired in as the workspace rustc wrapper and an isolated `target/cgp` directory
(`target_dir.rs`); `driver_path.rs` locates the driver (via the `CARGO_CGP_DRIVER` override or as a
sibling); `sysroot.rs` discovers the toolchain sysroot; `dylib.rs` builds the dynamic-library search
path; and `preflight.rs` verifies a matching driver and the pinned toolchain, directing the user to
`cargo cgp setup` on any failure. `check.rs` then just runs that command, inheriting cargo's stdio so
its output streams through untouched (the driver does every diagnostic transform, so the front-end
captures and processes nothing). The `expand/` directory holds the expand command: `command.rs` runs
the wrapped `cargo rustc` with the marker flag that puts the driver in expand mode and prints what the
driver wrote, over `output.rs` (the per-process output file) and `profile.rs` (the default
`--profile check`). `setup.rs` and `update.rs` are the provisioning and upgrade subcommands. The distribution design — the pinned-toolchain forcing, the preflight version handshake,
and the `setup`/`update` flow — is documented in
[Distribution](docs/implementation/distribution.md).

The driver (`crates/cargo-cgp-driver/src`) is organized around the compiler wrapping and the
diagnostic transform. `run.rs` is the entrypoint that runs the compiler through `rustc_driver`,
answering a direct (non-wrapper) `--help`/`-h`/no-args from `help.rs` and `--version`/`-V` from
`version.rs` before compiling; `args.rs` turns the wrapper's process arguments into a rustc argument
vector (dropping the injected `rustc` path and injecting `--sysroot`); `callbacks.rs` holds the
`Callbacks` implementation, whose `config` hook installs the diagnostic-transforming emitter and whose
`after_expansion` hook serves expand mode; `config.rs` holds the shared names and the injected flags.
The `expand/` directory is expand mode: `request.rs` takes the front-end's `--cgp-expand=<path>` marker
out of the argument vector, and `print.rs` prints the expanded crate through the compiler's own
`pprust::print_crate`, resugars it through `cargo-cgp-expand`, and writes it out — the whole crate is
expanded, only CGP's constructs are resugared (see
[The expand command](docs/implementation/expand-command.md)). The transform is split across two directories. `emitter/` is the `CgpEmitter<E>`
seam: `install.rs` rebuilds whichever inner emitter the compiler's default would build — a
`JsonEmitter` or an `AnnotateSnippetEmitter` — and wraps it, `cgp_emitter.rs` is the wrapper type and
its `emit_diagnostic` orchestration (first recognize a `#[cgp_impl]` provider naming the wrong trait
in its header or an inner-provider bound — its `E0107` reshaped into a
`[CGP-E013]`/`[CGP-E014]`/`[CGP-E015]` header and the rest of its cascade suppressed —
or a duplicate-key `E0119` conflict — suppressing
the redundant `IsProviderFor` half, rewriting the wiring half — or an orphan-rule `E0210`/`E0117`
namespace registration, reworded into its `[CGP-E011]` header, or a capability used in a
`#[cgp_fn]`/`#[cgp_impl]` body without a `#[uses(…)]` declaration, reworded into its `[CGP-E012]`
header, or a higher-order provider calling an inner provider it never imported with `#[use_provider]`,
reworded into its `[CGP-E016]` header; else try the typed
resolver, else the text rewrite with its method-probe-advice and overflow cleanups; then always
post-process; then cross-diagnostic de-duplicate through the rustc-free `DedupLedger` — drop a
transformed diagnostic whose span-independent signature was already emitted, so one mistake re-reported
at many wiring sites is shown once; then, rather than emit, hold every diagnostic in an arrival-ordered
buffer flushed from `Drop` — the only point after all diagnostics have arrived — where consumer
failures that *share a root cause* are grouped by `group_by_shared_cause` and coalesce into one
`[CGP-E001]` headline listing every affected consumer, the trailing `rustc --explain` footer is
rebuilt to name only codes still shown, and everything else emits verbatim in place), and `edit.rs` holds the
`DiagInner`-editing helpers. `resolve/` is the typed root-cause resolver, one sub-directory per
stage: `anchor/` recovers the
failing obligation from a `check_components!` entry, a hand-written `impl Trait for Context` block
the failure surfaces inside (reconstructing the obligation from the impl's CGP consumer supertrait,
concrete parameters preserved), a hand-written `impl Trait for Foreign` block whose `Self` is a
foreign wrapper holding the context (descending its supertrait's `where`-clause hops — through a
projection bound's base trait — to a CGP consumer on the context, the routing-glue case), or the use
site of a broken consumer-method call (`E0599`), with `call_site/` the last-resort re-read of the
failing call expression itself; `walk/` descends the wiring to each terminal leaf, `classify/` turns
a leaf into the
rustc-free model by inspecting the struct it lands on, `label/` renders each path predicate as a
tree label, `conflict/` classifies a duplicate-key `E0119` by reading the two conflicting wiring
impls off the compiler (which keys collide, whether either is a `RedirectLookup`), `orphan.rs`
classifies an orphan-rule `E0210`/`E0117` by reading the offending namespace-registration impl off
the compiler (which foreign namespace and key it names, whether a re-open or a registration),
`undeclared.rs` recognizes a `#[uses(…)]`-undeclared capability call (its `[CGP-E012]` reshape),
`cgp_impl_misuse.rs` recognizes a `#[cgp_impl]` provider naming the wrong trait — a consumer trait or a
non-component in its header, or a consumer trait in an inner-provider bound — off the consumer/provider
fingerprints (its `[CGP-E013]`/`[CGP-E014]`/`[CGP-E015]` reshape), `missing_use_provider.rs` recognizes
a higher-order provider calling an inner provider it never imported with `#[use_provider]` (its
`[CGP-E016]` reshape),
and `cgp_item.rs` holds the DefId-anchored CGP-trait recognition every stage relies on (see
[Typed root-cause resolution](docs/implementation/typed-root-cause-resolution.md) and its per-stage
sub-documents). `cache.rs` memoizes the walk at every node — keyed on the region-erased obligation and context —
so a mistake re-reported at many sites, or a capability depended on by many providers, is resolved
once (see [Cached dependency resolution](docs/implementation/cached-dependency-resolution.md)).
`component_map.rs` builds the component-marker → consumer/provider trait-name map by querying the
trait solver. Everything downstream of the resolver's rustc-free `Resolved` model — the
header/note/help wording, the tree-label templates, the diagnostic plan, the wiring rewrite, the
post-processing transforms, the de-duplication ledger, the text signals,
and the lazily-built `ComponentNameMap` — lives in the `cargo-cgp-error-processing` crate (the
driver's one ordinary dependency), so it is unit-tested without the driver's `rustc_private` linkage.

The helper library (`crates/cargo-cgp-error-processing/src`) is the rustc-free half, holding no
compiler linkage and driven entirely by the driver's emitter (the front-end no longer touches
diagnostics), so its logic is unit-tested on any toolchain. `postprocess/` holds the fallback text
transforms the driver applies to a diagnostic's messages — one module per transform (stripping CGP
path prefixes, resugaring `Symbol!` and `Path!`, resugaring `Product!`/`Sum!` lists and their
`Struct!`/`Enum!` record/variant forms, rewriting unmet `HasField` bounds into missing-field
messages), each a pure `&str -> Option<String>` function — plus `chain.rs` for the
`postprocess_message` chain, with `mod.rs` re-exporting only. `rewrite/` is the wiring-message
rewrite: `message.rs` (the note, header, and wiring-overflow rewrites), `names.rs` (the
`ComponentTraitNames` and the
lazily-built `ComponentNameMap`), `parse.rs` (the trait-bound parse), and `text.rs` (the shared
splitting utilities). `diagnosis/` is the rustc-free root-cause model and its wording:
`leaf.rs`/`resolved.rs` (the `Leaf`/`FieldIssue`/`Cause`/`Resolved` types the driver's resolver fills
in, plus `Causes` — the cause *set*, which enforces the one-cause-per-distinct-leaf invariant by
construction through `from_sub_chains`/`union`/`headed_by` over a private field, where it used to be
re-established by hand at each of the five sites that build a `Resolved`), `wording/` (the pure `Resolved`-to-text builders — headers, root-cause leads and their codes,
notes, derive helps, and the de-duplication `cause_signature` — textual, since the ledger shares a key
space with rendered-message keys), `group.rs` (`group_by_shared_cause`, partitioning the coalescible
failures into the connected components of the shares-a-cause relation — keyed on each context-scoped
`Leaf` *structurally*, so grouping never rides on how a lead is worded — which is what keeps one
mistake in one block when the depths it surfaces at each reach a different *subset* of its causes),
`coalesce.rs`
(`coalesce_underived_fields`, merging several underived fields on one struct into one heading cause),
`node.rs`/`graph.rs` (the structured `DepNode`/`ChainNode` chain nodes and the `DependencyGraph` that
merges every cause's paths into a DAG — shared nodes fused, a reuse `(*)`-referenced — and renders it,
see [Dependency-graph rendering](docs/implementation/dependency-graph-rendering.md)), `plan.rs`
(`plan_resolved`, which words
a `Resolved` into the header, help, and note strings the emitter emits, and holds the
`categorized_header` classification), `wiring.rs` (the `WiringConflict` model and
`plan_wiring_conflict`, which words a duplicate-key conflict into its `[CGP-E004]`–`[CGP-E008]` header, one code per conflict shape), and `orphan.rs` (the `OrphanConflict` model and `plan_orphan_conflict`/`orphan_conflict_help`, which word an orphan-rule namespace registration into its `[CGP-E011]` header and ownership fix). `tree.rs` is the `DependencyTree` type and its `cargo tree`-style
renderer (over the `termtree` crate), the target the graph expands into; `dedup.rs` is the `DedupLedger` behind the
emitter's cross-diagnostic de-duplication; `signals.rs` holds the stable rustc phrasings the
emitter's candidate checks and cleanups key on; and `code.rs` holds the `CGP-E` error-code constants stamped on
classified main messages (catalogued in docs/error-code.md). Its tests in `tests/` drive the
post-processors, the rewrite, the diagnosis plan and wording, the coalescing, the text signals, the
de-duplication ledger, the tree renderer, and the dependency-graph build-and-render (`insta` inline
snapshots in `tests/graph.rs`) over hand-built inputs, so they run on any toolchain.

## Commands

The commands mirror the `cgp` workspace. Run them from the repository root.

- **Build:** `cargo build` builds both binaries into `target/debug`. They must stay in the same
  directory, since the front-end locates the driver as its sibling.
- **Format:** `cargo +nightly fmt --all` (check with `-- --check`). The `.rustfmt.toml` uses the
  same unstable `group_imports`/`imports_granularity` settings as `cgp`, so formatting needs
  nightly.
- **Lint:** `cargo clippy --all-targets -- -D warnings`.
- **Test:** `cargo test` runs everything — the argument-handling tests in both tool crates, the
  helper library's transform tests (which run on any toolchain, needing no compiler), and the UI
  snapshot suite (below), which builds the driver and expects a sibling `cgp` checkout at `../cgp`.
  Every test lives in its crate's `tests/` directory (no inline `#[cfg(test)]`); prefer adding
  coverage there over ad-hoc checks.

### UI snapshot tests and running the tool

The UI suite is a custom Rust test harness modeled on Clippy's `compile-test`: the
[`cargo-cgp-ui-tests`](crates/cargo-cgp-ui-tests) crate has a `harness = false` test with its own
`fn main` that checks each fixture under [`tests/ui/`](tests/README.md) through two passes. The tool
pass runs `cargo-cgp` and diffs its stderr against `<name>.cgp.stderr`; the baseline pass runs plain
`cargo check` and diffs its stderr against `<name>.rust.stderr`, recording the untransformed "before"
so the diff against `.cgp.stderr` shows what the tool changes. Because the driver now renders the
diagnostics in-process, `<name>.cgp.stderr` is simply what `cargo-cgp` prints — there is no captured
JSON or separate processing pass to keep in sync. The crate is a full workspace member, so
`cargo test` runs the whole suite alongside the argument tests; a full run builds the driver and
expects a sibling `cgp` checkout at `../cgp`. Work with the suite directly through it:

```sh
cargo test -p cargo-cgp-ui-tests                                  # just the snapshot suite
cargo test -p cargo-cgp-ui-tests --test ui -- --bless             # regenerate .cgp.stderr and .rust.stderr
cargo test -p cargo-cgp-ui-tests --test ui -- -j 4                # check at most 4 fixtures at once
cargo test -q -p cargo-cgp-ui-tests --test ui -- usability        # only fixtures whose path contains "usability"
cargo test -q -p cargo-cgp-ui-tests --test ui -- --print greet    # print raw output for a fixture
```

Passing an argument to the harness needs `--test ui`, so the flag is not also handed to the crate's
other (libtest) tests. The harness checks fixtures in parallel across a pool of workers, each with its
own throwaway crate (so they never share a `src/main.rs` or a cargo target lock); `--jobs`/`-j` sets
the worker count, which otherwise defaults to the machine's parallelism capped at 8. The snapshots
capture `cargo-cgp`'s own output end to end, so they are what changes once the tool reformats
diagnostics; a passing suite is also the standing end-to-end proof that the driver runs as the
compiler. The fixtures are grouped under `tests/ui/` by the *quality of the output* the tool
produces: `ok/` for a clean compile, `acceptable/` for an error whose cause the tool already presents
well, and `usability/` for one with a remaining presentation problem — each split into concept
sub-directories so no directory grows crowded. Add a scenario by dropping a `<name>.rs` file (with a
`fn main`) into the sub-directory that matches its output quality and running
`cargo test -p cargo-cgp-ui-tests --test ui -- --bless` (which writes both snapshot files). A fixture
whose output improves enough to clear the usability bar graduates from `usability/` into
`acceptable/` (a plain move of its `.rs`/`.cgp.stderr`/`.rust.stderr` triple — the snapshots are
independent of the fixture's directory, so no re-bless is needed).
Snapshots are blessed under the pinned toolchain, so a toolchain bump can require a re-bless. The full
testing picture — the harness structure, why it drives the whole tool rather than the driver
directly, and the comparison with Clippy — is documented in
[Testing](docs/implementation/testing.md); read it before adding tests, and keep it in sync when the
test setup changes.

### Checking a CGP project elsewhere with Nix

To exercise the tool's real output on CGP source *outside* this repository, run the local build
through Nix rather than provisioning a nightly by hand. From the **target project's** directory,
point the flake reference at this repository:

```sh
cd /path/to/the/cgp/project          # a cargo package/workspace that uses `cgp`
nix run /path/to/cargo-cgp -- check   # == `cargo cgp check`, args after `--` go to `cargo check`
```

The full instructions — preferring a local checkout over the published flake, the other Nix entry
points, why no rustup or project toolchain is needed, and how the check isolates its `target/cgp` —
are in the usage reference:
[Usage](docs/reference/usage.md#running-on-a-project-outside-this-repository) and
[Installation](docs/reference/installation.md#installing-with-nix). A good input is a distilled CGP
program that reproduces an [error class](https://github.com/contextgeneric/cgp/blob/main/docs/errors/README.md),
or one of this repository's own [UI fixtures](tests/README.md), dropped into a throwaway cargo package. Rebuild
for a code change with `nix build` first, or just re-run `nix run`, which rebuilds only when a source
file in the flake's narrowed input actually changed.

## Improving error messages against real-world CGP code

The project's core loop is to feed `cargo-cgp` real CGP code that fails to compile and improve how it
presents the error. The work is *driven* from outside this repository — you are given the path to a
Rust project whose CGP code does not compile — but every change *lands* inside this repository, and it
lands as a **test fixture before any code**. This section codifies that loop; follow it whenever a
task points you at such a project.

**The tool knows CGP, never any one project — two rules that admit no exceptions.** An example
project is only a *source of failing input*; the improvement it motivates must generalize to every
CGP program with the same shape, so:

- **Never hard-code logic for an example's code inside `cargo-cgp`.** The resolver and the wording
  reason only about *core CGP constructs* — the consumer/provider traits, `DelegateComponent`,
  `HasField`, the `Symbol!`/`Cons`/`Either` spines, the handler combinators, and the like, all
  anchored by `DefId` to the `cgp` crates that define them. A fix must never match on a name, type, or
  module that belongs to the example project (`WebSocket`, `MyApp`, `keyword`, a DSL's own markers):
  such a rule would silently fail on the next project and is a defect even if it makes today's message
  perfect. If a failure seems to need example-specific knowledge, the real gap is in how a *core*
  construct is walked or worded — find and fix that instead.
- **Keep every explanation generalized; never frame it around a specific example.** A knowledge-base
  document, an inline doc comment, and a UI fixture must each be self-contained and framed around the
  *core CGP concept* it describes — never around any one project that happens to exhibit it. Assume
  the reader, human or agent, knows core CGP (via the [construct reference](https://github.com/contextgeneric/cgp/blob/main/docs/reference/README.md)
  and the `/cgp` skill) but **nothing** about any non-core CGP project. An example may *illustrate* a
  generalized concept, but only when the prose explains it in core-CGP terms that stand on their own:
  state the general rule first and present the example as one demonstration of it; never let a
  mechanism read as if it applied only to that example's own types, names, or domain; and name the
  originating project at most as a passing aside the explanation does not depend on — never as
  something the reader must already understand. Prefer generalizing the phrasing over naming any
  concrete type from an example at all — where a concrete type does appear, choose a self-explaining
  core-CGP one (a `HasField` field, a `std` bound like `AsRef<[u8]>`) over one whose meaning lives in
  the example. Reproduce a real failure as a *distilled, self-contained* fixture rather than a
  reference to the example. A core CGP construct that merely appears in examples (a handler combinator
  such as `ComposeHandlers`/`PipeHandlers`, say) is fair game, but introduce it as the CGP construct
  it is — linking the [construct reference](https://github.com/contextgeneric/cgp/blob/main/docs/reference/README.md) or a concept page such as
  [handlers](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/handlers.md) for background — not as "the thing that example uses."

Read these first, every time, before touching anything — the loop assumes their contents. They fall
into three groups: how the tool is run and its output read, how the tool builds that output, and what
the upstream error classes are.

- **Using the tool** — [Usage](docs/reference/usage.md) (running the check, and reading its output and
  `[CGP-Exxx]` codes), [Troubleshooting](docs/reference/troubleshooting.md) (when the tool itself will
  not run), and the [error-code catalog](docs/error-code.md) (what each code already means).
- **How the output is produced** — [Typed root-cause resolution](docs/implementation/typed-root-cause-resolution.md)
  (the resolver that turns a wiring failure into its root-cause tree — the main lever for a better
  message), [The driver](docs/implementation/driver.md) and [Error processing](docs/implementation/error-processing.md)
  (the emitter and the rustc-free wording/plan/tree it drives), [The error pipeline](docs/implementation/error-pipeline.md)
  (how the stages fit together), [rustc diagnostic internals](docs/implementation/rustc-diagnostic-internals.md)
  (where the compiler hides the information a good message needs), and [Testing](docs/implementation/testing.md)
  (the UI snapshot suite and its bless workflow, the mechanism the loop runs on).
- **The problem domain** — the upstream [CGP error catalog](https://github.com/contextgeneric/cgp/blob/main/docs/errors/README.md) (which error
  classes hide the root cause and where the cause sits), and the **`/cgp` skill**, invoked as always
  when reasoning about CGP constructs.

The loop then runs in order, and the ordering is the point — the fixture is written and confirmed to
reproduce the problem *before* the tool is changed, and the fixture is confirmed improved *before* the
target project is re-checked.

1. **Reproduce the failure in the target project.** Go to the given directory and run the check
   through the local Nix build, per
   [Checking a CGP project elsewhere with Nix](#checking-a-cgp-project-elsewhere-with-nix):
   `nix run /path/to/this/repo -- check` from the project's directory. Capture the exact message the
   tool prints.
2. **Learn the real root cause from the project's own diff.** The target project is set up with
   **uncommitted git changes that deliberately comment out the code triggering the error**, so its
   `git diff` / `git status` shows precisely what was removed and therefore what the true cause is.
   Read that diff to know what the ideal message *should* say. Treat the target project as read-only,
   like `../cgp`: never commit, revert, or otherwise change it — its uncommitted state is the
   diagnostic aid, not something to tidy, and reverting it destroys the reproduction the whole task
   rests on. Re-check `git status` there before you finish, because losing that state is easy to do
   by accident and silent when it happens: the check simply starts passing.
3. **Judge the gap.** Compare the tool's actual output against that known root cause and decide how the
   message should improve — what it buries, misnames, or omits.
4. **Reproduce it as a simplified UI fixture — do not fix the tool yet.** Before changing any
   `cargo-cgp` code, distill the failure into the smallest self-contained CGP program that provokes the
   same *class* of bad message, and add it as a `<name>.rs` fixture under
   [`tests/ui/`](tests/README.md) — in the sub-directory matching the tool's *current* output quality
   (`usability/` for a message that carries the cause but buries it, and the concept sub-directory that
   fits). Bless it (`cargo test -p cargo-cgp-ui-tests --test ui -- --bless`) and confirm the committed
   `.cgp.stderr` shows the same shortcoming you saw on the real project. A change with no fixture that
   reproduces its motivation does not belong in the tool.
5. **Change the tool, and verify the fixture first.** Make the improvement (usually in the
   [resolver](docs/implementation/typed-root-cause-resolution.md) or the rustc-free
   [wording](docs/implementation/error-processing.md)), then re-run the UI suite and confirm the
   fixture's `.cgp.stderr` improved as intended — and that no other snapshot regressed — **before** you
   go back to the target project. The simplified fixture, not the real project, is the fast feedback
   loop.
6. **Re-check the target project.** Only once the fixture is green and improved, re-run the Nix check on
   the target project and confirm the real-world message improved the same way. If it did not, the
   fixture did not capture the real cause — return to step 4 with a fixture that does.
7. **Confirm the fix the message names actually works.** When the improved message *tells the reader
   what to change* — a `help` naming a wiring entry, say — that claim is worth testing rather than
   inferring from the diff: a fix can be necessary without being sufficient, and a `help` that leaves
   the build failing is worse than none. Test it in a **`git worktree`**, never in the project itself:

   ```sh
   git -C /path/to/project worktree add ../scratch-verify HEAD
   git -C /path/to/project diff | git -C ../scratch-verify apply   # reproduce the failing state
   # apply the change the message names, then run the check from ../scratch-verify
   git -C /path/to/project worktree remove --force ../scratch-verify
   ```

   Place the worktree **as a sibling of the project**, as above: a CGP example workspace usually
   `[patch]`es `cgp` to a relative `../cgp`, which resolves from a sibling and breaks anywhere else.
   A worktree is the right tool because it cannot write into the original — it registers under the
   project's `.git` (so remove it when done) but leaves every tracked file untouched. Do **not**
   improvise a copy with `rsync`/`cp` plus an in-place edit — a stray `sed -i`, an unset variable
   collapsing a path, or a mistaken argument order will silently write back into the project and
   revert the very state step 2 depends on, and the only symptom is that the check starts passing.
8. **Graduate the fixture if it earned it.** A fixture whose output now clears the usability bar moves
   from `usability/` into `acceptable/` (a plain move of its `.rs`/`.cgp.stderr`/`.rust.stderr` triple,
   no re-bless), recording that this class of error is now presented well.

## Committing changes

Git commits are made **only when the user explicitly asks for one**, and each such request authorizes
exactly one commit of the changes then in the working tree — nothing more. These rules are absolute
and override any general default:

- **Never commit unless explicitly asked.** Do not commit as a side effect of finishing a task,
  passing tests, or "wrapping up." If the work is done and the user has not asked to commit, simply
  leave the changes uncommitted.
- **Never bring commits up yourself.** Leaving work uncommitted is the normal, expected state, so it
  is not news: do not report commit status when finishing a task ("nothing is committed", "these
  changes sit uncommitted on top of X"), do not ask whether to commit, and do not suggest or hint
  that committing is the next step. The user tracks this themselves and will say when they want a
  commit. Answer plainly if they ask about it — the rule is against volunteering it, not against
  discussing it.
- **A commit request is one-shot, never a standing mode.** A prompt such as "commit the changes"
  means *commit the current changes, this once*. It does **not** mean "commit automatically from now
  on." Treating a single commit request as a switch that turns on automatic committing is a serious
  misreading — the two are entirely different instructions and must never be conflated. After
  fulfilling a commit request, return to requiring an explicit ask for the next one.
- **Always commit on the current branch.** Commit onto whatever branch is checked out; do not create
  or switch branches, even when that branch is `main`/the default.

## Ask when in doubt

When something should be settled before the next step — an ambiguous intended behaviour, a design
choice with more than one defensible answer, or a change that would couple this project to `cgp` —
surface the question rather than guessing.
