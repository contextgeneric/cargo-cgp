//! Recognizing an orphan-rule namespace registration and recovering what it registered.
//!
//! Registering wiring into a namespace lowers to `impl<Param> Namespace<Param> for Key`, and Rust's
//! orphan rule rejects it with `E0210` (or its sibling `E0117`) when both the namespace trait and
//! the key are foreign: a downstream crate registering into an upstream namespace it does not own,
//! keyed on an upstream component it does not own either. The raw diagnostic names the machinery
//! parameter (`__Components__` from a `#[default_impl]`/`#[prefix]`, `__Table__` from a
//! `cgp_namespace!` re-open) rather than the namespace and key the programmer wrote.
//!
//! [`classify_orphan_conflict`] recovers those from the compiler, not the error text: it finds the
//! offending impl at the caret, reads its trait and `Self`, and, anchoring the namespace trait by
//! its structural [namespace-lookup fingerprint](crate::resolve::query::is_namespace_lookup_trait)
//! rather than by name, words the collision into the rustc-free
//! [`OrphanConflict`]. Because we only run on an `E0210`/`E0117`, which rustc emits *only* for a
//! genuine orphan, matching the caret to a foreign-namespace-trait-for-foreign-key impl is the whole
//! confirmation: rustc has already proved the orphan, and the caret already sits on the impl it
//! proved it for.

use cargo_cgp_error_processing::{OrphanConflict, OrphanTrigger, WiringKey};
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::{CGP_COMPONENT_CRATE, CGP_MACRO_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::query::{
    byte_range, describe_key, is_cgp_item, is_namespace_lookup_trait, local_trait_impls,
};

/// One local impl of a foreign namespace-lookup trait for a foreign key: a candidate orphan
/// registration.
struct OrphanImpl<'tcx> {
    impl_did: DefId,
    def_span: Span,
    trait_did: DefId,
    key: Ty<'tcx>,
}

/// Classify an `E0210`/`E0117` at `primary_span` as an orphan-rule namespace registration, or
/// `None` when no such impl sits at the caret (so the caller leaves the diagnostic to the ordinary
/// fallback). The recovered impl is a *foreign* namespace-lookup trait implemented locally for a
/// *foreign* key; the trigger is read from the impl's own machinery parameter name so the fix can
/// be worded for the construct the programmer wrote.
pub fn classify_orphan_conflict(tcx: TyCtxt<'_>, primary_span: Span) -> Option<OrphanConflict> {
    let candidates = orphan_impls(tcx);

    // The offending impl is the one the caret sits on, matched by source range (the macro re-spans
    // its generated impl onto the invocation, whose range the coherence caret shares). When the
    // caret lands on none of them, as for a `#[default_impl]`, whose caret is the `#[cgp_impl]`
    // attribute and whose impl is spanned on its key, a lone candidate is still unambiguous, so fall
    // back to it; more than one leaves the choice ambiguous and declines. The fallback needs a caret
    // a CGP macro generated: an `E0117` names no machinery parameter, so a plain Rust orphan beside a
    // registration reaches this scan too, its caret on an impl the program or its own macro wrote.
    let orphan = candidates
        .iter()
        .find(|candidate| {
            byte_range(candidate.def_span).overlaps_or_adjacent(byte_range(primary_span))
        })
        .or(match candidates.as_slice() {
            [only] if is_from_cgp_macro(tcx, primary_span) => Some(only),
            _ => None,
        })?;

    let namespace = tcx.item_name(orphan.trait_did).to_string();
    // An `E0210` carries no type in its message, so nothing has printed one with trimmed paths yet,
    // and the first such print takes the diagnostic context's lock, which the emitter holds. So a
    // type the key spells prints by its own name.
    let key = with_forced_trimmed_paths!(describe_key(tcx, orphan.key, orphan.impl_did))?;
    // A blanket key is a namespace *join*, not a registration, so decline it rather than word a
    // nonsensical fix.
    if matches!(key, WiringKey::Blanket(_)) {
        return None;
    }

    Some(OrphanConflict {
        namespace,
        key,
        trigger: orphan_trigger(tcx, orphan.impl_did),
    })
}

/// Every local impl of a foreign namespace-lookup trait for a foreign key. These are exactly the
/// impls the orphan rule rejects for a namespace registration, whichever construct generated them.
fn orphan_impls(tcx: TyCtxt<'_>) -> Vec<OrphanImpl<'_>> {
    let mut impls = Vec::new();
    for local in local_trait_impls(tcx) {
        let impl_did = local.to_def_id();
        let trait_ref = tcx.impl_trait_ref(impl_did).skip_binder();
        let trait_did = trait_ref.def_id;
        // Foreign namespace trait: a local one would be no orphan (you may implement your own
        // trait for anything). Recognized by the single-`Delegate` fingerprint, so a downstream
        // crate's own namespace trait is treated the same as CGP's built-in `DefaultNamespace`.
        // `DelegateComponent` has that shape too, but its `Self` is a wiring table, not a key.
        if trait_did.is_local()
            || !is_namespace_lookup_trait(tcx, trait_did)
            || is_cgp_item(
                tcx,
                trait_did,
                DELEGATE_COMPONENT_TRAIT,
                CGP_COMPONENT_CRATE,
            )
        {
            continue;
        }
        // Foreign key: a component marker, a `PathCons<…>` path, or a per-type default's type,
        // defined in another crate or built in. A local key would be covered and again no orphan,
        // and a generic parameter is a blanket impl rather than a registration.
        let key = trait_ref.self_ty();
        if matches!(key.kind(), ty::Param(_)) || is_local_type(key) {
            continue;
        }
        // A local type among the trait's arguments covers the impl as well, as the local component
        // of `impl DefaultImpls1<LocalComponent, __Components__> for String` does: a legal per-type
        // registration, whatever its key.
        if trait_ref.args.types().skip(1).any(is_local_type) {
            continue;
        }
        impls.push(OrphanImpl {
            impl_did,
            def_span: tcx.def_span(impl_did),
            trait_did,
            key,
        });
    }
    impls
}

/// Whether `span` comes from a CGP macro's expansion. Every construct that registers into a namespace
/// is a `cgp_`-named macro of CGP's proc-macro crate. Reading the macro's definition holds under a
/// renaming import and rejects a program's own macro of a similar name.
fn is_from_cgp_macro(tcx: TyCtxt<'_>, span: Span) -> bool {
    span.macro_backtrace().any(|expansion| {
        expansion.macro_def_id.is_some_and(|macro_did| {
            tcx.crate_name(macro_did.krate).as_str() == CGP_MACRO_CRATE
                && tcx.item_name(macro_did).as_str().starts_with("cgp_")
        })
    })
}

/// Whether the orphan rule counts `ty` as local: a type this crate defines, through any references,
/// which the rule looks through. The other fundamental types, `Box` and `Pin`, are not read.
fn is_local_type(ty: Ty<'_>) -> bool {
    match ty.kind() {
        ty::Adt(def, _) => def.did().is_local(),
        ty::Ref(_, inner, _) => is_local_type(*inner),
        _ => false,
    }
}

/// Which construct generated the impl, read from its own machinery type parameter: a
/// `cgp_namespace!` re-open names it `__Table__`, while a `#[default_impl]`/`#[prefix]` registration
/// names it `__Components__`. The name is a reserved identifier the CGP macros emit, so it is a
/// reliable, `DefId`-independent discriminator.
fn orphan_trigger(tcx: TyCtxt<'_>, impl_did: DefId) -> OrphanTrigger {
    let is_table = tcx.generics_of(impl_did).own_params.iter().any(|param| {
        matches!(param.kind, ty::GenericParamDefKind::Type { .. })
            && param.name.as_str() == "__Table__"
    });
    if is_table {
        OrphanTrigger::Reopen
    } else {
        OrphanTrigger::Register
    }
}
