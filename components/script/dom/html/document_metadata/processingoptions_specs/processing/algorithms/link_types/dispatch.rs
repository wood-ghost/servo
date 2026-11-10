/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Per-relation HTTP Link processing, following the HTML link-type sections.

use vstd::prelude::*;
use super::super::super::model::*;
use super::super::super::model_trait::LinkProcessingModel;
use super::{preload, preconnect};

verus! {

// https://html.spec.whatwg.org/multipage/semantics.html#process-a-link-header
// All link types that can be external resource links define a process a link
// header algorithm. The concrete definitions live in each link-type section.
pub(crate) open spec fn process_link_header<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
    rel: Seq<char>,
) -> LinkHeaderPlan {
    if rel == "preload"@ {
        // links.html#link-type-preload:process-a-link-header: preload options.
        LinkHeaderPlan::Preload { preload: preload::preload(model, options, None) }
    } else if rel == "preconnect"@ {
        // links.html#link-type-preconnect:process-a-link-header: preconnect options.
        LinkHeaderPlan::Preconnect { preconnect: preconnect::preconnect(model, options) }
    } else {
        // The built-in HTML relation table gives no header processing for the
        // other keywords. Extension-defined handlers need their own model.
        LinkHeaderPlan::Noop
    }
}

pub(crate) open spec fn process_link_header_ensures<M: LinkProcessingModel>(
    model: M,
    options: LinkProcessingOptionsView,
    rel: Seq<char>,
    result: LinkHeaderPlan,
) -> bool {
    &&& result == process_link_header(model, options, rel)
    &&& match result {
        LinkHeaderPlan::Preload { preload: plan } => preload::preload_ensures(model, options, None, plan),
        _ => true,
    }
}

/// Invocation recorded by the common caller. The per-type result/effect contract
/// above is an additional obligation when this trace is connected to execution.
pub(crate) open spec fn process_link_header_call_matches(
    rel: Seq<char>,
    options: LinkProcessingOptionsView,
    step: LinkHeaderStep,
) -> bool {
    match step.action {
        LinkHeaderAction::Dispatch { process, .. } =>
            process.relation == rel && process.options == options,
        _ => false,
    }
}

} // verus!
