//! The traits a method call by a given name could resolve through.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Symbol;
use rustc_span::def_id::DefId;

use crate::resolve::query::{is_blanket_trait, is_consumer_trait, trait_items_named};

/// What kind of trait a call resolves through, which decides the class its failure is headed with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTrait {
    /// A CGP consumer trait (`[CGP-E001]`).
    Consumer,
    /// A `#[cgp_fn]` / `#[blanket_trait]` blanket trait (`[CGP-E003]`).
    Blanket,
}

/// Every trait declaring a `self`-receiver associated fn named `method` that a call by that name
/// could resolve through — the candidates
/// [`resolve_call_site`](crate::resolve::resolve_call_site) tries — each paired with that method's
/// `DefId` (whose declared signature the call-site anchor's `seed_from_call` unifies the call's
/// arguments against) and the [kind](CallTrait) of trait it is.
///
/// Two kinds qualify, and CGP consumer traits come **first** so a directly-wired consumer keeps
/// its precise `[CGP-E001]` recovery before any other trait is tried:
///
/// - a **CGP consumer trait** (recognized structurally, in any crate); and
/// - a **blanket-impl trait** that is not a consumer and shows CGP evidence — the shape `#[cgp_fn]`
///   / `#[blanket_trait]` generate (`impl<Context> Describe for Context where Self: …`). Its
///   obligation `Ctx: Describe` is walkable exactly as a consumer's is, since its `Self` is the
///   context.
///
/// CGP's extensible-data traits are left to the
/// [data-call anchor](crate::resolve::resolve_data_call), which types a receiver through a chain of
/// data operations.
///
/// Requiring CGP evidence of the second kind excludes ordinary blanket traits (`Into`, `From`); the
/// `self`-method requirement and the failing-obligation gate in
/// [`resolve_call_site`](crate::resolve::resolve_call_site) keep a wrong guess from fabricating a
/// diagnostic.
pub fn traits_with_method(tcx: TyCtxt<'_>, method: Symbol) -> Vec<(DefId, DefId, CallTrait)> {
    let mut found: Vec<(DefId, DefId, CallTrait)> = Vec::new();
    let with_method = trait_items_named(tcx, method, |item| {
        matches!(item.kind, ty::AssocKind::Fn { has_self: true, .. })
    });
    for (trait_did, method_did) in with_method {
        if is_consumer_trait(tcx, trait_did) {
            found.push((trait_did, method_did, CallTrait::Consumer));
        } else if is_blanket_trait(tcx, trait_did) {
            found.push((trait_did, method_did, CallTrait::Blanket));
        }
    }
    // A stable sort keeps each kind in the order the traits were listed.
    found.sort_by_key(|&(_, _, kind)| kind as u8);
    found
}
