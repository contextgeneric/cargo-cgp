//! What a stage decides about a diagnostic, as a value.

use cargo_cgp_error_processing::ClassPlan;
use rustc_span::Span;

/// A stage's decision about one diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Not this stage's diagnostic: the next stage reads it.
    Continue,
    /// A consequence of a failure already reported: nothing is buffered.
    Drop,
    /// This stage's class: the emitter writes the reshape and buffers the result.
    Reshape(Reshape),
}

impl Verdict {
    /// This verdict, or the one `next` gives when this one leaves the diagnostic to the next stage.
    pub fn or_else(self, next: impl FnOnce() -> Verdict) -> Verdict {
        match self {
            Verdict::Continue => next(),
            decided => decided,
        }
    }
}

/// Every edit a reshaping stage makes, as data: the class's wording, which of rustc's spans stay,
/// whether the result's spans anchor a later `?` cascade, and the edit of the diagnostics already
/// buffered. Replacing the header always drops rustc's own notes and suggestions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reshape {
    pub plan: ClassPlan,
    pub span: SpanPolicy,
    pub records_spans: bool,
    pub buffer_edit: BufferEdit,
}

impl Reshape {
    /// The reshape most stages make: `plan` over `span`, recording its spans and leaving the buffer
    /// as it is.
    pub fn new(plan: ClassPlan, span: SpanPolicy) -> Self {
        Self {
            plan,
            span,
            records_spans: true,
            buffer_edit: BufferEdit::None,
        }
    }
}

/// Which of rustc's spans a reshaped diagnostic keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanPolicy {
    /// rustc's spans and labels as they are, such as a conflict's two carets.
    Keep,
    /// The primary span alone, without its label, when the diagnostic has one.
    Primary,
    /// The call site of the primary span, when the diagnostic has one, so a caret inside a macro's
    /// expansion moves to the macro's invocation.
    PrimaryCallSite,
    /// The two definitions of a generated name defined twice: the primary span labelled `second`
    /// and the span rustc labels as the earlier definition labelled `first`.
    TwoDefinitions {
        first: &'static str,
        second: &'static str,
    },
}

/// An edit of the diagnostics already buffered, which a reshape makes beside its own diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferEdit {
    /// The buffer stays as it is.
    None,
    /// Drop the buffered siblings of a `#[cgp_impl]` header mistake: the macro-lowering errors in
    /// the impl at `impl_span` or at the macro call site `macro_span`, which can arrive before the
    /// error the mistake is recognized from.
    DropCgpImplSiblings { impl_span: Span, macro_span: Span },
    /// Remove the buffered entry at this index, the partner the reshape words together with its
    /// own diagnostic.
    Remove(usize),
}
