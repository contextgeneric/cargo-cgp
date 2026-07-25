# Resugaring

CGP's type-level constructs are written as macros and compiled as deeply nested types, so a
programmer writes `Symbol!("height")` and the compiler talks back about
`Symbol<6, Chars<'h', Chars<'e', …>>>`; resugaring is the family of transforms that reverses each of
those expansions, so every CGP construct cargo-cgp shows a reader is spelled the way they wrote it.

This document is the single home for that logic. The transforms are spread across the tool by
necessity — three implementations exist because there are three different inputs to resugar — and the
rules they must all obey are the same, so gathering them here keeps one description of each construct
instead of one per call site. The subsystems that *drive* them describe only that: how the driver
applies its diagnostic chain is [Error processing](error-processing.md), how the resolver labels a
dependency chain is [walking to the root cause](typed-resolution-walk.md), and how the expand command
post-processes a printed crate is [The expand command](expand-command.md).

## Why resugaring exists

Every CGP type-level macro is sugar over a nested type, and nothing downstream of the macro remembers
the sugar. `Symbol!("height")` expands to a length and a right-folded character list; `Product![A, B]`
to a `Cons` spine; `Path!(@app.Greeter)` to a `PathCons` chain. The macro emits the expanded form, the
compiler interns it, and every later mention — a trait bound in an error, a wiring key in a coherence
conflict, a type in a printed expansion — is that expanded form. A reader who never wrote it is then
asked to read it, and the encodings are large: a six-character field name is a seven-level type, and
the compiler will happily break it across four lines mid-spine.

Resugaring is worth the machinery because the expanded form does not merely look bad, it *hides the
fact the reader needs*. The whole point of a missing-field error is the field's name, and that name is
the one thing a `Chars` chain buries — worse still when rustc elides part of the spine, which is why
the driver also injects `--verbose` (see
[rustc diagnostic internals](rustc-diagnostic-internals.md#the-suppression-points)). Un-eliding gets
the characters into the text; resugaring turns them back into a name.

**A resugaring must never lie, and that constraint shapes every rule below.** Rewriting
`Symbol<6, …>` to `Symbol!("height")` is a claim about what the programmer wrote, so a transform that
guesses is worse than one that does nothing: a reader who is shown a construct that was never in the
source has no way to tell, and will go looking for it. So every transform matches **exactly or
declines**, leaving the raw type in place, and the sections below say precisely what "exactly" means
for each construct.

## The three approaches

Resugaring happens at three points in the tool, and each works over a different representation of the
same construct: an interned compiler type, a rendered string, and a parsed syntax tree. They agree on
*what* every construct resugars to — that is the whole subject of
[The resugarings](#the-resugarings) below — and they share nothing at all in *how* they recognize it,
because the three inputs offer different information and demand different output. The sub-sections here
describe each approach and what forces it to be its own implementation; the section after them collects
the reasons the three cannot be collapsed into one.

### Typed: `Ty<'tcx>` inside the resolver

The typed approach resugars a construct while the driver still holds it as a real compiler type, before
any string exists. It is
[`render_ty`](../../crates/cargo-cgp-driver/src/resolve/label/render_ty.rs) plus
[`decode_symbol`](../../crates/cargo-cgp-driver/src/resolve/cgp_item.rs), and every dependency-chain
label and classified leaf the [typed root-cause resolver](typed-root-cause-resolution.md) produces goes
through it, so a reshaped diagnostic is already sugared when it is built.

Its distinguishing asset is **identity**: a `Ty<'tcx>` carries the `DefId` of the type it names, so
every cell can be checked against the crate that defines it — `Cons`/`Nil`/`Symbol`/`Chars` in
`cgp-base-types`, `Either`/`Void`/`Field` in `cgp-field`. A type merely *spelled* `Cons` in some other
crate is therefore never resugared, whatever its shape, and this is what lets the typed pass be the
liberal one: the `DefId` check does the discriminating, so the structural rules need only describe the
shape rather than defend against coincidence.

Two more things are available here and nowhere else. A character in a `Symbol!` spine is a **const
generic argument**, which this pass reads out of its valtree as an exact `char` — the other two must
parse a printed literal and give up on anything escaped. And an inference **placeholder is visible as
such**, so the call-site anchor's stand-in for an untyped argument renders as the `_` a programmer
would write instead of rustc's internal `!N` form.

The cost is where it must live. Reaching a `Ty<'tcx>` means linking the compiler, so this
implementation sits inside the `rustc_private` half of the tool and cannot be unit-tested on an
ordinary toolchain — its coverage comes from UI fixtures compiled end to end.

### Text: `&str` in the rustc-free crate

The text approach resugars constructs inside a diagnostic that has already been rendered to strings. It
is the [`postprocess`](../../crates/cargo-cgp-error-processing/src/postprocess) chain, and it runs on
every diagnostic the driver emits: on one the resolver *declined* it is the entire cleanup, and on one
the resolver reshaped it tidies the compiler-formatted types a constructed message still embeds.

It exists as a separate implementation because **by the time it runs, the types are gone**. This is the
point worth being precise about, since the compiler is still right there in the process: rustc records
no mapping from a substring of a rendered message back to the `Ty` it was printed from, so there is
nothing for the typed pass to be handed. A declined diagnostic is a tree of strings, and a string is
all its transforms can consult.

Working on text forces two kinds of machinery the typed pass never needs. The first is **structural
parsing by hand**: balancing angle, paren, and bracket nesting to find where a cell's head ends and its
tail begins, skipping string literals so a `>` or `,` inside one does not mislead the scan, and
tolerating whatever whitespace and line breaks rustc's renderer inserted mid-type. The second is
**name-collision defence**, since there is no `DefId` to ask: the list pass must check that a cell name
stands alone rather than ending a longer identifier, or the `Cons<` at the tail of `PathCons<` would be
read as a spine cell.

One hazard belongs to this approach alone. rustc splits a message into styled fragments, and its
"similar impl" hint splits at *every difference between two types* — shredding a spine so that no
fragment contains a whole construct to match. So the driver post-processes each fragment and then reads
them again as the single line they render as (see
[Error processing](error-processing.md#how-the-driver-applies-the-transforms)).

The compensating benefit is testability: a transform is a pure `&str -> Option<String>` function, so
the whole catalogue of match and decline cases is exercised as ordinary library tests with no compiler
in the loop.

### Syntax tree: `syn::Type` in the expand command

The syntax-tree approach resugars a whole printed crate rather than a diagnostic, and it is the
[`cargo-cgp-expand`](../../crates/cargo-cgp-expand) crate that `cargo cgp expand` drives (see
[The expand command](expand-command.md#the-rustc-free-resugaring)). Its input is the source text the
compiler's pretty-printer produced, re-parsed with `syn`.

It cannot reuse the typed pass, because the expanded AST it works from is **pre-resolution** — there are
no `DefId`s and no `Ty`s, only names — and because it must live outside the compiler linkage to stay
testable. So, like the text pass, it recognizes constructs by name and must match exactly.

It cannot reuse the text pass either, and that is a measured result rather than a preference. The text
matchers were written against rustc's *diagnostic* rendering, and on printed source they are
formatting-sensitive: `prettyplease` breaks a long generic list across lines and ends it with a trailing
comma before the closing `>`, which the `Symbol!` matcher's final `>` check rejects. On a rectangle
example that left `Symbol!("width")` resugared and `height` a raw spine, purely because the longer name
was the one that got wrapped. Matching `syn::Type` is immune to formatting, because the structure is
already parsed.

Four demands are specific to this approach, and each shapes the code. Its **output must be a syntax
node**, not a string: `Symbol!("height")` is built as a macro-call type the printer then formats, where
the other two approaches simply emit text. Its passes must **fold each spine outermost-first**, because
a visitor recurses innermost-first and a spine's tail is itself a spine — folding the inner cell first
leaves a two-element list as `Cons<A, Product![B]>` — so each pass folds before recursing and then
recurses into the elements it collected. That same innermost-first instinct is what makes the shared
`Nil` overlap hazard bite hardest here, which is why the passes stay separate whole-tree visits (see
[the rules](#the-rules-every-resugaring-follows)). And it **emits only real syntax**, which is the one
place the three implementations deliberately differ — the next section.

A fifth demand is not about matching at all but about printing, and it is why the crate ends with a
text pass despite being built to avoid text matching. A resugared construct is a macro call whose body
holds ordinary types, and the printer lays a macro body out token by token: it cannot know the body is a
type list, so it prints `Product![Multiply < Symbol!("foo") >]`. Its spacing rules cannot be coaxed into
the conventional form, because the space *before* a token is the printer's decision and an identifier
cannot ask for it to be dropped. So one narrow pass removes spaces inside the bodies of the four macros
the crate emits — never inside a literal, never anywhere else in the program — which only ever removes a
space and so cannot alter meaning.

### Only source output is held to real syntax

Two of the forms a diagnostic shows are **presentation-only**: the `Struct! { … }` / `Enum! { … }`
record forms an all-field list folds to, and the trailing `.*` wildcard an open-ended path takes. No
such CGP macros exist and neither would parse back. They earn their place in a diagnostic because the
alternative is unreadable — a chain of `Field` cells, a raw `PathCons` spine — and a diagnostic is prose
about the program, not the program.

Source output is different, and the syntax-tree pass therefore **does not emit either**. An expansion is
read as code: a reader may copy a line out of it, and every construct in it should be something they
could have written. So a field list stays `Product![Field<Symbol!("width"), f64>, …]` — real, writable,
and true to the type — and an open-ended path stays its raw chain. (In an expansion the open tail is a
named generic parameter anyway, not the `_` a diagnostic renders.)

This is the one sanctioned divergence between the implementations. The rule that generalizes it: the two
diagnostic passes may show a form that reads better than it parses, and the source pass may not.

### Why the three cannot be one

The differences are not incidental; each approach is pinned by what its input offers and what its output
has to be. Laid side by side, the four axes that decide the implementation:

| | Typed | Text | Syntax tree |
|---|---|---|---|
| **Input** | interned `Ty<'tcx>` | rendered diagnostic string | re-parsed printed source |
| **Identity available** | `DefId` — the defining crate | name only | name only |
| **Structure available** | the type itself | none; parsed by hand | the `syn` tree |
| **Output form** | a string | a string | a syntax node |
| **May live outside `rustc_private`** | no | yes | yes |

Reading down the columns explains every divergence in the code. The typed pass is short because the
compiler hands it both identity and structure; the text pass is the longest because it must rebuild
structure from characters *and* defend against name collisions without identity; the syntax-tree pass is
in between, given structure but not identity, and owes the extra step of constructing printable nodes.
No pair of them shares an input, so no pair can share a matcher.

What they do share is the *specification* — every rule in the next two sections — and that is the
standing obligation this document exists to serve: a change to what a construct resugars to, or to what
counts as an exact match, belongs in all three at once.

## The rules every resugaring follows

Four rules hold across all the constructs, and they are stated once here rather than repeated per
section.

**Match exactly or decline.** Each transform reconstructs the surface form only when the type matches
the expansion the macro produces, level for level, and leaves the raw type alone otherwise. Declining
is a normal outcome, not a failure — an inference placeholder mid-spine, a wrong terminator, or a
same-named foreign type all take that path.

**The order is fixed, because the transforms read each other's output.** Module qualifiers are
stripped first, so everything after matches bare names; then `Symbol!`, whose output the `Path!` pass
reads as a segment and the list pass reads as a `Field`'s tag; then `Path!`; then the lists. The text
chain in [`postprocess/chain.rs`](../../crates/cargo-cgp-error-processing/src/postprocess/chain.rs)
sequences them in that order, and any other implementation must too.

That ordering is load-bearing in one way that is easy to get wrong. **`Nil` terminates three different
spines** — a `Symbol`'s character list, a `PathCons` path, and an empty `Cons` list — so a pass that
rewrites a bare `Nil` before the enclosing construct is examined destroys the enclosing match. On a
syntax tree, where a visitor naturally recurses innermost-first, this bites immediately: one combined
visitor turns a `Symbol`'s terminating `Nil` into `Product![]`, after which the `Symbol` no longer
matches and *every field name silently stays raw*. Separate whole-tree passes in chain order are what
keep the constructs apart, and no implementation may fold them into one traversal.

**A few surface forms are presentation-only.** `Struct! { … }`, `Enum! { … }`, and `Path!`'s trailing
`.*` wildcard are not real CGP macros and would not parse back. They exist because the shape they
describe reads far better than the spine, and they are the one place resugaring shows something other
than what the programmer could have written — so they are shown in a *diagnostic* only, never in source
output, per [only source output is held to real syntax](#only-source-output-is-held-to-real-syntax).
Every other output is real, writable syntax everywhere.

**An empty spine is left as its terminator.** A bare `Nil` or `Void` is not rewritten to `Product![]`
or `Sum![]`: the terminator alone reads as the plain type it is, and resugaring it would mean claiming
an empty list where a type was written. Both list implementations require a first cell before they
begin.

## The resugarings

Each construct below is one transform in each implementation that owns it. The sections share a shape:
the CGP source a programmer writes, the type it expands to, what resugaring turns it back into, and
what makes the transform decline.

### `Symbol!` — the type-level string

`Symbol!` encodes a string as a type so it can key a trait, which is how every CGP field name travels
([`Symbol!` reference](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/symbol.md)).
A getter declares the fields it reads:

```rust
#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}
```

and its generated blanket impl requires `HasField<Symbol!("height")>` on the context. What the
compiler holds, and prints, is the expansion — a byte length and a right-folded character list closed
by `Nil`:

```text
Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>
```

Resugaring reverses it to `Symbol!("height")`, and in a message the tool constructs itself the name is
usually unwrapped further into prose — `` missing field `height` on `Rectangle` `` — since a
root-cause lead reads better naming the field than the tag type.

The match is exact in three ways, and any of them failing leaves the type as it stands. The **declared
length must equal the decoded string's byte length**, because `Symbol!` bakes in `str::len()` rather
than a character count, so a length that disagrees means this is not a `Symbol!` expansion. The
**spine must be `Chars` all the way down to `Nil`**, with nothing else in the chain. And each `Chars`
head must be a **single plain character literal**: an escaped or multi-character literal declines,
rather than being decoded by guesswork. The empty string is a legitimate match — `Symbol<0, Nil>`
resugars to `Symbol!("")`.

The typed implementation is `decode_symbol`, which additionally checks `Symbol`, `Chars`, and `Nil`
by `DefId` against `cgp-base-types` and reads each character out of the const argument's valtree;
the text implementation is
[`resugar_symbol`](../../crates/cargo-cgp-error-processing/src/postprocess/resugar_symbol.rs).

### `Path!` — the type-level path

`Path!` encodes a routing path as a type-level list, which is what a namespace or an `open` statement
dispatches on
([`Path!` reference](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/path.md)).
A context that dispatches one component per encoded type writes:

```rust
delegate_components! {
    App {
        open ItemEncoderComponent;

        @ItemEncoderComponent.Vec<u8>: EncodeHex,
    }
}
```

The wiring key that entry generates is a `PathCons` chain, which is what shows up in a coherence
conflict or a missing-wiring leaf:

```text
PathCons<ItemEncoderComponent, PathCons<Vec<u8>, Nil>>
```

Resugaring reverses it to the path the programmer wrote, `@ItemEncoderComponent.Vec<u8>`. **Which of
two forms it renders is chosen by the caller**, through `resugar_path`'s `wrap` parameter: a message
the tool *constructs* — a coded header, a root-cause note — wants the bare `@…`, because there it reads
as a path the sentence names; a message the tool merely *cleaned up* wants the macro form
`Path!(@ItemEncoderComponent.Vec<u8>)`, because there a raw type is being shown back in source form.
The emitter passes `wrap` accordingly, bare on a rewrite and wrapped on a fallback.

Segments are rendered back the way `Path!` classifies them going forward, which is what makes the
round trip faithful. `Path!` turns a lowercase, non-primitive identifier into a `Symbol` and keeps
every other segment verbatim as a named type, so resugaring unwraps a `Symbol!("app")` head to the
bare segment `app` only when `app` is such an identifier, and renders every other head verbatim —
a capitalized component marker, a primitive like `u32`, or a compound value type an `open` statement
dispatches on (`Vec<u8>`, `&Coord`, `DateTime<Utc>`). A namespace path therefore reads as written:

```text
PathCons<Symbol!("app"), PathCons<GreeterComponent, Nil>>   →   @app.GreeterComponent
```

Two segment shapes decline, leaving the whole spine raw rather than risking a mangled path. A
**module-qualified** segment folds to its final identifier only when every part is a plain identifier
and the tail is a type — the [module strip](#the-path-strips-that-run-first) normally removes the
qualifier before this pass, so a residual `::` means the spine is not the bare form `Path!` writes. A
**bare lowercase identifier** declines outright: `Path!` would have encoded it as a `Symbol`, so
meeting one as a plain type is ambiguous.

One non-`Nil` tail is still resugared, and it is the presentation-only case. An **open-ended path**
ends in a generic "rest of path" parameter instead of `Nil`, which rustc prints as the inference
placeholder `_` — the shape that appears in the conflicting-wiring `E0119` blocks over a duplicated
`@`-path key. That tail becomes a trailing `.*` wildcard:

```text
PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), _>>   →   Path!(@foo.bar.*)
```

`.*` is not `Path!` syntax and would not parse back, but it reads far better than the spine and says
what the path means: it matches any continuation. Only a bare `_` triggers it, since `_` is never a
concrete segment; any other non-`Nil` tail declines. Being presentation-only, it is a diagnostic form:
the source pass leaves an open-ended chain raw.

The text implementation is
[`resugar_path`](../../crates/cargo-cgp-error-processing/src/postprocess/resugar_path.rs), which also
mirrors the macro's own `is_primitive_type` rule so the two classifications cannot drift.

### `Product!` and `Sum!` — the list spines

`Product!` and `Sum!` encode a type list and a type sum, the spines behind every field list, variant
list, and handler pipeline
([`Product!`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/product.md),
[`Sum!`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/sum.md)). A wiring
table names one directly:

```rust
delegate_components! {
    App {
        ComputerComponent: PipeHandlers<Product![Multiply<Symbol!("foo")>, Add<Symbol!("bar")>]>,
    }
}
```

Both expand to right-nested spines, a product through `Cons` to `Nil` and a sum through `Either` to
`Void`, and resugaring folds each back to the flat list:

```text
Cons<u64, Cons<String, Nil>>        →   Product![u64, String]
Either<u64, Either<f64, Void>>      →   Sum![u64, f64]
```

The value is highest in a dependency chain, where a recursive provider walks a list one cell at a
time: without resugaring, each hop of the chain restates a slightly shorter spine and the reader has to
diff them to see progress; with it, each hop names the list it is working on.

Three details make the match exact. The spine must **close on its own terminator** — `Nil` for a
product, `Void` for a sum — so an open-ended or wrongly-terminated spine is left alone, and a tail
cell must be the *whole* remaining tail rather than something a trailing token follows. Elements are
**resugared recursively**, so a `Sum!` nested inside a `Product!`, or a `Symbol!` inside an element,
folds in turn. And the text pass takes one precaution the typed pass gets for free from its `DefId`
check: it requires the cell name to stand alone rather than end a longer identifier, so the `Cons<` at
the end of `PathCons<` is never mistaken for a spine cell.

The typed implementation is `render_ty`'s `cgp_spine`; the text implementation is
[`resugar_lists`](../../crates/cargo-cgp-error-processing/src/postprocess/resugar_list.rs).

### `Struct!` and `Enum!` — a list of named fields

When every element of a list spine is a **named field**, the list is not really a list to the reader —
it is a record or a variant set, and resugaring folds it one step further. This is the shape
`#[derive(HasFields)]` produces:

```rust
#[derive(HasFields)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}
```

whose `Fields` associated type is a product of `Field` cells, each pairing a `Symbol!` name tag with
the field's type. A product of fields becomes a struct and a sum of fields becomes an enum:

```text
Cons<Field<Symbol!("width"), f64>, Cons<Field<Symbol!("height"), f64>, Nil>>
    →   Struct! { width: f64, height: f64 }

Either<Field<Symbol!("Rect"), u64>, Either<Field<Symbol!("Circle"), f64>, Void>>
    →   Enum! { Rect(u64), Circle(f64) }
```

`Struct!` and `Enum!` are presentation-only — no such CGP macros exist — and they are worth that
exception because the alternative is unreadable: a real record provider's chain hop otherwise names a
chain of `Field` cells several fields long, where `Struct! { message_id: u64, date: DateTime<Utc>, … }`
says the same thing at a glance. For that same reason they are a *diagnostic* form only; source output
stops at the `Product!`/`Sum!` list, which is real syntax
([why](#only-source-output-is-held-to-real-syntax)).

The fold applies only when **every** element is a bare `Field` cell whose tag is a plain symbol
literal; a single element that is not drops the whole list back to its plain `Product!`/`Sum!` form,
because a half-record form would misrepresent the shape. Field values are resugared recursively, so a
nested record folds in turn.

### The path strips that run first

Two transforms are not resugarings but must run before them, since every rule above matches bare type
names. They also do real work of their own: rustc prints types fully qualified, and in a CGP
diagnostic those qualifiers are noise the reader did not write.

[`strip_module_paths`](../../crates/cargo-cgp-error-processing/src/postprocess/strip_modules.rs)
collapses every `a::b::C` identifier run to its final segment:

```text
contexts::app::MockApp                            →   MockApp
interfaces::types::QuantityTypeProviderComponent  →   QuantityTypeProviderComponent
f64: std::cmp::Eq                                 →   f64: Eq
```

It scans the ASCII identifier run by byte but copies every other character whole by its UTF-8 width,
so multi-byte text — a rendered dependency tree's `└─` — is never split into invalid bytes. It skips
string literals, so a name inside a `Symbol!("a::b")` is never mangled, and it leaves a turbofish, an
associated-type `>::Assoc` tail, and a lone identifier alone.

[`strip_cgp_prefixes`](../../crates/cargo-cgp-error-processing/src/postprocess/strip_prefixes.rs)
removes the specific CGP re-export paths (`cgp::prelude::`, `cgp::macro_prelude::`, `cgp::cgp_core::`,
`cgp::cgp_extra::`), turning `cgp::prelude::Chars` into `Chars`. With the general module strip running
first this is largely redundant, and it is kept as the explicit CGP-specific fallback.

The expand command treats these differently from a diagnostic, and deliberately so: it strips the
`cgp::macro_prelude::` qualifier the macros emit, because that is pure noise, but **keeps** general
module qualifiers, because in printed source a qualifier carries information a reader may want (see
[The expand command](expand-command.md#the-rustc-free-resugaring)).

## What is left unresugared

Three things a reader might expect to see resugared are deliberately not, and each has a reason worth
recording so nobody adds it as a "missing" transform.

An **empty spine** stays as its terminator, per the shared rule above: a bare `Nil` or `Void` is a
plain type in the reader's source too. A **type-level index** (`Index<0>`, the tuple-field tag) needs
nothing — `Index<N>` *is* the surface form a programmer writes, so there is no expansion to reverse.
And a **`Life<'a>` lifetime lift** is decoded rather than resugared: the resolver reads the region back
out of it when rebuilding an obligation whose trait wants a real lifetime
([anchoring the starting obligation](typed-resolution-anchors.md)), and the label then shows the
lifetime in its ordinary position rather than as a `Life<…>` argument.

One rendering decision sits beside the resugarings in `render_ty` without being one. A **call-site
placeholder** — the rigid stand-in the [call-site anchor](typed-resolution-call-site.md) seeds for a
parameter the call leaves to inference — renders as the `_` a programmer would write, never as rustc's
internal `!N` form, and a tuple is rendered element by element so a placeholder nested inside one
still prints as `_`.

## Tests

Resugaring is pure text-to-text or type-to-text, so most of it is pinned by unit tests with no
compiler, and the UI suite pins the typed pass end to end.

- [`crates/cargo-cgp-error-processing/tests/postprocess.rs`](../../crates/cargo-cgp-error-processing/tests/postprocess.rs) —
  the text implementations over crafted inputs: an exactly-matched `Symbol!` and the empty symbol, a
  wrong length and a foreign `Symbol` left alone, the `PathCons` → `@…`/`Path!(@…)` forms across symbol,
  type, primitive, generic-value, and reference-value segments, the open `_` tail folded to `.*`, the
  qualified-tail and lowercase-segment declines, the `Product!`/`Sum!` folds, the `Struct!`/`Enum!`
  record forms, a mixed list kept as a plain product, a nested list, a non-terminating spine declined,
  the `Cons`-inside-`PathCons` guard, the module and CGP-prefix strips (including the multi-byte
  box-drawing and string-literal cases), and the `postprocess_message` chain end to end.
- [`tests/ui/acceptable/fields/base_area_1`](../../tests/ui/acceptable/fields/base_area_1.rs) — the
  typed `Symbol!` decode in a real diagnostic: the missing field is named `height` in the lead, where
  raw rustc renders a `Chars` spine (and elides part of it).
- [`record_field_chain`](../../tests/ui/acceptable/wiring/missing-wiring/record_field_chain.rs) — the
  typed `Cons`/`Nil` → `Struct! { … }` fold, in a record provider's chain.
- [`sum_variant_chain`](../../tests/ui/acceptable/wiring/missing-wiring/sum_variant_chain.rs) and
  [`enum_variant_chain`](../../tests/ui/acceptable/wiring/missing-wiring/enum_variant_chain.rs) — the sum
  spine as a plain `Sum![…]` list of bare types, and as an `Enum! { … }` of named variants.
- The path fixtures —
  [`unregistered_prefix_path`](../../tests/ui/acceptable/resolution/unregistered_prefix_path.rs),
  [`qualified_prefix_path`](../../tests/ui/acceptable/wiring/namespace-paths/qualified_prefix_path.rs)
  (a module-qualified path folded to a clean `@…`), and
  [`open_missing_type_key`](../../tests/ui/acceptable/wiring/namespace-paths/open_missing_type_key.rs) —
  the typed path rendering, in a namespace redirect and an `open` dispatch key.

- [`crates/cargo-cgp-expand/tests/resugar.rs`](../../crates/cargo-cgp-expand/tests/resugar.rs) — the
  syntax-tree implementation over hand-written expanded source, run through the whole pipeline the
  driver runs: each construct and its decline cases, both `Nil` overlap hazards, the outermost-first
  fold, the tightened spacing of a generic element, the two diagnostic-only forms confirmed *absent*,
  and the prelude strip including the qualified-path shapes whose index it has to correct.

What is *not* guarded: no test asserts that the three implementations agree with each other on the same
construct — consistency between them rests on this document — and the expand command has no end-to-end
fixture harness yet (see [The expand command](expand-command.md#tests)).

## Source

- [`crates/cargo-cgp-driver/src/resolve/label/render_ty.rs`](../../crates/cargo-cgp-driver/src/resolve/label/render_ty.rs)
  — the typed pass: the `cgp_spine` walk, the `named_fields` record/variant fold, the recursive element
  rendering, and the placeholder/tuple rendering.
- [`crates/cargo-cgp-driver/src/resolve/cgp_item.rs`](../../crates/cargo-cgp-driver/src/resolve/cgp_item.rs)
  — `decode_symbol`, the typed `Symbol!` decode, and `is_cgp_item`, the `DefId` anchor every typed
  recognition goes through.
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate
  and type-name constants the typed pass anchors against (`CONS_TYPE`, `NIL_TYPE`, `EITHER_TYPE`,
  `VOID_TYPE`, `FIELD_TYPE`, `PATH_CONS_TYPE`, and their defining crates).
- [`crates/cargo-cgp-error-processing/src/postprocess/`](../../crates/cargo-cgp-error-processing/src/postprocess)
  — the text passes, one module each: `resugar_symbol.rs`, `resugar_path.rs`, `resugar_list.rs`,
  `strip_modules.rs`, `strip_prefixes.rs`, and `chain.rs`, which sequences them.
- [`crates/cargo-cgp-expand/src/resugar/`](../../crates/cargo-cgp-expand/src/resugar) — the
  syntax-tree passes, one module per construct (`symbol.rs`, `path.rs`, `list.rs`), plus `strip.rs`
  (the prelude qualifier and the qualified-path index it corrects), `spacing.rs` (the post-print
  tightening), `parts.rs` (the shared shape reading and macro-node building), and `file.rs`, which
  sequences the passes. Driven by [`source.rs`](../../crates/cargo-cgp-expand/src/source.rs); see
  [The expand command](expand-command.md).

## Further reading

The CGP construct references define each expansion this document reverses, and are the ground truth
whenever a rule here needs checking against the macro's own behaviour.

- [`Symbol!`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/symbol.md) — the
  length-plus-`Chars` encoding, including why the length is `str::len()`.
- [`Path!`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/path.md) — the
  segment classification the `Path!` resugaring mirrors, and the `PathCons` spine.
- [`Product!`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/product.md) and
  [`Sum!`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/macros/sum.md) — the `Cons`/`Nil`
  and `Either`/`Void` spines.
- [`Field`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/types/field.md) and
  [`HasFields`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/traits/has_fields.md) —
  the named-field cell and the shape the `Struct!`/`Enum!` fold describes.
