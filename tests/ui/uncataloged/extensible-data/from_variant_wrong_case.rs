//! Constructing a variant through a tag with the wrong case.
//!
//! `Symbol!("circle")` for a `Circle` variant is an unsatisfied `FromVariant` bound, which rustc
//! reports twice, at two carets of one call; the tool merges the repeat, keeping its caret as a
//! secondary one labelled "also required here". The near-miss `Circle` appears only in the impl
//! list.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
