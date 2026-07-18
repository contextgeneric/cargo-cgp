# rustc diagnostic internals

`cargo-cgp` exists to read and rewrite the compiler's diagnostics, so it has to know how rustc builds
those diagnostics and — the point of this document — where rustc deliberately *drops* information
while building them. A CGP error is unusually vulnerable to that dropping: its types are enormous
nested spines (`Symbol<6, Chars<'h', …>>`, `Cons<A, Cons<B, …>>`), and rustc's diagnostic machinery
compresses long or repetitive types on the assumption that a human does not want to read them in
full. For a downstream consumer trying to recover a root cause, that compression can erase the one
fact that matters. This document maps the compiler code responsible, so the next agent can find a
suppression point, decide whether to defeat it, and confirm it against the source rather than
rediscovering it.

This document has a second subject that the same foothold forces on the tool. Because the driver
reaches the live `TyCtxt` and re-runs compiler code — the trait solver, and the queries it pulls in —
from *inside* `emit_diagnostic`, it runs that code in a place rustc never expected arbitrary compiler
work to run. Doing so can **panic the compiler**, and the panics are not hypothetical: two distinct
ones have already been hit and fixed. The information-dropping map below is what the tool must defeat;
the [panic hazards](#panic-hazards-running-compiler-code-inside-the-emitter) section near the end is
what it must *avoid*, and it is the more dangerous half — an elision produces a worse message, but a
re-entrancy panic aborts the whole compilation.

Every path here points into the read-only Rust checkout at
[`../external/rust`](../../../external/rust), pinned to the same nightly the driver embeds (see
[Toolchain and `rustc_private`](../../AGENTS.md#toolchain-and-rustc_private)). These are unstable
internals that move between nightlies, so this document names **functions**, not line numbers; when a
reference has drifted, grep for the function name to relocate it. Treat the file as read-only — nothing
in `../external/rust` is edited to serve this tool.

## Where diagnostics are built

Three areas of the compiler produce the text of a CGP error, and knowing which one owns a given piece
of output tells you where to look when that piece is wrong.

**Type and const printing lives in
[`rustc_middle/src/ty/print/pretty.rs`](../../../external/rust/compiler/rustc_middle/src/ty/print/pretty.rs).**
This is what turns a `ty::Ty` into a string — the `PrettyPrinter` trait and the `FmtPrinter` that
backs `Ty`'s `Display`. It is where a `Symbol` or a `Chars` chain becomes the characters you see, and
where the `pretty_print_const_scalar_int` function renders a `char` const literal such as `'h'`. It
also holds `should_print_verbose`, the gate on the compiler's *internal* debug printing, discussed
under [the two verbosity switches](#the-two-verbosity-switches-verbose-versus-verbose-internals) below.

**Trait-error reporting lives in
[`rustc_trait_selection/src/error_reporting/`](../../../external/rust/compiler/rustc_trait_selection/src/error_reporting).**
The `traits/fulfillment_errors.rs` module builds the `E0277` "trait bound is not satisfied" errors
that dominate CGP output, including the two-line *similar impl* hint ("the trait `X` is not
implemented … but trait `Y` is"). The `infer/mod.rs` module holds the type-diffing helpers `cmp` and
`cmp_traits`, which the similar-impl hint calls to render two related types side by side with their
differences highlighted. Most of the suppression this document catalogs lives in these two files.

**Diagnostic structure lives in `rustc_errors`.** A built diagnostic is a tree of styled strings
(`DiagStyledString`, whose parts are "normal" or "highlighted"); the reporting code above pushes
strings into that structure, and the emitter later renders it to the terminal. When a suppression
replaces text with `_` or `...`, it is doing so at the point the string is pushed, not at emission —
so the elided information never reaches the diagnostic at all, which is exactly why a text-only
post-processor cannot recover it.

## The two verbosity switches: `--verbose` versus `-Zverbose-internals`

rustc has two distinct verbosity flags, and the difference between them is the single most useful fact
in this document, because one is the surgical lever `cargo-cgp` wants and the other is not.

**`opts.verbose` is the broad switch, and it is what gates every *elision* suppression below.** It is
set in [`rustc_session/src/config.rs`](../../../external/rust/compiler/rustc_session/src/config.rs)
(in `build_session_options`) by `matches.opt_present("verbose") || unstable_opts.verbose_internals` —
so either the stable `--verbose` / `-v` flag *or* the unstable `-Zverbose-internals` turns it on. The
elision sites read it as `tcx.sess.opts.verbose`.

**`verbose_internals` is the narrow switch, and it gates the compiler's internal debug printing.** It
is the `-Zverbose-internals` unstable option, read through `Session::verbose_internals()`, and it is
what `should_print_verbose` in `pretty.rs` returns. Turning it on makes the pretty-printer emit debug
detail meant for compiler developers: disambiguator suffixes on paths, region and inference-variable
identifiers, explicit `for<…>` binders, and more.

The consequence is that **`--verbose` defeats the elisions without triggering the internal debug
noise, whereas `-Zverbose-internals` does both.** Because `-Zverbose-internals` sets `opts.verbose` as
a side effect, it would also un-elide — but it drags in the disambiguators and region ids that make a
type like `Symbol<6/#0, …>` harder to read, not easier. `cargo-cgp` wants the full field name, not the
compiler's internal bookkeeping, so it injects the stable `--verbose`. This is the reasoning behind
[`config::VERBOSE_FLAG`](../../crates/cargo-cgp-driver/src/config.rs); the injection itself, and the
reason it is skipped for cargo's info queries, is documented in
[The driver](driver.md#un-eliding-the-diagnostic).

## The suppression points

Each suppression below drops information from a CGP diagnostic. For each, this section names the
function, says what it erases and under what condition, and states whether the driver's `--verbose`
injection defeats it. The first three are text elisions gated on `opts.verbose`, so `--verbose` closes
all three; the fourth is left deliberately on.

### Matching generic arguments are elided to `_`

The similar-impl hint diffs the two traits it names and replaces every generic argument they *share*
with `_`. This is the bug that first motivated the `--verbose` injection. The diffing is done by `cmp`
and `cmp_traits` in `infer/mod.rs`; inside `cmp`, the local helper `maybe_highlight` is the culprit:
given two values at the same position, it highlights them if they differ, but if they are equal it
pushes the literal string `"_"` for both — unless `opts.verbose` is set. The type and lifetime arms of
`cmp` do the same, pushing `"_"` or `"'_"` for arguments that compare equal.

For ordinary types this is a readable shorthand — `Foo<_, Bar>` says "the first argument is the same
in both, look at the second". For a CGP `Symbol` it is destructive. The
[`acceptable/fields/base_area_1`](../../tests/ui/acceptable/fields/base_area_1.rs) fixture asks for a `height` field on
a `Rectangle` that only has `width`, so rustc diffs `HasField<Symbol!("height")>` against
`HasField<Symbol!("width")>`. The two symbols share the character `'h'` (fifth in `height`, last in
`width`), so `maybe_highlight` collapsed that shared `'h'` to `_` in *both* names, printing `h,e,i,g,_,t`
and `w,i,d,t,_`. The field name could not be read back from the text at all. Under `--verbose` the same
hint prints both symbols in full, which is what the fixture's blessed
[`.cgp.stderr`](../../tests/ui/acceptable/fields/base_area_1.cgp.stderr) now records.

### A long type is truncated and written to a file

When a type's printed form grows long, rustc does not print it inline; it writes the full form to a
`long-type-<hash>.txt` file and shows a length-limited version in the diagnostic, ending the note with
a pointer to the file. The decision is made by `short_string_namespace` in
[`rustc_middle/src/ty/error.rs`](../../../external/rust/compiler/rustc_middle/src/ty/error.rs): it
truncates when `write_long_types_to_disk` is on (the default) *and* `opts.verbose` is off, and when the
regular rendering exceeds a fraction of the diagnostic width. Reporting code opts a type into this
treatment by calling `tcx.short_string(ty, diag.long_ty_path())` rather than printing the type
directly, which is common in the `E0277` builders.

For CGP this truncation lands routinely, because a `Symbol` for a normal field name or a `Product!`
list is already long enough to trip the width test. It shows up as the `...` inside a nested type — for
instance a similar-impl *candidate list* that reads `HasField<Symbol<4, Chars<'m', Chars<'a', Chars<'s',
...>>>>>` — where the tail of the name is replaced by `...` and moved to the file. `--verbose` sets
`opts.verbose`, so `short_string_namespace` returns the full type inline and the field name is never
sent to a temp file. (When the type *is* diverted to a file, the UI-test harness normalizes the
hash-named path away; see [Testing](testing.md).)

### Extra impl candidates are collapsed to "and N others"

When more than nine impls could satisfy a bound, rustc prints a few and summarizes the rest. The cutoff
is in `report_similar_impl_candidates` in `fulfillment_errors.rs`: the number of candidates it shows is
the full count when `candidates.len() <= 9 || opts.verbose`, and otherwise a truncated head with an
"and N others" tail. A context wired with many components can exceed nine `HasField`/provider impls, so
the candidate that names the relevant field can fall into the collapsed remainder. `--verbose` shows
every candidate. This one is less central than the first two — the primary "not implemented for" line
usually still carries the failing bound — but it is the same `opts.verbose` switch, so the injection
gets it for free.

### Internal debug printing is left off (deliberately)

The `should_print_verbose` branches throughout `pretty.rs` are *not* something `cargo-cgp` turns on.
They are gated on `verbose_internals`, not `opts.verbose`, so `--verbose` leaves them off, and that is
intentional: they add disambiguator suffixes (`#0`), region and inference-variable ids, and explicit
binders that make CGP types longer and less readable without recovering any field name. If a future
need arises for one of these details, `-Zverbose-internals` is the lever — but it should be a
considered choice, because it changes the shape of nearly every type in the output and would re-bless
the whole UI suite.

## A different kind of suppression: the trait solver

Not every hidden cause is a printing elision. The archetypal CGP hidden error — a consumer-method call
whose real missing bound the *default* trait solver never even computes — is suppressed earlier, during
trait solving, not during printing. No verbosity flag recovers it, because the information was never
produced. The driver defeats it with a different injected flag, `-Znext-solver=globally`, which selects
a solver that does descend to the leaf bound. That mechanism, and why it is a separate lever from
`--verbose`, is documented in [The driver](driver.md#choosing-the-trait-solver);
it is mentioned here only so the two are not confused. The rule of thumb: if the cause is *present but
compressed* in the text, it is a printing elision and `--verbose` is the lever; if the cause is *absent*
because the solver stopped short, it is a solver problem and `-Znext-solver` is the lever.

## Panic hazards: running compiler code inside the emitter

The driver runs its typed root-cause resolver from inside a diagnostic emitter, and that placement is
the source of every panic risk the tool has hit. rustc calls the emitter to *render* a finished
diagnostic; it does not expect the emitter to turn around and drive the trait solver, force queries,
or relate types. The resolver does exactly that (it re-runs the failing obligation through a fresh
`InferCtxt`), so it lives in territory rustc's invariants do not cover, and it must respect a few of
those invariants by hand or the whole compilation aborts. This section collects the hazards so the
next agent recognizes them before adding a new compiler interaction to the resolver; the per-feature
detail lives in [Typed root-cause resolution](typed-root-cause-resolution.md) and
[The driver](driver.md), and this is the index to it.

### Re-entering the diagnostic context (`lock was already held`)

**The `DiagCtxt` is guarded by a non-reentrant lock, and the emitter runs with that lock held, so any
compiler operation the resolver triggers that *emits a diagnostic* re-enters the lock and aborts the
compiler.** `DiagCtxt::emit_diagnostic` in
[`rustc_errors/src/lib.rs`](../../../external/rust/compiler/rustc_errors/src/lib.rs) is
`self.inner.borrow_mut()` — a plain `Lock`, single-entry, which panics with `lock was already held`
on re-entry rather than blocking. rustc holds that borrow across the whole emitter call, so a nested
emission from within the resolver is fatal. There is no escape valve: `make_silent` and `set_emitter`
would swap in a no-op emitter, but they *also* take `inner.borrow_mut()`, so they cannot even be
called while the lock is held, and swapping the emitter would not matter anyway — the panic is the
borrow, not the emitter behind it.

The trap is indirect, because the resolver does not emit diagnostics itself. It forces a **query**,
and the query emits. Two query paths are known to do this. Forcing `predicates_of` on an impl whose
`where`-clause has a type that fails to lower makes `gather_explicit_predicates_of` →
`lower_ty` → `resolve_type_relative_path` report a resolution error (an `E0599` such as `no variant
named …`) the first time it is computed; if the resolver forces that query while a diagnostic is being
emitted, the query's own emission re-enters the lock. Forcing `tcx.typeck` is worse: it is a body-level
query that *replays its cached diagnostics* when forced, so forcing it from the emitter re-emits every
error in that body.

The rule that keeps the resolver safe follows from *when* these queries are still uncached. A query
that already ran during analysis is cached, and forcing it from the emitter just returns the cached
value without re-emitting — which is why the resolver can freely solve obligations against providers
that type-checking already visited. The danger is only a query computed for the *first time* during
emission, which happens when the diagnostic being emitted was itself produced *during* that query —
i.e. during the collection / type-lowering phase, not the later trait-solving phase. So the resolver
**declines diagnostics from the collection phase**: a resolution-class `E0599` (`no variant named …`,
`no associated item …`) is emitted mid-`predicates_of`, so `try_resolve` filters it out before running
the solver, keeping only the method-bounds `E0599` (which is reported during method resolution, after
collection). The same reasoning bans forcing `tcx.typeck` from the emitter outright. `E0271`/`E0277`
are trait-solving failures reported after collection, so the queries their resolution forces are
already cached and they are safe to trace. The gate that encodes this is in
[Typed root-cause resolution](typed-root-cause-resolution.md); the `tcx.typeck` ban is why one
use-site shape is deliberately out of reach, documented there under the resolver's boundaries.

### Feeding escaping bound variables to inference (`has_escaping_bound_vars`)

**Relating a term that carries an escaping bound variable trips an assertion in rustc's generalizer,
so any term the resolver hands to `ocx.eq` (or any inference relation) must have its binders
instantiated first.** `InferCtxt::generalize` in
[`rustc_infer/src/infer/relate/generalize.rs`](../../../external/rust/compiler/rustc_infer/src/infer/relate/generalize.rs)
asserts `!source_term.has_escaping_bound_vars()`. The natural mistake is to reach a `ty::TraitRef` out
of a `ty::PolyTraitPredicate` with `skip_binder()` and relate it: for a higher-ranked obligation such
as `Self: for<'a> CanSerializeValue<&'a Value>` — the shape a recursive provider like
`SerializeIterator` carries — `skip_binder()` leaves the `'a` bound variable escaping, and the relation
panics mid-emit. The fix is to instantiate the binder before relating, with placeholders via
`InferCtxt::enter_forall_and_leak_universe` (rigid placeholder regions, which also let a *nested*
higher-ranked hop's projection normalize) rather than `skip_binder()`. Whenever a new resolver stage
relates a type reached through a `Binder`, it must instantiate first.

### Contaminating one inference context with another's variables

**A stray inference or region variable from one `InferCtxt` panics another, so every predicate the
resolver moves between contexts must be instantiated, normalized, and region-erased at the boundary.**
The resolver builds a fresh `InferCtxt`/`ObligationCtxt` per impl it tests, and a variable minted in one
must never appear in another. This is not a distinct compiler bug so much as a discipline the resolver
keeps: it region-erases predicates before they cross into a fresh context, and drops a predicate still
carrying non-region inference variables rather than re-checking it. The mechanics — the
`fresh_args_for_item`-plus-unification impl match, the normalize-and-erase step — are in
[Typed root-cause resolution](typed-root-cause-resolution.md); the note here is only that the erase is
load-bearing, not cosmetic.

The through-line is that **the resolver's one safe compiler interaction is the trait solver on a fresh
`InferCtxt`**, proven re-entrant-safe against a diagnostic being emitted mid-solve, and everything else
— forcing a global query that can emit, relating an un-instantiated binder, leaking a variable across
contexts — must be avoided or carefully neutralized. A new compiler interaction added to the resolver
should be checked against all three hazards before it is trusted.

## Finding these again, and going further

Two grep patterns relocate the elision sites when a nightly bump moves them. Searching the compiler for
`opts.verbose` (excluding `verbose_internals`) enumerates every elision gate — today that is
`maybe_highlight` and the lifetime/type arms in `infer/mod.rs`, `short_string_namespace` in
`ty/error.rs`, and `report_similar_impl_candidates` in `fulfillment_errors.rs`. Searching `pretty.rs`
for `should_print_verbose` enumerates the internal-debug branches `--verbose` intentionally leaves off.

Both suppression families are defeated with the coarse argument lever — inject a flag, change nothing
else. The finer work uses the compiler's own state directly rather than its printed output, and the
driver's [`Callbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs) is where that happens: its
`config` hook installs a custom emitter that reaches the live `TyCtxt` (from thread-local scope) and
rewrites diagnostics before they are serialized. The first use of it renames CGP wiring notes to name
the consumer and provider traits behind a component marker
([Naming the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker)),
and the same seam can re-run trait fulfillment through the `InferCtxt` / `ObligationCtxt` API to
reconstruct a chain the printed form renders tersely. This is the reason the foothold is worth having:
once inside the compiler, the full `Symbol` is an interned type you can read exactly, whatever the
printer chose to show.

## Further reading

- [Diagnostic and subdiagnostic structs — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/diagnostics/diagnostic-structs.html)
  — how a diagnostic is assembled, the level above the string-pushing this document describes.
- [Errors and lints — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
  — the emission pipeline and the crates involved.
