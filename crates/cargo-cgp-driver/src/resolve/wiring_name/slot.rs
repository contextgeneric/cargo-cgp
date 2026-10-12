//! Finding the wiring entry or the `#[cgp_impl]` header a name sits in.
//!
//! `delegate_components!` keeps the key and the value the user wrote as the spans of the generated
//! `impl DelegateComponent<Key> for Context { type Delegate = Value; }`, and `#[cgp_impl(Provider)]`
//! keeps the provider's span as the impl's `Self` type. A name that does not resolve carries no
//! trace of the macro in its own span, so the position is read back from the HIR.

use rustc_hir::def::Res;
use rustc_hir::def_id::LocalDefId;
use rustc_hir::{GenericArg, ImplItemKind, ItemKind, QPath, TyKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::cgp_item::is_cgp_item;
use crate::resolve::unimported::has_context_param;

/// Where a name at a span sits.
pub(crate) enum NameSlot<'tcx> {
    /// The key of a wiring entry, as the HIR type the impl's trait reference carries.
    Key { key: &'tcx rustc_hir::Ty<'tcx> },
    /// The value of a wiring entry, with the entry's key as written and the entry's impl.
    Value {
        key: &'tcx rustc_hir::Ty<'tcx>,
        impl_did: LocalDefId,
    },
    /// The provider a `#[cgp_impl]` header names, the generated impl's `Self` type.
    CgpImplProvider,
}

/// The wiring entry or `#[cgp_impl]` header whose key, value, or provider is exactly the type at
/// `span`: a `DelegateComponent` impl (by `DefId`, from the `cgp-component` crate) whose trait
/// argument or `Delegate` type is that type, or an impl declaring the reserved `__Context__`
/// parameter whose unresolved `Self` type is. A name inside a larger key or value, such as a path
/// segment of an `@`-path, is not the entry's key or value, so it gives `None`.
pub(crate) fn name_slot_at(tcx: TyCtxt<'_>, span: Span) -> Option<NameSlot<'_>> {
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(
            tcx.def_kind(local),
            rustc_hir::def::DefKind::Impl { of_trait: true }
        ) {
            continue;
        }
        let ItemKind::Impl(imp) = tcx.hir_expect_item(local).kind else {
            continue;
        };
        let Some(header) = imp.of_trait else {
            continue;
        };
        let trait_path = header.trait_ref.path;
        let is_wiring = trait_path.res.opt_def_id().is_some_and(|trait_did| {
            is_cgp_item(
                tcx,
                trait_did,
                DELEGATE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        });
        if is_wiring {
            let Some(key) = trait_path
                .segments
                .last()
                .and_then(|segment| segment.args)
                .and_then(|args| {
                    args.args.iter().find_map(|arg| match arg {
                        GenericArg::Type(ty) => Some(ty.as_unambig_ty()),
                        _ => None,
                    })
                })
            else {
                continue;
            };
            if spans_exactly(key.span, span) {
                return Some(NameSlot::Key { key });
            }
            let value_at = imp.items.iter().any(|&item_id| {
                let item = tcx.hir_impl_item(item_id);
                matches!(item.kind, ImplItemKind::Type(value) if spans_exactly(value.span, span))
            });
            if value_at {
                return Some(NameSlot::Value {
                    key,
                    impl_did: local,
                });
            }
        } else if has_context_param(tcx, local)
            && spans_exactly(imp.self_ty.span, span)
            && matches!(imp.self_ty.kind, TyKind::Path(QPath::Resolved(None, path)) if path.res == Res::Err)
        {
            return Some(NameSlot::CgpImplProvider);
        }
    }
    None
}

/// Whether `ty_span` covers exactly the source range of `span`, whatever their expansion contexts.
fn spans_exactly(ty_span: Span, span: Span) -> bool {
    ty_span.lo() == span.lo() && ty_span.hi() == span.hi()
}
