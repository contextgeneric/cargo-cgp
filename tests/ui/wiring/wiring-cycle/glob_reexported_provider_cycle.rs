//! A provider reached through glob re-exports of private modules, named in a cycle's help.
//!
//! `EvalAdd` is defined once, in the private module `providers::eval::add`, and reaches `providers`
//! through `pub use add::*` and `pub use eval::*`. It provides `Greeter` by requiring `CanGreet` on
//! the context, so wiring it as `App`'s greeter loops back (`E0275`). The compiler qualifies every
//! glob-imported name, and prints an item's path from its nearest enclosing module whose name is
//! unique, so the provider prints as `add::EvalAdd`. That path names the item the globs bring in
//! under `EvalAdd`, so the `[CGP-E020]` help names the provider bare, as `EvalAdd`.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).

//@quality: acceptable
//@codes: CGP-E020

use cgp::prelude::*;

use crate::components::*;

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }
}

pub mod providers {
    mod eval {
        mod add {
            use cgp::prelude::*;

            use crate::components::*;

            #[cgp_impl(new EvalAdd)]
            #[uses(CanGreet)]
            impl Greeter {
                fn greet(&self) {
                    self.greet()
                }
            }
        }

        pub use add::*;
    }

    pub use eval::*;
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: providers::EvalAdd,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
