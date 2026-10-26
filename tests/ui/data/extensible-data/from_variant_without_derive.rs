//! A type-relative `from_variant` call on an enum that derives `HasFields` but not `FromVariant`.
//!
//! `Shape` declares the variant `Circle`, but nothing implements `FromVariant` for it, so the cause
//! is the missing derive, not a missing variant: the tool reports that `Shape` does not implement
//! `FromVariant`, with a `help` naming `#[derive(FromVariant)]`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E123

use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

#[derive(HasFields)]
pub enum Shape {
    Circle(Circle),
}

fn main() {
    let _ = Shape::from_variant(PhantomData::<Symbol!("Circle")>, Circle { radius: 1.0 });
}
