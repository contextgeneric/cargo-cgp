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

pub mod shape_a {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculatorA)]
    pub trait CanCalculateAreaA {
        fn area(&self) -> f64;
    }

    #[cgp_auto_getter]
    pub trait HasRectangleFieldsA {
        fn width(&self) -> f64;

        fn height(&self) -> f64;
    }

    #[cgp_impl(new RectangleAreaA)]
    impl AreaCalculatorA
    where
        Self: HasRectangleFieldsA,
    {
        fn area(&self) -> f64 {
            self.width() * self.height()
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: i32,
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorAComponent: RectangleAreaA,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorAComponent,
        }
    }
}

pub mod shape_b {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculatorB)]
    pub trait CanCalculateAreaB {
        fn area(&self) -> f64;
    }

    #[cgp_auto_getter]
    pub trait HasRectangleFieldsB {
        fn width(&self) -> f64;

        fn height(&self) -> f64;
    }

    #[cgp_impl(new RectangleAreaB)]
    impl AreaCalculatorB
    where
        Self: HasRectangleFieldsB,
    {
        fn area(&self) -> f64 {
            self.width() * self.height()
        }
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f64,
        pub height: i16,
    }

    delegate_components! {
        Rectangle {
            AreaCalculatorBComponent: RectangleAreaB,
        }
    }

    check_components! {
        Rectangle {
            AreaCalculatorBComponent,
        }
    }
}

fn main() {}
