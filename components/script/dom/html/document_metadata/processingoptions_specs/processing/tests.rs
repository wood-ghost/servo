/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Regression proofs for the numbered reference algorithms.
//! The fixture supplies external-parser/Fetch observations; it does not verify
//! those primitives or assert that Servo produces a particular execution trace.

use std::sync::Arc;
use net_traits::ReferrerPolicy as NativeReferrerPolicy;
use net_traits::request::{CorsSettings as NativeCorsSettings, Destination as NativeDestination};
use vstd::prelude::*;
use super::model::*;
use super::types::*;
use super::model_trait::LinkProcessingModel;
use super::algorithms::{callbacks, cors, headers, options, request, link_types};
use super::algorithms::enumerated_attributes::{matching_keyword, determine_the_state_of_an_attribute};
use super::context_specs::fetch_context;
use super::super::predicates::{ascii_lowercase, ascii_case_insensitive_match, is_referrer_policy};
use super::super::servo_api::views as SpecViews;

verus! {

broadcast use super::algorithms::link_types::match_preload_type::runner_finishes;

pub(crate) struct Fixture {
    pub mime: Option<int>,
    pub supported: bool,
    pub url: Option<ServoUrlView>,
}

impl LinkProcessingModel for Fixture {
    type Mime = int;
    open spec fn get_decode_and_split_link_headers(&self, headers: HeaderListView) -> Seq<Seq<char>> {
        headers.map_values(|header: (Seq<char>, Seq<char>)| header.1)
    }
    open spec fn parse_link_header(&self, value: Seq<char>) -> Seq<Option<ParsedLinkHeader>> {
        if value == "missing target"@ { seq![None] }
        else if value == "two relations"@ {
            seq![Some(link("same target"@, attributes())), Some(ParsedLinkHeader {
                relation_type: "preconnect"@, ..link("same target"@, attributes())
            })]
        } else { seq![Some(link(value, attributes()))] }
    }
    open spec fn parse_mime_type(&self, value: Seq<char>) -> Option<int> { self.mime }
    open spec fn mime_supported(&self, mime: int) -> bool { self.supported }
    open spec fn is_javascript_mime(&self, mime: int) -> bool { mime == 1 }
    open spec fn is_image_mime(&self, mime: int) -> bool { mime == 2 }
    open spec fn is_font_mime(&self, mime: int) -> bool { mime == 3 }
    open spec fn mime_essence(&self, mime: int) -> Seq<char> {
        if mime == 0 { "text/css"@ } else if mime == 4 { "text/vtt"@ } else { "other/type"@ }
    }
    open spec fn encoding_parse_url(&self, href: Seq<char>, base_url: ServoUrlView) -> Option<ServoUrlView> { self.url }
    open spec fn new_request(&self) -> RequestView {
        RequestView {
            url: fixture_url(0), destination: DestinationView::from_native(NativeDestination::None), mode: RequestMode::NoCors,
            credentials_mode: CredentialsMode::Omit, use_url_credentials: false,
            policy_container: None, integrity: Seq::empty(), nonce: Seq::empty(),
            referrer_policy: ReferrerPolicyView::from_native(NativeReferrerPolicy::EmptyString),
            client: None, priority: LinkFetchPriorityView::Auto,
            initiator_type: None, other: 17,
        }
    }
    open spec fn select_image_source(&self, source_set: SourceSetView) -> Seq<char> { "selected.png"@ }
    open spec fn url_is_http_or_https(&self, url: ServoUrlView) -> bool { url.scheme == "https"@ || url.scheme == "http"@ }
    open spec fn url_origin(&self, url: ServoUrlView) -> OriginView { OriginView::Opaque { identity: 100 + url.port.unwrap_or(0) as int } }
    open spec fn same_origin(&self, left: OriginView, right: OriginView) -> bool { left == right }
    open spec fn network_partition_key(&self, environment: EnvironmentView) -> int { environment.id + 200 }
    open spec fn network_error(&self) -> ResponseView {
        ResponseView { network_error: true, body: None, other: 0 }
    }
    open spec fn unsafe_shared_current_time(&self) -> int { 1000 }
    open spec fn document_relevant_global(&self, document: DocumentView) -> int { document.id + 300 }
}

pub(crate) open spec fn fixture() -> Fixture {
    Fixture { mime: Some(0int), supported: true, url: Some(fixture_url(7)) }
}

pub(crate) open spec fn fixture_url(port: u16) -> ServoUrlView {
    ServoUrlView {
        scheme: "https"@, host: Some(HostView::Domain { value: "example.test"@ }),
        port: Some(port), ..ServoUrlView::initial()
    }
}

/// An arbitrary shared policy view; tests require preservation, not a particular
/// policy identity or an assumption about allocation/reference counting.
pub(crate) uninterp spec fn fixture_policy_container() -> Arc<PolicyContainerView>;

pub(crate) open spec fn context(effects: int) -> LinkHeaderDocumentState {
    LinkHeaderDocumentState {
        document: DocumentView { id: 1 },
        base_url: fixture_url(2),
        origin: OriginView::Opaque { identity: 3 },
        environment: EnvironmentView { id: 4 },
        policy_container: fixture_policy_container(), effects,
    }
}

pub(crate) open spec fn attributes() -> LinkAttributes { map!["as"@ => "script"@] }

pub(crate) open spec fn link(target: Seq<char>, attributes: LinkAttributes) -> ParsedLinkHeader {
    ParsedLinkHeader { target_uri: target, relation_type: "preload"@, target_attributes: attributes }
}

pub(crate) open spec fn initial() -> LinkProcessingOptionsView {
    options::initial_options(link("https://example.test/a"@, attributes()), context(0))
}

pub(crate) open spec fn apply_call(link: ParsedLinkHeader, state: LinkHeaderDocumentState) -> ApplyLinkOptionsCall {
    let before = options::initial_options(link, state);
    let (result, after) = options::apply_link_options_from_parsed_header(before, link.target_attributes, link.relation_type);
    ApplyLinkOptionsCall { before, attributes: link.target_attributes, relation: link.relation_type, after, result }
}

pub(crate) open spec fn dispatch(index: nat, link: ParsedLinkHeader, state: LinkHeaderDocumentState) -> LinkHeaderStep {
    let apply = apply_call(link, state);
    LinkHeaderStep {
        header_index: index, media: None,
        action: LinkHeaderAction::Dispatch {
            apply, source_set: None,
            process: ProcessLinkHeaderCall { relation: link.relation_type, options: apply.after },
        },
    }
}

proof fn destination_type_domain(destination: DestinationView) {
    let empty = DestinationView::from_native(NativeDestination::None);
    let script = DestinationView::from_native(NativeDestination::Script);
    assert(destination.well_formed());
    assert(destination_types().contains(destination.as_str()));
    assert(empty.as_str() == ""@);
    assert(script.as_str() == "script"@);
    assert(!destination_types().contains("fetch"@));
    assert(!destination_types().contains("SCRIPT"@));
}

proof fn referrer_policy_type_domain(policy: ReferrerPolicyView, value: Seq<char>) {
    let empty = ReferrerPolicyView::from_native(NativeReferrerPolicy::EmptyString);
    lemma_referrer_policy_valid(policy);
    lemma_referrer_policy_from_name(value);
    assert(policy.well_formed());
    assert(ReferrerPolicyView::from_name(policy.as_str()) == Some(policy));
    assert((ReferrerPolicyView::from_name(value) is Some) == is_referrer_policy(value));
    assert(ReferrerPolicyView::from_name("bad"@) is None);
    assert(ReferrerPolicyView::from_name("ORIGIN"@) is None);
    assert(ReferrerPolicyView::from_name(""@) == Some(empty));
    assert(SpecViews::referrer_policy_view(policy.as_native()) == policy);
}

proof fn cors_settings_type_domain(settings: CorsSettingsView, native: Option<NativeCorsSettings>) {
    assert(settings.well_formed());
    let view = SpecViews::cors_view(native);
    assert(match view {
        Some(settings) => native == Some(settings.as_native()),
        None => native is None,
    });
}

proof fn servo_destination_view_roundtrips(native: NativeDestination, abstract_value: DestinationView) {
    assert(SpecViews::destination_from_view(SpecViews::destination_view(native)) == native);
    assert(SpecViews::destination_view(SpecViews::destination_from_view(abstract_value)) == abstract_value);
    assert(SpecViews::destination_name(native) == SpecViews::destination_view(native).as_str());
}

proof fn domain_type_invariants_apply_to_tracked_values(
    tracked destination: DestinationView,
    tracked policy: ReferrerPolicyView,
    tracked cors: CorsSettingsView,
) {
    use_type_invariant(&destination);
    use_type_invariant(&policy);
    use_type_invariant(&cors);
    assert(destination_types().contains(destination.as_str()));
    assert(is_referrer_policy(policy.as_str()));
    assert(cors_settings_attribute_definition().states.contains(Some(cors)));
    let tracked empty_destination = DestinationView { value: NativeDestination::None };
    let tracked empty_policy = ReferrerPolicyView { value: NativeReferrerPolicy::EmptyString };
    let tracked anonymous = CorsSettingsView { value: NativeCorsSettings::Anonymous };
    use_type_invariant(&empty_destination);
    use_type_invariant(&empty_policy);
    use_type_invariant(&anonymous);
    assert(empty_destination.as_str() == ""@ && empty_policy.as_str() == ""@);
    assert(cors_settings_attribute_definition().states.contains(Some(anonymous)));
}

pub(crate) open spec fn enumerated_attribute_fixture() -> EnumerateAttribute<int> {
    EnumerateAttribute {
        states: set![1int, 2int, 3int, 4int],
        keyword_states: map!["on"@ => 1int, "yes"@ => 1int],
        invalid_value_default: Some(4int),
        missing_value_default: Some(2int),
        empty_value_default: Some(3int),
    }
}

proof fn enumerated_attribute_definition_constraints() {
    assert(ascii_lowercase("on"@) =~= "on"@);
    assert(ascii_lowercase("ON"@) =~= "on"@);
    assert(ascii_lowercase("yes"@) =~= "yes"@);
    let attribute = enumerated_attribute_fixture();
    assert(attribute.well_formed());
    // Two distinct keywords may represent the same state.
    assert(attribute.keyword_states["on"@] == attribute.keyword_states["yes"@]);
    let missing_state = EnumerateAttribute { states: set![2int, 3int, 4int], ..attribute };
    assert(missing_state.keyword_states.dom().contains("on"@));
    assert(!missing_state.states.contains(missing_state.keyword_states["on"@]));
    assert(!missing_state.well_formed());
    let invalid_default = EnumerateAttribute { invalid_value_default: Some(9int), ..attribute };
    assert(!invalid_default.well_formed());
    let same_state = EnumerateAttribute {
        keyword_states: map!["on"@ => 1int, "ON"@ => 1int], ..attribute
    };
    assert(same_state.well_formed());
    let conflicting_states = EnumerateAttribute {
        keyword_states: map!["on"@ => 1int, "ON"@ => 2int], ..attribute
    };
    assert(ascii_case_insensitive_match("on"@, "ON"@));
    assert(!conflicting_states.well_formed());
}

proof fn enumerated_attribute_state_resolution_order() {
    assert(ascii_lowercase("on"@) =~= "on"@);
    assert(ascii_lowercase("yes"@) =~= "yes"@);
    assert(ascii_lowercase("ON"@) =~= "on"@);
    assert(ascii_lowercase(""@) =~= ""@);
    assert(ascii_lowercase("bad"@) =~= "bad"@);
    assert(ascii_lowercase(" on "@) =~= " on "@);
    let attribute = enumerated_attribute_fixture();
    assert(ascii_case_insensitive_match("ON"@, "on"@));
    assert(determine_the_state_of_an_attribute(attribute, None) == EnumeratedAttributeState::State { value: 2int });
    assert(determine_the_state_of_an_attribute(attribute, Some("ON"@)) == EnumeratedAttributeState::State { value: 1int });
    assert(determine_the_state_of_an_attribute(attribute, Some(""@)) == EnumeratedAttributeState::State { value: 3int });
    assert(determine_the_state_of_an_attribute(attribute, Some("bad"@)) == EnumeratedAttributeState::State { value: 4int });
    // ASCII case matching does not strip whitespace.
    assert(matching_keyword(attribute, " on "@) is None);
    assert(determine_the_state_of_an_attribute(attribute, Some(" on "@)) == EnumeratedAttributeState::State { value: 4int });
}

proof fn enumerated_empty_keyword_precedes_empty_default() {
    assert(ascii_lowercase("on"@) =~= "on"@);
    assert(ascii_lowercase("yes"@) =~= "yes"@);
    assert(ascii_lowercase(""@) =~= ""@);
    let original = enumerated_attribute_fixture();
    let attribute = EnumerateAttribute {
        keyword_states: original.keyword_states.insert(""@, 1int), ..original
    };
    assert(ascii_case_insensitive_match(""@, ""@));
    assert(attribute.well_formed());
    assert(determine_the_state_of_an_attribute(attribute, Some(""@)) == EnumeratedAttributeState::State { value: 1int });
}

proof fn enumerated_no_state_and_default_fallthrough() {
    assert(ascii_lowercase("on"@) =~= "on"@);
    assert(ascii_lowercase("yes"@) =~= "yes"@);
    assert(ascii_lowercase("ON"@) =~= "on"@);
    assert(ascii_lowercase(""@) =~= ""@);
    assert(ascii_lowercase("bad"@) =~= "bad"@);
    let attribute = EnumerateAttribute {
        missing_value_default: None, empty_value_default: None, invalid_value_default: None,
        ..enumerated_attribute_fixture()
    };
    assert(ascii_case_insensitive_match("ON"@, "on"@));
    assert(determine_the_state_of_an_attribute(attribute, None) is NoState);
    assert(determine_the_state_of_an_attribute(attribute, Some(""@)) is NoState);
    assert(determine_the_state_of_an_attribute(attribute, Some("bad"@)) is NoState);
    assert(determine_the_state_of_an_attribute(attribute, Some("ON"@)) == EnumeratedAttributeState::State { value: 1int });
    let invalid_default = EnumerateAttribute { invalid_value_default: Some(4int), ..attribute };
    assert(determine_the_state_of_an_attribute(invalid_default, Some(""@)) == EnumeratedAttributeState::State { value: 4int });
    // Missing input returns at step 1; it does not fall through to the invalid default.
    assert(determine_the_state_of_an_attribute(invalid_default, None) is NoState);
}

proof fn enumerated_defaults_always_provide_a_state<State>(
    attribute: EnumerateAttribute<State>,
    value: Option<Seq<char>>,
)
    requires
        attribute.well_formed(),
        attribute.missing_value_default is Some,
        attribute.invalid_value_default is Some,
    ensures determine_the_state_of_an_attribute(attribute, value) is State,
{}

proof fn cors_attribute_definition_matches_standard()
    ensures cors_settings_attribute_definition().well_formed(),
{
    assert(ascii_lowercase("anonymous"@) =~= "anonymous"@);
    assert(ascii_lowercase("use-credentials"@) =~= "use-credentials"@);
    let anonymous = Some(CorsSettingsView::from_native(NativeCorsSettings::Anonymous));
    let credentials = Some(CorsSettingsView::from_native(NativeCorsSettings::UseCredentials));
    let attribute = cors_settings_attribute_definition();
    assert(attribute.states =~= set![None, anonymous, credentials]);
    assert(attribute.keyword_states.dom() =~= set!["anonymous"@, "use-credentials"@]);
    assert(attribute.keyword_states["anonymous"@] == anonymous);
    assert(attribute.keyword_states["use-credentials"@] == credentials);
    assert(attribute.missing_value_default == Some(None));
    assert(attribute.empty_value_default == Some(anonymous));
    assert(attribute.invalid_value_default == Some(anonymous));
    let absent = determine_the_state_of_an_attribute(attribute, None);
    // No CORS is a valid state. It is not the generic "no state" result.
    assert(absent == EnumeratedAttributeState::State { value: None });
    assert(!(absent is NoState));
}

proof fn cors_attribute_always_has_a_state(value: Option<Seq<char>>)
    ensures determine_the_state_of_an_attribute(cors_settings_attribute_definition(), value) is State,
{
    cors_attribute_definition_matches_standard();
    enumerated_defaults_always_provide_a_state(cors_settings_attribute_definition(), value);
}

proof fn fetch_priority_definition_and_defaults()
    ensures fetch_priority_attribute_definition().well_formed(),
{
    assert(ascii_lowercase("high"@) =~= "high"@);
    assert(ascii_lowercase("low"@) =~= "low"@);
    assert(ascii_lowercase("auto"@) =~= "auto"@);
    assert(ascii_lowercase("HIGH"@) =~= "high"@);
    assert(ascii_lowercase(""@) =~= ""@);
    assert(ascii_lowercase("bad"@) =~= "bad"@);
    let attribute = fetch_priority_attribute_definition();
    assert(ascii_case_insensitive_match("HIGH"@, "high"@));
    assert(attribute.missing_value_default == Some(LinkFetchPriorityView::Auto));
    assert(attribute.invalid_value_default == Some(LinkFetchPriorityView::Auto));
    assert(attribute.empty_value_default is None);
    assert(determine_the_state_of_an_attribute(attribute, None) == EnumeratedAttributeState::State { value: LinkFetchPriorityView::Auto });
    assert(determine_the_state_of_an_attribute(attribute, Some("HIGH"@)) == EnumeratedAttributeState::State { value: LinkFetchPriorityView::High });
    assert(determine_the_state_of_an_attribute(attribute, Some(""@)) == EnumeratedAttributeState::State { value: LinkFetchPriorityView::Auto });
    assert(determine_the_state_of_an_attribute(attribute, Some("bad"@)) == EnumeratedAttributeState::State { value: LinkFetchPriorityView::Auto });
    // Header processing checks keywords; it does not apply attribute defaults.
    assert(options::fetch_priority_keyword_state("bad"@) is None);
}

proof fn destination_translation() {
    let fetch = DestinationView::from_native(NativeDestination::None);
    let font = DestinationView::from_native(NativeDestination::Font);
    let image = DestinationView::from_native(NativeDestination::Image);
    let script = DestinationView::from_native(NativeDestination::Script);
    let style = DestinationView::from_native(NativeDestination::Style);
    let track = DestinationView::from_native(NativeDestination::Track);
    assert(link_types::preload::translate_a_preload_destination("fetch"@) == Some(fetch));
    assert(link_types::preload::translate_a_preload_destination("font"@) == Some(font));
    assert(link_types::preload::translate_a_preload_destination("image"@) == Some(image));
    assert(link_types::preload::translate_a_preload_destination("script"@) == Some(script));
    assert(link_types::preload::translate_a_preload_destination("style"@) == Some(style));
    assert(link_types::preload::translate_a_preload_destination("track"@) == Some(track));
    assert(link_types::preload::translate_a_preload_destination("SCRIPT"@) is None);
    assert(link_types::preload::translate_a_preload_destination("json"@) is None);
}

proof fn cors_keywords_and_defaults_are_different() {
    let anonymous = CorsSettingsView::from_native(NativeCorsSettings::Anonymous);
    let credentials = CorsSettingsView::from_native(NativeCorsSettings::UseCredentials);
    assert(ascii_lowercase("AnOnYmOuS"@) =~= "anonymous"@);
    assert(ascii_lowercase("anonymous"@) =~= "anonymous"@);
    assert(ascii_lowercase("use-credentials"@) =~= "use-credentials"@);
    assert(ascii_lowercase("USE-CREDENTIALS"@) =~= "use-credentials"@);
    assert(ascii_lowercase(""@) =~= ""@);
    assert(ascii_lowercase("invalid"@) =~= "invalid"@);
    assert(ascii_case_insensitive_match("AnOnYmOuS"@, "anonymous"@));
    assert(ascii_case_insensitive_match("USE-CREDENTIALS"@, "use-credentials"@));
    assert(cors::determine_cors_settings_for_token("AnOnYmOuS"@) == Some(anonymous));
    assert(cors::determine_cors_settings_for_token("USE-CREDENTIALS"@) == Some(credentials));
    assert(cors::determine_cors_settings_for_token(""@) is None);
    assert(cors::cors_settings_attribute(None) is None);
    assert(cors::cors_settings_attribute(Some(""@)) == Some(anonymous));
    assert(cors::cors_settings_attribute(Some("invalid"@)) == Some(anonymous));
}

proof fn empty_header_keywords_preserve_existing_states() {
    assert(ascii_lowercase(""@) =~= ""@);
    assert(ascii_lowercase("anonymous"@) =~= "anonymous"@);
    assert(ascii_lowercase("use-credentials"@) =~= "use-credentials"@);
    assert(ascii_lowercase("high"@) =~= "high"@);
    assert(ascii_lowercase("low"@) =~= "low"@);
    assert(ascii_lowercase("auto"@) =~= "auto"@);
    let credentials = CorsSettingsView::from_native(NativeCorsSettings::UseCredentials);
    let before = LinkProcessingOptionsView {
        crossorigin: Some(credentials), fetch_priority: LinkFetchPriorityView::High, ..initial()
    };
    let attrs = map!["crossorigin"@ => ""@, "fetchpriority"@ => ""@];
    assert(options::apply_link_options_from_parsed_header(before, attrs, "preconnect"@) == (true, before));
}

proof fn applying_missing_or_invalid_as_preserves_options() {
    let before = initial();
    assert(options::apply_link_options_from_parsed_header(before, Map::empty(), "preload"@) == (false, before));
    assert(options::apply_link_options_from_parsed_header(before, map!["as"@ => "bad"@], "preload"@) == (false, before));
    assert(options::apply_link_options_from_parsed_header(before, Map::empty(), "preconnect"@) == (true, before));
}

proof fn attribute_updates_and_frame() {
    let style = DestinationView::from_native(NativeDestination::Style);
    let no_referrer = ReferrerPolicyView::from_native(NativeReferrerPolicy::NoReferrer);
    let anonymous = CorsSettingsView::from_native(NativeCorsSettings::Anonymous);
    assert(ascii_lowercase("ANONYMOUS"@) =~= "anonymous"@);
    assert(ascii_lowercase("NO-REFERRER"@) =~= "no-referrer"@);
    assert(ascii_lowercase("HIGH"@) =~= "high"@);
    assert(ascii_lowercase("anonymous"@) =~= "anonymous"@);
    assert(ascii_lowercase("use-credentials"@) =~= "use-credentials"@);
    assert(ascii_lowercase("high"@) =~= "high"@);
    assert(ascii_lowercase("low"@) =~= "low"@);
    assert(ascii_lowercase("auto"@) =~= "auto"@);
    assert(ascii_case_insensitive_match("ANONYMOUS"@, "anonymous"@));
    assert(ascii_case_insensitive_match("HIGH"@, "high"@));
    let before = LinkProcessingOptionsView {
        source_set: Some(SourceSetView { id: 8 }),
        on_document_ready: Some(DocsAlgoView { id: 12 }),
        ..initial()
    };
    let attrs = map![
        "as"@ => "style"@, "crossorigin"@ => "ANONYMOUS"@,
        "integrity"@ => "sha256-example"@, "referrerpolicy"@ => "NO-REFERRER"@,
        "nonce"@ => "n"@, "type"@ => "text/css; charset=utf-8"@, "fetchpriority"@ => "HIGH"@
    ];
    let (result, after) = options::apply_link_options_from_parsed_header(before, attrs, "preload"@);
    assert(result);
    assert(after.destination == style);
    assert(after.crossorigin == Some(anonymous));
    assert(after.integrity == "sha256-example"@);
    assert(after.referrer_policy == no_referrer);
    assert(after.referrer_policy.as_str() == "no-referrer"@);
    assert(after.cryptographic_nonce_metadata == "n"@);
    assert(after.type_ == "text/css; charset=utf-8"@);
    assert(after.fetch_priority == LinkFetchPriorityView::High);
    assert(after.href == before.href && after.base_url == before.base_url);
    assert(after.origin == before.origin && after.environment == before.environment);
    assert(after.document == before.document && after.source_set == before.source_set);
    assert(after.policy_container == before.policy_container);
    assert(after.on_document_ready == before.on_document_ready);
}

proof fn invalid_keywords_preserve_existing_settings() {
    assert(ascii_lowercase("bad"@) =~= "bad"@);
    assert(ascii_lowercase("high"@) =~= "high"@);
    assert(ascii_lowercase("low"@) =~= "low"@);
    assert(ascii_lowercase("auto"@) =~= "auto"@);
    let before = LinkProcessingOptionsView {
        crossorigin: Some(CorsSettingsView::from_native(NativeCorsSettings::UseCredentials)),
        referrer_policy: ReferrerPolicyView::from_native(NativeReferrerPolicy::Origin),
        fetch_priority: LinkFetchPriorityView::High,
        ..initial()
    };
    let attrs = map!["crossorigin"@ => "bad"@, "referrerpolicy"@ => "bad"@, "fetchpriority"@ => "bad"@];
    assert(options::apply_link_options_from_parsed_header(before, attrs, "preconnect"@) == (true, before));
}

proof fn matching_type_order_and_essence() {
    let style = DestinationView::from_native(NativeDestination::Style);
    let fetch = DestinationView::from_native(NativeDestination::None);
    let script = DestinationView::from_native(NativeDestination::Script);
    let unavailable = Fixture { mime: None, supported: false, ..fixture() };
    assert(link_types::preload::type_matches_destination(unavailable, ""@, style));
    assert(link_types::preload::type_matches_destination(unavailable, "invalid"@, fetch));
    assert(!link_types::preload::type_matches_destination(unavailable, "invalid"@, script));
    let unsupported = Fixture { supported: false, ..fixture() };
    assert(!link_types::preload::type_matches_destination(unsupported, "text/css"@, style));
    // The fixture supplies a successfully parsed CSS record. Parameters do not
    // change its essence, and full-record equality with TEXT_CSS is not required.
    assert(link_types::preload::type_matches_destination(fixture(), "text/css; charset=utf-8"@, style));
    assert(!link_types::preload::type_matches_destination(fixture(), "text/css"@, script));
    let javascript = Fixture { mime: Some(1int), ..fixture() };
    assert(link_types::preload::type_matches_destination(javascript, "text/javascript"@, script));
}

proof fn potential_cors_request_modes() {
    let image = DestinationView::from_native(NativeDestination::Image);
    let url = fixture_url(7);
    let anonymous_settings = CorsSettingsView::from_native(NativeCorsSettings::Anonymous);
    let no_cors = cors::create_a_potential_cors_request(fixture(), url, image, None, false);
    assert(no_cors.mode == RequestMode::NoCors && no_cors.credentials_mode == CredentialsMode::Include);
    assert(no_cors.use_url_credentials);
    let fallback = cors::create_a_potential_cors_request(fixture(), url, image, None, true);
    assert(fallback.mode == RequestMode::SameOrigin);
    let anonymous = cors::create_a_potential_cors_request(fixture(), url, image, Some(anonymous_settings), true);
    assert(anonymous.mode == RequestMode::Cors && anonymous.credentials_mode == CredentialsMode::SameOrigin);
}

proof fn link_request_failure_and_fields(policy_container: Arc<PolicyContainerView>) {
    let options = LinkProcessingOptionsView {
        fetch_priority: LinkFetchPriorityView::High, integrity: "i"@, policy_container, ..initial()
    };
    assert(request::create_link_request(Fixture { url: None, ..fixture() }, options) is None);
    let result = request::create_link_request(fixture(), options);
    assert(result is Some);
    let request = result->Some_0;
    assert(request.url == fixture_url(7) && request.destination == options.destination);
    assert(request.priority == LinkFetchPriorityView::High);
    assert(request.integrity == "i"@ && request.policy_container == Some(policy_container));
    assert(request.client == Some(EnvironmentView { id: 4 }));
    assert(request::create_link_request_ensures(fixture(), options, result));
    let empty = LinkProcessingOptionsView { href: Seq::empty(), ..options };
    assert(!request::create_link_request_ensures(fixture(), empty, result));
}

proof fn preload_branches_and_commit_timing() {
    let invalid_type = LinkProcessingOptionsView {
        type_: "invalid"@, destination: DestinationView::from_native(NativeDestination::Script), ..initial()
    };
    assert(link_types::preload::preload(Fixture { mime: None, ..fixture() }, invalid_type, None) is TypeMismatch);
    assert(link_types::preload::preload(Fixture { url: None, ..fixture() }, initial(), None) is RequestFailure);
    let normal = link_types::preload::preload(fixture(), initial(), None);
    assert(normal is Fetch);
    assert(normal->Fetch_entry.response is None);
    assert(normal->Fetch_initial_controller is None);
    assert(normal->Fetch_initial_unsafe_end_time == 0);
    assert(normal->Fetch_report_timing is ReportTiming);
    assert(normal->Fetch_fetch.process_response_consume_body is ProcessResponseConsumeBody);
    assert(normal->Fetch_fetch.request == normal->Fetch_request);
    assert(normal->Fetch_commit_callback is Commit);
    assert(normal->Fetch_commit == PreloadCommit::Document { document: DocumentView { id: 1 } });
    let early_options = LinkProcessingOptionsView { document: None, ..initial() };
    let early = link_types::preload::preload(fixture(), early_options, None);
    assert(early is Fetch);
    assert(early->Fetch_request.initiator_type == Some("early hint"@));
    assert(early->Fetch_commit is OnDocumentReady);
}

proof fn image_source_selection_and_preload_key() {
    let image = DestinationView::from_native(NativeDestination::Image);
    let options = LinkProcessingOptionsView { destination: image, source_set: Some(SourceSetView { id: 8 }), ..initial() };
    assert(options.type_ =~= ""@);
    assert(image.as_str() == "image"@);
    link_types::match_preload_type::runner_finishes(fixture(), options.type_, "image"@);
    assert(link_types::preload::type_matches_destination(fixture(), options.type_, image));
    let plan = link_types::preload::preload(fixture(), options, None);
    assert(plan is Fetch);
    assert(plan->Fetch_options.href == "selected.png"@);
    let key = plan->Fetch_key;
    assert(key.url == plan->Fetch_request.url);
    assert(key.destination == image);
    assert(key.mode == plan->Fetch_request.mode);
    assert(key.credentials_mode == plan->Fetch_request.credentials_mode);
}

proof fn per_relation_dispatch_and_preconnect_permission() {
    assert(link_types::process_link_header(fixture(), initial(), "preload"@) is Preload);
    assert(link_types::process_link_header(fixture(), initial(), "stylesheet"@) is Noop);
    let options = LinkProcessingOptionsView {
        crossorigin: Some(CorsSettingsView::from_native(NativeCorsSettings::Anonymous)), ..initial()
    };
    let plan = link_types::preconnect::preconnect(fixture(), options);
    assert(plan == PreconnectPlan::MayConnect { partition_key: 204, origin: OriginView::Opaque { identity: 107 }, use_credentials: false });
    let same = LinkProcessingOptionsView { origin: OriginView::Opaque { identity: 107 }, ..options };
    let same_plan = link_types::preconnect::preconnect(fixture(), same);
    assert(same_plan is MayConnect && same_plan->MayConnect_use_credentials);
}

pub(crate) open spec fn callback_state() -> PreloadCallbackState {
    let request = fixture().new_request();
    PreloadCallbackState {
        entry: PreloadEntryView { integrity: "i"@, response: None, on_response_available: None },
        key: link_types::preload::create_preload_key(request),
        controller: Some(9int), unsafe_end_time: 0, document: Some(DocumentView { id: 1 }), process_response: Some(12int),
    }
}

proof fn response_callbacks_store_bytes_and_preserve_order() {
    let before = callback_state();
    let response = ResponseView { network_error: false, body: None, other: 10 };
    let result = callbacks::preload_response(fixture(), before, response, BodyResult::Bytes { bytes: seq![1u8, 2u8] });
    assert(result.state.unsafe_end_time == 1000);
    assert(result.state.entry.response is Some);
    assert(result.state.entry.response->Some_0.body == Some(seq![1u8, 2u8]));
    assert(result.effects.len() == 2);
    assert(result.effects[0] == CallbackEffect::ReportTiming { controller: Some(9int), global: 301 });
    assert(result.effects[1] is InvokeResponse);
    assert(result.effects[1]->InvokeResponse_callback == 12);
}

proof fn response_failure_and_waiting_consumer() {
    let before = PreloadCallbackState {
        entry: PreloadEntryView { on_response_available: Some(11int), ..callback_state().entry },
        document: None, ..callback_state()
    };
    let response = ResponseView { network_error: false, body: None, other: 10 };
    let result = callbacks::preload_response(fixture(), before, response, BodyResult::Failure);
    assert(result.state.entry.response is None);
    assert(result.effects.len() == 2);
    assert(result.effects[0]->InvokeResponse_callback == 11);
    assert(result.effects[0]->InvokeResponse_response.network_error);
    assert(result.effects[1]->InvokeResponse_callback == 12);
}

proof fn commit_reports_only_available_response() {
    let state = callback_state();
    let pending = callbacks::commit_preload(fixture(), state, DocumentView { id: 1 });
    assert(pending.len() == 1 && pending[0] is StorePreload);
    let available = PreloadCallbackState {
        entry: PreloadEntryView { response: Some(fixture().network_error()), ..state.entry }, ..state
    };
    let completed = callbacks::commit_preload(fixture(), available, DocumentView { id: 1 });
    assert(completed.len() == 2);
    assert(completed[0] is ReportTiming && completed[1] is StorePreload);
}

proof fn extraction_preserves_order_and_skips_missing_target() {
    let headers = seq![("Link"@, "a"@), ("Link"@, "missing target"@), ("Link"@, "b"@)];
    reveal_with_fuel(headers::extract_links_from_values, 4);
    reveal_with_fuel(headers::append_parsed_link_objects, 2);
    let links = headers::extract_links_from_headers(fixture(), headers);
    assert(links.len() == 2);
    assert(links[0].target_uri == "a"@ && links[1].target_uri == "b"@);
}

proof fn extraction_retains_multiple_relations() {
    reveal_with_fuel(headers::extract_links_from_values, 2);
    reveal_with_fuel(headers::append_parsed_link_objects, 3);
    let result = headers::extract_links_from_headers(fixture(), seq![("Link"@, "two relations"@)]);
    assert(result.len() == 2);
    assert(result[0].target_uri == result[1].target_uri);
    assert(result[0].relation_type == "preload"@ && result[1].relation_type == "preconnect"@);
}

proof fn empty_trace_cannot_omit_a_header() {
    let state = context(0);
    let empty = ProcessLinkHeadersTrace { steps: seq![], states: seq![state] };
    assert(headers::process_link_headers_ensures(seq![], state, LinkHeadersPhase::PreMedia, empty));
    assert(!headers::process_link_headers_ensures(seq![link("a"@, attributes())], state, LinkHeadersPhase::PreMedia, empty));
}

proof fn phase_skip_and_media_skip_continue() {
    let state = context(0);
    let media = link("a"@, attributes().insert("media"@, "not all"@));
    let phase_step = LinkHeaderStep { header_index: 0, media: None, action: LinkHeaderAction::SkipPhase };
    let phase_trace = ProcessLinkHeadersTrace { steps: seq![phase_step], states: seq![state, state] };
    assert(headers::process_link_headers_ensures(seq![media], state, LinkHeadersPhase::PreMedia, phase_trace));
    let skip = LinkHeaderStep {
        header_index: 0,
        media: Some(MediaQueryCall { document: DocumentView { id: 1 }, query: "not all"@, result: false }),
        action: LinkHeaderAction::SkipMedia,
    };
    let next = link("b"@, attributes().insert("srcset"@, Seq::empty()));
    let full = ProcessLinkHeadersTrace { steps: seq![skip, dispatch(1, next, state)], states: seq![state, state, context(1)] };
    assert(headers::process_link_headers_ensures(seq![media, next], state, LinkHeadersPhase::Media, full));
    let cut_short = ProcessLinkHeadersTrace { steps: seq![skip], states: seq![state, state] };
    assert(!headers::process_link_headers_ensures(seq![media, next], state, LinkHeadersPhase::Media, cut_short));
}

proof fn failed_application_stops_the_list() {
    let state = context(0);
    let bad = link("bad"@, map!["as"@ => "invalid"@]);
    let good = link("good"@, attributes());
    let failed = LinkHeaderStep { header_index: 0, media: None, action: LinkHeaderAction::ReturnAfterApply { apply: apply_call(bad, state) } };
    let trace = ProcessLinkHeadersTrace { steps: seq![failed], states: seq![state, state] };
    assert(headers::process_link_headers_ensures(seq![bad, good], state, LinkHeadersPhase::PreMedia, trace));
    let continued = ProcessLinkHeadersTrace { steps: seq![failed, dispatch(1, good, state)], states: seq![state, state, context(1)] };
    assert(!headers::process_link_headers_ensures(seq![bad, good], state, LinkHeadersPhase::PreMedia, continued));
}

proof fn trace_checks_application_result_not_only_call_shape() {
    let state = context(0);
    let good = link("good"@, attributes());
    let actual = apply_call(good, state);
    let invented_failure = ApplyLinkOptionsCall { result: false, ..actual };
    let step = LinkHeaderStep { header_index: 0, media: None, action: LinkHeaderAction::ReturnAfterApply { apply: invented_failure } };
    let trace = ProcessLinkHeadersTrace { steps: seq![step], states: seq![state, state] };
    assert(!headers::process_link_headers_ensures(seq![good], state, LinkHeadersPhase::PreMedia, trace));
}

proof fn native_buffering_and_notification_adapter() {
    assert(seq![1u8] + seq![2u8] =~= seq![1u8, 2u8]);
    let before = LinkFetchContextView { kind: LinkFetchContextTypeView::Preload, url: fixture_url(7), link: Some(8int), response_body: seq![1u8], capacity: 4 };
    let after = LinkFetchContextView { response_body: seq![1u8, 2u8], ..before };
    assert(fetch_context::process_response_chunk_ensures(before, seq![2u8], after));
    assert(fetch_context::resource_timing_information(before) == ("preload"@, fixture_url(7)));
    let eof = fetch_context::process_response_eof_effects(before, 3, 4, true);
    assert(eof.len() == 2 && eof[0] is SubmitTiming);
    assert(eof[1] == CallbackEffect::FireEvent { element: 8, name: "error"@ });
}

proof fn duplicate_headers_and_order_are_preserved() {
    let state = context(0);
    let same = link("same"@, attributes());
    let first = dispatch(0, same, state);
    let second = dispatch(1, same, context(1));
    let trace = ProcessLinkHeadersTrace { steps: seq![first, second], states: seq![state, context(1), context(2)] };
    assert(headers::process_link_headers_ensures(seq![same, same], state, LinkHeadersPhase::PreMedia, trace));
    let reversed = ProcessLinkHeadersTrace { steps: seq![second, first], ..trace };
    assert(!headers::process_link_headers_ensures(seq![same, same], state, LinkHeadersPhase::PreMedia, reversed));
    let omitted = ProcessLinkHeadersTrace { steps: seq![first], states: seq![state, context(1)] };
    assert(!headers::process_link_headers_ensures(seq![same, same], state, LinkHeadersPhase::PreMedia, omitted));
}

proof fn source_set_uses_both_attributes_and_null_image() {
    let state = context(0);
    let attrs = attributes().insert("imagesrcset"@, Seq::empty()).insert("imagesizes"@, Seq::empty());
    let header = link("target"@, attrs);
    let apply = apply_call(header, state);
    let source_set = CreateSourceSetCall {
        default_source: "target"@, srcset: Seq::empty(), sizes: Seq::empty(), img: None, result: SourceSetView { id: 7 }
    };
    let process = ProcessLinkHeaderCall {
        relation: "preload"@, options: LinkProcessingOptionsView { source_set: Some(SourceSetView { id: 7 }), ..apply.after }
    };
    let step = LinkHeaderStep { header_index: 0, media: None, action: LinkHeaderAction::Dispatch { apply, source_set: Some(source_set), process } };
    let trace = ProcessLinkHeadersTrace { steps: seq![step], states: seq![state, context(1)] };
    // Empty values are present, and step 2.8 does not require destination=image.
    assert(headers::process_link_headers_ensures(seq![header], state, LinkHeadersPhase::Media, trace));
    let wrong = CreateSourceSetCall { img: Some(1int), ..source_set };
    let wrong_step = LinkHeaderStep { action: LinkHeaderAction::Dispatch { apply, source_set: Some(wrong), process }, ..step };
    let wrong_trace = ProcessLinkHeadersTrace { steps: seq![wrong_step], ..trace };
    assert(!headers::process_link_headers_ensures(seq![header], state, LinkHeadersPhase::Media, wrong_trace));
}

proof fn phase_is_presence_and_snapshot_shape_is_checked() {
    let state = context(0);
    let skipped = LinkHeaderStep { header_index: 0, media: None, action: LinkHeaderAction::SkipPhase };
    let trace = ProcessLinkHeadersTrace { steps: seq![skipped], states: seq![state, state] };
    let srcset = link("a"@, attributes().insert("srcset"@, Seq::empty()));
    let image = link("a"@, attributes().insert("imagesrcset"@, Seq::empty()));
    let media = link("a"@, attributes().insert("media"@, Seq::empty()));
    let sizes = link("a"@, attributes().insert("imagesizes"@, Seq::empty()));
    assert(headers::process_link_headers_ensures(seq![srcset], state, LinkHeadersPhase::PreMedia, trace));
    assert(headers::process_link_headers_ensures(seq![image], state, LinkHeadersPhase::PreMedia, trace));
    assert(headers::process_link_headers_ensures(seq![media], state, LinkHeadersPhase::PreMedia, trace));
    assert(!headers::process_link_headers_ensures(seq![sizes], state, LinkHeadersPhase::PreMedia, trace));
    let missing_state = ProcessLinkHeadersTrace { states: seq![state], ..trace };
    assert(!headers::process_link_headers_ensures(seq![media], state, LinkHeadersPhase::PreMedia, missing_state));
}

proof fn native_view_cannot_represent_nondefault_fetch_priority(
    native_options: &crate::dom::html::document_metadata::processingoptions::LinkProcessingOptions,
) {
    assert(ascii_lowercase("high"@) =~= "high"@);
    let (_, expected) = options::apply_link_options_from_parsed_header(initial(), map!["fetchpriority"@ => "high"@], "preconnect"@);
    assert(ascii_lowercase("low"@) =~= "low"@);
    assert(ascii_lowercase("auto"@) =~= "auto"@);
    assert(ascii_case_insensitive_match("high"@, "high"@));
    assert(expected.fetch_priority == LinkFetchPriorityView::High);
    let native_view = super::super::servo_api::views::view(native_options);
    assert(native_view.fetch_priority == LinkFetchPriorityView::Auto);
    assert(native_view != expected);
}

proof fn native_invalid_policy_conversion_is_not_the_standard_guard() {
    let origin = ReferrerPolicyView::from_native(NativeReferrerPolicy::Origin);
    let empty = ReferrerPolicyView::from_native(NativeReferrerPolicy::EmptyString);
    assert(ascii_lowercase("bad"@) =~= "bad"@);
    let before = LinkProcessingOptionsView { referrer_policy: origin, ..initial() };
    let (_, expected) = options::apply_link_options_from_parsed_header(before, map!["referrerpolicy"@ => "bad"@], "preconnect"@);
    let converted = super::context_specs::native_referrer_policy_result("bad"@);
    let native_value = super::super::servo_api::views::referrer_policy_view(converted);
    assert(expected.referrer_policy == origin);
    assert(native_value == empty);
    assert(native_value != expected.referrer_policy);
}

} // verus!
