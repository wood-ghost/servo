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
