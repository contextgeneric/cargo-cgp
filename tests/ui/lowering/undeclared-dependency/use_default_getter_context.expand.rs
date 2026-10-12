#![feature(prelude_import)]
//! A `UseDefault` provider whose default body calls a getter it does not declare in `#[uses]`.
//!
//! The component's `#[extend(HasName)]` requires `HasName` of every context, so rustc reports
//! `E0277` on `__Context__: HasName` at the `#[cgp_impl(UseDefault)]` attribute, leaking the
//! generated `__Context__`. The tool reshapes it into `[CGP-E036]`, naming `UseDefault` as the
//! provider, with `#[uses(HasName)]` in a `help`. The resolver skips the `NameGetterComponent`
//! marker, defined under the `#[cgp_getter]` attribute rustc points at, as a context candidate, so
//! it states no false context.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::UseDefault;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str {
        "John"
    }
}
impl<__Context__> HasName for __Context__
where
    __Context__: NameGetter<__Context__>,
{
    fn name(&self) -> &str {
        __Context__::name(self)
    }
}
pub trait NameGetter<__Context__>: IsProviderFor<NameGetterComponent, __Context__, ()> {
    fn name(__context__: &__Context__) -> &str {
        "John"
    }
}
impl<__Provider__, __Context__> NameGetter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<NameGetterComponent>
        + IsProviderFor<NameGetterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        NameGetterComponent,
    >>::Delegate: NameGetter<__Context__>,
{
    fn name(__context__: &__Context__) -> &str {
        <__Provider__ as DelegateComponent<
            NameGetterComponent,
        >>::Delegate::name(__context__)
    }
}
pub struct NameGetterComponent;
impl<__Context__> NameGetter<__Context__> for UseContext
where
    __Context__: HasName,
{
    fn name(__context__: &__Context__) -> &str {
        __Context__::name(__context__)
    }
}
impl<__Context__> IsProviderFor<NameGetterComponent, __Context__, ()> for UseContext
where
    __Context__: HasName,
{}
impl<__Context__, __Components__, __Path__> NameGetter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: NameGetter<__Context__>,
{
    fn name(__context__: &__Context__) -> &str {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::name(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<NameGetterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<NameGetterComponent, __Context__, ()>
        + NameGetter<__Context__>,
{}
impl<__Context__> NameGetter<__Context__> for UseFields
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __context__.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
impl<__Context__> IsProviderFor<NameGetterComponent, __Context__, ()> for UseFields
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{}
impl<__Context__, __Tag__> NameGetter<__Context__> for UseField<__Tag__>
where
    __Context__: HasField<__Tag__, Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __context__.get_field(::core::marker::PhantomData::<__Tag__>).as_str()
    }
}
impl<__Context__, __Tag__> IsProviderFor<NameGetterComponent, __Context__, ()>
for UseField<__Tag__>
where
    __Context__: HasField<__Tag__, Value = String>,
{}
impl<__Context__, __Provider__> NameGetter<__Context__> for WithProvider<__Provider__>
where
    __Provider__: FieldGetter<__Context__, NameGetterComponent, Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __Provider__::get_field(
                __context__,
                ::core::marker::PhantomData::<NameGetterComponent>,
            )
            .as_str()
    }
}
impl<__Context__, __Provider__> IsProviderFor<NameGetterComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: FieldGetter<__Context__, NameGetterComponent, Value = String>,
{}
pub trait CanGreet: HasName {
    fn greet(&self) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", self.name()))
        })
    }
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: HasName,
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> String {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()>
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", __context__.name()))
        })
    }
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Context__: HasName,
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: HasName,
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) -> String {
        __Context__::greet(__context__)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseContext
where
    __Context__: HasName,
    __Context__: CanGreet,
{}
impl<__Context__, __Components__, __Path__> Greeter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasName,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
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
    __Context__: HasName,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, ()>
        + Greeter<__Context__>,
{}
impl<__Context__> Greeter<__Context__> for UseDefault {}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseDefault {}
fn main() {}
