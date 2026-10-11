//! Calling `finalize_extract_result` without importing `FinalizeExtractResult`.
//!
//! The error is plain Rust: the trait that provides the method is not in scope. The tool must print
//! what plain `cargo check` prints, short paths included (`Result<T, E>` in the headline).
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

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
