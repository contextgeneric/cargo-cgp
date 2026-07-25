#![feature(prelude_import)]
//! A component whose single generic parameter is instantiated with a *tuple* type.
//!
//! CGP's params encoding is ambiguous here: a check entry `(u32, u64)` is the same
//! params tuple whether the component has two parameters or one tuple-typed one.
//! The resolver rebuilds the consumer obligation from that slot, so it must not
//! mistake the one tuple-typed parameter for two separate ones.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanFormatPair<T> {
    fn format_pair(&self, value: T) -> String;
}
impl<__Context__, T> CanFormatPair<T> for __Context__
where
    __Context__: PairFormatter<__Context__, T>,
{
    fn format_pair(&self, value: T) -> String {
        __Context__::format_pair(self, value)
    }
}
pub trait PairFormatter<
    __Context__,
    T,
>: IsProviderFor<PairFormatterComponent, __Context__, (T)> {
    fn format_pair(__context__: &__Context__, value: T) -> String;
}
impl<__Provider__, __Context__, T> PairFormatter<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<PairFormatterComponent>
        + IsProviderFor<PairFormatterComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        PairFormatterComponent,
    >>::Delegate: PairFormatter<__Context__, T>,
{
    fn format_pair(__context__: &__Context__, value: T) -> String {
        <__Provider__ as DelegateComponent<
            PairFormatterComponent,
        >>::Delegate::format_pair(__context__, value)
    }
}
pub struct PairFormatterComponent;
impl<__Context__, T> PairFormatter<__Context__, T> for UseContext
where
    __Context__: CanFormatPair<T>,
{
    fn format_pair(__context__: &__Context__, value: T) -> String {
        __Context__::format_pair(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<PairFormatterComponent, __Context__, (T)>
for UseContext
where
    __Context__: CanFormatPair<T>,
{}
impl<__Context__, T, __Components__, __Path__> PairFormatter<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: PairFormatter<__Context__, T>,
{
    fn format_pair(__context__: &__Context__, value: T) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::format_pair(__context__, value)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<PairFormatterComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<PairFormatterComponent, __Context__, (T)>
        + PairFormatter<__Context__, T>,
{}
pub trait HasSeparator {
    fn separator(&self) -> &String;
}
impl<__Context__> HasSeparator for __Context__
where
    __Context__: HasField<Symbol!("separator"), Value = String>,
{
    fn separator(&self) -> &String {
        self.get_field(::core::marker::PhantomData::<Symbol!("separator")>)
    }
}
impl<__Context__> PairFormatter<__Context__, (u32, u64)> for FormatWithSeparator
where
    __Context__: HasSeparator,
{
    fn format_pair(__context__: &__Context__, value: (u32, u64)) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(
                format_args!("{0}{1}{2}", value.0, __context__.separator(), value.1),
            )
        })
    }
}
impl<__Context__> IsProviderFor<PairFormatterComponent, __Context__, ((u32, u64))>
for FormatWithSeparator
where
    __Context__: HasSeparator,
{}
pub struct FormatWithSeparator;
pub struct App {
    pub dummy: (),
}
impl HasField<Symbol!("dummy")> for App {
    type Value = ();
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("dummy")>,
    ) -> &Self::Value {
        &self.dummy
    }
}
impl HasFieldMut<Symbol!("dummy")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("dummy")>,
    ) -> &mut Self::Value {
        &mut self.dummy
    }
}
impl DelegateComponent<PairFormatterComponent> for App {
    type Delegate = FormatWithSeparator;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<PairFormatterComponent, __Context__, __Params__> for App
where
    FormatWithSeparator: IsProviderFor<PairFormatterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<PairFormatterComponent, (u32, u64)> for App {}
fn main() {}
