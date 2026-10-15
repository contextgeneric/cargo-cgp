#![feature(prelude_import)]
//! Merging a source record with a field the target does not declare.
//!
//! `Person { first_name, surname }` merged into `Employee { first_name, last_name }` fails on the
//! private `FieldsBuilder` recursion over the source's fields, which the chain drops:
//! ``[CGP-E050] missing field `surname` on `Employee` ``, under the `BuildField` step that names
//! the field.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;
pub struct Person {
    pub first_name: String,
    pub surname: String,
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
impl HasField<Symbol!("surname")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("surname")>,
    ) -> &Self::Value {
        &self.surname
    }
}
impl HasFieldMut<Symbol!("surname")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("surname")>,
    ) -> &mut Self::Value {
        &mut self.surname
    }
}
impl HasFields for Person {
    type Fields = Struct! { first_name: String, surname: String };
}
impl HasFieldsRef for Person {
    type FieldsRef<'__a> = Struct! { first_name: &'__a String, surname: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Person {
    fn from_fields(Cons(first_name, Cons(surname, Nil)): Self::Fields) -> Self {
        Self {
            first_name: first_name.value,
            surname: surname.value,
        }
    }
}
impl ToFields for Person {
    fn to_fields(self) -> Self::Fields {
        Cons(self.first_name.into(), Cons(self.surname.into(), Nil))
    }
}
impl ToFieldsRef for Person {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.first_name).into(), Cons((&self.surname).into(), Nil))
    }
}
pub struct __PartialPerson<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub surname: <__F1__ as MapType>::Map<String>,
}
impl HasBuilder for Person {
    type Builder = __PartialPerson<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialPerson {
            first_name: (),
            surname: (),
        }
    }
}
impl IntoBuilder for Person {
    type Builder = __PartialPerson<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialPerson {
            first_name: self.first_name,
            surname: self.surname,
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
            surname: self.surname,
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
                surname: self.surname,
            },
        )
    }
}
impl<
    __F0__: MapType,
    __M1__: MapType,
    __M2__: MapType,
> UpdateField<Symbol!("surname"), __M2__> for __PartialPerson<__F0__, __M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialPerson<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("surname")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.surname,
            __PartialPerson {
                first_name: self.first_name,
                surname: value,
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
impl<__F0__: MapType> HasField<Symbol!("surname")>
for __PartialPerson<__F0__, IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("surname")>,
    ) -> &Self::Value {
        &self.surname
    }
}
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
}
impl HasField<Symbol!("first_name")> for Employee {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &Self::Value {
        &self.first_name
    }
}
impl HasFieldMut<Symbol!("first_name")> for Employee {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &mut Self::Value {
        &mut self.first_name
    }
}
impl HasField<Symbol!("last_name")> for Employee {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("last_name")>,
    ) -> &Self::Value {
        &self.last_name
    }
}
impl HasFieldMut<Symbol!("last_name")> for Employee {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("last_name")>,
    ) -> &mut Self::Value {
        &mut self.last_name
    }
}
impl HasFields for Employee {
    type Fields = Struct! { first_name: String, last_name: String };
}
impl HasFieldsRef for Employee {
    type FieldsRef<'__a> = Struct! { first_name: &'__a String, last_name: &'__a String }
    where
        Self: '__a;
}
impl FromFields for Employee {
    fn from_fields(Cons(first_name, Cons(last_name, Nil)): Self::Fields) -> Self {
        Self {
            first_name: first_name.value,
            last_name: last_name.value,
        }
    }
}
impl ToFields for Employee {
    fn to_fields(self) -> Self::Fields {
        Cons(self.first_name.into(), Cons(self.last_name.into(), Nil))
    }
}
impl ToFieldsRef for Employee {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.first_name).into(), Cons((&self.last_name).into(), Nil))
    }
}
pub struct __PartialEmployee<__F0__: MapType, __F1__: MapType> {
    pub first_name: <__F0__ as MapType>::Map<String>,
    pub last_name: <__F1__ as MapType>::Map<String>,
}
impl HasBuilder for Employee {
    type Builder = __PartialEmployee<IsNothing, IsNothing>;
    fn builder() -> Self::Builder {
        __PartialEmployee {
            first_name: (),
            last_name: (),
        }
    }
}
impl IntoBuilder for Employee {
    type Builder = __PartialEmployee<IsPresent, IsPresent>;
    fn into_builder(self) -> Self::Builder {
        __PartialEmployee {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> PartialData
for __PartialEmployee<__F0__, __F1__> {
    type Target = Employee;
}
impl FinalizeBuild for __PartialEmployee<IsPresent, IsPresent> {
    fn finalize_build(self) -> Self::Target {
        Employee {
            first_name: self.first_name,
            last_name: self.last_name,
        }
    }
}
impl<
    __M1__: MapType,
    __M2__: MapType,
    __F1__: MapType,
> UpdateField<Symbol!("first_name"), __M2__> for __PartialEmployee<__M1__, __F1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialEmployee<__M2__, __F1__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("first_name")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.first_name,
            __PartialEmployee {
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
> UpdateField<Symbol!("last_name"), __M2__> for __PartialEmployee<__F0__, __M1__> {
    type Value = String;
    type Mapper = __M1__;
    type Output = __PartialEmployee<__F0__, __M2__>;
    fn update_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("last_name")>,
        value: __M2__::Map<Self::Value>,
    ) -> (__M1__::Map<Self::Value>, Self::Output) {
        (
            self.last_name,
            __PartialEmployee {
                first_name: self.first_name,
                last_name: value,
            },
        )
    }
}
impl<__F1__: MapType> HasField<Symbol!("first_name")>
for __PartialEmployee<IsPresent, __F1__> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("first_name")>,
    ) -> &Self::Value {
        &self.first_name
    }
}
impl<__F0__: MapType> HasField<Symbol!("last_name")>
for __PartialEmployee<__F0__, IsPresent> {
    type Value = String;
    fn get_field(
        &self,
        tag: ::core::marker::PhantomData<Symbol!("last_name")>,
    ) -> &Self::Value {
        &self.last_name
    }
}
fn main() {
    let _: Employee = Employee::builder()
        .build_from(Person {
            first_name: "A".into(),
            surname: "B".into(),
        })
        .finalize_build();
}
