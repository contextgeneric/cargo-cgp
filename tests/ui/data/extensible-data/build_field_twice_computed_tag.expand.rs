#![feature(prelude_import)]
//! Building a tuple struct's field twice, with the tag's index computed as `{ 0 + 1 }`.
//!
//! The data-call anchor reads a const argument only when it is a literal, bare or in braces, since
//! evaluating a constant from inside the emitter could report into the diagnostic it is rewriting.
//! So it cannot type this tag, declines, and the compiler's `E0271` on the builder's `Mapper`
//! stays, printed with the full types the tool's `--verbose` gives, where `Index<1>` writes the same
//! tag and reads as ``[CGP-E050] field `1` of `Pair` is already set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Pair(pub u32, pub u64);
impl HasField<Index<0>> for Pair {
    type Value = u32;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
impl HasFieldMut<Index<0>> for Pair {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<0>>,
    ) -> &mut Self::Value {
        &mut self.0
    }
}
impl HasField<Index<1>> for Pair {
    type Value = u64;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<1>>) -> &Self::Value {
        &self.1
    }
}
impl HasFieldMut<Index<1>> for Pair {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<1>>,
    ) -> &mut Self::Value {
        &mut self.1
    }
}
impl HasFields for Pair {
    type Fields = Struct!(u32, u64);
}
impl HasFieldsRef for Pair {
    type FieldsRef<'__a> = Struct!(&'__a u32, &'__a u64) where Self: '__a;
}
impl FromFields for Pair {
    fn from_fields(Cons(field_1, Cons(field_0, Nil)): Self::Fields) -> Self {
        Self(field_1.value, field_0.value)
    }
}
impl ToFields for Pair {
    fn to_fields(self) -> Self::Fields {
        Cons(self.0.into(), Cons(self.1.into(), Nil))
    }
}
impl ToFieldsRef for Pair {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.0).into(), Cons((&self.1).into(), Nil))
    }
}
pub struct __PartialPair<__F0__: MapType, __F1__: MapType>(
    pub <__F0__ as MapType>::Map<u32>,
    pub <__F1__ as MapType>::Map<u64>,
);
impl HasBuilder for Pair {
    type Builder = __PartialPair<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialPair { 0: (), 1: () }
    }
}
impl IntoBuilder for Pair {
    type Builder = __PartialPair<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialPair {
            0: self.0,
            1: self.1,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialPair<__F0__, __F1__> {
    type Target = Pair;
}
impl FinalizeBuild for __PartialPair<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Pair { 0: self.0, 1: self.1 }
    }
}
impl<__M1__: MapType, __M2__: MapType, __F1__: MapType> UpdateField<Index<0>, __M2__>
for __PartialPair<__M1__, __F1__> {
    type Value = u32;
    type Mapper = __M1__;
    type Output = __PartialPair<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Index<0>>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.0,
            __PartialPair {
                0: value,
                1: self.1,
            },
        )
    }
}
impl<__F0__: MapType, __M1__: MapType, __M2__: MapType> UpdateField<Index<1>, __M2__>
for __PartialPair<__F0__, __M1__> {
    type Value = u64;
    type Mapper = __M1__;
    type Output = __PartialPair<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Index<1>>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.1,
            __PartialPair {
                0: self.0,
                1: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Index<0>> for __PartialPair<IsPresent, __F1__> {
    type Value = u32;
    fn get_field(&self, tag: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
impl<__F0__: MapType> HasField<Index<1>> for __PartialPair<__F0__, IsPresent> {
    type Value = u64;
    fn get_field(&self, tag: ::core::marker::PhantomData<Index<1>>) -> &Self::Value {
        &self.1
    }
}
fn main() {
    let _ = Pair::builder()
        .build_field(PhantomData::<Index<{ 0 + 1 }>>, 3)
        .build_field(PhantomData::<Index<{ 0 + 1 }>>, 4);
}
