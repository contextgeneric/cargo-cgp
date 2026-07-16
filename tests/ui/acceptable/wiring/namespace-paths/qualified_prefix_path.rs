//! Acceptable: the unregistered-namespace-path failure (as in
//! `acceptable/resolution/unregistered_prefix_path`), but with the prefixed component defined
//! in a *sub-module* and filed under a multi-segment path, so rustc prints the component
//! segment module-qualified (`finance::QuantityTypeProviderComponent`). The `resugar_path`
//! post-processor folds such a qualified segment to its final identifier, so the redirect path
//! reads as `Path!(@app.finance.types.QuantityTypeProviderComponent)` rather than a raw
//! `PathCons<…>` spine. This is the multi-module case a real project (cgp-examples/transfer)
//! surfaced and the single-module fixtures never exercised — before the fold, the raw spine
//! appeared three times in the one error.
//!
//! `App` joins `DefaultNamespace`, which routes the prefixed `QuantityTypeProviderComponent`
//! to `@app.finance.types.QuantityTypeProviderComponent`, but nothing ever terminates that
//! path with a provider, so the lookup finds no delegate and the `check_components!` fails.

use cgp::prelude::*;

use finance::*;

pub mod finance {
    use cgp::prelude::*;

    #[cgp_type]
    #[prefix(@app.finance.types in DefaultNamespace)]
    pub trait HasQuantityType {
        type Quantity;
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;
    }
}

check_components! {
    App {
        QuantityTypeProviderComponent,
    }
}

fn main() {}
