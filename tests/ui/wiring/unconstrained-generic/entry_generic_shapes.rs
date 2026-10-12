//! Per-entry generics that never reach the key, in three entry shapes beside the plain one.
//!
//! An entry's generic list binds its parameters on the impls the entry lowers to, and only the key
//! can constrain them there. So the compiler rejects each parameter the key leaves out with
//! `E0207`, once per generated impl, and suggests adding it to the context or removing it. Here an
//! entry binds two such parameters, an entry binds one it never uses, and a `->` entry forwards to a
//! table named with its parameter.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct GreetWith<T>(pub PhantomData<T>);

#[cgp_provider]
impl<Context, T> Greeter<Context> for GreetWith<T> {
    fn greet(_context: &Context) {}
}

pub struct GreetPair<A, B>(pub PhantomData<(A, B)>);

#[cgp_provider]
impl<Context, A, B> Greeter<Context> for GreetPair<A, B> {
    fn greet(_context: &Context) {}
}

pub struct Table<T>(pub PhantomData<T>);

delegate_components! {
    <T> Table<T> {
        GreeterComponent: GreetWith<T>,
    }
}

pub struct Person;

// Two parameters, both in the provider and neither in the key.
delegate_components! {
    Person {
        <T, U> GreeterComponent: GreetPair<T, U>,
    }
}

pub struct Robot;

// A parameter used nowhere.
delegate_components! {
    Robot {
        <T> GreeterComponent: GreetWith<u32>,
    }
}

pub struct Kiosk;

// A parameter in the table the entry forwards to.
delegate_components! {
    Kiosk {
        <T> GreeterComponent -> Table<T>,
    }
}

fn main() {}
