#![feature(prelude_import)]
//! Acceptable: the unregistered-namespace-path failure (as in
//! `acceptable/resolution/unregistered_prefix_path`), but with the prefixed component defined
//! in a *sub-module* and filed under a multi-segment path, so rustc prints the component
//! segment module-qualified (`finance::QuantityTypeProviderComponent`). The `resugar_path`
//! post-processor folds such a qualified segment to its final identifier, so the redirect path
//! reads as `Path!(@app.finance.types.QuantityTypeProviderComponent)` rather than a raw
//! `PathCons<…>` spine. This is the multi-module case a real project (cgp-examples/transfer)
//! surfaced and the single-module fixtures never exercised — before the fold, the raw spine
//! appeared three times in the one error.
//!
//! `App` joins `DefaultNamespace`, which routes the prefixed `QuantityTypeProviderComponent`
//! to `@app.finance.types.QuantityTypeProviderComponent`, but nothing ever terminates that
//! path with a provider, so the lookup finds no delegate and the `check_components!` fails.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use finance::*;
pub mod finance {
    use cgp::prelude::*;
    pub trait HasQuantityType {
        type Quantity;
    }
    impl<__Context__> HasQuantityType for __Context__
    where
        __Context__: QuantityTypeProvider<__Context__>,
    {
        type Quantity = <__Context__ as QuantityTypeProvider<__Context__>>::Quantity;
    }
    pub trait QuantityTypeProvider<
        __Context__,
    >: IsProviderFor<QuantityTypeProviderComponent, __Context__, ()> {
        type Quantity;
    }
    impl<__Provider__, __Context__> QuantityTypeProvider<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<QuantityTypeProviderComponent>
            + IsProviderFor<QuantityTypeProviderComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            QuantityTypeProviderComponent,
        >>::Delegate: QuantityTypeProvider<__Context__>,
    {
        type Quantity = <<__Provider__ as DelegateComponent<
            QuantityTypeProviderComponent,
        >>::Delegate as QuantityTypeProvider<__Context__>>::Quantity;
    }
    pub struct QuantityTypeProviderComponent;
    impl<__Context__> QuantityTypeProvider<__Context__> for UseContext
    where
        __Context__: HasQuantityType,
    {
        type Quantity = <__Context__ as HasQuantityType>::Quantity;
    }
    impl<__Context__> IsProviderFor<QuantityTypeProviderComponent, __Context__, ()>
    for UseContext
    where
        __Context__: HasQuantityType,
    {}
    impl<__Context__, __Components__, __Path__> QuantityTypeProvider<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: QuantityTypeProvider<__Context__>,
    {
        type Quantity = <<__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate as QuantityTypeProvider<__Context__>>::Quantity;
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<QuantityTypeProviderComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<QuantityTypeProviderComponent, __Context__, ()>
            + QuantityTypeProvider<__Context__>,
    {}
    impl<__Components__> DefaultNamespace<__Components__>
    for QuantityTypeProviderComponent {
        type Delegate = RedirectLookup<
            __Components__,
            Path!(@app.finance.types.QuantityTypeProviderComponent),
        >;
    }
    impl<Quantity, __Context__> QuantityTypeProvider<__Context__> for UseType<Quantity> {
        type Quantity = Quantity;
    }
    impl<
        Quantity,
        __Context__,
    > IsProviderFor<QuantityTypeProviderComponent, __Context__, ()>
    for UseType<Quantity> {}
    impl<__Provider__, Quantity, __Context__> QuantityTypeProvider<__Context__>
    for WithProvider<__Provider__>
    where
        __Provider__: TypeProvider<
            __Context__,
            QuantityTypeProviderComponent,
            Type = Quantity,
        >,
    {
        type Quantity = Quantity;
    }
    impl<
        __Provider__,
        Quantity,
        __Context__,
    > IsProviderFor<QuantityTypeProviderComponent, __Context__, ()>
    for WithProvider<__Provider__>
    where
        __Provider__: TypeProvider<
            __Context__,
            QuantityTypeProviderComponent,
            Type = Quantity,
        >,
    {}
}
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<QuantityTypeProviderComponent, ()> for App {}
fn main() {}
