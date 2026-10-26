#![feature(prelude_import)]
//! A getter with two methods wired to `UseField` on a context that lacks one of the getter's
//! fields.
//!
//! `UseField<Symbol!("width")>` reads one field, so it implements no getter with two methods, and
//! the check reports it as no provider of `DimensionsGetter`. `UseFields`, which reads each method's
//! field by the method's name, would not implement the getter either, since `Rectangle` has no
//! `height`, so no `help` names it.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasDimensions {
    fn width(&self) -> &f64;
    fn height(&self) -> &f64;
}
impl<__Context__> HasDimensions for __Context__
where
    __Context__: DimensionsGetter<__Context__>,
{
    fn width(&self) -> &f64 {
        __Context__::width(self)
    }
    fn height(&self) -> &f64 {
        __Context__::height(self)
    }
}
pub trait DimensionsGetter<
    __Context__,
>: IsProviderFor<DimensionsGetterComponent, __Context__, ()> {
    fn width(__context__: &__Context__) -> &f64;
    fn height(__context__: &__Context__) -> &f64;
}
impl<__Provider__, __Context__> DimensionsGetter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DimensionsGetterComponent>
        + IsProviderFor<DimensionsGetterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DimensionsGetterComponent,
    >>::Delegate: DimensionsGetter<__Context__>,
{
    fn width(__context__: &__Context__) -> &f64 {
        <__Provider__ as DelegateComponent<
            DimensionsGetterComponent,
        >>::Delegate::width(__context__)
    }
    fn height(__context__: &__Context__) -> &f64 {
        <__Provider__ as DelegateComponent<
            DimensionsGetterComponent,
        >>::Delegate::height(__context__)
    }
}
pub struct DimensionsGetterComponent;
impl<__Context__> DimensionsGetter<__Context__> for UseContext
where
    __Context__: HasDimensions,
{
    fn width(__context__: &__Context__) -> &f64 {
        __Context__::width(__context__)
    }
    fn height(__context__: &__Context__) -> &f64 {
        __Context__::height(__context__)
    }
}
impl<__Context__> IsProviderFor<DimensionsGetterComponent, __Context__, ()>
for UseContext
where
    __Context__: HasDimensions,
{}
impl<__Context__, __Components__, __Path__> DimensionsGetter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: DimensionsGetter<__Context__>,
{
    fn width(__context__: &__Context__) -> &f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::width(__context__)
    }
    fn height(__context__: &__Context__) -> &f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::height(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DimensionsGetterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DimensionsGetterComponent, __Context__, ()>
        + DimensionsGetter<__Context__>,
{}
impl<__Context__> DimensionsGetter<__Context__> for UseFields
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("height"), Value = f64>,
{
    fn width(__context__: &__Context__) -> &f64 {
        __context__.get_field(::core::marker::PhantomData::<Symbol!("width")>)
    }
    fn height(__context__: &__Context__) -> &f64 {
        __context__.get_field(::core::marker::PhantomData::<Symbol!("height")>)
    }
}
impl<__Context__> IsProviderFor<DimensionsGetterComponent, __Context__, ()> for UseFields
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("height"), Value = f64>,
{}
pub struct Rectangle {
    pub width: f64,
}
impl HasField<Symbol!("width")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &Self::Value {
        &self.width
    }
}
impl HasFieldMut<Symbol!("width")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &mut Self::Value {
        &mut self.width
    }
}
impl DelegateComponent<DimensionsGetterComponent> for Rectangle {
    type Delegate = UseField<Symbol!("width")>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<DimensionsGetterComponent, __Context__, __Params__> for Rectangle
where
    UseField<
        Symbol!("width"),
    >: IsProviderFor<DimensionsGetterComponent, __Context__, __Params__>,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<DimensionsGetterComponent, ()> for Rectangle {}
fn main() {}
