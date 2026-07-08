# Error transformation

`cargo-cgp` exists to turn rustc's raw diagnostics for CGP code into readable, root-cause-first
errors; this document records *how* it transforms them. Today it does so in one coarse way — choosing
the trait solver — and this is the place every later transformation is written down as it is added.

## Why a transformation layer exists

A CGP macro expands to ordinary Rust, so most CGP mistakes are caught not by the macro but by the
compiler type-checking the generated code, where the diagnostic is shaped by CGP's machinery in ways
that make it hard to read: a single mistake can cascade across generated types the programmer never
wrote, and the real cause is often buried or hidden entirely. The [CGP error
catalog](../../../cgp/docs/errors/README.md) maps those classes — which hide the root cause, which
surface it, and where the cause sits. `cargo-cgp`'s job is to take rustc's output for those classes
and re-present it with the cause first; this document is the running account of the transformations
it applies to get there.

## Where transformation happens

All transformation happens in the driver, because the driver runs the real compiler in-process
through `rustc_driver` (the front-end only wires the driver into cargo — see
[Executable structure](executable-structure.md)). That gives two levers, and each transformation
below is one or the other:

- **The rustc arguments the driver injects** before compilation. This is coarse — it changes how the
  compiler produces diagnostics — but needs no diagnostic parsing. The current transformation is of
  this kind.
- **The driver's [`Callbacks`](../../crates/cargo-cgp-driver/src/callbacks.rs)**, which can read the
  compiler's diagnostics after analysis and rewrite them. This is the finer lever and is where the
  planned transformations will live. It is currently unused (`CgpCallbacks` is empty).

## Choosing the trait solver (current)

The driver injects `-Znext-solver=globally` into every workspace-crate compilation
([`config::NEXT_SOLVER_FLAG`](../../crates/cargo-cgp-driver/src/config.rs), applied in
[`args::rustc_args`](../../crates/cargo-cgp-driver/src/args.rs)), turning on the next-generation trait
solver. This is an argument-lever transformation, and it targets the class of CGP error whose root
cause the *default* solver hides.

When a provider's impl-side dependency is unmet — say a `name` field the context does not carry — and
the failure is reached by calling the consumer method directly, the default solver's
method-resolution path bottoms out at the provider trait (`Person: Greeter<Person>`) and never
reports the real missing leaf bound. That leaf is not merely omitted from the printed diagnostic: on
this path the default solver does not compute it at all (confirmed by tracing
`rustc_hir_typeck::method::probe` — the leaf predicate never appears).

The next-generation solver does compute it. Under `-Znext-solver=globally` the same mistake reports
`HasField is not implemented for Person with the field: Symbol<…"name"…>`, names the concrete
`Person: HasField<Symbol!("name")>` bound, and even renders CGP's own
`#[diagnostic::on_unimplemented]` hint ("add `#[derive(HasField)]`"). So merely compiling the
workspace crate under the new solver un-hides the cause — no diagnostic parsing required yet. The
flag is scoped to workspace crates (only they go through the driver), so dependencies still build
with the default solver. The before/after is pinned by the
[`hidden/unsatisfied_dependency`](../../tests/ui/hidden/unsatisfied_dependency.stderr) UI snapshot.

### Caveats

Two things follow from changing the solver. The new solver is not perfectly compatible with the old
one (see [Significant changes and quirks](https://rustc-dev-guide.rust-lang.org/solve/significant-changes.html)),
so in principle a crate could compile under a plain `cargo check` yet report a spurious error under
`cargo cgp check`, or the reverse; this is the accepted cost of using the new solver as the
diagnostic engine, and an explicit `-Znext-solver` on the command line still overrides the injection.
Separately, the richer cross-crate diagnostics name absolute paths (the `cgp` checkout) and can point
at a hash-named temp file for an elided long type — volatile details the UI-test harness normalizes
away, described in [Testing](testing.md).

## Planned transformations

The transformations below are not implemented yet. Record each here as it lands — with its lever, the
error class it targets, and a before/after pinned by a UI fixture — so this document stays the map of
what `cargo-cgp` does to a diagnostic. They are listed roughly in order of expected value:

- **Decode CGP's type-level encodings.** A type like `Symbol<4, Chars<'n', Chars<'a', Chars<'m',
  Chars<'e', Nil>>>>>` is the field name `"name"`; `Product![A, B]` is `Cons<A, Cons<B, Nil>>`; and so
  on. Rewriting these back to their surface form (`Symbol!("name")`, `Product![…]`) in a diagnostic
  would remove most of the visual noise a CGP error carries. This is a `Callbacks`-lever transform
  over the emitted diagnostics.
- **Lift the root cause and collapse the cascade.** For the verbose-cascade class, one deep mistake is
  reported at every transitively dependent provider; the transform would detect the repetition,
  present the single root cause first, and summarize or drop the repeats. Needs the `Callbacks` lever
  and the catalog's per-class signatures.
- **Recover chains the solver still renders tersely.** Where even the next-gen solver stops short,
  the driver can re-run trait fulfillment on the failing obligation via the compiler's
  `InferCtxt`/`ObligationCtxt` API to extract the full derived-obligation chain (the surfaced form
  that `check_components!` forces at the source level), then attach it to the diagnostic.
- **Map diagnostics to catalog classes.** Recognize an error's [catalog](../../../cgp/docs/errors/README.md)
  class from its shape and annotate it (or link the relevant catalog entry), so a user is told which
  kind of CGP mistake they are looking at.

## Comparison with Clippy

Clippy is also a diagnostic transformer, and it works entirely on the `Callbacks` lever: its
`config` callback calls `register_lints` to add lint passes that run during the same compilation (see
[`external/rust-clippy/src/driver.rs`](../../../external/rust-clippy/src/driver.rs)). `cargo-cgp`'s
first transformation is coarser and of the other lever — a solver flag, not a callback — because it
buys a large improvement for no diagnostic-parsing work. The planned transformations move onto the
same `Callbacks` hook Clippy uses. The aims differ, though: Clippy *adds* new diagnostics (lints),
whereas `cargo-cgp` *rewrites and clarifies* the diagnostics rustc already produces.

## Tests

- [`tests/ui/hidden/unsatisfied_dependency.stderr`](../../tests/ui/hidden/unsatisfied_dependency.stderr) —
  the UI snapshot that pins the un-hidden output the solver switch produces; it is the current
  regression guard for this transformation.
- [`crates/cargo-cgp-driver/tests/args.rs`](../../crates/cargo-cgp-driver/tests/args.rs) — tests that
  the injected flag is appended when absent and skipped when the invocation already sets
  `-Znext-solver`.

## Source

- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) —
  `NEXT_SOLVER_FLAG`, with the rationale.
- [`crates/cargo-cgp-driver/src/args.rs`](../../crates/cargo-cgp-driver/src/args.rs) — injects the
  flag into the rustc argument vector.
- [`crates/cargo-cgp-driver/src/run.rs`](../../crates/cargo-cgp-driver/src/run.rs) — passes the flag
  set to `rustc_args`.
- [`crates/cargo-cgp-driver/src/callbacks.rs`](../../crates/cargo-cgp-driver/src/callbacks.rs) — the
  (empty) hook the planned `Callbacks`-lever transformations will use.

## Further reading

- [Next-gen trait solving — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html)
  — what the solver `-Znext-solver` selects is and how it evaluates goals.
- [Significant changes and quirks — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/solve/significant-changes.html)
  — how the new solver differs from the old, the basis for the compatibility caveat above.
