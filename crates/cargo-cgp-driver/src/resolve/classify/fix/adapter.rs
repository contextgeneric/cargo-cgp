//! The adapter types that make a wired type a provider of the trait its slot needs.

use cargo_cgp_error_processing::Adapter;
use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};
use rustc_span::def_id::DefId;

use crate::resolve::classify::known_obligation;
use crate::resolve::label::render_ty;
use crate::resolve::query::{holds, import_path, is_provider_trait};

/// The path prefix of the items the prelude exports, which need no `use` in a `help`.
const PRELUDE_PATH: &str = "cgp::prelude::";

/// How many adapters a `help` names at most.
const MAX_ADAPTERS: usize = 3;

/// The adapter types that make the provider of `parent`, a provider-trait obligation
/// `Provider: Trait<Context, …>` the wiring cannot meet, a provider of the trait: for each impl of
/// the trait whose `Self` is an ADT over one bare type parameter that the impl bounds by a trait
/// (`WithProvider<P>` where `P: FieldGetter`, `Promote<P>` where `P: Computer`), the ADT applied to
/// `Provider`, kept when the solver confirms it implements the trait for the same arguments. An ADT
/// that merely carries its parameter as a type (`UseType<T>`) bounds it by nothing, so it adapts no
/// provider and is never named. A fix is named only when the solver confirms it for the slot, never by an
/// adapter's name.
///
/// Empty when `parent` is not fully known (it holds an inference variable, a placeholder, a
/// parameter, or a projection), so no adapter is confirmed against a guess.
pub fn wrapping_adapters<'tcx>(tcx: TyCtxt<'tcx>, parent: ty::TraitRef<'tcx>) -> Vec<Adapter> {
    if !is_provider_trait(tcx, parent.def_id) {
        return Vec::new();
    }
    let Some(parent) = known_obligation(tcx, parent) else {
        return Vec::new();
    };
    let provider = parent.self_ty();
    let mut adapters: Vec<Adapter> = Vec::new();
    for impl_did in tcx.all_impls(parent.def_id) {
        if adapters.len() == MAX_ADAPTERS {
            break;
        }
        let impl_self = tcx.impl_trait_ref(impl_did).skip_binder().self_ty();
        let Some(candidate) = adapt(tcx, impl_did, impl_self, provider) else {
            continue;
        };
        let mut args = parent.args.to_vec();
        args[0] = candidate.into();
        let applied = ty::TraitRef::new_from_args(tcx, parent.def_id, tcx.mk_args(&args));
        if !holds(tcx, ty::Binder::dummy(applied).upcast(tcx)) {
            continue;
        }
        let ty = render_ty(tcx, candidate);
        if adapters.iter().any(|adapter| adapter.ty == ty) {
            continue;
        }
        let import = match candidate.kind() {
            ty::Adt(def, _) => {
                import_path(tcx, def.did()).filter(|path| !path.starts_with(PRELUDE_PATH))
            }
            _ => None,
        };
        adapters.push(Adapter { ty, import });
    }
    adapters
}

/// The adapter ADT of `impl_self` applied to `provider`, when `impl_self` is an ADT whose only
/// generic argument is one type parameter that `impl_did` bounds by a trait, and the ADT is not the
/// provider's own.
fn adapt<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_did: DefId,
    impl_self: Ty<'tcx>,
    provider: Ty<'tcx>,
) -> Option<Ty<'tcx>> {
    let ty::Adt(adapter, args) = impl_self.kind() else {
        return None;
    };
    let [arg] = args.as_slice() else {
        return None;
    };
    let param = arg.as_type()?;
    if !matches!(param.kind(), ty::Param(_)) {
        return None;
    }
    // The implicit `Sized` bound every type parameter carries does not make it a provider.
    let sized = tcx.lang_items().sized_trait();
    let bounded = tcx.clauses_of(impl_did).clauses.iter().any(|(clause, _)| {
        clause.as_trait_clause().is_some_and(|bound| {
            let bound = bound.skip_binder();
            bound.self_ty() == param && Some(bound.def_id()) != sized
        })
    });
    if !bounded {
        return None;
    }
    if matches!(provider.kind(), ty::Adt(own, _) if own.did() == adapter.did()) {
        return None;
    }
    Some(Ty::new_adt(tcx, *adapter, tcx.mk_args(&[provider.into()])))
}
