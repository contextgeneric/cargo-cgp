//! A `for` loop that wires each `ErrorHandlers` entry under the
//! prefixed path `@cgp.core.error.ErrorRaiserComponent.Key`, beside a
//! `namespace AppDefaults;` join that already registers
//! `@cgp.core.error.ErrorRaiserComponent.String`. The loop lowers to a
//! `DelegateComponent<PathCons<.., PathCons<Key, ..>>>` whose generic key
//! `@cgp.core.error.ErrorRaiserComponent.Key` overlaps the namespace forwarding, so
//! coherence rejects the pair with E0119.
//!
//! This is the *prefixed* `for`-key counterpart of the bare-key
//! [for_loop_bare_key.rs]: a bare loop key overlaps *every* key, while a prefixed
//! loop key overlaps only where the prefix path is itself routed by the namespace —
//! here `AppDefaults` registers `@cgp.core.error.ErrorRaiserComponent.String`, so the
//! generic loop tail collides with it. (A prefix the namespace does not register does
//! *not* overlap: the orphan rule proves it, which is why prefixing an otherwise-bare
//! `for` key is the fix.)
//!
//! The tool drops the redundant `IsProviderFor` half and rewrites the
//! `DelegateComponent` half to `[CGP-E011] conflicting wiring: `App` wires
//! `@cgp.core.error.ErrorRaiserComponent.Key`, which `AppDefaults` already covers`,
//! exercising the typed path renderer's naming of a `for`-loop key
//! parameter as written.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).

//@quality: acceptable
//@codes: CGP-E011

use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::DisplayError;
use cgp::prelude::*;

cgp_namespace! {
    new ErrorHandlers {
        String: DisplayError,
    }
}

cgp_namespace! {
    new AppDefaults: DefaultNamespace {
        @cgp.core.error.ErrorRaiserComponent.String: DisplayError,
    }
}

pub struct App;

delegate_components! {
    App {
        namespace AppDefaults;

        for <Key, Value> in ErrorHandlers {
            @cgp.core.error.ErrorRaiserComponent.Key: Value,
        }
    }
}

fn main() {}
