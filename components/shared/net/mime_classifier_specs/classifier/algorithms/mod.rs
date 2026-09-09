//! WHATWG pattern-matching and sniffing algorithms.

pub mod pattern_matching;
pub mod sniffing;
pub mod mime_type_sniffing;

pub use pattern_matching::*;
pub use sniffing::*;
pub use mime_type_sniffing::*;
