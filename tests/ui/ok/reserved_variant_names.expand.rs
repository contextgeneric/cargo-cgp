#![feature(prelude_import)]
//! Clean compile: an enum whose variants are named after the associated types the variant derives
//! implement, under `#[derive(CgpData)]`.
//!
//! The derives name `Value`, `Remainder`, `Extractor`, `ExtractorRef`, `ExtractorMut`, `Fields`, and
//! `FieldsRef` through qualified projections such as `<Self as ExtractField<Tag>>::Value`, which the
//! expansion shows, so a variant with any of those names cannot make a generated path ambiguous.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub enum Token {
    Value(u64),
    Remainder(u64),
    Extractor(String),
    ExtractorRef(String),
    ExtractorMut(String),
    Fields(bool),
    FieldsRef(char),
}
impl HasFields for Token {
    type Fields = Enum! { Value(u64), Remainder(u64), Extractor(String), ExtractorRef(String), ExtractorMut(String), Fields(bool), FieldsRef(char) };
}
impl HasFieldsRef for Token {
    type FieldsRef<'__a> = Enum! { Value(&'__a u64), Remainder(&'__a u64), Extractor(&'__a String), ExtractorRef(&'__a String), ExtractorMut(&'__a String), Fields(&'__a bool), FieldsRef(&'__a char) }
    where
        Self: '__a;
}
impl FromFields for Token {
    fn from_fields(rest: <Self as HasFields>::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Value(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Remainder(field)
                    }
                    Either::Right(rest) => {
                        match rest {
                            Either::Left(field) => {
                                let field = field.value;
                                Self::Extractor(field)
                            }
                            Either::Right(rest) => {
                                match rest {
                                    Either::Left(field) => {
                                        let field = field.value;
                                        Self::ExtractorRef(field)
                                    }
                                    Either::Right(rest) => {
                                        match rest {
                                            Either::Left(field) => {
                                                let field = field.value;
                                                Self::ExtractorMut(field)
                                            }
                                            Either::Right(rest) => {
                                                match rest {
                                                    Either::Left(field) => {
                                                        let field = field.value;
                                                        Self::Fields(field)
                                                    }
                                                    Either::Right(rest) => {
                                                        match rest {
                                                            Either::Left(field) => {
                                                                let field = field.value;
                                                                Self::FieldsRef(field)
                                                            }
                                                            Either::Right(rest) => match rest {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
impl ToFields for Token {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Value(field) => Either::Left(field.into()),
            Self::Remainder(field) => Either::Right(Either::Left(field.into())),
            Self::Extractor(field) => {
                Either::Right(Either::Right(Either::Left(field.into())))
            }
            Self::ExtractorRef(field) => {
                Either::Right(Either::Right(Either::Right(Either::Left(field.into()))))
            }
            Self::ExtractorMut(field) => {
                Either::Right(
                    Either::Right(
                        Either::Right(Either::Right(Either::Left(field.into()))),
                    ),
                )
            }
            Self::Fields(field) => {
                Either::Right(
                    Either::Right(
                        Either::Right(
                            Either::Right(Either::Right(Either::Left(field.into()))),
                        ),
                    ),
                )
            }
            Self::FieldsRef(field) => {
                Either::Right(
                    Either::Right(
                        Either::Right(
                            Either::Right(
                                Either::Right(Either::Right(Either::Left(field.into()))),
                            ),
                        ),
                    ),
                )
            }
        }
    }
}
impl ToFieldsRef for Token {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Value(field) => Either::Left(field.into()),
            Self::Remainder(field) => Either::Right(Either::Left(field.into())),
            Self::Extractor(field) => {
                Either::Right(Either::Right(Either::Left(field.into())))
            }
            Self::ExtractorRef(field) => {
                Either::Right(Either::Right(Either::Right(Either::Left(field.into()))))
            }
            Self::ExtractorMut(field) => {
                Either::Right(
                    Either::Right(
                        Either::Right(Either::Right(Either::Left(field.into()))),
                    ),
                )
            }
            Self::Fields(field) => {
                Either::Right(
                    Either::Right(
                        Either::Right(
                            Either::Right(Either::Right(Either::Left(field.into()))),
                        ),
                    ),
                )
            }
            Self::FieldsRef(field) => {
                Either::Right(
                    Either::Right(
                        Either::Right(
                            Either::Right(
                                Either::Right(Either::Right(Either::Left(field.into()))),
                            ),
                        ),
                    ),
                )
            }
        }
    }
}
impl FromVariant<Symbol!("Value")> for Token {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Value")>,
        value: <Self as FromVariant<Symbol!("Value")>>::Value,
    ) -> Self {
        Self::Value(value)
    }
}
impl FromVariant<Symbol!("Remainder")> for Token {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Remainder")>,
        value: <Self as FromVariant<Symbol!("Remainder")>>::Value,
    ) -> Self {
        Self::Remainder(value)
    }
}
impl FromVariant<Symbol!("Extractor")> for Token {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Extractor")>,
        value: <Self as FromVariant<Symbol!("Extractor")>>::Value,
    ) -> Self {
        Self::Extractor(value)
    }
}
impl FromVariant<Symbol!("ExtractorRef")> for Token {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("ExtractorRef")>,
        value: <Self as FromVariant<Symbol!("ExtractorRef")>>::Value,
    ) -> Self {
        Self::ExtractorRef(value)
    }
}
impl FromVariant<Symbol!("ExtractorMut")> for Token {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("ExtractorMut")>,
        value: <Self as FromVariant<Symbol!("ExtractorMut")>>::Value,
    ) -> Self {
        Self::ExtractorMut(value)
    }
}
impl FromVariant<Symbol!("Fields")> for Token {
    type Value = bool;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Fields")>,
        value: <Self as FromVariant<Symbol!("Fields")>>::Value,
    ) -> Self {
        Self::Fields(value)
    }
}
impl FromVariant<Symbol!("FieldsRef")> for Token {
    type Value = char;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("FieldsRef")>,
        value: <Self as FromVariant<Symbol!("FieldsRef")>>::Value,
    ) -> Self {
        Self::FieldsRef(value)
    }
}
pub enum __PartialToken<
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> {
    Value(<__F0__ as MapType>::Map<u64>),
    Remainder(<__F1__ as MapType>::Map<u64>),
    Extractor(<__F2__ as MapType>::Map<String>),
    ExtractorRef(<__F3__ as MapType>::Map<String>),
    ExtractorMut(<__F4__ as MapType>::Map<String>),
    Fields(<__F5__ as MapType>::Map<bool>),
    FieldsRef(<__F6__ as MapType>::Map<char>),
}
pub enum __PartialRefToken<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> {
    Value(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Remainder(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Extractor(<__F2__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
    ExtractorRef(<__F3__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
    ExtractorMut(<__F4__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
    Fields(<__F5__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, bool>>),
    FieldsRef(<__F6__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, char>>),
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> PartialData
for __PartialToken<__F0__, __F1__, __F2__, __F3__, __F4__, __F5__, __F6__> {
    type Target = Token;
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> PartialData
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    __F1__,
    __F2__,
    __F3__,
    __F4__,
    __F5__,
    __F6__,
> {
    type Target = Token;
}
impl HasExtractor for Token {
    type Extractor = __PartialToken<
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
    >;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Value(value) => __PartialToken::Value(value),
            Self::Remainder(value) => __PartialToken::Remainder(value),
            Self::Extractor(value) => __PartialToken::Extractor(value),
            Self::ExtractorRef(value) => __PartialToken::ExtractorRef(value),
            Self::ExtractorMut(value) => __PartialToken::ExtractorMut(value),
            Self::Fields(value) => __PartialToken::Fields(value),
            Self::FieldsRef(value) => __PartialToken::FieldsRef(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialToken::Value(value) => Self::Value(value),
            __PartialToken::Remainder(value) => Self::Remainder(value),
            __PartialToken::Extractor(value) => Self::Extractor(value),
            __PartialToken::ExtractorRef(value) => Self::ExtractorRef(value),
            __PartialToken::ExtractorMut(value) => Self::ExtractorMut(value),
            __PartialToken::Fields(value) => Self::Fields(value),
            __PartialToken::FieldsRef(value) => Self::FieldsRef(value),
        }
    }
}
impl HasExtractorRef for Token {
    type ExtractorRef<'__a__> = __PartialRefToken<
        '__a__,
        IsRef,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
    >
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Value(value) => __PartialRefToken::Value(value),
            Self::Remainder(value) => __PartialRefToken::Remainder(value),
            Self::Extractor(value) => __PartialRefToken::Extractor(value),
            Self::ExtractorRef(value) => __PartialRefToken::ExtractorRef(value),
            Self::ExtractorMut(value) => __PartialRefToken::ExtractorMut(value),
            Self::Fields(value) => __PartialRefToken::Fields(value),
            Self::FieldsRef(value) => __PartialRefToken::FieldsRef(value),
        }
    }
}
impl HasExtractorMut for Token {
    type ExtractorMut<'__a__> = __PartialRefToken<
        '__a__,
        IsMut,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
        IsPresent,
    >
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Value(value) => __PartialRefToken::Value(value),
            Self::Remainder(value) => __PartialRefToken::Remainder(value),
            Self::Extractor(value) => __PartialRefToken::Extractor(value),
            Self::ExtractorRef(value) => __PartialRefToken::ExtractorRef(value),
            Self::ExtractorMut(value) => __PartialRefToken::ExtractorMut(value),
            Self::Fields(value) => __PartialRefToken::Fields(value),
            Self::FieldsRef(value) => __PartialRefToken::FieldsRef(value),
        }
    }
}
impl FinalizeExtract
for __PartialToken<IsVoid, IsVoid, IsVoid, IsVoid, IsVoid, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefToken<
    '__a__,
    __R__,
    IsVoid,
    IsVoid,
    IsVoid,
    IsVoid,
    IsVoid,
    IsVoid,
    IsVoid,
> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Value")>
for __PartialToken<IsPresent, __F1__, __F2__, __F3__, __F4__, __F5__, __F6__> {
    type Value = u64;
    type Remainder = __PartialToken<
        IsVoid,
        __F1__,
        __F2__,
        __F3__,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Value")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Value")>>::Value,
        <Self as ExtractField<Symbol!("Value")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Ok(value),
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Extractor(value) => Err(__PartialToken::Extractor(value)),
            __PartialToken::ExtractorRef(value) => {
                Err(__PartialToken::ExtractorRef(value))
            }
            __PartialToken::ExtractorMut(value) => {
                Err(__PartialToken::ExtractorMut(value))
            }
            __PartialToken::Fields(value) => Err(__PartialToken::Fields(value)),
            __PartialToken::FieldsRef(value) => Err(__PartialToken::FieldsRef(value)),
        }
    }
}
impl<
    __F0__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Remainder")>
for __PartialToken<__F0__, IsPresent, __F2__, __F3__, __F4__, __F5__, __F6__> {
    type Value = u64;
    type Remainder = __PartialToken<
        __F0__,
        IsVoid,
        __F2__,
        __F3__,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Remainder")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Remainder")>>::Value,
        <Self as ExtractField<Symbol!("Remainder")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Remainder(value) => Ok(value),
            __PartialToken::Extractor(value) => Err(__PartialToken::Extractor(value)),
            __PartialToken::ExtractorRef(value) => {
                Err(__PartialToken::ExtractorRef(value))
            }
            __PartialToken::ExtractorMut(value) => {
                Err(__PartialToken::ExtractorMut(value))
            }
            __PartialToken::Fields(value) => Err(__PartialToken::Fields(value)),
            __PartialToken::FieldsRef(value) => Err(__PartialToken::FieldsRef(value)),
        }
    }
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Extractor")>
for __PartialToken<__F0__, __F1__, IsPresent, __F3__, __F4__, __F5__, __F6__> {
    type Value = String;
    type Remainder = __PartialToken<
        __F0__,
        __F1__,
        IsVoid,
        __F3__,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Extractor")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Extractor")>>::Value,
        <Self as ExtractField<Symbol!("Extractor")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Extractor(value) => Ok(value),
            __PartialToken::ExtractorRef(value) => {
                Err(__PartialToken::ExtractorRef(value))
            }
            __PartialToken::ExtractorMut(value) => {
                Err(__PartialToken::ExtractorMut(value))
            }
            __PartialToken::Fields(value) => Err(__PartialToken::Fields(value)),
            __PartialToken::FieldsRef(value) => Err(__PartialToken::FieldsRef(value)),
        }
    }
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("ExtractorRef")>
for __PartialToken<__F0__, __F1__, __F2__, IsPresent, __F4__, __F5__, __F6__> {
    type Value = String;
    type Remainder = __PartialToken<
        __F0__,
        __F1__,
        __F2__,
        IsVoid,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("ExtractorRef")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("ExtractorRef")>>::Value,
        <Self as ExtractField<Symbol!("ExtractorRef")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Extractor(value) => Err(__PartialToken::Extractor(value)),
            __PartialToken::ExtractorRef(value) => Ok(value),
            __PartialToken::ExtractorMut(value) => {
                Err(__PartialToken::ExtractorMut(value))
            }
            __PartialToken::Fields(value) => Err(__PartialToken::Fields(value)),
            __PartialToken::FieldsRef(value) => Err(__PartialToken::FieldsRef(value)),
        }
    }
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("ExtractorMut")>
for __PartialToken<__F0__, __F1__, __F2__, __F3__, IsPresent, __F5__, __F6__> {
    type Value = String;
    type Remainder = __PartialToken<
        __F0__,
        __F1__,
        __F2__,
        __F3__,
        IsVoid,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("ExtractorMut")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("ExtractorMut")>>::Value,
        <Self as ExtractField<Symbol!("ExtractorMut")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Extractor(value) => Err(__PartialToken::Extractor(value)),
            __PartialToken::ExtractorRef(value) => {
                Err(__PartialToken::ExtractorRef(value))
            }
            __PartialToken::ExtractorMut(value) => Ok(value),
            __PartialToken::Fields(value) => Err(__PartialToken::Fields(value)),
            __PartialToken::FieldsRef(value) => Err(__PartialToken::FieldsRef(value)),
        }
    }
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Fields")>
for __PartialToken<__F0__, __F1__, __F2__, __F3__, __F4__, IsPresent, __F6__> {
    type Value = bool;
    type Remainder = __PartialToken<
        __F0__,
        __F1__,
        __F2__,
        __F3__,
        __F4__,
        IsVoid,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Fields")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Fields")>>::Value,
        <Self as ExtractField<Symbol!("Fields")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Extractor(value) => Err(__PartialToken::Extractor(value)),
            __PartialToken::ExtractorRef(value) => {
                Err(__PartialToken::ExtractorRef(value))
            }
            __PartialToken::ExtractorMut(value) => {
                Err(__PartialToken::ExtractorMut(value))
            }
            __PartialToken::Fields(value) => Ok(value),
            __PartialToken::FieldsRef(value) => Err(__PartialToken::FieldsRef(value)),
        }
    }
}
impl<
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
> ExtractField<Symbol!("FieldsRef")>
for __PartialToken<__F0__, __F1__, __F2__, __F3__, __F4__, __F5__, IsPresent> {
    type Value = char;
    type Remainder = __PartialToken<
        __F0__,
        __F1__,
        __F2__,
        __F3__,
        __F4__,
        __F5__,
        IsVoid,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("FieldsRef")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("FieldsRef")>>::Value,
        <Self as ExtractField<Symbol!("FieldsRef")>>::Remainder,
    > {
        match self {
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Extractor(value) => Err(__PartialToken::Extractor(value)),
            __PartialToken::ExtractorRef(value) => {
                Err(__PartialToken::ExtractorRef(value))
            }
            __PartialToken::ExtractorMut(value) => {
                Err(__PartialToken::ExtractorMut(value))
            }
            __PartialToken::Fields(value) => Err(__PartialToken::Fields(value)),
            __PartialToken::FieldsRef(value) => Ok(value),
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Value")>
for __PartialRefToken<
    '__a__,
    __R__,
    IsPresent,
    __F1__,
    __F2__,
    __F3__,
    __F4__,
    __F5__,
    __F6__,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        IsVoid,
        __F1__,
        __F2__,
        __F3__,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Value")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Value")>>::Value,
        <Self as ExtractField<Symbol!("Value")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Ok(value),
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Extractor(value) => {
                Err(__PartialRefToken::Extractor(value))
            }
            __PartialRefToken::ExtractorRef(value) => {
                Err(__PartialRefToken::ExtractorRef(value))
            }
            __PartialRefToken::ExtractorMut(value) => {
                Err(__PartialRefToken::ExtractorMut(value))
            }
            __PartialRefToken::Fields(value) => Err(__PartialRefToken::Fields(value)),
            __PartialRefToken::FieldsRef(value) => {
                Err(__PartialRefToken::FieldsRef(value))
            }
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Remainder")>
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    IsPresent,
    __F2__,
    __F3__,
    __F4__,
    __F5__,
    __F6__,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        __F0__,
        IsVoid,
        __F2__,
        __F3__,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Remainder")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Remainder")>>::Value,
        <Self as ExtractField<Symbol!("Remainder")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Remainder(value) => Ok(value),
            __PartialRefToken::Extractor(value) => {
                Err(__PartialRefToken::Extractor(value))
            }
            __PartialRefToken::ExtractorRef(value) => {
                Err(__PartialRefToken::ExtractorRef(value))
            }
            __PartialRefToken::ExtractorMut(value) => {
                Err(__PartialRefToken::ExtractorMut(value))
            }
            __PartialRefToken::Fields(value) => Err(__PartialRefToken::Fields(value)),
            __PartialRefToken::FieldsRef(value) => {
                Err(__PartialRefToken::FieldsRef(value))
            }
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Extractor")>
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    __F1__,
    IsPresent,
    __F3__,
    __F4__,
    __F5__,
    __F6__,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        __F0__,
        __F1__,
        IsVoid,
        __F3__,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Extractor")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Extractor")>>::Value,
        <Self as ExtractField<Symbol!("Extractor")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Extractor(value) => Ok(value),
            __PartialRefToken::ExtractorRef(value) => {
                Err(__PartialRefToken::ExtractorRef(value))
            }
            __PartialRefToken::ExtractorMut(value) => {
                Err(__PartialRefToken::ExtractorMut(value))
            }
            __PartialRefToken::Fields(value) => Err(__PartialRefToken::Fields(value)),
            __PartialRefToken::FieldsRef(value) => {
                Err(__PartialRefToken::FieldsRef(value))
            }
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F4__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("ExtractorRef")>
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    __F1__,
    __F2__,
    IsPresent,
    __F4__,
    __F5__,
    __F6__,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        __F0__,
        __F1__,
        __F2__,
        IsVoid,
        __F4__,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("ExtractorRef")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("ExtractorRef")>>::Value,
        <Self as ExtractField<Symbol!("ExtractorRef")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Extractor(value) => {
                Err(__PartialRefToken::Extractor(value))
            }
            __PartialRefToken::ExtractorRef(value) => Ok(value),
            __PartialRefToken::ExtractorMut(value) => {
                Err(__PartialRefToken::ExtractorMut(value))
            }
            __PartialRefToken::Fields(value) => Err(__PartialRefToken::Fields(value)),
            __PartialRefToken::FieldsRef(value) => {
                Err(__PartialRefToken::FieldsRef(value))
            }
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F5__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("ExtractorMut")>
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    __F1__,
    __F2__,
    __F3__,
    IsPresent,
    __F5__,
    __F6__,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        __F0__,
        __F1__,
        __F2__,
        __F3__,
        IsVoid,
        __F5__,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("ExtractorMut")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("ExtractorMut")>>::Value,
        <Self as ExtractField<Symbol!("ExtractorMut")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Extractor(value) => {
                Err(__PartialRefToken::Extractor(value))
            }
            __PartialRefToken::ExtractorRef(value) => {
                Err(__PartialRefToken::ExtractorRef(value))
            }
            __PartialRefToken::ExtractorMut(value) => Ok(value),
            __PartialRefToken::Fields(value) => Err(__PartialRefToken::Fields(value)),
            __PartialRefToken::FieldsRef(value) => {
                Err(__PartialRefToken::FieldsRef(value))
            }
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F6__: MapType,
> ExtractField<Symbol!("Fields")>
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    __F1__,
    __F2__,
    __F3__,
    __F4__,
    IsPresent,
    __F6__,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, bool>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        __F0__,
        __F1__,
        __F2__,
        __F3__,
        __F4__,
        IsVoid,
        __F6__,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Fields")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Fields")>>::Value,
        <Self as ExtractField<Symbol!("Fields")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Extractor(value) => {
                Err(__PartialRefToken::Extractor(value))
            }
            __PartialRefToken::ExtractorRef(value) => {
                Err(__PartialRefToken::ExtractorRef(value))
            }
            __PartialRefToken::ExtractorMut(value) => {
                Err(__PartialRefToken::ExtractorMut(value))
            }
            __PartialRefToken::Fields(value) => Ok(value),
            __PartialRefToken::FieldsRef(value) => {
                Err(__PartialRefToken::FieldsRef(value))
            }
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
    __F3__: MapType,
    __F4__: MapType,
    __F5__: MapType,
> ExtractField<Symbol!("FieldsRef")>
for __PartialRefToken<
    '__a__,
    __R__,
    __F0__,
    __F1__,
    __F2__,
    __F3__,
    __F4__,
    __F5__,
    IsPresent,
> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, char>;
    type Remainder = __PartialRefToken<
        '__a__,
        __R__,
        __F0__,
        __F1__,
        __F2__,
        __F3__,
        __F4__,
        __F5__,
        IsVoid,
    >;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("FieldsRef")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("FieldsRef")>>::Value,
        <Self as ExtractField<Symbol!("FieldsRef")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Extractor(value) => {
                Err(__PartialRefToken::Extractor(value))
            }
            __PartialRefToken::ExtractorRef(value) => {
                Err(__PartialRefToken::ExtractorRef(value))
            }
            __PartialRefToken::ExtractorMut(value) => {
                Err(__PartialRefToken::ExtractorMut(value))
            }
            __PartialRefToken::Fields(value) => Err(__PartialRefToken::Fields(value)),
            __PartialRefToken::FieldsRef(value) => Ok(value),
        }
    }
}
fn main() {}
