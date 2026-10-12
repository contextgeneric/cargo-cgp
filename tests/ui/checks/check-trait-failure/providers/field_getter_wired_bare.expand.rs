#![feature(prelude_import)]
//! A hand-written `FieldGetter` wired directly as a component's provider.
//!
//! `[CGP-E115]` is accurate; the fix, `WithProvider<…>`, is not hinted.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
pub struct Profile {
    pub display_name: String,
}
pub struct Account {
    pub profile: Profile,
}
impl HasField<Symbol!("profile")> for Account {
    type Value = Profile;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("profile")>,
    ) -> &Self::Value {
        &self.profile
    }
}
impl HasFieldMut<Symbol!("profile")> for Account {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("profile")>,
    ) -> &mut Self::Value {
        &mut self.profile
    }
}
pub struct ReadDisplayName;
impl<Tag> FieldGetter<Account, Tag> for ReadDisplayName {
    type Value = String;
    fn get_field(context: &Account, _tag: PhantomData<Tag>) -> &String {
        &context.profile.display_name
    }
}
impl DelegateComponent<NameGetterComponent> for Account {
    type Delegate = ReadDisplayName;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<NameGetterComponent, __Context__, __Params__> for Account
where
    ReadDisplayName: IsProviderFor<NameGetterComponent, __Context__, __Params__>,
{}
trait __CheckAccount<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckAccount<NameGetterComponent, ()> for Account {}
fn main() {}
