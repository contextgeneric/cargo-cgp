#![feature(prelude_import)]
//! Calling the optional builder's `set` on a core builder.
//!
//! `Context::builder().set(…)` fails on `Mapper == IsOptional`, since a core builder holds `foo`
//! unset rather than optional: ``[CGP-E050] field `foo` of `Context` is not optional``. The `help`
//! names the conversion with the import its trait needs, since rustc's own import suggestion names
//! a path that does not compile.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::field::impls::SetOptional;
use cgp::prelude::*;
pub struct Context {
    pub foo: String,
    pub bar: u64,
}
impl HasField<Symbol!("foo")> for Context {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &Self::Value {
        &self.foo
    }
}
impl HasFieldMut<Symbol!("foo")> for Context {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &mut Self::Value {
        &mut self.foo
    }
}
impl HasField<Symbol!("bar")> for Context {
    type Value = u64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
impl HasFieldMut<Symbol!("bar")> for Context {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &mut Self::Value {
        &mut self.bar
    }
}
impl HasFields for Context {
    type Fields = Struct! { foo: String, bar: u64 };
}
impl HasFieldsRef for Context {
    type FieldsRef<'__a> = Struct! { foo: &'__a String, bar: &'__a u64 }
    where
        Self: '__a;
}
impl FromFields for Context {
    fn from_fields(Cons(foo, Cons(bar, Nil)): Self::Fields) -> Self {
        Self {
            foo: foo.value,
            bar: bar.value,
        }
    }
}
impl ToFields for Context {
    fn to_fields(self) -> Self::Fields {
        Cons(
            ::core::convert::Into::into(self.foo),
            Cons(::core::convert::Into::into(self.bar), Nil),
        )
    }
}
impl ToFieldsRef for Context {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons(
            ::core::convert::Into::into(&self.foo),
            Cons(::core::convert::Into::into(&self.bar), Nil),
        )
    }
}
///The partial builder for [`Context`].
pub struct __PartialContext<__F0__: MapType, __F1__: MapType> {
    pub foo: <__F0__ as MapType>::Map<String>,
    pub bar: <__F1__ as MapType>::Map<u64>,
}
impl HasBuilder for Context {
    type Builder = __PartialContext<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialContext {
            foo: (),
            bar: (),
        }
    }
}
impl IntoBuilder for Context {
    type Builder = __PartialContext<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialContext {
            foo: self.foo,
            bar: self.bar,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialContext<__F0__, __F1__> {
    type Target = Context;
}
impl FinalizeBuild for __PartialContext<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Context {
            foo: self.foo,
            bar: self.bar,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("foo"), __M2__> for __PartialContext<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialContext<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("foo")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.foo,
            __PartialContext {
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
> UpdateField<Symbol!("bar"), __M2__> for __PartialContext<__F0__, __M1__> {
    type Value = u64;
    type Mapper = __M1__;
    type Output = __PartialContext<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("bar")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.bar,
            __PartialContext {
                foo: self.foo,
                bar: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("foo")> for __PartialContext<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("foo")>,
    ) -> &Self::Value {
        &self.foo
    }
}
impl<__F0__: MapType> HasField<Symbol!("bar")> for __PartialContext<__F0__, IsPresent> {
    type Value = u64;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("bar")>,
    ) -> &Self::Value {
        &self.bar
    }
}
fn main() {
    let _ = Context::builder().set(PhantomData::<Symbol!("foo")>, "foo".to_owned());
}
