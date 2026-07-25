# The expand command

`cargo cgp expand` shows a CGP programmer the ordinary Rust their CGP macros generate: a full macro
expansion in the style of `cargo-expand`, with CGP's type-level sugar resugared by the driver before
the text is handed back, so a field name reads as `Symbol!("height")` rather than as a six-deep
`Chars` spine.

**Status: blueprint ahead of implementation.** Nothing here is built yet. The document records the
design agreed for the work, the compiler facts it rests on (each verified against
[`../external/rust`](../../../external/rust)), and the selective-expansion phase deliberately
deferred, so a later agent can carry it out without re-deriving any of it. It extends
[Executable structure](executable-structure.md), whose front-end/driver split it reuses, and
[The driver](driver.md), whose `Callbacks` it adds a second hook to. The reference implementation for
the command as a whole is not Clippy but `cargo-expand`, checked out read-only at
[`../external/cargo-expand`](../../../external/cargo-expand).

## What the command shows

The command answers one question a CGP programmer asks constantly: *what did that macro actually
generate?* CGP is macro-heavy by design, so most of what the compiler type-checks is code nobody
wrote, and reading it is how a programmer confirms a wiring table means what they intended, learns
why a provider needs the bound it needs, or checks a suspicion raised by a `cargo cgp check` error.
Take a rectangle whose area provider reads two context fields through an auto-getter:

```rust
#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}
```

`#[cgp_auto_getter]` generates a blanket impl requiring the context to carry both fields, and the
requirement it emits is a `HasField` bound keyed by a type-level string. Expanded and resugared, that
blanket impl reads the way the programmer thinks about it:

```rust
impl<__Context__> HasRectangleFields for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("height"), Value = f64>,
{
    fn width(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
    …
}
```

Without the resugaring, each of those bounds is a wall of characters that the compiler's own
pretty-printer then line-breaks mid-spine:

```rust
    __Context__: HasField<Symbol<6,
    Chars<'h',
    Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>, Value = f64>,
```

That is the difference this command exists to make, and it is the same difference the
[diagnostic resugaring](error-processing.md) makes to an error message; the two are one idea applied
to two outputs.

**Expanding is not checking.** The driver stops the compilation once expansion is done, so type
analysis never runs and no CGP diagnostic is produced. A malformed macro invocation still fails, since
that failure happens *during* expansion, but a wiring mistake does not: `cargo cgp check` remains the
diagnostic command, and `expand` is the reading tool beside it.

## Why the whole crate is expanded, not only the CGP macros

The command expands everything, because `rustc` offers no way to expand selectively and the
alternative — expanding only CGP macros ourselves — buys selectivity at the cost of fidelity to the
project's own `cgp` version. Three compiler facts settle the first half of that claim.

Expansion is a whole-crate fixed point, driven to completion before anything can look at the result.
`-Zunpretty=expanded` does not expand; it *prints* an already-expanded AST, and its `PpSourceMode`
has exactly four variants — `Normal`, `Expanded`, `ExpandedIdentified`, `ExpandedHygiene`
([`rustc_session/src/config.rs`](../../../external/rust/compiler/rustc_session/src/config.rs)) — none
of which filters by macro.

The compiler exposes no expansion hook. `rustc_interface::interface::Config` offers `psess_created`,
`register_lints`, `override_queries`, `file_loader`, `extra_symbols`, and `make_codegen_backend`, and
`rustc_driver::Callbacks` offers `config`, `after_crate_root_parsing`, `after_expansion`, and
`after_analysis`. Nothing sits *inside* expansion, and `after_crate_root_parsing` is too early to
help — its own doc comment notes that submodules are not yet parsed when it runs, so it cannot even
see most of the crate.

What remains is therefore *un*-expansion: let the compiler expand everything, then put the non-CGP
parts back. That is a real design, the compiler records exactly the provenance it needs, and it is
[the deferred second phase](#selective-expansion-the-deferred-phase) below rather than part of the
first slice.

The cost of expanding everything is real but bounded, and worth stating plainly so nobody is
surprised by the output. A nine-line file carrying `#[derive(Debug, Clone)]` and one `println!`
expands to thirty-one lines, including a `#![feature(prelude_import)]` header, two derived impls, and
`::std::io::_print(format_args!("rect = {0:?}\n", rect))`. On CGP code the ratio is far better,
because CGP's own expansion is what the reader came for and dwarfs the std noise — but the noise is
present, and removing it is the point of the deferred phase.

## The front-end: the `expand` subcommand

The front-end gains a fourth subcommand that launches the same wrapped compilation `check` does and
then prints what the driver produced. [`run::dispatch`](../../crates/cargo-cgp/src/run.rs) grows an
`expand` arm beside `check`, `setup`, and `update`, and the [help text](../../crates/cargo-cgp/src/help.rs)
grows the matching line.

**It runs `cargo rustc`, not `cargo check`, and the difference is load-bearing.** The driver has to be
told which crate to expand, and `cargo rustc` is the one cargo subcommand that appends extra rustc
arguments to a *single* target's invocation. So the front-end runs, in effect:

```text
cargo rustc --profile check <forwarded args> -- --cgp-expand=<output path>
```

`--profile check` skips codegen, exactly as `cargo-expand` does. The marker flag after `--` reaches
only the selected target, so the crate's dependencies — including workspace siblings, which also go
through the driver as the workspace rustc wrapper — compile normally and produce their metadata. An
environment variable would have been the other way to signal expand mode, but it would reach every
workspace crate at once and the driver would then have to reconstruct which one the user meant from
`--crate-name`; letting cargo do the scoping is both simpler and more accurate.

Everything else about launching the compilation is what `check` already does, and should be *shared
code rather than a copy*: the [preflight](distribution.md) that verifies a matching driver, forcing
`RUSTUP_TOOLCHAIN` to the pinned nightly, `CARGO_CGP_SYSROOT`, the dynamic-library path, and the
isolated `target/cgp` directory. The natural shape is to lift that setup out of
[`check/command.rs`](../../crates/cargo-cgp/src/check/command.rs) into a module both subcommands call,
leaving each command to choose the cargo subcommand and the arguments.

**Target selection is cargo's problem, not ours.** `cargo rustc` requires exactly one target and
errors when the choice is ambiguous, so the front-end forwards `--lib`, `--bin`, `-p`, `--features`,
and the rest verbatim and lets cargo's own error tell the user to disambiguate. `cargo-expand`
re-declares every one of those flags with `clap` and additionally consults the manifest's
`default-run`; the front-end has no tool-specific arguments today and this keeps it that way.

**The driver writes the finished text to a file and the front-end prints it.** The output path is a
temp file under the isolated target directory, passed in the marker flag; when the compilation is
done the front-end reads it and writes it to stdout. Routing the content through a file rather than
the driver's stdout keeps it from interleaving with cargo's progress output, which is why
`cargo-expand` passes `-o` too, and it leaves the front-end's role unchanged from `check`: it never
parses or reshapes what the driver produced, it only relays it.

Judging success needs care, because the compilation deliberately does not finish. The unit produces
no metadata, so cargo may report a failure for a run that did exactly what was asked. The front-end
therefore treats the presence of non-empty output as success — `cargo-expand` makes the same call,
checking `outfile_path.exists()` and reporting `ERROR: rustc produced no expanded output`
otherwise — and propagates cargo's exit code only when there is no output to show. The exact cargo
behaviour here is the one part of this design that must be confirmed empirically rather than from the
source, and it is listed under [Open decisions](#open-decisions-to-confirm-during-implementation).

## The driver: expand mode

The driver recognizes one flag, and stripping it is the first thing it does. `--cgp-expand=<path>`
is not a rustc flag, so [`args::rustc_args`](../../crates/cargo-cgp-driver/src/args.rs) removes it
from the vector it builds — the same normalization step that already drops cargo's injected `rustc`
path and injects `--sysroot` — and records the path as the driver's expand request. The flag is the
second half of the argument-and-environment contract between the two executables, alongside
`CARGO_CGP_SYSROOT`, and like that variable its spelling is declared independently in each crate's
`config` module.

**The driver must not set `-Zunpretty=expanded`, and the reason is the single most important fact in
this document.** When `sess.opts.pretty` is set, `run_compiler` prints the crate and exits *before*
any callback runs:

```rust
let mut krate = passes::parse(sess);
if let Some(pp_mode) = sess.opts.pretty {
    // … create_and_enter_global_ctxt, pretty::print, write_dep_info …
    return early_exit();
}
if callbacks.after_crate_root_parsing(compiler, &mut krate) == Compilation::Stop { … }
```

([`rustc_driver_impl/src/lib.rs`](../../../external/rust/compiler/rustc_driver_impl/src/lib.rs).)
So under that flag the driver never gets a turn, and could not resugar anything — it would have to
capture rustc's output through `Config.output_file` and post-process it after `run_compiler` returned,
which also trips the `IgnoringOutDir` warning that `build_output_filenames` emits whenever an output
file is set alongside cargo's `--out-dir`
([`rustc_interface/src/util.rs`](../../../external/rust/compiler/rustc_interface/src/util.rs)).
Doing the work in a callback instead avoids both, and lands the driver on precisely the seam the
deferred selectivity phase needs.

So the driver hooks `after_expansion` and does the printing itself. By the time that callback runs,
macro expansion has happened — `run_compiler` forces `tcx.resolver_for_lowering()` immediately before
calling it, which is what drives expansion and name resolution — and the expanded AST is reachable as
`&'tcx Steal<ast::Crate>` from that same query, which is exactly where rustc's own
[`pretty.rs`](../../../external/rust/compiler/rustc_driver_impl/src/pretty.rs) reads it. The hook
clones the crate out from behind the `Steal` (`ast::Crate` is `Clone`) and renders it with the
compiler's own printer:

```rust
pprust::print_crate(
    sess.source_map(), &krate, src_name, src,
    &NoAnn,        // a one-line `impl PpAnn for NoAnn {}`; rustc's own is private
    true,          // is_expanded
    sess.psess.edition, &sess.psess.attr_id_generator,
)
```

This is the same call `pretty::print` makes for `-Zunpretty=expanded` over an unmodified crate, so the
text is the text `cargo-expand` would consume, and the `is_expanded` flag keeps the faked
`#![feature(prelude_import)]` / `#![no_std]` preamble that stops the printed source from re-injecting
libstd. The `src` and `src_name` arguments are the crate root's source text and name, read back from
the `SourceMap` the way `pretty.rs`'s own small `get_source` helper does, and they are what lets the
printer interleave the original comments.

The driver then hands that text to the [rustc-free resugaring crate](#the-rustc-free-resugaring),
writes the result to the requested path, and returns `Compilation::Stop`. Stopping is both sufficient
and deliberate: nothing downstream is wanted, and running analysis on a crate we are only reading
would cost a full type-check for no output.

Two smaller decisions round out expand mode. **The injected diagnostic flags are skipped**, because
`-Znext-solver=globally` and `--verbose` exist to shape diagnostics produced during analysis, which
never runs here; leaving them off keeps expand mode minimal and its cost predictable. **The CGP
emitter stays installed**, so a genuine expansion-time error — a `delegate_components!` body that does
not parse — is still rendered the way `check` would render it. Such an error aborts before the printing
hook, so the driver writes no output and the front-end reports that nothing was produced.

## The rustc-free resugaring

The resugaring lives in a new library crate, `cargo-cgp-expand`, which links no compiler internals
and exposes one entry point the driver calls — roughly a
`resugar_expanded_source(&str, &ExpandOptions) -> String`. Keeping it out of the `rustc_private`
linkage is the same rule that put the diagnostic
wording in [`cargo-cgp-error-processing`](error-processing.md): the logic is pure text-to-text, so it
builds and its tests run on any toolchain, with no compiler in the loop. It is a separate crate rather
than a module of the error-processing crate because expansion output is not diagnostics, and the two
share no types.

The pipeline inside it is four steps: parse the compiler's text with `syn::parse_file`, rewrite the
CGP spines on the syntax tree, optionally strip CGP path qualifiers, and print with
`prettyplease::unparse`. When `syn` cannot parse the compiler's output — rare, but expansion can
produce shapes `syn` does not accept — the crate returns the input text unchanged, so the command
degrades to plain `cargo-expand` output rather than failing. `cargo-expand` has the same ladder, with
an extra `rustfmt` rung this crate does not need.

**What each construct folds back to is not this document's subject — [Resugaring](resugaring.md) is.**
That document specifies every construct (`Symbol!`, `Path!`, the `Product!`/`Sum!` spines and their
`Struct!`/`Enum!` forms), the exact-match rule each obeys, and the fixed pass order they must run in.
This pass is the third of the three implementations it describes, and the two facts that make it a
separate implementation rather than a reuse of the existing ones are worth stating here, since both
were measured on a prototype.

**It matches on `syn::Type`, not on rendered text, because the text matchers are formatting-sensitive.**
The diagnostic post-processors are `&str -> Option<String>` functions written against rustc's
*diagnostic* rendering; `prettyplease` breaks a long generic list across lines and ends it with a
trailing comma before the closing `>`, which the `Symbol!` matcher's final `>` check rejects. In the
prototype that left `Symbol!("width")` resugared and `height` a raw spine, purely because the longer
name was the one that got wrapped. Matching a parsed type is immune to formatting, and it lets the
printer format the resugared macro call — which is why the output above is one tidy line.

**Its passes must stay separate whole-tree visits, because a visitor recurses innermost-first.** This is
the sharpest form of the `Nil` overlap hazard [Resugaring](resugaring.md#the-rules-every-resugaring-follows)
describes: one combined visitor rewrites a `Symbol`'s terminating `Nil` to `Product![]` before it
examines the enclosing `Symbol`, and every field name silently stays raw. The prototype hit it twice,
once on `Symbol` and once on an `open` statement's `PathCons<AreaCalculatorComponent, Nil>`.

Sugar the *user* wrote needs no attention at all — a hand-written
`PipeHandlers<Product![StepOne, StepTwo]>` comes out as written, because the CGP macro that copied it
never expanded it.

The one option the first slice needs is how much path noise to remove. CGP macros emit fully-qualified
`::cgp::macro_prelude::Symbol<…>`, which is pure noise to a reader and which the resugaring must see
past anyway, so **the `cgp::macro_prelude::` qualifier is stripped by default** (`cgp`'s own
`strip_macro_prelude` does this for its expansion snapshots, and the diagnostic chain's
`strip_cgp_prefixes` does it for errors). General module qualifiers are **kept**, unlike in a
diagnostic, because in source they carry information a reader may want.

A `--verbatim` flag turning the stripping off, so the output stays compilable, is the natural escape
hatch — and it would be the front-end's *first* tool-specific argument, so it has to be recognized and
removed before the rest are forwarded to cargo. That is the one place where `expand` cannot stay the
pure pass-through `check` is, and it is why the flag is worth deferring until someone wants it.

## Open decisions to confirm during implementation

Four questions are settled enough to build on but should be verified or revisited rather than
inherited as fact. The first two are empirical, the last two are judgement calls a first user will
sharpen.

- **How cargo reports a unit that produced no artifact.** The design treats non-empty output as
  success and falls back to cargo's exit code otherwise, following `cargo-expand`; confirm the actual
  exit code and whether cargo re-runs the unit on the next invocation because its fingerprint is
  unsatisfied.
- **Whether any spurious rustc warning survives.** Not setting an output file should avoid
  `IgnoringOutDir` entirely; if some other warning appears in expand mode, suppress it in the
  driver's emitter rather than filtering cargo's stderr in the front-end, since the front-end does not
  process output. (`cargo-expand` filters a handful of such lines by text in `ignore_cargo_err`, an
  approach this tool should not need.)
- **Whether module qualifiers should be stripped.** Kept by default above; if real output proves
  unreadable, the diagnostic chain's `strip_module_paths` is the ready-made pass to reuse.
- **What the second slice adds first**: syntax highlighting and paging (`cargo-expand` uses `bat`), an
  `--item` filter (it uses `syn-select`), or the selectivity below. All three are out of the first
  slice.

Implementing the command also carries two synchronization obligations beyond this document:
[`reference/usage.md`](../reference/usage.md) gains an `expand` section, and the front-end help text
gains its line.

## Selective expansion: the deferred phase

Expanding only the CGP macros remains the goal this command started from, and the design above is
chosen so that it can be added at the same seam rather than rebuilt. The compiler records exactly the
provenance the work needs: every `ExpnData` carries `macro_def_id: Option<DefId>` alongside
`kind: ExpnKind::Macro(MacroKind, Symbol)` and the `call_site` span
([`rustc_span/src/hygiene.rs`](../../../external/rust/compiler/rustc_span/src/hygiene.rs)). So each
node of the expanded AST can be traced to the macro that produced it, recognized by *defining crate*
rather than by name — the same `DefId`-anchored discipline the typed resolver holds itself to. The
work is then to walk the cloned crate, find each maximal subtree produced by a non-CGP expansion, and
replace it with the invocation the programmer wrote, reconstructed from the source at its call site as
an `ast::MacCall` node the printer already knows how to print.

Three sub-problems make this a phase of its own rather than an afternoon. One invocation commonly
expands to several sibling items, so a run of siblings sharing an expansion must collapse into one
printed invocation. A derive is worse: expansion strips `#[derive(Serialize)]` from the item and
appends the impls it generated, so restoring it means re-adding the attribute *and* deleting those
impls. And some things cannot come back at all — code a `cfg` eliminated is simply absent, and a CGP
macro invoked from inside a non-CGP macro's output would have to stay expanded inside an
otherwise-unexpanded parent.

**A syntactic expander was prototyped and rejected as the primary design**, and the reasons are worth
keeping because they are the reasons to revisit it if driver printing ever proves impractical. CGP's
macro entry points are ordinary functions — `cgp_macro_lib::cgp_impl(attr, body) -> syn::Result<TokenStream>` —
and `cgp`'s own `cgp-macro-test-util-lib` already calls them that way to pin expansion snapshots, so a
`syn`-based expander that walks a file and expands only the constructs it recognizes is both small and
exactly selective; a prototype of roughly 250 lines handled components, providers, auto-getters,
derives, and wiring tables, leaving `#[derive(Debug)]`, `println!`, and `macro_rules!` untouched. It
was rejected for three reasons: it would bake one `cgp` version into `cargo-cgp` and could show a user
an expansion their own `cgp` never produces; it would be `cargo-cgp`'s first *code* dependency on a
`cgp` crate (permitted by the one-way rule, which forbids only the reverse, but an architectural
change of its own); and it cannot see `cfg`, a `mod` tree, or a CGP macro emitted by another macro.
One concrete gap belongs to that route alone and is recorded so it is not rediscovered:
`cgp-async-macro` keeps its logic inside the proc-macro crate with no library counterpart, so
`#[async_trait]` would need one added upstream to be expandable that way.

## Comparison with cargo-expand

`cargo-expand` is this command's reference implementation, in the way Clippy is the reference for the
rest of the tool, so knowing where the two agree and diverge is the fastest way to understand why
`expand` is shaped as it is. Clippy itself has no analogue — it adds lints and lets the compiler's own
emitter render them, and never prints a crate — so it plays no part in this comparison.

**`cargo-expand` owns no expansion logic, and that framing explains most of the design.** Its whole
job is to run `cargo rustc --profile check -- -o <tmpfile> -Zunpretty=expanded`, read the file rustc
wrote, and make the text nicer: `syn::parse_file` it, discard the comments the compiler misplaced,
print it with `prettyplease` (falling back to `rustfmt`, then to rustc's own output), and pipe the
result through `bat` for highlighting
([`main.rs`](../../../external/cargo-expand/src/main.rs)). Everything load-bearing is the compiler's.
`cargo cgp expand` has one thing to add — the resugaring — and every divergence below follows from
where that addition has to happen.

The first slice follows `cargo-expand` on the points where it has already found the right answer. It
runs the same `cargo rustc --profile check` for the same reason (per-target rustc arguments, no
codegen); it routes the expansion through a file rather than stdout, so the content never interleaves
with cargo's progress output; it judges success by whether output was produced rather than by cargo's
exit code, since the unit deliberately makes no artifact; and it post-processes with `syn` plus
`prettyplease` behind a fallback that returns the compiler's own text when parsing fails.

Four divergences are deliberate, and each follows from something this tool already has or wants.

- **The post-processing lives in the driver, not the front-end.** `cargo-expand` is a single binary
  with no driver, so its only option is to post-process the text after cargo exits. `cargo-cgp` already
  has a driver that owns every transform while the front-end merely relays output
  ([Executable structure](executable-structure.md)), and putting the resugaring anywhere else would
  break that split.
- **No `-Zunpretty=expanded`.** `cargo-expand` has nothing to add once rustc has printed, so the flag
  is ideal for it. The driver has to run *after* expansion to resugar, and the flag makes rustc print
  and exit before any callback, so the driver calls `pprust::print_crate` itself instead.
- **No `RUSTC_BOOTSTRAP` machinery.** Roughly half of `cargo-expand`'s `main.rs` —
  `needs_rustc_bootstrap`, `do_rustc_wrapper`, the `CARGO_EXPAND_RUSTC_WRAPPER` handoff — exists only
  to enable a `-Z` flag on a stable toolchain. `cargo cgp expand` forces the pinned nightly the way
  `check` does, so none of it is needed.
- **No `$crate` placeholder hack and no `rustfmt` rung.** `cargo-expand` rewrites `$crate` to a
  same-width `Ξcrate` so `rustfmt` can parse its input; this crate never invokes `rustfmt`, so its
  fallback ladder is two rungs rather than four.

The gaps where `cargo-expand` does more are first-slice omissions rather than decisions, and they are
the obvious candidates for a second slice: no syntax highlighting or paging (its `bat` dependency), no
`--item` filter (its `syn-select` dependency), no `--ugly` raw mode, no theme selection, and no
re-declared cargo flags of its own — including the manifest `default-run` lookup it uses to pick a
default binary, where this command leaves the ambiguity for cargo to report.

The one thing `cargo cgp expand` means to do that `cargo-expand` never will is CGP-aware output: the
resugaring in the first slice, and [the selectivity](#selective-expansion-the-deferred-phase) after it.
Both need a foothold inside the compilation rather than after it, which is the whole reason the
printing happens in a callback.

## Further reading

Two external references explain mechanisms this design leans on rather than re-derives, and both are
worth reading before changing the launching or printing halves.

- [`cargo rustc` — The Cargo Book](https://doc.rust-lang.org/cargo/commands/cargo-rustc.html) — defines
  the behaviour the front-end depends on: arguments after `--` are passed to the compiler invocation
  for the *selected target only*, which is what scopes expand mode to one crate.
- [Macro expansion — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/macro-expansion.html)
  — the fixed-point expansion process and its interleaving with name resolution, the reason there is no
  per-macro switch to ask for.

## Tests

Nothing is guarded yet, since nothing is implemented; this section records the coverage the
implementation owes, and should be rewritten as each test lands.

- **Unit tests in `cargo-cgp-expand`** over hand-written expanded-source inputs, running with no
  compiler: one per spine (`Symbol!`, `Path!`, `Product!`/`Sum!` and their `Struct!`/`Enum!` forms), the
  pass-ordering hazard (a `Symbol` whose terminating `Nil` a combined visitor would turn into
  `Product![]`), a structurally-wrong `Symbol` left untouched, and a `syn`-unparsable input returned
  verbatim.
- **`insta` inline snapshots** of whole-file resugaring, so the printed shape is pinned the way
  [`tests/graph.rs`](../../crates/cargo-cgp-error-processing/tests/graph.rs) pins the dependency graph.
- **An expand fixture harness** mirroring the [UI suite](testing.md): a `<name>.rs` fixture and a
  committed `<name>.expand.rs` snapshot, blessed the same way. It is a separate harness from the UI one
  because the artifact compared is stdout rather than stderr, and because a fixture must compile far
  enough to expand rather than fail to compile.
- **Front-end argument tests** for the new dispatch arm, alongside the existing
  [`tests/args.rs`](../../crates/cargo-cgp/tests/args.rs) cases.

## Source

None of these exist yet; the list is the intended shape of the change, so a reader can map this
document onto the tree as it lands.

- `crates/cargo-cgp/src/expand/` — the front-end subcommand: building and running the wrapped
  `cargo rustc`, passing the marker flag, and printing what the driver wrote.
- `crates/cargo-cgp/src/check/command.rs` → a shared launch module — the preflight, toolchain forcing,
  sysroot, dylib path, and target-directory injection both subcommands need.
- [`crates/cargo-cgp/src/run.rs`](../../crates/cargo-cgp/src/run.rs),
  [`help.rs`](../../crates/cargo-cgp/src/help.rs),
  [`config.rs`](../../crates/cargo-cgp/src/config.rs) — the new dispatch arm, its help line, and the
  marker-flag constant.
- `crates/cargo-cgp-driver/src/expand/` — expand mode: the request parsed off the marker flag, the
  `pprust::print_crate` call, and writing the resugared text.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs),
  [`callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs),
  [`config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — stripping the marker flag, the
  `after_expansion` hook, and the driver's half of the flag contract.
- `crates/cargo-cgp-expand/` — the rustc-free resugaring crate: the entry point, one module per pass,
  and the options.
