//! Finalizing an extraction while a variant is still possible.
//!
//! `finalize_extract` on `__PartialShape<IsPresent, IsVoid>` is an `E0599` with a stray note that the
//! method exists on `<IsVoid as MapType>::Map<Rectangle>`. The unhandled `Circle` is never named.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
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
    let shape = Shape::Circle(Circle { radius: 1.0 });
    let _: f64 = match shape.to_extractor().extract_field(PhantomData::<Symbol!("Rectangle")>) {
        Ok(rect) => rect.width,
        Err(remainder) => remainder.finalize_extract(),
    };
}
