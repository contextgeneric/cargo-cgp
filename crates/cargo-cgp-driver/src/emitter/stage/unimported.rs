//! The stages for an abstract type used without `#[use_type]`, and for a requirement a provider
//! does not declare, which share the record of the types already reported.

use cargo_cgp_error_processing::{
    ClassPlan, UndeclaredRequirement, UnimportedAbstractType, unresolved_type_name,
};
use rustc_errors::codes::{E0220, E0277, E0425};
use rustc_errors::{DiagArgValue, DiagInner};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::config::CGP_IMPL_CONTEXT_PARAM;
use crate::detect::{self, DetectedUndeclaredRequirement};
use crate::emitter::buffer::BufEntry;
use crate::emitter::edit::main_message_text;
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict, apply_reshape};

/// An abstract type reported as used but not imported: the trait declaring it, the generated item
/// the use sits in, and the use's caret. A later report of the same type is the same mistake when
/// it shares the item (a second use) or the caret (one signature the macro copied into two items).
pub struct UnimportedReport {
    pub trait_did: DefId,
    pub item: DefId,
    pub span: Span,
}

/// The unimported abstract type's and the undeclared requirement's stages, with their memory: the
/// abstract types reported as used but not imported, and the provider impls reported for a
/// requirement they do not declare.
#[derive(Default)]
pub struct UnimportedStage {
    pub reported: Vec<UnimportedReport>,
    pub requirement_impls: Vec<DefId>,
}

impl UnimportedStage {
    /// Reword the `E0220` an abstract type used without its `#[use_type(…)]` import produces on the
    /// generated context, naming the type and carrying the import, once per mistake. The type and
    /// its context are read from the diagnostic's arguments, since its main message is a Fluent
    /// one, and [`detect::detect_unimported_abstract_type`] confirms it.
    pub fn unimported_abstract_type(
        &mut self,
        tcx: Option<TyCtxt<'_>>,
        diag: &DiagInner,
    ) -> Verdict {
        if diag.code != Some(E0220) {
            return Verdict::Continue;
        }
        let arg = |name: &str| match diag.args.get(name) {
            Some(DiagArgValue::Str(value)) => Some(value.to_string()),
            _ => None,
        };
        if arg("assoc_kind").as_deref() != Some("type") {
            return Verdict::Continue;
        }
        let (Some(type_name), Some(qself), Some(tcx), Some(primary_span)) = (
            arg("assoc_ident"),
            arg("qself"),
            tcx,
            diag.span.primary_span(),
        ) else {
            return Verdict::Continue;
        };
        let Some(detected) =
            detect::detect_unimported_abstract_type(tcx, primary_span, &type_name, &qself)
        else {
            return Verdict::Continue;
        };
        let repeated = self.reported.iter().any(|report| {
            report.trait_did == detected.trait_did
                && (report.item == detected.item || report.span == primary_span)
        });
        if repeated {
            return Verdict::Drop;
        }
        self.reported.push(UnimportedReport {
            trait_did: detected.trait_did,
            item: detected.item,
            span: primary_span,
        });
        Verdict::Reshape(Reshape::new(
            ClassPlan::unimported_abstract_type(&detected.unimported),
            SpanPolicy::Primary,
        ))
    }

    /// Reword the first `E0277` on the generated `__Context__` that a `#[cgp_impl]` provider gets
    /// for each requirement its provider trait places on the context and the provider does not
    /// declare, naming the provider trait, the provider, and each declaration, and drop the rest.
    /// An abstract type among them that the provider uses without importing is reported as that use
    /// instead: through a report [`unimported_abstract_type`](Self::unimported_abstract_type)
    /// already made, or by converting the name-resolution `E0425` of a bare use in `buffer`. The
    /// caret moves to the macro's call site, without rustc's note that the error originates in the
    /// macro.
    pub fn requirement(
        &mut self,
        detected: DetectedUndeclaredRequirement,
        buffer: &mut Vec<BufEntry>,
    ) -> Verdict {
        if self.requirement_impls.contains(&detected.impl_did) {
            return Verdict::Drop;
        }
        self.requirement_impls.push(detected.impl_did);
        let DetectedUndeclaredRequirement {
            requirement,
            impl_did,
            impl_span,
            traits,
        } = detected;
        let mut kept = Vec::new();
        for (required, (trait_did, type_name)) in requirement.requirements.into_iter().zip(traits) {
            let explained = type_name.is_some_and(|type_name| {
                self.reported
                    .iter()
                    .any(|report| report.trait_did == trait_did && report.item == impl_did)
                    || self.convert_unresolved_abstract_type(
                        buffer,
                        &type_name,
                        trait_did,
                        &required.bound,
                        impl_did,
                        impl_span,
                    )
            });
            if !explained {
                kept.push(required);
            }
        }
        if kept.is_empty() {
            return Verdict::Drop;
        }
        let requirement = UndeclaredRequirement {
            requirements: kept,
            ..requirement
        };
        Verdict::Reshape(Reshape::new(
            ClassPlan::undeclared_requirement(&requirement),
            SpanPolicy::PrimaryCallSite,
        ))
    }

    /// Turn the buffered `E0425` "cannot find type `name`" inside the provider impl at `impl_span`
    /// into the `[CGP-E035]` report of the abstract type it names, which `trait_name` declares, and
    /// drop the later ones for the same name in the impl: name resolution reports a bare `Error`
    /// before the compiler can say which trait declares it, so the requirement that names the trait
    /// is where the use is recognized. Returns whether one was found.
    fn convert_unresolved_abstract_type(
        &mut self,
        buffer: &mut Vec<BufEntry>,
        name: &str,
        trait_did: DefId,
        trait_name: &str,
        impl_did: DefId,
        impl_span: Span,
    ) -> bool {
        let is_use = |diag: &DiagInner| {
            diag.code == Some(E0425)
                && main_message_text(diag).and_then(unresolved_type_name) == Some(name)
                && diag
                    .span
                    .primary_span()
                    .is_some_and(|span| impl_span.contains(span))
        };
        let unimported = UnimportedAbstractType {
            type_name: name.to_owned(),
            trait_name: trait_name.to_owned(),
        };
        let mut reported = None;
        buffer.retain_mut(|entry| {
            let BufEntry::Plain(diag) = entry else {
                return true;
            };
            if !is_use(diag) {
                return true;
            }
            if reported.is_some() {
                return false;
            }
            let primary_span = diag.span.primary_span();
            apply_reshape(
                diag,
                ClassPlan::unimported_abstract_type(&unimported),
                SpanPolicy::Primary,
            );
            reported = primary_span;
            true
        });
        let Some(span) = reported else {
            return false;
        };
        self.reported.push(UnimportedReport {
            trait_did,
            item: impl_did,
            span,
        });
        true
    }
}

/// Recognize `diag` as a requirement a provider trait places on the context that a `#[cgp_impl]`
/// provider does not declare: an `E0277` on the generated `__Context__`, confirmed by
/// [`detect::detect_undeclared_requirement`]. This is
/// [`requirement`](UnimportedStage::requirement)'s query, run before the buffer is read.
pub fn undeclared_requirement(
    tcx: Option<TyCtxt<'_>>,
    diag: &DiagInner,
) -> Option<DetectedUndeclaredRequirement> {
    if diag.code != Some(E0277)
        || !main_message_text(diag)
            .is_some_and(|text| text.contains(&format!("`{CGP_IMPL_CONTEXT_PARAM}: ")))
    {
        return None;
    }
    detect::detect_undeclared_requirement(tcx?, diag.span.primary_span()?)
}
