//! Finalizing with defaults when an unset field's type has no `Default`.
//!
//! `finalize_with_default` with `port: Port` unset and `Port` lacking `Default` is an `E0599` "no
//! method named `finalize_with_default`" with no notes at all. The data-call anchor walks the
//! defaulted finalize to the bound it needs: ``[CGP-E050] the trait bound `Port: Default` is not
//! satisfied``. A step of the chain names the internal `TransformMapDefault` transform.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E117

use cgp::extra::field::impls::CanFinalizeWithDefault;
use cgp::prelude::*;

pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

fn main() {
    let _ = Server::builder()
        .build_field(PhantomData::<Symbol!("host")>, "localhost".to_owned())
        .finalize_with_default();
}
