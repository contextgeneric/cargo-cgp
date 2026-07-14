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
  target, and the parallel/empty-struct cases where several fields are genuinely absent.
- [`field-types/`](field-types) — a field present but of the wrong type (the `[CGP-E003]` mismatch),
  read through a getter, an implicit argument, and across modules.
- [`providers/`](providers) — higher-order, nested, and transitive provider chains (the scaled,
  higher-order, deep-nesting, and density cases), where the tree pinpoints the failing layer.
- [`generic/`](generic) — components generic over a type parameter, with the parameters reattached to
  the consumer and provider names.
- [`resolution/`](resolution) — resolver edge pins: an ordinary non-CGP bound kept uncoded, same-named
  components in different modules, a CGP error beside an untouched ordinary one, and an unregistered
  namespace path.
- [`use-site/`](use-site) — a broken dependency reached by a direct consumer-method call (`E0599`),
  recovered from the use site with the misleading method-syntax advice dropped.
- [`wiring/`](wiring) — coherence conflicts whose output is already clear: a duplicate component
  name, a duplicate default impl on user types, and an inherited-override conflict at the top level,
  plus the duplicate delegate-key `E0119` reshaped into a coded `[CGP-E004]`–`[CGP-E008]` headline (the redundant
  `IsProviderFor` half dropped, the colliding key named). The latter is split into `duplicate-keys/`
  (a key wired twice, an overlapping generic, an `open` redirect collision or duplicate, a bare-key
  namespace forwarding) and `namespace-paths/` (a duplicated or overriding `@`-path, a path that is a
  prefix of another, two joined namespaces, a prefixed `for`-loop key, and a bare unprefixed key over
  a namespace).
- [`lowering/`](lowering) — a macro-lowering error rustc already states well on its own
  (`use_type_unknown_assoc`, whose typo and fix rustc names).

## Origins and the imported mirror

Most of these fixtures are verbatim copies of the upstream CGP compile-fail suite; the rest are the
hand-curated worked examples and typed-resolver pins the knowledge base references. The full account
of the two origins, the four upstream fixtures deliberately not imported, and the re-sync workflow
lives in the [top-level tests README](../../README.md). Refresh an imported fixture by re-copying it
from upstream over its current location and re-blessing; a new upstream fixture goes into whichever
concept sub-directory matches the quality of the output cargo-cgp produces for it.
