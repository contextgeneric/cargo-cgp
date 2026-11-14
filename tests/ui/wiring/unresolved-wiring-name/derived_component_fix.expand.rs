#![feature(prelude_import)]
//! The attribute `[CGP-E070]` offers for a provider header whose derived component name is not
//! its component's, as the fix an editor or `cargo fix` applies.
//!
//! The fix adds the component to the header's attribute: `: Key` after the provider of
//! `#[cgp_impl(...)]`, and `(Key)` after `#[cgp_provider]`. A second, unchanged part spans the
//! trait's name, where the error points, so an editor offers the fix there. A key the module does
//! not import is written by its path, `crate::components::GreeterComponent`, since the attribute
//! names it where the header is. The fix is machine-applicable when the header writes an argument
//! for each of the key's parameters, and only maybe correct when it does not, as for
//! `AreaCalculatorComponent<Shape>` under `impl AreaCalculator`, where the parameter's name stands
//! in for the argument.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod components {
    use cgp::prelude::*;
    pub trait CanGreet {
        fn greet(&self) -> String;
    }
    impl<__Context__> CanGreet for __Context__
    where
        __Context__: Greeter<__Context__>,
    {
        fn greet(&self) -> String {
            __Context__::greet(self)
        }
    }
    ///Provider trait for [`CanGreet`]. A provider imports this trait's component key, [`GreeterComponent`], beside it.
    pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
        fn greet(__context__: &__Context__) -> String;
    }
    impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
    where
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
    ///Component marker for [`CanGreet`].
    pub struct GreeterComponent;
    impl<__Context__> Greeter<__Context__> for UseContext
    where
        __Context__: CanGreet,
    {
        fn greet(__context__: &__Context__) -> String {
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
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<GreeterComponent, __Context__, ()>
            + Greeter<__Context__>,
    {}
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }
    impl<__Context__, Shape> CanCalculateArea<Shape> for __Context__
    where
        __Context__: AreaCalculator<__Context__, Shape>,
    {
        fn area(&self, shape: &Shape) -> f64 {
            __Context__::area(self, shape)
        }
    }
    ///Provider trait for [`CanCalculateArea`]. A provider imports this trait's component key, [`AreaCalculatorComponent`], beside it.
    pub trait AreaCalculator<
        __Context__,
        Shape,
    >: IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)> {
        fn area(__context__: &__Context__, shape: &Shape) -> f64;
    }
    impl<__Provider__, __Context__, Shape> AreaCalculator<__Context__, Shape>
    for __Provider__
    where
        __Provider__: DelegateComponent<AreaCalculatorComponent<Shape>>
            + IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)>,
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent<Shape>,
        >>::Delegate: AreaCalculator<__Context__, Shape>,
    {
        fn area(__context__: &__Context__, shape: &Shape) -> f64 {
            <__Provider__ as DelegateComponent<
                AreaCalculatorComponent<Shape>,
            >>::Delegate::area(__context__, shape)
        }
    }
    ///Component marker for [`CanCalculateArea`].
    pub struct AreaCalculatorComponent<Shape>(pub ::core::marker::PhantomData<Shape>);
    impl<__Context__, Shape> AreaCalculator<__Context__, Shape> for UseContext
    where
        __Context__: CanCalculateArea<Shape>,
    {
        fn area(__context__: &__Context__, shape: &Shape) -> f64 {
            __Context__::area(__context__, shape)
        }
    }
    impl<
        __Context__,
        Shape,
    > IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)> for UseContext
    where
        __Context__: CanCalculateArea<Shape>,
    {}
    impl<__Context__, Shape, __Components__, __Path__> AreaCalculator<__Context__, Shape>
    for RedirectLookup<__Components__, __Path__>
    where
        __Path__: ConcatPath<Path!(@Shape)>,
        __Components__: DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >,
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >>::Delegate: AreaCalculator<__Context__, Shape>,
    {
        fn area(__context__: &__Context__, shape: &Shape) -> f64 {
            <__Components__ as DelegateComponent<
                <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
            >>::Delegate::area(__context__, shape)
        }
    }
    impl<
        __Context__,
        Shape,
        __Components__,
        __Path__,
    > IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)>
    for RedirectLookup<__Components__, __Path__>
    where
        __Path__: ConcatPath<Path!(@Shape)>,
        __Components__: DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >,
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >>::Delegate: IsProviderFor<AreaCalculatorComponent<Shape>, __Context__, (Shape)>
            + AreaCalculator<__Context__, Shape>,
    {}
}
pub mod renamed_without_key {
    use cgp::prelude::*;
    use crate::components::Greeter as G;
    impl<__Context__> G<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) -> String {
            "Hello".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<GComponent, __Context__, ()> for GreetHello {}
    ///Provider `GreetHello` for [`G`].
    pub struct GreetHello;
}
pub mod renamed_raw_provider {
    use cgp::prelude::*;
    use crate::components::{Greeter as G, GreeterComponent};
    pub struct GreetRaw;
    impl<Context> G<Context> for GreetRaw {
        fn greet(_context: &Context) -> String {
            "Hi".to_owned()
        }
    }
    impl<Context> IsProviderFor<GComponent, Context, ()> for GreetRaw {}
}
pub mod parameterized {
    use cgp::prelude::*;
    use crate::components::{AreaCalculator, AreaCalculatorComponent};
    pub struct Square(pub f64);
    impl<__Context__> AreaCalculator<__Context__, Square> for SquareArea {
        fn area(__context__: &__Context__, shape: &Square) -> f64 {
            shape.0 * shape.0
        }
    }
    impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, (Square)>
    for SquareArea {}
    ///Provider `SquareArea` for [`AreaCalculator`].
    pub struct SquareArea;
    impl<__Context__> AreaCalculator<__Context__> for AnyArea {
        fn area(__context__: &__Context__, _shape: &u32) -> f64 {
            1.0
        }
    }
    impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
    for AnyArea {}
    ///Provider `AnyArea` for [`AreaCalculator`].
    pub struct AnyArea;
}
fn main() {}
