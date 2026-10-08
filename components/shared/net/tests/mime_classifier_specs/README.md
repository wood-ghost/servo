# MIME specification regression tests

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
            assert(mime_view.params =~= map![]);
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

The cases check type, base subtype, suffix, essence, and exact parameter-map
contents for unique parameter names. Inputs cover normalization, quoted values,
backslashes, Unicode, multiple parameters, suffix boundaries, and rejection.
Rejection tests still require `Ok(())`: Verus must prove that parsing returns
`Err` for the hardcoded input.

The duplicate-name case checks the executable `get_param("foo")` result against
`"first"`. Ordinary Rust `params()` retains both duplicate entries in order; the
current `MimeView.params: Map` cannot represent that full sequence. This case
does not silently choose a new duplicate-to-map projection.

These are specifications-to-be-satisfied tests. `mime_parse_requires` is still
uninterpreted, and the general parsing-result relation remains unfinished.
The MIME cases keep `=> Ok(())` even while they fail. Implementing the contracts
is the next step; the tests add no assumptions, external bodies, or exemptions.

Three `harness_*` controls distinguish a verified assertion, a deliberately
failing assertion, and Rust type error `E0308`. Only the latter two controls
expect an error. Missing tools, crashes, timeouts, and arbitrary compiler errors
do not count as an expected assertion failure.

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
This requires a verifier build supporting the attribute (see the follow-up
below). The hash tests use a small recording hasher and assert bytes, not
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

# Run both targets even if one still has failing contracts.
cargo +1.95.0 test --locked -p servo-net-traits \
  --test mime_parse --test mime_api --no-fail-fast -- --test-threads=2

# Check the verifier runner independently of the unfinished contracts.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_parse harness_

# Inspect one parsing contract test.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_parse \
  parse_plain_mime -- --exact --nocapture
```

Use `verus` on `PATH`, or set `VERUS_PATH` to its executable. The runner obtains
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
  `FromStrSpecImpl`, and the parsing declarations in `trusted.rs`.
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
not capture `Name` equality's sensitivity flag. The existing equality contracts
in `mime_api/trusted.rs` need review against these observations. This is distinct
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

Both cases currently stop with **`unrecognized verifier attribute`** at the
function annotation, using the installed `0.2026.10.05.4558d3d.dirty` build.
This is a verifier-feature/toolchain blocker, before proof checking. A verifier
build with auto-reveal support is required to check these attribute-only cases.
