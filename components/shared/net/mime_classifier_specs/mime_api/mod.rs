//! Specifications for the external `mime` library.
//!
//! Sub-modules:
//! - [`constants`] — named model values and trusted bindings for library constants
//! - [`lemmas`] — proved MIME API support lemmas and broadcast groups
//! - `mime` — the `MimeView` abstraction and external API contracts, including parsing
//! - `name` — the `Name` abstraction, contracts, and equality axioms
//! - [`parser`] — parsing requirements, parser models, and state-machine proofs
//! - [`utils`] — character predicates, whitespace handling, and sequence utilities
//!
//! External type specifications, `assume_specification` declarations, and
//! broadcast axioms are trusted boundaries, co-located with the API they model.
//! Parser correspondence and the existing equality-model fidelity gaps remain
//! separate specification obligations; organization does not discharge them.

pub mod constants;
pub mod lemmas;
mod mime;
mod name;
pub mod parser;
pub mod utils;

// Preserve the umbrella API used as SpecMime and by existing glob imports.
// Keep the implementation module `mime` private so it cannot shadow ::mime.
pub use self::constants::*;
pub use self::lemmas::*;
pub use self::mime::*;
pub use self::name::*;
pub use self::parser::*;
pub use self::utils::*;
