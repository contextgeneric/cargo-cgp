//! The stages for a provider-trait impl written without `#[cgp_provider]`, and the failure it
//! leaves behind.

use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::DiagInner;
use rustc_errors::codes::E0277;
use rustc_middle::ty::TyCtxt;

use crate::config::IS_PROVIDER_FOR_TRAIT;
use crate::detect;
use crate::emitter::edit::main_message_text;
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};

/// The macro-less provider's stages, with their memory: the provider structs reported as written
/// without `#[cgp_provider]`, so a later failure on one is that report's consequence.
#[derive(Default)]
pub struct MacrolessProviderStage {
    /// The provider structs reported, by name.
    pub providers: Vec<String>,
}

impl MacrolessProviderStage {
    /// Reword the `E0277` a provider-trait impl without `#[cgp_provider]` produces. rustc rejects
    /// it on the provider trait's `IsProviderFor` supertrait and lists every unrelated implementor
    /// beside it; the reshape names the provider, carries the macro in a help, and keeps the caret
    /// on the impl. Gated to an `E0277` whose message names `IsProviderFor`, confirmed structurally
    /// by [`detect::detect_provider_impl_without_macro`].
    pub fn run(&mut self, tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
        if diag.code != Some(E0277)
            || !main_message_text(diag).is_some_and(|text| text.contains(IS_PROVIDER_FOR_TRAIT))
        {
            return Verdict::Continue;
        }
        let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
            return Verdict::Continue;
        };
        match detect::detect_provider_impl_without_macro(tcx, primary_span) {
            Some(mistake) => {
                self.providers.push(mistake.provider.clone());
                Verdict::Reshape(Reshape::new(
                    ClassPlan::provider_impl_without_macro(&mistake),
                    SpanPolicy::Primary,
                ))
            }
            None => Verdict::Continue,
        }
    }

    /// Drop a check or call that fails on the `IsProviderFor` bound of a provider already reported,
    /// since it restates that one mistake.
    pub fn consequence(&self, diag: &DiagInner) -> Verdict {
        let restates = diag.code == Some(E0277)
            && main_message_text(diag).is_some_and(|text| {
                self.providers.iter().any(|provider| {
                    text.contains(&format!("`{provider}: {IS_PROVIDER_FOR_TRAIT}<"))
                })
            });
        if restates {
            Verdict::Drop
        } else {
            Verdict::Continue
        }
    }
}
