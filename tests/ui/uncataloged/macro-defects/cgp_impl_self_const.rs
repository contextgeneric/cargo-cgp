//! `Self::LIMIT` inside a `#[cgp_impl]` body, which the macro rewrites to the context.
//!
//! The `E0599` names `__Context__`, with suggestions spliced after the attribute. A `cgp` fix that
//! exempts associated constants from the rewrite would remove the error.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(RateLimiter)]
pub trait CanRateLimit {
    const LIMIT: u32;
    fn allow(&self, count: u32) -> bool;
}

#[cgp_impl(new AllowUnderLimit)]
impl RateLimiter {
    const LIMIT: u32 = 10;

    fn allow(&self, count: u32) -> bool {
        count < Self::LIMIT
    }
}

pub struct App;

delegate_components! {
    App {
        RateLimiterComponent: AllowUnderLimit,
    }
}

fn main() {
    App.allow(1);
}
