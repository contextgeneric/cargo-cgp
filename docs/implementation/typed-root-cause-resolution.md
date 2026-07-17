# Typed root-cause resolution

The driver can transform a CGP check-failure diagnostic into the root-cause **dependency tree** it
recovers by *asking the compiler* rather than by reading the error text. This document describes that
resolver: what it transforms, why it runs where it does, how it recovers the chain through the trait
solver, and the boundaries that decide when it steps aside and lets the older text-rewrite path
handle the diagnostic instead.

This is the second, deeper transformation the driver's emitter performs. The first —
[naming the traits behind a component marker](driver.md#naming-the-traits-behind-a-component-marker) —
edits the compiler's diagnostic in place, renaming its wording. The resolver goes further: it walks
the wiring's typed obligations to the real root cause and renders the whole chain as a `cargo tree`,
rewriting the main message into its coded CGP form when it is an identified CGP class and replacing
the sub-notes with one `root cause:` note per leaf. It realizes the compiler-state enrichment that
[The driver](driver.md) and [The error pipeline](error-pipeline.md) anticipated.

## What it transforms, and what it leaves alone

The resolver considers **any diagnostic that names a CGP wiring or field trait**
(`CanUseComponent`, `IsProviderFor`, or `HasField`) and **every `E0599`, `E0271`, and `E0277`** — not
only wiring-worded ones. This breadth is deliberate: a failure that names *no* CGP construct can still
be a consequence of a CGP component failing — a hand-written `Send`-recovery wrapper whose `async fn`
forwards to a wired method fails with an `E0271` opaque-future mismatch, a downstream trait bound needs
a method the context cannot supply — and the resolver traces the dependency chain to find out, treating
the error as CGP-related exactly when a CGP component failure sits in that chain. It recovers a starting
obligation three ways: from a `check_components!` entry when the caret sits on one; from a hand-written
*`impl Trait for Context` block* the failure surfaces inside (below) — which anchors the manual-wrapper
cascade, since its raw `E0271`/`E0277` land inside such a block whose supertrait is a CGP consumer
trait; and otherwise from the *use site* of a broken consumer-method call (below). Any of the three
walks the wiring obligations down to the terminal unmet bound(s) they rest on, and the diagnostic is
then transformed in two independent halves. A diagnostic whose chain reaches no CGP cause declines and
passes through untouched.

**The main message is rewritten — and stamped with its [CGP error code](../error-code.md) — only when
it is an identified CGP class.** An unsatisfied `CanUseComponent` bound is a
[check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md), so it becomes
`[CGP-E001] the consumer trait \`CanCalculateArea\` is not implemented for context \`Rectangle\``,
worded from the typed resolution (whose full-path marker keys make the consumer name exact even for
same-named components); a consumer-method `E0599`, whose text names no wiring trait, gets the same
`CGP-E001` form from the resolution; an unsatisfied `IsProviderFor` bound becomes the `[CGP-E002]`
provider form via the text rewrite; and a field-type mismatch — an `E0271` the resolver traced to a
`HasField` projection — becomes the `[CGP-E003]` field form
`[CGP-E003] expected a \`height\` field of type \`f64\` on \`Rectangle\`, but found \`i32\``, the
expected type read from the failing projection and the actual type queried from the struct. The
rewrite restates the same fact readably — the caret is re-aimed at the failing entry alone, and the
diagnostic's own Rust code (`E0277`, `E0599`, `E0271`) is kept. A main message that is *not* a CGP
class — an ordinary bound (`f64: Eq`) the next-gen solver descended to *that is itself a recovered
root cause* — stays rustc's own, header, labels, and caret untouched
([`ordinary_bound_unsatisfied`](../../tests/ui/acceptable/resolution/ordinary_bound_unsatisfied.rs)).
But when the solver descended to an ordinary bound that is *not* one of the recovered leaves — a
mid-chain symptom, such as a getter bound on a request whose real cause is a missing wiring a level
down — the `CGP-E001` consumer header is truer than that symptom and replaces it
([`foreign_getter_missing_wiring`](../../tests/ui/acceptable/resolution/foreign_getter_missing_wiring.rs)).

**The sub-messages are replaced either way.** rustc's obligation-chain notes, supplementary help, and
structured suggestions are discarded, and each recovered root cause becomes one `= note:` opening with
a `root cause:` lead naming its leaf, followed by the dependency chain rendered as a tree. Both the
lead and every tree entry carry a [`CGP-E1xx`/`CGP-E2xx` code](../error-code.md) — the wording
examples below omit the prefix for brevity, but each `root cause:` lead reads `root cause: [CGP-E1xx]
…` in the real output. The chain
**repeats the root cause as its terminal leaf**, so it always bottoms out *at* the cause rather than
one step before it — the same shape whether the leaf is a missing field, an unmet bound, a missing
wiring, or a redirect. Every path in these rewritten messages renders as a bare `@app.GreeterComponent`
(the `Path!(@…)` macro form is reserved for the resugaring fallback), and module qualifiers are
stripped throughout, so `contexts::app::MockApp` reads as `MockApp`. The lead is
worded by *why* the leaf is unmet, which the resolver decides by inspecting the actual struct a
`HasField` bound lands on (detailed under
[How the root cause is recovered](#how-the-root-cause-is-recovered)): a genuinely absent field reads as
`root cause: missing field \`height\` on \`Rectangle\`` (no `context` qualifier, since `HasField` can
land on any struct), while a field the struct *does* carry but has not derived reads as `root cause:
accessor trait \`HasField\` with field \`name\` is not implemented for \`Person\``, with a separate
`help` naming the fix — `make sure that \`#[derive(HasField)]\` is used for \`Person\`` (pointed at the
`Deref` target when the field is only reachable through one; that is exactly the
[`missing_has_field_derive` fixture](../../tests/ui/acceptable/fields/missing_has_field_derive.rs), the
present-but-underived case a plain "missing field" would misdescribe). A component the context does
not wire at all — an unmet `DelegateComponent<Marker>` on the context — is the wiring counterpart of a
missing field, reading as `root cause: context \`App\` does not contain any delegate entry for
\`BarProviderComponent\`` and naming the component marker the programmer writes to fix it
([`basic_missing_wiring`](../../tests/ui/acceptable/wiring/missing-wiring/basic_missing_wiring.rs)). A
namespace redirect that resolves to nothing — a `RedirectLookup<Ctx, Path>` whose `Path` the context
does not terminate, surfacing as an unmet namespace-lookup bound (`Path: DefaultNamespace<Ctx>`, or a
user `cgp_namespace!` trait) — is the path-keyed counterpart, reading the **same** way with the path
as the key (`root cause: context \`App\` does not contain any delegate entry for
\`@app.finance.types.QuantityTypeProviderComponent\``); its dependency chain renders each
`RedirectLookup` hop as `redirect lookup to \`@…\` in \`App\`` and ends on that same
missing-delegate statement, so a multi-layer redirect reads as its successive hops down to the
unterminated path ([`unregistered_prefix_path`](../../tests/ui/acceptable/resolution/unregistered_prefix_path.rs),
[`qualified_prefix_path`](../../tests/ui/acceptable/wiring/namespace-paths/qualified_prefix_path.rs),
[`multi_redirect_missing`](../../tests/ui/acceptable/wiring/namespace-paths/multi_redirect_missing.rs)).
Any other leaf restates its
bound — `root cause: the trait bound \`f64: Eq\` is not satisfied` (module qualifiers stripped) —
except when the kept
main message already states that very bound, where the lead would only repeat the header and the note
carries the chain alone
([`ordinary_bound_unsatisfied`](../../tests/ui/acceptable/resolution/ordinary_bound_unsatisfied.rs)).

**A use-site failure is handled the same way, once its obligation is recovered.** CGP wiring is lazy,
so a broken provider dependency often surfaces not at a check but where the consumer method is *called*
— `person.greet()` on a `Person` that cannot satisfy `HasName` — as an `E0599` "method exists but its
trait bounds were not satisfied". There is no check impl to anchor on, so the resolver instead recovers
the context type from the diagnostic's own spans (the "method not found for this struct" span lands on
`Person`'s definition) and re-checks every component that context wires. So
[`missing_dependency`](../../tests/ui/acceptable/use-site/missing_dependency.rs) and
[`unsatisfied_dependency`](../../tests/ui/acceptable/use-site/unsatisfied_dependency.rs)
become `[CGP-E001] the consumer trait \`CanGreet\` is not implemented for context \`Person\`` (the
code stays `E0599`, since the error is still rustc's) over a `missing field` root-cause note — and the
misleading "this is an associated function… use associated function syntax instead" advice, which the
method probe emits for CGP's `self`-less provider methods, is dropped with the rest of rustc's
sub-notes. The ordinary-bound use-site case
([`use-site/ordinary_bound_unsatisfied`](../../tests/ui/acceptable/use-site/ordinary_bound_unsatisfied.rs))
gets the same `CGP-E001` header over a `root cause: the trait bound \`f64: std::cmp::Eq\` is not
satisfied` note.

A field whose name matches but whose **type** does not is handled as its own class. With the derive
present, the `HasField` trait bound still holds (for the wrong `Value`), and only the associated-type
projection `<Rectangle as HasField<Symbol!("height")>>::Value == f64` fails — an `E0271`. The
trait-clause walk cannot see the projection directly, so when a branch reaches an impl whose
trait-clause dependencies all hold, the resolver inspects that impl's own predicates for an unmet
`HasField` projection; finding one, it takes the expected type from the projection and the actual type
from the struct and yields a field-type-mismatch leaf, worded as the `[CGP-E003]` header (detailed
under [How the root cause is recovered](#how-the-root-cause-is-recovered)).
[`field_type_mismatch`](../../tests/ui/acceptable/field-types/field_type_mismatch.rs) and its shorter-chain
sibling [`field_type_mismatch_1`](../../tests/ui/acceptable/field-types/field_type_mismatch_1.rs) pin it.

One boundary still keeps the transform honest. A diagnostic that is neither a check entry nor a method
`E0599` — a manual supertrait bound like `use_type_foreign_unsatisfied`/`use_type_nested_unsatisfied` —
has no context to recover and falls back; so does a branch that reaches an impl whose trait clauses all
hold *and* carries no unmet `HasField` projection, a projection the walk still cannot explain.
Everything the resolver declines flows through the fallback `rewrite` and `postprocess` transforms
instead. `mixed_rust_error` shows both sides at once: its CGP check failure becomes a tree while its
ordinary `E0308` type mismatch passes through the fallback.

## Why it runs in the emitter

The natural home for whole-crate typed analysis would be an `after_analysis` callback, where the
compiler hands the driver a `TyCtxt` directly. That door is closed for the crates that matter here.
The `analysis` query raises a fatal error the moment type-checking reports any non-lint error
(`rustc_interface`'s `analysis` calls `has_errors_excluding_lint_errors().raise_fatal()`), and that
unwind happens *before* the driver's `after_analysis` hook is reached — so for a crate with a CGP
check failure, which by definition has an error, `after_analysis` never runs. The same fact is why
Clippy's late passes only see code that type-checks.

The one place that executes *while the error exists but before the fatal* is the diagnostic emitter,
which the compiler calls as it emits each error during trait solving. At that moment a `TyCtxt` is in
thread-local scope — the driver already relies on this for the trait-renaming rewrite — so the
resolver reaches the compiler through `rustc_middle::ty::tls` from inside `emit_diagnostic`. The cost
is a subtlety the resolver has to be sound against: it re-enters the trait solver *from within a
diagnostic that is itself being emitted mid-solve*. Building a fresh `InferCtxt` and `ObligationCtxt`
and solving a concrete obligation there turns out to work cleanly, and that re-entrancy is the
load-bearing assumption of the whole design — it was proven on `base_area_1` before any of the
machinery was built.

## How the root cause is recovered

The recovery runs in the driver's [`resolve`](../../crates/cargo-cgp-driver/src/resolve) module — a
directory of stage files (`anchor`, `walk`, `classify`, `label`, `cgp_item`) behind a re-exporting
`mod.rs` — driven by the emitter's `try_resolve`, and it is a chain of typed lookups with no string
parsing until the very last step decodes a field name. What it produces is the rustc-free
[`Resolved`](../../crates/cargo-cgp-error-processing/src/diagnosis/resolved.rs) model: the module
imports `Cause`, `Leaf`, `FieldIssue`, and `Resolved` from the error-processing crate and fills them
with owned `String`s, rather than defining those types itself, so the wording that consumes them needs
no compiler. Each stage is anchored by `DefId` to the CGP crate that defines the trait or type it
matches, so a same-named item from an unrelated crate can never drive a replacement — the same
discipline [`component_map`](../../crates/cargo-cgp-driver/src/component_map.rs) uses for
`IsProviderFor`.

**Find the entry.** A `check_components!` entry expands to a concrete impl of a generated check
trait — `impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}` — whose check trait
carries `CanUseComponent<Marker, Params>` as a supertrait. The macro re-spans the context type in
that impl onto the entry the user wrote, so the impl's `Self`-type span equals the failing
diagnostic's primary span. The resolver walks the crate's check traits (those with a
`cgp_component::CanUseComponent` supertrait) and their impls, and picks the impl whose `Self` span
matches the caret. This span match is what ties *this* diagnostic to *this* entry without reading
either one's text.

**Recover the concrete obligation.** The check trait's supertrait is generic —
`Self: CanUseComponent<__Component__, __Params__>`. Instantiating it with the matched impl's trait
reference (`instantiate_supertrait`) substitutes the concrete types back in, yielding the real
obligation the compiler failed to prove: `Rectangle: CanUseComponent<AreaCalculatorComponent, ()>`.

**Or recover it at an impl site.** A wiring failure often surfaces inside a *hand-written* `impl
Trait for Context` block rather than at a check entry or a plain call — the transfer example's
per-endpoint `impl CanHandleApiSend<Api> for MockApp`, a wrapper trait that carries the CGP consumer
trait `CanHandleApi<Api>` as a supertrait and is implemented directly on the context to add a `Send`
bound the component cannot express. The caret then lands on the impl (its header, a method signature,
or a forwarding call in the body), never on the context's own type definition, so the use-site anchor
below cannot recover the context from a struct-definition span. `resolve_impl_site` handles it: it
finds the enclosing trait impl whose *full* HIR span (not `def_span`, which for an impl covers only
the header) contains a diagnostic span, takes its `Self` type as the context, and instantiates the
impl's trait supertraits for that `Self`. A supertrait that is a CGP **consumer trait** and does not
hold is the wiring failure; the resolver reconstructs the `Ctx: CanUseComponent<Marker, Params>`
obligation it stands for — recovering the marker through the consumer's blanket impl and the
provider trait's `IsProviderFor` supertrait (the per-consumer form of the inversion
[`component_map`](error-processing.md) performs), and grouping the consumer trait's extra arguments
into `Params` exactly as CGP does (none as `()`, one bare, several as a tuple), **with the concrete
component parameter preserved** (`CanCalculateArea<Rectangle>`, not the `()` the use-site re-check
would substitute). The walk of that reconstructed obligation is then **headed by the impl's own
trait** — the wrapper the programmer wrote (`CanHandleApiSend<Api>`) — so the tree reads
`CanHandleApiSend → CanHandleApi → …` and points at their code, rather than dropping the wrapper and
starting at the CGP supertrait. The wrapper heads the diagnostic too, and its header wording depends
on the wrapper's own **fingerprint**: a wrapper that is itself a CGP consumer trait (a blanket impl
routing to a provider trait) reads `[CGP-E001] the consumer trait …`, while a plain wrapper such as
`CanHandleApiSend` — with only a concrete impl — reads `[CGP-E009] the trait …`. Because the wrapper
is a distinct trait from the CGP supertrait it reduces to, its error is reported on its own rather
than de-duplicating into the `check_components!` entry for that supertrait. This anchor is tried
*before* the use-site one, so its precise obligation wins over the parameterless re-check. It fires
only for an impl on a *local* struct or enum (an `impl … for Router<Arc<App>>` on a foreign type, or
an impl on a provider struct, carries no consumer supertrait on a context and is skipped).

**Or recover it at a use site.** When no check impl matches the caret — a consumer-method `E0599` —
`resolve_use_site` recovers the obligation instead from the diagnostic's spans. It scans every local
struct/enum whose definition span contains one of the diagnostic's spans (the receiver's type is one
such), and for each candidate reads the `DelegateComponent<Marker>` impls that context carries — the
components it wires — building a fresh `Ctx: CanUseComponent<Marker, ()>` per marker and keeping the
ones that do not hold. A diagnostic span can also land on a *provider* struct, so a candidate that
wires no failing component is discarded, which selects the real context. From there the walk is
identical.

**Walk the dependency graph downward.** From that obligation the resolver walks *down* the wiring's
trait obligations, because the tree shows the transitive path to each root cause, not only the root.
For a failing obligation it finds the impl that would satisfy it and takes that impl's `where`-clause
obligations as its direct dependencies, then recurses into just the ones that do **not** already hold —
a satisfied dependency (an already-present field, a wired provider that checks out) is pruned.

A branch ends at a **terminal leaf**, and which obligations count as terminal is what keeps the tree
honest. The descent follows only the CGP wiring vocabulary — `CanUseComponent`, `IsProviderFor`,
`DelegateComponent`, any provider trait, and any obligation whose `Self` is the context (its getter and
capability traits) — and treats everything else as a leaf. An unmet `HasField` is the field leaf. An
unmet `DelegateComponent<Marker>` **on the context** is the missing-wiring leaf: the context does not
delegate that component to any provider, so the wiring is absent (that a `DelegateComponent` on any
*other* type — a provider struct that implements its provider trait directly rather than delegating —
is instead a dead-end is covered below). An unmet **namespace-lookup bound** — recognized not by name
but by the trait's fingerprint, a single `Delegate` associated type (`DefaultNamespace`, the
`DefaultImpls*` traits, and every user `cgp_namespace!` trait all share it, so a same-named user
namespace is caught without a `DefId` anchor) — is the missing-redirect-wiring leaf: a `RedirectLookup`
forwarded the lookup to a path the context's table does not terminate. An
ordinary bound on a *foreign* type (`f64: Eq`) is a terminal leaf too, and crucially the descent does
not blindly walk into whatever unrelated `std` blanket impl happens to match its `Self` (an
`impl<F: FnPtr> Eq for F` would otherwise fabricate a misleading `f64: FnPtr` step). The one foreign
bound it *does* descend is a getter or capability trait applied to a non-context type whose satisfying
impl depends on the **context** — a request struct's `HasBasicAuthHeader<Ctx>`, whose
`#[cgp_auto_getter]` blanket impl requires `Ctx: HasPasswordType`. There the walk looks into that
blanket impl and follows only its context-side dependencies, so the real cause on the context
surfaces (and de-duplicates with the same cause reached down another branch) instead of the opaque
`Request: HasBasicAuthHeader<Ctx>` bound being reported as a second, misleading root cause. Following
only the context-side dependencies is what preserves the `f64: Eq` guarantee — a foreign `f64: FnPtr`
step is not context-side, so it is never followed — and it skips the getter's own `Ctx::Assoc`-typed
`HasField` clause on the request, which is present but a projection mismatch a plain descent would
misreport as a missing field. Two further rules
handle the remaining cases. An obligation whose satisfying impl's trait-clause `where`-obligations
**all hold**, yet is itself unmet, is failing for a projection/associated-type mismatch the
trait-clause walk cannot see. The resolver looks among that impl's own predicates for the one form it
can pin down — an unmet `HasField` projection (`<Ctx as HasField<Symbol!("f")>>::Value == T`), a field
present with the wrong type — and, finding one, completes the branch with that field's `HasField` trait
ref, tagging the path with the expected type so the leaf renders as a `FieldTypeMismatch` (the
`E0271` field case); a branch with no such projection yields nothing and declines to the fallback. And
a branch that bottoms out on pure wiring plumbing — an unmet `CanUseComponent` or `IsProviderFor`, or a
`DelegateComponent` on a type *other* than the context — is a routing dead-end and is dropped, since
the real cause is found down another branch. A `DelegateComponent` on the context is the exception the
rule turns on: it is never plumbing but the missing-wiring leaf itself, because a delegation that
*holds* is pruned before it can be a leaf, so the only way one bottoms out unmet on the context is that
the context genuinely does not wire it.

Two mechanical properties matter. First, following *every* unmet dependency, not just the first, is
what surfaces independent causes as **separate** paths — the next-generation solver short-circuits a
conjunction at its first unmet bound, so a provider that needs two absent fields would otherwise hide
one. Second, finding the satisfying impl uses the `fresh_args_for_item`-plus-unification dance rather
than `SelectionContext`, which asserts against the next-generation solver the driver runs under; each
matched impl's predicates are instantiated, normalized, and region-erased before they cross into the
fresh inference context that checks whether they hold, since a stray inference or region variable from
one context panics another.

**Decode the field name.** The `HasField` leaf carries the field name as a type-level `Symbol!`, a
nested `Chars<'h', Chars<'e', …>>` spine. The resolver decodes it structurally — walking the spine and
reading each `char` const argument until `Nil` — rather than un-sugaring the printed type. Reading the
name from the type rather than the text is why the replacement never needs the `--verbose` un-eliding
the [text path depends on](driver.md#un-eliding-the-diagnostic): the characters are in the `Symbol`
arguments whether or not the diagnostic would have printed them.

**Classify why the field is unmet.** A "missing" `HasField` bound does not always mean an absent
field. The resolver inspects the struct the bound lands on — the leaf's self type — and its `Deref`
chain, to tell three cases apart. If the struct carries no field of that name and neither does any
`Deref` target, the field is genuinely **missing**. If the struct itself carries the field, the bound
is unmet only because the struct is missing (or has an incomplete) `#[derive(HasField)]` — **present**.
If the field lives on a struct reached through `Deref` (CGP's `HasField` forwards across `Deref` via a
blanket impl, so the bound *would* hold if that target derived the field), the fault is on the target —
**present-via-`Deref`** — and the resolver records that target's name so the fix can point at it. The
inspection reads named struct fields directly and follows `Deref` by reading each `impl Deref`'s
`Target` associated type, so it needs no inference context; it is bounded against a cyclic `Deref`.
This classification is what lets the emitter word a present field's diagnostic as an unimplemented
accessor with a concrete fix rather than as a bare "missing field".

A field present with a mismatched *type* is a fourth case, reached differently. Its `HasField` trait
impl holds, so the trait-clause walk never treats it as unmet; instead the branch's impl matches with
every trait clause satisfied, and the resolver then finds the unmet `HasField` projection among that
impl's predicates (see the projection rule under
[Walk the dependency graph downward](#how-the-root-cause-is-recovered) above). From that projection it
reads the **expected** type — the projection's right-hand side (`f64`) — and it queries the struct for
the field's **actual** type: it reads the named field's declared type straight off the struct's
`DefId`, with the struct's own generic arguments substituted, so a same-named struct in another module
is never the one queried and a generic context's field type instantiates correctly. The leaf records
the field name, the owning struct, and both types, and the emitter words it as the `[CGP-E003]`
`expected a \`height\` field of type \`f64\` on \`Rectangle\`, but found \`i32\`` header.

A non-field leaf carries no struct to inspect, so it is simply restated as `self: Trait`
(`f64: std::cmp::Eq`) for its note lead and for de-duplicating a leaf reached by several paths.

**Render each root cause as its own sub-error.** Each root-cause path is a list of typed predicates,
and rendering it is where every CGP wiring trait is replaced by the concept it stands for, so the reader
never meets a raw `IsProviderFor` or `Symbol`. `CanUseComponent<Marker>` becomes the consumer-trait impl
(`consumer trait impl \`CanCalculateArea\` for context \`Rectangle\``), an `IsProviderFor` becomes the
provider-trait impl naming its provider trait, context, and provider struct (`provider trait impl
\`AreaCalculator\` with context \`Rectangle\` for provider \`RectangleArea\``), and `HasField` becomes
the field-trait impl (`field trait impl \`HasField\` with field \`height\` for \`Rectangle\``); a user's
own capability trait — or a terminal ordinary bound — renders as `trait impl \`Trait\` for \`Self\``
(`trait impl \`HasRectangleFields\` for \`Rectangle\``, `trait impl \`Eq\` for \`f64\``). A **generic**
component's parameters are reattached to its consumer and provider labels from the `Params` slot of
`CanUseComponent`/`IsProviderFor` — a single one bare, several unwrapped from their tuple — so the
trait reads as written (`CanCalculateArea<u32, u64, bool>`, `AreaCalculator<u32, u64, bool>`). The
marker-to-trait-name lookups go through the same [`ComponentNameMap`](error-processing.md) the
trait-renaming rewrite is built on, but keyed by each marker's **full path** (`def_path_str`) rather
than its bare name, so two components that share a name in different modules resolve to their own trait
names instead of one clobbering the other. Pure plumbing that carries no information — the
`DelegateComponent` table, the routing `IsProviderFor` for the *context itself* (as opposed to the real
provider), and a bare provider-trait obligation that an `IsProviderFor` node already stands for — is
dropped, so the chain stays legible without losing a real dependency step. Each rendered entry is
stamped with its own [`CGP-E1xx` code](../error-code.md) — one per template, so `consumer trait impl`
(`CGP-E101`), `provider trait impl` (`CGP-E102`), `redirect lookup` (`CGP-E104`), and the general
`trait impl \`Trait\` for \`Type\`` (`CGP-E105`) each carry a distinct tag; a terminal leaf takes a
leaf code (`CGP-E106`–`CGP-E109`) via `dependency_tree_leaf`, except a pass-through ordinary bound
(`the trait bound \`f64: Eq\` is not satisfied`), which stays uncoded. Each cleaned path folds into
a [`DependencyTree`](error-processing.md) spine, rendered as `cargo tree`-style indented text by the
[`termtree`](https://crates.io/crates/termtree) crate (a tiny, dependency-free renderer) hosted in the
rustc-free `cargo-cgp-error-processing` crate so the rendering is unit-tested on any toolchain.

**Emit.** The wording is decided rustc-free and only *applied* by the emitter. The emitter maps the
diagnostic's own rustc code to a rustc-free
[`DiagKind`](../../crates/cargo-cgp-error-processing/src/diagnosis/plan.rs) (`E0271` → field mismatch,
`E0599` → use-site method, everything else → a plain check) and hands that, the main-message text, the
`Resolved`, and the name map to `plan_resolved`, which returns a `DiagnosisPlan`: the rewritten header
(or `None`), the derive `help`s, and one note per cause. `plan_resolved`'s `categorized_header` is what
recognizes the class — the `CGP-E001` consumer form worded from the resolution's context and consumer
trait(s) (pluralized when a use-site failure spans several components), the `CGP-E002` provider form
from the text rewrite, or the `CGP-E003` field-type-mismatch form worded from the mismatch leaf when the
kind is a field mismatch the resolver traced to a `HasField` projection. A field-mismatch-coded
(`E0271`) failure the resolver traced to a *non*-mismatch cause instead — a manual `Send`-recovery
wrapper's opaque-future error, whose `type mismatch resolving …` message is unreadable — takes the
`CGP-E001` consumer form, since it is really the consumer trait failing to be implemented. It keeps
rustc's own header
(yields `None`) only when the main message restates a **genuine recovered leaf** — an ordinary bound
such as `f64: Eq` the solver descended to, which is itself the root cause. When rustc instead descended
to a *mid-chain symptom* — an ordinary bound that is not one of the recovered leaves (a getter bound on
a request whose real cause is a missing wiring one level down) — the consumer `CGP-E001` header is
truer than that symptom, so it wins over keeping rustc's; a main message that is neither a trait bound
nor a resolved class (an unrelated `E0308`) still yields `None`. `transform_resolved` then only mutates rustc's own `DiagInner`: when the
plan carries a header it replaces the main message and collapses the span to the primary caret, since
the original labels restate the replaced message; an unrecognized (`None`) header leaves the header,
labels, and caret alone. Either way it replaces the children with the plan's `help`s (one per distinct
type that must derive — `make sure that \`#[derive(HasField)]\` is used for \`Rectangle\``, or the
`Deref` target; a field-type mismatch contributes none, its field being present and derived) and the
plan's notes — **one per root cause**, each opening with its `root cause:` lead over `this is required
through the dependency chain:` and the tree indented beneath (the lead omitted, and the note the chain
alone, when the kept header states the same bound or when the leaf is a field-type mismatch whose
`CGP-E003` header already states it in full). rustc's structured suggestions are discarded with its
notes (the misleading "use associated function syntax" a method `E0599` carries). The diagnostic's
code is never touched, so a check failure stays `E0277` and a use-site failure stays `E0599`. A
provider with two absent dependencies yields two notes, each a self-contained path to its leaf, and
the JSON emitter regenerates every rendered and structured field from the `DiagInner` for free, with
rustc's note-continuation indentation aligning each tree's box-drawing under its `= note:`.

## Boundaries and open ends

The resolver is deliberately bounded, and a few of its edges are worth recording. It recovers a
starting obligation three ways — a `check_components!` entry by **exact span match** (the check macro
re-spans the context type onto the entry), a hand-written `impl Trait for Context` block by finding
the enclosing impl whose `Self` is a local context and reconstructing the failing consumer
supertrait's obligation, and a use-site `E0599` by finding the context ADT from the diagnostic's
spans — so a wiring failure that is *none* of the three still declines. Two shapes still find nothing
to anchor on: a manual supertrait bound written as a free `where` clause or a trait definition
(`use_type_foreign_unsatisfied`/`use_type_nested_unsatisfied`, where no `impl` on a context encloses
the caret), and a failure whose only caret sits on a *provider* struct's own impl (its `Self` is the
provider, which carries no consumer supertrait on a context) or inside the generic component's trait
definition. The impl-site path recovers the concrete component parameter from the supertrait, but the
use-site path still builds each `CanUseComponent<Marker, ()>` with an **empty `Params` slot**, so a
generic component whose real parameters matter is not re-checked *there* (the check and impl-site
paths recover those). It renders only leaves it can trust: a `HasField` field (missing, underived, or —
via its projection — present with the wrong type), a component the context does not wire (an unmet
`DelegateComponent` on the context), a namespace redirect the context does not terminate (an unmet
namespace-lookup bound whose `Self` is the redirect path), an ordinary bound on a foreign type, or a
terminal capability bound — but it still *declines* an associated-type projection mismatch that is **not** a
`HasField` one (the projection form it cannot word), and drops pure wiring-plumbing dead-ends (an
unmet `CanUseComponent`/`IsProviderFor`, or a `DelegateComponent` on a *provider* rather than the
context), so a diagnostic whose only recoverable leaf is one of those falls back. And it uses an **empty parameter
environment** throughout, which suits the concrete check impls the fixtures exercise but will need the
impl's own environment to extend cleanly to checks that carry generic parameters. (Parallel branches,
deep nesting, and non-field leaves, by contrast, are handled: independent unmet dependencies become
separate sub-errors, the descent follows the wiring to any depth up to a recursion bound, and an
ordinary or capability bound renders as its own tree.)

How a transformed diagnostic is *marked* as CGP is settled by the [error-code scheme](../error-code.md):
a rewritten, classified main message carries its `[CGP-Exxx]` code inline, and everything else — a kept
header over rewritten sub-messages included — is deliberately unmarked, keeping rustc's own
`error[E0277]:` form. There is no separate header brand; the inline code is the only marking, which
says what it needs to without altering the header's shape.

## Source

- [`crates/cargo-cgp-driver/src/resolve/`](../../crates/cargo-cgp-driver/src/resolve) — the typed
  resolution, split by stage behind a re-exporting `mod.rs` and building the rustc-free `Resolved`
  model: `anchor.rs` (`resolve_check_failure` finding the check impl by span, `resolve_impl_site`
  recovering the context and the exact failing obligation from the enclosing hand-written `impl Trait
  for Context` block's CGP consumer supertrait, then heading the tree and the header with the impl's
  own wrapper trait — `[CGP-E001]` or `[CGP-E009]` by the wrapper's blanket-impl fingerprint — and
  `resolve_use_site` recovering the context ADT from
  the diagnostic's spans and its wired components from `DelegateComponent` impls), `walk.rs` (walking the cause chain down to each terminal leaf — the
  descendable-vocabulary rule, the plumbing-leaf drop, the foreign-getter descent that follows a
  non-context getter bound's blanket impl into just its context-side dependencies, `is_reportable_leaf`
  keeping an unmet `DelegateComponent` only when it lands on the context, `has_field_projection_mismatch`
  finding an unmet `HasField` projection where the trait clauses all hold, and — after building the inner
  labels from the chain *above* the leaf — appending the coded `dependency_tree_leaf` as the tree's
  terminal, so the chain ends on the root cause), `classify.rs` (classifying a leaf as a
  field by inspecting the struct and its `Deref` chain, a field-type mismatch with `field_type` reading
  the actual field type off the struct by `DefId`, a missing wiring naming the unwired component
  marker, a missing *redirect* wiring naming the unterminated namespace path, or a bound), `label.rs`
  (folding the inner chain into a `DependencyTree` with each wiring trait replaced by its human form
  and stamped with its per-template `CGP-E1xx` code — a `RedirectLookup` provider as
  `[CGP-E104] redirect lookup to \`@…\` in \`Ctx\``, and the `DelegateComponent`
  table lookup and namespace lookup dropped as plumbing since the caller re-states the leaf — generic
  parameters reattached),
  and `cgp_item.rs` (the `DefId`-anchored CGP-trait recognition, the `Symbol!` field-name decode,
  and `is_namespace_lookup_trait` recognizing a namespace trait by its single-`Delegate`-associated-type
  fingerprint rather than by name).
  A sibling `conflict.rs` handles the duplicate-key coherence conflict (`E0119`) rather than a check
  failure — a separate transform documented in
  [The driver](driver.md#reshaping-a-duplicate-key-conflict), not part of this resolution.
- [`crates/cargo-cgp-driver/src/emitter/`](../../crates/cargo-cgp-driver/src/emitter) — the
  `try_resolve` seam (gated by a cheap `mentions_wiring` scan, or an `E0599`/`E0271`/`E0277` code, so a
  raw cascade with no CGP wording is still traced) that tries the check anchor, then the impl-site
  anchor, then the use-site anchor, and the `transform_resolved`
  mutation it feeds: it maps the
  diagnostic's rustc code to a `DiagKind` (`edit::diag_kind`), calls the rustc-free `plan_resolved`
  for the rewritten header and the help/note strings, and applies that plan to the `DiagInner`,
  falling back to the in-place text rewrite when resolution returns `None`. A final cross-diagnostic
  de-duplication (keyed on the recovered cause, the rendered text, or the coded header) then suppresses
  a transformed diagnostic that re-reports a failure already shown.
- [`crates/cargo-cgp-error-processing/src/diagnosis/`](../../crates/cargo-cgp-error-processing/src/diagnosis)
  — the rustc-free model and wording the resolution feeds: `leaf.rs` and `resolved.rs` (the `Leaf`,
  `FieldIssue`, `Cause`, and `Resolved` types the resolver builds), `wording.rs` (the
  `Resolved`→`String` builders — the coded headers, the `root cause:` notes, and the derive `help`s),
  and `plan.rs` (`DiagKind`, `DiagnosisPlan`, and `plan_resolved` with its `categorized_header`), with
  unit tests in [`tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs).
- [`crates/cargo-cgp-error-processing/src/tree.rs`](../../crates/cargo-cgp-error-processing/src/tree.rs)
  — the rustc-free `DependencyTree` type and its `cargo tree`-style renderer (over `termtree`), with
  unit tests in [`tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
- [`crates/cargo-cgp-driver/src/config.rs`](../../crates/cargo-cgp-driver/src/config.rs) — the crate
  and trait-name anchors (`CanUseComponent`, `IsProviderFor`, `HasField`, and the `Symbol` spine's
  crate) the resolution matches against.

## Tests

The resolver is exercised end to end by the UI snapshot suite: the fixtures it reshapes live under
[`tests/ui/acceptable/`](../../tests/ui/acceptable) — the `fields/`, `field-types/`, `providers/`,
`generic/`, `resolution/`, `wiring/`, and `use-site/` subgroups — and carry `.cgp.stderr` snapshots showing the
transformed output, while the fixtures it declines keep their fallback snapshots under
[`tests/ui/usability/use-type/`](../../tests/ui/usability/use-type), so the two together pin both the
transform and the decline boundary. Several fixtures pin the harder cases: `parallel_branches` (two
independent missing fields → two sub-errors), `deep_nesting` (a stack of higher-order providers nested
four deep → one long spine), `dependency_cascade` (a chain of providers each depending on the next),
`mixed_rust_error` (a CGP tree beside an untouched ordinary `E0308`), `missing_has_field_derive` (a
field the struct carries but has not derived → the unimplemented-accessor header plus the derive
`help`), `field_via_deref` (a field on a `Deref` target that does not derive `HasField` → the `help`
pointed at the target), `field_type_mismatch` and `field_type_mismatch_1` (a matching field name with
a mismatched type, read through a getter trait and directly via an `#[implicit]` argument → the
`CGP-E003` field-type-mismatch header over the dependency chain), `field_type_mismatch_modules` (two
different `Rectangle` contexts in separate modules, each with a differently-typed `height` → each
error reports its own struct's actual type, proving the field query is `DefId`-anchored),
`same_name_components` (two components forced to share
a marker name in different modules, with distinct consumer *and* provider trait names, both checked →
full-path resolution names each one's own traits with no cross-over), `generic_area_multi` (a
three-parameter component → the parameters reattached to the consumer and provider labels), and
`ordinary_bound_unsatisfied` (a non-field `f64: Eq` bound, whose rustc header is kept over a lead-less
chain note), `foreign_getter_missing_wiring` (a `#[cgp_auto_getter]` getter on a *request* type,
depending on the context's abstract type, wrapped in a higher-order provider — the transfer example's
`UseBasicAuth` shape — so the failure surfaces as the opaque `Request: HasCredential<App>` bound; the
walk descends that getter's blanket impl into its context-side dependency and the misleading second
root cause collapses into the one missing-wiring cause, under a promoted `CGP-E001` header), and
`unregistered_prefix_path`/`qualified_prefix_path` (an unwired namespace redirect
behind an `IsProviderFor` header, rewritten to the `CGP-E002` form over a `root cause:` note whose
chain is the redirect hop(s) down to a terminal naming the context with no delegate entry for the
bare `@…` path — the latter defined across sub-modules, pinning that a module-qualified path still
folds to a clean `@…`; and `multi_redirect_missing` pins a chain of several hops). The missing-wiring
leaf is pinned by the [`acceptable/wiring/missing-wiring/`](../../tests/ui/acceptable/wiring/missing-wiring)
fixtures: `basic_missing_wiring` (a provider's `#[uses]` dependency on an unwired component → a
`missing wiring` note over the transitive chain), `direct_missing_wiring` (a `check_components!` entry
for a component the context wires nowhere → a single-node chain), and `parallel_missing_wiring` (a
provider needing two unwired components → two `missing wiring` notes, one per component). The use-site path
is pinned by the [`acceptable/use-site/`](../../tests/ui/acceptable/use-site)
fixtures: `missing_dependency` and `unsatisfied_dependency` (a consumer-method `E0599` → the
`CGP-E001` header, the misleading method-syntax advice dropped, and a `missing field` root-cause note),
`missing_wiring` (a use-site `E0599` whose provider needs an unwired component → the `CGP-E001` header
over a `missing wiring` note),
and `ordinary_bound_unsatisfied` (a use-site `f64: Eq` → the `CGP-E001` header, code kept `E0599`,
over the `f64: Eq` root-cause note). The impl-site path is pinned by `manual_supertrait_impl` in the
same directory (a wrapper trait carrying a *generic* CGP consumer supertrait, implemented directly on
the context — the transfer example's `CanHandleApiSend` shape — failing both at the impl header
`E0277` and its forwarding-call `E0599`, each resolved to the same tree with the concrete component
parameter preserved. Both recover the same cause and collapse to a single block headed
`[CGP-E009] the trait \`CanCalculateAreaChecked<Rectangle>\`` — the wrapper being a plain trait, not a
CGP consumer — over a tree that leads with the wrapper and descends through its `CanCalculateArea`
supertrait to the missing field). The `CGP-E009` wrapper header and the raw `E0271` trace are pinned
by [`traced_send_wrapper`](../../tests/ui/acceptable/duplication/traced_send_wrapper.rs) (an async
`Send`-recovery wrapper whose opaque-future `E0271` names no CGP construct, traced to the wrapper-headed
tree). The leaf wording — a missing field, a
present-but-underived one, a `Deref`-target one, and a missing wiring — is unit-tested over hand-built `Resolved` values
in [`cargo-cgp-error-processing/tests/diagnosis.rs`](../../crates/cargo-cgp-error-processing/tests/diagnosis.rs),
and the renderer itself in
[`cargo-cgp-error-processing/tests/tree.rs`](../../crates/cargo-cgp-error-processing/tests/tree.rs).
[Testing](testing.md) describes the suite and its passes.

## Further reading

- [The driver](driver.md) — the emitter seam this resolver extends, and the trait-renaming rewrite it
  falls back to.
- [The error pipeline](error-pipeline.md) — where this driver-side transformation sits among the
  pipeline's four stages.
- [CGP check-trait failure](../../../cgp/docs/errors/checks/check-trait-failure.md) — the upstream
  error class the resolver reshapes.
