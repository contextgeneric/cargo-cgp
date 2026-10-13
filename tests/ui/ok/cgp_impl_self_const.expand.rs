#![feature(prelude_import)]
//! Clean compile: a provider reads its own associated const as `Self::LIMIT` inside a
//! `#[cgp_impl]` body.
//!
//! `#[cgp_impl]` rewrites `Self` to the context, but a `Self::LIMIT` naming a const the block
//! declares means the provider's own const. The macro rewrites it to the qualified
//! `<AllowUnderLimit as RateLimiter<__Context__>>::LIMIT`, which the expansion shows, since a bare
//! `Self::LIMIT` on the provider would be ambiguous with the consumer trait's `LIMIT`.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanRateLimit {
    const LIMIT: u32;
    fn allow(&self, count: u32) -> bool;
}
impl<__Context__> CanRateLimit for __Context__
where
    __Context__: RateLimiter<__Context__>,
{
    const LIMIT: u32 = <__Context__ as RateLimiter<__Context__>>::LIMIT;
    fn allow(&self, count: u32) -> bool {
        __Context__::allow(self, count)
    }
}
pub trait RateLimiter<
    __Context__,
>: IsProviderFor<RateLimiterComponent, __Context__, ()> {
    const LIMIT: u32;
    fn allow(__context__: &__Context__, count: u32) -> bool;
}
impl<__Provider__, __Context__> RateLimiter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<RateLimiterComponent>
        + IsProviderFor<RateLimiterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        RateLimiterComponent,
    >>::Delegate: RateLimiter<__Context__>,
{
    const LIMIT: u32 = <<__Provider__ as DelegateComponent<
        RateLimiterComponent,
    >>::Delegate as RateLimiter<__Context__>>::LIMIT;
    fn allow(__context__: &__Context__, count: u32) -> bool {
        <__Provider__ as DelegateComponent<
            RateLimiterComponent,
        >>::Delegate::allow(__context__, count)
    }
}
pub struct RateLimiterComponent;
impl<__Context__> RateLimiter<__Context__> for UseContext
where
    __Context__: CanRateLimit,
{
    const LIMIT: u32 = <__Context__ as CanRateLimit>::LIMIT;
    fn allow(__context__: &__Context__, count: u32) -> bool {
        __Context__::allow(__context__, count)
    }
}
impl<__Context__> IsProviderFor<RateLimiterComponent, __Context__, ()> for UseContext
where
    __Context__: CanRateLimit,
{}
impl<__Context__, __Components__, __Path__> RateLimiter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: RateLimiter<__Context__>,
{
    const LIMIT: u32 = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as RateLimiter<__Context__>>::LIMIT;
    fn allow(__context__: &__Context__, count: u32) -> bool {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::allow(__context__, count)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<RateLimiterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<RateLimiterComponent, __Context__, ()>
        + RateLimiter<__Context__>,
{}
impl<__Context__> RateLimiter<__Context__> for AllowUnderLimit {
    const LIMIT: u32 = 10;
    fn allow(__context__: &__Context__, count: u32) -> bool {
        count < <AllowUnderLimit as RateLimiter<__Context__>>::LIMIT
    }
}
impl<__Context__> IsProviderFor<RateLimiterComponent, __Context__, ()>
for AllowUnderLimit {}
pub struct AllowUnderLimit;
pub struct App;
impl DelegateComponent<RateLimiterComponent> for App {
    type Delegate = AllowUnderLimit;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<RateLimiterComponent, __Context__, __Params__> for App
where
    AllowUnderLimit: IsProviderFor<RateLimiterComponent, __Context__, __Params__>,
{}
fn main() {
    App.allow(1);
}
