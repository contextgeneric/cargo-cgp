//! A use-site failure on a namespace-joined context, resolved to its root cause
//! *through* the namespace.
//!
//! `App` joins `MyNamespace` (inheriting the `Greeter` wiring) but has no `name`
//! field, so `GreetHello`'s `Self: HasName` dependency cannot be met and
//! `app.greet()` fails at the use site as an `E0599`. A namespace join gives `App`
//! only a blanket `DelegateComponent<__Key__>` forwarding, so its concrete wiring is
//! not in its own `DelegateComponent` impls; the resolver instead anchors on the
//! `CanGreet` consumer trait the diagnostic names, then walks `App: CanGreet` down
//! through the namespace's `RedirectLookup` to the real `GreetHello` provider and
//! its missing `name` field. The walk reads the real consumer/provider trait
//! obligations (never `IsProviderFor`), and the blanket `__Key__` key is skipped as
//! the non-component it is, so no placeholder noise leaks.

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @app.GreeterComponent:
            GreetHello,
    }
}

#[derive(HasField)]
pub struct App {
    // No `name` field — `GreetHello`'s `Self: HasName` dependency cannot be met.
}

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

fn main() {
    let app = App {};
    app.greet();
}
