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
        // A bare `DelegateComponent<Key>` on a type *other* than the context is a non-context
        // delegation table missing an entry — an aggregate provider missing a component wiring, or a
        // `UseDelegate`/`UseInputDelegate` table missing a branch for the type it dispatches on (a
        // `Code` fragment or an `Input` value's type). The table wires other keys but not this one.
        // The key is named in full (it may be a dispatched-on type, not just a marker), and the owner
        // is the table. (`is_reportable_leaf` only lets a real delegation table reach here.)
        if self_ty != context {
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
/// is a real root cause in two shapes:
///
/// - on the **context** itself, the context does not wire the component (a [`Leaf::MissingWiring`]);
/// - on a **dispatch table** — a `UseDelegate`/`UseInputDelegate` inner table, recognized because it
///   wires *some* other key (it has at least one `DelegateComponent` impl) — the table has no entry
///   for the type it dispatches on (a [`Leaf::MissingDispatchEntry`]).
///
/// It is dropped only when it lands on a type that is neither. The case that makes the gate
/// load-bearing (not just cautious): a **leaf provider** whose concrete impl fixes an input type the
/// walk cannot match. A pipeline stage like `HandleShout` (`impl Handler<Code, String>`) fed an
/// *unknown* input — a call-site placeholder an earlier stage's `::Output` never resolved — does not
/// unify with its concrete impl, so `impl_where_obligations` falls through to the delegation blanket
/// and produces an unmet `HandleShout: DelegateComponent<HandlerComponent>`. That is a dead-end, not
/// a missing entry: `HandleShout` is a valid provider, it simply is not a table. It carries no
/// `DelegateComponent` impl, so `is_delegation_table` returns `false` and it is dropped — where a real
/// table (which wires *some* key) is kept.
pub(crate) fn is_reportable_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
) -> bool {
    let did = leaf_ref.def_id;
    if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE) {
        let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
        return self_ty == context || is_delegation_table(tcx, did, self_ty);
    }
    !is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        && !is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
}

/// Whether `owner` is a delegation table — a struct that wires at least one key through a
/// `DelegateComponent` impl (an aggregate provider, or a `UseDelegate`/`UseInputDelegate` inner
/// table). This tells a table *missing one key* (a real root cause) apart from a leaf provider
/// reached only via the delegation blanket because its concrete impl did not unify (a routing
/// dead-end). Having *any* one entry is enough to be recognized as a table — the owner it excludes
/// is one with zero delegation impls, which is exactly the leaf-provider dead-end. `delegate_did` is
/// the `DelegateComponent` trait's own `DefId`, taken from the leaf being classified.
fn is_delegation_table<'tcx>(tcx: TyCtxt<'tcx>, delegate_did: DefId, owner: Ty<'tcx>) -> bool {
    let ty::Adt(owner_def, _) = owner.kind() else {
        return false;
    };
    tcx.all_impls(delegate_did).any(|impl_did| {
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
