//! The facts a leaf carries for its `help`s alone, which are not part of its identity.

/// What a missing field's `help`s can name. It rides on a [`Leaf::Field`](crate::diagnosis::Leaf::Field) but is not part of the
/// leaf's identity: every equality and hash treats two values alike, so one missing field reached
/// through requirements that state its type differently stays one cause, and
/// [`Leaf::reconcile_fix`](crate::diagnosis::Leaf::reconcile_fix) keeps only what every route agrees on.
#[derive(Debug, Clone, Default)]
pub struct FieldFix {
    /// Whether adding the field is confirmed to fix the cause: its struct implements `HasField` for
    /// another field, so it derives it. A struct with no `HasField` impl may lack the derive too.
    pub confirmed: bool,
    /// The type the requirement gives the field (the `f64` of
    /// `HasField<Symbol!("height"), Value = f64>`), or `None` when no requirement states a concrete
    /// one.
    pub ty: Option<String>,
    /// A field the struct carries whose name is a likely slip of the missing one
    /// ([`similar_name`](crate::diagnosis::similar_name)), such as `first_name` for `frist_name`.
    pub similar: Option<String>,
    /// The `Index<N>` tag of the tuple field the missing field's name is the position of, when the
    /// struct is a tuple struct and the tag was written as `Symbol!("N")`.
    pub index_tag: Option<String>,
}

/// A key the context wires that is a likely slip of a missing one
/// ([`similar_key`](crate::diagnosis::similar_key)), such as `@App.GreeterComponent` for
/// `@app.GreeterComponent`. Like [`FieldFix`], it is carried for a `help` and is not part of the
/// leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct NearMiss(pub Option<String>);

/// The adapter types a `help` names for a [`Leaf::NotAProvider`](crate::diagnosis::Leaf::NotAProvider): each wraps the wired type, and the
/// solver confirmed the result is a provider of the slot's trait (`WithProvider<ReadName>` for a
/// hand-written getter). Like [`NearMiss`], it is carried for the `help` and is not part of the
/// leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct AdapterFix(pub Vec<Adapter>);

/// An adapter type applied to a wired type, with the `use` path its name needs when the prelude does
/// not export it (`Promote` comes from `cgp::extra::handler`, `WithProvider` from the prelude).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adapter {
    /// The adapter applied to the wired type, e.g. `Promote<Double>`.
    pub ty: String,
    /// The path a `use` imports the adapter by, or `None` when the prelude exports it or no public
    /// path is found.
    pub import: Option<String>,
}

/// The other components a [`Leaf::NotAProvider`](crate::diagnosis::Leaf::NotAProvider)'s wired type is a provider of, each confirmed by
/// the solver for the same context: the likely mix-up of a provider wired under the wrong key. Like
/// [`NearMiss`], it is carried for the `help` and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct ProvidesFix(pub Vec<ProvidedComponent>);

/// A component a wired type provides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidedComponent {
    /// The component's provider trait, e.g. `Farewell`.
    pub provider_trait: String,
    /// The component's marker, the key to wire it under, e.g. `FarewellComponent`.
    pub component: String,
}

/// The fix a `help` names for a [`Leaf::NotAProvider`](crate::diagnosis::Leaf::NotAProvider) that is a misused CGP core provider. Like
/// [`NearMiss`], it is carried for the `help` and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct CoreProviderFix(pub Option<CoreProviderHint>);

/// What serves a slot a CGP core provider cannot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreProviderHint {
    /// `UseField` is wired for a multi-method getter, which has no `UseField` impl; `UseFields`
    /// serves it.
    UseFields,
    /// `UseDelegate` is wired for a component that does not declare `#[derive_delegate]`; an `open`
    /// statement dispatches on the component's parameters without it. `component` is the key to
    /// open, and `params` the names of the parameters the entries' paths end in.
    OpenDispatch {
        /// The component marker to open.
        component: String,
        /// The names of the component's parameters, one path segment each.
        params: Vec<String>,
    },
}

/// The derive a `help` names for a [`Leaf::DataTraitUnimplemented`](crate::diagnosis::Leaf::DataTraitUnimplemented), when one can be added to the
/// type. Like [`NearMiss`], it is carried for the `help` and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct DeriveFix(pub Option<String>);

/// The provider a `help` names for a [`Leaf::WiringWithoutProviderFor`](crate::diagnosis::Leaf::WiringWithoutProviderFor), to write the entry that
/// replaces a hand-written `DelegateComponent` impl. Like [`NearMiss`], it is carried for the `help`
/// and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct EntryFix(pub Option<String>);

/// A method a `help` tells the reader to call: `to_optional` on a core builder before an optional
/// one's `set`, or `downcast_fields` on an extractor in place of `downcast`. Like [`NearMiss`], it
/// is carried for the `help` and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct MethodFix(pub Option<SuggestedCall>);

/// The method a [`MethodFix`] names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedCall {
    /// The method to call, e.g. `downcast_fields`.
    pub method: String,
    /// The method the call replaces, e.g. `downcast`, or `None` for a call made first.
    pub replaces: Option<String>,
    /// The path that imports the method's trait, e.g. `cgp::core::field::impls::CanDowncastFields`,
    /// since the data traits are not in the prelude. `None` when no public path is found.
    pub trait_path: Option<String>,
}

/// Equality and hashing that treat every value alike, for the facts a leaf carries for its `help`s
/// alone.
macro_rules! identity_free {
    ($ty:ty) => {
        impl PartialEq for $ty {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }

        impl Eq for $ty {}

        impl std::hash::Hash for $ty {
            fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
        }
    };
}

identity_free!(FieldFix);
identity_free!(AdapterFix);
identity_free!(ProvidesFix);
identity_free!(CoreProviderFix);
identity_free!(NearMiss);
identity_free!(DeriveFix);
identity_free!(EntryFix);
identity_free!(MethodFix);
