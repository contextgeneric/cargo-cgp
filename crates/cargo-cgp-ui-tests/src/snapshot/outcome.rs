//! How reviewing one fixture came out.

/// The result of reviewing one fixture.
pub enum Outcome {
    /// Output matched the committed snapshot.
    Ok,
    /// Snapshot was (re)written from the current output.
    Blessed,
    /// Output differed from the committed snapshot, or a check across passes failed; the
    /// carried string is the rendered report, held so the fixture's block prints whole.
    Mismatch(String),
}
