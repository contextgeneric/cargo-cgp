//! The stages for a provider-trait impl written without `#[cgp_provider]`, and the failure it
//! leaves behind.

use cargo_cgp_error_processing::{ClassPlan, ProviderImplWithoutMacro};
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
        match detect_mistake(tcx, diag) {
            Some(mistake) => {
                self.remember(&mistake.provider);
                Verdict::Reshape(Reshape::new(
                    ClassPlan::provider_impl_without_macro(&mistake),
                    SpanPolicy::Primary,
                ))
            }
            None => Verdict::Continue,
        }
    }

    /// Remember the provider of `diag` when it is a provider-trait impl written without
    /// `#[cgp_provider]`, before any diagnostic is transformed: the collect pass over the queue of
    /// `analysis`, so [`consequence`](Self::consequence) drops a check that restates the provider's
    /// mistake even when the check arrived first.
    pub fn collect(&mut self, tcx: Option<TyCtxt<'_>>, diag: &DiagInner) {
        if let Some(mistake) = detect_mistake(tcx, diag) {
            self.remember(&mistake.provider);
        }
    }

    /// Record `provider` as reported, once.
    fn remember(&mut self, provider: &str) {
        if !self.providers.iter().any(|known| known == provider) {
            self.providers.push(provider.to_owned());
        }
    }

    /// Drop a check or call that fails on the `IsProviderFor` bound of a provider reported or
    /// collected, since it restates that one mistake.
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

/// The provider-trait impl written without `#[cgp_provider]` that `diag` reports: an `E0277` whose
/// message names `IsProviderFor`, confirmed structurally by
/// [`detect::detect_provider_impl_without_macro`].
fn detect_mistake(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Option<ProviderImplWithoutMacro> {
    if diag.code != Some(E0277)
        || !main_message_text(diag).is_some_and(|text| text.contains(IS_PROVIDER_FOR_TRAIT))
    {
        return None;
    }
    detect::detect_provider_impl_without_macro(tcx?, diag.span.primary_span()?)
}
