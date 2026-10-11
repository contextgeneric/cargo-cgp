#![feature(prelude_import)]
//! A `HasField` bound whose tag misspells a field the struct derives.
//!
//! `Person` derives `HasField` and has `name`; the bound asks for `nmae`. The tool rewrites rustc's
//! "similar impl" help into "`#[derive(HasField)]` is required to access field `nmae`", which is
//! false: the derive is present, and the mistake is the spelling.
//!
//! CGP error class: none; a plain `HasField` bound outside any wiring.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Person {
    pub name: String,
}
impl HasField<Symbol!("name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
fn print_name<Context>(context: &Context)
where
    Context: HasField<Symbol!("nmae"), Value = String>,
{
    {
        ::std::io::_print(
            format_args!(
                "{0}\n", context.get_field(PhantomData::<::cgp::macro_prelude::Symbol <
                4, ::cgp::macro_prelude::Chars <'n', ::cgp::macro_prelude::Chars <'m',
                ::cgp::macro_prelude::Chars <'a', ::cgp::macro_prelude::Chars <'e',
                ::cgp::macro_prelude::Nil >>>>>>)
            ),
        );
    };
}
fn main() {
    print_name(&Person { name: "A".into() });
}
