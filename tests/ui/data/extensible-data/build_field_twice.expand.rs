#![feature(prelude_import)]
//! Building the same record field twice, which the builder's type rejects.
//!
//! The second `build_field` for `first_name` fails on the `UpdateField` bound behind `BuildField`,
//! whose `Mapper` finds the field already set. The data-call anchor types the builder through the
//! chain of `builder()` and `build_field` calls and reads its presence markers:
//! ``[CGP-E050] field `first_name` of `Person` is already set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}
impl HasField<Symbol!("first_name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &Self::Value {
        &self.first_name
    }
}
impl HasFieldMut<Symbol!("first_name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &mut Self::Value {
        &mut self.first_name
    }
}
impl HasField<Symbol!("last_name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("last_name")>,
    ) -> &Self::Value {
        &self.last_name
    }
}
impl HasFieldMut<Symbol!("last_name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("last_name")>,
    ) -> &mut Self::Value {
        &mut self.last_name
    }
}
impl HasFields for Person {
    type Fields = Struct! { first_name: String, last_name: String };
}
impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Struct! { first_name: &'__a String, last_name: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Person {
    fn from_fields(Cons(first_name, Cons(last_name, Nil)): Self::Fields) -> Self {
        Self {
            first_name: first_name.value,
            last_name: last_name.value,
        }
    }
}
impl ToFields for Person {
    fn to_fields(self) -> Self::Fields {
        Cons(self.first_name.into(), Cons(self.last_name.into(), Nil))
    }
}
impl ToFieldsRef for Person {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.first_name).into(), Cons((&self.last_name).into(), Nil))
    }
}
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialPerson {
            first_name: (),
            last_name: (),
        }
    }
}
impl IntoBuilder for Person {
    type Builder = __PartialPerson<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialPerson {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialPerson<__F0__, __F1__> {
    type Target = Person;
}
impl FinalizeBuild for __PartialPerson<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Person {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("first_name"), __M2__> for __PartialPerson<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialPerson<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("first_name")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.first_name,
            __PartialPerson {
                first_name: value,
                last_name: self.last_name,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("last_name"), __M2__> for __PartialPerson<__F0__, __M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialPerson<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("last_name")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.last_name,
            __PartialPerson {
                first_name: self.first_name,
                last_name: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("first_name")>
for __PartialPerson<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &Self::Value {
        &self.first_name
    }
}
impl<__F0__: MapType> HasField<Symbol!("last_name")>
for __PartialPerson<__F0__, IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("last_name")>,
    ) -> &Self::Value {
        &self.last_name
    }
}
fn main() {
    let _ = Person::builder()
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
        .build_field(PhantomData::<Symbol!("first_name")>, "Bob".to_owned());
}
