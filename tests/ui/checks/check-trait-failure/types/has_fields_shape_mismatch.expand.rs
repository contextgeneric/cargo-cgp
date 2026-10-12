#![feature(prelude_import)]
//! A provider bound on the context's whole shape, wired on a context whose field type differs.
//!
//! `DescribeNamed` requires `HasFields<Fields = Struct! { name: String }>` and `App` has `name: u32`.
//! The projection mismatch becomes `[CGP-E005]` naming the associated type `Fields` of `HasFields`,
//! with both shapes resugared, over a tree ending in the `[CGP-E116]` leaf.
//!
//! CGP error class: check-trait failure, its projection-mismatch face
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanDescribe {
    fn describe(&self) -> String;
}
impl<__Context__> CanDescribe for __Context__
where
    __Context__: Describer<__Context__>,
{
    fn describe(&self) -> String {
        __Context__::describe(self)
    }
}
pub trait Describer<__Context__>: IsProviderFor<DescriberComponent, __Context__, ()> {
    fn describe(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Describer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DescriberComponent>
        + IsProviderFor<DescriberComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DescriberComponent,
    >>::Delegate: Describer<__Context__>,
{
    fn describe(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            DescriberComponent,
        >>::Delegate::describe(__context__)
    }
}
pub struct DescriberComponent;
impl<__Context__> Describer<__Context__> for UseContext
where
    __Context__: CanDescribe,
{
    fn describe(__context__: &__Context__) -> String {
        __Context__::describe(__context__)
    }
}
impl<__Context__> IsProviderFor<DescriberComponent, __Context__, ()> for UseContext
where
    __Context__: CanDescribe,
{}
impl<__Context__, __Components__, __Path__> Describer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Describer<__Context__>,
{
    fn describe(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::describe(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DescriberComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DescriberComponent, __Context__, ()>
        + Describer<__Context__>,
{}
impl<__Context__> Describer<__Context__> for DescribeNamed
where
    __Context__: HasFields<
        Fields = Struct! { name: String },
    >,
{
    fn describe(__context__: &__Context__) -> String {
        "named".into()
    }
}
impl<__Context__> IsProviderFor<DescriberComponent, __Context__, ()> for DescribeNamed
where
    __Context__: HasFields<
        Fields = Struct! { name: String },
    >,
{}
pub struct DescribeNamed;
pub struct App {
    pub name: u32,
}
impl HasFields for App {
    type Fields = Struct! { name: u32 };
}
impl HasFieldsRef for App {
    type FieldsRef<'__a> = Struct! { name: &'__a u32 }
    where
        Self: '__a;
}
impl FromFields for App {
    fn from_fields(Cons(name, Nil): Self::Fields) -> Self {
        Self { name: name.value }
    }
}
impl ToFields for App {
    fn to_fields(self) -> Self::Fields {
        Cons(self.name.into(), Nil)
    }
}
impl ToFieldsRef for App {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.name).into(), Nil)
    }
}
impl DelegateComponent<DescriberComponent> for App {
    type Delegate = DescribeNamed;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<DescriberComponent, __Context__, __Params__> for App
where
    DescribeNamed: IsProviderFor<DescriberComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<DescriberComponent, ()> for App {}
fn main() {}
