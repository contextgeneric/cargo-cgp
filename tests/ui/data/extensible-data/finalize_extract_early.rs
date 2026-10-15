//! Finalizing an extraction while a variant is still possible.
//!
//! `finalize_extract` on the remainder a `match` arm binds is an `E0599`, since only the
//! all-ruled-out extractor implements `FinalizeExtract`:
//! ``[CGP-E050] variant `Circle` of `Shape` is not extracted``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E121

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
    let _: f64 = match shape.to_extractor().extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => rect.width,
        Err(remainder) => remainder.finalize_extract(),
    };
}
