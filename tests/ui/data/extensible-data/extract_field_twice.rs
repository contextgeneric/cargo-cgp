//! Extracting a variant again from the remainder that already ruled it out.
//!
//! With only the `Rectangle` impl left, rustc settles on it and reports the tag argument as an
//! `E0308` labelled "expected `9`, found `6`". The data-call anchor types the remainder the
//! `Err(remainder)` pattern binds and reads its markers:
//! ``[CGP-E050] variant `Circle` of `Shape` is already extracted``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E124

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
