# Current link-processing verification scope

**Working-tree status 2026-11-10 — `wood-ghost/servo`, branch `mime_classifier`**

Reviewed squash: `c81886f107b`. Its parent is `0ad88c2b746`; the original task and
review checkpoints mentioned in the historical log were combined into this commit.
The earlier clean-source validation was at the pre-squash **`cd1f1134795`**.
The per-task commands, results, step tables, self-checks, witnesses and corrections
are in [VERIFICATION.md](VERIFICATION.md). This summary supersedes its historical
intermediate statuses. The procedure makes correspondence reviewable; it does
not prove that a formal specification equals the standard's prose.

## Results and limitations

| Check | Result |
| --- | --- |
| Current script verification via `bash verus_cmd.sh verify` | **91 verified, 0 errors** |
| Last fresh net-traits verification of these changes | **210 verified, 0 errors** (reused by the final script-only rebuild) |
| Current `bash verus_cmd.sh native`, Rust 1.97.1, locked/offline | **Passed** |
| Earlier plain `cargo check -p servo-script --locked`, without `--config` | **Passed after source preparation; Cargo.lock unchanged** |
| Local/transitive trust-label lint | **80 Rust files; 276 labelled boundary items; 0 missing labels** |
| Temporary rewrite inventory | **3 let chains, each with original code retained** |
| Focused MIME iterator/CSS regressions (preceding review) | **4 passed, 0 failed** (including one expected-failure Verus control) |

### Destination broadcast facts — current working changes

- Retained `DestinationView::well_formed`'s type-invariant attribute under the
  requested tracked-construction exception. The existing
  `processing/tests.rs::domain_type_invariants_apply_to_tracked_values` accepts a
  tracked DestinationView, constructs `let tracked empty_destination =
  DestinationView { value: NativeDestination::None }`, and calls
  `use_type_invariant` on both. This is a proof regression, not a browser-runtime
  constructor. Arbitrary spec-mode values still need an independent domain fact.
- Enabled `lemma_destination_name_valid` as a broadcast proof with the requested
  `d.as_str()` trigger and `destination_types().contains(d.as_str())` conclusion.
  Restored `translation_has_destination_type` as a broadcast proof requiring
  `is_potential_destination(pd)` and triggering on
  `translate_a_potential_destination(pd)`.
- Translation now recommends `is_potential_destination(pd)`: the empty string
  is a destination type but is excluded from the recommended input domain;
  `"fetch"` is a potential destination and translates to the empty string.
- Added both broadcasts to `processing/algorithms/link_types/preload.rs` and
  `processing/tests.rs`. The existing domain/translation regression proofs hide
  the relevant definitions while checking the broadcast consequences, then reveal
  them for their concrete cases. No explicit calls to these destination lemmas
  remained to remove; `preload_translation_meets_linked_domain` already had an
  empty body. Native `translate_a_preload_destination` received no new contract.
- Both broadcast lemmas **verify with empty bodies**, and the hidden-definition
  regression checks pass. No per-variant assertions, extra fuel or resource-limit
  changes were needed for these facts. Native `translate_a_preload_destination`
  still has its original trusted contract, with no added result assumption.

### Direct MIME parsing in type matching

`type_matches_destination(type_, destination)` now directly expresses the seven
ordered steps and calls `mime_api::parse_mime_type`. It has no model parameter,
does not call the old matching-machine runner, and compares the destination with
`translate_a_potential_destination("fetch")` instead of checking the native None
variant. The supplied step-comment text and the native executable body are kept.
The older matching machine remains a separately checked reference artifact; its
broadcast/manual uses in the active native proof and regressions were removed.

`parse_mime_type` is a record/failure projection of the existing
`mime_parse_final_state` run. A new theorem proves that None corresponds to its
Failure state, not an unfinished run. Existing image, JavaScript and font
predicates now expose record-level forms, sharing their definitions with native
Mime predicates. Native parsing correspondence follows the **existing trusted
FromStr contract**; the extra uninterpreted `StandardModel.parse_mime_type`
agreement premise is removed. UA support remains an explicitly labelled
`is_supported_by_user_agent(&MimeView)` predicate and a native correspondence
premise. Legacy generic model methods delegate to these same direct operations.

This does not certify the parser model as a complete WHATWG parser. Its documented
Rust/WHATWG differences, notably quoted parameters, remain. The new parser proofs
establish termination/result projection, empty-type failure, and preservation of
the normalized header during unquoted-parameter processing, not external-library
or general standards correspondence.

Regressions now use actual parser-spec results instead of configurable fake MIME
results. They cover early returns, malformed input, plain/parameterized CSS and
JavaScript, with successful matching related explicitly to the UA-support
predicate. The parameterized CSS case proves success and essence from input
syntax alone. Its initial combined regression exceeded the default solver budget;
splitting independent cases resolved that without increasing the limit. Factoring
native MIME predicates initially hid the `essence_str(mt)` trigger used by existing
lemmas; shared essence-list helpers now preserve that trigger as well as record
entry points. No executable predicate or assumed contract was changed for this.

Final checks: `bash verus_cmd.sh verify` reports **91 script verified, 0 errors**;
the unchanged net-traits sources were freshly verified at **210/0** in the preceding
run. `bash verus_cmd.sh native` **passed** (6.47s). Trust lint and `git diff --check`
passed; Cargo.lock is unchanged. The earlier unresolved-model and private
generated-transition-helper errors are resolved using direct operations and proved
parser lemmas in the owning crate.

### Earlier successful checks

Both package results and both native checks were re-run on **2026-11-10** after
review of `073ff48`, using root patches to the recorded `.verus-source` snapshot
and no `--config`. The plain locked check first reproduced the missing-root-patch
failure before compilation, then passed after the fix; Cargo.lock stayed unchanged
throughout. The driver verification rechecked both packages (**206/87**, no errors).

The earlier clean-source run used a
detached worktree of `cd1f1134795`, the then-committed driver
and `.cargo/verus.toml`, and its own prepared source snapshot. Cargo's build cache
was shared via `CARGO_TARGET_DIR`; this was a clean-source check, not a cold-cache
or clean-machine build. Both packages were rechecked, all four focused regressions
passed, and the worktree (including Cargo.lock) remained clean afterward.
In the initial recheck, commands stopped at Cargo's lockfile check because the shared
Verus source checkout had moved to newer library versions. The final recheck
made **no further lockfile change**. The squashed implementation commit itself
**did** bump vstd, builtin-macros and vstd's state-machine macro dependency from
`2026-10-18-0147` to `2026-10-25-0158`. The checks used that committed October 25
lockfile; the direct `2026-06-14-0213` state-machine dependency is separate.

The verifier runs disable **lifetime and trait-conflict checking**. The counts
include reference proofs/helpers; they are not counts of verified native browser
algorithms. Native checking does not prove GC safety or browser conformance.

Two principal native algorithm bodies are checked against standard-result
contracts, under explicit restrictions:

The result is **"native body equals the standard result, assuming component
agreement on this input and the stated input restrictions"**. No native caller
currently proves these preconditions: preload, dispatch, the element caller and
the header loop are excluded, and servoparser's HTTP ingress is unverified.
Network/DOM input is not asserted to satisfy them. The type-matching proof checks
composition of the seven-step control flow, not independent component agreement.

- `type_matches_destination`: inherited MIME library requirements and its trusted
  parsing contract, plus UA/category correspondence, preload-destination domain
  and CSS essence/full-record agreement.
- `apply_link_options_from_parsed_header`: invalid CORS/referrer resets must preserve
  the old state, and recognized fetch priority must be representable as Auto.
  A finite-map lookup theorem supplies the native/attribute-map bridge. This
  function uses a documented local solver resource limit of 30. Its native-only
  frame also preserves `referrer`, which is absent from the standard options view.

| Remaining native area | Current status / reason |
| --- | --- |
| Header parameter lookup bodies | Trusted scope contracts for ordered first-match semantics |
| Preload destination / CORS-token matching | Trusted Verus-gap contracts; October 30 probes reproduced unsupported string patterns / const blocks |
| Request creation | Trusted standard-result contract; URL/Fetch-default/view correspondence unproved |
| Preload, per-link dispatch, header loop | Excluded after review; full standard plan/trace relations take supplied invocation snapshots/results; native capture is not implemented |
| Native header extraction | Excluded scope boundary; RFC list results versus HTML's singular object remain unresolved |
| Link-element preload caller and helpers | Explicitly trusted/excluded; DOM-input assumptions are recorded |
| Native fetch/listener callbacks | Excluded scope boundary; reference callback order/map effects are specified, not observed execution |

**Full-chain native conformance is not established.** Review found that the
reference-only `observed_*` producers would freeze mutable DOM state. They have
been removed; the full standard relations take explicit snapshot/plan/trace inputs.
The four uninstrumented native bodies are `external` rather than supplying those
invalid callable postconditions. This corrects the trusted boundary, not a failed
proof. Producing actual per-invocation traces and establishing their correspondence
remain unfinished. The new regression permits different preload plans for the same
element identity after changing its captured `as` keyword.

Source-set resolution/size/selection and media result obligations likewise take
`LinkHeaderDocumentState`, including its observable-state token, rather than only
document identity. Their native calls have signature-only boundaries until capture
is implemented. API base URL is taken from the returned URL value, not frozen as a
function of GlobalScope; immutable creation URLs retain their existing observations.

## Standard-facing records

- `EnumerateAttribute<State>`: conditional rename completed; finite states,
  keyword mapping and distinct invalid/missing/empty defaults retained.
- `SourceSetView`: identity-ordered image sources, URL/descriptor records and
  source-size syntax/used value. Native lazy size-list and URL resolution require
  explicit document context; f64/CSS correspondence remains trusted.
- `ServoUrlView`: all URL record components, distinct host/path forms, nullable
  query/fragment and a contextual blob entry. Native getter contracts are labelled.
- `OriginView`: opaque identity or scheme/host/nullable port/domain tuple;
  defined native variant projection and separate standard equality/domain operations.
- `EnvironmentView`: seven base fields, with the eight settings-object observations
  in a separate extension. GlobalScope, partial RequestClient, worker readiness,
  Performance time origin and supplemental context are distinguished rather than
  inventing native fields. Readiness is an observed atomic-read result, not an
  assumed immutable value; clock/owner correspondence remains trusted.
- `PolicyContainerView`: CSP, embedder, referrer, integrity and report-only integrity
  policies. Missing native self-origin/reporting/integrity data are supplemental.
- `DocumentView` / `DocsAlgoView` remain typed identities as requested. The user's
  `Arc<PolicyContainerView>`, `LinkProcessingOptionsView` and `type_` naming remain.

Native `referrer` is implementation data; the HTML options record retains
`referrer_policy` and `environment` rather than copying the native field layout.
Context-free projections remain explicitly unresolved where native data cannot
recover the whole standard record without supplemental context.

## Algorithm coverage

HTML snapshots below were inspected **2026-10-30**. Fetch/URL use the recorded
**2026-10-19** snapshots. Exact step-to-item/self-check tables are in the task log.

| Standard algorithm | Form and correspondence |
| --- | --- |
| [Type matches preload destination](https://html.spec.whatwg.org/multipage/links.html#match-preload-type) | Direct ordered specification, steps 1–7; direct MIME parser/predicate calls |
| [Apply parsed attributes](https://html.spec.whatwg.org/multipage/semantics.html#apply-link-options-from-parsed-header-attributes) | Separate 1, 1.1–1.4, 2–8 transitions; updates and nested returns |
| [Create link request](https://html.spec.whatwg.org/multipage/semantics.html#create-a-link-request) | 11 transitions; assertion guard and individual field updates |
| [Potential-CORS request](https://html.spec.whatwg.org/multipage/urls-and-fetching.html#create-a-potential-cors-request) | 5 transitions; mode/credentials updates |
| [Preload](https://html.spec.whatwg.org/multipage/links.html#preload) | 13 transitions; controller assignment and on-document-ready installation included |
| Preload response / commit callbacks | Separate 11.1–11.6 and 12.1–12.2 machines; ordered effects and explicit map mutation |
| [Extract Link headers](https://html.spec.whatwg.org/multipage/semantics.html#extract-links-from-headers) | Loop-head/body transitions; constructed complete trace; raw input preserved |
| [Process Link headers](https://html.spec.whatwg.org/multipage/semantics.html#process-link-headers) | 1, 2, 2.1–2.9 transitions; every accepted supplied coarse trace expands to a complete run |
| [Preconnect](https://html.spec.whatwg.org/multipage/links.html#preconnect) | 8 transitions; preserves permission to skip/partially establish a connection |
| [Element preload caller](https://html.spec.whatwg.org/multipage/links.html#link-type-preload:fetch-and-process-the-linked-resource-2) | 6 transitions and separate event callback 6.1; linked DOM observations trusted |
| [Enumerated attributes](https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#enumerated-attribute) | Direct ordered branches, including distinct defaults and NoState |
| [HTML destination translation](https://html.spec.whatwg.org/multipage/links.html#translate-a-preload-destination) | Direct 2-step specification calling the separate general Fetch algorithm |
| [Fetch destination translation](https://fetch.spec.whatwg.org/#concept-potential-destination-translate) | Direct returns; Step 2 Assert exported to a conditional entry obligation |
| [Preload key creation](https://html.spec.whatwg.org/multipage/links.html#create-a-preload-key) | Full unnumbered single-return definition; no invented step number |
| [Same origin](https://html.spec.whatwg.org/multipage/browsers.html#same-origin) / [effective domain](https://html.spec.whatwg.org/multipage/browsers.html#concept-origin-effective-domain) | Direct ordered returns |
| [Same origin-domain](https://html.spec.whatwg.org/multipage/browsers.html#same-origin-domain) | Separate parent/substep transitions; preserves nested returns |
| [URL equality](https://url.spec.whatwg.org/#concept-url-equals) | 3 transitions; serialization remains a labelled linked primitive |

For type matching, the user's later instruction selects a direct specification
with an immutable MIME temporary, superseding the earlier once-assigned-temporary
state-machine rule for this algorithm.
One textual-order qualification remains explicit: Fetch translation's Assert is
written in its entry recommendation, as allowed by the assertion-to-contract
rule, while return branches retain order. Older transitive MIME algorithms were
not all migrated to this new layout protocol; their library/model boundary is
documented rather than certified by the label lint.

## Input assumptions and findings

`type_matches_standard_requires` now names two separate classes of premise:

| Premise | Kind / evidence |
| --- | --- |
| Preload destinations only | Theorem-domain restriction; excluded example: Audio with `audio/mpeg` |
| CSS full-record comparison agrees with essence | Input restriction; `text/css; charset=utf-8` violates it, reproduced with the native classifier and mime crate |
| Native/model MIME parse results | Derived from the existing trusted FromStr contract and parser-result projection; external-library and full WHATWG correspondence remain unproved |
| Native recognition equals UA support | Model-agreement assumption; actual decoder support unmodeled. An unrecognized but supported type is ruled out by the premise rather than discovered |
| JS/image category agreement | Agreement premise against the defined MIME predicates; no mismatch witness found here |
| Font category agreement | Agreement premise against the defined record-level MIME predicate |

The UA-support predicate is not independently defined by a decoder implementation.
The theorem assumes its agreement with native recognition, so it supplies no
evidence that recognition adequately represents actual UA support. Parsing and
font-category results are now defined rather than free primitive choices, but
their library/standard correspondence must still be assessed separately. The
inherited MIME library requirements are obligations too, not web-input validation.

Requirements are labelled `IMPLICIT` or `INHERITED` and stop at HTTP/DOM inputs:
MIME/decoder agreement, representable priority, valid native reset behavior,
normalized nonempty source sets, and nonempty original/selected href. The header
candidate condition is explicitly sufficient and conservative after a possible
earlier return. The unused invariant inside the excluded native loop was removed;
requirements must be re-established at each iteration's captured state. Only the
reference loop invariant/trace construction is proved.

Recorded standard ambiguities include:

1. Potential/preload destination `fetch` versus translated empty Fetch destination,
   including the preload-key definition/constructor domain mismatch.
2. Fetch-priority **keyword** assignment versus the options field's **state** type.
3. Singular HTML parsed `linkObject` versus RFC/native multi-object results.
4. Source selection returning source/density while preload assigns to string href.
5. HTML's initially empty CSP list versus the newer CSP self-origin requirement;
   the inspected policy-clone algorithm omits copying report-only integrity policy.

HTML issues #9715 (URL encoding context) and #1142 (preload event behavior) remain
linked unresolved standard notes. The earlier SVG misclassification witness was
**withdrawn** after inspecting the explicit image exclusion in native `is_xml`.
The corrected native buffer-capacity adapter error was a model error, not a Servo bug.

### Native findings to carry into an upstream report

- **CSS:** the native classifier returns Css for `text/css; charset=utf-8`, but
  preload type matching compares the full record with `mime::TEXT_CSS` and rejects
  it. The ordinary-Rust regression checks classification, essence and inequality;
  a browser/WPT reproduction remains separate work.
- **Invalid attribute resets:** application assigns No CORS / empty referrer policy
  for invalid tokens where HTML preserves the previous fields. Witnesses use old
  UseCredentials / Origin and `"bad"`; `""` is not a CORS keyword. The current
  header loop initializes these fields to their reset defaults, so this
  method-level mismatch alone does not demonstrate an affected reachable HTTP path.
- **Fetch priority:** native step 7 is a TODO and the runtime options omit priority;
  the conditional theorem excludes recognized High/Low updates.

No upstream issue/PR was filed in this review. The historical task log is retained
for the thesis fork; it is not presented as an upstream-ready documentation patch.

## Trusted base and reproduction

Labels: **119 external-lib, 162 scope, 2 Verus gaps, 3 rewrites**. The complete
grouped item inventory is reproducible with the checker below. Many library
contracts are marked **not yet independently validated**, including the existing
general MIME FromStr correspondence and text-only Name equality model. Passing
these proofs does not validate those assumptions. The compiler/SMT/Verus/vstd and
generated machinery are also part of the trusted base; Verus sources were not edited.

The three old uninterpreted parser/support/font methods on StandardModel became
defined delegations; one standalone UA-support predicate replaces the support
method's trust boundary. No new parser or font assumption was introduced.

The `Mime::params` migration was audited against vstd's PR #3330: production
flags plus finite `remaining()` and `next`'s laws retain exact exhaustion behavior.
A generic client proves exact item count and repeated None, and ordinary Rust
checks all three native iterator representations. This validates selected cases
and the contract's composition, not the general external view or implementation.
Details and the focused test command are in
[`mime_classifier_specs/README.md`](../../../../../shared/net/tests/mime_classifier_specs/README.md#params-iterator-migration-audit-2026-11-10).

Recorded installed verifier: `0.2026.10.29.62a9ca4.dirty`, release, Rust **1.99.1**.
Dependency source snapshot: `dea85441338f13fbab017b7c0ebafc9ad11464ce`
(not claimed to be the installed binary revision). Native Servo toolchain: **1.97.1**.

During the earlier validation the shared Verus checkout changed independently from
`servo-stable` at `dea854413` to `char` at `bf0d49cdf`. Its November library
versions do not match Servo's October lock entries. The original successful recheck
used an unmodified archive of the recorded dependency revision. This was a local
override, not a reproducible repository configuration. The first portable setup
required `.cargo/verus.toml`; review of `073ff48` found that ordinary builds did
not select those patches and failed with `--locked`. The current setup restores
relative root patches, so native builds and verification select the same source.

The published, non-yanked vstd `0.0.0-2026-10-25-0158` was downloaded and compared
with `dea854413:source/vstd`. It is **not the same source**: its recorded revision
is `9be4c4300440e393a54d3397a7443c2a34e8293e`, and its iterator API still uses
`will_return_none()` instead of `produces()`. Other Rust files differ as well.
Consequently this branch retains pinned-source patches rather than switching to
the same-version registry package. vstd is now an exact-version dependency in
both servo-script and servo-net-traits, preventing resolution to a newer release.

### Portable source preparation and commands

Prerequisites: the recorded Verus/cargo-verus installation on PATH, Rust 1.99.1
and 1.97.1, Servo's native build dependencies, Bash/Git/tar/Python 3, and a Verus Git clone
containing `dea85441338f13fbab017b7c0ebafc9ad11464ce`. The recorded binary is a
dirty local build: its full build provenance is not reconstructible from this
Servo commit alone. Preparing the dependency source does not rebuild the verifier.

From the Servo root, using any such Verus Git clone:

```sh
bash verus_cmd.sh prepare /path/to/verus-git-clone
bash verus_cmd.sh verify
bash verus_cmd.sh native
```

The preparation command archives that exact revision into ignored
`.verus-source`, independent of the clone's current branch. It refuses to overwrite
an existing snapshot. The root `Cargo.toml` uses paths relative to this Servo
checkout; no command-line config is required. **Every fresh checkout, including
ordinary Cargo/mach builds, must run preparation once.** The source directory is
outside `target` and survives `cargo clean` / `./mach clean`; the earlier
`target/verus-source` location did not. Preparation uses Python's portable rename
instead of GNU-only `mv -T`. Both script modes retain `--locked`.
The script supplies the recorded Linux/GCC 13 bindgen flags by default; set
`BINDGEN_EXTRA_CLANG_ARGS` for a different native toolchain.

This resolves vstd/builtin-macros to `0.0.0-2026-10-25-0158` and builtin to
`0.0.0-2026-10-11-0230`, with their original transitive dependencies. The explicit
state-machine dependency remains `0.0.0-2026-06-14-0213`. Use the installed release
verifier named above; this is not a validation of the newer `char` sources.

Equivalent Cargo commands from the Servo root (with the same bindgen environment):

```sh
BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.99.1 verus focus -p servo-net-traits -p servo-script --locked -- \
  --compile --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20

BINDGEN_EXTRA_CLANG_ARGS="-isystem /usr/include/c++/13 -isystem /usr/include/x86_64-linux-gnu/c++/13 -isystem /usr/include/c++/13/backward" \
cargo +1.97.1 check -p servo-script --lib --locked --offline

python3 components/script/dom/html/document_metadata/processingoptions_specs/check_trust_labels.py
python3 components/script/dom/html/document_metadata/processingoptions_specs/check_trust_labels.py --inventory
```

The **2026-10-30** minimal probes rejected string patterns, let-chain conditions
and const blocks; equivalent supported positive controls verified 3 functions.
Exact errors, source shapes and commands are recorded in the task log. Those
probes were not rerun for this documentation checkpoint. All three temporary rewrites
retain their original source. No browser/WPT run, unrestricted input theorem,
native effect collector, or proof of complete library/GC behavior is claimed.
