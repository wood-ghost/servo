# MIME specification regression tests

## Library specification layout

The production specification modules are under
`components/shared/net/mime_classifier_specs/mime_api/`:

| File | Responsibility |
| --- | --- |
| `mod.rs` | Existing umbrella exports, including the `SpecMime` import alias used by the classifier |
| `constants.rs` | All 38 `Name` and 33 `Mime` constants from mime 0.3.17, paired with their trusted external bindings; a separate section for parsing-literal identities |
| `mime.rs` | `MimeView`, view/essence functions, the `Mime` external type and operation contracts, and the essence proof lemmas |
| `name.rs` | `name_identity`, the `Name` external type and operation contracts, and equality axioms |
| `parser.rs` | `FromStr`/`str::parse` contracts, `FromStrError`, parsing requirements/results, automata and the existing legacy model |

The `mime` and `name` implementation modules are private, with their public
items re-exported through `mime_api`. In particular, a glob import of `mime_api`
does not introduce a module named `mime` that shadows the external Rust crate.
External library imports within these modules use the explicit `::mime` path.

Contracts and axioms remain visibly marked as trusted in their API-owning
modules. The layout refactor retains the existing predicates and assumptions,
including the unfinished parsing relation and known equality-model gaps.

## Test format

`mime_parse.rs` and `mime_api.rs` follow Verus's regression-test convention:

```rust
test_verify_one_file! {
    #[test] parse_plain_mime verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/html".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![]);
        }
    } => Ok(())
}
```

The outer macro creates an ordinary Cargo test that invokes Verus. The executable
`.parse::<Mime>()` call is **inside the verified snippet**. Assertions contain
hardcoded expected values observed from ordinary Rust execution of `mime 0.3.17`.
`=> Ok(())` requires successful verification; it does not merely require the Rust
library to accept the input.

MIME accessor results are bound in executable statements before being used in
assertions: these external methods currently lack specification-mode call
redirection. String contents are compared through `@`, rather than logical
equality of `&str` values. When testing `PartialEq`, the comparison itself runs
in an executable `let` statement and its boolean result is asserted. The harness
injects the existing `mime_api/mod.rs` umbrella as
`crate::mime_api`, including its external contracts.

## Cases and current specification work

The cases check type, base subtype, suffix, essence, and exact parameter-sequence
contents for unique parameter names. Inputs cover normalization, quoted values,
backslashes, Unicode, multiple parameters, suffix boundaries, and rejection.
Rejection tests still require `Ok(())`: Verus must prove that parsing returns
`Err` for the hardcoded input.

The duplicate-name case expects both ordered entries in `MimeView.params` and
checks the executable `get_param("foo")` result against `"first"`. Ordinary Rust
`params()` retains both duplicate entries; the current automaton filters the
second entry, so the Rust-expected sequence is a pending fidelity regression.

These are specifications-to-be-satisfied tests. `mime_parse_requires` currently
checks nothing (`true`), and the parsing contract uses the state-machine result.
Correspondence between that model and the Rust library remains unfinished.
The MIME cases keep `=> Ok(())` even while they fail; the tests add no assumptions,
external bodies, or exemptions.

Three `harness_*` controls distinguish a verified assertion, a deliberately
failing assertion, and Rust type error `E0308`. Only the latter two controls
expect an error. Missing tools, crashes, timeouts, and arbitrary compiler errors
do not count as an expected assertion failure.

### Parent-parser execution: 2026-10-13

`mime_parse_final_state` now runs from the initial state until `Done` or `Failure`.
Initialization and successor selection reuse `State::initialize` and `State::next`.
The generated `take_step` proof helpers supply initialization and transition
witnesses. Uniqueness, invariant preservation, and progress are proved over these
relations. Termination uses a lexicographic measure: the initial
trimming phase, remaining input, and phase rank. A constructive proof supplies
a completed trace whose last state is the returned result.

The `model_*` cases cover suffix splitting, the `text/html` path, malformed
headers, and two ordered parameters. The plain-MIME case also verifies an
executable `.parse::<Mime>()` client under a precondition fixing only the input's
character sequence. Its parsing-result facts are proved from the model.

Checked on Servo base `c76752ce50a` plus this change, using Verus
`0.2026.10.09.d4e03ec` (`verus-smoke`, Rust 1.98.1):

- `--verify-module mime_api::parser`: **56 verified, 0 errors**; sibling modules
  and the external Rust parser implementation are outside this proof scope.
- `mime_parse model_`: **4 passed, 0 failed**, using the Rust 1.95.0 Cargo driver.
- The original literal-based `parse_plain_mime` still fails its unwrap/component
  obligations. The current verifier lacks `auto_reveal_strlit`; the model cases
  use character sequences and explicit, proved evaluation steps.

Lifetime and trait-conflict checking remain enabled, with the default SMT
resource limit. Model proofs do not establish Rust-library correspondence:
the automaton still contains the documented WHATWG/Rust differences.

### Fixed-behavior regressions and SVG proofs: 2026-10-13

State-machine behavior is held fixed while its Rust/model differences are recorded
for later manual correction. The following cases keep their Rust-expected results
and `=> Ok(())`; they are neither ignored nor converted to expected-error tests:

| Case | Rust mime 0.3.17 | Current model difference |
| --- | --- | --- |
| Duplicate `foo` / `FOO` names | Both entries remain, in order | Later entry is discarded |
| Unquoted `foo=` at EOF | Stores the empty value | Reaches Done without storing it |
| `foo`, `foo;`, or `foo=;bar=baz` | Parsing returns Err | Skips the incomplete/empty parameter and reaches Done |
| `CHARSET=UTF-8` | Stores lowercase `utf-8` | Retains `UTF-8` |
| Quoted `é🦀` | Retains the Unicode value | Drops the entry during the parent's value validation |

`model_mime_parse_final_state_svg` proves the model result and an executable
parsing client for known input contents `image/svg+xml` and `IMAGE/SVG+XML`.
It checks type `image`, base subtype `svg`, suffix `xml`, and empty parameters.

Using the installed verifier `0.2026.10.12.033ca5e.dirty` (Rust 1.98.1), with the
Rust 1.95.0 Cargo driver:

- `mime_parse model_`: **5 passed, 0 failed**.
- The five fidelity-regression test functions above: **5 failed**, reaching proof
  checking rather than syntax/mode errors. Other unproved literal-evaluation,
  unwrap, and API-bridging obligations still appear in these diagnostics; these
  are not yet isolated proofs of just the behavioral differences.
- Independent ordinary-Rust execution against `mime 0.3.17`: **9 reference inputs
  passed**, comprising the seven mismatch inputs and the two SVG inputs.

### Current workflow: unmodified servo-stable, 2026-10-14

Active proof work is on Servo's `mime_classifier` branch and uses the original
Verus `servo-stable` revision `3a347310d098e784cdfbea97294315b079bb409e`.
The rebuilt release reports **`0.2026.09.24.3a34731`**, without `.dirty`.
Verus's tracked source is unchanged. Its release rebuild verified **2059 vstd
obligations, 0 errors**.

The previous experiment is preserved on the local branch
`checkpoint/mime-trigger-experiment-2026-10-14` in both repositories:

- Servo checkpoint: `47c227850ab` (including the existing lockfile adjustment and
  `test.log` deletion).
- Verus checkpoint: `a6e2e70c0` (the compiler trigger patch and its tests).

The active Servo implementation uses the supported sequence-index trigger form:

```rust
strslice_get_char(s, i) == #[trigger] s@[i]
```

A controlled check on the original verifier accepted and proved this lemma.
Its known-index client and explicit-index literal control also verified.
The raw-character-only and literal membership clients without an index witness
remained unproved (**3 verified, 2 errors** in that diagnostic). This records an
instantiation limitation of those clients, not a rejection of the trigger form.

For literal parse sites, `lemma_mime_parse_plain_type_hint` proves the separator,
HTTP-token conditions, and normalized type prefix before applying the general
parser theorem. The prefix length supplies an index witness for the existing
string/sequence axioms. Suffix clients use the analogous suffix-position helper.
These are reusable proofs, not assumptions or MIME-specific success axioms.
The original executable `.parse()` expressions and automaton transitions remain
intact; proof blocks supply the witnesses. The main result theorem still broadcasts
for symbolic inputs satisfying `mime_parse_plain_input`.

`mime_parse_final_state` is opaque to clients. Its initialization proof reveals
the definition and relates it to the DSL-based runner, so callers can use proved
contracts without automatically expanding the recursive parser. Witness helpers
are called explicitly rather than broadcast over every string in a query.

Checked with the unmodified verifier:

- The package command below: **207 verified, 0 errors**, with lifetime and
  trait-conflict checking disabled. `focus` compiles dependencies without checking
  their proofs.
- `sequence_index_trigger`, `parse_image_x_icon`, and `parse_plain_broadcast*`:
  **5 passed, 0 failed**. The snippet verifier keeps lifetime and trait-conflict
  checking enabled. No existing fidelity regression was ignored or had its
  Rust-expected result weakened.
- Rust formatting checks and `git diff --check` passed for the changed files.

```bash
cargo +1.98.1 verus focus --locked --offline -p servo-net-traits -- \
  --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20

VERUS_PATH=/home/yizhiy/Desktop/verus/source/target-verus/release/verus \
TMPDIR=/tmp/opencode \
cargo +1.95.0 test --locked --offline -p servo-net-traits --test mime_parse \
  --message-format=short -- sequence_index_trigger parse_image_x_icon \
  parse_plain_broadcast --test-threads=2
```

The success theorem covers parameter-free MIME syntax; the trusted external
parsing contract and documented model/library differences are still explicit
boundaries. The full MIME regression suite is not claimed to pass.

### Archived trigger-patch experiment: 2026-10-14

The following results describe the saved checkpoint, not the active verifier
requirement. The current workflow above does not use this compiler patch.

`lemma_mime_parse_plain` now proves successful model execution for arbitrary-length
`type/subtype` inputs with nonempty HTTP-token components and exactly one slash.
Its view includes ASCII normalization, last-non-leading-plus suffix splitting,
and empty parameters. The collector proof is inductive; no literal-dependent
unfolding bound is used. The generic theorem supersedes the icon-specific lemma.

The `mime_parse_lemmas` broadcast group connects the parsing contract's
`mime_parse_final_state(input)` term to this theorem. Proved bridges expose
literal-character witnesses and establish component, suffix, and essence facts.
Character witnesses are triggered only for parsing syntax/view queries. The
classifier enables string-literal revelation at its parsing functions and the
byte-matcher constructor impl, avoiding unnecessary expansion in comparison-only
predicates. Its executable parsing expressions and state-machine transitions are
unchanged. `is_javascript` needed its existing local resource limit increased
from 20 to 30 for the package-level checking context.

This uses Servo base `296200a05cb` plus the local proof changes and the local
Verus `servo-stable` base `3a347310d098` plus a trigger patch. The rebuilt release
reports `0.2026.09.24.3a34731.dirty`, with Rust 1.98.1. The Verus patch permits
`StrGetChar` at a manual trigger's root and classifies its operands as native,
matching their SMT representation. It adds no axioms. Before that patch, the new
direct-trigger regressions stopped at trigger validation and the nested-trigger
client failed its membership proof.

Checked with the patched release:

- Verus `triggers` suite: **41 passed**; `auto_reveal` suite: **28 passed**.
  These include four new trigger regressions; the negative control still rejects
  wrong-character and out-of-bounds membership claims.
- Release `vstd` rebuild: **2059 verified, 0 errors**.
- Servo package command below: **205 verified, 0 errors**. This checks all
  verification-enabled code selected by `focus -p servo-net-traits`, with **both
  lifetime and trait-conflict checking disabled** and dependency proofs skipped.
- `parse_image_x_icon` and `parse_plain_broadcast*`: **4 passed, 0 failed**,
  with lifetime and trait-conflict checking enabled in the snippet verifier.
  Coverage includes an arbitrary-input client under the syntax predicate, long
  and uppercase literals, structured suffixes, essence contents, and the
  predicate's boundaries. Sequence-content assertions use `ext_equal` where
  extensional reasoning is needed; this is logically equivalent to spec equality.
- Independent ordinary-Rust execution of mime 0.3.17 checked components, suffix,
  empty parameters, and essence for eight inputs: `image/x-icon`,
  `application/vnd.ms-fontobject`, `IMAGE/X-ICON`, `IMAGE/SVG+XML`,
  `application/a+b+json`, `*/*`, `text/vtt`, and `text/cache-manifest`.

```bash
cargo +1.98.1 verus focus --locked --offline -p servo-net-traits -- \
  --no-lifetime --no-trait-conflicts --expand-errors --multiple-errors 20

VERUS_PATH=/home/yizhiy/Desktop/verus/source/target-verus/release/verus \
TMPDIR=/tmp/opencode \
cargo +1.95.0 test --locked --offline -p servo-net-traits --test mime_parse \
  --message-format=short -- parse_image_x_icon parse_plain_broadcast --test-threads=2
```

The syntax predicate is a sufficient condition for this theorem, not an external
parser precondition or a complete MIME grammar. Parameterized inputs and surrounding
whitespace are outside its scope, and rejection of the predicate does not imply a
parse error. The trusted external parsing contract and the documented Rust/model
fidelity gaps remain. These focused results do not make the entire MIME regression
suite green or verify the external Rust parser implementation.

### Library constants

`mime_constants.rs` covers every public constant in the pinned crate: 38 `Name`
constants, 32 primary `Mime` constants, and the deprecated `TEXT_JAVSCRIPT` alias.
Small table macros expand to `test_verify_one_file!` / `verus_code!` cases with
hardcoded expectations and `=> Ok(())`. Name cases check `as_str`; Mime cases
check type, base subtype, suffix, exact parameter sequence, and essence. They use the
auto-reveal attribute, not explicit literal-revealing calls.

All constants are checked independently against ordinary-Rust accessor results.
Their Verus bindings are still **trusted** `assume_specification` declarations:
the passing model checks do not verify the external library implementation.

### Other MIME API functions

`mime_api.rs` contains 48 additional cases:

| Area | Observations checked |
| --- | --- |
| Accessors and cloning | Constant and parsed components, suffixes, preserved parameter values, `Mime::clone`, `Name::clone`/copy |
| Parameter lookup | Present/missing keys, string and `Name` arguments, case-insensitive lookup, first duplicate |
| Parameter iteration | `next`, `size_hint` before and after advancement, order, duplicate entries, repeated exhaustion, `Debug` |
| Equality | `Mime` and `Name`, strings in both operand orders, sensitivity, parameter spelling/order/value edge cases |
| Conversion and formatting | `AsRef<str>`, `Name::as_str`, `Name` into `&str`, `Display` through `to_string`, `Debug` |
| Ordering and hashing | `Ord`/`PartialOrd`, representation-sensitive ordering, bytes delivered to a recording `Hasher` |
| Direct parsing and errors | `Mime::from_str`, error display/debug output, `Error::source` |
| MIME-list iteration | `MimeIter::new`, `next`, `size_hint`, advancement, cloning at a noninitial position, error slices, exhaustion, `Debug` |

Constants isolate available accessor/clone contracts from parsing requirements.
The two constant-component tests use `#[verifier::auto_reveal_strlit]` and
sequence extensionality for essence contents. The generated crate also sets
`#![verifier::auto_reveal_strlit]` at its root for imported specification modules.
Use the local `servo-stable` verifier build for this attribute (see the branch
validation below). The hash tests use a small recording hasher and assert bytes, not
toolchain-dependent `DefaultHasher` numeric digests.

## Running

From the Servo root:

```bash
# The full suite currently fails on the unfinished MIME specifications.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_parse \
  -- --test-threads=2

# Run the additional API cases.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_api \
  -- --test-threads=2

# Check every library constant.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_constants \
  -- --test-threads=2

# Run both targets even if one still has failing contracts.
cargo +1.95.0 test --locked -p servo-net-traits \
  --test mime_parse --test mime_api --no-fail-fast -- --test-threads=2

# Check the verifier runner independently of the unfinished contracts.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_parse harness_

# Inspect one parsing contract test.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_parse \
  parse_plain_mime -- --exact --nocapture
```

Use the release verifier built from the local `servo-stable` branch on `PATH`,
or select it explicitly for a command:

```bash
VERUS_PATH=/home/yizhiy/Desktop/verus/source/target-verus/release/verus \
  cargo +1.95.0 test --locked -p servo-net-traits --test mime_api \
  constant_components -- --test-threads=2
```

The runner obtains
the verifier's Rust toolchain from `--version --output-json` and uses `rustup`
to build the pinned `mime` and state-machine macro dependencies with that
toolchain. The toolchain must already be installed. Dependency builds run
offline using a temporary copy of Servo's lockfile and the Cargo registry cache.

`common.rs` provides local macros with the Verus test calling convention.
`verus_code!` serializes Rust tokens with `stringify!`, preserving literal tokens
but not comments or original source spans. This avoids a dependency on Verus's
nightly-only source-span proc macro and compiler-internal test crate. Failures
retain generated `test.rs` and complete stdout/stderr logs, with a rerun command.

The runner uses the installed verifier's bundled `vstd` and builtins, plus
Servo's pinned `verus_state_machines_macros = 0.0.0-2026-06-14-0213`. As in
Verus's own runner, `--internal-test-mode` allows explicit library selection
without also injecting a second state-machine macro. Dependency artifacts are
cached below the test profile's `mime-spec-deps/<toolchain>/` directory.

Each invocation checks `--verify-root`, with `--multiple-errors 20`, one verifier
thread, the default SMT resource limit, and a 120-second subprocess timeout.
Lifetime and trait-conflict checking stay enabled. Only the snippet's functions
are verified; the imported parser automata and sibling API proofs are outside
this focused scope. Existing external MIME contracts and `vstd` remain trusted
boundaries. Passing client tests would not verify the dependency implementation
or establish general WHATWG conformance.

## Ordinary-Rust reference evidence

On 2026-10-06, before conversion to this Verus format, the corresponding 24
native Rust tests passed against `mime 0.3.17` with Rust `1.95.0`. They checked
complete successful results and rejection cases, including duplicate iteration
and first-match lookup. Those executions establish the hardcoded reference
values; they are not a verification pass for the current suite.

## Verus test evidence: 2026-10-06

- Servo base: `e2a3437c2eed0b21030dc13d5ca99690a635006a`, with this test patch.
- Verus source checkout: `ddfb159f0e4917614801167c99d71697942fb29b`.
- Installed verifier actually used: `0.2026.10.05.4558d3d.dirty`, release,
  Linux x86_64, reporting commit `4558d3d30b33d59e8ec7e519b7b6e7e9843edab5`
  and Rust `1.98.1-x86_64-unknown-linux-gnu`. Its dirty build is distinct from
  the current source checkout; the runner uses its bundled verification libraries.
- Cargo test driver: Rust `1.95.0`.

Executed from the Servo root, with `TMPDIR=/tmp/opencode`:

```bash
cargo +1.95.0 test --locked --offline -p servo-net-traits \
  --test mime_parse -- --test-threads=2
```

Result: **3 harness controls passed; all 24 MIME contract tests failed**.
The Cargo command returned exit status 101, as required by their `=> Ok(())`
expectations. The failures divide into:

- **23 verification failures:** unsatisfied `str::parse` call preconditions,
  insufficient success/suffix guarantees for `unwrap`, and unproved hardcoded
  component or rejection assertions. These reach proof checking; they are not
  syntax or mode errors. See `parser.rs`'s `mime_parse_requires` and
   `FromStrSpecImpl`, and the parsing declarations then in `trusted.rs`
   (now co-located in `parser.rs`).
- **1 missing library contract:** `parse_duplicate_parameter_lookup` stops at
  unsupported `mime::Mime::get_param`, before proving its first-match assertion.

No resource-limit failures occurred. The default SMT resource limit and enabled
lifetime/trait-conflict checks were retained. Formatting and diff checks passed.
This records the starting point for implementing the specifications, rather than
a passing MIME verification baseline.

## Additional API reference observations: 2026-10-08

A standalone ordinary-Rust probe was compiled and executed with both Rust
`1.95.0` and `1.98.1` against `mime = "=0.3.17"`. The output agreed exactly on
both toolchains, including formatting, iterator traces, and recorded hash-byte
writes. This supplies the hardcoded values in `mime_api.rs` independently of
the trusted MIME specifications.

Some pinned-library observations are particularly important for model fidelity:

| Operation and inputs | Ordinary-Rust result |
| --- | --- |
| Equality: `text/plain;foo=first` vs `text/plain;foo=second` | `true` in both directions |
| Equality: `text/plain;foo=first` vs `text/plain;foo=first;bar=last` | `true` in both directions |
| Equality after reordering `foo=first;bar=last` to `bar=last;foo=first` | `false` in both directions |
| `TEXT_PLAIN_UTF_8 == "text/plain; charset=abcde".parse::<Mime>().unwrap()` | `false`; reversing the operands gives `true` |
| `mime::TEXT` vs the `foo=text` parameter value from the probe | Both expose `"text"`, but `Name == Name` is `false` in both directions |
| Name hashing of those two `"text"` values | Concatenated writes are `b"text\xff\x01"` and `b"text\xff\x00"` respectively |
| Parameter iteration for `foo=first;FOO=second;bar=last` | Three entries retained; size hints advance from 3 to 2 to 1 to 0 |

These expectations deliberately describe the pinned implementation, including
its surprising equality behavior. In particular, asymmetric `Mime` equality
cannot be modeled by symmetric equality of views. Likewise, text alone does
not capture `Name` equality's sensitivity flag. The existing equality contracts,
formerly in `mime_api/trusted.rs` and now in `mime_api/mime.rs` and
`mime_api/name.rs`, need review against these observations. This is distinct
from a failed proof caused only by a missing library contract or solver hint.

The new tests retain `=> Ok(())`; they neither change these contracts nor assume
the desired outcomes. A successful future implementation of the specifications
must address their fidelity as well as the currently missing APIs.

## API verification evidence: 2026-10-08

- Servo base: `4fbecda3e06a0b9d235213c67a383592ce633ffd`, with this test patch.
- Verus source: `ddfb159f0e4917614801167c99d71697942fb29b`.
- Selected verifier: `0.2026.10.05.4558d3d.dirty`, release, Linux x86_64,
  using its bundled libraries and Rust `1.98.1-x86_64-unknown-linux-gnu`.
- Dependency: `mime 0.3.17`; Cargo test driver: Rust `1.95.0`.

Executed from the Servo root with `TMPDIR=/tmp/opencode`:

```bash
cargo +1.95.0 test --locked --offline -p servo-net-traits \
  --test mime_api --test mime_parse --no-fail-fast -- --test-threads=2
```

Results, counted as Rust test cases:

| Target | Passed | Failed |
| --- | ---: | ---: |
| `mime_api` | 4 | 44 |
| `mime_parse` | 3 harness controls | 24 MIME contract cases |

At that revision, the four passing API cases were `mime_constant_components`,
`mime_clone_constant_components`, `mime_eq_constants`, and `name_as_str`.
Of the other API cases, 22 reach proof checking and fail on current parsing,
comparison, cloning, conversion, or formatting contracts. Another 22 stop on
missing function/type/trait support, including `get_param`, `params`, `Params`,
`MimeIter`, `AsRef`, `Hash`, `Error`, and some constants. These are different
failure stages, not 44 demonstrated defects in Rust's implementation.

The parser target still reports 23 proof failures and the missing `get_param`
contract in the duplicate-lookup case. Its string-result assertions were
corrected to content equality (`s@ == "..."@`); the required runtime values and
`=> Ok(())` expectations are retained.

No resource limits or lifetime/trait-conflict checks were relaxed, and no
assumptions were added. Formatting and diff checks passed. The full command
returns 101 until the outstanding specification work is completed.

### Attribute-only literal revealing follow-up: 2026-10-08

The constant-component cases now use `#[verifier::auto_reveal_strlit]` on their
snippet functions in place of explicit literal-revealing calls. Their earlier
passing results above precede this change.

Focused check:

```bash
cargo +1.95.0 test --locked --offline -p servo-net-traits --test mime_api \
  constant_components -- --test-threads=2
```

At this stage, both cases stopped with **`unrecognized verifier attribute`** at
the function annotation using `0.2026.10.05.4558d3d.dirty`. This was a
verifier-feature/toolchain blocker before proof checking, resolved by the
`servo-stable` build below.

### Local `servo-stable` validation: 2026-10-08

The local Verus checkout was switched to `servo-stable` at
`3a347310d098e784cdfbea97294315b079bb409e` and rebuilt with `vargo build --release`.
The resulting executable identifies as **`0.2026.09.24.3a34731`**, release,
Linux x86_64, using Rust `1.98.1-x86_64-unknown-linux-gnu`. The build's `vstd`
verification reported **2059 verified, 0 errors**. Z3 was `4.16.0`.

Servo was at `acc228da4d33ff49b8a4be2cf14457748c71e4b4`. Its path-patched `vstd`
lock entry was aligned to this branch's `0.0.0-2026-09-20-0158` package version.
The test driver remained Rust `1.95.0`, with `mime 0.3.17` and the pinned
`verus_state_machines_macros 0.0.0-2026-06-14-0213` dependency.

The focused `constant_components` run, with `--locked --offline` and
`--test-threads=2 --nocapture`, passed **both constant-component cases**, using
only the auto-reveal attributes and no explicit literal-revealing calls.
The complete regression run was then executed from the Servo root:

```bash
VERUS_PATH=/home/yizhiy/Desktop/verus/source/target-verus/release/verus \
TMPDIR=/tmp/opencode \
  cargo +1.95.0 test --locked --offline -p servo-net-traits \
  --test mime_api --test mime_parse --no-fail-fast -- --test-threads=2
```

| Target | Passed | Failed |
| --- | ---: | ---: |
| `mime_api` | 4 | 44 |
| `mime_parse` | 3 harness controls | 24 MIME contract cases |

The API failures remain 22 proof failures and 22 missing-support failures.
The attribute-recognition error is gone; the remaining failures concern the
unfinished contracts documented above. The full command returns 101. Both
lifetime and trait-conflict checks remain enabled, and the SMT resource limit
was not changed. These focused client tests do not establish whole-classifier
or dependency-implementation verification.

### Type-oriented specification layout validation: 2026-10-09

The specification layout was reorganized on Servo base
`06c732921d18ac77964d2cccc05e0a10740508bf`. Both checks used the local
`servo-stable` verifier `0.2026.09.24.3a34731` at
`3a347310d098e784cdfbea97294315b079bb409e`, its bundled `vstd`, Rust `1.98.1`,
and the same pinned MIME/state-machine dependencies as the test runner.

The before/after regression command, from the Servo root, was:

```bash
VERUS_PATH=/home/yizhiy/Desktop/verus/source/target-verus/release/verus \
TMPDIR=/tmp/opencode \
  cargo +1.95.0 test --locked --offline -p servo-net-traits \
  --test mime_api --test mime_parse --no-fail-fast -- --test-threads=2
```

| Check | Before | After |
| --- | --- | --- |
| `mime_api` regression target | 4 passed, 44 failed | Same passing/failing cases; 22 proof failures and 22 missing-support failures |
| `mime_parse` regression target | 3 controls passed, 24 failed | Same passing/failing cases |
| All source modules of the isolated `mime_api` specification crate | 47 verified, 0 errors | 47 verified, 0 errors |

The isolated check used a temporary Rust entry point importing the actual
`mime_api/mod.rs`, with crate-level `auto_reveal_strlit`, and the dependency
artifacts supplied by the regression runner. It used `--internal-test-mode`,
`--crate-type=lib`, `--edition=2021`, `--output-json`, `--error-format=json`,
`--multiple-errors 20`, and `--num-threads 2`, without module filters. This checks
the parser-model proofs and the moved essence lemma, rather than only clients
of their contracts. A follow-up also checked `SpecMime`, wildcard imports,
unqualified `mime::Mime` resolution, and both re-exported broadcast groups using
the classifier's `broadcast use` syntax.

No lifetime/trait-conflict checks or SMT resource limits were relaxed. The
external specifications and axioms were moved with their formulas and trust
annotations; their existing fidelity gaps remain. The 47-verification result
concerns this isolated specification crate, not the external MIME implementation
or the whole classifier.

Formatting checks passed for the umbrella, constants, and new type modules.
The parser's existing model/legacy body formatting was retained; a whole-file
format check still proposes pre-existing whitespace and legacy indentation
changes there. `git diff --check` passed.

### Complete constant bindings: 2026-10-09

On Servo base `c18673b79e5a6f9cae807cf78a252efee11d9975`, the constant specifications
were extended from 21 to 71 bindings: 29 additional `Name` constants and 21
additional `Mime` constants. This includes all UTF-8 variants, wildcards,
`IMAGE_SVG`, font/application constants, parameter names, and the deprecated
alias. The existing `application_javascript_identity` was reused in the library
constant section. The binding inventory matches the pinned crate's public
constant declarations exactly, with no omissions or duplicate bindings.

A standalone ordinary-Rust checker directly read every constant and asserted
its expected components using the public accessors. It passed on both Rust
`1.95.0` and `1.98.1`: **38 Name values and 33 complete Mime results checked**.

Verification used the same local `servo-stable` build
`0.2026.09.24.3a34731` (`3a347310d098e784cdfbea97294315b079bb409e`):

```bash
VERUS_PATH=/home/yizhiy/Desktop/verus/source/target-verus/release/verus \
TMPDIR=/tmp/opencode \
  cargo +1.95.0 test --locked --offline -p servo-net-traits \
  --test mime_constants --test mime_api --test mime_parse --no-fail-fast \
  -- --test-threads=2
```

| Target | Passed | Failed |
| --- | ---: | ---: |
| `mime_constants` | 71 | 0 |
| `mime_api` | 5 | 43 |
| `mime_parse` | 3 harness controls | 24 MIME contract cases |

The existing `mime_constant_suffix` case now passes with the `IMAGE_SVG` binding.
The `get_param` cases still need method contracts, even though `CHARSET` is now
specified. The isolated specification/proof-module check still reports
**47 verified, 0 errors**. Lifetime and trait-conflict checks remain enabled,
and the default SMT resource limit was retained. Formatting and diff checks passed.
