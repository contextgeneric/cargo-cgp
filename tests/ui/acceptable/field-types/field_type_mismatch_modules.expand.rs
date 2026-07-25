#![feature(prelude_import)]
//! Module-path struct resolution for the field-type mismatch: two *different* `Rectangle` contexts,
//! one per module, each derive `HasField` and each wire an area calculator whose `height()` getter
//! needs `f64` — but `shape_a::Rectangle` carries `height: i32` and `shape_b::Rectangle` carries
//! `height: i16`. Both fail with a `HasField<Symbol!("height")>::Value == f64` projection mismatch
//! (`E0271`), so the resolver rewrites each into its `[CGP-E003]` field-type-mismatch form.
//!
//! The point is that the *actual* field type in each message is read from the real struct by its
//! `DefId`, not by matching the bare name `Rectangle`: `shape_a`'s error must report the actual type
//! `i32` and `shape_b`'s must report `i16`, with no cross-over. A string-identifier lookup keyed on
//! `Rectangle` alone could not tell the two structs apart and would risk reporting one struct's field
//! type against the other's mismatch. The distinct `i32`/`i16` actual types are the discriminator
//! that proves the query is module-path based.
//!
//! The sibling of [`same_name_components`](same_name_components.rs), which proves the *component*
//! name map is full-path keyed; this proves the *struct* field query is `DefId`-anchored.
//!
//! See docs/implementation/typed-root-cause-resolution.md (field-type mismatch, and its
//! `DefId`-anchored struct query).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod shape_a {
    use cgp::prelude::*;
    pub trait CanCalculateAreaA {
        fn area(&self) -> f64;
    }
    impl<__Context__> CanCalculateAreaA for __Context__
    where
        __Context__: AreaCalculatorA<__Context__>,
    {
        fn area(&self) -> f64 {
            __Context__::area(self)
        }
    }
    pub trait AreaCalculatorA<
        __Context__,
    >: IsProviderFor<AreaCalculatorAComponent, __Context__, ()> {
        fn area(__context__: &__Context__) -> f64;
    }
    impl<__Provider__, __Context__> AreaCalculatorA<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<AreaCalculatorAComponent>
            + IsProviderFor<AreaCalculatorAComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            AreaCalculatorAComponent,
        >>::Delegate: AreaCalculatorA<__Context__>,
    {
        fn area(__context__: &__Context__) -> f64 {
            <__Provider__ as DelegateComponent<
                AreaCalculatorAComponent,
            >>::Delegate::area(__context__)
        }
    }
    pub struct AreaCalculatorAComponent;
    impl<__Context__> AreaCalculatorA<__Context__> for UseContext
    where
        __Context__: CanCalculateAreaA,
    {
        fn area(__context__: &__Context__) -> f64 {
            __Context__::area(__context__)
        }
    }
    impl<__Context__> IsProviderFor<AreaCalculatorAComponent, __Context__, ()>
    for UseContext
    where
        __Context__: CanCalculateAreaA,
    {}
    impl<__Context__, __Components__, __Path__> AreaCalculatorA<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: AreaCalculatorA<__Context__>,
    {
        fn area(__context__: &__Context__) -> f64 {
            <__Components__ as DelegateComponent<__Path__>>::Delegate::area(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<AreaCalculatorAComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<AreaCalculatorAComponent, __Context__, ()>
            + AreaCalculatorA<__Context__>,
    {}
    pub trait HasRectangleFieldsA {
        fn width(&self) -> f64;
        fn height(&self) -> f64;
    }
    impl<__Context__> HasRectangleFieldsA for __Context__
    where
        __Context__: HasField<Symbol!("width"), Value = f64>,
        __Context__: HasField<Symbol!("height"), Value = f64>,
    {
        fn width(&self) -> f64 {
            self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
        }
        fn height(&self) -> f64 {
            self.get_field(::core::marker::PhantomData::<Symbol!("height")>).clone()
        }
    }
    impl<__Context__> AreaCalculatorA<__Context__> for RectangleAreaA
    where
        __Context__: HasRectangleFieldsA,
    {
        fn area(__context__: &__Context__) -> f64 {
            __context__.width() * __context__.height()
        }
    }
    impl<__Context__> IsProviderFor<AreaCalculatorAComponent, __Context__, ()>
    for RectangleAreaA
    where
        __Context__: HasRectangleFieldsA,
    {}
    pub struct RectangleAreaA;
    pub struct Rectangle {
        pub width: f64,
        pub height: i32,
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
    impl HasField<Symbol!("height")> for Rectangle {
        type Value = i32;
        fn get_field(
            &self,
            key: ::core::marker::PhantomData<Symbol!("height")>,
        ) -> &Self::Value {
            &self.height
        }
    }
    impl HasFieldMut<Symbol!("height")> for Rectangle {
        fn get_field_mut(
            &mut self,
            key: ::core::marker::PhantomData<Symbol!("height")>,
        ) -> &mut Self::Value {
            &mut self.height
        }
    }
    impl DelegateComponent<AreaCalculatorAComponent> for Rectangle {
        type Delegate = RectangleAreaA;
    }
    impl<
        __Context__,
        __Params__,
    > IsProviderFor<AreaCalculatorAComponent, __Context__, __Params__> for Rectangle
    where
        RectangleAreaA: IsProviderFor<AreaCalculatorAComponent, __Context__, __Params__>,
    {}
    trait __CheckRectangle<
        __Component__,
        __Params__: ?Sized,
    >: CanUseComponent<__Component__, __Params__> {}
    impl __CheckRectangle<AreaCalculatorAComponent, ()> for Rectangle {}
}
pub mod shape_b {
    use cgp::prelude::*;
    pub trait CanCalculateAreaB {
        fn area(&self) -> f64;
    }
    impl<__Context__> CanCalculateAreaB for __Context__
    where
        __Context__: AreaCalculatorB<__Context__>,
    {
        fn area(&self) -> f64 {
            __Context__::area(self)
        }
    }
    pub trait AreaCalculatorB<
        __Context__,
    >: IsProviderFor<AreaCalculatorBComponent, __Context__, ()> {
        fn area(__context__: &__Context__) -> f64;
    }
    impl<__Provider__, __Context__> AreaCalculatorB<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<AreaCalculatorBComponent>
            + IsProviderFor<AreaCalculatorBComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            AreaCalculatorBComponent,
        >>::Delegate: AreaCalculatorB<__Context__>,
    {
        fn area(__context__: &__Context__) -> f64 {
            <__Provider__ as DelegateComponent<
                AreaCalculatorBComponent,
            >>::Delegate::area(__context__)
        }
    }
    pub struct AreaCalculatorBComponent;
    impl<__Context__> AreaCalculatorB<__Context__> for UseContext
    where
        __Context__: CanCalculateAreaB,
    {
        fn area(__context__: &__Context__) -> f64 {
            __Context__::area(__context__)
        }
    }
    impl<__Context__> IsProviderFor<AreaCalculatorBComponent, __Context__, ()>
    for UseContext
    where
        __Context__: CanCalculateAreaB,
    {}
    impl<__Context__, __Components__, __Path__> AreaCalculatorB<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: AreaCalculatorB<__Context__>,
    {
        fn area(__context__: &__Context__) -> f64 {
            <__Components__ as DelegateComponent<__Path__>>::Delegate::area(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<AreaCalculatorBComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<AreaCalculatorBComponent, __Context__, ()>
            + AreaCalculatorB<__Context__>,
    {}
    pub trait HasRectangleFieldsB {
        fn width(&self) -> f64;
        fn height(&self) -> f64;
    }
    impl<__Context__> HasRectangleFieldsB for __Context__
    where
        __Context__: HasField<Symbol!("width"), Value = f64>,
        __Context__: HasField<Symbol!("height"), Value = f64>,
    {
        fn width(&self) -> f64 {
            self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
        }
        fn height(&self) -> f64 {
            self.get_field(::core::marker::PhantomData::<Symbol!("height")>).clone()
        }
    }
    impl<__Context__> AreaCalculatorB<__Context__> for RectangleAreaB
    where
        __Context__: HasRectangleFieldsB,
    {
        fn area(__context__: &__Context__) -> f64 {
            __context__.width() * __context__.height()
        }
    }
    impl<__Context__> IsProviderFor<AreaCalculatorBComponent, __Context__, ()>
    for RectangleAreaB
    where
        __Context__: HasRectangleFieldsB,
    {}
    pub struct RectangleAreaB;
    pub struct Rectangle {
        pub width: f64,
        pub height: i16,
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
    impl HasField<Symbol!("height")> for Rectangle {
        type Value = i16;
        fn get_field(
            &self,
            key: ::core::marker::PhantomData<Symbol!("height")>,
        ) -> &Self::Value {
            &self.height
        }
    }
    impl HasFieldMut<Symbol!("height")> for Rectangle {
        fn get_field_mut(
            &mut self,
            key: ::core::marker::PhantomData<Symbol!("height")>,
        ) -> &mut Self::Value {
            &mut self.height
        }
    }
    impl DelegateComponent<AreaCalculatorBComponent> for Rectangle {
        type Delegate = RectangleAreaB;
    }
    impl<
        __Context__,
        __Params__,
    > IsProviderFor<AreaCalculatorBComponent, __Context__, __Params__> for Rectangle
    where
        RectangleAreaB: IsProviderFor<AreaCalculatorBComponent, __Context__, __Params__>,
    {}
    trait __CheckRectangle<
        __Component__,
        __Params__: ?Sized,
    >: CanUseComponent<__Component__, __Params__> {}
    impl __CheckRectangle<AreaCalculatorBComponent, ()> for Rectangle {}
}
fn main() {}
