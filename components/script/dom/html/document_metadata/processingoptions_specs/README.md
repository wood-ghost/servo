# Link-processing specifications

The current procedure, baseline, trust inventory, standard findings, and
step-by-step compliance gaps are recorded in [VERIFICATION.md](VERIFICATION.md).
Its dated task log supersedes historical status below. In particular, a temporary
carried between numbered steps requires a state machine even when assigned once.

The organization follows `mime_classifier_specs`: shared predicates and requirements,
an external API layer, specification views/model interfaces, numbered algorithms,
and executable-facing context contracts. The base link-processing algorithms follow
the HTML source updated **2026-10-26** (inspection **2026-10-29**). Enumerated
attributes, CORS settings, and fetch-priority definitions were checked against the
**2026-10-30** Living Standard on **2026-10-30**.

## Read in the standard's call order

```text
processing/algorithms/headers.rs
  extract_links_from_headers                         HTML steps 1–4
  process_link_headers_ensures                        HTML steps 1, 2.1–2.9
    -> options.rs: apply_link_options_from_parsed_header   steps 1–8
       -> link_types/preload.rs: translate_a_preload_destination
        -> cors.rs: CORS keyword recognition
           -> enumerated_attributes.rs: ASCII case-insensitive keyword matching
        -> options.rs: fetch-priority keyword recognition
           -> enumerated_attributes.rs: ASCII case-insensitive keyword matching
    -> create-a-source-set                            external CSS/image algorithm
    -> link_types/dispatch.rs: process_link_header
       -> link_types/preload.rs: preload               steps 1–13
          -> type_matches_destination                 steps 1–7
          -> select-image-source                      external image algorithm
          -> request.rs: create_link_request           steps 1–11
             -> encoding-parse-a-URL                   external URL algorithm
             -> cors.rs: create_a_potential_cors_request   steps 1–5
          -> create_preload_key
          -> callbacks.rs: report_timing               step 10
          -> callbacks.rs: preload_response            steps 11.1–11.6
          -> callbacks.rs: commit_preload              steps 12.1–12.2
       -> link_types/preconnect.rs: preconnect         steps 1–8
```

The outer algorithm bodies preserve the standard's variables, conditions, ordering,
and numbered comments. Functional shadowing represents updates. Recursion represents
the extraction loop; ordered trace positions represent the processing loop. Trace
format checks live in `processing/trace.rs`, rather than determining the visible
structure of the standard algorithms.

Every `mod.rs` is a thin interface containing module documentation, module-level
attributes, declarations, and re-exports. Definitions live in named files, including
`processing/context_specs/contracts.rs`, `servo_api/external_specs.rs`, and
`processing/algorithms/link_types/dispatch.rs`.

`processing/model.rs` contains `LinkProcessingOptionsView`, including its `type_`
field and the standard's fields that Servo's runtime structure currently lacks.

`processing/types.rs` owns the abstraction's constrained value types:

- `DestinationView`: a local wrapper around `net_traits::request::Destination`,
  reusing its 24 Fetch destination variants. The wrapper has the type invariant
  `destination_types().contains(self.as_str())` and `from_native`/`as_native` operations.
  Its domain is the `destination_types()` set.
- `CorsSettingsView`: a wrapper around Servo's `CorsSettings`. The user's
  `Option<CorsSettingsView>` represents No CORS as `None`; `Some` carries
  Anonymous or UseCredentials. Its invariant checks membership of `Some(self)`
  in the standard definition's state set.
- `ReferrerPolicyView`: a wrapper around Servo's existing `ReferrerPolicy`, with
  the invariant `is_referrer_policy(self.as_str())`. `from_name` rejects invalid
  strings; attribute processing performs ASCII folding before this conversion.
- `LinkFetchPriorityView`: the closed Auto/High/Low abstraction enum, with an
  explicit enumerated-attribute keyword table and defaults.

The three enum wrappers have `#[verifier::type_invariant]`. Destination and
referrer-policy views expose strings; CORS uses state membership. Verus enforces
these invariants for exec/tracked values. The domain regression proofs also cover
arbitrary spec-mode values directly, including the types whose standalone lemmas
the user has commented out. Conversions in `servo_api/views.rs` wrap the existing
Servo enums, preserving optional CORS states.

`ServoUrlView` now contains the URL Standard's record components, including
distinct opaque/list paths, nullable query/fragment, and a contextual blob entry.
`servo_api/url.rs` labels the native getter correspondence assumptions.

`OriginView` distinguishes opaque identities from tuples of scheme, host, nullable
port, and nullable domain. The native adapter reads `ImmutableOrigin` variants;
opaque identity and host correspondence remain labelled assumptions.

`SourceSetView` contains an identity-ordered set of image sources and a source
size with CSS syntax and a contextual used value. `view_in_document` uses the
document-context adapter for lazy native size-list evaluation and URL resolution.

`EnvironmentView`,
`PolicyContainerView`, `DocumentView`, and `DocsAlgoView` are distinct typed
abstract identities. Their contents and operations remain external-model
boundaries. Requests, document snapshots, source-set call records, callbacks,
and `LinkProcessingModel` use the corresponding view types throughout.

The policy container retains the user's `Arc<PolicyContainerView>` representation,
including in document snapshots and constructed requests. Regression proofs check
preservation for an arbitrary shared policy view; the fixture's policy handle is
uninterpreted and does not assert allocation or reference-counting behavior.

## Enumerated attributes: standard-facing structure

`processing/types.rs` captures the definition as `EnumerateAttribute<State>`:
the finite state set, keyword/state mapping, and optional invalid-, missing-, and
empty-value defaults. Multiple keywords may map to one state. Its well-formedness
predicate checks membership of mapped/default states and coherence of
case-insensitive keyword matches; these are conditions on the definition, not
restrictions on attribute input strings.

`processing/algorithms/enumerated_attributes.rs` mirrors **determine the state of
an attribute**, with the standard's comments and steps **1.1–1.2, 2, 3, 4, 5** in
order. `matching_keyword` expresses the referenced ASCII-case-insensitive match
using the Infra predicate in `predicates.rs`. The mathematical choice of a matching
keyword is unambiguous at the state level for a well-formed definition.

`cors_settings_attribute_definition` and `fetch_priority_attribute_definition`
contain the literal standard tables and defaults. CORS's `None` is a valid
**No CORS** state, represented by `EnumeratedAttributeState::State { value: None }`;
it is distinct from the generic `EnumeratedAttributeState::NoState` result.
The CORS totality proof establishes that the full CORS state resolver always
returns a state before the CORS adapter extracts its value.

Link-header application steps 2 and 7 request **keyword matches**, so they use the
shared keyword relation without applying attribute defaults. The regression proofs
check this distinction for both empty and invalid header values. They also check
keyword precedence over the empty default, empty-default precedence over the
invalid default, absence of defaults, case-insensitive aliases, and definition
consistency.

## Native function coverage

| Runtime function | Specification location |
|---|---|
| `has_key_in_link_header`, `value_for_key_in_link_header` | `servo_api/link_header.rs`, with trait contracts in the executable file |
| `apply_link_options_from_parsed_header` | `processing/algorithms/options.rs` and the before/after wrapper in `processing/context_specs/contracts.rs` |
| `process_link_header` | `processing/algorithms/link_types/dispatch.rs` |
| `translate_a_preload_destination`, `type_matches_destination`, `preload` | `processing/algorithms/link_types/preload.rs` |
| `create_link_request` | `processing/algorithms/request.rs` |
| `determine_cors_settings_for_token` | `processing/algorithms/cors.rs` |
| `extract_links_from_headers`, `process_link_headers` | `processing/algorithms/headers.rs` |
| response/body/CSP/timing methods on `LinkFetchContext` | `processing/context_specs/fetch_context.rs`, linked to `algorithms/callbacks.rs` |
| `owner_global`, document/window accessors, media and source-set APIs | `servo_api/external_specs.rs` and explicit external-model interfaces |
| native preload's string, ID, key, URL, buffer, and dispatch dependencies | `servo_api/preload.rs` |

Native storage and callback adapters are not independently named WHATWG algorithms;
their comments identify the standard operation they support without inventing step numbers.
`extract_links_from_headers` retains its native external boundary while its reference
algorithm is defined over the header/parser model interface.

## Proof and model boundaries

- `processing/model_trait.rs` has the same role as `MimeClassifierModel`: it names
  algorithms delegated to MIME Sniffing, URL, Fetch, and RFC 8288. A fixture or
  `StandardModel` implementation is **not** a proof of those primitives.
- `servo_api/views.rs` is the runtime/view layer. Opaque identities, parser agreement,
  UA format support, and RequestView correspondence need their own proofs.
- `servo_api/preload.rs` supplies trusted external API contracts for the checked
  preload body. `RequestBuilder` and `PreloadKey` are transparent so their public
  fields can be observed; supporting request-field types are opaque. The ID alias
  `PipelineId` is represented through its actual `NamespaceIndex<PipelineIndex>`
  definition. String conversion/default, ID cloning, the preload-ID setter, and
  key construction have contracts matching the inspected native code. Image
  selection, document-map insertion, and background fetching currently have
  signature-only boundaries. The listener trait declaration supplies bounds only.
- `servo_api/link_header.rs` specifies the native first-match, case-sensitive lookup,
  including a first match with no value. Its valued-attribute map is not asserted
  to implement RFC parameter normalization. RFC parsing can expand one field into
  multiple relation-specific objects; the extraction model retains those objects.
- The common caller trace records all visited links, skipped links, options, calls,
  and state snapshots. The called algorithms have separate contracts. Real trace
  capture and correspondence remain explicit follow-ups.
- `PreloadPlan`, `PreloadFetch`, and callback effects describe selected operations and
  callback definitions. They do not establish that a network fetch or callback ran.
- `assume_specification` and remaining `external_body` boundaries are trusted.
  Request construction and preload keep explicitly named native execution hooks
  until their result/effect observations are connected to the reference contracts.

The abstraction-only setup temporarily marks four native bodies
`#[verifier::external_body]`:

- `apply_link_options_from_parsed_header`
- `process_link_header`
- `type_matches_destination`
- `process_link_headers`

Their pre/postconditions are retained as trusted call contracts. Skipping the last
body also suspends its internal MIME caller-precondition check. Remove these four
attributes to resume the native proofs. Reference algorithms, model regression
proofs, and requirement lemmas remain enabled. The separate `external_body`
attribute on native `preload` is currently commented out for the feature check below.

Current agreement work includes invalid-keyword preservation, missing fetch priority,
MIME-parser/UA-support correspondence, and CSS **essence** comparison rather than
full MIME-record equality. A failed proof alone is not a demonstrated runtime defect;
the regression proofs separately expose concrete model/representation distinctions.

## Checks

`processing/tests.rs` contains regression proofs for the reference algorithms and
their boundaries. They cover typed domains, tracked type-invariant use, enum-view
conversions, optional CORS states, shared policy-container and document-algorithm
preservation, control flow, request construction, early hints, callback order,
parsing/list order, and rejected traces.
They are included in the focused package verification run.

From the Servo root:

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus -p servo-net-traits -p servo-script --locked -- \
  --compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20
```

This disables both lifetime and trait-conflict checking. Model checks and native
body proof obligations must be reported separately; it is not a complete safety or
browser-conformance verification.

To check the reference proofs independently of native `preload` proof failures,
select the script package and the proof-bearing leaf modules:

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus -p servo-script --locked -- \
  --compile --no-lifetime --no-trait-conflicts \
  --verify-module dom::html::document_metadata::processingoptions_specs::processing::tests \
  --verify-module dom::html::document_metadata::processingoptions_specs::processing::types \
  --verify-module dom::html::document_metadata::processingoptions_specs::requires \
  --verify-module dom::html::document_metadata::processingoptions_specs::processing::algorithms::headers \
  --verify-module dom::html::document_metadata::processingoptions_specs::servo_api::link_header
```

This is explicitly partial verification. It covers the current regression proofs,
domain/requirement lemmas, and recursive helpers. The umbrella `mod.rs` files are
thin interfaces, so the leaf modules are named explicitly. Selecting only
`servo-script` keeps its module filters from being forwarded to `servo-net-traits`;
Cargo-Verus still produces the dependency's Verus metadata.

### Enumerated-attribute checks (2026-10-30)

Using Verus `0.2026.10.29.62a9ca4`, Rust 1.99.1, Servo base commit `56404a45064`
plus working changes, with both lifetime and trait-conflict checking disabled:

- The reference-module command above: **49 verified, 0 errors**, including all
  **40 regression proofs**.
- The full focused script command: **49 verified, 1 failing native preload check**,
  with the three diagnostics detailed below.
- Native `cargo +1.97.1 check -p servo-script --lib --locked --offline`: passed.

The native preload body stays active for feature checking. The passing reference
run checks the actual specification files and Servo view adapters in their normal
crate context.

### Preload body feature check (2026-10-30)

Native `preload` now uses nested `if` / `if let` statements in place of its let-chain,
with `external_body` commented out. With the external support in `servo_api/preload.rs`,
the entire body passes Verus's frontend and reaches proof checking. Using Verus
`0.2026.10.29.62a9ca4`, Rust 1.99.1, Servo base commit `56404a45064` plus working
changes, and the focused command above:

- Native `cargo +1.97.1 check -p servo-script --lib --locked --offline`: passed.
- Focused script verification: **49 verified, 1 error**. The failing preload check
  produces three diagnostics: the nonempty-`href` precondition at
  `create_link_request`, and `preload_execution_ensures` at the early return and
  normal function exit.

These are proof obligations, not missing-type or unsupported-feature diagnostics.
The external declarations are trusted boundaries; they establish frontend support
under the stated exclusions, not native preload conformance.

Before adding these declarations, the same date's feature probe stopped with
**9 frontend errors**:
  - Opaque-field access: `request.url`, because `ExRequestBuilder` was `external_body`.
  - Missing type support: `USVString`, `PreloadId`, `PreloadKey`, and `UrlWithBlobClaim`.
  - Missing method specifications: `SourceSet::select_image_source_from_source_set`,
    `RequestBuilder::preload_id`, `PreloadKey::new`, and `UrlWithBlobClaim::url`.

### Earlier verification snapshots

Recorded on **2026-10-30** after propagating the user's typed
`LinkProcessingOptionsView` layout through the abstraction, with Verus
`0.2026.10.29.62a9ca4`, Rust 1.99.1, and Servo base commit
`56404a45064` plus the working changes:

- Fresh focused script verification: **42 verified, 0 errors**, including all
  **31 regression proofs** and the domain/requirement lemmas.
- Native `cargo +1.97.1 check -p servo-script --lib --locked --offline`: passed.

Rechecked on **2026-10-30** after replacing the duplicate destination enum with
`DestinationView`, a wrapper around Servo's existing enum: **40 verified, 0 errors**;
the native check passed. Wrapping/unwrapping round trips and tracked construction
with the local type invariant are covered by the existing regression proofs.

The four marked native bodies are trusted in this result. Their five earlier
failed checks are suspended; the abstraction's proofs remain enabled.

Earlier native-proof baseline, recorded on 2026-10-29 using the same Verus version and Rust 1.99.1,
Servo base commit `56404a45064` plus the working changes:

- Model-only run: **26 verified, 0 errors**, explicitly partial module verification.
- Full script run: **34 verified, 5 failed verification checks**. The failures are
  native attribute-update/type-matching agreement, per-link processing effects,
  header-loop effects, and the inherited MIME caller requirement.
- Native `cargo +1.97.1 check -p servo-script --lib --locked --offline`: passed.

Rechecked on **2026-10-30** after extracting the thin module interfaces, using the
same Verus version and Servo base commit: model-only **26 verified, 0 errors**;
full script **34 verified, 5 failed verification checks**, at the same native proof
obligations. The saved post-extraction native-check output was also reviewed and
confirmed successful.

With this `cargo-verus` build, changing only the forwarded verifier arguments reused
cached model-only diagnostics. A fresh full script run was obtained by clearing
only that package's partial-verification artifacts before rerunning the full command:

```sh
cargo +1.99.1 clean -p servo-script --target-dir target/verus-partial
```

The model checks use configured primitive observations. They do not verify RFC/URL
parsers, user-agent decoder support, network I/O, or the connection of a trace to
the running browser.
