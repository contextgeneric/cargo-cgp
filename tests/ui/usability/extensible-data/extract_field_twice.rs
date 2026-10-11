//! Extracting a variant again from the remainder that already ruled it out.
//!
//! With only the `Rectangle` impl left, rustc settles on it and reports the tag argument as an
//! `E0308` labelled "expected `9`, found `6`", the lengths of the two `Symbol` tags. The note below
//! reads `Symbol!("Rectangle")` against `Symbol!("Circle")`, but the label stays cryptic and nothing
//! says `Circle` was already extracted.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
    let shape = Shape::Circle(Circle { radius: 1.0 });
    if let Err(remainder) = shape.to_extractor().extract_field(PhantomData::<Symbol!("Circle")>) {
        let _ = remainder.extract_field(PhantomData::<Symbol!("Circle")>);
    }
}
