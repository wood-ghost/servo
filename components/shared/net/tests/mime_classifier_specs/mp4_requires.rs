/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Domain checks for the production MP4 precondition. The corresponding native
//! cases in tests/mime_classifier.rs execute the unmodified upstream matcher.

#[macro_use]
mod common;

const REQUIREMENTS: &str = concat!(
    "#[path = \"",
    env!("CARGO_MANIFEST_DIR"),
    "/mime_classifier_specs/requires/mp4.rs\"]\nmod mp4_requires;\n",
);

test_verify_one_file! {
    #[test] mp4_requires_excludes_small_boxes &format!("{REQUIREMENTS}{}", verus_code! {
        use vstd::prelude::*;
        use crate::mp4_requires::mp4_matches_requires;

        proof fn test(size: u8)
            requires size == 0 || size == 4 || size == 8 || size == 12,
        {
            let data = seq![0u8, 0, 0, size, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm'];
            assert(!mp4_matches_requires(data));
        }
    }) => Ok(())
}

test_verify_one_file! {
    #[test] mp4_requires_allows_early_exits &format!("{REQUIREMENTS}{}", verus_code! {
        use vstd::prelude::*;
        use crate::mp4_requires::mp4_matches_requires;

        #[verifier::auto_reveal_byteslit]
        proof fn test() {
            assert(mp4_matches_requires(b""@));
            assert(mp4_matches_requires(b"\x00\x00\x00\x0cftypiso"@));
            assert(mp4_matches_requires(b"\x00\x00\x00\x05ftypisom"@));
            assert(mp4_matches_requires(b"\x00\x00\x00\x14ftypisom"@));
            assert(mp4_matches_requires(b"\x00\x00\x00\x0cfreeisom"@));
            assert(mp4_matches_requires(b"\x00\x00\x00\x0cftypmp41"@));
        }
    }) => Ok(())
}

test_verify_one_file! {
    #[test] mp4_requires_allows_valid_ranges &format!("{REQUIREMENTS}{}", verus_code! {
        use vstd::prelude::*;
        use crate::mp4_requires::mp4_matches_requires;

        #[verifier::auto_reveal_byteslit]
        proof fn test() {
            assert(mp4_matches_requires(b"\x00\x00\x00\x10ftypisom\x00\x00\x00\x00"@));
            assert(mp4_matches_requires(b"\x00\x00\x00\x14ftypisom\x00\x00\x00\x00mp41"@));
        }
    }) => Ok(())
}

test_verify_one_file! {
    #[test] mp4_requires_rejects_invalid_claim &format!("{REQUIREMENTS}{}", verus_code! {
        use vstd::prelude::*;
        use crate::mp4_requires::mp4_matches_requires;

        #[verifier::auto_reveal_byteslit]
        proof fn test() {
            assert(mp4_matches_requires(b"\x00\x00\x00\x0cftypisom"@)); // FAILS
        }
    }) => Err(err) => common::assert_one_fails(err)
}
