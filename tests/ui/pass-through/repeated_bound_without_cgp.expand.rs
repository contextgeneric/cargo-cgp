#![feature(prelude_import)]
//! A plain Rust call that fails one bound twice on one line, at the type and at the argument.
//!
//! `Shape::from_tag` needs `Shape: FromTag<CircleTag>`, which no impl gives, and its argument's
//! type `<Shape as FromTag<CircleTag>>::Value` needs the same bound, so rustc reports the `E0277`
//! twice on the call's line. In a crate that uses CGP the tool merges such a repeat into the first
//! block. This crate does not use CGP, so both blocks must stay exactly as rustc prints them.
//!
//! CGP error class: none; plain Rust the tool passes through.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::marker::PhantomData;
pub trait FromTag<Tag>: Sized {
    type Value;
    fn from_tag(tag: PhantomData<Tag>, value: Self::Value) -> Self;
}
pub struct Shape;
pub struct Square;
pub struct SquareTag;
pub struct Triangle;
pub struct TriangleTag;
pub struct Circle;
pub struct CircleTag;
impl FromTag<SquareTag> for Shape {
    type Value = Square;
    fn from_tag(_tag: PhantomData<SquareTag>, _value: Square) -> Self {
        Shape
    }
}
impl FromTag<TriangleTag> for Shape {
    type Value = Triangle;
    fn from_tag(_tag: PhantomData<TriangleTag>, _value: Triangle) -> Self {
        Shape
    }
}
fn main() {
    let _ = Shape::from_tag(PhantomData::<CircleTag>, Circle);
}
