//! The rustc-free model and wording for an inner provider a higher-order provider never imported.

use crate::diagnosis::wording::quoted_list;

/// An inner provider a higher-order provider's body calls (`inner::method(self)`) without importing
/// it, so the parameter carries no provider-trait bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingUseProvider {
    /// The inner provider parameter's name.
    pub inner: String,
    /// The provider trait to import it as, as `#[use_provider]` writes it: with the component's
    /// parameters after the context, which the attribute supplies (`AreaCalculator<Shape>`), each
    /// one the call does not give away written as the trait's own parameter name. `None` when the
    /// tool cannot write the bound, as for a trait with a lifetime or const parameter.
    pub provider_trait: Option<String>,
    /// The provider trait's parameters the call does not give away, named in `provider_trait` for
    /// the reader to replace.
    pub unresolved: Vec<String>,
}

/// Word a [`MissingUseProvider`] into the `[CGP-E031]` header that replaces the raw `E0599`, naming
/// the inner provider rather than the generated `__Context__` scaffolding rustc's suggestion leaks.
pub fn plan_missing_use_provider(missing: &MissingUseProvider) -> String {
    format!(
        "[{}] the inner provider `{}` is used but not imported",
        crate::code::INNER_PROVIDER_NOT_IMPORTED,
        missing.inner,
    )
}

/// The `help` accompanying the header, the fix the raw error never states: import the inner
/// provider with `#[use_provider]`, which supplies the leading context argument the provider trait
/// needs. `None` when the bound cannot be written.
pub fn missing_use_provider_help(missing: &MissingUseProvider) -> Option<String> {
    let provider_trait = missing.provider_trait.as_ref()?;
    let import = format!(
        "import it with `#[use_provider({}: {provider_trait})]`",
        missing.inner,
    );
    Some(match missing.unresolved.len() {
        0 => import,
        1 => format!(
            "{import}, with the type the call uses in place of {}",
            quoted_list(&missing.unresolved),
        ),
        _ => format!(
            "{import}, with the types the call uses in place of {}",
            quoted_list(&missing.unresolved),
        ),
    })
}
