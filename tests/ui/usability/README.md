# Usability fixtures

These are the fixtures for the [usability](../../../docs/issues/usability.md) category: CGP compile
errors that carry their root cause but bury it in volume, duplication, encoding, or misleading
framing, so the work is re-presentation rather than recovery. Every fixture here is one whose cause
cargo-cgp's transformed output (`-Znext-solver=globally` + `--verbose`) does contain — a fixture whose
cause were *absent* would belong under
[hidden-root-cause](../../../docs/issues/hidden-root-cause.md) instead. A fixture whose presentation
improves enough to clear the bar graduates out of here into [`../acceptable/`](../acceptable); the
whole check-trait-failure family has already done so.

The fixtures are grouped by the *kind* of remaining usability problem, one sub-directory per issue
class in [the usability issue document](../../../docs/issues/usability.md):

- [`duplication/`](duplication) — one mistake reported as many errors: a single cause that fans out
  across several top-level error blocks (`density_3`, `dependency_cascade`), or a single missing
  `#[derive(HasField)]` reported field by field (`base_area_2`).
- [`use-type/`](use-type) — a `#[use_type]` abstract-type dependency the resolver does not recognize,
  so it falls through untransformed and leaks generated `__…__` placeholder names (and, in the nested
  case, a misleading `trivial_bounds` suggestion).
- [`lowering/`](lowering) — a macro lowered accepted input into ill-formed Rust, and the error lands
  on the macro attribute without naming the real cause: an unsized generated type (`option_slice`) or
  a cyclic `#[use_type]` routing (`use_type_cyclic_context`).
- [`wiring/`](wiring) — structural coherence conflicts (`E0119`/`E0207`/`E0275`) that pass through
  with only light post-processing, so one mistake fans out into paired blocks exposing the internal
  `IsProviderFor`/`DelegateComponent` traits. It is split further into `duplicate-keys/` (a key or
  name wired twice, an overlapping generic), `namespace-paths/` (a namespace path registered or
  overridden twice — also where the incomplete `Path!` resugaring shows), and `constraints/` (an
  unconstrained per-entry generic and a `UseContext` wiring cycle).

## Origins and the imported mirror

Each sub-directory mixes hand-curated fixtures with verbatim copies of the upstream CGP compile-fail
suite. The full account of the two origins, the four upstream fixtures deliberately not imported, and
the re-sync workflow lives in the [top-level tests README](../../README.md); most of the imported
mirror now sits under [`../acceptable/`](../acceptable), since every reproducible class carries its
cause and most are presented well. When a fixture here is fixed, delete its issue from
[docs/issues/usability.md](../../../docs/issues/usability.md) and move its
`.rs`/`.cgp.stderr`/`.rust.stderr` triple into the matching `../acceptable/` concept sub-directory (no
re-bless is needed — a snapshot is independent of the fixture's directory).
