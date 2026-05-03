//! Public Rust API for readable regex construction.

/// A typed readable regex pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    inner: rx_core::Pattern,
}

impl Pattern {
    /// Emit this pattern as a compact standard regex string.
    pub fn to_regex(&self) -> String {
        self.inner.to_regex()
    }
}

impl From<rx_core::Pattern> for Pattern {
    fn from(inner: rx_core::Pattern) -> Self {
        Self { inner }
    }
}

/// Construct a pattern that matches the provided text literally.
pub fn literal(value: impl Into<String>) -> Pattern {
    rx_core::Pattern::literal(value).into()
}

/// Common imports for day-to-day `rx` usage.
pub mod prelude {
    pub use crate::{literal, Pattern};
}
