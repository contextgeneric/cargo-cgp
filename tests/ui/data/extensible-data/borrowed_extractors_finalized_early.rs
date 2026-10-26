//! A borrowed extractor finalized before every variant is extracted, by reference and by mutable
//! reference.
//!
//! Each match extracts `Rectangle` and finalizes the remainder, which may still hold `Circle`. The
//! two failures share one cause and merge into one block, whose chain names each extractor by the
//! borrow it holds: the reference extractor and the mutable reference extractor of `Shape`.
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

pub fn by_reference(shape: &Shape) {
    let _: f64 = match shape.extractor_ref().extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => rect.width,
        Err(remainder) => remainder.finalize_extract(),
    };
}

pub fn by_mutable_reference(shape: &mut Shape) {
    let _: f64 = match shape.extractor_mut().extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => rect.width,
        Err(remainder) => remainder.finalize_extract(),
    };
}

fn main() {}
