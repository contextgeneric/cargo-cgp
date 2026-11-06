//! The facts about the crate the emitter reads from the compiler once it can answer.

use std::cell::OnceCell;
use std::sync::LazyLock;

use cargo_cgp_error_processing::{AmbiguousNames, Origin};

use crate::{detect, resolve};

/// No ambiguous names, for a message post-processed before the compiler can say which names are
/// ambiguous, which strips every qualifier.
pub static EMPTY_AMBIGUOUS_NAMES: LazyLock<AmbiguousNames> = LazyLock::new(AmbiguousNames::default);

/// What the emitter knows about the crate being compiled, each fact decided on the first
/// diagnostic for which the compiler can answer and kept for the rest of the compilation.
#[derive(Default)]
pub struct CrateFacts {
    /// Whether the crate uses CGP ([`resolve::query::crate_uses_cgp`]). A crate that does not use
    /// CGP can carry no CGP encoding, so its untransformed diagnostics are left exactly as rustc
    /// built them.
    pub cgp_used: OnceCell<bool>,
    /// The type names that name more than one item ([`detect::ambiguous_type_names`]), so the
    /// post-processing keeps the qualifier that tells such a name's items apart. This set knows
    /// the glob-imported names alone.
    pub ambiguous: OnceCell<AmbiguousNames>,
    /// The same, also knowing the items those globs bring in, read from HIR, so it serves only a
    /// diagnostic the tool rewrote, which arrives after lowering.
    pub ambiguous_with_glob_items: OnceCell<AmbiguousNames>,
}

impl CrateFacts {
    /// Whether the crate being compiled uses CGP, decided once the compiler can answer. Before then
    /// (a parse or name-resolution error) the answer is `false`, which leaves the diagnostic as
    /// rustc built it.
    pub fn uses_cgp(&self) -> bool {
        if let Some(&known) = self.cgp_used.get() {
            return known;
        }
        let Some(known) =
            rustc_middle::ty::tls::with_opt(|tcx| tcx.and_then(resolve::query::crate_uses_cgp))
        else {
            return false;
        };
        let _ = self.cgp_used.set(known);
        known
    }

    /// The type names that name more than one item, decided once the compiler can answer. Before
    /// then the set is empty, which strips every qualifier. A message the tool rewrote also knows
    /// which item each glob import brings in. Reading that needs the HIR, which an untransformed
    /// diagnostic emitted during name resolution would force the compiler to build under the
    /// emitter, so an untransformed message gets the glob-imported names alone.
    pub fn ambiguous_names(&self, origin: Origin) -> &AmbiguousNames {
        let read_glob_items = origin == Origin::Rewritten;
        let cell = if read_glob_items {
            &self.ambiguous_with_glob_items
        } else {
            &self.ambiguous
        };
        if let Some(names) = cell.get() {
            return names;
        }
        match rustc_middle::ty::tls::with_opt(|tcx| {
            tcx.and_then(|tcx| detect::ambiguous_type_names(tcx, read_glob_items))
        }) {
            Some(names) => cell.get_or_init(|| names),
            None => &EMPTY_AMBIGUOUS_NAMES,
        }
    }
}
