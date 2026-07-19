//! The constrained-key delegation failure in its canonical core-CGP form: wiring a component to
//! `PipeHandlers<Product![]>`, an *empty* pipeline.
//!
//! `PipeHandlers<Providers>`'s own `delegate_components!` (in `cgp-handler`) carries a constrained
//! generic list — `Providers: ComposeProviders<Provider = Provider>` — so its
//! `DelegateComponent<Component> for PipeHandlers<Providers>` impl applies only when `Providers`
//! composes. `ComposeProviders` is defined for a non-empty `Cons` list but not for the empty `Nil`,
//! so `PipeHandlers<Product![]>` (whose list is `Nil`) has no working delegation. `App` wires
//! `GreeterComponent` to it — `PipeHandlers`'s delegation is generic over the component, so any
//! component routes through it — and the check fails because the delegate entry's constrained key is
//! unsatisfiable, not because the component is unwired. The resolver should lead with the real
//! composition bound rather than the `IsProviderFor`/`DelegateComponent` scaffolding.

use cgp::extra::handler::PipeHandlers;
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct App;

// `Product![]` is the empty list `Nil`, which `PipeHandlers`'s `ComposeProviders` bound cannot
// satisfy, so this delegation's constrained key is unmet.
delegate_components! {
    App {
        GreeterComponent: PipeHandlers<Product![]>,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
