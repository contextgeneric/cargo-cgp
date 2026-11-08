#![feature(prelude_import)]
//! A provider reading six fields the context lacks, through `#[implicit]` arguments.
//!
//! `SumFields` reads `alpha` to `zeta` as implicit arguments, and `App` declares none of them,
//! so the check has six root causes of one kind on one owner. The headline states them as one
//! merged sentence, the `root causes:` list names each, and the tree branches to each field.
//! The fixture pins two rules: the causes appear in the order the arguments are written, and a
//! long merged headline names the first few fields and counts the rest, while the list and the
//! tree stay complete.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanSum {
    fn sum(&self) -> u64;
}
impl<__Context__> CanSum for __Context__
where
    __Context__: Summer<__Context__>,
{
    fn sum(&self) -> u64 {
        __Context__::sum(self)
    }
}
pub trait Summer<__Context__>: IsProviderFor<SummerComponent, __Context__, ()> {
    fn sum(__context__: &__Context__) -> u64;
}
impl<__Provider__, __Context__> Summer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<SummerComponent>
        + IsProviderFor<SummerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<SummerComponent>>::Delegate: Summer<__Context__>,
{
    fn sum(__context__: &__Context__) -> u64 {
        <__Provider__ as DelegateComponent<SummerComponent>>::Delegate::sum(__context__)
    }
}
///Component marker for [`CanSum`].
pub struct SummerComponent;
impl<__Context__> Summer<__Context__> for UseContext
where
    __Context__: CanSum,
{
    fn sum(__context__: &__Context__) -> u64 {
        __Context__::sum(__context__)
    }
}
impl<__Context__> IsProviderFor<SummerComponent, __Context__, ()> for UseContext
where
    __Context__: CanSum,
{}
impl<__Context__, __Components__, __Path__> Summer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Summer<__Context__>,
{
    fn sum(__context__: &__Context__) -> u64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::sum(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<SummerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<SummerComponent, __Context__, ()> + Summer<__Context__>,
{}
impl<__Context__> Summer<__Context__> for SumFields
where
    __Context__: HasField<Symbol!("alpha"), Value = u64>
        + HasField<Symbol!("beta"), Value = u64>
        + HasField<Symbol!("gamma"), Value = u64>
        + HasField<Symbol!("delta"), Value = u64>
        + HasField<Symbol!("epsilon"), Value = u64>
        + HasField<Symbol!("zeta"), Value = u64>,
{
    fn sum(__context__: &__Context__) -> u64 {
        let alpha: u64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("alpha")>,
            ),
        );
        let beta: u64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("beta")>,
            ),
        );
        let gamma: u64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("gamma")>,
            ),
        );
        let delta: u64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("delta")>,
            ),
        );
        let epsilon: u64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("epsilon")>,
            ),
        );
        let zeta: u64 = ::core::clone::Clone::clone(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("zeta")>,
            ),
        );
        alpha + beta + gamma + delta + epsilon + zeta
    }
}
impl<__Context__> IsProviderFor<SummerComponent, __Context__, ()> for SumFields
where
    __Context__: HasField<Symbol!("alpha"), Value = u64>
        + HasField<Symbol!("beta"), Value = u64>
        + HasField<Symbol!("gamma"), Value = u64>
        + HasField<Symbol!("delta"), Value = u64>
        + HasField<Symbol!("epsilon"), Value = u64>
        + HasField<Symbol!("zeta"), Value = u64>,
{}
///Provider `SumFields` for [`Summer`].
pub struct SumFields;
pub struct App {
    pub name: String,
}
impl HasField<Symbol!("name")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
impl DelegateComponent<SummerComponent> for App {
    type Delegate = SumFields;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<SummerComponent, __Context__, __Params__> for App
where
    SumFields: IsProviderFor<SummerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<SummerComponent, ()> for App {}
fn main() {}
