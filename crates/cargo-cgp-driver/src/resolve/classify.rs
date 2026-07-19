//! Classifying the terminal leaf a dependency chain bottoms out on.
//!
//! Once the [walk](crate::resolve::walk) reaches a terminal predicate, this module turns it into
//! the rustc-free [`Leaf`] the emitter words — inspecting the actual struct a `HasField` bound
//! lands on (and its `Deref` chain) so a genuinely missing field is told apart from one present
//! but underived, reading a mismatched field's actual type straight off the struct by
//! `DefId`, and naming the unwired component behind an unmet `DelegateComponent` on the
//! context (a missing wiring — a bare-marker key, or an `open`-dispatch redirect *path* key whose
//! whole `PathCons` is named rather than flattened to its item name).

use cargo_cgp_error_processing::{FieldIssue, Leaf};
use rustc_middle::ty::print::PrintTraitRefExt as _;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE,
    DELEGATE_COMPONENT_TRAIT, HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT, PATH_CONS_TYPE,
};
use crate::resolve::cgp_item::{decode_symbol, is_cgp_item, is_namespace_lookup_trait};

/// Bound on how far the `Deref` chain is followed when looking for a field, so a cyclic `Deref`
/// (`A: Deref<Target = B>`, `B: Deref<Target = A>`) cannot make the search loop.
const MAX_DEREF: u32 = 16;

/// Classify the terminal predicate a dependency chain bottoms out on. A `HasField` whose branch
/// carried an unmet projection (`mismatch` is `Some(expected)`) becomes a
/// [`Leaf::FieldTypeMismatch`], its actual field type queried from the struct; a plain `HasField`
/// becomes a [`Leaf::Field`] (inspecting the struct so the emitter can tell missing from
/// underived); an unmet `DelegateComponent<Marker>` — a component the context does not wire —
/// becomes a [`Leaf::MissingWiring`] naming that component marker; an unmet namespace lookup
/// (`Path: DefaultNamespace<Ctx>` or a user `cgp_namespace!` trait) — a `RedirectLookup` whose path
/// the context does not terminate — becomes a [`Leaf::MissingRedirectWiring`] naming the path; any
/// other bound becomes a [`Leaf::Bound`] restating it as `self: Trait`.
pub(crate) fn classify_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    mismatch: Option<Ty<'tcx>>,
) -> Leaf {
    if is_cgp_item(
        tcx,
        leaf_ref.def_id,
        DELEGATE_COMPONENT_TRAIT,
        CGP_COMPONENT_CRATE,
    ) {
        let key = leaf_ref.args.type_at(1);
        let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        let owner = self_ty.to_string();
        // A `DelegateComponent<PathCons<…>>` key is a redirect *path* an `open` statement or a
        // namespace routed the lookup along, not a bare component marker — the context's own table
        // has no entry terminating it. Rendering only its ADT item name would flatten the whole path
        // to a useless `PathCons`, so it becomes a [`Leaf::MissingRedirectWiring`] naming the full
        // path (its `PathCons` spine resugars to `@…` when the note is post-processed), parallel to
        // the namespace-lookup leaf below.
        if is_path_cons(tcx, key) {
            return Leaf::MissingRedirectWiring {
                path: tcx.erase_and_anonymize_regions(key).to_string(),
                context: owner,
            };
        }
        if self_ty != context {
            // A `DelegateComponent<Key>` on a *non-context* type splits two ways (both let through
            // by `is_reportable_leaf`). If the owner is a delegation table — a separate-table
            // dispatch lookup (`is_dispatch_lookup`) or an owner that wires some other key
            // (`owner_has_impl_of` for `DelegateComponent`) — it is an aggregate provider or a
            // `UseDelegate`/`UseInputDelegate` table missing this entry: a [`Leaf::MissingDispatchEntry`]
            // naming the table and the key (named in full, since it may be a dispatched-on type).
            let is_dispatch = parent
                .is_some_and(|p| is_dispatch_lookup(tcx, self_ty, p.self_ty()))
                || owner_has_impl_of(tcx, leaf_ref.def_id, self_ty);
            if is_dispatch {
                return Leaf::MissingDispatchEntry {
                    key: tcx.erase_and_anonymize_regions(key).to_string(),
                    table: owner,
                };
            }
            // Otherwise the owner is not a table at all: a type wired where a provider was expected
            // that does not implement the provider trait. Name the provider trait from the parent
            // obligation `owner: ProviderTrait<Ctx>` whose blanket produced this leaf.
            if let Some(parent) = parent {
                return Leaf::NotAProvider {
                    provider: owner,
                    provider_trait: tcx.item_name(parent.def_id).to_string(),
                };
            }
            // No parent trait to name (a root-level `DelegateComponent`); fall back to the
            // dispatch-entry wording rather than invent a trait name.
            return Leaf::MissingDispatchEntry {
                key: tcx.erase_and_anonymize_regions(key).to_string(),
                table: owner,
            };
        }
        // A bare `DelegateComponent<Marker>` on the context with no satisfying impl: the context does
        // not wire the component at all. The marker's own item name (`BarProviderComponent`) is what
        // the programmer writes to fix it, so it names the leaf.
        return Leaf::MissingWiring {
            component: component_marker_name(tcx, key),
            owner,
        };
    }
    if is_namespace_lookup_trait(tcx, leaf_ref.def_id) {
        // A namespace lookup trait (`DefaultNamespace`, a user `cgp_namespace!` trait, …) unmet at
        // the terminal: a `RedirectLookup` forwarded the lookup to this path inside the context's
        // wiring, but nothing terminates it. The `Self` type is the redirect path (its `PathCons`
        // spine resugars to `Path!(@…)` when the note is post-processed) and the trait's last type
        // argument is the context whose table carries no entry for it.
        let path = tcx
            .erase_and_anonymize_regions(leaf_ref.self_ty())
            .to_string();
        let context = leaf_ref
            .args
            .types()
            .last()
            .map(|ctx| tcx.erase_and_anonymize_regions(ctx).to_string())
            .unwrap_or_else(|| path.clone());
        return Leaf::MissingRedirectWiring { path, context };
    }
    if is_cgp_item(tcx, leaf_ref.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE)
        && let Some(name) = decode_symbol(tcx, leaf_ref.args.type_at(1))
    {
        let owner = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        if let Some(expected) = mismatch {
            return Leaf::FieldTypeMismatch {
                actual: field_type(tcx, owner, &name).unwrap_or_else(|| "_".to_owned()),
                name,
                owner: owner.to_string(),
                expected: expected.to_string(),
            };
        }
        let issue = field_issue(tcx, owner, &name);
        return Leaf::Field {
            name,
            owner: owner.to_string(),
            issue,
        };
    }
    Leaf::Bound {
        summary: format!(
            "{}: {}",
            leaf_ref.self_ty(),
            leaf_ref.print_only_trait_path()
        ),
    }
}

/// Whether a terminal leaf is a real root cause worth reporting, rather than pure wiring plumbing.
/// A `CanUseComponent` or `IsProviderFor` that bottoms out unmet is a routing dead-end (the real
/// cause sits down another branch), so it is dropped instead of shown. An unmet `DelegateComponent`
/// is a real root cause in three shapes:
///
/// - on the **context** itself, the context does not wire the component (a [`Leaf::MissingWiring`]);
/// - as a **dispatch lookup into a separate table** — the recognized-structurally case: the
///   obligation is `Components: DelegateComponent<Key>` where `Components` is a *proper part* of the
///   parent obligation's `Self` (as `Components` is of `UseDelegate<Components>` /
///   `UseInputDelegate<Components>`, or any provider that dispatches through a table it holds as a
///   parameter). Such a `where`-clause is unambiguously a table lookup — its owner is not the
///   provider itself — so an unmet one is a missing entry regardless of whether that table wires any
///   other key (`is_dispatch_lookup`); this is what reaches an *empty* dispatch table, which
///   `is_delegation_table` cannot see;
/// - on a **delegation table reached via the generic blanket** — an aggregate provider whose own
///   table lacks a key, recognized because the owner wires *some* other key
///   (`is_delegation_table`).
///
/// It is dropped only when it is none of these. The case that makes the last gate load-bearing (not
/// just cautious): a **leaf provider** whose concrete impl fixes an input type the walk cannot match.
/// A pipeline stage like `HandleShout` (`impl Handler<Code, String>`) fed an *unknown* input — a
/// call-site placeholder an earlier stage's `::Output` never resolved — does not unify with its
/// concrete impl, so `impl_where_obligations` falls through to the delegation blanket and produces an
/// unmet `HandleShout: DelegateComponent<HandlerComponent>`. There the owner *is* the parent's `Self`
/// (the blanket keys on the provider itself, not a separate table), so `is_dispatch_lookup` is
/// false, and `HandleShout` wires nothing, so `is_delegation_table` is false too — a dead-end,
/// correctly dropped, since `HandleShout` is a valid provider and simply is not a table.
///
/// A `DelegateComponent` that is *none* of the above splits one more way, by whether the owner is
/// genuinely a provider for the parent trait at all. It arises via the generic blanket for the
/// parent provider trait `T` (owner == parent `Self`), reached because no concrete `impl T for
/// owner` unified. Two sub-cases, told apart by whether such a concrete impl *exists*:
///
/// - the owner **has** a concrete impl of `T` that merely did not unify (a leaf provider fed the
///   wrong input, the `HandleShout` dead-end) — dropped, its real cause runs through that impl;
/// - the owner has **no** concrete impl of `T` at all — it is genuinely not a provider, wired where
///   one was expected (`UseBasicAuth<QueryBalanceRequest>`, a request type in a handler slot) —
///   reported as a [`Leaf::NotAProvider`].
///
/// `parent` is the obligation one hop above the leaf (the impl whose `where`-clause produced it), or
/// `None` at the root: its `Self` tells the separate-table lookup from the self-keyed blanket, and
/// its trait is the `T` the not-a-provider check and leaf name against.
pub(crate) fn is_reportable_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
) -> bool {
    let did = leaf_ref.def_id;
    if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE) {
        let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        if self_ty == context
            || parent.is_some_and(|p| is_dispatch_lookup(tcx, self_ty, p.self_ty()))
            || owner_has_impl_of(tcx, did, self_ty)
        {
            return true;
        }
        // Reached via the generic blanket for the parent provider trait: report only when the owner
        // has no concrete impl of that trait (a genuine non-provider), never when it has one that
        // merely failed to unify (a valid provider reached by an input mismatch — a dead-end).
        return parent.is_some_and(|p| !owner_has_impl_of(tcx, p.def_id, self_ty));
    }
    !is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        && !is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
}

/// Whether an unmet `DelegateComponent` on `owner` is a **dispatch lookup into a separate table**:
/// `owner` is a *proper subterm* of `parent_self`, the `Self` of the provider impl whose
/// `where`-clause introduced the lookup. This is the shape every dispatcher provider shares —
/// `UseDelegate<Components>` / `UseInputDelegate<Components>` and any custom dispatcher hold their
/// table as a parameter and look a key up in it (`Components: DelegateComponent<Key>`), so the lookup's
/// owner is *inside* the provider's `Self`, never equal to it. The generic delegation blanket, by
/// contrast, keys on the provider *itself* (`P: DelegateComponent<Marker>` with `Self = P`), where
/// owner equals `parent_self` — so this returns `false` for it, leaving the blanket cases to the
/// context and `is_delegation_table` checks. Recognizing the lookup structurally means an entry-less
/// table is still reported, where the owner-property heuristic (which needs a wired key to find) would
/// miss it.
fn is_dispatch_lookup<'tcx>(tcx: TyCtxt<'tcx>, owner: Ty<'tcx>, parent_self: Ty<'tcx>) -> bool {
    let parent_self = tcx.erase_and_anonymize_regions(parent_self);
    parent_self != owner
        && parent_self
            .walk()
            .filter_map(|arg| arg.as_type())
            .any(|sub| tcx.erase_and_anonymize_regions(sub) == owner)
}

/// Whether `owner`'s ADT appears as the concrete `Self` of any impl of `trait_did`. This backs the
/// two "is this owner meant to X" checks the `DelegateComponent` classification needs, each keyed on
/// a different trait:
///
/// - with the **`DelegateComponent`** trait: whether `owner` is a *delegation table* — an aggregate
///   provider or a `UseDelegate`/`UseInputDelegate` inner table wiring at least one key. Any one
///   entry is enough; the owner it excludes is one with zero delegation impls.
/// - with the **parent provider trait**: whether `owner` has a *concrete impl of that provider
///   trait*, which tells a leaf provider reached via the blanket by an input mismatch (has one — a
///   dead-end) from a genuine non-provider (has none — a [`Leaf::NotAProvider`]).
///
/// A blanket impl (whose `Self` is a bare type parameter, like the CGP delegation blanket) is not a
/// concrete `Self`, so it never counts — only an `impl … for SomeAdt` does.
fn owner_has_impl_of<'tcx>(tcx: TyCtxt<'tcx>, trait_did: DefId, owner: Ty<'tcx>) -> bool {
    let ty::Adt(owner_def, _) = owner.kind() else {
        return false;
    };
    tcx.all_impls(trait_did).any(|impl_did| {
        matches!(
            tcx.impl_trait_ref(impl_did).skip_binder().self_ty().kind(),
            ty::Adt(def, _) if def.did() == owner_def.did()
        )
    })
}

/// The plain item name of a component marker type — `BarProviderComponent` for the
/// `DelegateComponent<BarProviderComponent>` key — which is the identifier a programmer writes on
/// the left of a `delegate_components!` entry. Falls back to the marker's printed form when it is
/// not an ADT (which a real component marker always is).
fn component_marker_name<'tcx>(tcx: TyCtxt<'tcx>, marker: Ty<'tcx>) -> String {
    match marker.kind() {
        ty::Adt(def, _) => tcx.item_name(def.did()).to_string(),
        _ => marker.to_string(),
    }
}

/// Whether `ty` is CGP's type-level path spine `PathCons<…>` — the key shape an `open`/namespace
/// redirect looks up, as opposed to a bare component marker. Anchored by `DefId` to
/// [`CGP_BASE_TYPES_CRATE`].
fn is_path_cons(tcx: TyCtxt<'_>, ty: Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(tcx, def.did(), PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE))
}

/// Classify why the `HasField` bound on `owner` for `field` is unmet: whether `owner` genuinely
/// lacks the field, carries it directly (so only the `HasField` impl or its type is at fault), or
/// reaches it only through a `Deref` target the derive does not cross.
fn field_issue<'tcx>(tcx: TyCtxt<'tcx>, owner: Ty<'tcx>, field: &str) -> FieldIssue {
    if adt_has_field(owner, field) {
        return FieldIssue::Present;
    }
    let mut current = owner;
    for _ in 0..MAX_DEREF {
        let Some(target) = deref_target(tcx, current) else {
            break;
        };
        if adt_has_field(target, field) {
            return FieldIssue::PresentViaDeref {
                target: target.to_string(),
            };
        }
        current = target;
    }
    FieldIssue::Missing
}

/// The declared type of `ty`'s named field `field`, as a display string, or `None` when `ty` is
/// not a struct or has no such field. Read from the struct's `DefId` with the type's own generic
/// arguments substituted, so a same-named struct in another module is never queried and a generic
/// context's field type is instantiated correctly.
fn field_type<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>, field: &str) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !def.is_struct() {
        return None;
    }
    let field_def = def
        .non_enum_variant()
        .fields
        .iter()
        .find(|f| f.name.as_str() == field)?;
    let field_ty = field_def.ty(tcx, args).skip_norm_wip();
    Some(tcx.erase_and_anonymize_regions(field_ty).to_string())
}

/// Whether `ty` is a struct with a named field called `field`. Only a struct can carry named
/// fields a `HasField` derive would key on, so an enum, tuple, or non-ADT is never a match.
fn adt_has_field(ty: Ty<'_>, field: &str) -> bool {
    match ty.kind() {
        ty::Adt(def, _) if def.is_struct() => def
            .non_enum_variant()
            .fields
            .iter()
            .any(|f| f.name.as_str() == field),
        _ => false,
    }
}

/// The `Deref::Target` of `ty`, read straight from the concrete `impl Deref for ty` rather than by
/// normalizing a projection, so it needs no inference context. Returns `None` when `ty` has no
/// matching `Deref` impl. Matches the impl by its `Self` type, so a generic `Deref` impl whose
/// `Self` is not exactly `ty` is skipped — sufficient for the concrete contexts a check targets.
fn deref_target<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let deref_trait = tcx.lang_items().deref_trait()?;
    let ty = tcx.erase_and_anonymize_regions(ty);

    for impl_did in tcx.all_impls(deref_trait) {
        let impl_self = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        if tcx.erase_and_anonymize_regions(impl_self) != ty {
            continue;
        }
        // The `Deref` impl's single associated type is its `Target`; its value is the target type.
        for assoc in tcx.associated_items(impl_did).in_definition_order() {
            if assoc.kind.tag() == ty::AssocTag::Type {
                let target = tcx
                    .type_of(assoc.def_id)
                    .instantiate_identity()
                    .skip_norm_wip();
                return Some(tcx.erase_and_anonymize_regions(target));
            }
        }
    }
    None
}
