//! Typed root-cause resolution for CGP check-trait failures.
//!
//! This is the compiler-internals half of the diagnostic replacement. When the emitter sees a
//! trait-bound error whose caret sits on a `check_components!` entry, it asks this module to
//! recover the *real* root cause by re-running the check obligation through the trait solver
//! rather than by reading the rendered error text.
//!
//! The flow, all DefId-anchored to the CGP crates so a same-named type from elsewhere can
//! never drive it:
//!
//! 1. A `check_components!` entry expands to `impl __CheckCtx<Marker, Params> for Ctx {}`,
//!    whose check trait carries `CanUseComponent<Marker, Params>` as a supertrait. We find
//!    the impl whose `Self` type span equals the diagnostic's primary span — that is the
//!    entry the error is about — and instantiate the supertrait with the impl's trait ref to
//!    get the concrete obligation `Ctx: CanUseComponent<Marker, Params>`.
//! 2. We register that obligation in a fresh `ObligationCtxt` and solve it. This runs *during*
//!    trait solving — the emitter reaches the live `TyCtxt` through `ty::tls` while a check
//!    error is being emitted — yet a fresh inference context re-entered here solves cleanly;
//!    that re-entrancy is the load-bearing assumption behind the whole design.
//! 3. The first fulfillment error whose leaf is a genuine `cgp_field::HasField` bound *is* the
//!    root cause: its `Symbol!` argument is decoded structurally (walking the `Chars` spine)
//!    into the missing field name. Anything else yields `None`, and the caller falls back to
//!    the untouched text-rewrite pipeline.

use rustc_hir::ItemKind;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, Ty, TyCtxt, TypingMode};
use rustc_span::Span;
use rustc_span::def_id::DefId;
use rustc_trait_selection::traits::{Obligation, ObligationCause, ObligationCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE,
    HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};

/// A recovered root cause, in owned form so it outlives the inference context it was read
/// from. Today the only variant is a missing `HasField`; more leaf kinds will join it.
pub enum RootCause {
    /// A provider the context is wired to needs a field the context's struct does not have.
    MissingField {
        /// The context type, e.g. `Rectangle`.
        context: String,
        /// The missing field name, decoded from the `Symbol!`, e.g. `height`.
        field: String,
        /// The component marker whose provider needs the field, e.g. `AreaCalculatorComponent`.
        marker: String,
    },
}

/// Whether `def_id` is a trait/type named `name` defined by crate `krate` — the DefId anchor
/// that keeps a same-named item from an unrelated crate from driving resolution, exactly as
/// `component_map::is_cgp_is_provider_for` does for `IsProviderFor`.
fn is_cgp_item(tcx: TyCtxt<'_>, def_id: DefId, name: &str, krate: &str) -> bool {
    tcx.item_name(def_id).as_str() == name && tcx.crate_name(def_id.krate).as_str() == krate
}

/// Resolve the root cause of the check failure whose diagnostic caret sits at `primary_span`,
/// or `None` if this is not a resolvable `CanUseComponent` check failure (in which case the
/// caller leaves the original diagnostic to the text-rewrite fallback).
pub fn resolve_check_failure(tcx: TyCtxt<'_>, primary_span: Span) -> Option<RootCause> {
    for trait_did in tcx.all_traits_including_private() {
        let Some(super_clause) = can_use_component_supertrait(tcx, trait_did) else {
            continue;
        };

        for impl_did in tcx.all_impls(trait_did) {
            // The entry the error is about is the impl whose `Self` type carries the caret's
            // span (the macro re-spans the context type onto the entry, so they coincide).
            if impl_self_ty_span(tcx, impl_did) != Some(primary_span) {
                continue;
            }

            let trait_ref = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip();
            let concrete = super_clause.instantiate_supertrait(tcx, ty::Binder::dummy(trait_ref));

            if let Some(cause) = solve_missing_field(tcx, concrete) {
                return Some(cause);
            }
        }
    }
    None
}

/// The `CanUseComponent<..>` supertrait clause of `trait_did`, if it carries one — the marker
/// of a `check_components!` check trait. Anchored by DefId to `cgp_component`.
fn can_use_component_supertrait(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<ty::Clause<'_>> {
    for &(clause, _) in tcx.explicit_super_predicates_of(trait_did).skip_binder() {
        if let Some(tp) = clause.as_trait_clause()
            && is_cgp_item(
                tcx,
                tp.def_id(),
                CAN_USE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        {
            return Some(clause);
        }
    }
    None
}

/// The source span of an impl's `Self` type, e.g. the `Rectangle` in
/// `impl __CheckRectangle<..> for Rectangle {}` — which the check macro re-spans onto the
/// `check_components!` entry, so it matches the failing diagnostic's primary span.
fn impl_self_ty_span(tcx: TyCtxt<'_>, impl_did: DefId) -> Option<Span> {
    let local = impl_did.as_local()?;
    match tcx.hir_expect_item(local).kind {
        ItemKind::Impl(imp) => Some(imp.self_ty.span),
        _ => None,
    }
}

/// Bound on how many CGP obligations the resolver descends through before giving up, so a
/// pathological or cyclic wiring cannot make the descent loop. Real dependency chains are far
/// shorter than this.
const MAX_DEPTH: u32 = 32;

/// Solve the concrete `Ctx: CanUseComponent<Marker, Params>` obligation and, if it bottoms out
/// on a genuine `HasField` leaf, return that missing field as the root cause.
fn solve_missing_field<'tcx>(tcx: TyCtxt<'tcx>, concrete: ty::Clause<'tcx>) -> Option<RootCause> {
    // The marker is the second argument of `CanUseComponent<Ctx, Marker, Params>`.
    let marker = concrete
        .as_trait_clause()?
        .skip_binder()
        .trait_ref
        .args
        .type_at(1);
    let marker = adt_name(tcx, marker)?;

    let (context, field) = find_missing_field(tcx, concrete.as_predicate(), 0)?;
    Some(RootCause::MissingField {
        context,
        field,
        marker,
    })
}

/// Solve `predicate` and return the missing `HasField` it ultimately fails on, as
/// `(context, field)`. The solver often reports the failure at an intermediate CGP obligation
/// (an `IsProviderFor`/`CanUseComponent` for a provider one dependency layer down) rather than
/// at the `HasField` leaf itself, so when a failing leaf is such an obligation we re-solve it
/// to descend one more layer — repeating until a real `HasField` surfaces or [`MAX_DEPTH`] is
/// reached. Descent is confined to CGP wiring traits (DefId-anchored), so an unmet *ordinary*
/// bound simply yields `None` and the diagnostic is left to the text-rewrite fallback.
fn find_missing_field<'tcx>(
    tcx: TyCtxt<'tcx>,
    predicate: ty::Predicate<'tcx>,
    depth: u32,
) -> Option<(String, String)> {
    if depth > MAX_DEPTH {
        return None;
    }

    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new_with_diagnostics(&infcx);
    ocx.register_obligation(Obligation::new(
        tcx,
        ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        predicate,
    ));
    let errors = ocx.evaluate_obligations_error_on_ambiguity();

    // Prefer a `HasField` leaf reported directly at this layer.
    for err in &errors {
        if let Some(leaf) = err.obligation.predicate.as_trait_clause() {
            let leaf = leaf.skip_binder().trait_ref;
            if is_cgp_item(tcx, leaf.def_id, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
                let context = leaf.self_ty().to_string();
                let field = decode_symbol(tcx, leaf.args.type_at(1))?;
                return Some((context, field));
            }
        }
    }

    // Otherwise descend through the first intermediate CGP wiring obligation.
    for err in &errors {
        if let Some(leaf) = err.obligation.predicate.as_trait_clause()
            && is_descendable_cgp(tcx, leaf.def_id())
            && let Some(found) = find_missing_field(tcx, err.obligation.predicate, depth + 1)
        {
            return Some(found);
        }
    }
    None
}

/// Whether a failing leaf trait is a CGP wiring obligation worth re-solving to descend one
/// dependency layer deeper — `IsProviderFor` or `CanUseComponent`, both defined by
/// `cgp-component`. An ordinary trait is deliberately excluded so the descent never strays
/// outside CGP's own machinery.
fn is_descendable_cgp(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    is_cgp_item(tcx, def_id, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, def_id, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
}

/// The item name of `ty` when it is an ADT (`Rectangle`, `AreaCalculatorComponent`, …).
fn adt_name(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    match ty.kind() {
        ty::Adt(def, _) => Some(tcx.item_name(def.did()).to_string()),
        _ => None,
    }
}

/// Decode a CGP `Symbol!` type into its string, by walking the `Chars<'c', Tail>` spine and
/// reading each `char` const argument until `Nil`. Anchored to `cgp_base_types`, and returns
/// `None` for any type that is not a well-formed `Symbol`.
fn decode_symbol(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), "Symbol", CGP_BASE_TYPES_CRATE) {
        return None;
    }

    // `Symbol<const LEN, Chars>` — the second argument is the head of the `Chars` spine.
    let mut current = args.type_at(1);
    let mut name = String::new();
    loop {
        let ty::Adt(def, args) = current.kind() else {
            return None;
        };
        let item = tcx.item_name(def.did());
        if is_cgp_item(tcx, def.did(), "Nil", CGP_BASE_TYPES_CRATE) {
            break;
        }
        if item.as_str() != "Chars"
            || tcx.crate_name(def.did().krate).as_str() != CGP_BASE_TYPES_CRATE
        {
            return None;
        }

        // `Chars<const CHAR: char, Tail>` — read the char, then follow the tail.
        let scalar = args.const_at(0).try_to_value()?.valtree.try_to_leaf()?;
        name.push(char::from_u32(scalar.to_u32())?);
        current = args.type_at(1);
    }
    Some(name)
}
