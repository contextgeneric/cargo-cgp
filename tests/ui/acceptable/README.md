# Acceptable fixtures

These are the fixtures whose output already meets `cargo-cgp`'s presentation bar: the tool leads with
a coded `[CGP-Exxx]` headline, states the root cause as one plain sentence, decodes any type-level
construct, renders the dependency path as a compact `cargo tree`, and suppresses the
`IsProviderFor`/`CanUseComponent`/`__Check…` scaffolding. A fixture lands here — rather than under
[`../usability/`](../usability) — when there is no meaningful remaining presentation problem, so this
category is the passing baseline for *reformatted errors*, the counterpart to [`../ok/`](../ok) (which
holds only clean compiles). Its snapshots are the standing proof the tool keeps producing good output.

The whole check-trait-failure family lives here, carried across by the driver's
[typed root-cause resolver](../../../docs/implementation/typed-root-cause-resolution.md). The fixtures
are split into concept sub-directories so no directory grows crowded:

- [`fields/`](fields) — a missing or underived context field, reached directly or through a `Deref`
  target, and the parallel/empty-struct cases where several fields are genuinely absent. Several
  underived fields on one struct coalesce into a single add-the-derive root cause (`base_area_2`),
  with the group's boundaries pinned by `underived_and_missing_field` (a lone underived field beside
  a genuinely missing one stays two causes).
- [`field-types/`](field-types) — a field present but of the wrong type (the `[CGP-E003]` mismatch),
  read through a getter, an implicit argument, and across modules.
- [`providers/`](providers) — higher-order, nested, and transitive provider chains (the scaled,
  higher-order, deep-nesting, and density cases), where the tree pinpoints the failing layer —
  including a `#[check_providers(...)]` per-layer assertion resolved to the failing layer's root
  cause (`check_providers_layer`).
- [`generic/`](generic) — components generic over a type parameter, with the parameters reattached to
  the consumer and provider names; the params-slot ungrouping is pinned by a component carrying a
  *lifetime* parameter (`lifetime_component`, whose `Life<'a>` lift is restored to a region rather
  than leaked as a type) and one whose single parameter is itself a *tuple* type
  (`tuple_param_component`, kept whole rather than spread into two parameters).
- [`resolution/`](resolution) — resolver edge pins: an ordinary non-CGP bound kept uncoded, same-named
  components in different modules, a CGP error beside an untouched ordinary one, an unregistered
  namespace path, a getter on a foreign request type resolved to the context's missing wiring
  rather than the opaque getter bound (`foreign_getter_missing_wiring`), and a record field-list
  descent crossing *nested* higher-ranked projection hops (`nested_higher_ranked_descent`).
- [`duplication/`](duplication) — one wiring mistake re-reported at many sites collapsed to one block
  by the emitter's cross-diagnostic de-duplication (`cross_site_dedup`: a check entry, a wrapper impl,
  and its forwarding call for one missing field → a single error, the un-deduplicated cascade kept in
  the `.rust.stderr` baseline).
- [`use-site/`](use-site) — a broken dependency reached by a direct consumer-method call (`E0599`),
  recovered from the use site with the misleading method-syntax advice dropped — including
  `namespace_join_use_site`, whose namespace-joined context is anchored on the consumer trait the
  diagnostic names and walked through the namespace to its root cause, and the two shapes resolved by
  [the call-site anchor](../../../docs/implementation/typed-resolution-call-site.md):
  `cascade_after_use_site` (an unconditionally-dispatched `E0277` re-read from the failing call into
  one root-cause block, its await-site re-report de-duplicated and its `?`-operator cascade
  suppressed) and `generic_consumer_use_site` (the dispatch parameter recovered from a plain written
  value argument). The declining counterpart, `generic_consumer_unwritten_arg` (a parameter carried
  by an argument the call never types), pins the fallback's cleanup: the method-probe advice is
  stripped so the unmet wiring bound leads.
- [`verbosity/`](verbosity) — the presentation of a *resolved* failure on a program-sized `Code`
  type: `deep_dispatch_chain` pins the repeated-generics elision, where a hop that exactly repeats
  its predecessor's trait parameters renders as `Handler<…>`.
- [`use-type/`](use-type) — an unsatisfiable `#[use_type]` abstract-type import, recovered into a
  `[CGP-E001]` missing-wiring tree (`use_type_foreign_unsatisfied`, `use_type_nested_unsatisfied`)
  by the same consumer-trait anchor, rather than leaking generated `__…__` placeholder names.
- [`wiring/`](wiring) — the structural wiring failures, split by shape. `duplicate-keys/` holds the
  duplicate delegate-key `E0119` reshaped into a coded `[CGP-E004]`–`[CGP-E008]` headline (the
  redundant `IsProviderFor` half dropped, the colliding key named), a duplicate component name, and
  a duplicate provider name (its redundant `IsProviderFor` conflict suppressed, leaving the `E0428`
  and the provider-trait conflict). `namespace-paths/` holds the `@`-path collisions — a duplicated
  or overriding path, a path that is a prefix of another, two joined namespaces, a prefixed
  `for`-loop key, a bare unprefixed key over a namespace, a duplicate default impl, an
  inherited-override conflict, and the duplicate `cgp_namespace!` path reshaped into `[CGP-E008]`
  naming both redirect targets. `missing-wiring/` holds the unwired-component chains, and
  `constrained-key/` a delegation whose constrained key is unsatisfied. `constraints/` holds the
  `UseContext` cycle, its `E0275` rewritten into a `[CGP-E010]` headline over a `help` naming the
  usual cause.
- [`lowering/`](lowering) — a macro-lowering error rustc already states well on its own
  (`use_type_unknown_assoc`, whose typo and fix rustc names).

## Origins and the imported mirror

Most of these fixtures are verbatim copies of the upstream CGP compile-fail suite; the rest are the
hand-curated worked examples and typed-resolver pins the knowledge base references. The full account
of the two origins, the four upstream fixtures deliberately not imported, and the re-sync workflow
lives in the [top-level tests README](../../README.md). Refresh an imported fixture by re-copying it
from upstream over its current location and re-blessing; a new upstream fixture goes into whichever
concept sub-directory matches the quality of the output cargo-cgp produces for it.
