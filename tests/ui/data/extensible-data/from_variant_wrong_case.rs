//! Constructing a variant through a tag with the wrong case.
//!
//! `Symbol!("circle")` for a `Circle` variant is an unsatisfied `FromVariant` bound, which rustc
//! reports twice, at two carets of one call; the tool merges the repeat, keeping its caret as a
//! secondary one labelled "also required here". The near-miss `Circle` appears only in the impl
//! list.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

fn main() {
    let _ = Shape::from_variant(PhantomData::<Symbol!("circle")>, Circle { radius: 1.0 });
}
