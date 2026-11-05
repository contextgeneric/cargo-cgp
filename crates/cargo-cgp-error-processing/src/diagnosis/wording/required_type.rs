//! The required type both mismatch cause sentences render.

/// Render a required type: the un-normalized form, followed by what it reduces to in parentheses
/// when that differs. Both mismatch cause sentences render their required type through it.
///
/// The un-normalized form leads because it names *where* the requirement comes from: a provider
/// reading `&Pool<Db>` under an imported abstract type requires
/// `Pool<<App as HasDbType>::Db>`, which points at the wiring, while the reader compares the reduced
/// form against the field, so neither alone is enough.
pub fn required_type(expected: &str, expected_normalized: Option<&str>) -> String {
    match expected_normalized {
        Some(normalized) => format!("`{expected}` (`{normalized}`)"),
        None => format!("`{expected}`"),
    }
}
