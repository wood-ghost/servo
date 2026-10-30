/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Small reusable Infra/HTML predicates, analogous to the MIME predicate modules.

use vstd::prelude::*;

verus! {

/// <https://infra.spec.whatwg.org/#ascii-lowercase>
/// The same character mapping used by mime_api::utils::ascii_lowercase.
pub open spec fn ascii_lowercase(input: Seq<char>) -> Seq<char> {
    input.map(|i: int, c: char|
        if 'A' <= c && c <= 'Z' { ((c as u32 + 0x20) as u32) as char } else { c }
    )
}

/// <https://infra.spec.whatwg.org/#ascii-case-insensitive>
/// Two strings are an ASCII case-insensitive match when their ASCII-lowercased
/// forms are identical.
pub(crate) open spec fn ascii_case_insensitive_match(left: Seq<char>, right: Seq<char>) -> bool {
    ascii_lowercase(left) == ascii_lowercase(right)
}

pub(crate) open spec fn is_referrer_policy(value: Seq<char>) -> bool {
    ||| value == ""@
    ||| value == "no-referrer"@
    ||| value == "no-referrer-when-downgrade"@
    ||| value == "origin"@
    ||| value == "origin-when-cross-origin"@
    ||| value == "same-origin"@
    ||| value == "strict-origin"@
    ||| value == "strict-origin-when-cross-origin"@
    ||| value == "unsafe-url"@
}

pub(crate) open spec fn is_preload_destination(value: Seq<char>) -> bool {
    ||| value == "fetch"@
    ||| value == "font"@
    ||| value == "image"@
    ||| value == "script"@
    ||| value == "style"@
    ||| value == "track"@
}

} // verus!
