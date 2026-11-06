//! The stage for a structural mistake in a wiring or its checks.

use cargo_cgp_error_processing::{
    ClassPlan, FIRST_CHECK_TABLE_LABEL, SECOND_CHECK_TABLE_LABEL, StructuralMistake,
};
use rustc_errors::codes::{E0119, E0207, E0277, E0428};
use rustc_errors::{DiagArgValue, DiagInner};
use rustc_middle::ty::TyCtxt;

use crate::detect::{self, DuplicateCheck};
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};

/// The structural stage, with its memory: the structural mistakes already reported, so a mistake
/// rustc reports as several errors becomes one.
#[derive(Default)]
pub struct StructuralStage {
    /// The structural mistakes already reported.
    pub reported: Vec<StructuralMistake>,
}

impl StructuralStage {
    /// Reword an error on the generated items of a structural mistake, such as a check table
    /// listing one entry twice, to name the mistake and carry its fix, or drop a repeat of one
    /// already reported. One mistake can produce several errors: a loop's key and value variables,
    /// or each parameter of an entry, are unconstrained in the entry and in its `IsProviderFor`
    /// twin.
    pub fn run(&mut self, tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
        let mistake = match structural_mistake(tcx, diag) {
            Some(Found::Mistake(mistake)) => mistake,
            Some(Found::Restated) => return Verdict::Drop,
            None => return Verdict::Continue,
        };
        if self.reported.contains(&mistake) {
            return Verdict::Drop;
        }
        self.reported.push(mistake.clone());
        let span = match mistake {
            // rustc's suggestions to remove the parameter or add it to the context, or to add a
            // `where` clause to a check entry, are not the fix; the caret stays on the parameter or
            // the entry alone.
            StructuralMistake::LoopKeyWithoutVariable { .. }
            | StructuralMistake::GenericDefaultImpl { .. }
            | StructuralMistake::EntryParametersNotInKey { .. }
            | StructuralMistake::CheckAtUnitParameters { .. } => SpanPolicy::Primary,
            // rustc's labels and note name the generated check trait; the two tables are labelled
            // instead, the reported one as the second.
            StructuralMistake::DuplicateCheckTrait { .. } => SpanPolicy::TwoDefinitions {
                first: FIRST_CHECK_TABLE_LABEL,
                second: SECOND_CHECK_TABLE_LABEL,
            },
            // rustc's carets on the two sites stay.
            StructuralMistake::DuplicateCheck { .. }
            | StructuralMistake::SharedDispatchMethod { .. } => SpanPolicy::Keep,
        };
        Verdict::Reshape(Reshape::new(ClassPlan::structural_mistake(&mistake), span))
    }
}

/// What the generated items at a diagnostic's caret show.
enum Found {
    /// A structural mistake, reported once.
    Mistake(StructuralMistake),
    /// A second report of a mistake another stage words, dropped.
    Restated,
}

/// The structural mistake recovered from the generated items at `diag`'s caret: an `E0119` between
/// two identical check impls is a check table listing one entry twice (restating the wiring
/// conflict when `delegate_and_check_components!` wrote it), an `E0428` on a check trait
/// is two tables whose check traits share a name, an `E0277` on a check entry checked at the unit
/// parameters is an entry that names no parameters for a component that takes some, and an `E0207`
/// is a `for` loop whose key does not use its variable, a `#[default_impl]` on a generic provider
/// impl, or a wiring entry whose key does not use its generic parameters. `None` for any other
/// diagnostic.
fn structural_mistake(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Option<Found> {
    let primary_span = diag.span.primary_span()?;
    let mistake = match diag.code {
        Some(E0119) => {
            return Some(match detect::detect_duplicate_check(tcx?, primary_span)? {
                DuplicateCheck::Entry(mistake) => Found::Mistake(mistake),
                DuplicateCheck::WiredTwice => Found::Restated,
            });
        }
        // Name resolution reports a name defined twice, and the derived diagnostic carries the
        // name as an argument rather than in a plain-string message; its one secondary label is
        // the earlier definition.
        Some(E0428) => {
            let Some(DiagArgValue::Str(name)) = diag.args.get("name") else {
                return None;
            };
            let first = diag
                .span
                .span_labels()
                .into_iter()
                .find(|label| !label.is_primary)?
                .span;
            detect::detect_duplicate_check_trait(name, first, primary_span)
        }
        Some(E0277) => detect::detect_check_at_unit_parameters(tcx?, primary_span),
        Some(E0207) => {
            let tcx = tcx?;
            detect::detect_loop_key_without_variable(tcx, primary_span)
                .or_else(|| detect::detect_generic_default_impl(tcx, primary_span))
                .or_else(|| detect::detect_entry_parameters_not_in_key(tcx, primary_span))
        }
        _ => None,
    };
    mistake.map(Found::Mistake)
}
