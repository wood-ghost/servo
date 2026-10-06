# MIME specification regression tests

`mime_parse.rs` follows Verus's regression-test convention:

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
            assert(type_ == "text");
            assert(subtype == "html");
            assert(suffix.is_none());
            assert(essence == "text/html");
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
redirection. The harness injects the existing `mime_api/mod.rs` umbrella as
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

## Running

From the Servo root:

```bash
# The full suite currently fails on the unfinished MIME specifications.
cargo +1.95.0 test --locked -p servo-net-traits --test mime_parse \
  -- --test-threads=2

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
