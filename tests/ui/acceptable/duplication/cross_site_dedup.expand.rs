#![feature(prelude_import)]
//! Acceptable: one wiring mistake, its re-reports collapsed per failing trait, each error headed by
//! the code the programmer wrote. CGP wiring is lazy, so the single missing `name` field `GreetHello`
//! needs fans out across the `check_components!` entry and the hand-written `CanGreetSend` impl (both
//! its header and its forwarding `self.greet()` call) — the transfer example's `Send`-recovery shape.
//! The tool collapses the re-reports of each failing trait to one block: the check entry becomes a
//! `[CGP-E001]` `CanGreet` consumer-trait error, and the `CanGreetSend` impl (header and call) a
//! single `[CGP-E009]` error. A wrapper is a plain trait, not a CGP consumer, so its header reads
//! "the trait", and its tree is headed by `CanGreetSend` (the code the programmer wrote), descending
//! through its `CanGreet` supertrait to the missing field rather than hiding the wrapper. The two
//! blocks stay distinct because they are distinct traits; no re-report repeats. The `.rust.stderr`
//! baseline shows the full cascade for contrast.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
pub trait CanGreet {
    fn greet(&self);
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__);
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) {
        __Context__::greet(__context__)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseContext
where
    __Context__: CanGreet,
{}
impl<__Context__, __Components__, __Path__> Greeter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, ()>
        + Greeter<__Context__>,
{}
impl<__Context__> Greeter<__Context__> for GreetHello
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasName,
{}
pub struct GreetHello;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
pub trait CanGreetSend: CanGreet + Send {
    fn greet_send(&self);
}
impl CanGreetSend for App {
    fn greet_send(&self) {
        self.greet()
    }
}
fn main() {}
