#![feature(prelude_import)]
//! Calling a `#[cgp_getter]` method on a context that does not wire the getter.
//!
//! The headline names `NameGetterComponent` as the context, over a `[CGP-E107]` leaf saying the same
//! marker lacks its own entry. The context is `Person`; plain rustc names it correctly.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str;
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
    fn name(__context__: &__Context__) -> &str;
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
pub struct Person {
    pub name: String,
}
impl HasField<Symbol!("name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
fn main() {
    let _ = Person { name: String::new() }.name();
}
