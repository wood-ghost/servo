/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! WHATWG preconnect, including the standard's permission to skip the connection.

use net_traits::request::CorsSettings as NativeCorsSettings;
use vstd::prelude::*;
use super::super::super::model::*;
use super::super::super::model_trait::LinkProcessingModel;

verus! {

// https://html.spec.whatwg.org/multipage/links.html#preconnect
pub(crate) open spec fn preconnect<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
) -> PreconnectPlan {
    // 1. If options's href is an empty string, return.
    if options.href.len() == 0 {
        PreconnectPlan::Noop
    } else {
        // 2. Let url be the result of encoding-parsing href relative to base URL.
        let url = model.encoding_parse_url(options.href, options.base_url);
        match url {
            // 3. If url is failure, then return.
            None => PreconnectPlan::Noop,
            Some(url) => {
                // 4. If url's scheme is not an HTTP(S) scheme, return.
                if !model.url_is_http_or_https(url) {
                    PreconnectPlan::Noop
                } else {
                    // 5. Let partitionKey be the network partition key for environment.
                    let partition_key = model.network_partition_key(options.environment);
                    // 6. Let useCredentials be true.
                    let use_credentials = true;
                    // 7. If crossorigin is Anonymous and origins differ, set it to false.
                    let use_credentials = if options.crossorigin is Some
                        && options.crossorigin->Some_0.as_native() == NativeCorsSettings::Anonymous
                        && !model.same_origin(options.origin, model.url_origin(url))
                    { false } else { use_credentials };
                    // 8. The user agent should obtain a connection. The following
                    // paragraph permits partial handshakes or skipping entirely.
                    PreconnectPlan::MayConnect {
                        partition_key, origin: model.url_origin(url), use_credentials,
                    }
                }
            },
        }
    }
}

} // verus!
