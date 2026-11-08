#![feature(prelude_import)]
//! A `#[check_providers]` check whose component shares its marker name with another component.
//!
//! `shapes_a` and `shapes_b` each define a component whose marker is named `MeasurerComponent`,
//! with the provider traits `MeasurerA` and `MeasurerB`. The check on `MeasureWidthA` fails for a
//! missing field. The provider-check anchor maps the entry's marker to its provider trait by
//! `DefId`, so the tree's provider step names `MeasurerA`, never the same-named marker's
//! `MeasurerB`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub mod shapes_a {
    use cgp::prelude::*;
    pub trait CanMeasureA {
        fn measure(&self) -> f64;
    }
    impl<__Context__> CanMeasureA for __Context__
    where
        __Context__: MeasurerA<__Context__>,
    {
        fn measure(&self) -> f64 {
            __Context__::measure(self)
        }
    }
    pub trait MeasurerA<__Context__>: IsProviderFor<MeasurerComponent, __Context__, ()> {
        fn measure(__context__: &__Context__) -> f64;
    }
    impl<__Provider__, __Context__> MeasurerA<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<MeasurerComponent>
            + IsProviderFor<MeasurerComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            MeasurerComponent,
        >>::Delegate: MeasurerA<__Context__>,
    {
        fn measure(__context__: &__Context__) -> f64 {
            <__Provider__ as DelegateComponent<
                MeasurerComponent,
            >>::Delegate::measure(__context__)
        }
    }
    ///Component marker for [`CanMeasureA`].
    pub struct MeasurerComponent;
    impl<__Context__> MeasurerA<__Context__> for UseContext
    where
        __Context__: CanMeasureA,
    {
        fn measure(__context__: &__Context__) -> f64 {
            __Context__::measure(__context__)
        }
    }
    impl<__Context__> IsProviderFor<MeasurerComponent, __Context__, ()> for UseContext
    where
        __Context__: CanMeasureA,
    {}
    impl<__Context__, __Components__, __Path__> MeasurerA<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: MeasurerA<__Context__>,
    {
        fn measure(__context__: &__Context__) -> f64 {
            <__Components__ as DelegateComponent<
                __Path__,
            >>::Delegate::measure(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<MeasurerComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<MeasurerComponent, __Context__, ()>
            + MeasurerA<__Context__>,
    {}
    pub trait HasWidthA {
        fn width_a(&self) -> f64;
    }
    impl<__Context__> HasWidthA for __Context__
    where
        __Context__: HasField<Symbol!("width_a"), Value = f64>,
    {
        fn width_a(&self) -> f64 {
            ::core::clone::Clone::clone(
                HasField::get_field(
                    self,
                    ::core::marker::PhantomData::<Symbol!("width_a")>,
                ),
            )
        }
    }
    impl<__Context__> MeasurerA<__Context__> for MeasureWidthA
    where
        __Context__: HasWidthA,
    {
        fn measure(__context__: &__Context__) -> f64 {
            __context__.width_a()
        }
    }
    impl<__Context__> IsProviderFor<MeasurerComponent, __Context__, ()> for MeasureWidthA
    where
        __Context__: HasWidthA,
    {}
    ///Provider `MeasureWidthA` for [`MeasurerA`].
    pub struct MeasureWidthA;
}
pub mod shapes_b {
    use cgp::prelude::*;
    pub trait CanMeasureB {
        fn measure(&self) -> f64;
    }
    impl<__Context__> CanMeasureB for __Context__
    where
        __Context__: MeasurerB<__Context__>,
    {
        fn measure(&self) -> f64 {
            __Context__::measure(self)
        }
    }
    pub trait MeasurerB<__Context__>: IsProviderFor<MeasurerComponent, __Context__, ()> {
        fn measure(__context__: &__Context__) -> f64;
    }
    impl<__Provider__, __Context__> MeasurerB<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<MeasurerComponent>
            + IsProviderFor<MeasurerComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            MeasurerComponent,
        >>::Delegate: MeasurerB<__Context__>,
    {
        fn measure(__context__: &__Context__) -> f64 {
            <__Provider__ as DelegateComponent<
                MeasurerComponent,
            >>::Delegate::measure(__context__)
        }
    }
    ///Component marker for [`CanMeasureB`].
    pub struct MeasurerComponent;
    impl<__Context__> MeasurerB<__Context__> for UseContext
    where
        __Context__: CanMeasureB,
    {
        fn measure(__context__: &__Context__) -> f64 {
            __Context__::measure(__context__)
        }
    }
    impl<__Context__> IsProviderFor<MeasurerComponent, __Context__, ()> for UseContext
    where
        __Context__: CanMeasureB,
    {}
    impl<__Context__, __Components__, __Path__> MeasurerB<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: MeasurerB<__Context__>,
    {
        fn measure(__context__: &__Context__) -> f64 {
            <__Components__ as DelegateComponent<
                __Path__,
            >>::Delegate::measure(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<MeasurerComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<MeasurerComponent, __Context__, ()>
            + MeasurerB<__Context__>,
    {}
    pub trait HasWidthB {
        fn width_b(&self) -> f64;
    }
    impl<__Context__> HasWidthB for __Context__
    where
        __Context__: HasField<Symbol!("width_b"), Value = f64>,
    {
        fn width_b(&self) -> f64 {
            ::core::clone::Clone::clone(
                HasField::get_field(
                    self,
                    ::core::marker::PhantomData::<Symbol!("width_b")>,
                ),
            )
        }
    }
    impl<__Context__> MeasurerB<__Context__> for MeasureWidthB
    where
        __Context__: HasWidthB,
    {
        fn measure(__context__: &__Context__) -> f64 {
            __context__.width_b()
        }
    }
    impl<__Context__> IsProviderFor<MeasurerComponent, __Context__, ()> for MeasureWidthB
    where
        __Context__: HasWidthB,
    {}
    ///Provider `MeasureWidthB` for [`MeasurerB`].
    pub struct MeasureWidthB;
}
pub struct App {
    pub other: f64,
}
impl HasField<Symbol!("other")> for App {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &Self::Value {
        &self.other
    }
}
impl HasFieldMut<Symbol!("other")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &mut Self::Value {
        &mut self.other
    }
}
impl DelegateComponent<shapes_a::MeasurerComponent> for App {
    type Delegate = shapes_a::MeasureWidthA;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<shapes_a::MeasurerComponent, __Context__, __Params__> for App
where
    shapes_a::MeasureWidthA: IsProviderFor<
        shapes_a::MeasurerComponent,
        __Context__,
        __Params__,
    >,
{}
impl DelegateComponent<shapes_b::MeasurerComponent> for App {
    type Delegate = shapes_b::MeasureWidthB;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<shapes_b::MeasurerComponent, __Context__, __Params__> for App
where
    shapes_b::MeasureWidthB: IsProviderFor<
        shapes_b::MeasurerComponent,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Context__,
    __Params__: ?Sized,
>: IsProviderFor<__Component__, __Context__, __Params__> {}
impl __CheckApp<shapes_a::MeasurerComponent, App, ()> for shapes_a::MeasureWidthA {}
fn main() {}
