/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Link-processing specifications, organized like mime_classifier_specs::classifier.
//!
//! `algorithms` follows the WHATWG call hierarchy. `model` contains specification
//! views, `model_trait` names the referenced external algorithms, and
//! `context_specs` connects the models to executable Servo contracts.

pub(crate) mod model;
pub(crate) mod types;
pub(crate) mod model_trait;
pub(crate) mod trace;
pub(crate) mod algorithms;
pub(crate) mod context_specs;
pub(crate) mod tests;

pub(crate) use self::model::*;
pub(crate) use self::types::*;
