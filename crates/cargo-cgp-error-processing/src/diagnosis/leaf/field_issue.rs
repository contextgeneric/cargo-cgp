//! Why a required `HasField` bound is unmet.

/// Why a required `HasField` bound is unmet — the distinction that tells a genuinely missing
/// field apart from one some struct actually carries but has not derived. CGP's `HasField`
/// follows `Deref` (a blanket impl forwards to the target), so a field on a `Deref` target
/// resolves when the target derives it; the failure diagnosed here is a field present on some
/// struct that has no `HasField` impl for it. (A field present *with a mismatched type* keeps
/// its `HasField` trait impl and fails only the associated-type projection, so it never reaches
/// this classification — it is a [`Leaf::FieldTypeMismatch`](crate::diagnosis::Leaf::FieldTypeMismatch) instead.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldIssue {
    /// No struct in the context's `Deref` chain carries a field of this name: it is genuinely
    /// missing and must be added.
    Missing,
    /// The context struct itself carries a field of this name, yet the `HasField` bound is unmet:
    /// the struct is missing (or has an incomplete) `#[derive(HasField)]` for it.
    Present,
    /// The context does not carry the field directly, but a struct reached through its `Deref`
    /// chain does. Since `HasField` follows `Deref`, the bound would hold if that target derived
    /// the field; the fault is that the target does not derive `HasField`.
    PresentViaDeref {
        /// The `Deref`-reachable struct that carries the field, e.g. `AppFields`.
        target: String,
    },
}
