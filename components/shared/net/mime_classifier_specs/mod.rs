#![allow(unsafe_code)]
#![verifier::auto_reveal_strlit]
#![verifier::auto_reveal_byteslit]

// pub mod model;
pub mod mime_api;
pub mod predicates;
pub mod flag;
pub mod byte_matcher;
pub mod classifier;
pub mod mp4_matcher;
pub mod requires;
pub mod iter;

// Keep experimental protocol work separate.
// pub mod supplied_type;

pub use predicates::*;
