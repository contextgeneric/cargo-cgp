#![feature(prelude_import)]
//! Merging two source records that both carry the same field.
//!
//! The second `build_from` brings `bar` again, which the builder already holds. The output is an
//! `E0277` on `FieldsBuilder` over `Struct! { bar: String, baz: bool }`; it never says `bar` is set
//! twice. The reference documents quote an `E0271` on `Mapper == IsNothing` instead.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}
impl HasField<Symbol!("foo")> for FooBar {
    type Value = u64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &Self::Value {
        &self.foo
    }
}
impl HasFieldMut<Symbol!("foo")> for FooBar {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &mut Self::Value {
        &mut self.foo
    }
}
impl HasField<Symbol!("bar")> for FooBar {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
impl HasFieldMut<Symbol!("bar")> for FooBar {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &mut Self::Value {
        &mut self.bar
    }
}
impl HasFields for FooBar {
    type Fields = Struct! { foo: u64, bar: String };
}
impl HasFieldsRef for FooBar {
    type FieldsRef<'__a> = Struct! { foo: &'__a u64, bar: &'__a String }
    where
        Self: '__a;
}
impl FromFields for FooBar {
    fn from_fields(Cons(foo, Cons(bar, Nil)): Self::Fields) -> Self {
        Self {
            foo: foo.value,
            bar: bar.value,
        }
    }
}
impl ToFields for FooBar {
    fn to_fields(self) -> Self::Fields {
        Cons(self.foo.into(), Cons(self.bar.into(), Nil))
    }
}
impl ToFieldsRef for FooBar {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.foo).into(), Cons((&self.bar).into(), Nil))
    }
}
pub struct __PartialFooBar<__F0__: MapType, __F1__: MapType> {
    pub foo: <__F0__ as MapType>::Map<u64>,
    pub bar: <__F1__ as MapType>::Map<String>,
}
impl HasBuilder for FooBar {
    type Builder = __PartialFooBar<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialFooBar {
            foo: (),
            bar: (),
        }
    }
}
impl IntoBuilder for FooBar {
    type Builder = __PartialFooBar<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialFooBar {
            foo: self.foo,
            bar: self.bar,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialFooBar<__F0__, __F1__> {
    type Target = FooBar;
}
impl FinalizeBuild for __PartialFooBar<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        FooBar {
            foo: self.foo,
            bar: self.bar,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("foo"), __M2__> for __PartialFooBar<__M1__, __F1__> {
    type Value = u64;
    type Mapper = __M1__;
    type Output = __PartialFooBar<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("foo")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.foo,
            __PartialFooBar {
                foo: value,
                bar: self.bar,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("bar"), __M2__> for __PartialFooBar<__F0__, __M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialFooBar<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("bar")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.bar,
            __PartialFooBar {
                foo: self.foo,
                bar: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("foo")> for __PartialFooBar<IsPresent, __F1__> {
    type Value = u64;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &Self::Value {
        &self.foo
    }
}
impl<__F0__: MapType> HasField<Symbol!("bar")> for __PartialFooBar<__F0__, IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
pub struct BarBaz {
    pub bar: String,
    pub baz: bool,
}
impl HasField<Symbol!("bar")> for BarBaz {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
impl HasFieldMut<Symbol!("bar")> for BarBaz {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &mut Self::Value {
        &mut self.bar
    }
}
impl HasField<Symbol!("baz")> for BarBaz {
    type Value = bool;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("baz")>,
    ) -> &Self::Value {
        &self.baz
    }
}
impl HasFieldMut<Symbol!("baz")> for BarBaz {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("baz")>,
    ) -> &mut Self::Value {
        &mut self.baz
    }
}
impl HasFields for BarBaz {
    type Fields = Struct! { bar: String, baz: bool };
}
impl HasFieldsRef for BarBaz {
    type FieldsRef<'__a> = Struct! { bar: &'__a String, baz: &'__a bool }
    where
        Self: '__a;
}
impl FromFields for BarBaz {
    fn from_fields(Cons(bar, Cons(baz, Nil)): Self::Fields) -> Self {
        Self {
            bar: bar.value,
            baz: baz.value,
        }
    }
}
impl ToFields for BarBaz {
    fn to_fields(self) -> Self::Fields {
        Cons(self.bar.into(), Cons(self.baz.into(), Nil))
    }
}
impl ToFieldsRef for BarBaz {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.bar).into(), Cons((&self.baz).into(), Nil))
    }
}
pub struct __PartialBarBaz<__F0__: MapType, __F1__: MapType> {
    pub bar: <__F0__ as MapType>::Map<String>,
    pub baz: <__F1__ as MapType>::Map<bool>,
}
impl HasBuilder for BarBaz {
    type Builder = __PartialBarBaz<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialBarBaz {
            bar: (),
            baz: (),
        }
    }
}
impl IntoBuilder for BarBaz {
    type Builder = __PartialBarBaz<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialBarBaz {
            bar: self.bar,
            baz: self.baz,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialBarBaz<__F0__, __F1__> {
    type Target = BarBaz;
}
impl FinalizeBuild for __PartialBarBaz<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        BarBaz {
            bar: self.bar,
            baz: self.baz,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("bar"), __M2__> for __PartialBarBaz<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialBarBaz<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("bar")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.bar,
            __PartialBarBaz {
                bar: value,
                baz: self.baz,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("baz"), __M2__> for __PartialBarBaz<__F0__, __M1__> {
    type Value = bool;
    type Mapper = __M1__;
    type Output = __PartialBarBaz<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("baz")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.baz,
            __PartialBarBaz {
                bar: self.bar,
                baz: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("bar")> for __PartialBarBaz<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
impl<__F0__: MapType> HasField<Symbol!("baz")> for __PartialBarBaz<__F0__, IsPresent> {
    type Value = bool;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("baz")>,
    ) -> &Self::Value {
        &self.baz
    }
}
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}
impl HasField<Symbol!("foo")> for FooBarBaz {
    type Value = u64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &Self::Value {
        &self.foo
    }
}
impl HasFieldMut<Symbol!("foo")> for FooBarBaz {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &mut Self::Value {
        &mut self.foo
    }
}
impl HasField<Symbol!("bar")> for FooBarBaz {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
impl HasFieldMut<Symbol!("bar")> for FooBarBaz {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &mut Self::Value {
        &mut self.bar
    }
}
impl HasField<Symbol!("baz")> for FooBarBaz {
    type Value = bool;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("baz")>,
    ) -> &Self::Value {
        &self.baz
    }
}
impl HasFieldMut<Symbol!("baz")> for FooBarBaz {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("baz")>,
    ) -> &mut Self::Value {
        &mut self.baz
    }
}
impl HasFields for FooBarBaz {
    type Fields = Struct! { foo: u64, bar: String, baz: bool };
}
impl HasFieldsRef for FooBarBaz {
    type FieldsRef<'__a> = Struct! { foo: &'__a u64, bar: &'__a String, baz: &'__a bool }
    where
        Self: '__a;
}
impl FromFields for FooBarBaz {
    fn from_fields(Cons(foo, Cons(bar, Cons(baz, Nil))): Self::Fields) -> Self {
        Self {
            foo: foo.value,
            bar: bar.value,
            baz: baz.value,
        }
    }
}
impl ToFields for FooBarBaz {
    fn to_fields(self) -> Self::Fields {
        Cons(self.foo.into(), Cons(self.bar.into(), Cons(self.baz.into(), Nil)))
    }
}
impl ToFieldsRef for FooBarBaz {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.foo).into(), Cons((&self.bar).into(), Cons((&self.baz).into(), Nil)))
    }
}
pub struct __PartialFooBarBaz<__F0__: MapType, __F1__: MapType, __F2__: MapType> {
    pub foo: <__F0__ as MapType>::Map<u64>,
    pub bar: <__F1__ as MapType>::Map<String>,
    pub baz: <__F2__ as MapType>::Map<bool>,
}
impl HasBuilder for FooBarBaz {
    type Builder = __PartialFooBarBaz<IsNothing, IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialFooBarBaz {
            foo: (),
            bar: (),
            baz: (),
        }
    }
}
impl IntoBuilder for FooBarBaz {
    type Builder = __PartialFooBarBaz<IsPresent, IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialFooBarBaz {
            foo: self.foo,
            bar: self.bar,
            baz: self.baz,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType, __F2__: MapType> PartialData
for __PartialFooBarBaz<__F0__, __F1__, __F2__> {
    type Target = FooBarBaz;
}
impl FinalizeBuild for __PartialFooBarBaz<IsPresent, IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        FooBarBaz {
            foo: self.foo,
            bar: self.bar,
            baz: self.baz,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
    __F2__: MapType,
> UpdateField<Symbol!("foo"), __M2__> for __PartialFooBarBaz<__M1__, __F1__, __F2__> {
    type Value = u64;
    type Mapper = __M1__;
    type Output = __PartialFooBarBaz<__M2__, __F1__, __F2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("foo")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.foo,
            __PartialFooBarBaz {
                foo: value,
                bar: self.bar,
                baz: self.baz,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
    __F2__: MapType,
> UpdateField<Symbol!("bar"), __M2__> for __PartialFooBarBaz<__F0__, __M1__, __F2__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialFooBarBaz<__F0__, __M2__, __F2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("bar")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.bar,
            __PartialFooBarBaz {
                foo: self.foo,
                bar: value,
                baz: self.baz,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("baz"), __M2__> for __PartialFooBarBaz<__F0__, __F1__, __M1__> {
    type Value = bool;
    type Mapper = __M1__;
    type Output = __PartialFooBarBaz<__F0__, __F1__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("baz")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.baz,
            __PartialFooBarBaz {
                foo: self.foo,
                bar: self.bar,
                baz: value,
            },
        )
    }
}
impl<__F1__: MapType, __F2__: MapType> HasField<Symbol!("foo")>
for __PartialFooBarBaz<IsPresent, __F1__, __F2__> {
    type Value = u64;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &Self::Value {
        &self.foo
    }
}
impl<__F0__: MapType, __F2__: MapType> HasField<Symbol!("bar")>
for __PartialFooBarBaz<__F0__, IsPresent, __F2__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
impl<__F0__: MapType, __F1__: MapType> HasField<Symbol!("baz")>
for __PartialFooBarBaz<__F0__, __F1__, IsPresent> {
    type Value = bool;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("baz")>,
    ) -> &Self::Value {
        &self.baz
    }
}
fn main() {
    let _: FooBarBaz = FooBarBaz::builder()
        .build_from(FooBar {
            foo: 1,
            bar: "bar".into(),
        })
        .build_from(BarBaz {
            bar: "bar".into(),
            baz: true,
        })
        .finalize_build();
}
