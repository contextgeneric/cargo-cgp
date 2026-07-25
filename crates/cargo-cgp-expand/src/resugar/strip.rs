//! Dropping the `cgp::macro_prelude::` qualifier the CGP macros emit.
//!
//! Every construct a CGP macro references is emitted fully qualified, so the expansion is full of
//! `::cgp::macro_prelude::DelegateComponent` where the programmer wrote `DelegateComponent`. That
//! qualifier is noise nobody wrote, and stripping it is what makes the rest of the expansion read
//! like source. General module qualifiers are left alone — in source, unlike in a diagnostic, a
//! qualifier carries information.

use syn::visit_mut::{self, VisitMut};
use syn::{ExprPath, Path, TypePath};

/// How many segments the qualifier occupies: `cgp` and `macro_prelude`.
const PRELUDE_SEGMENTS: usize = 2;

/// The prelude-qualifier strip. Optional, controlled by
/// [`ExpandOptions::strip_cgp_prefixes`](crate::ExpandOptions::strip_cgp_prefixes).
pub struct StripPrelude;

impl VisitMut for StripPrelude {
    fn visit_path_mut(&mut self, path: &mut Path) {
        visit_mut::visit_path_mut(self, path);
        strip_prelude(path);
    }

    // A **qualified** path — `<P as DelegateComponent<C>>::Delegate` — indexes into its own
    // segments to say where the qualifier ends, so dropping segments from the front without
    // moving that index leaves the two inconsistent. The printer asserts on exactly that, so the
    // two node kinds that carry a `QSelf` correct it here. Generated CGP code is full of them.
    fn visit_type_path_mut(&mut self, node: &mut TypePath) {
        let before = node.path.segments.len();
        visit_mut::visit_type_path_mut(self, node);
        correct_qself(&mut node.qself, before, node.path.segments.len());
    }

    fn visit_expr_path_mut(&mut self, node: &mut ExprPath) {
        let before = node.path.segments.len();
        visit_mut::visit_expr_path_mut(self, node);
        correct_qself(&mut node.qself, before, node.path.segments.len());
    }
}

/// Drop a leading `cgp::macro_prelude::` from `path`, if it carries one.
///
/// Only the exact two-segment prefix is dropped, and only when something follows it, so a path
/// that merely starts with a `cgp` module keeps its qualifier.
fn strip_prelude(path: &mut Path) {
    let leading: Vec<String> = path
        .segments
        .iter()
        .take(PRELUDE_SEGMENTS)
        .map(|segment| segment.ident.to_string())
        .collect();

    if leading == ["cgp", "macro_prelude"] && path.segments.len() > PRELUDE_SEGMENTS {
        path.segments = path
            .segments
            .iter()
            .skip(PRELUDE_SEGMENTS)
            .cloned()
            .collect();
        // The prefix carried the path's `::` root, so clear it or the result reads `::Symbol`.
        path.leading_colon = None;
    }
}

/// Move a qualified path's `position` back by however many segments were stripped from the front,
/// so it still points at the segment it named.
fn correct_qself(qself: &mut Option<syn::QSelf>, before: usize, after: usize) {
    if let Some(qself) = qself {
        qself.position = qself.position.saturating_sub(before - after);
    }
}
