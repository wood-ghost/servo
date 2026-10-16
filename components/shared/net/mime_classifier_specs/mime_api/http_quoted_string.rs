//! Quoted-string parser model and proofs for `mime 0.3.17`.
//!
//! The collector selects a completed trace. The accompanying proofs establish
//! existence and agreement of completed traces; WHATWG/Rust differences are
//! documented alongside the transitions.

use verus_state_machines_macros::{case_on_next, state_machine};
use vstd::prelude::*;

use super::utils::is_rust_quoted_value_char;

verus! {

// Historical compatibility-only model notes; these are no longer input restrictions.
// Common quoted-value domain for the simplified scan. Rust mime 0.3.17 rejects TAB
// and mishandles empty quoted values; backslashes have different escaping semantics.
// The remaining character restriction comes from WHATWG MIME parameter validation.
    // Compatibility restriction: exclude a reachable missing-closing-quote error.
    // Covers both EOF after ordinary content ("abc) and EOF after a backslash ("abc\).
    // WHATWG accepts both; Rust mime 0.3.17 rejects these unterminated quoted values.
    // TODO: Extend the automaton and compatibility conditions for the remaining branches.
    // Also exclude collected values outside the common quoted-value domain.

pub ghost enum QuotedStringError {
    MissingQuote,
    InvalidToken,
}

// https://fetch.spec.whatwg.org/#collect-an-http-quoted-string
// Historical scaffold notes (superseded by the Rust-parser trace implementation below):
// TODO: Define using the quoted-string automaton.
// Models extract-value = true for MIME parsing.
// Returns (extracted value, updated position); currently a dummy implementation.
// Intended entry: 0 <= position < input.len() and input[position] == '\u{0022}'.
pub open spec fn collect_http_quoted_string(
    input: Seq<char>,
    position: int,
    extract_value: bool,
) -> Result<(Seq<char>, int), QuotedStringError>
    recommends
        0 <= position < input.len() && input[position] == '\u{0022}',
{
    // TODO: Replace this stub with the Fetch quoted-string automaton.
    // TODO: Use extract_value here to select the final value or consumed input after choosing a trace.
    // This only skips the opening quote and returns an empty value; it does not parse quotes.
    // The former stub above is now replaced by completed-trace selection.
    if 0 <= position < input.len() && input[position] == '\u{0022}' {
        let trace = choose|trace: Seq<HttpQuotedStringAutomaton::State>|
            http_quoted_string_complete_trace(input, position, trace);
        http_quoted_string_output(trace.last(), extract_value)
    } else {
        Err(QuotedStringError::InvalidToken)
    }
}

pub ghost enum HttpQuotedStringState {
    Init,
    ReadingValue,
    ContentReady,
    ContentAppended,
    ClosingQuoteReady,
    ClosingQuoteConsumed,
    Done,
    MissingQuote,
    InvalidToken,
}

state_machine! {
    HttpQuotedStringAutomaton {
        fields {
            pub input: Seq<char>,
            pub position: int,
            pub position_start: int,
            pub value: Seq<char>,
            pub state: HttpQuotedStringState,
        }

        // To collect an HTTP quoted string from a string input, given a position variable position and an optional boolean extract-value (default false):
        init! {
            initialize(input: Seq<char>, position: int) {
                // 3. Assert: the code point at position within input is U+0022 (").
                require(0 <= position < input.len() && input[position] == '\u{0022}');

                init input = input;
                init position = position;
                init state = HttpQuotedStringState::Init;

                // 1. Let positionStart be position.
                init position_start = position;
                // 2. Let value be the empty string.
                init value = Seq::empty();
            }
        }

        transition! {
            skip_opening_quote() {
                require(pre.state == HttpQuotedStringState::Init);

                // 4. Advance position by 1.
                update position = pre.position + 1;
                update state = HttpQuotedStringState::ReadingValue;
            }
        }

        // 5.1 Append the result of collecting a sequence of code points that are not U+0022 (") or U+005C (\) from input, given position, to value.
        // Rust-style deviation: mime 0.3.17 parser, U+005C (\) is ordinary quoted-value data:
        // it is preserved and does not escape the following character.
        // For nonempty quoted values containing accepted bytes, scanning continues
        // through backslashes and stops at U+0022 (") or EOF.
        // Example (actual header text): foo="a\b" retains a\b in Rust, whereas
        // WHATWG extracts ab. Excluding backslashes from quoted values is therefore
        // a compatibility restriction for agreement with WHATWG.
        transition! {
            append_content() {
                require(pre.state == HttpQuotedStringState::ContentReady);

                // Append one accepted scalar. Backslashes, and a quote at the first
                // value position, are ordinary data in mime 0.3.17.
                update value = pre.value.push(pre.input[pre.position]);
                update state = HttpQuotedStringState::ContentAppended;
            }
        }

        transition! {
            advance_value_position() {
                require(pre.state == HttpQuotedStringState::ContentAppended);

                update position = pre.position + 1;
                update state = HttpQuotedStringState::ReadingValue;
            }
        }

        // 5.2 If position is past the end of input, then break.
        // Rust-style deviation: mime 0.3.17 rejects an unterminated quoted value
        // (e.g. text/html;foo="abc) with MissingQuote; WHATWG returns the accumulated value.
        transition! {
            check_position_1() {
                require(pre.state == HttpQuotedStringState::ReadingValue);

                if pre.position >= pre.input.len() {
                    update state = HttpQuotedStringState::MissingQuote;
                } else if pre.input[pre.position] == '\u{0022}'
                    && pre.position > pre.position_start + 1 {
                    // Match mime 0.3.17's `i > start` guard: an immediate quote
                    // is data, so "" reaches MissingQuote but """ contains a quote.
                    update state = HttpQuotedStringState::ClosingQuoteReady;
                } else if is_rust_quoted_value_char(pre.input[pre.position]) {
                    update state = HttpQuotedStringState::ContentReady;
                } else {
                    update state = HttpQuotedStringState::InvalidToken;
                }
            }
        }

        // 5.3 Let quoteOrBackslash be the code point at position within input.
        // Rust-style deviation: mime 0.3.17 treats the backslash as ordinary quoted-value data therefore there is no need to distinguish quote and backslash within quoted values.
        transition! {
            skip_past_delimiter() {
                require(pre.state == HttpQuotedStringState::ClosingQuoteReady);

                // 5.4 Advance position by 1.
                update position = pre.position + 1;
                update state = HttpQuotedStringState::ClosingQuoteConsumed;
            }
        }

        transition! {
            finish() {
                require(pre.state == HttpQuotedStringState::ClosingQuoteConsumed);
                require(0 < pre.position <= pre.input.len());

                // 5.5 If quoteOrBackslash is U+005C (\), then:
                // Rust-style deviation: this branch no longer exists
                // 6. Otherwise:
                // 6.1 Assert: quoteOrBackslash is U+0022 (").
                // The consumed character is represented by input[position - 1].
                // Proved in finish_inductive using closing_quote_was_consumed.
                require(pre.input[pre.position - 1] == '\u{0022}');
                // 6.2 Break.
                update state = HttpQuotedStringState::Done;
            }
        }

        //TODO: other strings whatwg accepts but Rust rejects


        #[invariant]
        pub fn positions_in_bounds(&self) -> bool {
            0 <= self.position_start < self.input.len()
                && self.position_start <= self.position <= self.input.len()
        }

        #[invariant]
        pub fn start_is_opening_quote(&self) -> bool {
            self.input[self.position_start] == '\u{0022}'
        }

        #[invariant]
        pub fn initial_values(&self) -> bool {
            self.state == HttpQuotedStringState::Init ==> (
                self.position == self.position_start
                && self.value == Seq::<char>::empty()
            )
        }

        #[invariant]
        pub fn missing_quote_is_at_end(&self) -> bool {
            self.state == HttpQuotedStringState::MissingQuote
                ==> self.position == self.input.len()
        }

        #[invariant]
        pub fn scan_positions(&self) -> bool {
            self.state != HttpQuotedStringState::Init
                ==> self.position >= self.position_start + 1
        }

        #[invariant]
        pub fn content_position_is_valid(&self) -> bool {
            (self.state == HttpQuotedStringState::ContentReady
                || self.state == HttpQuotedStringState::ContentAppended) ==> (
                0 <= self.position < self.input.len()
                && is_rust_quoted_value_char(self.input[self.position])
            )
        }

        #[invariant]
        pub fn value_length_matches_position(&self) -> bool {
            match self.state {
                HttpQuotedStringState::Init => self.value.len() == 0,
                HttpQuotedStringState::ContentAppended =>
                    self.value.len() == self.position - self.position_start,
                HttpQuotedStringState::ClosingQuoteConsumed | HttpQuotedStringState::Done =>
                    self.value.len() == self.position - self.position_start - 2,
                _ => self.value.len() == self.position - self.position_start - 1,
            }
        }

        #[invariant]
        pub fn invalid_token_is_at_invalid_character(&self) -> bool {
            self.state == HttpQuotedStringState::InvalidToken ==> (
                0 <= self.position < self.input.len()
                && !is_rust_quoted_value_char(self.input[self.position])
            )
        }

        #[invariant]
        pub fn closing_quote_ready_points_to_quote(&self) -> bool {
            self.state == HttpQuotedStringState::ClosingQuoteReady ==> (
                0 <= self.position < self.input.len()
                && self.input[self.position] == '\u{0022}'
                && self.position > self.position_start + 1
            )
        }

        #[invariant]
        pub fn closing_quote_was_consumed(&self) -> bool {
            (
                self.state == HttpQuotedStringState::ClosingQuoteConsumed
                || self.state == HttpQuotedStringState::Done
            ) ==> (
                0 < self.position <= self.input.len()
                && self.input[self.position - 1] == '\u{0022}'
                && self.position > self.position_start + 2
            )
        }

        #[inductive(initialize)]
        fn initialize_inductive(post: Self, input: Seq<char>, position: int) {}

        #[inductive(skip_opening_quote)]
        fn skip_opening_quote_inductive(pre: Self, post: Self) {}

        #[inductive(append_content)]
        fn append_content_inductive(pre: Self, post: Self) {}

        #[inductive(advance_value_position)]
        fn advance_value_position_inductive(pre: Self, post: Self) {}

        #[inductive(check_position_1)]
        fn check_position_1_inductive(pre: Self, post: Self) {}

        #[inductive(skip_past_delimiter)]
        fn skip_past_delimiter_inductive(pre: Self, post: Self) {}

        #[inductive(finish)]
        fn finish_inductive(pre: Self, post: Self) {}
    }
}

// The explicit finish guards follow from the phase invariant, so they cannot block
// a valid execution after the closing quote has been consumed.
pub proof fn http_quoted_string_finish_enabled(pre: HttpQuotedStringAutomaton::State)
    requires
        pre.invariant(),
        pre.state == HttpQuotedStringState::ClosingQuoteConsumed,
    ensures HttpQuotedStringAutomaton::State::finish_enabled(pre),
{
}

// A finite execution prefix. Initialization is independent of compatibility requirements,
// so error-reaching executions remain available when defining those requirements.
pub open spec fn http_quoted_string_steps(trace: Seq<HttpQuotedStringAutomaton::State>) -> bool {
    forall|i: int| 0 <= i && i + 1 < trace.len() ==>
        #[trigger] HttpQuotedStringAutomaton::State::next(trace[i], trace[i + 1])
}

pub proof fn http_quoted_string_step_at(trace: Seq<HttpQuotedStringAutomaton::State>, i: int)
    requires http_quoted_string_steps(trace), 0 <= i, i + 1 < trace.len(),
    ensures HttpQuotedStringAutomaton::State::next(trace[i], trace[i + 1]),
{
    reveal(http_quoted_string_steps);
}

pub open spec fn http_quoted_string_trace(
    input: Seq<char>,
    position: int,
    trace: Seq<HttpQuotedStringAutomaton::State>,
) -> bool {
    &&& trace.len() > 0
    &&& HttpQuotedStringAutomaton::State::initialize(trace.first(), input, position)
    &&& http_quoted_string_steps(trace)
}

pub open spec fn http_quoted_string_terminal(state: HttpQuotedStringState) -> bool {
    state == HttpQuotedStringState::Done
        || state == HttpQuotedStringState::MissingQuote
        || state == HttpQuotedStringState::InvalidToken
}

pub open spec fn http_quoted_string_complete_trace(
    input: Seq<char>,
    position: int,
    trace: Seq<HttpQuotedStringAutomaton::State>,
) -> bool {
    http_quoted_string_trace(input, position, trace)
        && http_quoted_string_terminal(trace.last().state)
}

pub open spec fn http_quoted_string_initial_state(
    input: Seq<char>, position: int,
) -> HttpQuotedStringAutomaton::State {
    HttpQuotedStringAutomaton::State {
        input,
        position,
        position_start: position,
        value: Seq::empty(),
        state: HttpQuotedStringState::Init,
    }
}

// A deterministic witness for a single transition. Its relationship to `next`
// is proved below; it is used to construct a terminating witness trace.
pub open spec fn http_quoted_string_step(
    pre: HttpQuotedStringAutomaton::State,
) -> HttpQuotedStringAutomaton::State {
    match pre.state {
        HttpQuotedStringState::Init => HttpQuotedStringAutomaton::State {
            position: pre.position + 1,
            state: HttpQuotedStringState::ReadingValue,
            ..pre
        },
        HttpQuotedStringState::ReadingValue => HttpQuotedStringAutomaton::State {
            state: if pre.position >= pre.input.len() {
                HttpQuotedStringState::MissingQuote
            } else if pre.input[pre.position] == '\u{0022}'
                && pre.position > pre.position_start + 1 {
                HttpQuotedStringState::ClosingQuoteReady
            } else if is_rust_quoted_value_char(pre.input[pre.position]) {
                HttpQuotedStringState::ContentReady
            } else {
                HttpQuotedStringState::InvalidToken
            },
            ..pre
        },
        HttpQuotedStringState::ContentReady => HttpQuotedStringAutomaton::State {
            value: pre.value.push(pre.input[pre.position]),
            state: HttpQuotedStringState::ContentAppended,
            ..pre
        },
        HttpQuotedStringState::ContentAppended => HttpQuotedStringAutomaton::State {
            position: pre.position + 1,
            state: HttpQuotedStringState::ReadingValue,
            ..pre
        },
        HttpQuotedStringState::ClosingQuoteReady => HttpQuotedStringAutomaton::State {
            position: pre.position + 1,
            state: HttpQuotedStringState::ClosingQuoteConsumed,
            ..pre
        },
        HttpQuotedStringState::ClosingQuoteConsumed => HttpQuotedStringAutomaton::State {
            state: HttpQuotedStringState::Done,
            ..pre
        },
        _ => pre,
    }
}

pub open spec fn http_quoted_string_phase_rank(state: HttpQuotedStringState) -> nat {
    match state {
        HttpQuotedStringState::Init => 7,
        HttpQuotedStringState::ReadingValue => 6,
        HttpQuotedStringState::ContentReady => 5,
        HttpQuotedStringState::ContentAppended => 4,
        HttpQuotedStringState::ClosingQuoteReady => 3,
        HttpQuotedStringState::ClosingQuoteConsumed => 2,
        _ => 0,
    }
}

pub open spec fn http_quoted_string_trace_from(
    pre: HttpQuotedStringAutomaton::State,
) -> Seq<HttpQuotedStringAutomaton::State>
    decreases pre.input.len() - pre.position, http_quoted_string_phase_rank(pre.state),
{
    // Totalize the witness constructor on malformed states without making its
    // computation depend on proof invariants. Valid nonterminal states keep stepping.
    if http_quoted_string_terminal(pre.state)
        || pre.position < 0 || pre.position > pre.input.len()
        || (pre.position == pre.input.len()
            && pre.state != HttpQuotedStringState::ReadingValue
            && pre.state != HttpQuotedStringState::ClosingQuoteConsumed) {
        seq![pre]
    } else {
        seq![pre] + http_quoted_string_trace_from(http_quoted_string_step(pre))
    }
}

pub proof fn http_quoted_string_step_correct(pre: HttpQuotedStringAutomaton::State)
    requires pre.invariant(), !http_quoted_string_terminal(pre.state),
    ensures
        http_quoted_string_step(pre).invariant(),
        HttpQuotedStringAutomaton::State::next(pre, http_quoted_string_step(pre)),
        http_quoted_string_step(pre).input == pre.input,
        http_quoted_string_step(pre).position_start == pre.position_start,
        pre.position <= http_quoted_string_step(pre).position <= pre.input.len(),
{
    let post = http_quoted_string_step(pre);
    match pre.state {
        HttpQuotedStringState::Init => {
            HttpQuotedStringAutomaton::show::skip_opening_quote(pre, post);
        },
        HttpQuotedStringState::ReadingValue => {
            HttpQuotedStringAutomaton::show::check_position_1(pre, post);
        },
        HttpQuotedStringState::ContentReady => {
            HttpQuotedStringAutomaton::show::append_content(pre, post);
        },
        HttpQuotedStringState::ContentAppended => {
            HttpQuotedStringAutomaton::show::advance_value_position(pre, post);
        },
        HttpQuotedStringState::ClosingQuoteReady => {
            HttpQuotedStringAutomaton::show::skip_past_delimiter(pre, post);
        },
        HttpQuotedStringState::ClosingQuoteConsumed => {
            HttpQuotedStringAutomaton::show::finish(pre, post);
        },
        _ => {},
    }
}

pub proof fn http_quoted_string_next_is_deterministic(
    pre: HttpQuotedStringAutomaton::State,
    post: HttpQuotedStringAutomaton::State,
)
    requires HttpQuotedStringAutomaton::State::next(pre, post),
    ensures post == http_quoted_string_step(pre), !http_quoted_string_terminal(pre.state),
{
    case_on_next! { pre, post, HttpQuotedStringAutomaton => {
        skip_opening_quote() => {}
        check_position_1() => {}
        append_content() => {}
        advance_value_position() => {}
        skip_past_delimiter() => {}
        finish() => {}
    }}
}

pub proof fn http_quoted_string_trace_from_properties(pre: HttpQuotedStringAutomaton::State)
    requires pre.invariant(),
    ensures ({
        let trace = http_quoted_string_trace_from(pre);
        &&& trace.len() > 0
        &&& trace.first() == pre
        &&& http_quoted_string_steps(trace)
        &&& trace.last().invariant()
        &&& http_quoted_string_terminal(trace.last().state)
        &&& trace.last().input == pre.input
        &&& trace.last().position_start == pre.position_start
        &&& pre.position <= trace.last().position <= pre.input.len()
    }),
    decreases pre.input.len() - pre.position, http_quoted_string_phase_rank(pre.state),
{
    if !http_quoted_string_terminal(pre.state) {
        http_quoted_string_step_correct(pre);
        let post = http_quoted_string_step(pre);
        http_quoted_string_trace_from_properties(post);
        let rest = http_quoted_string_trace_from(post);
        let trace = seq![pre] + rest;
        assert forall|i: int| 0 <= i && i + 1 < trace.len() implies
            #[trigger] HttpQuotedStringAutomaton::State::next(trace[i], trace[i + 1]) by {
            if i > 0 {
                http_quoted_string_step_at(rest, i - 1);
            }
        }
    }
}

pub proof fn http_quoted_string_complete_trace_unique(
    pre: HttpQuotedStringAutomaton::State,
    trace: Seq<HttpQuotedStringAutomaton::State>,
)
    requires
        pre.invariant(),
        trace.len() > 0,
        trace.first() == pre,
        http_quoted_string_steps(trace),
        http_quoted_string_terminal(trace.last().state),
    ensures trace.last() == http_quoted_string_trace_from(pre).last(),
    decreases trace.len(),
{
    if trace.len() > 1 {
        http_quoted_string_step_at(trace, 0);
        http_quoted_string_next_is_deterministic(pre, trace[1]);
        http_quoted_string_step_correct(pre);
        let rest = trace.subrange(1, trace.len() as int);
        http_quoted_string_complete_trace_unique(trace[1], rest);
        http_quoted_string_trace_from_properties(trace[1]);
    }
}

pub open spec fn http_quoted_string_output(
    final_state: HttpQuotedStringAutomaton::State,
    extract_value: bool,
) -> Result<(Seq<char>, int), QuotedStringError> {
    match final_state.state {
        HttpQuotedStringState::Done => Ok((
            if extract_value {
                final_state.value
            } else {
                final_state.input.subrange(final_state.position_start, final_state.position)
            },
            final_state.position,
        )),
        HttpQuotedStringState::MissingQuote => Err(QuotedStringError::MissingQuote),
        _ => Err(QuotedStringError::InvalidToken),
    }
}

// A completed trace exists for every structurally valid invocation, including failures.
// All completed traces agree, so the outer function's choice has a unique result.
pub proof fn collect_http_quoted_string_properties(
    input: Seq<char>, position: int, extract_value: bool,
)
    requires 0 <= position < input.len(), input[position] == '\u{0022}',
    ensures
        collect_http_quoted_string(input, position, extract_value)
            == http_quoted_string_output(
                http_quoted_string_trace_from(http_quoted_string_initial_state(input, position)).last(),
                extract_value,
            ),
        match collect_http_quoted_string(input, position, extract_value) {
            Ok((_, end)) => position < end <= input.len(),
            Err(_) => true,
        },
{
    let initial = http_quoted_string_initial_state(input, position);
    http_quoted_string_trace_from_properties(initial);
    let witness = http_quoted_string_trace_from(initial);
    // Supply an explicit witness for the completed-trace choice.
    assert(http_quoted_string_complete_trace(input, position, witness));
    let trace = choose|trace: Seq<HttpQuotedStringAutomaton::State>|
        http_quoted_string_complete_trace(input, position, trace);
    http_quoted_string_complete_trace_unique(initial, trace);
}

pub proof fn http_quoted_string_input_and_start_preserved(
    pre: HttpQuotedStringAutomaton::State,
    post: HttpQuotedStringAutomaton::State,
)
    requires HttpQuotedStringAutomaton::State::next(pre, post),
    ensures
        post.input == pre.input,
        post.position_start == pre.position_start,
        post.state != HttpQuotedStringState::Init,
{
    case_on_next! { pre, post, HttpQuotedStringAutomaton => {
        skip_opening_quote() => {}
        append_content() => {}
        advance_value_position() => {}
        check_position_1() => {}
        skip_past_delimiter() => {}
        finish() => {}
    }}
}

pub proof fn http_quoted_string_missing_quote_is_terminal(
    pre: HttpQuotedStringAutomaton::State,
    post: HttpQuotedStringAutomaton::State,
)
    requires pre.state == HttpQuotedStringState::MissingQuote,
    ensures !HttpQuotedStringAutomaton::State::next(pre, post),
{
    if HttpQuotedStringAutomaton::State::next(pre, post) {
        case_on_next! { pre, post, HttpQuotedStringAutomaton => {
            skip_opening_quote() => {}
            append_content() => {}
            advance_value_position() => {}
            check_position_1() => {}
            skip_past_delimiter() => {}
            finish() => {}
        }}
    }
}

pub proof fn http_quoted_string_done_is_terminal(
    pre: HttpQuotedStringAutomaton::State,
    post: HttpQuotedStringAutomaton::State,
)
    requires pre.state == HttpQuotedStringState::Done,
    ensures !HttpQuotedStringAutomaton::State::next(pre, post),
{
    if HttpQuotedStringAutomaton::State::next(pre, post) {
        case_on_next! { pre, post, HttpQuotedStringAutomaton => {
            skip_opening_quote() => {}
            append_content() => {}
            advance_value_position() => {}
            check_position_1() => {}
            skip_past_delimiter() => {}
            finish() => {}
        }}
    }
}

} // verus!
