//! Usability: a use-site failure on a namespace-joined context, which the resolver
//! must not mis-recover through the namespace's blanket forwarding.
//!
//! `App` joins `MyNamespace` (so it inherits the `Greeter` wiring) but has no
//! `name` field, so `GreetHello`'s `Self: HasName` dependency cannot be met and
//! `App.greet()` fails at the use site as an `E0599`. The resolver finds `App` from
//! the receiver span and re-checks the components it wires — but a namespace join
//! gives `App` only a *blanket* `DelegateComponent<__Key__>` forwarding, whose key
//! is a bare type parameter, not a concrete component marker. Re-checking that
//! parameter as if it were a component produced pure garbage
//! (`the consumer trait `__Key__``, `__Key__: Sized`, `no delegate entry for
//! `__Key__``). The resolver now skips such a param key and declines to the text
//! rewrite, so the output is at least truthful rustc rather than nonsense.
//!
//! What remains a usability gap is that the declined `E0599` keeps rustc's
//! misleading "use associated function syntax instead" advice — the same residual
//! the non-namespace use-site cases carry until the resolver can recover a
//! namespace-joined context's real wiring. See docs/issues/usability.md.

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
