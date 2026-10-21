//! `#[check_providers(GreetAndBye)]` over two components that each fail for a field of their own.
//!
//! The table generates one check impl per component, both with `GreetAndBye` as `Self` on the one
//! token the attribute names, so the two errors share a caret. The tool reads each error's own
//! impl and gives one `[CGP-E002]` block per component: `Greeter` needs `name`, `Farewell` needs
//! `title`. Matching the impls by source position alone would read the first impl twice and drop
//! the second block as a repeat; the impls differ only in their spans' syntax contexts.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E002 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Farewell)]
pub trait CanSayBye {
    fn bye(&self) -> String;
}

#[cgp_impl(new GreetAndBye)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("hello {name}")
    }
}

#[cgp_impl(GreetAndBye)]
impl Farewell {
    fn bye(&self, #[implicit] title: &str) -> String {
        format!("bye {title}")
    }
}

pub struct App;

check_components! {
    #[check_providers(GreetAndBye)]
    App {
        GreeterComponent,
        FarewellComponent,
    }
}

fn main() {}
