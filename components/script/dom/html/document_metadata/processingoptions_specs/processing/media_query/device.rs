/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! Media Queries Level 4 device observations, at the instant the query is evaluated.
//! <https://drafts.csswg.org/mediaqueries-4/#mf-dimensions>
use vstd::prelude::*;
verus! {

pub enum Resolution {
    Dppx(real),
    Infinite,
}

pub enum Scan {
    Progressive,
    Interlace,
}

pub enum Update {
    None,
    Slow,
    Fast,
}

pub enum Overflow {
    None,
    Scroll,
    Paged,
}

pub enum Pointer {
    None,
    Coarse,
    Fine,
}

pub enum Gamut {
    BelowSrgb,
    Srgb,
    P3,
    Rec2020,
}

pub struct MediaDeviceView {
    pub print: bool,
    // CSS px, including scrollbars (page box for paged media). None means inapplicable.
    pub viewport: Option<(real, real)>,
    pub device_size: Option<(real, real)>,
    // Initial font metrics in CSS px, never values derived from styling the queried page.
    pub em_px: real,
    pub ex_px: real,
    pub ch_px: real,
    pub cap_px: real,
    pub ic_px: real,
    pub lh_px: real,
    // Unit bases are distinct from the width/height *features* (e.g. scrollbars).
    // The default viewport units use the large viewport. Inline/block follow
    // the initial writing mode, with a small viewport fallback for container units.
    pub small_viewport_px: (real, real),
    pub large_viewport_px: (real, real),
    pub dynamic_viewport_px: (real, real),
    pub vertical_writing: bool,
    pub resolution: Option<Resolution>,
    // Positive CSS px per device pixel for snapping line widths, and the UA's
    // finite numeric saturation limit in canonical units (CSS Values 4 §10.12).
    pub css_px_per_device_pixel: real,
    pub numeric_limit: real,
    pub scan: Option<Scan>,
    pub grid: bool,
    pub update: Update,
    pub overflow_block: Overflow,
    pub inline_scroll: bool,
    pub color_bits: nat,
    pub color_index: nat,
    pub monochrome_bits: nat,
    pub gamut: Gamut,
    pub pointer: Pointer,
    pub hover: bool,
    // Capabilities of additional input devices; the primary device is included by evaluation.
    pub additional_coarse: bool,
    pub additional_fine: bool,
    pub additional_hover: bool,
}

} // verus!
