#![feature(prelude_import)]
//! Merging a record whose source type lacks `#[derive(HasFields)]`.
//!
//! `CanBuildFrom` walks the source's field list, so the source needs `HasFields` as well as a builder.
//! Under the pinned nightly the output is only "`CanBuildFrom<FooBar>` is not implemented" for the
//! target's builder; nothing names `HasFields`, which the reference documents quote as the error.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/hidden-root-cause.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
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
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
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
    let _ = FooBarBaz::builder()
        .build_from(FooBar {
            foo: 1,
            bar: "bar".into(),
        })
        .build_field(PhantomData::<Symbol!("baz")>, true)
        .finalize_build();
}
