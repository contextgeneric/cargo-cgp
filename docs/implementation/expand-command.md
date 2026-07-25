# The expand command

`cargo cgp expand` shows a CGP programmer the ordinary Rust their CGP macros generate: a full macro
expansion in the style of `cargo-expand`, with CGP's type-level sugar resugared by the driver before
the text is handed back, so a field name reads as `Symbol!("height")` rather than as a six-deep
`Chars` spine.

**Status: implemented.** `cargo cgp expand` runs: the front-end launches a wrapped `cargo rustc`, the
driver prints the expanded crate from `after_expansion`, and the rustc-free
[`cargo-cgp-expand`](../../crates/cargo-cgp-expand) crate resugars it before the front-end prints it.
Every UI fixture now carries an `.expand.rs` snapshot, so the command is covered end to end.
What is *not* built is the [selective expansion](#selective-expansion-the-deferred-phase) the command
started from — the whole crate is expanded — and the conveniences listed under
[what the first slice leaves out](#comparison-with-cargo-expand). The document extends
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

Everything else about launching the compilation is what `check` already does, and it is *shared code
rather than a copy*: the [preflight](distribution.md) that verifies a matching driver, forcing
`RUSTUP_TOOLCHAIN` to the pinned nightly, `CARGO_CGP_SYSROOT`, the dynamic-library path, and the
isolated `target/cgp` directory all live in
[`launch/`](../../crates/cargo-cgp/src/launch), lifted out of what used to be the `check/` directory,
and each command only chooses the cargo subcommand and the arguments. `check.rs` is now just the run
itself.

**Target selection is cargo's problem, not ours.** `cargo rustc` requires exactly one target and
errors when the choice is ambiguous, so the front-end forwards `--lib`, `--bin`, `-p`, `--features`,
and the rest verbatim and lets cargo's own error tell the user to disambiguate. `cargo-expand`
re-declares every one of those flags with `clap` and additionally consults the manifest's
`default-run`; the front-end has no tool-specific arguments today and this keeps it that way.

**`expand` answers `--help` itself**, unlike `check`. Forwarding a help request is right for `check`,
whose every argument is cargo's, but `expand` has a flag of its own: `cargo rustc --help` would
describe cargo's options and never mention `--item`, so the one thing a reader needs to discover would
be the one thing missing — and the run would then end with "no expansion was produced", reporting a
help request as a failure. So a help flag anywhere in its arguments prints
[`expand_help_text`](../../crates/cargo-cgp/src/help.rs) and succeeds.

**`--item <path>` is the one argument the front-end does not forward.** A whole crate's expansion is
long, so the command narrows it on request, and the path travels to the driver as a second marker flag
(`--cgp-expand-item=<path>`) beside the output one. The form is a flag rather than the bare positional
`cargo-expand` accepts, and that follows from forwarding: with every other argument passed through
untouched, a bare word cannot be told from the value of a cargo flag (`--bin my_module`) without
re-declaring cargo's whole argument grammar here, which is what forwarding exists to avoid. The path's
*shape* is checked in the front-end, before anything compiles, so a typo costs nothing; the driver
parses it again for real, since the matching lives there.

A **crate-root prefix is accepted and dropped** — `crate::contexts::app`, `::contexts::app`, and
`self::contexts::app` all mean `contexts::app`. Matching is against module paths within the crate, which
carry no such prefix, but `crate::…` is how the module is spelled in the source, so it is what a reader
reaches for.

What a path selects is three rules, and the third is the CGP-shaped one:

- an item **declared** at that path — and a module selects its *contents*, since the `mod` wrapper is
  noise around what was asked for;
- an `impl` whose **self type** is that path, so `--item Rectangle` shows the struct with its
  `HasField` impls and its wiring;
- an `impl` whose **trait** is that path, so `--item AreaCalculator` shows a component's provider trait
  with the blanket impls, the `UseContext` impl, and each provider's impl of it.

The third rule is what makes the filter useful here rather than merely available. A CGP component's
generated items are almost all impls, and impls have no names of their own — so a filter that matched
only declarations would answer "what does this component generate?" with just the trait definition.

**A path that matches nothing yields nothing — never the whole crate — and the *driver* reports it.**
Which layer reports what matters here, because getting it wrong produced a genuinely misleading message.
The front-end sees only an absent expansion, and cannot tell a path that names nothing from a crate that
never got far enough to expand, nor from cargo declining to run at all (a package with several targets
needs `--lib` or `--bin NAME`). It used to guess, and so contradicted cargo's own correct explanation by
blaming the path. Now each layer reports what it knows: cargo says it would not run, the compiler says
the crate failed, the driver says the path matched nothing, and the front-end only reports that nothing
came back and points up.

The driver prints that one message straight to stderr rather than emitting a compiler diagnostic, for
two reasons. It is not about the code being compiled, so it should not add to the crate's error count
and make cargo report a failed compilation. And the driver's own
[post-processing](error-processing.md) would rewrite it: the module-path strip that shortens CGP type
names in an error would shorten the very path the message quotes, turning `contexts::nope` into `nope`.

**The driver writes the finished text to a file and the front-end prints it.** The path is a temp
file named after the front-end's process id, so two concurrent runs never read each other's output
([`expand/output.rs`](../../crates/cargo-cgp/src/expand/output.rs)); the front-end clears it before
launching cargo, reads it afterwards, and writes it to stdout. Routing the content through a file
rather than the driver's stdout keeps it from interleaving with cargo's progress output, which is why
`cargo-expand` passes `-o` too, and it leaves the front-end's role unchanged from `check`: it never
parses or reshapes what the driver produced, it only relays it.

The expansion also builds under `--profile check`, since it needs no codegen — added unless the
caller chose a profile of their own, which `forwards_profile` decides the way the target-directory
injection decides its own default.

Judging success needs care, because the compilation deliberately does not finish. The unit produces
no metadata, so cargo could report a failure for a run that did exactly what was asked. The
front-end therefore treats non-empty output as success and falls back to cargo's exit code only when
there is nothing to show — the same call `cargo-expand` makes, checking `outfile_path.exists()`. In
practice cargo exits 0 and simply re-runs that unit on the next invocation, since its fingerprint is
never satisfied; a following `cargo cgp check` in the same target directory re-checks the crate
normally.

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
would cost a full type-check for no output. A write that fails is reported as a compiler warning
rather than swallowed, so a bad path does not look like a crate that would not expand.

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
That document specifies every construct, the exact-match rule each obeys, and the fixed pass order.
This pass is the third of the three implementations it describes; what belongs here is the handful of
facts particular to resugaring *source* rather than a diagnostic, each of which the implementation
either confirmed or forced.

**It matches on `syn::Type`, not on rendered text, because the text matchers are formatting-sensitive.**
The diagnostic post-processors are `&str -> Option<String>` functions written against rustc's
*diagnostic* rendering; `prettyplease` breaks a long generic list across lines and ends it with a
trailing comma before the closing `>`, which the `Symbol!` matcher's final `>` check rejects. On this
document's own rectangle example that left `Symbol!("width")` resugared and `height` a raw spine,
purely because the longer name was the one that got wrapped. Matching a parsed type is immune to
formatting.

**Its passes are separate whole-tree visits, and each spine is folded outermost-first.** The
separation is the `Nil` overlap hazard [Resugaring](resugaring.md#the-rules-every-resugaring-follows)
describes, at its sharpest here because a visitor recurses innermost-first: one combined visitor
rewrites a `Symbol`'s terminating `Nil` to `Product![]` before examining the enclosing `Symbol`, and
every field name silently stays raw. The direction is a second, subtler form of the same problem — a
spine's *tail is itself a spine*, so folding the innermost cell first leaves a two-element list as
`Cons<A, Product![B]>`. Each pass therefore folds a spine before recursing, and recurses into the
elements it collected.

**Only real syntax is emitted, so two diagnostic-only forms are deliberately not produced.** A
diagnostic folds an all-field list on to `Struct! { width: f64, … }` and renders an open-ended path
with a trailing `.*`; neither is a real CGP macro, and this pass writes source, where every construct
shown should be something the programmer could have written. So a field list stays
`Product![Field<Symbol!("width"), f64>, …]` and an open-ended path stays its raw chain. This is the
one place the three implementations deliberately differ, and the reason is recorded in both documents.

**The printer's spacing has to be corrected after the fact.** A resugared construct is a macro call
whose body holds ordinary types, and the printer lays a macro body out token by token — it cannot know
the body is a type list, so it prints `Product![Multiply < Symbol!("foo") >]`. Its rules cannot be
coaxed into the conventional form, because the space *before* a token is the printer's decision and an
identifier cannot ask for it to be dropped. So the crate ends with one narrow text pass that removes
spaces inside the bodies of the four macros it emits, never inside a literal, and never anywhere else
in the program.

Sugar the *user* wrote needs no attention at all — a hand-written
`PipeHandlers<Product![StepOne, StepTwo]>` comes out as written, because the CGP macro that copied it
never expanded it.

One implementation hazard is worth recording, because it fails loudly rather than subtly. Stripping
the `cgp::macro_prelude::` qualifier changes a path's segment count, and a **qualified** path —
`<__Provider__ as DelegateComponent<C>>::Delegate`, which generated CGP code is full of — carries an
index saying where its qualifier ends. Dropping segments without moving that index leaves the two
inconsistent, and the printer asserts on exactly that, panicking the driver mid-compilation. The strip
corrects the index for both node kinds that carry one, and two unit tests pin it.

The one option the first slice needs is how much path noise to remove. CGP macros emit fully-qualified
`::cgp::macro_prelude::Symbol<…>`, which is pure noise to a reader and which the resugaring must see
past anyway, so **the `cgp::macro_prelude::` qualifier is stripped by default** (`cgp`'s own
`strip_macro_prelude` does this for its expansion snapshots, and the diagnostic chain's
`strip_cgp_prefixes` does it for errors). General module qualifiers are **kept**, unlike in a
diagnostic, because in source they carry information a reader may want.

The switch that turns the stripping off — [`ExpandOptions`](../../crates/cargo-cgp-expand/src/options.rs)
carries it, so the output can be kept compilable — has no command-line flag yet. A `--verbatim` flag
would be the front-end's *first* tool-specific argument, so it has to be recognized and removed before
the rest are forwarded to cargo; that is the one place where `expand` cannot stay the pure
pass-through `check` is, and it is why the flag waits until someone wants it.

## What the implementation settled

Three of the four questions this design was uncertain about are now answered by running it, and the
answers are recorded here rather than left as open items.

- **Cargo tolerates the unit that produces no artifact.** It exits 0, and simply re-runs that unit on
  the next invocation, since the fingerprint is never satisfied. A `cargo cgp check` afterwards in the
  same target directory re-checks the crate normally, so the two commands share the directory without
  confusing each other.
- **No spurious compiler warning appears.** Not setting an output file avoids `IgnoringOutDir`
  entirely, and nothing else surfaced, so the front-end needs none of the stderr filtering
  `cargo-expand` does in `ignore_cargo_err`.
- **Module qualifiers stay.** Real output reads well with them, and in source they carry information a
  diagnostic's reader does not need.

One question remains a judgement call for a first user to sharpen: **what the second slice adds
first** — syntax highlighting and paging (`cargo-expand` uses `bat`), the `--verbatim` flag above, or
the selectivity below. (The `--item` filter, which was on this list, is built.)

Running it across the whole fixture tree also surfaced one boundary worth knowing about. An `open`
statement's per-key wiring entry is generic over the *rest* of the path, so the impl the macro writes
keys on `PathCons<ItemEncoderComponent, PathCons<u64, __Wildcard__>>` — a chain whose tail is a named
type parameter rather than `Nil`. The path fold declines it, correctly: there is no `Path!` syntax for
"this prefix followed by anything", and inventing one would break the real-syntax rule above. So the
one construct an expansion still shows raw is an `open`-generated key, and the surrounding
`RedirectLookup<App, Path!(@ItemEncoderComponent)>` beside it reads normally.

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
`--ugly` raw mode, no theme selection, and no re-declared cargo flags of its own — including the
manifest `default-run` lookup it uses to pick a default binary, where this command leaves the ambiguity
for cargo to report.

The item filter both tools have differs in two ways, each following from a choice made above.
`cargo-expand` takes the path as a **bare positional** and matches it with `syn-select`; this command
takes it as `--item`, because forwarding the rest of the arguments to cargo means a bare word is
ambiguous, and it matches with its own [`select`](resugaring.md) rules — which add the CGP-shaped third
rule, selecting the impls *of* a named trait, so naming a component's provider trait shows what the
component generates.

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

The resugaring is pure text-to-text, so it is pinned by unit tests with no compiler; the launching and
printing halves need a real compilation, so they are exercised by running the command.

- [`crates/cargo-cgp-expand/tests/resugar.rs`](../../crates/cargo-cgp-expand/tests/resugar.rs) — the
  resugaring end to end over hand-written expanded source: each construct and its decline cases, the
  two overlap hazards (a `Symbol`'s and a path's terminating `Nil`, either of which a combined pass
  would turn into `Product![]`), the outermost-first fold (a two-element field list), the tightened
  spacing of a generic element, the diagnostic-only forms deliberately *not* emitted (a field list kept
  as a `Product!`, an open-ended path left raw), the prelude strip including both qualified-path
  shapes, an ordinary module qualifier kept, and unparsable input returned verbatim.
- [`crates/cargo-cgp-driver/tests/expand.rs`](../../crates/cargo-cgp-driver/tests/expand.rs) — the
  marker flag: the request and its path recovered, the flag stripped from the vector the compiler
  sees, an ordinary compilation carrying no request, and an empty path declining.
- [`crates/cargo-cgp/tests/help.rs`](../../crates/cargo-cgp/tests/help.rs) — that the expand help
  documents `--item` and its three rules, names the target-selection trap, and points at
  `cargo rustc --help` for the forwarded options; and that the top-level help points at both
  subcommands' helps, since their options come from different places.
- [`crates/cargo-cgp/tests/expand.rs`](../../crates/cargo-cgp/tests/expand.rs) — the front-end's pure
  helpers: the profile detection in both forms (and a `--bin release` value that must not count as
  one), the per-process output path, and the `--item` extraction — both spellings, everything else
  forwarded untouched, a bare word left to cargo, and the two rejection shapes (a flag-shaped mistake
  names the flag, a malformed path names the path).
- [`crates/cargo-cgp-expand/tests/select.rs`](../../crates/cargo-cgp-expand/tests/select.rs) — the
  three selection rules over a two-module program: a module giving its unwrapped contents, a type
  giving its declaration and the impls for it, an unqualified path reaching into a module, a trait
  giving the impls of it, a path matching nothing yielding nothing, and the path-shape parser.

- The [UI suite](testing.md#three-passes-per-fixture) runs the command over **every fixture** as its
  third pass, diffing the expansion against a committed `<name>.expand.rs`. That is the end-to-end
  coverage of the whole path — the marker flag, the `after_expansion` hook, the `pprust` call, the
  resugaring, the file handoff — and it doubles as a wide test of the resugaring itself, since the
  fixtures between them exercise every CGP construct the tool knows. It also pins the expansion of
  code the tool's own diagnostics are about, so a `.cgp.stderr` and the `.expand.rs` beside it can be
  read together.

## Source

- [`crates/cargo-cgp/src/expand/`](../../crates/cargo-cgp/src/expand) — the front-end subcommand:
  `command.rs` runs the wrapped `cargo rustc` with the marker flags and prints what the driver wrote,
  `item.rs` takes `--item <path>` out of the forwarded arguments and checks its shape, `output.rs`
  holds the per-process output path and reads it back, and `profile.rs` decides whether
  `--profile check` is added.
- [`crates/cargo-cgp/src/launch/`](../../crates/cargo-cgp/src/launch) — the setup both subcommands
  share, lifted out of the old `check/` directory: `command.rs` builds the wrapped cargo command (the
  preflight, the forced toolchain, the sysroot and dylib path), with `driver_path.rs`, `dylib.rs`,
  `preflight.rs`, `sysroot.rs`, and `target_dir.rs` beside it. [`check.rs`](../../crates/cargo-cgp/src/check.rs)
  is now just the `cargo check` run.
- [`crates/cargo-cgp/src/run.rs`](../../crates/cargo-cgp/src/run.rs),
  [`help.rs`](../../crates/cargo-cgp/src/help.rs),
  [`config.rs`](../../crates/cargo-cgp/src/config.rs) — the dispatch arm, the top-level help line and
  the subcommand's own `expand_help_text`, and the front-end's half of the marker-flag contract
  (`EXPAND_FLAG`).
- [`crates/cargo-cgp-driver/src/expand/`](../../crates/cargo-cgp-driver/src/expand) — expand mode:
  `request.rs` takes the marker flag out of the argument vector, and `print.rs` prints the expanded
  crate through `pprust::print_crate`, resugars it, and writes it out.
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs),
  [`run.rs`](../../crates/cargo-cgp-driver/src/run.rs),
  [`config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the `after_expansion` hook, the request
  threaded into the callbacks (which also drops the diagnostic flag injections in expand mode), and the
  driver's half of the flag contract.
- [`crates/cargo-cgp-expand/`](../../crates/cargo-cgp-expand) — the rustc-free resugaring crate:
  `source.rs` is the entry point, `resugar/` holds one module per pass (`symbol`, `path`, `list`,
  `strip`, `spacing`) over the shared `parts.rs`, `select.rs` narrows an expansion to one module or
  item, and `options.rs` carries the item path and the strip switch. See [Resugaring](resugaring.md).
