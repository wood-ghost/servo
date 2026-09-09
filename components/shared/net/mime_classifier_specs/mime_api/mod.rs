//! MIME API abstraction layer.
//!
//! Split into three sub-modules for clarity:
//! - [`views`] — the abstract `MimeView` model and derived accessors
//! - [`constants`] — concrete `MimeView` identity values for known MIME types
//! - [`trusted`] — external contracts, axioms, and the `FromStr` bridge (the trusted boundary)

pub mod views;
pub mod constants;
pub mod trusted;
pub mod essence_lemmas;

// Re-export everything — no name conflicts since the module is `views` (plural).
pub use views::*;
pub use constants::*;
pub use trusted::*;
pub use essence_lemmas::*;
