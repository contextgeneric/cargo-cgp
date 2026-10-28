//! The stage for a `#[cgp_impl]` header that names the wrong trait.
//!
//! Naming the component's consumer trait, or a non-CGP trait, where the provider trait belongs
//! makes the macro generate an inside-out impl of the wrong trait, which produces a burst of
//! macro-lowering errors (`E0425`, `E0107`, `E0186`, `E0207`, and an `E0308` cascade) plus a
//! downstream check failure, none naming the mistake. The stage recognizes the mistake once from
//! the compiler and reshapes the `E0107` whose caret is on the misused trait name, dropping the rest
//! of the burst.

use cargo_cgp_error_processing::{CgpImplMisuse, ClassPlan, Leaf, Resolved};
use rustc_errors::codes::{E0107, E0186, E0207, E0308, E0425};
use rustc_errors::{DiagInner, DiagMessage, Style};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::emitter::buffer::BufEntry;
use crate::emitter::pipeline::{BufferEdit, Reshape, SpanPolicy, Verdict};
use crate::resolve::{self, DetectedCgpImplMisuse};

/// Stage 2, with its memory: the crate's `#[cgp_impl]` header mistakes, detected once.
#[derive(Default)]
pub struct CgpImplMisuseStage {
    /// The mistakes found, or `None` until a `TyCtxt` was in scope to look. The macro-lowering
    /// errors that trigger detection can arrive before trait solving puts one there, so detection
    /// is retried on a later diagnostic; an empty list means none was found. Owned names and `Copy`
    /// spans, so the memo outlives the `TyCtxt`.
    pub misuses: Option<Vec<DetectedCgpImplMisuse>>,
}

impl CgpImplMisuseStage {
    /// Reshape the `E0107` on a misused trait name, drop a sibling error of the same mistake, or
    /// leave anything else to the next stage.
    pub fn run(&mut self, tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
        if !matches!(
            diag.code,
            Some(E0107) | Some(E0425) | Some(E0186) | Some(E0207) | Some(E0308)
        ) {
            return Verdict::Continue;
        }
        // Detection forces HIR and trait-graph queries, which re-enter the `DiagCtxt` lock and
        // abort the compiler if run while an early-phase diagnostic is emitting (name resolution,
        // where the `E0425` lands, is mid-`hir_owner`). So only the `E0107`, a type-lowering-phase
        // error always present for this mistake, triggers it. The siblings that precede it are
        // purged from the buffer when it is reshaped; those that follow are dropped here.
        if diag.code == Some(E0107) && self.misuses.is_none() {
            self.misuses = tcx.map(resolve::detect_cgp_impl_misuses);
        }
        self.verdict(diag)
    }

    /// The verdict for `diag` against the mistakes already detected.
    fn verdict(&self, diag: &DiagInner) -> Verdict {
        let Some(misuses) = self.misuses.as_deref() else {
            return Verdict::Continue;
        };
        let Some(primary) = diag.span.primary_span() else {
            return Verdict::Continue;
        };
        if diag.code == Some(E0107)
            && let Some(misuse) = misuses
                .iter()
                .find(|misuse| misuse.trait_ref_span.overlaps(primary))
        {
            // Keep rustc's caret, already on the misused trait name, without its "expected N
            // generic arguments" label, which no longer fits the reshaped message.
            return Verdict::Reshape(Reshape {
                buffer_edit: BufferEdit::DropCgpImplSiblings {
                    impl_span: misuse.impl_span,
                    macro_span: misuse.macro_span,
                },
                ..Reshape::new(
                    ClassPlan::cgp_impl_misuse(&misuse.misuse),
                    SpanPolicy::Primary,
                )
            });
        }
        if misuses
            .iter()
            .any(|misuse| is_cgp_impl_sibling(diag, misuse.impl_span, misuse.macro_span))
        {
            return Verdict::Drop;
        }
        Verdict::Continue
    }

    /// Whether `resolved` is the downstream check re-report of a detected consumer-trait mistake:
    /// every cause a `NotAProvider` leaf naming that mistake's provider struct and the provider
    /// trait its header should have targeted. Such a failure is a consequence of the `[CGP-E032]`
    /// already shown. A requirement of that impl which fails on its own, such as a field the
    /// context lacks, resolves to its own cause through the impl as written, so a report carrying
    /// it is kept.
    pub fn is_check_report(&self, resolved: &Resolved) -> bool {
        let Some(misuses) = self.misuses.as_deref() else {
            return false;
        };
        if misuses.is_empty() || resolved.causes.is_empty() {
            return false;
        }
        resolved.causes.iter().all(|cause| {
            matches!(
                &cause.leaf,
                Leaf::NotAProvider { provider, provider_trait, .. }
                    if misuses.iter().any(|misuse| matches!(
                        &misuse.misuse,
                        CgpImplMisuse::ConsumerTrait { provider: expected, .. }
                            if provider == &misuse.self_ty && provider_trait == expected
                    ))
            )
        })
    }
}

/// Whether `diag` is a sibling macro-lowering error of a `#[cgp_impl]` mistake, landing in the impl
/// body (`impl_span`) or at the macro call site the synthesized tokens share (`macro_span`, where
/// the `E0425` and `E0207` sit). `E0425`, `E0186`, and `E0207` are always siblings when they land
/// there; an `E0308` mismatch cascade (from a malformed inner-provider bound) is one only when it
/// mentions the generated `__Context__`, so a user's own type error is never dropped.
pub fn is_cgp_impl_sibling(diag: &DiagInner, impl_span: Span, macro_span: Span) -> bool {
    let Some(primary) = diag.span.primary_span() else {
        return false;
    };
    if !(impl_span.overlaps(primary) || macro_span.overlaps(primary)) {
        return false;
    }
    match diag.code {
        Some(E0425) | Some(E0186) | Some(E0207) => true,
        Some(E0308) => mentions_generated_context(diag),
        _ => false,
    }
}

/// Whether a buffered entry is a sibling of the impl at `(impl_span, macro_span)`, the shape purged
/// when the reshaped `E0107` is produced, since a sibling (notably the name-resolution `E0425`) can
/// arrive before it.
pub fn is_cgp_impl_sibling_entry(entry: &BufEntry, impl_span: Span, macro_span: Span) -> bool {
    matches!(entry, BufEntry::Plain(diag) if is_cgp_impl_sibling(diag, impl_span, macro_span))
}

/// Whether any message of `diag` or its children names the generated `__Context__` parameter, the
/// tell that an `E0308` mismatch is a macro-lowering cascade rather than a user's own type error.
fn mentions_generated_context(diag: &DiagInner) -> bool {
    let in_messages = |messages: &[(DiagMessage, Style)]| {
        messages.iter().any(|(message, _)| {
            matches!(message, DiagMessage::Str(text) if text.contains("__Context__"))
        })
    };
    in_messages(&diag.messages)
        || diag
            .children
            .iter()
            .any(|child| in_messages(&child.messages))
}
