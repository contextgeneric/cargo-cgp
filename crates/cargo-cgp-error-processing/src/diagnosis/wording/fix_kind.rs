//! The kinds of fix a `help` names.

/// The kinds of fix a `help` names. The fixes of one kind on one owner merge into one `help`.
#[derive(PartialEq, Eq)]
pub enum FixKind {
    /// Add the missing fields to a struct.
    AddFields,
    /// Derive `HasField` on the struct that carries the fields.
    Derive,
    /// Wire components or paths on a context.
    Wire,
    /// Add entries to a provider's table.
    AddEntries,
    /// Add data derives (`HasFields`, `ExtractField`, …) to a struct or enum.
    DeriveData,
    /// A `help` worded whole, such as a near miss or an abstract type to reconcile (its owner is
    /// the text itself).
    Whole,
}
