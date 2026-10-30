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
