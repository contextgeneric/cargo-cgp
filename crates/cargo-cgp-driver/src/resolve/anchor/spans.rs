//! Finding the local items a diagnostic's spans land on.
//!
//! Every span-matching anchor starts from these lookups: the trait impls a failure can surface
//! inside (and whether a caret inside one is about it), and the struct/enum definitions a use-site
//! failure's spans touch.

use cargo_cgp_error_processing::byte_range::{ByteRange, CaretPlace, caret_place};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, Interner as _, Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::call_site::method_calls_at;
use crate::resolve::query::{
    byte_range, is_component_marker, is_provider_struct, local_trait_impls,
};

/// The local trait-impl blocks (`impl Trait for Ty { … }`) whose source span contains one of the
/// diagnostic's spans: the impls a wiring failure can surface inside, whether the caret lands on
/// the impl header or deep in a method body. A trait impl (not an inherent one) is required
/// because the recovery reads the impl's trait supertraits. The *full* item span is taken from HIR
/// rather than `def_span`, which for an impl covers only the header and would miss a caret inside
/// the body (a forwarding `self.method(..)` call, say).
pub fn enclosing_trait_impls(tcx: TyCtxt<'_>, spans: &[Span]) -> Vec<DefId> {
    let mut impls = Vec::new();
    for local in local_trait_impls(tcx) {
        let impl_span = tcx.hir_expect_item(local).span;
        if spans.iter().any(|&span| impl_span.contains(span)) {
            impls.push(local.to_def_id());
        }
    }
    impls
}

/// The candidate context types of a use-site failure: every local struct or enum whose definition
/// span contains one of the diagnostic's spans: for an `E0599` method error that includes the
/// "method not found for this struct" span on the receiver's type. Each ADT is returned with
/// identity arguments (so a generic context keeps its generic form). A provider struct that merely
/// shares a span (rustc names it in a "required for … to implement …" note) is excluded, since a
/// provider is never a context; the caller then picks the candidate that actually wires a failing
/// component. A span on the ADT's generics is left out too: rustc's "required by a bound in `Holder`"
/// note lands on a bound of `struct Holder<T: Trait>`, which `Holder` asks of what it holds.
pub fn context_candidates_from_spans<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<Ty<'tcx>> {
    let mut candidates = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        let did = local.to_def_id();
        if !matches!(tcx.def_kind(did), DefKind::Struct | DefKind::Enum) {
            continue;
        }
        let def_span = tcx.def_span(did);
        let generics_span = tcx.hir_get_generics(local).map(|generics| generics.span);
        let names_adt = |span: Span| {
            def_span.contains(span) && !generics_span.is_some_and(|inner| inner.contains(span))
        };
        if spans.iter().any(|&span| names_adt(span)) {
            let ty = tcx.type_of(did).instantiate_identity().skip_norm_wip();
            // A provider struct or a component marker is never a context; exclude both so an
            // anchor does not mis-recover one as the failing context. A marker's definition sits
            // under the component macro's attribute, which rustc's notes point at.
            if !is_provider_struct(tcx, ty) && !is_component_marker(tcx, ty) {
                candidates.push(ty);
            }
        }
    }
    candidates
}

/// Whether one of the diagnostic's primary `spans` is about the impl itself rather than about other
/// code its methods run: a caret on the impl outside every method body (its header), or a caret on
/// a call inside a body to a method that the impl's trait or one of its supertraits declares (a
/// forwarding `self.greet()`, which fails because the supertrait does). An unrelated error inside
/// a body, such as an `Rc` passed to a `Send` bound, is neither, so it keeps its own diagnostic.
pub fn is_about_impl(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId, spans: &[Span]) -> bool {
    let Some(local) = impl_did.as_local() else {
        return false;
    };
    let impl_range = byte_range(tcx.hir_expect_item(local).span);
    let bodies: Vec<ByteRange> = tcx
        .associated_item_def_ids(impl_did)
        .iter()
        .filter_map(|item| tcx.hir_maybe_body_owned_by(item.as_local()?))
        .map(|body| byte_range(body.value.span))
        .collect();
    let place = |span: Span| caret_place(byte_range(span), impl_range, &bodies);
    if spans
        .iter()
        .any(|&span| place(span) == CaretPlace::ImplHeader)
    {
        return true;
    }
    let in_bodies: Vec<Span> = spans
        .iter()
        .copied()
        .filter(|&span| place(span) == CaretPlace::MethodBody)
        .collect();
    if in_bodies.is_empty() {
        return false;
    }
    let traits: Vec<DefId> = std::iter::once(trait_did)
        .chain(tcx.supertrait_def_ids(trait_did))
        .collect();
    method_calls_at(tcx, &in_bodies).iter().any(|call| {
        traits.iter().any(|&trait_did| {
            tcx.associated_items(trait_did)
                .filter_by_name_unhygienic(call.method)
                .any(|item| matches!(item.kind, ty::AssocKind::Fn { .. }))
        })
    })
}
