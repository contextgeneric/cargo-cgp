//! `Symbol!` tags on tuple structs that spell no position of the struct.
//!
//! `GreetThird` requires the field `Symbol!("2")` of `Pair`, which has two fields, and
//! `GreetPadded` requires `Symbol!("01")` of `Couple`, a padded spelling of a position. A tuple
//! struct keys its fields by `Index<N>`, so each check reports a missing field, and neither gets the
//! `help` naming an `Index` tag, since no field sits at position 2 and `01` is no position's name.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> u32;
}

#[cgp_impl(new GreetThird)]
impl Greeter
where
    Self: HasField<Symbol!("2"), Value = u32>,
{
    fn greet(&self) -> u32 {
        *self.get_field(PhantomData::<Symbol!("2")>)
    }
}

#[cgp_impl(new GreetPadded)]
impl Greeter
where
    Self: HasField<Symbol!("01"), Value = u32>,
{
    fn greet(&self) -> u32 {
        *self.get_field(PhantomData::<Symbol!("01")>)
    }
}

#[derive(HasField)]
pub struct Pair(pub u32, pub u32);

#[derive(HasField)]
pub struct Couple(pub u32, pub u32);

delegate_components! {
    Pair {
        GreeterComponent: GreetThird,
    }
}

delegate_components! {
    Couple {
        GreeterComponent: GreetPadded,
    }
}

check_components! {
    Pair {
        GreeterComponent,
    }
}

check_components! {
    #[check_trait(CheckCouple)]
    Couple {
        GreeterComponent,
    }
}

fn main() {}
