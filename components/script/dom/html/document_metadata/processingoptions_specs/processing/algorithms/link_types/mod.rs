/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Per-relation HTTP Link processing algorithms.

pub(crate) mod dispatch;
pub(crate) mod preload;
pub(crate) mod preconnect;

pub(crate) use self::dispatch::*;
