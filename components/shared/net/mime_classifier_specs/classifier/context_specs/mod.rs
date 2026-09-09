//! Per-load-context MIME classification result specifications.
//!
//! - [`media_type`] — `get_media_type_spec` and the `mime_classifier_classify_spec` dispatcher
//! - [`browsing`] — the browsing-context result spec and its state-machine lemmas
//! - [`image`] — image-context sniffing results
//! - [`audio_video`] — audio/video-context sniffing results
//! - [`font`] — font-context sniffing results
//! - [`others`] — plugin/style/script/text-track/cache-manifest contexts

pub mod media_type;
pub mod browsing;
pub mod image;
pub mod audio_video;
pub mod font;
pub mod others;

pub use media_type::*;
pub use browsing::*;
pub use image::*;
pub use audio_video::*;
pub use font::*;
pub use others::*;