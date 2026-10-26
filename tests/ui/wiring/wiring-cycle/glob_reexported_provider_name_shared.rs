//! Two providers named `EvalAdd`, one glob-imported, each named in a cycle's help.
//!
//! One `EvalAdd` is defined in the private module `providers::eval::add` and reaches the crate root
//! through glob re-exports; the other is `eval::add::EvalAdd`, under a root module of the same
//! name. Each provides `Greeter` by requiring `CanGreet` on the context, so `App`, wired to the
//! root one, and `Other`, wired to the glob-imported one, both loop back (`E0275`). Neither module
//! name is unique, so the compiler prints each provider's full path. The bare name `EvalAdd` means
//! the glob-imported provider here, so `Other`'s `[CGP-E020]` help names it bare, and `App`'s keeps
//! `eval::add::EvalAdd`, though that path is also the tail of the other provider's.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).

//@quality: acceptable
//@codes: CGP-E020

use cgp::prelude::*;

use crate::components::*;
use crate::providers::*;

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

pub mod eval {
    pub mod add {
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
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: eval::add::EvalAdd,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

pub struct Other;

delegate_components! {
    Other {
        GreeterComponent: EvalAdd,
    }
}

check_components! {
    Other {
        GreeterComponent,
    }
}

fn main() {}
