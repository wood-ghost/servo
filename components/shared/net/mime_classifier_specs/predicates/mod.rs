//! MIME sniffing predicates and matching helpers.
//!
//! Split from the former single `predicates.rs` into sub-modules by concern:
//! - [`mime_type`] — predicates over the MIME type itself
//! - [`byte_matchers`] — simple fixed-signature byte matchers
//! - [`signature`] — structured WebM, MP3, and MP4 signatures

pub mod mime_type;
pub mod byte_matchers;
pub mod signature;

pub use mime_type::*;
pub use byte_matchers::*;
pub use signature::*;
