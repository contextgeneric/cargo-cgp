use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanUseFoo {
    fn foo(&self);
}

#[cgp_component(BarProvider)]
pub trait CanUseBar {
    fn bar(&self);
}

#[cgp_impl(new DoBar)]
impl BarProvider {
    fn bar(&self) {}
}

#[cgp_impl(new DoFooWithBar)]
#[uses(CanUseBar)]
impl FooProvider {
    fn foo(&self) {
        self.bar()
    }
}

delegate_components! {
    new CommonProvider {
        FooProviderComponent: DoFooWithBar,
    }
}

delegate_components! {
    new App {
        [
            FooProviderComponent,
            BarProviderComponent,
        ]: CommonProvider,
    }
}

check_components! {
    App {
        FooProviderComponent,
    }
}

fn main() {}
