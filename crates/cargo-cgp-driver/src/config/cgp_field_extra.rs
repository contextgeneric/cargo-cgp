//! The `cgp-field-extra` crate and the optional-builder items it defines.

/// The crate that defines CGP's extra extensible-data operations: the optional and defaulted
/// builders (`SetOptional`, `CanBuildWithDefault`, `CanFinalizeWithDefault`, `HasOptionalBuilder`).
/// Its traits, like those of [`CGP_FIELD_CRATE`](super::CGP_FIELD_CRATE), are extensible-data
/// traits the walk descends.
pub const CGP_FIELD_EXTRA_CRATE: &str = "cgp_field_extra";

/// The conversion (defined by [`CGP_FIELD_EXTRA_CRATE`]) that turns a record's core builder into an
/// optional one, whose fields `set` and `set_optional` update.
pub const TO_OPTIONAL_TRAIT: &str = "ToOptional";
