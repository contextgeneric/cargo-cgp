//! A component delegation that fails because the `DelegateComponent` impl carries a **constrained
//! key** whose `where`-clause is unsatisfied — the shape `PipeHandlers<Providers>` produces (its
//! `delegate_components!` generic list is `Providers: ComposeProviders<Provider = Provider>`, so an
//! un-composable `Providers` makes the delegation itself fail).
//!
//! `PickFirstProvider<List>` is such a dispatcher: its wiring delegates every component to the
//! provider its `List` parameter reduces to, but only when `List: PickFirst` holds — and `PickFirst`
//! is implemented for a non-empty `Cons`, never for the empty `Nil`. `App` wires `GreeterComponent`
//! to `PickFirstProvider<Product![]>`, whose `Nil` list has no `PickFirst` impl, so the generated
//! `DelegateComponent<GreeterComponent> for PickFirstProvider<Nil>` impl cannot apply. The delegate
//! entry *exists* but its own bound is unmet — distinct from a component wired nowhere.

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
pub struct PickFirstProvider<List>(pub PhantomData<List>);

/// The reduction: a non-empty list yields its head provider. `Nil` has no impl, so an empty list
/// cannot reduce.
pub trait PickFirst {
    type Provider;
}

impl<Head, Tail> PickFirst for Cons<Head, Tail> {
    type Provider = Head;
}

delegate_components! {
    <Component, Provider, List: PickFirst<Provider = Provider>>
    PickFirstProvider<List> {
        Component: Provider,
    }
}

pub struct App;

// The empty list `Product![]` is `Nil`, which has no `PickFirst` impl, so this delegation's
// constrained key is unsatisfiable.
delegate_components! {
    App {
        GreeterComponent: PickFirstProvider<Product![]>,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
