/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! HTML create-a-link-request. Numbering follows the 2026-10-26 standard;
//! Servo's executable comments currently retain an older numbering.

use vstd::prelude::*;
use super::super::model::*;
use super::super::model_trait::LinkProcessingModel;
use super::cors::create_a_potential_cors_request;

verus! {

// https://html.spec.whatwg.org/multipage/semantics.html#create-a-link-request
pub(crate) open spec fn create_link_request<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
) -> Option<RequestView>
    // 1. Assert: options's href is not the empty string.
    recommends options.href.len() > 0,
{
    // 2. Let url be the result of encoding-parsing href relative to base URL.
    let url = model.encoding_parse_url(options.href, options.base_url);
    match url {
        // 3. If url is failure, then return null.
        None => None,
        Some(url) => {
            // 4. Let request be the result of creating a potential-CORS request.
            let request = create_a_potential_cors_request(
                model, url, options.destination, options.crossorigin, false,
            );
            // 5. Set request's policy container to options's policy container.
            let request = RequestView { policy_container: Some(options.policy_container), ..request };
            // 6. Set request's integrity metadata to options's integrity.
            let request = RequestView { integrity: options.integrity, ..request };
            // 7. Set request's cryptographic nonce metadata to options's nonce.
            let request = RequestView { nonce: options.cryptographic_nonce_metadata, ..request };
            // 8. Set request's referrer policy to options's referrer policy.
            let request = RequestView { referrer_policy: options.referrer_policy, ..request };
            // 9. Set request's client to options's environment.
            let request = RequestView { client: Some(options.environment), ..request };
            // 10. Set request's priority to options's fetch priority.
            let request = RequestView { priority: options.fetch_priority, ..request };
            // 11. Return request.
            Some(request)
        },
    }
}

pub(crate) open spec fn create_link_request_ensures<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
    result: Option<RequestView>,
) -> bool {
    &&& options.href.len() > 0
    &&& result == create_link_request(model, options)
}

} // verus!
