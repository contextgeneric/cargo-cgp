//! Calling `finalize_extract_result` without importing `FinalizeExtractResult`.
//!
//! The headline names `std::result::Result<T, E>` under the tool, where plain `cargo check` prints
//! `Result<T, E>`.
//!
//! CGP error class: none; a driver path-printing defect affecting every untransformed diagnostic.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@rewrites-falsely: untransformed output prints full paths, because the driver does not set `trimmed_def_paths`

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
        let _ = remainder
            .extract_field(PhantomData::<Symbol!("Rectangle")>)
            .finalize_extract_result();
    }
}
