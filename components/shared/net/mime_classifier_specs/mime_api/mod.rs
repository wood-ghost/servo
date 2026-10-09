//! Specifications for the external `mime` library.
//!
//! Sub-modules:
//! - [`constants`] — named model values and trusted bindings for library constants
//! - `mime` — the `MimeView` abstraction, `Mime` contracts, and essence lemmas
//! - `name` — the `Name` abstraction, contracts, and equality axioms
//! - [`parser`] — `FromStr` contracts, parsing requirements, and parser models
//!
//! External type specifications, `assume_specification` declarations, and
//! broadcast axioms are trusted boundaries, co-located with the API they model.
//! Parser correspondence and the existing equality-model fidelity gaps remain
//! separate specification obligations; organization does not discharge them.

pub mod constants;
mod mime;
mod name;
pub mod parser;

// Preserve the umbrella API used as SpecMime and by existing glob imports.
// Keep the implementation module `mime` private so it cannot shadow ::mime.
pub use self::constants::*;
pub use self::mime::*;
pub use self::name::*;
pub use self::parser::*;
