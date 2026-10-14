#![feature(prelude_import)]
//! A provider bounded on `FieldsRef<'b>` for every lifetime, checked for an enum with a lifetime.
//!
//! The check entry fails with a bare `E0477` "the type `Token<'a>` does not fulfill the required
//! lifetime"; nothing names the for-every-lifetime bound that demands `'static`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::fmt::Debug;
use cgp::prelude::*;
pub trait CanShowValue<Value> {
    fn show_value(&self, value: &Value) -> String;
}
impl<__Context__, Value> CanShowValue<Value> for __Context__
where
    __Context__: ValueShower<__Context__, Value>,
{
    fn show_value(&self, value: &Value) -> String {
        __Context__::show_value(self, value)
    }
}
pub trait ValueShower<
    __Context__,
    Value,
>: IsProviderFor<ValueShowerComponent, __Context__, (Value)> {
    fn show_value(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value> ValueShower<__Context__, Value> for __Provider__
where
    __Provider__: DelegateComponent<ValueShowerComponent>
        + IsProviderFor<ValueShowerComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        ValueShowerComponent,
    >>::Delegate: ValueShower<__Context__, Value>,
{
    fn show_value(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            ValueShowerComponent,
        >>::Delegate::show_value(__context__, value)
    }
}
pub struct ValueShowerComponent;
impl<__Context__, Value> ValueShower<__Context__, Value> for UseContext
where
    __Context__: CanShowValue<Value>,
{
    fn show_value(__context__: &__Context__, value: &Value) -> String {
        __Context__::show_value(__context__, value)
    }
}
impl<__Context__, Value> IsProviderFor<ValueShowerComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanShowValue<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> ValueShower<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: ValueShower<__Context__, Value>,
{
    fn show_value(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::show_value(__context__, value)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<ValueShowerComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<ValueShowerComponent, __Context__, (Value)>
        + ValueShower<__Context__, Value>,
{}
impl<__Context__, Value> ValueShower<__Context__, Value> for ShowFields
where
    Value: ToFieldsRef,
    for<'b> Value::FieldsRef<'b>: Debug,
{
    fn show_value(__context__: &__Context__, value: &Value) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0:?}", value.to_fields_ref()))
        })
    }
}
impl<__Context__, Value> IsProviderFor<ValueShowerComponent, __Context__, (Value)>
for ShowFields
where
    Value: ToFieldsRef,
    for<'b> Value::FieldsRef<'b>: Debug,
{}
pub struct ShowFields;
pub enum Token<'a> {
    Word(&'a str),
    Number(u64),
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for Token<'a> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Self::Word(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Word", &__self_0)
            }
            Self::Number(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Number", &__self_0)
            }
        }
    }
}
impl<'a> HasFields for Token<'a> {
    type Fields = Enum! { Word(&'a str), Number(u64) };
}
impl<'a> HasFieldsRef for Token<'a> {
    type FieldsRef<'__a> = Enum! { Word(&'__a &'a str), Number(&'__a u64) }
    where
        Self: '__a;
}
impl<'a> FromFields for Token<'a> {
    fn from_fields(rest: <Self as HasFields>::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Word(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Number(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl<'a> ToFields for Token<'a> {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Word(field) => Either::Left(field.into()),
            Self::Number(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl<'a> ToFieldsRef for Token<'a> {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Word(field) => Either::Left(field.into()),
            Self::Number(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl<'a> FromVariant<Symbol!("Word")> for Token<'a> {
    type Value = &'a str;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Word")>,
        value: <Self as FromVariant<Symbol!("Word")>>::Value,
    ) -> Self {
        Self::Word(value)
    }
}
impl<'a> FromVariant<Symbol!("Number")> for Token<'a> {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Number")>,
        value: <Self as FromVariant<Symbol!("Number")>>::Value,
    ) -> Self {
        Self::Number(value)
    }
}
pub enum __PartialToken<'a, __F0__: MapType, __F1__: MapType> {
    Word(<__F0__ as MapType>::Map<&'a str>),
    Number(<__F1__ as MapType>::Map<u64>),
}
pub enum __PartialRefToken<
    '__a__,
    'a: '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
> {
    Word(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, &'a str>>),
    Number(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
}
impl<'a, __F0__: MapType, __F1__: MapType> PartialData
for __PartialToken<'a, __F0__, __F1__> {
    type Target = Token<'a>;
}
impl<'__a__, 'a, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefToken<'__a__, 'a, __R__, __F0__, __F1__> {
    type Target = Token<'a>;
}
impl<'a> HasExtractor for Token<'a> {
    type Extractor = __PartialToken<'a, IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Word(value) => __PartialToken::Word(value),
            Self::Number(value) => __PartialToken::Number(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialToken::Word(value) => Self::Word(value),
            __PartialToken::Number(value) => Self::Number(value),
        }
    }
}
impl<'a> HasExtractorRef for Token<'a> {
    type ExtractorRef<'__a__> = __PartialRefToken<
        '__a__,
        'a,
        IsRef,
        IsPresent,
        IsPresent,
    >
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Word(value) => __PartialRefToken::Word(value),
            Self::Number(value) => __PartialRefToken::Number(value),
        }
    }
}
impl<'a> HasExtractorMut for Token<'a> {
    type ExtractorMut<'__a__> = __PartialRefToken<
        '__a__,
        'a,
        IsMut,
        IsPresent,
        IsPresent,
    >
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Word(value) => __PartialRefToken::Word(value),
            Self::Number(value) => __PartialRefToken::Number(value),
        }
    }
}
impl<'a> FinalizeExtract for __PartialToken<'a, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, 'a, __R__: MapTypeRef> FinalizeExtract
for __PartialRefToken<'__a__, 'a, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'a, __F1__: MapType> ExtractField<Symbol!("Word")>
for __PartialToken<'a, IsPresent, __F1__> {
    type Value = &'a str;
    type Remainder = __PartialToken<'a, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Word")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Word")>>::Value,
        <Self as ExtractField<Symbol!("Word")>>::Remainder,
    > {
        match self {
            __PartialToken::Word(value) => Ok(value),
            __PartialToken::Number(value) => Err(__PartialToken::Number(value)),
        }
    }
}
impl<'a, __F0__: MapType> ExtractField<Symbol!("Number")>
for __PartialToken<'a, __F0__, IsPresent> {
    type Value = u64;
    type Remainder = __PartialToken<'a, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Number")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Number")>>::Value,
        <Self as ExtractField<Symbol!("Number")>>::Remainder,
    > {
        match self {
            __PartialToken::Word(value) => Err(__PartialToken::Word(value)),
            __PartialToken::Number(value) => Ok(value),
        }
    }
}
impl<'__a__, 'a, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Word")>
for __PartialRefToken<'__a__, 'a, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, &'a str>;
    type Remainder = __PartialRefToken<'__a__, 'a, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Word")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Word")>>::Value,
        <Self as ExtractField<Symbol!("Word")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Word(value) => Ok(value),
            __PartialRefToken::Number(value) => Err(__PartialRefToken::Number(value)),
        }
    }
}
impl<'__a__, 'a, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Number")>
for __PartialRefToken<'__a__, 'a, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefToken<'__a__, 'a, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Number")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Number")>>::Value,
        <Self as ExtractField<Symbol!("Number")>>::Remainder,
    > {
        match self {
            __PartialRefToken::Word(value) => Err(__PartialRefToken::Word(value)),
            __PartialRefToken::Number(value) => Ok(value),
        }
    }
}
pub struct App;
impl DelegateComponent<ValueShowerComponent> for App {
    type Delegate = ShowFields;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ValueShowerComponent, __Context__, __Params__> for App
where
    ShowFields: IsProviderFor<ValueShowerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl<'a> __CheckApp<ValueShowerComponent, Token<'a>> for App {}
fn main() {}
