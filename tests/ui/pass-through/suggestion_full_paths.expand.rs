#![feature(prelude_import)]
//! A `transform_map_fields` call whose transform and marker cannot be inferred.
//!
//! The error is plain Rust: type annotations are needed. The tool must print what plain
//! `cargo check` prints, including the short paths in rustc's fully qualified suggestion.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::traits::TransformMapFields;
use cgp::prelude::*;
pub struct Point {
    pub x: u64,
}
impl HasField<Symbol!("x")> for Point {
    type Value = u64;
    fn get_field(&self, key: ::core::marker::PhantomData<Symbol!("x")>) -> &Self::Value {
        &self.x
    }
}
impl HasFieldMut<Symbol!("x")> for Point {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("x")>,
    ) -> &mut Self::Value {
        &mut self.x
    }
}
impl HasFields for Point {
    type Fields = Struct! { x: u64 };
}
impl HasFieldsRef for Point {
    type FieldsRef<'__a> = Struct! { x: &'__a u64 }
    where
        Self: '__a;
}
impl FromFields for Point {
    fn from_fields(Cons(x, Nil): Self::Fields) -> Self {
        Self { x: x.value }
    }
}
impl ToFields for Point {
    fn to_fields(self) -> Self::Fields {
        Cons(::core::convert::Into::into(self.x), Nil)
    }
}
impl ToFieldsRef for Point {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons(::core::convert::Into::into(&self.x), Nil)
    }
}
///The partial builder for [`Point`].
pub struct __PartialPoint<__F0__: MapType> {
    pub x: <__F0__ as MapType>::Map<u64>,
}
impl HasBuilder for Point {
    type Builder = __PartialPoint<IsNothing>;
    fn builder() -> Self::Builder {
        __PartialPoint { x: () }
    }
}
impl IntoBuilder for Point {
    type Builder = __PartialPoint<IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialPoint { x: self.x }
    }
}
impl<__F0__: MapType> PartialData for __PartialPoint<__F0__> {
    type Target = Point;
}
impl FinalizeBuild for __PartialPoint<IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Point { x: self.x }
    }
}
impl<__M1__: MapType, __M2__: MapType> UpdateField<Symbol!("x"), __M2__>
for __PartialPoint<__M1__> {
    type Value = u64;
    type Mapper = __M1__;
    type Output = __PartialPoint<__M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("x")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (self.x, __PartialPoint { x: value })
    }
}
impl HasField<Symbol!("x")> for __PartialPoint<IsPresent> {
    type Value = u64;
    fn get_field(&self, tag: ::core::marker::PhantomData<Symbol!("x")>) -> &Self::Value {
        &self.x
    }
}
fn main() {
    let _ = Point { x: 1 }.into_builder().transform_map_fields();
}
