//! Contracts for link processing options.
//!
//! Requirements reuse the external MIME parser contract and the classifier's
//! input requirements. Parser/library correspondence remains a trusted boundary;
//! the requirement lemmas do not verify the external processing-options bodies.
//! Postcondition specifications live in `postconditions`.
//! Temporary type declarations and API contracts live in `servo_support`.

pub(crate) mod postconditions;
pub(crate) mod requires;
mod servo_support;

pub(crate) use self::postconditions::*;
pub(crate) use self::requires::*;
