#![feature(prelude_import)]
//! A generic consumer called on a field access. The use-site anchor reaches a component only with
//! the parameter its key carries, and this one rides in the call, whose receiver the call-site
//! anchor cannot type. So the resolver declines and the cause stays under rustc's headline.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> String;
}
impl<__Context__, Value> CanEncode<Value> for __Context__
where
    __Context__: Encoder<__Context__, Value>,
{
    fn encode(&self, value: &Value) -> String {
        __Context__::encode(self, value)
    }
}
pub trait Encoder<
    __Context__,
    Value,
>: IsProviderFor<EncoderComponent, __Context__, (Value)> {
    fn encode(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value> Encoder<__Context__, Value> for __Provider__
where
    __Provider__: DelegateComponent<EncoderComponent>
        + IsProviderFor<EncoderComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        EncoderComponent,
    >>::Delegate: Encoder<__Context__, Value>,
{
    fn encode(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            EncoderComponent,
        >>::Delegate::encode(__context__, value)
    }
}
///Component marker for [`CanEncode`].
pub struct EncoderComponent;
impl<__Context__, Value> Encoder<__Context__, Value> for UseContext
where
    __Context__: CanEncode<Value>,
{
    fn encode(__context__: &__Context__, value: &Value) -> String {
        __Context__::encode(__context__, value)
    }
}
impl<__Context__, Value> IsProviderFor<EncoderComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanEncode<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> Encoder<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: Encoder<__Context__, Value>,
{
    fn encode(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::encode(__context__, value)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<EncoderComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<EncoderComponent, __Context__, (Value)>
        + Encoder<__Context__, Value>,
{}
impl<__Context__, Value> Encoder<__Context__, Value> for EncodeNamed
where
    __Context__: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{
    fn encode(__context__: &__Context__, _value: &Value) -> String {
        let name: &str = ::core::convert::AsRef::as_ref(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("name")>,
            ),
        );
        name.to_owned()
    }
}
impl<__Context__, Value> IsProviderFor<EncoderComponent, __Context__, (Value)>
for EncodeNamed
where
    __Context__: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{}
///Provider `EncodeNamed` for [`Encoder`].
pub struct EncodeNamed;
pub struct App {
    pub other: String,
}
impl HasField<Symbol!("other")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &Self::Value {
        &self.other
    }
}
impl HasFieldMut<Symbol!("other")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &mut Self::Value {
        &mut self.other
    }
}
impl DelegateComponent<EncoderComponent> for App {
    type Delegate = EncodeNamed;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<EncoderComponent, __Context__, __Params__> for App
where
    EncodeNamed: IsProviderFor<EncoderComponent, __Context__, __Params__>,
{}
pub struct Holder {
    pub app: App,
}
fn main() {
    let holder = Holder {
        app: App { other: String::new() },
    };
    let _ = holder.app.encode(&1u32);
}
