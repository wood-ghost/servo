/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specifications for the external content-security-policy crate.
//! Re-exporting a type through net_traits does not change its implementation owner.
//!
//! - `destination`: destination conversions, predicates, API contracts, and proofs.
//! - `policy`: CSP list/policy/directive views and policy metadata declarations.
//! External declarations and assumed method contracts remain trusted boundaries.

mod destination;
mod policy;

pub(crate) use self::destination::*;
pub(crate) use self::policy::*;
