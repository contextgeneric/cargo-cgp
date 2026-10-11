//! The panic the emitter raises on purpose, so a fixture can prove it contains its own panics.
//!
//! The emitter falls back to rustc's own diagnostics when its transform or its flush panics, and
//! no known input makes either panic, so the UI fixtures that pin the fallback set
//! [`TEST_PANIC_ENV`](crate::config::TEST_PANIC_ENV) to choose a stage that panics.

/// A stage of the emitter that can be made to panic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestPanic {
    /// The transform of each error diagnostic.
    Transform,
    /// The flush that renders the buffered diagnostics.
    Flush,
}

impl TestPanic {
    /// The stage a [`TEST_PANIC_ENV`](crate::config::TEST_PANIC_ENV) value names, or `None` for any
    /// other value.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "transform" => Some(Self::Transform),
            "flush" => Some(Self::Flush),
            _ => None,
        }
    }
}

/// Panic when the armed stage is `stage`.
pub fn inject(armed: Option<TestPanic>, stage: TestPanic) {
    if armed == Some(stage) {
        panic!("injected test panic in the {stage:?} stage");
    }
}
