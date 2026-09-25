//! MIME API abstraction layer.
//!
//! Sub-modules:
//! - [`views`] — the abstract `MimeView` model and derived accessors
//! - [`constants`] — concrete `MimeView` identity values for known MIME types
//! - [`trusted`] — external contracts, axioms, and the `FromStr` bridge (the trusted boundary)
//! - [`parser`] — MIME parser model and specifications

pub mod views;
pub mod constants;
pub mod trusted;
pub mod essence_lemmas;
pub mod parser;

// Re-export everything — no name conflicts since the module is `views` (plural).
pub use views::*;
pub use constants::*;
pub use trusted::*;
pub use essence_lemmas::*;
