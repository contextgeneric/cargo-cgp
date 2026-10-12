//! The names `#[cgp_auto_dispatch]` gives the items it generates for a method.
//!
//! The macro names a method's helper function `__compute_{method}__` and its computer provider
//! `Compute{Method}`, after the method alone, so two dispatch traits in one module that share a
//! method name define both items twice. The rules mirror the macro's own (`cgp-macro-extra-core`'s
//! `derive_computer_ident` and the helper name in `cgp_auto_dispatch/method.rs`), so the driver can
//! pair the two `E0428`s one shared method produces.

/// The method a `#[cgp_auto_dispatch]` helper function is named after: `area` for
/// `__compute_area__`. `None` for any other name.
pub fn dispatch_helper_method(name: &str) -> Option<&str> {
    name.strip_prefix("__compute_")?
        .strip_suffix("__")
        .filter(|method| !method.is_empty())
}

/// The name of the computer `#[cgp_auto_dispatch]` generates for `method`: `Compute` and the method
/// name in PascalCase, each `_`-separated word capitalized, so `scaled_area` gives
/// `ComputeScaledArea`.
pub fn dispatch_computer_name(method: &str) -> String {
    let pascal: String = method
        .split('_')
        .filter(|word| !word.is_empty())
        .flat_map(|word| {
            word.chars().enumerate().map(|(index, c)| {
                if index == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
        })
        .collect();
    format!("Compute{pascal}")
}
