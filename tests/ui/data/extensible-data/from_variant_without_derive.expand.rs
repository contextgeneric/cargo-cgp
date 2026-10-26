#![feature(prelude_import)]
//! A type-relative `from_variant` call on an enum that derives `HasFields` but not `FromVariant`.
//!
//! `Shape` declares the variant `Circle`, but nothing implements `FromVariant` for it, so the cause
//! is the missing derive, not a missing variant: the tool reports that `Shape` does not implement
//! `FromVariant`, with a `help` naming `#[derive(FromVariant)]`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Circle {
    pub radius: f64,
}
pub enum Shape {
    Circle(Circle),
}
impl HasFields for Shape {
    type Fields = Enum! { Circle(Circle) };
}
impl HasFieldsRef for Shape {
    type FieldsRef<'__a> = Enum! { Circle(&'__a Circle) }
    where
        Self: '__a;
}
impl FromFields for Shape {
    fn from_fields(rest: <Self as HasFields>::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Circle(field)
            }
            Either::Right(rest) => match rest {}
        }
    }
}
impl ToFields for Shape {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Circle(field) => Either::Left(field.into()),
        }
    }
}
impl ToFieldsRef for Shape {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Circle(field) => Either::Left(field.into()),
        }
    }
}
fn main() {
    let _ = Shape::from_variant(
        PhantomData::<Symbol!("Circle")>,
        Circle { radius: 1.0 },
    );
}
