//! A `HasField` bound whose tag misspells a field the struct derives.
//!
//! `Person` derives `HasField` and has `name`; the bound asks for `nmae`. The tool rewrites rustc's
//! "similar impl" help into "`#[derive(HasField)]` is required to access field `nmae`", which is
//! false: the derive is present, and the mistake is the spelling.
//!
//! CGP error class: none; a plain `HasField` bound outside any wiring.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

fn print_name<Context>(context: &Context)
where
    Context: HasField<Symbol!("nmae"), Value = String>,
{
    println!("{}", context.get_field(PhantomData::<Symbol!("nmae")>));
}

fn main() {
    print_name(&Person { name: "A".into() });
}
