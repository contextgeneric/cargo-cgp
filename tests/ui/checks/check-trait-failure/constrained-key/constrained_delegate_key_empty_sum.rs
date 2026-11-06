//! A delegation whose constrained key fails on CGP's empty sum, `Sum![]`.
//!
//! The sibling of `constrained_delegate_key` for variants: `PickFirstVariantProvider<List>`
//! delegates every component only when `List: PickFirstVariant` holds, which is implemented for a
//! non-empty `Either` and never for the empty `Void`. `App` wires `GreeterComponent` to
//! `PickFirstVariantProvider<Sum![]>`, so the delegation's own bound is unmet. rustc prints the
//! empty sum as `Void`; the tool's tree and headline write it `Sum![]`, as the program does.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E101 CGP-E102 CGP-E117

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new HelloGreeter)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

/// A dispatcher provider that forwards a component to whatever provider its `List` reduces to.
pub struct PickFirstVariantProvider<List>(pub PhantomData<List>);

/// The reduction: a non-empty sum yields its head provider. `Void` has no impl, so an empty sum
/// cannot reduce.
pub trait PickFirstVariant {
    type Provider;
}

impl<Head, Tail> PickFirstVariant for Either<Head, Tail> {
    type Provider = Head;
}

delegate_components! {
    <Component, Provider, List: PickFirstVariant<Provider = Provider>>
    PickFirstVariantProvider<List> {
        Component: Provider,
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: PickFirstVariantProvider<Sum![]>,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
