//! A hand-written `FromVariant` impl beside the derived one.
//!
//! The `E0119` is not a wiring conflict, so the conflict classifier leaves it as rustc wrote it, with
//! the caret on the `Circle` variant the derived impl is spanned at.
//!
//! CGP error class: none; ordinary coherence on the derived trait
//! [`FromVariant`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/traits/from_variant.md).

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

impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;
    fn from_variant(_tag: PhantomData<Symbol!("Circle")>, value: Circle) -> Self {
        Self::Circle(value)
    }
}

fn main() {}
