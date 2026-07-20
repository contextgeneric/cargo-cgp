# Hidden root cause

This document catalogs the cases where `cargo-cgp` does not emit enough information to identify the
root cause of an error — the cases where the cause cannot be recovered from the output alone, no
matter how that output is later processed. It is the tool's most important measure of correctness,
because everything else the tool might do to a diagnostic assumes the cause is present to work with.

The test this document applies is *sufficiency*, not readability. `cargo-cgp`'s role is to produce
output from which a downstream consumer — a formatter, an IDE, or an AI agent — can recover the root
cause; how that output is spelled or laid out is a separate concern, handled in
[usability](usability.md). An issue belongs here only when the root cause is genuinely absent from,
or unrecoverable from, `cargo-cgp`'s output, so that no downstream processing of the text could
reconstruct it — and only when a fixture under [`tests/ui/`](../../tests/ui) reproduces it. These are
the cases that justify the tool's compiler-internal foothold: where the ordinary text output has lost
information, only a tool reading the compiler's own state can put it back.

## No reproduced cases remain

There is currently **no reproduced hidden-root-cause case**. The two archetypes this category was
built around are both defeated today, each by a flag the driver injects, so neither hides a cause the
output cannot recover. Per the rule in [the issues README](README.md) — a class with no reproducing
fixture counts as resolved — this document keeps no open entry; it records the two defeated archetypes
below so a future agent recognizes them, and states what a genuinely new case would have to look like
to belong here.

This is not merely the absence of a *known* case: every post-codegen compile-fail class CGP produces —
the cases migrated from `cgp`'s former compile-fail suite into [`tests/ui/`](../../tests/ui), across the
`acceptable/` and `usability/` categories — has been run through cargo-cgp and snapshotted, and every
reproducible class carries its root cause in the tool's output. So the CGP error catalog's own hidden
class (an unsatisfied dependency reached by a consumer-method call) surfaces here too, and nothing
across the catalog lands in this category. The one class that produces no usable diagnostic under
cargo-cgp, `inheritance_cycle`, does so because the next-gen solver *accepts* it (a missing error,
tracked as a solver caveat in [The driver](../implementation/driver.md#choosing-the-trait-solver)), not
because a cause is suppressed.

The two archetypes are defeated by *different* levers, and the distinction is the useful one to carry
forward. A cause can be hidden because the compiler **never computed it** or because the compiler
computed it and then **elided it while printing** — the first is a trait-solver problem, the second a
diagnostic-printing problem, and they need different flags. Both levers are argument injections
documented in [The error pipeline](../implementation/error-pipeline.md); the printing side is
mapped function-by-function in
[rustc diagnostic internals](../implementation/rustc-diagnostic-internals.md).

### Defeated: a cause the default trait solver never computed

The archetypal hidden failure was a wiring mistake exercised by a direct consumer-method call, where
the compiler reported only that a method's bounds were unsatisfied (`E0599`) without ever naming the
failed dependency. On that path the *default* solver's method-resolution heuristic bottoms out at the
provider trait and does not compute the real missing leaf bound at all, so no amount of text
processing could recover it — the leaf was never in the diagnostic. `cargo-cgp` defeats this by
injecting `-Znext-solver=globally`, which descends to the leaf and even renders CGP's own "add
`#[derive(HasField)]`" hint. The fixture that once showed the hidden form,
[`unsatisfied_dependency`](../../tests/ui/acceptable/use-site/unsatisfied_dependency.rs), now
demonstrates the recovered cause — and, since the typed resolver also cleaned up its once-verbose,
misleading presentation into a `[CGP-E001]` headline over a `root cause:` note, it lives under
[`acceptable/`](../../tests/ui/acceptable) rather than as a usability case.

### Defeated: a field name the printer elided a character from

The second archetype was a field name the compiler printed with a character missing, so the name
could not be read back from the diagnostic at all. When a provider needs a field the context lacks and
the context has a *near-miss* field, rustc reports the unmet bound through its two-line "similar impl
exists" hint and, in that hint, diffs the two `HasField` symbols and replaces every generic argument
they share with `_`. In [`base_area_1`](../../tests/ui/acceptable/fields/base_area_1.rs) a
`Rectangle` that has `width` but not `height` made the two symbols share the character `'h'`, and that
shared `'h'` was collapsed to `_` in *both* names — `h,e,i,g,_,t` for `height` — so the field name was
absent from the text, not merely encoded. `cargo-cgp` defeats this by injecting `--verbose`, which
turns off the compiler's matching-argument elision (along with two related compressions) so the full
`Symbol` always prints. That fixture, too, now lives under [`acceptable/`](../../tests/ui/acceptable):
the typed resolver reads the field name straight from the `Symbol!` and states it plainly, so the
readability burden it once carried is gone as well.

## What a new case would look like

Should a wiring mistake be found whose cause survives *both* levers — the next-gen solver still does
not compute it, and `--verbose` still does not print it — it belongs here, with a fixture under a
recreated [`tests/ui/hidden-root-cause/`](../../tests/ui) directory that proves the cause is
unrecoverable from the text. The upstream catalog's
[hidden-versus-surfaced axis](https://github.com/contextgeneric/cgp/blob/main/docs/errors/README.md#the-central-axis-hidden-versus-surfaced)
is the same distinction from the CGP side and is the reference for judging which classes carry their
cause and which suppress it.

The second thing that stays out is by scope: plain Rust or Cargo diagnostics that have nothing to do
with CGP are not `cargo-cgp`'s to recover and are not tracked here.
