//! The rustc-free model and wording for a wiring key or value that does not resolve.
//!
//! `delegate_components!` writes each entry `Key: Value` into an impl,
//! `impl DelegateComponent<Key> for Context { type Delegate = Value; }`, so a key or a value that
//! names nothing is a name the compiler cannot find there, and a consumer trait written as a key is
//! a trait where a type belongs. The compiler reports these during name resolution or type lowering
//! (`E0425` "cannot find type", `E0782` "expected a type, found a trait"), with suggestions that
//! follow the spelling rather than CGP: a similarly named trait, a `dyn`, a type parameter. A
//! `#[cgp_impl(Provider)]` written without `new` fails the same way, since nothing then declares
//! the provider struct.
//!
//! A `#[cgp_impl]` or `#[cgp_provider]` header names its component too, implicitly: the macro
//! derives the key from the provider trait's path as written, `Greeter` giving `GreeterComponent`,
//! so a key not imported, a module that re-exports the trait alone, or a trait name that is not the
//! component's fails there with the same `E0425`, or an `E0107` for a key with parameters.
//!
//! The driver recovers which of these mistakes a diagnostic is and the names that fix it, and
//! [`plan_unresolved_wiring_name`] words the coded header that replaces the message, with
//! [`unresolved_wiring_name_note`] saying why and [`unresolved_wiring_name_help`] carrying the fix.

use crate::code::{
    CGP_IMPL_PROVIDER_NOT_DECLARED, COMPONENT_NOT_IMPORTED, CONSUMER_TRAIT_AS_KEY,
    DERIVED_COMPONENT_NAME, KEY_NAMES_NO_COMPONENT, PATH_TYPE_NOT_DECLARED, PROVIDER_NOT_DECLARED,
};

/// A wiring key, a wiring value, a `#[cgp_impl]` provider, or the component a provider header
/// derives, that does not resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnresolvedWiringName {
    /// A key names the consumer trait `consumer` where its component `component` belongs.
    ConsumerTraitAsKey {
        /// The consumer trait the key names.
        consumer: String,
        /// The component the key should name.
        component: String,
    },
    /// A key, or the component a provider header derives, names the component `component`, which
    /// exists but is not in scope; `path` is the path a `use` imports it by.
    ComponentNotImported {
        /// The component the key names.
        component: String,
        /// The path a `use` imports it by.
        path: String,
    },
    /// A provider header names its provider trait through the module `module`, which re-exports the
    /// trait without its component `component`, so the component the header derives through the
    /// same module does not resolve; `trait_path` names the trait through the module that defines
    /// its component.
    ComponentNotInModule {
        /// The component the header derives.
        component: String,
        /// The module the header names the trait through.
        module: String,
        /// The trait's path through the module that defines its component.
        trait_path: String,
    },
    /// A provider header derives the component `derived` from the trait name `written`, but the
    /// component of the provider trait `provider_trait` is `component`, because the trait is
    /// imported under another name or the component's `name:` renames its key or gives it
    /// parameters; `attribute` is the header's attribute naming the component explicitly.
    DerivedComponentName {
        /// The component the header derives.
        derived: String,
        /// The trait name the header writes.
        written: String,
        /// The provider trait the name resolves to.
        provider_trait: String,
        /// The trait's component as declared, with its parameters.
        component: String,
        /// The header's attribute with the component named, such as
        /// `#[cgp_impl(new GreetHello: GreeterComponent)]`.
        attribute: String,
    },
    /// A key `key` names no component, and stripping its `Component` suffix names the consumer
    /// trait `consumer`, whose component is `component`.
    KeyNamesNoComponent {
        /// The key as written.
        key: String,
        /// The consumer trait the key names without its suffix.
        consumer: String,
        /// The component of that consumer trait.
        component: String,
    },
    /// A value names the provider `provider`, which is never declared, for the component `key`;
    /// `near_miss` is a provider of that component with a similar name, when one exists.
    ProviderNotDeclared {
        /// The provider the value names.
        provider: String,
        /// The component the value is wired for.
        key: String,
        /// A provider of that component with a similar name, when one exists.
        near_miss: Option<String>,
    },
    /// `#[cgp_impl(provider)]` names a provider struct that is never declared.
    CgpImplProviderNotDeclared {
        /// The provider the attribute names.
        provider: String,
    },
    /// A type `name` written in the `@`-path of a wiring key or value is never declared;
    /// `whole_segment` says the type is a segment of the path by itself, which a lowercase spelling
    /// would turn into a symbol, rather than a type nested in a segment's generic arguments.
    PathTypeNotDeclared {
        /// The type as written.
        name: String,
        /// Whether the type is a segment of the path by itself.
        whole_segment: bool,
    },
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
        UnresolvedWiringName::ComponentNotInModule {
            component, module, ..
        } => format!(
            "[{COMPONENT_NOT_IMPORTED}] the component `{component}` is not in module `{module}`"
        ),
        UnresolvedWiringName::DerivedComponentName {
            derived,
            written,
            provider_trait,
            component,
            ..
        } => format!(
            "[{DERIVED_COMPONENT_NAME}] the header derives the component `{derived}` from the \
             trait name `{written}`, but the component of `{provider_trait}` is `{component}`"
        ),
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
        UnresolvedWiringName::ComponentNotInModule {
            component,
            module,
            trait_path,
        } => Some(format!(
            "name the trait through the module that defines its component, `impl {trait_path}`, \
             or re-export `{component}` from `{module}` beside the trait"
        )),
        UnresolvedWiringName::DerivedComponentName {
            written,
            provider_trait,
            attribute,
            ..
        } if written != provider_trait => Some(format!(
            "name the component in the attribute, `{attribute}`, or import the trait under its \
             own name, `{provider_trait}`"
        )),
        UnresolvedWiringName::DerivedComponentName { attribute, .. } => Some(format!(
            "name the component in the attribute: `{attribute}`"
        )),
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

/// The `note` saying why the name does not resolve, for the provider-header mistakes whose cause
/// the header alone does not show, or `None`.
pub fn unresolved_wiring_name_note(name: &UnresolvedWiringName) -> Option<String> {
    match name {
        UnresolvedWiringName::ComponentNotInModule { module, .. } => Some(format!(
            "the header names its provider trait through `{module}`, so the macro names the \
             trait's component through that module too"
        )),
        UnresolvedWiringName::DerivedComponentName {
            written,
            provider_trait,
            component,
            ..
        } if written != provider_trait => Some(format!(
            "`{written}` imports `{provider_trait}` under another name, so an import of \
             `{component}` reads as unused while the header names `{written}`"
        )),
        _ => None,
    }
}
