/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Interface for the algorithms delegated to MIME Sniffing, Fetch, URL, and RFC 8288.
//! Like MimeClassifierModel, it keeps the numbered outer algorithms independent
//! of a particular implementation. Supplying a model does not prove correspondence.

use vstd::prelude::*;
use super::model::*;
use super::types::*;

verus! {

pub(crate) trait LinkProcessingModel {
    type Mime;

    spec fn get_decode_and_split_link_headers(&self, headers: HeaderListView) -> Seq<Seq<char>>;
    // RFC 8288 parsing can return multiple links (including relation expansion).
    // None represents a parsed object without target_uri. The concrete parser
    // adapter must establish the relationship to HTML's singular linkObject wording.
    spec fn parse_link_header(&self, value: Seq<char>) -> Seq<Option<ParsedLinkHeader>>;
    spec fn parse_mime_type(&self, value: Seq<char>) -> Option<Self::Mime>;
    spec fn mime_supported(&self, mime: Self::Mime) -> bool;
    spec fn is_javascript_mime(&self, mime: Self::Mime) -> bool;
    spec fn is_image_mime(&self, mime: Self::Mime) -> bool;
    spec fn is_font_mime(&self, mime: Self::Mime) -> bool;
    spec fn mime_essence(&self, mime: Self::Mime) -> Seq<char>;
    spec fn encoding_parse_url(&self, href: Seq<char>, base_url: ServoUrlView) -> Option<ServoUrlView>;
    spec fn new_request(&self) -> RequestView;
    spec fn select_image_source(&self, source_set: SourceSetView) -> Seq<char>;
    spec fn url_is_http_or_https(&self, url: ServoUrlView) -> bool;
    spec fn url_origin(&self, url: ServoUrlView) -> OriginView;
    spec fn same_origin(&self, left: OriginView, right: OriginView) -> bool;
    spec fn network_partition_key(&self, environment: EnvironmentView) -> int;
    spec fn network_error(&self) -> ResponseView;
    spec fn unsafe_shared_current_time(&self) -> int;
    spec fn document_relevant_global(&self, document: DocumentView) -> int;
    // https://fetch.spec.whatwg.org/#concept-fetch
    spec fn fetch_request(&self, request: RequestView, process_response_consume_body: PreloadCallback) -> int;
    // Identity of the algorithm defined by preload Step 12. Callback invocation
    // takes the then-current shared state; this seed does not freeze its values.
    spec fn commit_algorithm(&self, initial_state: PreloadCallbackState) -> DocsAlgoView;
}

} // verus!
