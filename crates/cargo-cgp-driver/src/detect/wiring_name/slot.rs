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

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, PATH_CONS_TYPE,
};
use crate::resolve::query::{has_context_param, is_cgp_item, local_trait_impls};

/// Where a name at a span sits.
pub enum NameSlot<'tcx> {
    /// The key of a wiring entry, as the HIR type the impl's trait reference carries.
    Key { key: &'tcx rustc_hir::Ty<'tcx> },
    /// The value of a wiring entry, with the entry's key as written and the entry's impl.
    Value {
        key: &'tcx rustc_hir::Ty<'tcx>,
        impl_did: LocalDefId,
    },
    /// The provider a `#[cgp_impl]` header names, the generated impl's `Self` type.
    CgpImplProvider,
    /// An undeclared type inside the `@`-path of a key or value (`@app.Missing`), found in the
    /// generated path spine: the entry's key as written, the entry's impl, whether the type sits in
    /// the value, and whether it is a whole segment of the path rather than nested in one.
    Inside {
        key: &'tcx rustc_hir::Ty<'tcx>,
        impl_did: LocalDefId,
        in_value: bool,
        whole_segment: bool,
    },
}

/// The wiring entry or `#[cgp_impl]` header whose key, value, or provider is exactly the type at
/// `span`: a `DelegateComponent` impl (by `DefId`, from the `cgp-component` crate) whose trait
/// argument or `Delegate` type is that type, or an impl declaring the reserved `__Context__`
/// parameter whose unresolved `Self` type is. A name inside a larger key or value, such as a path
/// segment of an `@`-path, is [`NameSlot::Inside`].
pub fn name_slot_at(tcx: TyCtxt<'_>, span: Span) -> Option<NameSlot<'_>> {
    let mut inside = None;
    for local in local_trait_impls(tcx) {
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
            if let Some(whole_segment) = path_segment_at(tcx, key, span, false) {
                inside = Some(NameSlot::Inside {
                    key,
                    impl_did: local,
                    in_value: false,
                    whole_segment,
                });
            }
            for &item_id in imp.items {
                let item = tcx.hir_impl_item(item_id);
                if let ImplItemKind::Type(value) = item.kind
                    && let Some(whole_segment) = path_segment_at(tcx, value, span, false)
                {
                    inside = Some(NameSlot::Inside {
                        key,
                        impl_did: local,
                        in_value: true,
                        whole_segment,
                    });
                }
            }
        } else if has_context_param(tcx, local)
            && spans_exactly(imp.self_ty.span, span)
            && matches!(imp.self_ty.kind, TyKind::Path(QPath::Resolved(None, path)) if path.res == Res::Err)
        {
            return Some(NameSlot::CgpImplProvider);
        }
    }
    inside
}

/// Whether `ty_span` covers exactly the source range of `span`, whatever their expansion contexts.
fn spans_exactly(ty_span: Span, span: Span) -> bool {
    ty_span.lo() == span.lo() && ty_span.hi() == span.hi()
}

/// Whether the unresolved type at `span` sits inside the path spine of `ty`, a `PathCons` chain
/// that `delegate_components!` builds from an `@`-path, and if so whether it is a whole segment of
/// the path (the first argument of a `PathCons`) rather than nested in a segment's generic
/// arguments. The macro gives the spine its own span and each segment the span the user wrote, so
/// the unresolved type is found by its own span, as a path that resolves to nothing. `in_spine` says
/// whether an enclosing type is already a `PathCons`.
fn path_segment_at(
    tcx: TyCtxt<'_>,
    ty: &rustc_hir::Ty<'_>,
    span: Span,
    in_spine: bool,
) -> Option<bool> {
    let TyKind::Path(QPath::Resolved(None, path)) = ty.kind else {
        return None;
    };
    if path.res == Res::Err && spans_exactly(ty.span, span) {
        return in_spine.then_some(false);
    }
    let is_spine = path
        .res
        .opt_def_id()
        .is_some_and(|did| is_cgp_item(tcx, did, PATH_CONS_TYPE, CGP_BASE_TYPES_CRATE));
    let args = path.segments.iter().filter_map(|segment| segment.args);
    let mut types = args.flat_map(|args| args.args).filter_map(|arg| match arg {
        GenericArg::Type(arg) => Some(arg.as_unambig_ty()),
        _ => None,
    });
    let in_spine = in_spine || is_spine;
    if is_spine && let Some(segment) = types.next() {
        if let TyKind::Path(QPath::Resolved(None, segment_path)) = segment.kind
            && segment_path.res == Res::Err
            && spans_exactly(segment.span, span)
        {
            return Some(true);
        }
        if let found @ Some(_) = path_segment_at(tcx, segment, span, in_spine) {
            return found;
        }
    }
    types.find_map(|arg| path_segment_at(tcx, arg, span, in_spine))
}
