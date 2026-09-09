//! MIME classifier specification layer.
//!
//! Split from the former single `classifier.rs` into sub-modules by concern:
//! - [`checker_trait`] — the checker/model trait contracts
//! - [`grouped`] — grouped classifier specs (binary/plaintext, first-success recursion)
//! - [`algorithms`] — WHATWG pattern-matching, sniffing, and the MIME sniffing state machine
//! - [`context_specs`] — per-load-context classification result specs

pub mod checker_trait;
pub mod grouped;
pub mod algorithms;
pub mod context_specs;

// Umbrella re-export so that `crate::mime_classifier_specs::classifier::ITEM`
// and `classifier as SpecClassifier` paths keep working unchanged.
pub use checker_trait::*;
pub use grouped::*;
pub use algorithms::*;
pub use context_specs::*;
