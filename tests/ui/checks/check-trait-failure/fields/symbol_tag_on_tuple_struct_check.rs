//! A provider that requires a tuple field by `Symbol!("0")`, checked on a tuple struct.
//!
//! `Pair` derives `HasField`, keyed by `Index<N>`, so `HasField<Symbol!("0")>` does not hold: a
//! digit string names no field. The block states the field is missing, and its `help` names the
//! `Index<0>` tag the field takes. The classifier used to read a tuple struct's fields as named `0`
//! and `1`, so it took the tag for a field `Pair` carries without the derive and blamed the derive,
//! which `Pair` has.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetFirst)]
impl Greeter
where
    Self: HasField<Symbol!("0"), Value = u32>,
{
    fn greet(&self) -> String {
        format!("hello {}", self.get_field(PhantomData::<Symbol!("0")>))
    }
}

#[derive(HasField)]
pub struct Pair(pub u32, pub String);

delegate_components! {
    Pair {
        GreeterComponent: GreetFirst,
    }
}

check_components! {
    Pair {
        GreeterComponent,
    }
}

fn main() {}
