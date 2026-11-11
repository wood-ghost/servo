# WHATWG correspondence task log

Historical task commit IDs below predate the user-requested squash into
`073ff48555e`. For current commands and claims, start with [STATUS.md](STATUS.md).

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

From the Servo root, prepare the pinned dependency snapshot once, then run:

```sh
bash verus_cmd.sh prepare /path/to/verus-git-clone
bash verus_cmd.sh verify
bash verus_cmd.sh native
```

The root Cargo patches select `.verus-source` for every build. The native mode
deliberately runs without `--config`, as do ordinary Cargo/mach builds after
preparation. Current dependency provenance and prerequisites are in STATUS.md.

### Historical command for the October 30 task entries

The following is the exact command referred to by the October 30 entries, run
from `/home/yizhiy/Desktop/servo` with that revision's local root Cargo patches:

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus -p servo-net-traits -p servo-script --locked -- \
  --compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20
```

Both lifetime and trait-conflict checking are disabled. Counts refer to the
reported script verification, not every dependency or browser behavior. The
installed binary reports `0.2026.10.29.62a9ca4.dirty`, release, Rust 1.99.1.
For the October 30 runs, the Verus source HEAD was `dea854413`; that is not the
installed binary's reported revision. No verifier changes were made for these
tasks. The November 10 revalidation below records the dependency snapshot used
after the shared source checkout moved independently to a different branch.

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
| Requested JS/image/font category agrees with classification | none found; the initially suggested SVG witness was rejected during the final source audit because native is_xml explicitly excludes image types | Native type checker; callers next |
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

## Task: preload response-consumption callback (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#preload>, callback
11.1–11.6, snapshot/inspection 2026-10-30. Row 1: response/entry/time mutation
and ordered callback effects. `preload_response.rs` maps 11.1→step11_1,
11.2→step11_2, 11.3→step11_3, 11.4→step11_4, 11.5→step11_5,
11.6→step11_6, each with its complete verbatim sentence. Step 11.1 skips 11.2
on its successful branch; the alternative is its own transition. No merged steps.

State is supplied at callback invocation. The machine records timing before
on-response-available before processResponse, updates the entry only on the
null-callback branch, and preserves the other invocation fields. The generated
successor theorem establishes termination/result equivalence; existing callback
ordering/failure/storage regression proofs consume the machine.

Full focused command: **68 verified, 0 errors**. Added scope label for the
byte-sequence-as-body projection: this view observes bytes, while Fetch stream/
source/length correspondence remains unverified. No other labels, executable
edits, new preconditions, findings or limitations. Self-check 1–11: yes for
these six substeps. Native callback execution and shared-state observation remain
outside this operation-plan proof. Remaining work includes commit callback,
header algorithms, linked cleanup and native/boundary accounting.

## Task: preload commit callback and map update (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#preload>, callback
12.1–12.2, snapshot/inspection 2026-10-30. Row 1: document-map mutation and
ordered effects. `callbacks.rs::commit_preload_machine` maps 12.1→step12_1
(conditional timing), 12.2→step12_2 (map insertion), verbatim and in order.
The map update is explicit, alongside its observation-log entry.

Map keys encode itemwise preload-key equality using URL serialization, destination,
mode and credentials mode, so blob-entry identity does not accidentally affect URL
equality. The theorem proves the final map is exactly the arbitrary input map
with the one key updated, and proves the effect list independently of that input
map. The legacy effect-only adapter uses that independence; it does not assume
that a real document starts with an empty map.

Full focused command: **69 verified, 0 errors**. Labels added/removed: none;
the existing URL-serializer and native effect-observation boundaries remain.
Self-check 1–11: yes for both substeps. No runtime edits, new preconditions,
findings or limitations. Not done: native document-map/async callback
correspondence, header algorithms and remaining linked/boundary tasks.

## Task: literal header-extraction loop machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/semantics.html#extract-links-from-headers>,
snapshot/inspection 2026-10-30. Row 1: accumulation, loop cursor and continue.
`extract_headers.rs` preserves the raw input sequence and advances an explicit
index; no input shortening or merged body steps.

| Step | `extract_headers_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes: empty links list |
| 2 | step2 / Step2 | Yes: separately linked header get/decode/split |
| 3 | step3 / Step3 | Yes: loop head, loop variable and exit |
| 3.1 | step3_1 / Step3_1 | Yes: separately linked singular parser |
| 3.2 | step3_2 / Step3_2 | Yes: missing target continues at loop head |
| 3.3 | step3_3 / Step3_3 | Yes: append and finish iteration |
| 4 | step4 / Step4 | Yes: return links |

The recursive computation is now justified by a **constructed complete machine
trace**, with termination measure remaining raw headers and loop-head invariant
index≤length. Trace-splicing and recursive proofs establish every generated next
edge, including missing-target paths and order-preserving accumulation. They
are reference traces, not observations of native execution.

FINDING resolution policy: take HTML's singular linkObject literally. Added
`parse_standard_link_header` for that interface. The pre-existing RFC/native
multi-object parser and its order/relation-expansion computation remain separate
in `headers.rs`; the multiple-relations regression explicitly checks that native
adapter. No native parser or executable loop was changed, and no assertion of
equivalence between the singular and plural interfaces is made. That ambiguity
must be resolved before claiming complete native extraction conformance.

Full focused command: **73 verified, 0 errors**. Added scope label to StandardModel's
singular parser correspondence primitive; none removed. Self-check 1–11: yes for
the literal algorithm. No runtime edits or extra external-input requirements.
No new Verus limitation. Remaining work: processing-loop machine, remaining
linked algorithms, trust/boundary audit and native effect correspondence.

## Task: process-link-headers loop machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers>,
snapshot/inspection 2026-10-30. Row 1: loop, temporaries, option updates, continue
and early return. `process_headers.rs` is the numbered specification. The existing
coarse observation validator is now named `processing_trace_computation`; the
general `processing_trace` theorem constructs a complete fine-grained run for
every accepted supplied trace.

| Step | `process_headers_machine` item | Self-check |
| --- | --- | --- |
| 1 | step1 / Step1 | Yes: separate supplied extraction operation |
| 2 | step2 / Step2 | Yes: loop head, variable binding and completion |
| 2.1 | step2_1 / Step2_1 | Yes: relation temporary |
| 2.2 | step2_2 / Step2_2 | Yes: attributes temporary |
| 2.3 | step2_3 / Step2_3 | Yes: expected phase |
| 2.4 | step2_4 / Step2_4 | Yes: phase skip to loop head |
| 2.5 | step2_5 / Step2_5 | Yes: media call/skip to loop head |
| 2.6 | step2_6 / Step2_6 | Yes: full field list and option defaults |
| 2.7 | step2_7 / Step2_7 | Yes: linked attribute update and early return |
| 2.8 | step2_8 / Step2_8 | Yes: both image attributes and null image argument |
| 2.9 | step2_9 / Step2_9 | Yes: linked dispatch invocation and next iteration |

All complete step text is directly above its transition. Inputs/links are never
shortened. The proof's loop invariant ties the consumed prefix to index and the
current document snapshot, including earlier iterations' effects. Both skip
paths preserve the document snapshot; only dispatch consumes its observed next
state. Early application failure ends at the final visited link. The pre-extracted
adapter supplies the earlier extraction result; the response adapter calls the
separate extraction specification. This is not native execution instrumentation.

Full focused command: **76 verified, 0 errors**. Added scope labels for observed
media results, source-set results and dispatch document effects. These are the
same supplied-call boundaries as before; guards check call arguments/trace format,
not the correctness of unverified media/CSS/dispatch implementations. No native
code or external-input requirements changed. Self-check 1–11: yes for the parent
algorithm relative to its labelled linked boundaries.

Resolved macro limitation: an initializer parameter written directly as
`f: spec_fn() -> T` produced `expected ','` / `cannot find type spec_fn` with the
pinned state-machine macro. A named type alias declared in `verus!` preserves
the mathematical function type and parses correctly; no trust/verifier change.
Findings: none new. Remaining work: remaining linked/secondary caller specs,
trust-label and requirement/boundary audit, feature probes/native build and report.

## Task: linked Fetch destination translation (2026-10-30)

Algorithm: <https://fetch.spec.whatwg.org/#concept-potential-destination-translate>,
snapshot 2026-10-19, inspected 2026-10-30. Row 3: sequential returns without
carried variables. `destination.rs` covers the general Fetch domain, not just
the six HTML preload names. Step 1→fetch branch; step 2→conditional domain
obligation; step 3→fall-through return. The domain theorem proves the translated
value belongs to the Fetch destination set for every potential destination.

Full focused command: **77 verified, 0 errors**. Labels added/removed: none.
Self-check 1–5, 7–11: yes. Check 6 textual-order qualification: as permitted by
the assertion-to-contract rule, step 2's verbatim Assert is attached to the entry
`recommends` clause (guarded for the early-return path), rather than appearing as
a branch or result conjunct. The executable return branches retain order 1, 3.
This obligation relocation is explicit; it is not a silently dropped step.
No runtime edits, new implementation preconditions or findings. Next task links
HTML translation to this general algorithm instead of inlining its behavior.

### Fresh feature probes, same installed verifier (2026-10-30)

Ran `verus --edition=2024 /tmp/opencode/<probe>.rs` (matching Servo's edition),
with the native functions enclosed in `verus!` and an ordinary empty main:

| Probe / minimal function body | Exact diagnostic / result |
| --- | --- |
| `link_string_pattern_probe`: `match value { "fetch" => true, _ => false }`, value: &str | `Verus does not currently support equality-testing this type in a pattern: &'{erased} str` |
| `link_let_chain_probe`: `if flag && let Some(x) = value { x } else { 0 }`, value: Option<u8> | `Verus does not currently support let expressions in conditions` |
| `link_const_block_probe`: `const { true }` | `Verus does not currently support const block expressions` |
| `link_feature_positive_control`: equivalent nested if/if-let, Option enum match, plain true | **3 verified, 0 errors** |

The three negative probes fail during frontend translation, not theorem checking.
Issues/PRs remain "not reported". Existing labelled let-chain rewrites and string/
macro-const trust boundaries therefore remain justified on this binary. No
verifier changes or unnecessary runtime rewrites were made.

## Task: HTML preload translation calls Fetch (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#translate-a-preload-destination>,
snapshot/inspection 2026-10-30. Row 3: conditional return then final return.
Step 1→invalid-preload-name branch; step 2→the separate general Fetch translation.
The canonical returned string is converted to DestinationView by a representation
lookup, rather than inlining six cases of the linked standard algorithm.

`DestinationView::from_name` covers the existing 24 native enum values; a
round-trip theorem checks the inverse mapping. A second theorem proves every
preload name meets the linked potential-destination domain and returns a view.
Existing destination/attribute/header regressions pass.

Full focused command: **79 verified, 0 errors**. Labels added/removed: none.
Self-check 1–11: yes for HTML's two steps, with Fetch's exported assertion
placement explicitly documented in the preceding task. No runtime changes,
extra implementation requirements or new findings. Native string-pattern
translation remains labelled trusted for the freshly reproduced Verus gap.
Remaining work: preconnect/secondary caller and small linked definitions,
trust/input-boundary audit, native build and final report.

## Task: preconnect state machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#preconnect>, snapshot/
inspection 2026-10-30. Row 1: URL/partitionKey temporaries and useCredentials
update. Steps 1–8 map one-to-one to `preconnect_machine::step1`–`step8`:
empty-href return, URL parse, failure return, HTTP(S) guard, partition key,
credentials initialization, anonymous cross-origin update, connection plan.
Each complete sentence is verbatim and directly above its transition.

The standard's paragraph permitting partial handshakes or skipping entirely is
retained, so `MayConnect` describes permission/recommendation rather than a
mandatory network effect. The HTTP(S) test reads the URL record and same-origin
calls the separate proved predicate. URL origin and network partition key remain
linked model operations. Prove all early returns and final result from generated
relations; existing preconnect/dispatch regression proofs pass.

Full focused command: **81 verified, 0 errors**. No new labels, executable changes,
input restrictions or findings. Self-check 1–11: yes for the eight-step algorithm.
An initial result-proof failure exposed the closed request input adapters:
`reveal` cannot open a closed function from another module. A theorem in their
owning module exposes their defined projections; no external assumption or
visibility widening was introduced. Native connection effects remain outside
the plan proof. Remaining work: secondary caller/small linked definitions and
trust/input-boundary audit/native build/report.

## Task: link-element preload caller reference machine (2026-10-30)

Algorithm: <https://html.spec.whatwg.org/multipage/links.html#link-type-preload:fetch-and-process-the-linked-resource-2>,
snapshot/inspection 2026-10-30. Row 1: options/destination temporaries and update.
`preload_element.rs` maps 1→step1 (source-set update operation), 2→step2
(create options observation after that update), 3→step3 (translate as-state
keyword), 4→step4 (null return), 5→step5 (destination update), 6→step6
(linked preload with response callback). The single response callback substep
6.1 is a direct branch specification with its complete text; the existing event
adapter now delegates to it.

The source-set update flag records the **planned operation**, not evidence of
native mutation. The options and attribute-state observations are supplied by
the unverified element/DOM boundary. Both linked source-update/create-options
boundaries have scope labels and URLs. The machine requires preload's href
obligation at Step 6 after assigning the translated destination. A general proof
establishes termination, the ordered update marker and the selected preload plan.

Full focused command: **82 verified, 0 errors**. Labels added: two scope labels
for linked element observations; none removed. Self-check 1–11: yes for the
six-step caller relative to those explicit boundaries. No executable edits or
new external-input restrictions in this reference-only task. HTML issue #1142
still records real-browser event differences; this spec follows the literal
error/load rule. Native handle_preload_url attachment and DOM-input assumptions
are the next task; source-set update and element-option construction are not
claimed independently verified.

## Task: native link-element boundary and DOM assumptions (2026-10-30)

Wrapped only `HTMLLinkElement::handle_preload_url` in `verus!` with a labelled
external body, inherited/implicit requirements and the standard reference-plan
postcondition. Original executable expressions, previous-type bookkeeping,
guards and comments are preserved. Its helper functions for destination lookup,
option creation, default fetch/request creation and response events are explicitly
scope-labelled exclusions. The existing six-step caller spec and callback 6.1
remain the standard contracts (URL/snapshot/form/step map in the prior task).

`servo_api/link_element.rs` labels both input and effect observations. These are
unfinished single-execution correspondence hooks; attaching the trusted contract
does not establish actual native mutation or network events. The native hoisted
type guard may produce the same no-fetch outcome as the standard preload's
TypeMismatch, without claiming the native function invoked preload on that path.

| DOM-boundary clause | Witness / status | Propagation stop |
| --- | --- | --- |
| Raw as translation agrees with enumerated-state keyword | candidate `as="SCRIPT"`; source inspection, browser execution not tested | DOM attribute input |
| MIME safety and parser/UA/category/CSS agreement | witnesses from the type-checker task | DOM type/as input and decoder/model configuration |
| Nonempty href after source selection | inherited request assertion | DOM href/imagesrcset plus contextual selection |
| Missing native fetch priority only observes Auto on fetching paths | `fetchpriority="high"` | DOM attribute input |
| Native source selection has a nonempty set with density descriptors | empty set / missing density cause remove/unwrap obligations | DOM source-set construction and selection boundary |

Per Section 7, these are **assumptions about external input/context**, not proofs
that all DOM input satisfies them. Stop propagating them into arbitrary web input.
The default-fetch helper's separate nonempty-href caller obligation is explicitly
recorded as unverified rather than inferred from code after its request call.

Checks: full focused command **82 verified, 0 errors**; native
`cargo +1.97.1 check -p servo-script --lib --locked --offline` with the same clang
environment **passed**. Added scope labels for five native methods and two
observation hooks, plus IMPLICIT/INHERITED clause comments. No trust removed.
Self-check: 1 and 9–11 yes for annotations; standard step checks use the prior
reference proof. Native bodies remain trusted-only. Native check adds the expected
existing-style `verus_only` cfg warnings; none suppressed. Remaining work:
HTTP-boundary requirements, local/transitive trust audit, small linked definition
comments and final report. No runtime meaning change is claimed from compilation
alone; the diff also preserves every executable expression in this task.

## Task: implicit attribute-update requirements (2026-10-30)

Added `apply_link_options_requires(old(self), link_object, rel)` without changing
the canonical state-update `ensures` or executable body. Early invalid/missing
preload destinations return before the other attributes, so they impose none
of the later-attribute restrictions.

| Clause | Witness | Discharge / propagation |
| --- | --- | --- |
| Invalid crossorigin may reset only an already-No-CORS state | old UseCredentials plus `crossorigin="bad"` | Header loop initializes No CORS; standalone calls must require it |
| Invalid referrerpolicy may reset only an already-empty policy | old Origin plus `referrerpolicy="bad"` | Header loop initializes EmptyString; standalone calls must require it |
| Recognized fetchpriority must be Auto | `fetchpriority="high"` / `"low"` | Native record has no priority field; HTTP input assumption pending |

All clauses carry IMPLICIT labels and witnesses. Full focused command:
**82 verified, 0 errors**; native attribute body remains labelled trusted at this
checkpoint. Standard algorithm URL/snapshot/form/steps remain the previously
proved application machine. Self-check 1–11 passes for the requirement-only
change; no standard-result weakening or runtime edits. Findings are the known
implementation differences above, not new standard ambiguity. Next: attempt
the native body, then inherit the applicable condition at the HTTP boundary.

## Task: native attribute updater verified conditionally (2026-10-30)

Removed the external-body annotation and scope label from
`apply_link_options_from_parsed_header`. Its complete state-update postcondition
still refers to the canonical application machine (HTML snapshot 2026-10-30,
same URL/form/step table as above). Original executable expressions are unchanged.

The missing bridge was between native first-value lookup and the finite valued-
attribute map. `valued_attributes_lookup` proves membership iff lookup returns
Some, plus exact mapped value, by induction on the ordered parameter sequence.
The executable body calls that lemma for its seven known attribute names in an
erased proof block. This does not assume RFC normalization or discard duplicate
native parameters. The machine's proved computation law supplies the postcondition.

Validation progression: initial active-body run **82 verified, 1 postcondition
failure**; adding the storage lemma yielded **83 verified, 1 resource-limit
failure**, including after replacing broad instantiation with explicit calls.
A local `#[verifier::rlimit(30)]` on this function then passed the unchanged
full focused command: **84 verified, 0 errors**. The resource adjustment changes
solver effort, not assumptions or the theorem. No global verifier flags changed.

Labels removed: the native attribute updater's scope label/external body. Labels
added: none. Existing invalid-CORS/referrer/priority restrictions remain explicit;
the theorem is conditional, not unrestricted header conformance. Header lookup,
token translation and native referrer conversion contracts remain trusted.
Self-check 1–11: yes for this native proof and its existing standard model. No
new standard finding. Next: inherit the restrictions and request/source-selection
obligations at the header input boundary, and finish the trust audit.

## Task: source-selection dependency contract (2026-10-30)

Native dependency: `SourceSet::select_image_source_from_source_set`, linked to
<https://html.spec.whatwg.org/multipage/images.html#select-an-image-source-from-a-source-set>.
Inspected native code unwraps each density and removes a chosen vector index;
empty or unnormalized sets violate those implicit domains. The trusted contract
now states that domain explicitly and relates the returned href component to
the same contextual model choice used by preload. It returns Some on this domain.

Full focused command: **84 verified, 0 errors**. The existing scope label remains;
an IMPLICIT requirement with empty-set/missing-density witnesses was added. This
is deliberately **trusted-only**: native CSS/f64/URL choice correspondence has
not been proved or independently tested, and the contextual view is assumed.
No standard selection step or executable expression was changed. Self-check
1/9–11 yes for this dependency contract; no new numbered algorithm is claimed.
Next: inherit normalization and nonempty selected href into preload/dispatch,
then quantify the construction-result obligation at the HTTP boundary.

## Task: inherit source-selection/request domains (2026-10-30)

Added `preload_selection_requires(options, document)`: image/source-set paths
inherit the selection dependency's normalized nonempty domain and require the
**selected** href to be nonempty; all other paths inherit the original nonempty
href assertion. Native preload requires this at entry. Header dispatch inherits
it only for preload relations whose earlier type-match guard succeeds. The DOM
boundary also explicitly inherits the source-view well-formedness condition.

An owning-module proof connects this native selection requirement to the
reference preload machine's domain using the same document-context projection.
It does not prove the contextual native/model correspondence, which remains in
the labelled dependency contract. No condition is discharged using code after
its call, and an image selection that replaces href is not confused with the
original href.

Full focused command: **85 verified, 0 errors**. Added INHERITED comments; no trust
removed or executable edits. Witnesses remain empty/missing-density source sets
and empty selected/original href. Self-check 1/9–11 yes for propagation, with
standard steps unchanged. Preload/dispatch effects remain trusted. Next task
terminates these obligations at the HTTP-header boundary as input/model assumptions.

## Task: HTTP-header boundary requirements and loop invariant (2026-10-30)

Native `process_link_headers` now inherits `process_link_headers_requires` and
records it as an erased `for`-loop invariant. `header_input_requires` uses raw
input parameters, phase checks and whether the media API contract permits a true
result. It does not use effects after dispatch to justify earlier calls.

| Inherited clause | Where established / boundary |
| --- | --- |
| Invalid CORS/referrer reset compatibility | New `header_defaults_discharge_attribute_resets` proves caller's initial None/EmptyString fields suffice |
| Supported priority representation | Header fetchpriority must be Auto or unrecognized on paths past application checks; high/low remain witness restrictions |
| MIME safety/standard agreement | Type/as input to a valid preload destination before type-checking/dispatch |
| Nonempty request href | Target URI when selection does not replace it |
| Normalized nonempty source selection and nonempty selected href | Every result allowed by the preceding SourceSet::create_source_set contract for those input attributes/document |

The construction-result quantifier follows the same modular pattern as the MIME
parser requirement: an actual returned value must meet the callee's domain before
the later call. These requirements stop here as assumptions about external HTTP
input and linked models. They are not asserted to hold for arbitrary headers and
do not add runtime filtering. The flat candidate condition is **conservative**:
it can require a later header's domain even when an earlier application failure
would already return. It is a sufficient domain, not a weakest precondition.

Full focused command: **86 verified, 0 errors**. Added INHERITED comments; no trust
removed. No executable expressions changed (only requires/invariant annotations).
The native loop remains external-body: its recorded invariant and all call-site
discharges are **not** a native body proof. The reference loop's prefix/state
invariant was independently proved in the earlier machine task. Self-check
1/9–11 yes for annotations/requirements; standard algorithm mapping is unchanged.
HTTP ingress callers in ServoParser remain outside verification. RFC normalization
and singular/plural parser correspondence remain separate labelled model gaps.
Remaining work: exact standard-result hook connections, trust inventory and final
native build/report.

## Task: connect native request postcondition to standard result (2026-10-30)

`create_link_request_execution_ensures` now delegates to the exact standard
result through the current options/request views, instead of being an
uninterpreted Boolean. The native function retains its labelled external body:
URL parsing, Fetch defaults, request fields and supplemental-context production
remain assumed correspondence, not newly verified code.

Full focused command: **86 verified, 0 errors**. No new labels/input restrictions
or executable changes. The prior eleven-step URL/snapshot/form/map and self-check
remain authoritative. Self-check 1/7/9–11 yes for this contract connection; the
nonempty-href assertion remains a precondition. Not done: proving this native
request body or the opaque/context-free view bridges, and remaining effect hooks.

## Task: connect native preload postcondition to standard plan (2026-10-30)

Replaced the uninterpreted preload-result Boolean with the canonical
`preload_ensures` relation over an explicitly named `observed_preload_plan`.
That observation producer has its own scope label. The native body is still
external-body, so its plan correspondence is **assumed**, not a consequence of
the proved reference runner. Actual per-invocation trace/event capture remains
unimplemented; the declaration must not be treated as an event collector.

Full focused command: **86 verified, 0 errors**. Added the observation scope label;
no runtime edits or new requirements. The prior thirteen-step/child-callback
URL/snapshot/form/step audit remains unchanged. Self-check 1/7/9–11 yes for the
contract connection. Native source/request/effect correspondence and the final
header-loop effect hook remain outstanding trusted boundaries.

## Task: connect native header postcondition to standard trace (2026-10-30)

`process_link_headers_execution_ensures` now checks the standard trace relation
against an explicitly labelled `observed_process_link_headers_trace`, replacing
the final uninterpreted Boolean effect hook. The observation producer and native
loop are still trusted; the reference theorem only proves that accepted supplied
observations expand to a complete numbered-machine run.

Full focused command: **86 verified, 0 errors**. Added the trace-observation scope
label; no runtime edits or new preconditions. The prior header URL/snapshot/form/
step audit is unchanged; contract self-check 1/7/9–11 yes. Native per-invocation
capture, RFC representation and document/effect correspondence remain unproved.
All three original result hooks now name standard results/relations explicitly;
their unverified native bridges remain visible rather than becoming silent axioms.

## Task: complete local/transitive trust-label audit (2026-10-30)

Added item-adjacent labels to the remaining StandardModel/DOM observation
boundaries and the transitive MIME API's type/method/constant/equality/formatting
contracts. The existing native flag equality assumptions and two legacy model
stubs are scope-labelled. No Rust/specification semantics changed in this task.

`check_trust_labels.py` checks explicit external/assume/axiom declarations and
uninterpreted model boundaries (excluding arbitrary test-fixture constants), and
checks that rewrite labels are followed by original-code comments. It is a
textual lint, not a Rust parser or a proof. `--inventory` emits every label with
path/line grouped by kind. Scope: primary file, selected link-element caller,
local specification tree, native MIME classifier and MIME specification tree.

Result: **80 Rust files, 278 labelled boundary items, 3 rewrites, 0 missing labels**.
Labels grouped by kind (labels are not proof-coverage counts):

| Kind | Count | Main contents |
| --- | ---: | --- |
| external-lib | 119 | mime 0.3.17 API/constants; content-security-policy 0.9.0; nom-rfc8288 0.4.0; url 2.5.8; mozjs 0.26.9; http 1.5.0; bytes 1.12.1; Rust core/alloc 1.99.1 |
| scope | 164 | Excluded native bodies; DOM/URL/CSS/Fetch/input/effect observations; supplemental view correspondence; native flag equality and legacy model stubs |
| verus-gap | 2 | Native string-match patterns and macro const-block expansion |
| rewrite | 3 | Existing let chains, each with its original source immediately below the label |

The MIME constant module's historical ordinary-Rust observation claim is retained
and qualified: this audit did not revalidate a retained independent test for all
constants. Labels conservatively say "not yet". In particular, the existing
general MIME FromStr correspondence and text-only Name equality model remain
unvalidated external contracts; `mime_parse_requires` currently returns true.
Successful model/native conditional proofs do **not** independently validate
those library assumptions. The older transitive MIME algorithms were not all
migrated to this task's new layout protocol; their library/model boundary remains
explicit. The trusted base also includes the installed Verus/compiler/SMT and
vstd specifications, plus generated state-machine machinery. Dependency source
trees were not edited (the user did not authorize modifying Verus).

**Correction of an earlier witness:** native `MimeClassifier::is_xml` explicitly
starts with `!Self::is_image(mt)` (`mime_classifier.rs`, inspected lines 577–584).
Thus `image/svg+xml` is **not** a witness for XML-before-image misclassification.
The requirement comment and earlier report row now say "none found" for that
category-agreement premise. The predicate/theorem itself did not change. This
corrects the earlier commit narrative; no history was amended or erased.

Fresh full focused command after the audit: **servo-net-traits 206 verified,
0 errors; servo-script 86 verified, 0 errors**. Native locked/offline Rust 1.97.1
script check with the documented clang environment: **passed**. Both Verus
exclusions remain in force. Self-check 1/9–11 yes for this annotation audit;
standard algorithms/results unchanged. Remaining work: small definition-comment
cleanup, final review/report, and any separately recorded native-effect/model
correspondence work that is explicitly trusted rather than completed.

## Task: correct native buffer-capacity adapter (2026-10-30)

Final source review found an error in the implementation-specific observation
predicate for `LinkFetchContext::process_content_length`: native code reserves
`size.saturating_sub(current_length)`, not `size` additional bytes. The adapter
incorrectly required capacity≥current_length+size. It now requires preservation
of previous capacity and enough total capacity for current bytes and size.

Witness: one buffered byte, previous capacity 4, reported total size 8, resulting
capacity 8 is valid; the old model wrongly demanded 9. The existing adapter
regression now checks this discriminator and the no-growth size-zero case.
This corrects a **native bookkeeping model error**, not a Servo implementation
defect or a weakened WHATWG step. Capacity reservation has no separately named
WHATWG algorithm; no step numbering is invented and no executable postcondition
was changed (the native listener body is still scope-excluded).

Full focused command: **86 verified, 0 errors**. Labels/requirements unchanged.
Self-check 1/9–11 yes for this adapter correction; normative step checks are not
applicable. No native code or allocation-success guarantee was added. Independent
BytesMut implementation verification remains outside this task.

## Task: final view vocabulary and definition layout (2026-10-30)

Aligned the linked record vocabulary: RequestView now uses `integrity_metadata`
and `cryptographic_nonce_metadata`, and PreloadEntryView uses
`integrity_metadata`. LinkProcessingOptionsView retains the standard's distinct
`integrity` name and the user-requested `type_`. All dependent setters,
computation laws and fixtures were updated without semantic changes.

The complete HTML option-item text is directly above each field, including its
default and value type. Request metadata/policy-container definition sentences,
preload entry/key field text and definition URLs are present. Request policy
container None explicitly denotes Fetch's **"client" sentinel**, not null.
The preload-key constructor carries its full unnumbered standard sentence and
row-3 justification; no step number is invented. Sources: HTML snapshot 2026-10-30,
Fetch snapshot 2026-10-19, inspected 2026-10-30, at the anchors in model.rs and
link_types/preload.rs.

FINDING: the preload-key definition calls its destination a **preload destination**,
but its constructor copies request.destination, which is already translated
(including the empty string for fetch). The model preserves the literal
constructor action and explicitly records the inconsistent declared domain.
It does not silently change the quoted field definition or normalize away the
conflict to match Servo.

Full focused command: **86 verified, 0 errors**. Trust-label lint still reports
**0 missing labels** (80 files / 278 boundary items / 3 rewrites). No runtime
code, trust labels or preconditions changed. Self-check 1/3/6/8–11 yes for the
record/constructor layout, with the key-domain finding stated above; existing
numbered algorithm mappings are unchanged. Remaining work is the consolidated
status/review report; the explicitly trusted implementation/model boundaries
remain unproved rather than being labelled complete.

## Task: connect worker readiness and performance-origin owners (2026-10-30)

Final environment review connected two components already represented elsewhere
in Servo instead of leaving their production solely as supplemental inputs:
`WorkerGlobalScope::is_execution_ready` and `Performance::time_origin`. Their
external types/getter contracts are scope-labelled, and defined adapters fill
the matching supplement components from those observations.

Worker readiness is an **atomic-read result relation**, with the observed bool
passed explicitly to the adapter. It does not assume successive reads of a
mutable worker always return one immutable value. Time origin is related to the
returned CrossProcessInstant through a labelled clock-coordinate observation;
the native epoch is platform-specific, so no Unix-epoch, rounding, granularity
or clock-arithmetic property is invented. Owner/global association remains an
explicit caller/model obligation.

Full focused command: **86 verified, 0 errors**. Added 8 scope labels; no runtime
edits or new executable input restrictions. Audit now covers **286 labelled
boundary items in 80 files**, with **0 missing labels**; label totals are
external-lib 119, scope 172, verus-gap 2, rewrite 3. Self-check 1/9–11 yes for
this view-binding task; HTML environment/settings definitions and their field
map are unchanged. These getter/clock/ownership relations remain trusted-only.

## Consolidated review checkpoint (2026-11-10)

[STATUS.md](STATUS.md) is the current concise report: checked versus trusted
native coverage, algorithm URLs/forms, record coverage, boundary assumptions,
findings, exact commands, toolchain and work not completed. README now points to
the canonical machine files and labels the obsolete five-module check as a
historical subset that excludes the new machine proofs.
The checker now carries the repository's MPL header; its audit logic is unchanged.
HTML/Fetch/URL snapshot dates remain their original inspection dates.

Final source review against base `56404a45064` found annotations, erased proof
code, labelled exclusions and the retained let-chain expansion in the target;
the link-element caller's executable expressions are unchanged. All specification
`mod.rs` files remain thin, and no stale EnumeratedAttributeView or identity-only
URL/source-set/origin/policy constructor remains in the current specification
sources. The worker/clock completion did not modify native implementations.

Fresh code-checkpoint evidence (`262eb99d09b`): script **86 verified, 0 errors**;
net-traits **206 verified, 0 errors**; locked/offline native script check **passed**;
trust-label lint **0 missing labels**, 80 files, 286 boundary items. Both packages
were rechecked after rebuilding the snapshot-path dependencies, not merely read
from cached diagnostics. Lifetime and trait-conflict checking remain disabled;
native attribute application retains its documented local rlimit(30).

### Reproduction and environment correction

The original full focused command and the locked/offline native check both first
stopped before compilation with `cannot update the lock file ... because --locked
was passed to prevent this`. Servo's manifests and lockfile had no working changes.
During the session the shared Verus checkout moved independently from `servo-stable`
at `dea854413` to `char` at `bf0d49cdf`, whose library manifests have November
versions. The installed release binary still reported `0.2026.10.29.62a9ca4.dirty`.

An unmodified Git archive of `dea85441338f13fbab017b7c0ebafc9ad11464ce` was unpacked
under `/tmp/opencode/verus-dea854413`. The first archive extraction using an absolute
`--one-top-level` failed with `Invalid cross-device link`; extraction with a relative
archive prefix from `/tmp/opencode` succeeded. A temporary Cargo configuration
overrode only the three existing path patches (vstd, builtin, builtin-macros).
Its exact contents and archive command are in STATUS.md. Locked/offline Cargo
metadata confirmed the original dependency versions before the successful checks:

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus -p servo-net-traits -p servo-script --locked \
  --config /tmp/opencode/link-verification-cargo.toml -- \
  --compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20

BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.97.1 check -p servo-script --lib --locked --offline \
  --config /tmp/opencode/link-verification-cargo.toml

python3 components/script/dom/html/document_metadata/processingoptions_specs/check_trust_labels.py --inventory
git diff --check
git diff --exit-code -- Cargo.lock Cargo.toml
```

The focused run completed in 2m 47s, and the native check in 1m 02s. Existing
derive/import/deprecation/cfg/vstd warnings remain visible. No extra verification
exclusion, solver limit, trust label, input restriction or executable change was
introduced to resolve the environment mismatch. Library validation and the October
30 minimal feature probes were not rerun by this documentation task.

### Self-check and publication scope

Self-check for this documentation/license task: **1 yes** (no executable changes);
**2–8 unchanged**, with the algorithm-specific maps and qualifications in the task
log; **9 yes**, labelled boundaries inventoried with zero omissions in the checker's
textual scope; **10 yes**, explicit input/model assumptions retained in the report;
**11 yes**, no specification result was changed to obtain a pass. This does not
extend the algorithm-layout audit to all older transitive MIME models.

The initial dependency-complete checkpoint included accumulated pre-existing
link-specification work. Subsequent finished tasks were committed independently,
then pulled with `--ff-only` and pushed to the same `mime_classifier` branch.
These tasks performed no force push, rebase, branch switch, PR creation, or Verus
source edit. The final recheck also preserved Servo's manifests and lockfile.
Generated `.history/` and `.verus-log/` remain untracked. Full-chain native
conformance, actual trace capture and independent validation of the documented
library/context assumptions remain **explicitly unproved**, not hidden failures
or claims established by a green reduced-checking run.

## Review follow-up: portable dependency setup (2026-11-10)

Review items 4/5 and the clean-checkout/version checks. A detached clean worktree
of `b0de98420a7` reproduced the original command's Cargo `--locked` failure before
compilation while the shared Verus clone was on `char`. Source inspection of
`dea85441338f13fbab017b7c0ebafc9ad11464ce:source/vstd/Cargo.toml` confirms
vstd/builtin-macros/state-machine-macros `2026-10-25-0158` and builtin
`2026-10-11-0230`. The squash did bump the October 18 lock entries; the November
10 final recheck made no *further* lockfile change. STATUS now states both facts.

Added `.cargo/verus.toml` with checkout-relative patches and replaced the two
stale commands in `verus_cmd.sh` with prepare/verify/native modes. Preparation
archives the exact Git object into ignored `target/verus-source`; it does not
check out a branch or build the installed verifier. Removed absolute path patches
from Cargo.toml. The snapshot is excluded from Servo's workspace so its crates
inherit Verus's workspace dependencies (without this exclusion Cargo attempted
to inherit `convert_case` from Servo and failed before compilation).

Checks: `bash -n verus_cmd.sh`; `bash verus_cmd.sh prepare
/home/yizhiy/Desktop/verus`; `bash verus_cmd.sh verify` (**206 net-traits / 86 script
verified, 0 errors**); `bash verus_cmd.sh native` (**passed**). The full verifier
command is the command in the log's opening section plus
`--config .cargo/verus.toml` before `--`, with the same clang environment and
both disabled checks. No lockfile, executable algorithm, trust label or theorem
changed. Self-check 1/9–11 yes; standard-layout checks unchanged. Independent
clean-worktree validation of the completed review changes follows below.

## Review follow-up: invocation snapshots and native frame (2026-11-10)

Review items 3/7 and annotation nits. Removed the deterministic native
`observations(element)`, `observed_plan(element)`, `link_headers_document_state`,
dispatch/preload/header-trace producers and callback-id projection. Full standard
result relations now take invocation snapshots and observed results explicitly.
Source-set context now includes the whole `LinkHeaderDocumentState` (including
observable layout/media state), not merely its DocumentView identity. The media
and source-construction relations are snapshot-parameterized too. Result relations
alone did not imply functionality, but did not identify a current invocation either.

Preload, dispatch, the header loop and the element caller now remain `external`:
their former callable postconditions were invalid without ghost capture. This is
an explicit withdrawal of trust, not proof of those bodies or a weakened numbered
standard algorithm. The full standard obligations remain available. Native source
selection, source creation and media evaluation have signature-only contracts until
an instrumented bridge can bind snapshots. API-base-URL context is supplied from
the returned value rather than a timeless GlobalScope projection.

The checked attribute-update contract now also establishes
`after.referrer == before.referrer`. No field was added to the HTML record.
Removed its commented-out ensures, the unused native loop invariant and stale
unsupported-assignment comment. SpecRequires now names the actual requires module.
No executable expression/signature changed. A new regression uses one element
identity with two captured `as` values and proves different reference outcomes.

`bash verus_cmd.sh verify` (the full command with `.cargo/verus.toml`): **87 script
verified, 0 errors**, net-traits reused the preceding **206/0** metadata.
`bash verus_cmd.sh native` **passed**. Trust lint:
**278 boundary items, 80 files, 0 missing labels**, 119 external-lib / 164 scope /
2 verus-gap / 3 rewrites. Eight scope labels/producers removed; four exclusions
changed from external-body to external. Self-check 1/9–11 yes for the correction;
2–8 unchanged for all numbered algorithms. Native/caller verification remains
conditional or excluded as specified in STATUS.md.

## Review follow-up: Params contract migration audit (2026-11-10)

Review item 6: the October 25 package contains the October 28 iterator-interface
change, Verus `ef1443266` / PR #3330. Audited `Mime::params` against that `next`
contract and mime 0.3.17's three iterator representations. The formulas are retained;
the adjacent comment now explains that all-true `produces` is only one part of the
exhaustion guarantee. A generic client proves exact count/repeated None and uses
a non-prophetic loop bound; the expected-extra-item assertion fails as intended.
The Rust oracle independently checks all three representations. Its limited
validation replaces this one contract's "not yet" label; general view/native
termination correspondence remains trusted.

`cargo +1.97.1 test -p servo-net-traits --test mime_api --locked --offline
--config .cargo/verus.toml -- params_contract
link_css_full_record_differs_from_essence --nocapture`: **4 passed, 0 failed**,
48 filtered out. This includes two ordinary-Rust tests, one positive Verus client
(2 checked obligations), and one expected-failure control. The standalone clients
keep lifetime and trait-conflict checks enabled. The CSS test reproduces the
full-record/essence distinction independently of the model. Full package command
`bash verus_cmd.sh verify`: **206 net-traits / 87 script verified, 0 errors**,
with its previously documented reduced-checking flags. No contract formula or
native code changed; self-check 1/9–11 yes, normative-layout checks unchanged.

## Review follow-up: conditional theorem premises (2026-11-10)

Review items 1/2 and native findings. Factored the existing conjunction into
`type_matches_input_restrictions` (preload domain and CSS comparison) and
`type_matches_component_agreement` (parser, UA and category equalities). Their
conjunction retains the previous short-circuit branches and formulas. The report
now uses "native body equals the standard result assuming component agreement
on this input" and explicitly states that no native caller establishes either
checked body's requirements. Uninterpreted support/font/parser primitives supply
no independent evidence: agreement assumptions can hide unsupported/misclassified
cases rather than discover them.

The CSS Rust regression additionally calls actual `MimeClassifier::get_media_type`
and gets Css before checking essence/full-record inequality. Invalid reset findings
are qualified as method-level differences for nondefault prior state: the known
header caller starts at reset defaults. Fetch-priority step 7 remains unimplemented.
TagTerminatedByteMatcher's delegated `classify_requires` is explicitly documented
as currently true, not a strengthened domain. No upstream report was submitted.

`bash verus_cmd.sh verify`: **206 net-traits / 87 script verified, 0 errors**.
`cargo +1.97.1 test -p servo-net-traits --test mime_api --locked --offline
--config .cargo/verus.toml link_css_full_record_differs_from_essence -- --exact`:
**1 passed**. Rustfmt/diff checks passed. No trust labels, executable statements
or accepted input domain changed. Self-check 1/9–11 yes; numbered standard
algorithms unchanged. The theorem remains conditional and native capture unproved.

## Review completion: clean-source reproduction (2026-11-10)

Validated committed code/specification checkpoint
`cd1f11347955d4edf2e2fab01f173d2f973bba6d` in the clean detached worktree
`/tmp/opencode/servo-review-clean`. The baseline `b0de984` failure without the old
temporary override is recorded above. The corrected worktree used only committed
configuration/driver files and an archive prepared by that driver. Its Cargo
metadata resolves all three patched crates inside its own `target/verus-source`.
The same installed dirty verifier is an explicit prerequisite; this does not
establish a clean-machine build of the verification toolchain itself.

Executed from that worktree (the supplied clone can be on any branch):

```sh
bash verus_cmd.sh prepare /home/yizhiy/Desktop/verus
CARGO_TARGET_DIR=/home/yizhiy/Desktop/servo/target bash verus_cmd.sh verify
CARGO_TARGET_DIR=/home/yizhiy/Desktop/servo/target bash verus_cmd.sh native
CARGO_TARGET_DIR=/home/yizhiy/Desktop/servo/target cargo +1.97.1 test \
  -p servo-net-traits --test mime_api --locked --offline --config .cargo/verus.toml \
  -- params_contract link_css_full_record_differs_from_essence --nocapture
python3 components/script/dom/html/document_metadata/processingoptions_specs/check_trust_labels.py
git diff --exit-code HEAD --
git status --short
```

The target-directory setting reuses a build cache, not a source/config override.
Results: fresh **206 net-traits / 87 script verified, 0 errors** (1m 56s), native
check **passed** (46.74s), **4 regression tests passed / 48 filtered out**, trust
lint **80 files / 278 boundaries / 0 missing labels**, and clean worktree after
all commands. The package run retains both disabled checks; isolated regression
clients keep them enabled. Existing warnings are unchanged.

Review disposition: 1/2 conditional claims and unproved callers made explicit;
3 invocation snapshot shape corrected and invalid native hooks withdrawn;
4 lockfile history corrected; 5 portable config/driver committed and exercised;
6 iterator migration independently audited and regression-checked; 7 native
referrer frame proved. Annotation nits were corrected, and source-level findings
are qualified in STATUS.md. Native caller/effect/library verification and browser
reproduction remain unfinished; no upstream submission or thesis-text edit is
claimed. Self-check for this evidence-only checkpoint: 1/9–11 yes, 2–8 unchanged.

## Review of 073ff48: ordinary builds and registry comparison (2026-11-10)

Reproduced `cargo check -p servo-script --locked`, with the recorded bindgen flags
but **without `--config`**: Cargo failed before compilation because the lockfile
needed updating. `git diff --exit-code -- Cargo.lock` confirmed no rewrite.

Checked the preferred registry-only alternative first. The public crates.io API
reports vstd `0.0.0-2026-10-25-0158` as published and not yanked. Downloaded
`https://crates.io/api/v1/crates/vstd/0.0.0-2026-10-25-0158/download`; its SHA-256
matches the registry checksum
`e738decec9a809c06589281c48d96929e007079707d4e03018734daa4b20967f`.
Its `.cargo_vcs_info.json` records `9be4c4300440e393a54d3397a7443c2a34e8293e`.
Compared the extracted package with the unmodified Git archive of
`dea85441338f13fbab017b7c0ebafc9ad11464ce:source/vstd`. They differ substantively:
published `std_specs/iter.rs` still has `will_return_none`, while the pinned source
has `produces` and PR #3330's different iterator contracts. String, function,
character and other Rust specifications also differ. Same package version does
not establish source equivalence, so the registry-only replacement was rejected.

Restored relative root Cargo patches and pinned both direct vstd requirements to
`=0.0.0-2026-10-25-0158`. Removed the separate `.cargo/verus.toml`; ordinary builds
and both driver modes now select the same dependencies without `--config`.
Every fresh checkout must prepare the snapshot once. It now lives in ignored
`.verus-source`, outside the build target directory. `mach clean` was inspected:
it removes `.venv` and invokes `cargo clean`; neither removes this source snapshot.
Preparation uses Python `os.rename` instead of GNU `mv -T`. Preparation and shell
syntax were tested on Linux; no macOS execution is claimed.

Fixed the two removed preload-postcondition names and made the top-level command
section point to the current driver; earlier exact commands are explicitly
historical. The source-selection declaration and its precondition status are
unchanged, as requested; restoring the safety domain remains a prerequisite for
future verified calls.

Checks after preparation:

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo check -p servo-script --locked
bash verus_cmd.sh native
bash verus_cmd.sh verify
python3 components/script/dom/html/document_metadata/processingoptions_specs/check_trust_labels.py
git diff --exit-code -- Cargo.lock
git diff --check
```

Plain native check **passed** (51.26s); native driver **passed**, locked/offline and
without config; full verification **206 net-traits / 87 script verified, 0 errors**
(2m 21s). The driver expands to the documented full cargo-verus command, retaining
`--compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20`.
Trust lint remains **80 files / 278 boundaries / 0 missing labels**; no label or
specification formula changed. Cargo.lock stayed byte-for-byte unchanged. The
Cargo check alternative was used; no full `mach build`, WPT run or new library
correctness claim is made. Self-check 1/9–11 yes; normative-layout checks unchanged.

## Direct type matching and destination facts (2026-11-10)

The user's later instructions replace the type-matching adapter with a direct
steps-1–7 spec, preserve the supplied comment text, and remove its model parameter.
The native ensures now calls that spec directly; the old result wrapper is removed.
Step 2 compares the destination name with Fetch's separate translation of `"fetch"`.
The translation now recommends the actual potential-destination domain, excluding
the empty string as an input while permitting it as the translated result.

`lemma_destination_name_valid` and `translation_has_destination_type` are proved
broadcast lemmas with triggers on `d.as_str()` and the translation call respectively.
Both empty bodies pass without per-variant assertions, extra fuel or increased
limits. Preload and regression modules import the broadcasts. Existing regressions
hide the relevant definitions to check automatic use of the facts. No manual
destination-lemma calls were present to remove. DestinationView's type invariant
is retained because the existing tracked-value regression constructs it and calls
`use_type_invariant`; arbitrary spec-mode values use the separate proved fact.
No ensures was added to native external-body destination translation.

MIME parsing now calls a record/failure projection of the existing parser runner,
not an uninterpreted model method. Its result theorem excludes nonterminal states.
Record-level image/JavaScript/font predicates share their definitions with the
existing native-value predicates. Native parser result correspondence is obtained
from the existing trusted FromStr contract, so the additional free-parser agreement
premise is removed. UA support is a separately labelled predicate; support/category
agreement and the CSS restriction remain conditional obligations. The existing
parser model's documented Rust/WHATWG differences, especially quoted values, are
not resolved by this integration.

Actual parser regression proofs replace configurable fake MIME results. Shared
parser lemmas prove empty-type failure and successful header preservation through
unquoted-parameter processing. The CSS-with-charset regression proves its parse
outcome from syntax rather than assuming it. Successful CSS/JavaScript matching is
related to the explicit UA-support predicate rather than asserting decoder support.
The older generic matching machine remains checked separately, but is no longer
called by the active spec or its native proof/regressions.

Proof engineering: generated parser `take_step`/`next` helpers were not callable
across the crate boundary, so transition reasoning stays in proved parser-owned
lemmas. Native JavaScript/font checks initially failed after their predicates lost
the `essence_str(mt)` term that triggers existing decomposition lemmas. Shared
essence-list helpers retain that term for native values and also accept record
essences, restoring the proofs without changing the tables. The combined concrete
type-matching regression exceeded the default solver budget; separating the
independent cases resolved it without increasing any resource limit.

A minimal check confirmed `spec functions cannot have requires/ensures` in the
installed verifier. `when_used_as_spec` was also inspected: it needs matching
parameter and result types. Native `FromStr` returns `Result<Mime, FromStrError>`
from `&str`, unlike the record-level `Seq<char> -> Option<MimeView>` projection.
No native autospec binding or new native-result assumption was introduced.

Checks: `bash verus_cmd.sh verify` completed with **91 script verified, 0 errors**.
The unchanged net-traits sources were freshly checked at **210 verified, 0 errors**
in the preceding full run and reused by the final script rebuild. The command is
the full recorded cargo-verus command with `--compile --no-lifetime
--no-trait-conflicts --expand-errors --multiple-errors 20`. `bash verus_cmd.sh native`
**passed** (6.47s); Cargo.lock and native executable statements are unchanged.
Trust lint: **80 files, 276 boundary items, 0 missing labels**; 119 external-lib,
162 scope, 2 Verus gaps, 3 rewrites. Three formerly uninterpreted StandardModel
methods became defined; one labelled UA-support predicate replaces that method's
boundary. Self-check: native meaning/result obligations retained, standard ordering
visible, comment text preserved, assumptions explicit, no solver-limit increase.

Per the user's final publication instruction, these changes are a **new commit**
above `c81886f107b`, preserved on a backup branch before resetting `mime_classifier`
to `0ad88c2b74641a1ae675caea3d98858d19180ff9` and updating its remote branch.
