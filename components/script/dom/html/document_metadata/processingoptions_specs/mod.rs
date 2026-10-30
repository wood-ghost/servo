//! Contracts for link processing options.
//!
//! Requirements reuse the external MIME parser contract and the classifier's
//! input requirements. Parser/library correspondence remains a trusted boundary;
//! the requirement lemmas do not verify the external processing-options bodies.
//! `processing/algorithms` follows the WHATWG call hierarchy, as the MIME
//! classifier's `classifier/algorithms` does. Shared predicates, input requirements,
//! executable-facing contracts, and external API support have separate modules.
//! Model-only checks validate the reference algorithms; native conformance also
//! requires representation/primitive agreement and recorded execution effects.

#![verifier::auto_reveal_strlit]

pub(crate) mod predicates;
pub(crate) mod requires;
pub(crate) mod processing;
pub(crate) mod servo_api;

// Preserve the Spec:: and SpecRequires:: umbrella paths used by executable code.
pub(crate) use self::processing::context_specs::*;
pub(crate) use self::requires::*;
