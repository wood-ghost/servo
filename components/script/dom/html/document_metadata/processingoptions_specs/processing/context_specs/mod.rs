/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Executable-facing contracts and callback adapters.

pub(crate) mod contracts;
pub(crate) mod fetch_context;

pub(crate) use self::contracts::*;
