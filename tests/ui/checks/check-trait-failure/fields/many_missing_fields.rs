//! A provider reading six fields the context lacks, through `#[implicit]` arguments.
//!
//! `SumFields` reads `alpha` to `zeta` as implicit arguments, and `App` declares none of them,
//! so the check has six root causes of one kind on one owner. The headline states them as one
//! merged sentence, the `root causes:` list names each, and the tree branches to each field.
//! The fixture pins two rules: the causes appear in the order the arguments are written, and a
//! long merged headline names the first few fields and counts the rest, while the list and the
//! tree stay complete.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(Summer)]
pub trait CanSum {
    fn sum(&self) -> u64;
}

#[cgp_impl(new SumFields)]
impl Summer {
    fn sum(
        &self,
        #[implicit] alpha: u64,
        #[implicit] beta: u64,
        #[implicit] gamma: u64,
        #[implicit] delta: u64,
        #[implicit] epsilon: u64,
        #[implicit] zeta: u64,
    ) -> u64 {
        alpha + beta + gamma + delta + epsilon + zeta
    }
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        SummerComponent: SumFields,
    }
}

check_components! {
    App {
        SummerComponent,
    }
}

fn main() {}
