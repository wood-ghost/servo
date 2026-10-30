# WHATWG correspondence task log

## Procedure adopted 2026-10-30

Priority: preserve executable meaning; then step-by-step standard correspondence;
then explicit trust. Passing Verus does not prove that a model equals the standard.
Use the user's eleven-point self-check before each task commit. Each completed
task is checked with the crate command below, committed separately, followed by
`git pull --ff-only` and `git push` on `mime_classifier`. Stop if fast-forward
pull is impossible. Do not change Verus, weaken a standard result contract, or
alter executable behavior to obtain a proof.

Form selection uses the first applicable row:

1. `state_machine!` for state updates, accumulation, non-local control flow,
   **or a temporary carried between numbered steps even when assigned only once**
   (explicitly approved by the user after the initial audit).
2. `forall` / `exists` for independent iterations establishing a property.
3. Direct `spec fn` for sequential return branches without cross-step variables.

One numbered step/substep has one transition/branch/clause. Put its complete
verbatim text immediately above the code. Keep linked algorithms separate,
inputs immutable, and assertions as obligations. Record ambiguous standard text
as `FINDING`. Keep the original code immediately below each temporary rewrite
label. Mark extra executable requirements `IMPLICIT` (with a witness or
"none found") or `INHERITED from <callee>`. At an external-input boundary,
record the unproved requirement as an assumption rather than hiding it.

## Command and toolchain

Run from `/home/yizhiy/Desktop/servo`:

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus -p servo-net-traits -p servo-script --locked -- \
  --compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20
```

Both lifetime and trait-conflict checking are disabled. Counts refer to the
reported script verification, not every dependency or browser behavior. The
installed binary reports `0.2026.10.29.62a9ca4.dirty`, release, Rust 1.99.1.
The current Verus source HEAD is `dea854413`; that is not the installed binary's
reported revision. No verifier changes were made for these tasks.

## Setup checkpoint: labelled existing boundaries

Base Servo revision: `56404a45064`, plus the existing uncommitted link-processing
work. This checkpoint preserves that accumulated setup/model extraction as one
dependency-complete checkpoint; it does not retroactively claim task-sized
commits or compliance for the older models. The lockfile and MIME iterator API
adaptation are required by this installed vstd. Generated history/logs are excluded.

Unchanged baseline on 2026-10-30: **49 verified, 0 errors**, command above.
The saved native `preload` has an active `external_body` annotation. Therefore
this result supersedes the earlier active-body failure as a current baseline;
it does not discharge the earlier obligations. Post-label result on 2026-10-30:
**49 verified, 0 errors**, same full focused command. Runtime bodies unchanged
by the label task; existing warning about `IntoStaticStr` remains.

### Algorithms inventoried

HTML snapshot inspected 2026-10-30, page last updated 2026-10-30. URLs below
were taken from the executable source links and resolved to their current pages.
The existing specifications retain older inspected dates until individually
migrated. "Required form" is the audit decision, not a claim of completion.

| Algorithm URL | Required form and reason | Current spec item / self-check |
| --- | --- | --- |
| [match-preload-type](https://html.spec.whatwg.org/multipage/links.html#match-preload-type) | State machine: MIME temporary used by steps 3–6 | `algorithms/link_types/preload.rs::type_matches_destination`; migration pending |
| [translate-a-preload-destination](https://html.spec.whatwg.org/multipage/links.html#translate-a-preload-destination) | Direct: conditional return then final return | same file, `translate_a_preload_destination`; linked Fetch translation is still inlined |
| [preload](https://html.spec.whatwg.org/multipage/links.html#preload) | State machine: options/request/controller updates and callbacks | same file, `preload`; functional operation plan, migration pending |
| [create-a-link-request](https://html.spec.whatwg.org/multipage/semantics.html#create-a-link-request) | State machine: URL/request temporaries and field updates | `algorithms/request.rs`; migration pending |
| [apply-link-options-from-parsed-header-attributes](https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes) | State machine: destination temporary and options updates | `algorithms/options.rs`; migration pending |
| [extract-links-from-headers](https://html.spec.whatwg.org/multipage/semantics.html#extract-links-from-headers) | State machine: list accumulation and continue | `algorithms/headers.rs`; recursive model, migration pending |
| [process-link-headers](https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers) | State machine: loop, temporaries, updates and early return | `algorithms/headers.rs`; supplied-trace relation, migration pending |
| [preload header dispatch](https://html.spec.whatwg.org/multipage/links.html#link-type-preload:process-a-link-header) | Unnumbered linked algorithm invocation; retain as dispatch adapter | `algorithms/link_types/dispatch.rs`; not a newly numbered algorithm |
| [preload element caller](https://html.spec.whatwg.org/multipage/links.html#link-type-preload:fetch-and-process-the-linked-resource-2) | State machine: options/destination temporaries and update | `htmllinkelement.rs::handle_preload_url` at baseline line 933; not verified |

### Step audit of the existing chain

| Standard step | Existing spec item | Self-check result |
| --- | --- | --- |
| Type matching 1, 2 | `type_matches_destination` first branches | No: form declaration missing |
| Type matching 3, 4, 5 | MIME parse/match/support branches | No: shared temporary is not a machine field |
| Type matching 6 | category disjunction | No: step text paraphrased |
| Type matching 7 | final false | No: machine migration pending |
| Request 1 | `create_link_request` recommends; `_ensures` conjunct | No: assertion also appears in result; must become an obligation |
| Request 2–11 | functional shadowing | No: state-machine form and verbatim text pending |
| Preload 1–13, 11.1–11.6, 12.1–12.2 | preload plan and `callbacks.rs` | No: merged/paraphrased text and effect correspondence pending |
| Attribute application 1, 1.1–1.4, 2–8 | `options.rs` | No: state-machine form and full comments pending |
| Extraction 1–4, 3.1–3.3 | `headers.rs` recursion | No: state-machine form pending |
| Header processing 1–2, 2.1–2.9 | `headers.rs` trace predicate | No: ordered trace is not yet the required machine |

### Requirements and boundary obligations

| Clause | Witness / reason | Discharge or inheritance |
| --- | --- | --- |
| `create_link_request_requires`: nonempty href | `href = ""` reaches Servo's assert | Native preload must establish this **after** source selection; not discharged |
| Preload's type-match requirement | `type = "invalid"`, destination Style causes Servo's assertion rather than standard early return | Header and element callers guard the call; bodies remain unverified |
| MIME parser input requirement | none found; existing unproved parser-model correspondence | Inherited by `type_matches_destination`, then preload/header dispatch; eventual HTTP/DOM assumptions are outstanding |
| MIME classifier input requirement | none found for this integration; inherited existing classifier restrictions | Same chain; no claim that all web input meets it |
| Clone requirement in `ToOwned` contract | Generic Clone need not return an equal value | Inherited from `T::clone` |

No new input restriction is introduced by this label-only task. Proof-lemma
hypotheses in the legacy tree still need their own audit; they are not executable
input assumptions. Native result/effect hooks are uninterpreted scaffolds rather
than standard result contracts; replacing them is an outstanding Section 7 task.

### Trust inventory by kind

- **scope**: native header lookup, attribute application, dispatch, request
  creation, type matching, preload, extraction, header loop, fetch context and
  listeners. External Servo types/APIs in `servo_api/external_specs.rs` and
  `servo_api/preload.rs` name their owning module/function and removal condition.
  Six native projections in `servo_api/views.rs` explicitly label correspondence
  assumptions. Uninterpreted primitives/effect hooks remain unresolved boundaries;
  they do not by themselves prove observations of actual execution.
- **external-lib**: content-security-policy 0.9.0, nom-rfc8288 0.4.0, url 2.5.8,
  mozjs 0.26.9, http 1.5.0, bytes 1.12.1, Rust core/alloc 1.99.1 in local API
  support. Validation says "not yet": reading implementation or proving a model
  is not an independent runtime test. The pre-existing transitive MIME/vstd trust
  inventory is not fully migrated to this label format in this checkpoint.
- **verus-gap**: native string-pattern match and `match_ignore_ascii_case!` const
  expansion. Reports are historical, not freshly reproduced feature tests.
- **rewrite**: three let chains expanded into nested conditionals, with original
  source retained immediately below labels (preload, media guard, image attributes).

### Verus limitations / repro status

| Feature | Minimal shape | Diagnostic evidence / disposition |
| --- | --- | --- |
| Let chains | `if flag && let Some(x) = value { use_value(x); }` | Historical unsupported-feature report; exact diagnostic not retained here; labelled rewrites, fresh probe pending |
| String match patterns | `match s { "fetch" => true, _ => false }` | Historical unsupported-feature report; exact diagnostic not retained here; trusted body, fresh probe pending |
| Const block from macro | `match_ignore_ascii_case! { s, "anonymous" => true, _ => false }` | Historical unsupported const-block report; fresh probe pending |
| IntoStaticStr derive | `#[derive(IntoStaticStr)] enum E { A }` | Current warning: "Verus doesn't know how to handle this automatically derived item; ignoring it"; external enum |

### Standard findings (not implementation fixes)

- `FINDING`: HTML type matching accepts a **preload destination**, including
  `fetch`, while the options field is a Fetch **destination type**, where `fetch`
  has already translated to the empty string. Keep the literal algorithm separate
  from the native translation adapter; do not silently change its step 2.
- `FINDING`: attribute application step 7 assigns a fetch-priority **keyword**,
  while the record definition calls the field a **state**. Record the table-based
  interpretation explicitly when migrating it.
- `FINDING`: extraction invokes RFC parsing as a singular link object, whereas
  the native parser can produce multiple link/relation objects. The representation
  relation is outstanding.
- HTML itself tracks URL encoding context in issue #9715 and preload event
  behavior in issue #1142. Neither issue is resolved by these models.

### Eleven-point self-check for this checkpoint

1. Yes for new edits: comments/labels only; earlier let-chain rewrites preserved.
2. No: legacy step mapping gaps tabulated above.
3. No: legacy comments include paraphrases.
4. No: legacy loops/state updates await machine conversion.
5. No: functional shadowing remains in legacy algorithms.
6. No: legacy shortened names/text await review.
7. No: request/preload result predicates still conjoin assertions.
8. No: Fetch destination translation is still inlined.
9. Yes for local executable/API annotations; transitive MIME/vstd label audit remains.
10. Yes for the target's executable requirements and local ToOwned contract;
    legacy proof hypotheses await audit.
11. Yes for this task: no specification semantics changed to obtain a pass.

### Not done

Native conformance, actual effect/trace capture, full detailed object views,
state-machine migrations, the conditional type rename, fresh limitation probes,
and caller verification remain tasks. This checkpoint is **trusted-only for the
native chain**, with already-existing reference proofs enabled. The 49 count
must not be described as 49 verified native chain functions.

## Task: seven-step type matching state machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#match-preload-type>.
Snapshot: HTML last updated and inspected 2026-10-30. Form: row 1, because
`mimeTypeRecord` is carried between steps 3–6, using the approved clarification.
Source: `processing/algorithms/link_types/match_preload_type.rs`.

| Step | Spec item | Self-check |
| --- | --- | --- |
| 1 | `match_preload_type::step1`, Step1 | Yes: empty type returns true |
| 2 | `match_preload_type::step2`, Step2 | Yes: literal `fetch` returns true |
| 3 | `match_preload_type::step3`, Step3 | Yes: named MIME parser sets the temporary |
| 4 | `match_preload_type::step4`, Step4 | Yes: parsing failure returns false |
| 5 | `match_preload_type::step5`, Step5 | Yes: named UA-support predicate |
| 6 | `match_preload_type::step6`, Step6 | Yes: all five verbatim bullets and named MIME predicates |
| 7 | `match_preload_type::step7`, Step7 | Yes: return false |

The generated transition relations define the runner's successors. The
`advance_correct` proof supplies generated witnesses, `runner_finishes` proves
termination and a derived Boolean result theorem, and `transition_properties`
proves input preservation, absence of terminal outgoing edges, and the
successful-parse condition needed before reading the MIME temporary. The
seven-advance bound is proved sufficient; it is not assumed fuel. Terminal
stuttering is a scheduling convention, not a machine transition or standard step.
Existing branch/essence/preload regressions now use this machine's result.

Checks: full focused command above **52 verified, 0 errors**; native
`cargo +1.97.1 check -p servo-script --lib --offline` passed with the same
`BINDGEN_EXTRA_CLANG_ARGS`. The native check resolved the new script dependency
on the same `verus_state_machines_macros` version already used by net-traits;
the subsequent Verus run used `--locked`. Both disabled checks remain disabled.

Labels added/removed: none. No executable changes or new implicit preconditions.
The six linked MIME operations use mathematical function parameters, with URLs;
their correspondence remains the existing `LinkProcessingModel` boundary.
The native type-checking body is still labelled trusted, not newly proved.

Verus integration details: generated public relations cannot call crate-private
trait methods (`in pub open spec function, cannot refer to private function`).
Use a public mathematical operation bundle inside the crate-private module,
without widening the existing data views. `take_step` functions are proof-mode
witness constructors, so the spec runner selects through the generated relations.
Generic `case_on_next!` requires `::<M>`; recursion-polarity annotations use the
item-level form. These are resolved API/visibility requirements, not new trusted
workarounds or verifier modifications.

FINDING retained: preload versus Fetch destination domains. The literal machine
takes the standard's string; the native adapter maps empty destination to
`fetch` outside all numbered steps. No other standard finding introduced.

Self-check 1–11: **yes for this algorithm**. There are no `Assert:` steps,
loops, position updates, or extra executable requirements in this task.
Initialization and scheduling/proof helpers are labelled as infrastructure;
they are not additional numbered algorithm steps. Legacy gaps in other
algorithms remain as tabulated in the setup audit.

Not done: native parser/UA/category correspondence, remaining algorithms and
views, and native caller proofs. Setup checkpoint `5adfff23466` was pulled
fast-forward (already current) and pushed to `origin/mime_classifier`.

## Task: conditional enumerated-attribute rename (2026-10-30)

The pre-edit repository search found no `EnumerateAttribute` definition in
Rust/TOML/WebIDL. Renamed the definition, impl, tables, resolver arguments,
fixtures and README to exactly `EnumerateAttribute`; the separate
`EnumeratedAttributeState` result type is unchanged. This is a name-only
abstraction task, not a new standard algorithm or a claim to have repaired the
legacy resolver's step layout. Definition URL and snapshot remain
<https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute>,
inspected 2026-10-30. Existing step map: 1/1.1/1.2/2/3/4/5 in
`determine_the_state_of_an_attribute`; comments/layout still await their own task.

Full focused command above: **52 verified, 0 errors**. Labels added/removed:
none. Findings: none new. Implicit preconditions and trust inventory unchanged.
Self-check 1 and 9–11: yes for the rename; 2–8: no new algorithm translation,
and the legacy gaps remain reported rather than being certified by this rename.
Not done: detailed views, remaining algorithm migrations, native correspondence.
State-machine task `d3c7945a968` was pulled fast-forward (already current) and pushed.

## Task: URL record and native observations (2026-10-30)

Definition: <https://url.spec.whatwg.org/#concept-url>, snapshot 2026-10-19
(`845bafca22fb0c5a42c4bf384a9bdea525a2f10d`), inspected 2026-10-30.
`ServoUrlView` now contains scheme, username, password, host, port, path, query,
fragment, and blob URL entry, with the complete field-definition sentences above
the fields. Supporting host/path variants preserve empty host versus null and
opaque string versus path list. Defaults and ASCII component-domain predicates
are explicit. Native parsing validity is not assumed merely by constructing a view.

This task adds a record definition, not a numbered parsing/serialization algorithm.
Definition-to-item audit: scheme→scheme, username→username, password→password,
host→host, port→port, path→path, query→query, fragment→fragment,
blob URL entry→blob_url_entry: **yes**, names/types/defaults preserved.
ASCII membership uses row 2 (`forall` over independent code points).

`servo_api/url.rs` supplies nine labelled native getter contracts. `None` native
password maps to the empty string; `port()` does not substitute a default port.
Host and path serialization remain separately labelled linked boundaries.
`native_url_record` is a labelled correspondence assumption. Blob-entry context
is supplied by `url_view_with_blob_entry`: the context-free projection uses None
because ServoUrl itself has no blob-store field. Connecting the actual blob claim
and environment to that contextual argument remains an execution obligation.

Full focused command above: **52 verified, 0 errors**. Labels added: scope for
nine getters and two serializers; the existing URL-projection label now names
`native_url_record`. Labels removed: none. No executable or implicit-precondition
changes. Existing fixtures now carry URL records rather than integer URL IDs.
Verus requires external contract visibility to match public Rust methods;
public **closed** projections expose their contracts without widening the local
standard-record types. No verifier changes or trusted workaround introduced.

Self-check: 1, 3, 6, 8–11 yes for this view task; 2, 4, 5, 7 have no numbered
algorithm steps to audit. Existing algorithm gaps remain reported. FINDING: none
new; the absence of blob cache storage in ServoUrl is a representation gap.
Not done: URL serializer/equality algorithm, native parsing correspondence,
blob-store observation, and the other requested views.

## Task: origin record and native projection (2026-10-30)

Definition: <https://html.spec.whatwg.org/multipage/browsers.html#concept-origin>,
HTML snapshot/inspection 2026-10-30. `OriginView` has opaque identity and tuple
variants. Tuple field map: scheme→scheme, host→host, port→port, domain→domain;
all four definition sentences are directly above the corresponding fields.
Opaque identity supports equality only; no serializable reconstruction is claimed.

`ImmutableOrigin` is now a transparent external enum. Its actual Tuple fields
are projected directly, with the stored effective port normalized using the URL
default-port table. `OpaqueOrigin` and url 2.5.8 `Host` remain labelled opaque
types; their identity/host projections remain labelled assumptions. Native
ImmutableOrigin has no document.domain state: the view sets domain to None,
and `with_domain` takes explicit supplemental MutableOrigin/OriginSnapshot data.

Full focused command: **52 verified, 0 errors**. Labels added: scope for
OpaqueOrigin/opaque identity; external-lib for Host/host projection. The former
whole-origin uninterpreted projection is replaced by a defined variant adapter;
the external-type scope label remains. No runtime edits or implicit preconditions.
Self-check: 1, 3, 6, 8–11 yes for this record/adapter; 2, 4, 5, 7 have no numbered
algorithm in this task. Findings: none new. No new Verus limitation encountered.
Not done: same-origin/effective-domain/same-origin-domain algorithms (next tasks),
mutable-domain native observation, origin serialization and broader site rules.

## Task: same-origin equality (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/browsers.html#same-origin>,
snapshot/inspection 2026-10-30. Form: row 3, ordered return branches, no carried
temporary. Map in `algorithms/origin.rs::same_origin`: step 1→opaque identity
branch; step 2→tuple scheme/port/host branch; step 3→false fall-through. All three
step comments are verbatim. Host equality is a separately named URL predicate.
`origin_equals` follows the origin definition by calling this algorithm, rather
than structural equality of records including the mutable domain.

Full focused command: **54 verified, 0 errors**. The two new proofs establish
symmetry and invariance under domain changes. StandardModel and test fixtures
now use this algorithm. Self-check 1–11: yes for this task; no assertions,
updates, loops, new executable preconditions or trust boundaries. Labels
added/removed: none. Findings/Verus limitations: none new. Native host/opaque
correspondence remains trusted as recorded above. Not done: same-origin-domain,
effective domain, serialization/site rules, other views and native caller proofs.

## Task: effective domain (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/browsers.html#concept-origin-effective-domain>,
snapshot/inspection 2026-10-30. Form: row 3, sequential returns without carried
variables. `algorithms/origin.rs::effective_domain` maps step 1→opaque/null,
step 2→non-null domain, step 3→host, each with verbatim text. Full focused
command: **54 verified, 0 errors**. Self-check 1–11: yes for this task; no
new trust labels, implicit preconditions, runtime changes or findings.
No claim of native domain accessor verification. Remaining tasks are unchanged.

Additional limitation probe (2026-10-30): `spec fn f(a: bool) -> bool {
if a { return true; } false }` is rejected with **"return is not allowed in
pure context"**. Reproduced with the installed `verus` in
`/tmp/opencode/spec_return_probe.rs`; no verifier code changed. This affects
literal nested returns in same-origin-domain; its control-flow representation
must preserve the parent step and both substeps rather than merging conditions.

## Task: source-set view and contextual native adapter (2026-10-30)

Definitions: <https://html.spec.whatwg.org/multipage/images.html#source-set>,
`#image-source`, `#source-size-2`; HTML snapshot/inspection 2026-10-30.
Record map: ordered image sources→`image_sources`, source size→`source_size`.
Image source URL→`url`, optional width/pixel-density descriptor→`descriptor`.
Definition sentences are directly above the corresponding records/fields.
`OrderedImageSourcesView` uses a sequence with a local invariant forbidding
duplicate **object identities**. It retains ordering and does not collapse two
separately created sources with the same URL. A regression proves that distinction.

Source-size CSS syntax and its contextual used CSS-pixel value are distinct.
Servo stores a lazy `SourceSizeList` and evaluates it against a Document to Au;
the model does not equate the list with a resolved size. Native source URL
strings also require contextual resolution. `source_set_view_in_context` takes
that document explicitly and `view_in_document` now uses it. The two primitive
observations are labelled scope boundaries. Context-free `source_set_view`
remains a labelled unresolved projection for callers that supply no document.
Finite density and positive infinity are separate; f64 rounding/invalid-value
correspondence and parser validity are not silently assumed proved.

Full focused command: **55 verified, 0 errors**. Labels added: scope for
contextual image-source and source-size observations; none removed. No new
executable requirements or runtime edits. Self-check 1, 3, 6, 8–11 yes for this
view task; 2, 4, 5, 7 have no numbered algorithm. Quantified uniqueness checks
are definition invariants, not translations of image-selection steps. Findings:
native lazy-size and unresolved-string representation gaps, not standard defects.
Not done: CSS/URL parsing, concrete contextual observation and image selection/
density-normalization proofs; other requested views and native chain obligations.

## Task: full environment record and native context (2026-10-30)

Definitions: <https://html.spec.whatwg.org/multipage/webappapis.html#environment>
and `#environment-settings-object`, snapshot/inspection 2026-10-30. The base
record's complete field map is id→id, creation URL→creation_url, top-level
creation URL→top_level_creation_url, top-level origin→top_level_origin, target
browsing context→target_browsing_context, active service worker→active_service_worker,
execution ready flag→execution_ready_flag. Top-level origin explicitly permits
null, an origin, or an implementation-defined value. The id is an opaque string.

Settings-object observations are a **separate extension**, preserving the base
environment distinction: realm execution context, module map, API base URL,
origin, has-cross-site-ancestor, policy container, cross-origin isolated
capability, and time origin. Module-map URL keys use canonical serialization
keys rather than structural URL-record equality including blob cache state.

`servo_api/environment.rs` supplies defined context adapters and three labelled
GlobalScope getter contracts. RequestClient supplies only origin/policy
observations here; it is not asserted to store the full environment. Remaining
base/settings data are explicit `EnvironmentSupplement`/`SettingsSupplement`
inputs. Inspected owners: GlobalScope creation/top-level/API URLs;
WorkerGlobalScope.execution_ready (`is_execution_ready`); Performance.time_origin.
Realm/module-map/navigation/service-worker observations still need their owning
modules' correspondence. `view_in_environment` allows explicit context at the
options boundary. The old context-free RequestClient projection remains labelled
unresolved, rather than claiming missing fields are derivable from RequestClient.

Full focused command: **55 verified, 0 errors**. Labels added: scope for three
GlobalScope observations/getters and two RequestClient field observations; none
removed. No executable changes or extra input requirements. Field-map self-check:
yes for all seven plus eight extension fields. Overall self-check 1, 3, 6, 8–11:
yes for this view task; 2, 4, 5, 7 have no numbered algorithm. FINDING/Verus
limitations: none new. Not done: context production, global ID uniqueness,
navigation/worker/realm/module-map proofs, policy contents, remaining algorithms
and native chain obligations. These context inputs are not browser-state proofs.

## Task: complete policy-container fields and native adapter (2026-10-30)

Definitions: <https://html.spec.whatwg.org/multipage/browsers.html#policy-container>
and `#embedder-policy` (HTML snapshot/inspection 2026-10-30);
<https://w3c.github.io/webappsec-subresource-integrity/#integrity-policy>
(Editor's Draft 2026-03-20); <https://w3c.github.io/webappsec-csp/#csp-list>
and `#content-security-policy-object` (Editor's Draft **2026-10-27**), both
inspected 2026-10-30.

Definition map: CSP list→csp_list, embedder policy→embedder_policy, referrer
policy→referrer_policy, integrity policy→integrity_policy, report-only integrity
policy→report_only_integrity_policy: **yes**. Nested embedder and integrity
fields/defaults are explicit. CSP includes policies plus self-origin; policies
include directive set, disposition and source, and directives contain name/value.
Ordered-set validity is an explicit predicate; projecting a native Vec does not
prove its validity or silently remove duplicate tokens.

`servo_api/policy_container.rs` projects the actual native public fields and
CSP records transparently. Supplemental context supplies fields absent from
native code: CSP self-origin; embedder reporting endpoint/report-only value and
endpoint; integrity and report-only integrity policies. Native Credentialless
is absent, while the standard-facing enum retains it. Native `None` CSP list
projects to empty policies. The shared-handle correspondence is a defined
relation on Arc contents, preserving the user's Arc representation without an
allocation/reference-counting assumption. The existing context-free Arc view
remains an unresolved labelled bridge until callers supply this relation.

Full focused command: **55 verified, 0 errors**. Labels added: five external-lib
type declarations for content-security-policy 0.9.0, two scope type declarations
for native embedder policy/value. Removed opaque-body annotation from native
PolicyContainer; its type trust label remains. No runtime edits or new requirements.
Self-check 1, 3, 6, 8–11 yes for this view task; 2, 4, 5, 7 no numbered algorithm.
No new Verus limitation. Default referrer policy is StrictOriginWhenCrossOrigin.

FINDING: HTML initializes an "empty" CSP list, whereas the current CSP definition
also requires self-origin without a default there. `initial(self_origin)` makes
that missing context explicit. FINDING: HTML clone-policy-container step 5 copies
integrity policy but the inspected algorithm does not copy report-only integrity
policy. Do not silently add such a step when that algorithm is formalized.
Not done: policy parsing/enforcement/clone algorithms, native context production,
remaining origin/URL operations and legacy algorithm migrations/native proofs.

## Task: type-matching implicit correspondence requirements (2026-10-30)

Added `type_matches_standard_requires`, separate from inherited parser/classifier
requirements, to the native type checker. Standard URL/snapshot/form/step map
remain the seven-step machine recorded above. No `ensures` or standard step was
weakened. The added requirement spells out primitive agreement, not equality to
the function's final result.

| Clause | Witness / evidence | Current propagation |
| --- | --- | --- |
| Preload destination domain on non-short-circuit paths | Audio + `audio/mpeg` is outside the standard preload-destination domain | Native type checker; callers next |
| Native parser result agrees with model parser | none found; unproved model correspondence, not a demonstrated implementation defect | Native type checker; callers next |
| Classification availability agrees with UA support | candidate `image/x-not-supported`; decoder configuration unvalidated | Native type checker; callers next |
| Requested JS/image/font category agrees with classification | Image + `image/svg+xml`: native classification checks XML before image; font witness none found for this interface | Native type checker; callers next |
| CSS full-record equality agrees with essence equality | Style + `text/css; charset=utf-8` | Native type checker; callers next |

Empty type and translated fetch destination bypass these premises, matching the
actual pre-call control flow. The premises narrow the theorem and are not a proof
that arbitrary HTTP/DOM input satisfies them. Parser/UA premises remain explicit
model/configuration assumptions until independently established.

Full focused command: **55 verified, 0 errors**; native type body is still trusted
at this requirement checkpoint. Labels added: IMPLICIT comments for each clause
and the executable requirement. No trust labels removed. No executable body edits.
Self-check 1–11: yes for this requirement-only task, using the existing seven-step
spec; no new numbered algorithm. Findings: implementation/model distinctions in
the table, not new standard ambiguity. Not done: enable/discharge native body,
inherit/discharge these premises at callers, and remaining algorithm migrations.

## Task: native type checker verified conditionally (2026-10-30)

Removed `external_body` and its scope label from native
`LinkProcessingOptions::type_matches_destination`. Its standard-result `ensures`
is unchanged and points to the seven-step state machine (HTML
<https://html.spec.whatwg.org/multipage/links.html#match-preload-type>, snapshot
2026-10-30, row 1 for the MIME temporary). Steps 1–7 retain the previously
reported machine mapping and pass the eleven-point self-check.

Full focused command: **56 verified, 0 errors**. The preceding trial was
55 verified, 1 failed native check at the empty-string early return; an erased
ghost assertion establishes extensional equality of the empty string sequence.
All executable expressions/control flow are preserved. The proved machine
result theorem is broadcast for the executable contract proof.

Labels removed: `TRUSTED(scope)` for native type matching; its external-body
annotation is removed. Labels added: none. The implicit requirements from the
previous task remain: native parser, decoder-support and category agreement,
preload destination domain, and CSS essence/full-record agreement. This is a
conditional implementation theorem, not proof that all browser inputs meet those
premises. Existing MIME external-library contracts remain trusted dependencies.
No new findings/Verus limitations. Not done: caller requirement propagation,
native preload/effects, other algorithms and remaining origin/URL operations.

## Task: inherit type-checker correspondence at immediate callers (2026-10-30)

Native `preload` now explicitly inherits `type_matches_standard_requires` before
its runtime assertion/call. `process_link_header_requires` inherits both safety
and standard-correspondence requirements only when rel is `preload`. The guard
and prior option state, not later request creation, determine these obligations.
The witnesses and assumptions are unchanged from the clause table above.

Full focused command: **56 verified, 0 errors**. Labels added: INHERITED comments;
none removed. No runtime body edits or standard-result weakening. These callers
remain labelled trusted, so adding their contracts does not prove call-site
discharge. Self-check 1, 9–11 yes; no new numbered algorithm, other items refer
to the unchanged standard specs and their reported legacy gaps. Findings and
Verus limitations: none new. Not done: header-loop and DOM-input propagation,
caller/effect proofs and remaining algorithm migrations.

## Task: same-origin-domain state machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/browsers.html#same-origin-domain>,
snapshot/inspection 2026-10-30. Row 1: returns inside the nested Step 2 block
exit the enclosing algorithm. Preserve that parent step and both substeps rather
than merge their conditions or duplicate the final return.

| Step | `same_origin_domain_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes, opaque identity |
| 2 | step2 / Step2 | Yes, enter tuple-origin block or proceed to 3 |
| 2.1 | step2_1 / Step2_1 | Yes, same scheme and equal non-null domains |
| 2.2 | step2_2 / Step2_2 | Yes, separately call same-origin with null domains |
| 3 | step3 / Step3 | Yes, false return |

All comments are verbatim. Public closed condition projections preserve the
crate-private origin view without introducing trust. Generated transition
relations define the runner. Proofs supply successor witnesses and establish
termination, input preservation and the result; the HTML table's different-port/
equal-domain example verifies the distinction from same-origin equality.

Full focused command: **59 verified, 0 errors**. Labels added/removed: none.
No executable edits, extra preconditions, or new findings. Self-check 1–11:
yes for this algorithm. Native MutableOrigin/OriginSnapshot domain observations
remain outside this proof. The previously recorded pure-spec-return limitation
is avoided by an explicit state machine, not a runtime rewrite. Remaining
serialization/site operations and legacy link algorithms are not completed here.

## Task: URL equality through serialization (2026-10-30)

Algorithm: <https://url.spec.whatwg.org/#concept-url-equals>, snapshot 2026-10-19,
inspected 2026-10-30. Row 1: serializedA/serializedB are carried to the result.
`algorithms/url.rs::url_equality_machine` maps 1→step1 (serialize A),
2→step2 (serialize B), 3→step3 (compare), with complete verbatim step comments.
The optional exclude-fragments argument defaults to false in `url_equal`.

The linked URL serializer is an explicitly labelled primitive over only the
serialized components; blob-entry identity is excluded from its inputs, and
excluded fragments are omitted. This specifies the equality algorithm relative
to that primitive without claiming a proof of URL/host/path serialization.
The generated relations yield a proved terminating three-transition run.
Two further proofs establish blob-entry independence and excluded-fragment
independence. Native `ServoUrl::as_str` and `is_equal_excluding_fragments` receive
labelled correspondence contracts through closed projections.

Full focused command: **62 verified, 0 errors**. Labels added: scope for the
linked URL serializer and two native method contracts; none removed. No new
implicit preconditions or executable changes. Self-check 1–11: yes for this
three-step algorithm. Findings/limitations: none new. Not done: serializer
implementation correspondence, broader URL parsing/origin operations and
remaining link algorithm migrations/native effects.

## Task: create-link-request state machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/semantics.html#create-a-link-request>,
snapshot/inspection 2026-10-30. Row 1: URL/request temporaries and field updates.
`algorithms/request.rs` now defines the standard through generated transition
relations, not functional shadowing. Closed field setters each update one field
and keep crate-private views compatible with generated public relations.

| Step | `create_link_request_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes: href assertion is a guard |
| 2 | step2 / Step2 | Yes: separately linked URL parser |
| 3 | step3 / Step3 | Yes: null return on parsing failure |
| 4 | step4 / Step4 | Yes: separately linked potential-CORS request |
| 5 | step5 / Step5 | Yes: policy container update |
| 6 | step6 / Step6 | Yes: integrity metadata update |
| 7 | step7 / Step7 | Yes: cryptographic nonce metadata update |
| 8 | step8 / Step8 | Yes: referrer policy update |
| 9 | step9 / Step9 | Yes: client update |
| 10 | step10 / Step10 | Yes: priority update |
| 11 | step11 / Step11 | Yes: return request |

Each transition has the complete verbatim sentence immediately above it. The
runner selects generated successors and stops on the early return. `request_result`
proves termination and derives the result/field-preservation law under the href
assertion. The assertion moved out of the result conjunction into the machine
guard and the explicit specification domain predicate; the executable already
requires the same nonempty href and retains its runtime assert. The standard
result is unchanged on its defined domain. The regression now checks the
empty-input domain obligation rather than treating an assertion as a false result.

Full focused command: **63 verified, 0 errors**. Labels added/removed: none.
Self-check 1–11: yes for this algorithm. No executable changes/new input
restrictions. Existing URL/CORS native correspondence remains a separate
boundary. Resolved integration diagnostic: Verus permits one module-level
`broadcast use` declaration; combine the proved laws in one declaration.
Findings: none new. Not done: native request-body conformance, potential-CORS
machine migration, preload effects and the remaining link algorithms.

## Task: potential-CORS request state machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/urls-and-fetching.html#create-a-potential-cors-request>,
snapshot/inspection 2026-10-30. Row 1: mode and credentialsMode are updated
across steps. `potential_cors.rs` preserves strings and the optional fallback
flag; the Fetch request constructor remains a separately named operation over
the supplied default record. `cors.rs` retains the shared keyword/state adapters
and re-exports the constructor for existing callers.

| Step | `potential_cors_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes: initialize mode from No CORS |
| 2 | step2 / Step2 | Yes: optional same-origin fallback update |
| 3 | step3 / Step3 | Yes: initialize credentialsMode to include |
| 4 | step4 / Step4 | Yes: Anonymous updates credentialsMode |
| 5 | step5 / Step5 | Yes: complete verbatim request-construction sentence |

The runner selects generated successors; its proof establishes termination and
the resulting fields. Existing mode/fallback/credentials and request/preload
regressions use the derived computation theorem. Full focused command:
**64 verified, 0 errors**. Labels added/removed: none. No executable edits,
new preconditions, findings or Verus limitations. Self-check 1–11: yes for this
algorithm. Not done: native Fetch builder/default correspondence, remaining
attribute/header/preload/callback migrations and native effect proofs.

## Task: enumerated-attribute direct-step layout (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute>,
snapshot/inspection 2026-10-30. Row 3: each step returns or falls through;
the absent-attribute block always returns. Removed implementation-only local
value/keyword bindings so step 2's comment immediately precedes its conditional.
Matching-keyword selection remains a separate specification term, with no
normalization beyond the referenced ASCII case-insensitive comparison.

Step map in `determine_the_state_of_an_attribute`: 1→absent branch,
1.1→missing-default branch, 1.2→no-state alternative, 2→keyword branch,
3→empty-default branch, 4→invalid-default branch, 5→final no-state.
Every step retains its complete text and order. Full focused command:
**64 verified, 0 errors**. Existing generic/CORS/fetch-priority proofs pass.
Self-check 1–11: yes for this algorithm. No labels added/removed, runtime changes,
new preconditions, findings or limitations. Other migration/native-effect work
remains as recorded above.

## Task: attribute-application state machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes>,
snapshot/inspection 2026-10-30. Row 1: options updates, a destination temporary,
and nested early returns. `apply_attributes.rs` is the canonical specification;
the old functional expression is an explicitly named computation helper whose
equivalence is proved, with its duplicate numbered comments removed.

| Step | `apply_attributes_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes: enter preload block |
| 1.1 | step1_1 / Step1_1 | Yes: missing as returns false |
| 1.2 | step1_2 / Step1_2 | Yes: linked destination translation |
| 1.3 | step1_3 / Step1_3 | Yes: null destination returns false |
| 1.4 | step1_4 / Step1_4 | Yes: destination update |
| 2 | step2 / Step2 | Yes: keyword-only CORS update |
| 3 | step3 / Step3 | Yes: integrity update |
| 4 | step4 / Step4 | Yes: valid referrer-policy update |
| 5 | step5 / Step5 | Yes: nonce update |
| 6 | step6 / Step6 | Yes: type update |
| 7 | step7 / Step7 | Yes: keyword-only fetch-priority update |
| 8 | step8 / Step8 | Yes: true return |

Full verbatim step text precedes each transition. Each option setter changes one
field and preserves the rest. Destination substep completion and whole-machine
termination/result equivalence are proved from generated relations. The existing
keyword/default/order/frame/trace regressions consume the new machine through
its derived theorem. Full focused command: **66 verified, 0 errors**.

Self-check 1–11: yes for this algorithm's steps; linked destination translation
still has its separately reported inlining cleanup pending. Labels added/removed:
none. No executable changes/new preconditions. FINDING retained at step 7:
standard says keyword but the record field is a state; use its explicit table
mapping without changing the quoted sentence. No new Verus limitation. Native
attribute updates remain trusted and may differ on invalid attributes/fetch
priority. Native effects and remaining preload/header/callback migrations remain.

## Task: thirteen-step preload machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#preload>, snapshot/
inspection 2026-10-30. Row 1: mutable options/request/controller and temporaries.
`link_types/preload_machine.rs` is canonical; the old expression is a derived
computation helper. Nested callback bodies remain separate specs to migrate next.

| Step | `preload_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes: linked type match and early return |
| 2 | step2 / Step2 | Yes: image/source-set guard and href update |
| 3 | step3 / Step3 | Yes: linked request construction; inherited assertion guard |
| 4 | step4 / Step4 | Yes: request-failure return |
| 5 | step5 / Step5 | Yes: unsafeEndTime initialization |
| 6 | step6 / Step6 | Yes: new preload entry |
| 7 | step7 / Step7 | Yes: linked preload-key constructor |
| 8 | step8 / Step8 | Yes: early-hint initiator update |
| 9 | step9 / Step9 | Yes: controller initialized to null |
| 10 | step10 / Step10 | Yes: reportTiming definition marker |
| 11 | step11 / Step11 | Yes: controller assigned from Fetch; separate response callback |
| 12 | step12 / Step12 | Yes: commit algorithm identity with shared-state seed |
| 13 | step13 / Step13 | Yes: set on-document-ready or record commit invocation |

All thirteen complete sentences are immediately above their transitions. The
result/termination theorem follows generated successors and both early-return
paths. The request's nonempty-href assertion is an explicit inherited guard
**after source selection**, not a conjunct of the preload result. The plan now
records the returned Fetch controller and commit-algorithm identity, and the
early-hint path actually updates options.on_document_ready. These repair omissions
in the old reference plan, not executable behavior. Existing tests additionally
check that the installed callback is the defined commit and the controller is
the Fetch result. Callback invocation still takes then-current shared state;
the identity seed does not assert by-value capture of mutable state.

Full focused command: **67 verified, 0 errors**. Added scope labels for Fetch
controller and commit-identity correspondence primitives in StandardModel.
No runtime edits. Self-check 1–11: yes for the outer thirteen steps, with nested
callback and linked-key/selection tasks explicitly remaining. FINDING: source
selection returns a source and density whereas this step assigns its result to
string href; the linked primitive exposes the href component, with URL/string
correspondence still explicit. Native preload remains trusted: a valid operation
plan does not establish that native fetching, callback installation or commit ran.
Not done: native effects, nested callback migrations, headers and remaining
linked algorithms/requirement propagation. No new Verus limitation.
