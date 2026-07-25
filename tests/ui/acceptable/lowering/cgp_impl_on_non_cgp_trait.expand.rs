#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Greet {
    fn greet(&self);
}
impl<__Context__> Greet<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) {
        {
            ::std::io::_print(format_args!("hello\n"));
        };
    }
}
impl<__Context__> IsProviderFor<GreetComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
fn main() {}
