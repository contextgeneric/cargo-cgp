//! Clean compile: a provider reads its own associated const as `Self::LIMIT` inside a
//! `#[cgp_impl]` body.
//!
//! `#[cgp_impl]` rewrites `Self` to the context, but a `Self::LIMIT` naming a const the block
//! declares means the provider's own const. The macro rewrites it to the qualified
//! `<AllowUnderLimit as RateLimiter<__Context__>>::LIMIT`, which the expansion shows, since a bare
//! `Self::LIMIT` on the provider would be ambiguous with the consumer trait's `LIMIT`.

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
