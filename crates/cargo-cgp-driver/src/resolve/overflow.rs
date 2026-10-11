//! Whether a wiring overflow follows from a type the compiler could not resolve.

use rustc_middle::ty::{TyCtxt, TypeVisitableExt as _};
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::anchor::check_entry_context;
use crate::resolve::cgp_item::find_cgp_trait;
use crate::resolve::conflict::local_delegate_impls;

/// Whether the context checked at `primary_span` wires some key with a type the compiler could not
/// resolve (a `{type error}` in an entry's key or `Delegate`, left by an earlier unresolved-name
/// error). A lookup through such an entry can overflow without any cycle in the wiring, so an
/// `E0275` there is not reported as one. `false` when no `check_components!` entry is at the span.
pub fn overflow_follows_type_error(tcx: TyCtxt<'_>, primary_span: Span) -> bool {
    let Some(context) = check_entry_context(tcx, primary_span) else {
        return false;
    };
    let Some(delegate_did) = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
    else {
        return false;
    };
    local_delegate_impls(tcx, delegate_did)
        .into_iter()
        .filter(|entry| entry.self_ty == context)
        .any(|entry| {
            entry.key.references_error() || entry.delegate.is_some_and(|d| d.references_error())
        })
}
