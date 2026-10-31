//! The diagnostic kinds a resolved failure plans differently.

/// The `rustc` error-code discriminant [`plan_resolved`](crate::diagnosis::plan_resolved) needs, in rustc-free form so the plan
/// is decided without linking the compiler. The emitter maps a diagnostic's `rustc` code to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagKind {
    /// A check-trait or ordinary-bound failure — rustc's `E0277`, the default.
    Check,
    /// An associated-type mismatch — rustc's `E0271`, traced to a failing projection: a `HasField`
    /// value type (the `[CGP-E004]` field form) or any other associated type, most often a CGP
    /// abstract type (the `[CGP-E005]` form).
    TypeMismatch,
    /// A consumer-method call failure recovered at the use site — rustc's `E0599`, or an `E0277`
    /// whose obligation the resolver re-read from the call expression itself.
    MethodNotFound,
}
