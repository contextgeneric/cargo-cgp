#![feature(prelude_import)]
//! A generic function's bound on a generic consumer trait, its context from a binding of reference
//! type.
//!
//! `format_it(app)` fills the declared input `&Context`, and `app` is declared `&App`, so the
//! bound-call anchor needs the binding's type with its reference to pin `Context = App`. It then
//! seeds the bound `App: CanFormatPair<u32>` and walks to the missing `separator` field. The consumer
//! takes a parameter, so no other anchor recovers this bound from the diagnostic's spans.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
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
///Component marker for [`CanFormatPair`].
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
        HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("separator")>)
    }
}
impl<__Context__> PairFormatter<__Context__, u32> for FormatWithSeparator
where
    __Context__: HasSeparator,
{
    fn format_pair(__context__: &__Context__, value: u32) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0}{1}", __context__.separator(), value))
        })
    }
}
impl<__Context__> IsProviderFor<PairFormatterComponent, __Context__, (u32)>
for FormatWithSeparator
where
    __Context__: HasSeparator,
{}
///Provider `FormatWithSeparator` for [`PairFormatter`].
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
    __Params__: ?Sized,
> IsProviderFor<PairFormatterComponent, __Context__, __Params__> for App
where
    FormatWithSeparator: IsProviderFor<PairFormatterComponent, __Context__, __Params__>,
{}
fn format_it<Context: CanFormatPair<u32>>(context: &Context) -> String {
    context.format_pair(1)
}
fn run(app: &App) -> String {
    format_it(app)
}
fn main() {}
