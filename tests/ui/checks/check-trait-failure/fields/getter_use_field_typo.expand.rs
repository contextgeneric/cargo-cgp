#![feature(prelude_import)]
//! `UseField<Symbol!("frist_name")>` for a field named `first_name`.
//!
//! The headline reads `` [CGP-E001] missing field `frist_name` on `Person` ``, with the same text
//! as the `` [CGP-E110] `` root cause, and a `help` names the similar field `first_name`, so the
//! typo is pointed out.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
    pub first_name: String,
}
impl HasField<Symbol!("first_name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &Self::Value {
        &self.first_name
    }
}
impl HasFieldMut<Symbol!("first_name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &mut Self::Value {
        &mut self.first_name
    }
}
impl DelegateComponent<NameGetterComponent> for Person {
    type Delegate = UseField<Symbol!("frist_name")>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<NameGetterComponent, __Context__, __Params__> for Person
where
    UseField<
        Symbol!("frist_name"),
    >: IsProviderFor<NameGetterComponent, __Context__, __Params__>,
{}
fn main() {
    let _ = Person {
        first_name: String::new(),
    }
        .name();
}
