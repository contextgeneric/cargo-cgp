//! Extracting a variant through a tag with the wrong case.
//!
//! `Symbol!("circle")` for a `Circle` variant is an unsatisfied `ExtractField` bound on the enum's
//! extractor, which has no member of that name. The data-call anchor types the extractor through
//! `to_extractor` and reports ``[CGP-E050] missing variant `circle` on `Shape` ``, with a near-miss
//! `help` naming `Circle` from the extractor's enum.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E120

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
    let _ = shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("circle")>);
}
