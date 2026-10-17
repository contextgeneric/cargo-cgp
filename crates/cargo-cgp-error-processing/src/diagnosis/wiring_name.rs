//! The rustc-free model and wording for a wiring key or value that does not resolve.
//!
//! `delegate_components!` writes each entry `Key: Value` into an impl,
//! `impl DelegateComponent<Key> for Context { type Delegate = Value; }`, so a key or a value that
//! names nothing is a name the compiler cannot find there, and a consumer trait written as a key is
//! a trait where a type belongs. The compiler reports these during name resolution or type
//! lowering (`E0425` "cannot find type", `E0782` "expected a type, found a trait"), with suggestions
//! that follow the spelling rather than CGP: a similarly named trait, a `dyn`, a type parameter. A
//! `#[cgp_impl(Provider)]` written without `new` fails the same way, since nothing then declares the
//! provider struct.
//!
//! The driver recovers which of these mistakes a diagnostic is and the names that fix it, and
//! [`plan_unresolved_wiring_name`] words the coded header that replaces the message, with
//! [`unresolved_wiring_name_help`] carrying the fix.

use crate::code::{
    CGP_IMPL_PROVIDER_NOT_DECLARED, COMPONENT_NOT_IMPORTED, CONSUMER_TRAIT_AS_KEY,
    KEY_NAMES_NO_COMPONENT, PATH_TYPE_NOT_DECLARED, PROVIDER_NOT_DECLARED,
};

/// A wiring key, a wiring value, or a `#[cgp_impl]` provider that does not resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnresolvedWiringName {
    /// A key names the consumer trait `consumer` where its component `component` belongs.
    ConsumerTraitAsKey { consumer: String, component: String },
    /// A key names the component `component`, which exists but is not in scope; `path` is the path
    /// a `use` imports it by.
    ComponentNotImported { component: String, path: String },
    /// A key `key` names no component, and stripping its `Component` suffix names the consumer
    /// trait `consumer`, whose component is `component`.
    KeyNamesNoComponent {
        key: String,
        consumer: String,
        component: String,
    },
    /// A value names the provider `provider`, which is never declared, for the component `key`;
    /// `near_miss` is a provider of that component with a similar name, when one exists.
    ProviderNotDeclared {
        provider: String,
        key: String,
        near_miss: Option<String>,
    },
    /// `#[cgp_impl(provider)]` names a provider struct that is never declared.
    CgpImplProviderNotDeclared { provider: String },
    /// A type `name` written in the `@`-path of a wiring key or value is never declared;
    /// `whole_segment` says the type is a segment of the path by itself, which a lowercase spelling
    /// would turn into a symbol, rather than a type nested in a segment's generic arguments.
    PathTypeNotDeclared { name: String, whole_segment: bool },
}

/// The coded header that replaces the compiler's message for `name`.
pub fn plan_unresolved_wiring_name(name: &UnresolvedWiringName) -> String {
    match name {
        UnresolvedWiringName::ConsumerTraitAsKey { consumer, .. } => format!(
            "[{CONSUMER_TRAIT_AS_KEY}] the wiring key `{consumer}` is a consumer trait, but a key \
             names a component",
        ),
        UnresolvedWiringName::ComponentNotImported { component, .. } => {
            format!("[{COMPONENT_NOT_IMPORTED}] the component `{component}` is not in scope")
        }
        UnresolvedWiringName::KeyNamesNoComponent { key, .. } => {
            format!("[{KEY_NAMES_NO_COMPONENT}] the wiring key `{key}` names no component")
        }
        UnresolvedWiringName::ProviderNotDeclared { provider, key, .. } => format!(
            "[{PROVIDER_NOT_DECLARED}] the provider `{provider}` wired for `{key}` is not declared",
        ),
        UnresolvedWiringName::CgpImplProviderNotDeclared { provider } => {
            format!("[{CGP_IMPL_PROVIDER_NOT_DECLARED}] the provider `{provider}` is not declared",)
        }
        UnresolvedWiringName::PathTypeNotDeclared { name, .. } => {
            format!("[{PATH_TYPE_NOT_DECLARED}] the type `{name}` in a wiring path is not declared")
        }
    }
}

/// The `help` naming the fix, or `None` when the tool has no fix to offer (an undeclared provider
/// with no near miss, a type nested in a segment of a path).
pub fn unresolved_wiring_name_help(name: &UnresolvedWiringName) -> Option<String> {
    match name {
        UnresolvedWiringName::ConsumerTraitAsKey { component, .. } => {
            Some(format!("name its component instead: `{component}`"))
        }
        UnresolvedWiringName::ComponentNotImported { path, .. } => {
            Some(format!("import it with `use {path};`"))
        }
        UnresolvedWiringName::KeyNamesNoComponent {
            consumer,
            component,
            ..
        } => Some(format!(
            "name the component of `{consumer}` instead: `{component}`"
        )),
        UnresolvedWiringName::ProviderNotDeclared { key, near_miss, .. } => {
            near_miss.as_ref().map(|near_miss| {
                format!("a provider of `{key}` with a similar name exists: `{near_miss}`")
            })
        }
        UnresolvedWiringName::CgpImplProviderNotDeclared { provider } => {
            Some(format!("declare it with `#[cgp_impl(new {provider})]`"))
        }
        UnresolvedWiringName::PathTypeNotDeclared {
            name,
            whole_segment: true,
            ..
        } => Some(format!(
            "declare or import the type `{name}`, or write the segment in lowercase to make it a \
             symbol"
        )),
        UnresolvedWiringName::PathTypeNotDeclared { .. } => None,
    }
}
