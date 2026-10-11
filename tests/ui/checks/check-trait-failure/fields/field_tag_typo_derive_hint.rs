//! A `HasField` bound whose tag misspells a field the struct derives.
//!
//! `Person` derives `HasField` and has `name`; the bound asks for `nmae`. The tool rewrites rustc's
//! "similar impl" help into "missing field `nmae` on `Person`", its caret on the `name` field that
//! does exist. It reads the derive as present from rustc's own landmark, "but trait
//! `HasField<Symbol!("name")>` is implemented for it", whose `is` rustc styles as a separate
//! fragment, so the check reads each message's fragments joined.
//!
//! CGP error class: none; a plain `HasField` bound outside any wiring.

//@quality: acceptable
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
