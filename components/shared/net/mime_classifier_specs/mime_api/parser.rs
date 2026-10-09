//! MIME parser model and specifications.
//!
//! The FromStr/str::parse declarations below are trusted external contracts.
//! The automata and their proofs supply models; their general correspondence
//! with the external parser remains an explicit TODO.

use ::mime::{self, Mime};
use core::str::FromStr;
use verus_state_machines_macros::{case_on_next, state_machine};
use vstd::prelude::*;

use super::constants::*;
use super::mime::{MimeView, view};

pub use legacy::{
    collect_a_sequence_of_code_points,
    collect_a_sequence_of_code_points_helper,
    is_ascii_alphanumeric_,
    parse_mime_type_spec,
    solely_contains_http_token_code_points,
};

verus! {

// TODO: Define the input requirements under which the Rust MIME parsing contract applies.
pub uninterp spec fn mime_parse_requires(input: Seq<char>) -> bool;

#[verifier::external_trait_specification]
#[verifier::external_trait_extension(FromStrSpec via FromStrSpecImpl)]
pub trait ExFromStr: Sized {
    type ExternalTraitSpecificationFor: FromStr;
    type Err;

    spec fn from_str_requires(i: Seq<char>) -> bool;
    spec fn from_str_ensures(i: Seq<char>, r: Result<Self, Self::Err>) -> bool;

    fn from_str(s: &str) -> (r: Result<Self, Self::Err>)
        requires
            Self::from_str_requires(s@),
        ensures
            Self::from_str_ensures(s@, r)
    ;
}

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExFromStrError(mime::FromStrError);

impl FromStrSpecImpl for Mime {
    open spec fn from_str_requires(input: Seq<char>) -> bool {
        mime_parse_requires(input)
    }

    open spec fn from_str_ensures(
        input: Seq<char>,
        result: Result<Mime, <Mime as FromStr>::Err>,
    ) -> bool {
        // TODO: Relate general parsing results to the automaton under mime_parse_requires.
        // Successful parsing does not imply essence == input: case and parameters can differ.
        // hardcode
        // &&& result is Ok
        &&& (input == "image/x-icon"@) ==> result is Ok
        &&& (input == "image/webp"@) ==> result is Ok
        &&& (input == "video/webm"@) ==> result is Ok
        &&& (input == "audio/basic"@) ==> result is Ok
        &&& (input == "audio/aiff"@) ==> result is Ok
        &&& (input == "audio/mpeg"@) ==> result is Ok
        &&& (input == "application/ogg"@) ==> result is Ok
        &&& (input == "audio/midi"@) ==> result is Ok
        &&& (input == "video/avi"@) ==> result is Ok
        &&& (input == "audio/wave"@) ==> result is Ok

        &&& (input == "application/postscript"@) ==> result is Ok
        &&& (input == "application/x-gzip"@) ==> result is Ok
        &&& (input == "application/zip"@) ==> result is Ok
        &&& (input == "application/x-rar-compressed"@) ==> result is Ok
        &&& (input == "application/font-woff"@) ==> result is Ok
        &&& (input == "application/font-sfnt"@) ==> result is Ok
        &&& (input == "application/vnd.ms-fontobject"@) ==> result is Ok
        &&& (input == "video/mp4"@) ==> result is Ok
        &&& (input == "text/vtt"@) ==> result is Ok
        &&& (input == "text/cache-manifest"@) ==> result is Ok

        &&& (input == "application/ecmascript"@) ==> result is Ok
        &&& (input == "application/javascript"@) ==> result is Ok
        &&& (input == "application/x-ecmascript"@) ==> result is Ok
        &&& (input == "application/x-javascript"@) ==> result is Ok
        &&& (input == "text/ecmascript"@) ==> result is Ok
        &&& (input == "text/javascript"@) ==> result is Ok
        &&& (input == "text/javascript1.0"@) ==> result is Ok
        &&& (input == "text/javascript1.1"@) ==> result is Ok
        &&& (input == "text/javascript1.2"@) ==> result is Ok
        &&& (input == "text/javascript1.3"@) ==> result is Ok
        &&& (input == "text/javascript1.4"@) ==> result is Ok
        &&& (input == "text/javascript1.5"@) ==> result is Ok
        &&& (input == "text/jscript"@) ==> result is Ok
        &&& (input == "text/livescript"@) ==> result is Ok
        &&& (input == "text/x-ecmascript"@) ==> result is Ok
        &&& (input == "text/x-javascript"@) ==> result is Ok

        &&& (input == "image/x-icon"@) ==> view(&result->Ok_0) == image_x_icon_identity()
        &&& (input == "image/webp"@) ==> view(&result->Ok_0) == image_webp_identity()
        &&& (input == "video/webm"@) ==> view(&result->Ok_0) == video_webm_identity()
        &&& (input == "audio/basic"@) ==> view(&result->Ok_0) == audio_basic_identity()
        &&& (input == "audio/aiff"@) ==> view(&result->Ok_0) == audio_aiff_identity()
        &&& (input == "audio/mpeg"@) ==> view(&result->Ok_0) == audio_mpeg_identity()
        &&& (input == "application/ogg"@) ==> view(&result->Ok_0) == application_ogg_identity()
        &&& (input == "audio/midi"@) ==> view(&result->Ok_0) == audio_midi_identity()
        &&& (input == "video/avi"@) ==> view(&result->Ok_0) == video_avi_identity()
        &&& (input == "audio/wave"@) ==> view(&result->Ok_0) == audio_wave_identity()

        &&& (input == "application/postscript"@) ==> view(&result->Ok_0) == application_postscript_identity()
        &&& (input == "application/x-gzip"@) ==> view(&result->Ok_0) == application_x_gzip_identity()
        &&& (input == "application/zip"@) ==> view(&result->Ok_0) == application_zip_identity()
        &&& (input == "application/x-rar-compressed"@) ==> view(&result->Ok_0) == application_x_rar_compressed_identity()
        &&& (input == "application/font-woff"@) ==> view(&result->Ok_0) == application_font_woff_identity()
        &&& (input == "application/font-sfnt"@) ==> view(&result->Ok_0) == application_font_sfnt_identity()
        &&& (input == "application/vnd.ms-fontobject"@) ==> view(&result->Ok_0) == application_vnd_ms_fontobject_identity()
        &&& (input == "video/mp4"@) ==> view(&result->Ok_0) == video_mp4_identity()
        &&& (input == "text/vtt"@) ==> view(&result->Ok_0) == text_vtt_identity()
        &&& (input == "text/cache-manifest"@) ==> view(&result->Ok_0) == text_cache_manifest_identity()

        &&& (input == "application/ecmascript"@) ==> view(&result->Ok_0) == application_ecmascript_identity()
        &&& (input == "application/javascript"@) ==> view(&result->Ok_0) == application_javascript_identity()
        &&& (input == "application/x-ecmascript"@) ==> view(&result->Ok_0) == application_x_ecmascript_identity()
        &&& (input == "application/x-javascript"@) ==> view(&result->Ok_0) == application_x_javascript_identity()
        &&& (input == "text/ecmascript"@) ==> view(&result->Ok_0) == text_ecmascript_identity()
        &&& (input == "text/javascript"@) ==> view(&result->Ok_0) == text_javascript_identity()
        &&& (input == "text/javascript1.0"@) ==> view(&result->Ok_0) == text_javascript1_0_identity()
        &&& (input == "text/javascript1.1"@) ==> view(&result->Ok_0) == text_javascript1_1_identity()
        &&& (input == "text/javascript1.2"@) ==> view(&result->Ok_0) == text_javascript1_2_identity()
        &&& (input == "text/javascript1.3"@) ==> view(&result->Ok_0) == text_javascript1_3_identity()
        &&& (input == "text/javascript1.4"@) ==> view(&result->Ok_0) == text_javascript1_4_identity()
        &&& (input == "text/javascript1.5"@) ==> view(&result->Ok_0) == text_javascript1_5_identity()
        &&& (input == "text/jscript"@) ==> view(&result->Ok_0) == text_jscript_identity()
        &&& (input == "text/livescript"@) ==> view(&result->Ok_0) == text_livescript_identity()
        &&& (input == "text/x-ecmascript"@) ==> view(&result->Ok_0) == text_x_ecmascript_identity()
        &&& (input == "text/x-javascript"@) ==> view(&result->Ok_0) == text_x_javascript_identity()
    }
}

// --------------------
// str::parse for Mime
// --------------------
pub assume_specification<F: FromStr>[ str::parse::<F> ](s: &str) -> (
    result: Result<F, <F as FromStr>::Err>
)
    requires
        call_requires(<F as FromStr>::from_str, (s,)),
    ensures
        call_ensures( <F as FromStr>::from_str, (s,), result),
;

// The precondition is inherited from FromStrSpec::from_str_requires in parser.rs.
pub assume_specification[ <Mime as FromStr>::from_str ](input: &str) -> (
    result: Result<Mime, <Mime as FromStr>::Err>
)
    ensures
        true // TODO:
;

// https://fetch.spec.whatwg.org/#http-whitespace
pub open spec fn is_http_whitespace(c: char) -> bool {
    c == '\u{0009}'    // TAB
    || c == '\u{000A}' // LF
    || c == '\u{000D}' // CR
    || c == '\u{0020}' // SPACE
}

// https://mimesniff.spec.whatwg.org/#http-token-code-point
pub open spec fn is_http_token_code_point(c: char) -> bool {
    ('\u{0030}' <= c && c <= '\u{0039}') // 0-9
    || ('\u{0041}' <= c && c <= '\u{005A}') // A-Z
    || ('\u{0061}' <= c && c <= '\u{007A}') // a-z
    || c == '\u{0021}' // !
    || c == '\u{0023}' // #
    || c == '\u{0024}' // $
    || c == '\u{0025}' // %
    || c == '\u{0026}' // &
    || c == '\u{0027}' // '
    || c == '\u{002A}' // *
    || c == '\u{002B}' // +
    || c == '\u{002D}' // -
    || c == '\u{002E}' // .
    || c == '\u{005E}' // ^
    || c == '\u{005F}' // _
    || c == '\u{0060}' // `
    || c == '\u{007C}' // |
    || c == '\u{007E}' // ~
}

// https://mimesniff.spec.whatwg.org/#http-quoted-string-token-code-point
pub open spec fn is_http_quoted_string_token_code_point(c: char) -> bool {
    c == '\u{0009}' // TAB
    || ('\u{0020}' <= c && c <= '\u{007E}')
    || ('\u{0080}' <= c && c <= '\u{00FF}')
}

pub open spec fn remove_leading_http_whitespace(input: Seq<char>) -> Seq<char>
    decreases input.len(),
{
    if input.len() == 0 {
        input
    } else if is_http_whitespace(input[0]) {
        remove_leading_http_whitespace(input.subrange(1, input.len() as int))
    } else {
        input
    }
}

pub open spec fn remove_trailing_http_whitespace(input: Seq<char>) -> Seq<char>
    decreases input.len(),
{
    if input.len() == 0 {
        input
    } else if is_http_whitespace(input[input.len() - 1]) {
        remove_trailing_http_whitespace(input.subrange(0, input.len() - 1))
    } else {
        input
    }
}

// Remove leading and trailing HTTP whitespace, preserving interior characters.
pub open spec fn remove_http_whitespace(input: Seq<char>) -> Seq<char> {
    remove_trailing_http_whitespace(remove_leading_http_whitespace(input))
}

// https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points
pub open spec fn collect_code_points(
    input: Seq<char>,
    position: int,
    condition: spec_fn(char) -> bool,
) -> (Seq<char>, int)
    decreases input.len() - position,
{
    if position < 0 || position >= input.len() {
        // 1. Let result be the empty string. 
        (Seq::empty(), position)
    // 2. While position doesn’t point past the end of input and the code point at position within input meets the condition condition: 
    } else if condition(input[position]) {
        // 2.2 Advance position by 1. 
        let (result, new_position) = collect_code_points(input, position + 1, condition);
        // 2.1 Append that code point to the end of result. 
        (seq![input[position]] + result, new_position)
    } else {
        // 1. Let result be the empty string. 
        (Seq::empty(), position)
    }
}

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

// mime 0.3.17 checks each quoted byte with `c > 31 && c != 127`.
// For a Rust string, every UTF-8 byte of a non-ASCII scalar satisfies that rule.
// Thus only ASCII controls and DEL are invalid in this character-indexed model.
pub open spec fn is_rust_quoted_value_char(c: char) -> bool {
    c >= '\u{0020}' && c != '\u{007F}'
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
        fn finish_inductive(pre: Self, post: Self) {
            assert(pre.input[pre.position - 1] == '\u{0022}');
        }
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
            if i == 0 {
                assert(trace[1] == post);
            } else {
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
    if trace.len() == 1 {
        assert(trace.last() == pre);
    } else {
        http_quoted_string_step_at(trace, 0);
        http_quoted_string_next_is_deterministic(pre, trace[1]);
        http_quoted_string_step_correct(pre);
        let rest = trace.subrange(1, trace.len() as int);
        assert forall|i: int| 0 <= i && i + 1 < rest.len() implies
            #[trigger] HttpQuotedStringAutomaton::State::next(rest[i], rest[i + 1]) by {
            http_quoted_string_step_at(trace, i + 1);
        }
        http_quoted_string_complete_trace_unique(trace[1], rest);
        http_quoted_string_trace_from_properties(trace[1]);
        assert(rest.last() == trace.last());
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
    assert(initial.invariant());
    http_quoted_string_trace_from_properties(initial);
    let witness = http_quoted_string_trace_from(initial);
    assert(http_quoted_string_complete_trace(input, position, witness));
    let trace = choose|trace: Seq<HttpQuotedStringAutomaton::State>|
        http_quoted_string_complete_trace(input, position, trace);
    assert(http_quoted_string_complete_trace(input, position, trace));
    assert(trace.first() == initial);
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


// https://infra.spec.whatwg.org/#ascii-lowercase
pub open spec fn ascii_lowercase(input: Seq<char>) -> Seq<char> {
    input.map(|i: int, c: char|
        if '\u{0041}' <= c && c <= '\u{005A}' {
            ((c as u32 + 0x20) as u32) as char
        } else {
            c
        }
    )
}

pub ghost enum MimeParseState {
    Init,
    HttpWhitespaceRemoved,
    TypeCollected,
    AwaitingSubtype,
    SubtypeCollected,
    SubtypeValidated,
    AwaitingParameters,
    LoopHead,
    SemicolonSkipped,
    HttpWhitespaceSkipped,
    ParameterNameCollected,
    ParameterNameLowercased,
    AwaitingParameterValue,
    ReadyToParseParameterValue,
    QuotedParameterValueCollected,
    UnquotedParameterValueCollected,
    UnquotedParameterValueTrimmed,
    Done,
    Failure,
}

// https://mimesniff.spec.whatwg.org/#parsing-a-mime-type
state_machine! {
    MimeParseAutomaton {
        fields {
            pub input: Seq<char>,
            pub type_: Seq<char>,
            pub subtype: Seq<char>,
            // Ordered accumulator; parameter insertion will enforce unique names.
            pub parameters: Seq<(Seq<char>, Seq<char>)>,
            pub parameter_name: Seq<char>,
            // The phase distinguishes an uncollected value from a collected empty string.
            pub parameter_value: Seq<char>,
            pub position: int,
            pub state: MimeParseState,
        }

        init! {
            initialize(input: Seq<char>) {
                init input = input;
                init type_ = Seq::empty();
                init subtype = Seq::empty();
                init parameters = Seq::empty();
                init parameter_name = Seq::empty();
                init parameter_value = Seq::empty();
                init position = 0;
                init state = MimeParseState::Init;
            }
        }

        // 1. Remove any leading and trailing HTTP whitespace from input. 
        transition! {
            remove_whitespace() {
                require(pre.state == MimeParseState::Init);

                update input = remove_http_whitespace(pre.input);
                update state = MimeParseState::HttpWhitespaceRemoved;
            }
        }

        // 2. Let position be a position variable for input, initially pointing at the start of input.
        // 3. Let type be the result of collecting a sequence of code points that are not U+002F (/) from input, given position.
        transition! {
            collect_type() {
                require(pre.state == MimeParseState::HttpWhitespaceRemoved);

                let (type_, position) = collect_code_points(
                    pre.input, 
                    0, 
                    |c: char| c != '\u{002F}'
                );

                update type_ = type_;
                update position = position;
                update state = MimeParseState::TypeCollected;
            }
        }

        transition! {
            validate_type_and_skip_slash() {
                require(pre.state == MimeParseState::TypeCollected);

                let valid_type = pre.type_.len() > 0 
                    && (forall|i: int| #![auto] 0 <= i < pre.type_.len()
                        ==> is_http_token_code_point(pre.type_[i]));

                // 4. If type is the empty string or does not solely contain HTTP token code points, then return failure.
                if !valid_type {
                    update state = MimeParseState::Failure;
                // 5. If position is past the end of input, then return failure. 
                } else if pre.position >= pre.input.len() {
                    update state = MimeParseState::Failure;
                } else {
                    // 6. Advance position by 1. (This skips past U+002F (/).)
                    update position = pre.position + 1;
                    update state = MimeParseState::AwaitingSubtype;
                }
            }
        }

        // 7. Let subtype be the result of collecting a sequence of code points that are not U+003B (;) from input, given position.
        transition! {
            collect_subtype() {
                require(pre.state == MimeParseState::AwaitingSubtype);

                let (subtype, position) = collect_code_points(
                    pre.input,
                    pre.position,
                    |c: char| c != '\u{003B}',
                );

                update subtype = subtype;
                update position = position;
                update state = MimeParseState::SubtypeCollected;
            }
        }

        transition! {
            trim_and_validate_subtype() {
                require(pre.state == MimeParseState::SubtypeCollected);

                // 8. Remove any trailing HTTP whitespace from subtype.
                let subtype = remove_trailing_http_whitespace(pre.subtype);
                let valid_subtype = subtype.len() > 0
                    && (forall|i: int| #![auto] 0 <= i < subtype.len()
                        ==> is_http_token_code_point(subtype[i]));

                update subtype = subtype;

                // 9. If subtype is the empty string or does not solely contain HTTP token code points, then return failure.
                if !valid_subtype {
                    update state = MimeParseState::Failure;
                } else {
                    update state = MimeParseState::SubtypeValidated;
                }
            }
        }

        // 10. Let mimeType be a new MIME type record whose type is type, in ASCII lowercase, and subtype is subtype, in ASCII lowercase. 
        transition! {
            create_mime_record() {
                require(pre.state == MimeParseState::SubtypeValidated);

                update type_ = ascii_lowercase(pre.type_);
                update subtype = ascii_lowercase(pre.subtype);
                update state = MimeParseState::AwaitingParameters;
            }
        }

        // The parameter loop of https://mimesniff.spec.whatwg.org/#parsing-a-mime-type
        transition! {
            enter_loop() {
                require(pre.state == MimeParseState::AwaitingParameters);

                // 11. While position is not past the end of input: 
                if pre.position < pre.input.len() {
                    update state = MimeParseState::LoopHead;
                } else {
                    update state = MimeParseState::Done;
                }
            }
        }

        transition! {
            check_loop_continuation() {
                require(pre.state == MimeParseState::LoopHead);

                // 11. While position is not past the end of input: 
                if pre.position < pre.input.len() {
                    update state = MimeParseState::SemicolonSkipped;
                    // 11.1 Advance position by 1. (This skips past U+003B (;).) 
                    update position = pre.position + 1;
                } else {
                    update state = MimeParseState::Done;
                }
            }
        }

        // 11.2 Collect a sequence of code points that are HTTP whitespace from input given position. 
        transition! {
            skip_http_whitespace() {
                require(pre.state == MimeParseState::SemicolonSkipped);

                let (_, position) = collect_code_points(
                    pre.input,
                    pre.position,
                    |c: char| is_http_whitespace(c),
                );

                update position = position;
                update state = MimeParseState::HttpWhitespaceSkipped;
            }
        }

        // 11.3 Let parameterName be the result of collecting a sequence of code points that are not U+003B (;) or U+003D (=) from input, given position. 
        transition! {
            collect_parameter_name() {
                require(pre.state == MimeParseState::HttpWhitespaceSkipped);

                let (parameter_name, position) = collect_code_points(
                    pre.input,
                    pre.position,
                    |c: char| c != '\u{003B}' && c != '\u{003D}',
                );

                update parameter_name = parameter_name;
                update position = position;
                update state = MimeParseState::ParameterNameCollected;
            }
        }

        // 11.4 Set parameterName to parameterName, in ASCII lowercase. 
        transition! {
            lowercase_parameter_name() {
                require(pre.state == MimeParseState::ParameterNameCollected);

                update parameter_name = ascii_lowercase(pre.parameter_name);
                update state = MimeParseState::ParameterNameLowercased;
            }
        }

        transition! {
            handle_parameter_name_delimiter() {
                require(pre.state == MimeParseState::ParameterNameLowercased);

                // 11.5 If position is not past the end of input, then:
                if pre.position < pre.input.len() {
                    // 11.5.1 If the code point at position within input is U+003B (;), then continue.
                    if pre.input[pre.position] == '\u{003B}' {
                        // Preserve the delimiter for the next loop iteration (e.g. foo/bar;foo;).
                        // Rust mime 0.3.17 comparison (source inspection): text/html;foo;
                        // is rejected with InvalidToken at the second ';'. WHATWG ignores
                        // the incomplete parameter and continues, ultimately returning no parameters.
                        update state = MimeParseState::LoopHead;
                    } else {
                        // 11.5.2 Advance position by 1. (This skips past U+003D (=).) 
                        update position = pre.position + 1;
                        update state = MimeParseState::AwaitingParameterValue;
                    } 
                } else {
                    // 11.6 If position is past the end of input, then break. 
                    // WHATWG ignores the incomplete parameter and returns those already stored.
                    // Rust mime 0.3.17 comparison (source inspection): text/html;foo
                    // is rejected with MissingEqual. WHATWG accepts text/html with no parameters.
                    update state = MimeParseState::Done;
                }
            }
        }

        transition! {
            check_parameter_value_available() {
                require(pre.state == MimeParseState::AwaitingParameterValue);

                // 11.6 If position is past the end of input, then break. 
                if pre.position >= pre.input.len() {
                    // Rust mime 0.3.17 comparison (source inspection): text/html;foo=
                    // is accepted with foo -> "". WHATWG ignores foo and returns no parameters.
                    update state = MimeParseState::Done;
                } else {
                    // 11.7. Let parameterValue be null; the phase uses an empty placeholder.
                    update parameter_value = Seq::empty();
                    update state = MimeParseState::ReadyToParseParameterValue;
                }
            }
        }

        transition! {
            collect_parameter_value() {
                require(pre.state == MimeParseState::ReadyToParseParameterValue);
                require(0 <= pre.position < pre.input.len());

                // 11.8 If the code point at position within input is U+0022 ("), then: 
                if pre.input[pre.position] == '\u{0022}' {
                    // Rust mime 0.3.17 comparison (source inspection): text/html;foo=""
                    // is rejected with MissingQuote for this exact input. WHATWG accepts
                    // the empty quoted value and ultimately stores foo -> "".
                    // 11.8.1 Set parameterValue to the result of collecting an HTTP quoted string from input, given position and true. 
                    match collect_http_quoted_string(pre.input, pre.position, true) {
                        Ok((parameter_value, after_quote)) => {
                            // 11.8.2 Collect a sequence of code points that are not U+003B (;) from input, given position
                            // Rust deviation: only SPACE is accepted after a quoted value,
                            // before a semicolon or EOF. Other trailing text is InvalidToken.
                            let (_, position) = collect_code_points(
                                pre.input, after_quote, |c: char| c == '\u{0020}',
                            );
                            if position < pre.input.len() && pre.input[position] != '\u{003B}' {
                                update state = MimeParseState::Failure;
                            } else {
                                update parameter_value = parameter_value;
                                update position = position;
                                update state = MimeParseState::QuotedParameterValueCollected;
                            }
                        },
                        Err(_) => {
                            update state = MimeParseState::Failure;
                        },
                    }
                } else {
                    // 11.9. Otherwise: 
                    // 11.9.1. Set parameterValue to the result of collecting a sequence of code points that are not U+003B (;) from input, given position.  
                    let (unquoted_value, unquoted_position) = collect_code_points(
                        pre.input,
                        pre.position,
                        |c: char| c != '\u{003B}',
                    );
                    update parameter_value = unquoted_value;
                    update position = unquoted_position;
                    update state = MimeParseState::UnquotedParameterValueCollected;
                }
            }
        }

        // 11.9.2. Remove any trailing HTTP whitespace from parameterValue.
        transition! {
            trim_unquoted_parameter_value() {
                require(pre.state == MimeParseState::UnquotedParameterValueCollected);

                update parameter_value = remove_trailing_http_whitespace(pre.parameter_value);
                update state = MimeParseState::UnquotedParameterValueTrimmed;
            }
        }

        // 11.9.3. If parameterValue is the empty string, then continue.
        transition! {
            continue_after_empty_unquoted_value() {
                require(pre.state == MimeParseState::UnquotedParameterValueTrimmed);
                require(pre.parameter_value.len() == 0);

                // Rust mime 0.3.17 comparison (source inspection): text/html;foo=;bar=baz
                // is rejected with InvalidToken at the ';' after '='. WHATWG ignores foo
                // and continues, ultimately storing bar -> "baz".
                update state = MimeParseState::LoopHead;
            }
        }

        // 11.10. Store the parameter only if all four conditions hold.
        transition! {
            store_parameter_if_valid() {
                require(
                    pre.state == MimeParseState::QuotedParameterValueCollected
                    || (
                        pre.state == MimeParseState::UnquotedParameterValueTrimmed
                        && pre.parameter_value.len() > 0
                    )
                );

                // parameterName is not the empty string 
                // parameterName solely contains HTTP token code points
                let name_is_valid = pre.parameter_name.len() > 0
                    && (forall|i: int| #![auto] 0 <= i < pre.parameter_name.len()
                        ==> is_http_token_code_point(pre.parameter_name[i]));

                // parameterValue solely contains HTTP quoted-string token code points
                let value_is_valid = forall|i: int| #![auto]
                    0 <= i < pre.parameter_value.len()
                        ==> is_http_quoted_string_token_code_point(pre.parameter_value[i]);

                // mimeType’s parameters[parameterName] does not exist
                let name_is_absent = forall|i: int| #![auto]
                    0 <= i < pre.parameters.len()
                        ==> pre.parameters[i].0 != pre.parameter_name;

                if name_is_valid && value_is_valid && name_is_absent {
                    // then set mimeType’s parameters[parameterName] to parameterValue. 
                    update parameters = pre.parameters.push(
                        (pre.parameter_name, pre.parameter_value),
                    );
                }

                // Invalid parameters and duplicate names are ignored.
                update state = MimeParseState::LoopHead;
            }
        }

        #[invariant]
        pub fn before_type_collection(&self) -> bool {
            (
                self.state == MimeParseState::Init
                || self.state == MimeParseState::HttpWhitespaceRemoved
            ) ==> (
                self.type_ == Seq::<char>::empty()
                && self.subtype == Seq::<char>::empty()
                && self.parameters == Seq::<(Seq<char>, Seq<char>)>::empty()
                && self.position == 0
            )
        }

        #[invariant]
        pub fn before_subtype_collection(&self) -> bool {
            (
                self.state == MimeParseState::TypeCollected
                || self.state == MimeParseState::AwaitingSubtype
            ) ==> (
                self.subtype == Seq::<char>::empty()
                && self.parameters == Seq::<(Seq<char>, Seq<char>)>::empty()
            )
        }

        #[invariant]
        pub fn parameters_empty_after_subtype_collection(&self) -> bool {
            (
                self.state == MimeParseState::SubtypeCollected
                || self.state == MimeParseState::SubtypeValidated
                || self.state == MimeParseState::AwaitingParameters
            ) ==> self.parameters == Seq::<(Seq<char>, Seq<char>)>::empty()
        }

        #[invariant]
        pub fn validated_subtype_is_nonempty_token(&self) -> bool {
            self.state == MimeParseState::SubtypeValidated ==> (
                self.subtype.len() > 0
                && (forall|i: int| #![auto] 0 <= i < self.subtype.len()
                    ==> is_http_token_code_point(self.subtype[i]))
            )
        }

        // Use an empty placeholder in the existing pre-value phases.
        #[invariant]
        pub fn parameter_value_empty_before_collection(&self) -> bool {
            (
                self.state == MimeParseState::Init
                || self.state == MimeParseState::ReadyToParseParameterValue
            )
                ==> self.parameter_value == Seq::<char>::empty()
        }

        #[invariant]
        pub fn position_in_bounds(&self) -> bool {
            0 <= self.position <= self.input.len()
        }

        #[invariant]
        pub fn done_is_at_end(&self) -> bool {
            self.state == MimeParseState::Done ==> self.position == self.input.len()
        }

        #[inductive(initialize)]
        fn initialize_inductive(post: Self, input: Seq<char>) {}

        #[inductive(remove_whitespace)]
        fn remove_whitespace_inductive(pre: Self, post: Self) {}

        #[inductive(collect_type)]
        fn collect_type_inductive(pre: Self, post: Self) {
            collect_code_points_position_bounds(pre.input, 0, |c: char| c != '\u{002F}');
        }

        #[inductive(validate_type_and_skip_slash)]
        fn validate_type_and_skip_slash_inductive(pre: Self, post: Self) {}

        #[inductive(collect_subtype)]
        fn collect_subtype_inductive(pre: Self, post: Self) {
            collect_code_points_position_bounds(
                pre.input, pre.position, |c: char| c != '\u{003B}',
            );
        }

        #[inductive(trim_and_validate_subtype)]
        fn trim_and_validate_subtype_inductive(pre: Self, post: Self) {}

        #[inductive(create_mime_record)]
        fn create_mime_record_inductive(pre: Self, post: Self) {}

        #[inductive(enter_loop)]
        fn enter_loop_inductive(pre: Self, post: Self) {}

        #[inductive(check_loop_continuation)]
        fn check_loop_continuation_inductive(pre: Self, post: Self) {}

        #[inductive(skip_http_whitespace)]
        fn skip_http_whitespace_inductive(pre: Self, post: Self) {
            collect_code_points_position_bounds(
                pre.input, pre.position, |c: char| is_http_whitespace(c),
            );
        }

        #[inductive(collect_parameter_name)]
        fn collect_parameter_name_inductive(pre: Self, post: Self) {
            collect_code_points_position_bounds(
                pre.input, pre.position, |c: char| c != '\u{003B}' && c != '\u{003D}',
            );
        }

        #[inductive(lowercase_parameter_name)]
        fn lowercase_parameter_name_inductive(pre: Self, post: Self) {}

        #[inductive(handle_parameter_name_delimiter)]
        fn handle_parameter_name_delimiter_inductive(pre: Self, post: Self) {}

        #[inductive(check_parameter_value_available)]
        fn check_parameter_value_available_inductive(pre: Self, post: Self) {}

        #[inductive(collect_parameter_value)]
        fn collect_parameter_value_inductive(pre: Self, post: Self) {
            if pre.input[pre.position] != '\u{0022}' {
                collect_code_points_position_bounds(
                    pre.input, pre.position, |c: char| c != '\u{003B}',
                );
            } else {
                collect_http_quoted_string_properties(pre.input, pre.position, true);
                if let Ok((_, after_quote)) = collect_http_quoted_string(pre.input, pre.position, true) {
                    collect_code_points_position_bounds(
                        pre.input, after_quote, |c: char| c == '\u{0020}',
                    );
                }
            }
            // TODO: Recheck this proof against the real quoted-string automaton, not the stub.
            // The proof above now uses the completed-trace theorem for the Rust model.
        }

        #[inductive(trim_unquoted_parameter_value)]
        fn trim_unquoted_parameter_value_inductive(pre: Self, post: Self) {}

        #[inductive(continue_after_empty_unquoted_value)]
        fn continue_after_empty_unquoted_value_inductive(pre: Self, post: Self) {
            assert(pre.parameter_value =~= Seq::<char>::empty());
        }

        #[inductive(store_parameter_if_valid)]
        fn store_parameter_if_valid_inductive(pre: Self, post: Self) {}
    }
}

// Collection advances the cursor without passing the end of input.
pub proof fn collect_code_points_position_bounds(
    input: Seq<char>,
    position: int,
    condition: spec_fn(char) -> bool,
)
    requires 0 <= position <= input.len(),
    ensures position <= collect_code_points(input, position, condition).1 <= input.len(),
    decreases input.len() - position,
{
    if position < input.len() && condition(input[position]) {
        collect_code_points_position_bounds(input, position + 1, condition);
    }
}

// Collection stops at the end of input or at a character that does not meet the condition.
pub proof fn collect_code_points_stopping_condition(
    input: Seq<char>,
    position: int,
    condition: spec_fn(char) -> bool,
)
    requires 0 <= position <= input.len(),
    ensures ({
        let end = collect_code_points(input, position, condition).1;
        end == input.len() || (0 <= end < input.len() && !condition(input[end]))
    }),
    decreases input.len() - position,
{
    if position < input.len() && condition(input[position]) {
        collect_code_points_stopping_condition(input, position + 1, condition);
    }
}

// Interpret a completed accumulator as the parent's optional parameter map.
// None represents WHATWG's empty map, including when all parameters were ignored.
pub open spec fn parameter_result(
    final_state: MimeParseAutomaton::State,
) -> Option<Seq<(Seq<char>, Seq<char>)>>
    recommends final_state.state == MimeParseState::Done,
{
    if final_state.parameters.len() == 0 {
        None
    } else {
        Some(final_state.parameters)
    }
}

// Once trimming is complete, every step preserves input and cannot return to Init.
pub proof fn input_preserved_after_trimming(
    pre: MimeParseAutomaton::State,
    post: MimeParseAutomaton::State,
)
    requires
        MimeParseAutomaton::State::next(pre, post),
        pre.state != MimeParseState::Init,
    ensures post.input == pre.input, post.state != MimeParseState::Init,
{
    case_on_next! { pre, post, MimeParseAutomaton => {
        remove_whitespace() => {}
        collect_type() => {}
        validate_type_and_skip_slash() => {}
        collect_subtype() => {}
        trim_and_validate_subtype() => {}
        create_mime_record() => {}
        enter_loop() => {}
        check_loop_continuation() => {}
        skip_http_whitespace() => {}
        collect_parameter_name() => {}
        lowercase_parameter_name() => {}
        handle_parameter_name_delimiter() => {}
        check_parameter_value_available() => {}
        collect_parameter_value() => {}
        trim_unquoted_parameter_value() => {}
        continue_after_empty_unquoted_value() => {}
        store_parameter_if_valid() => {}
    }}
}

pub proof fn done_is_terminal(
    pre: MimeParseAutomaton::State,
    post: MimeParseAutomaton::State,
)
    requires pre.state == MimeParseState::Done,
    ensures !MimeParseAutomaton::State::next(pre, post),
{
    if MimeParseAutomaton::State::next(pre, post) {
        case_on_next! { pre, post, MimeParseAutomaton => {
            remove_whitespace() => {}
            collect_type() => {}
            validate_type_and_skip_slash() => {}
            collect_subtype() => {}
            trim_and_validate_subtype() => {}
            create_mime_record() => {}
            enter_loop() => {}
            check_loop_continuation() => {}
            skip_http_whitespace() => {}
            collect_parameter_name() => {}
            lowercase_parameter_name() => {}
            handle_parameter_name_delimiter() => {}
            check_parameter_value_available() => {}
            collect_parameter_value() => {}
            trim_unquoted_parameter_value() => {}
            continue_after_empty_unquoted_value() => {}
            store_parameter_if_valid() => {}
        }}
    }
}

// Failure has no outgoing transitions.
pub proof fn failure_is_terminal(
    pre: MimeParseAutomaton::State,
    post: MimeParseAutomaton::State,
)
    requires pre.state == MimeParseState::Failure,
    ensures !MimeParseAutomaton::State::next(pre, post),
{
    if MimeParseAutomaton::State::next(pre, post) {
        case_on_next! { pre, post, MimeParseAutomaton => {
            remove_whitespace() => {}
            collect_type() => {}
            validate_type_and_skip_slash() => {}
            collect_subtype() => {}
            trim_and_validate_subtype() => {}
            create_mime_record() => {}
            enter_loop() => {}
            check_loop_continuation() => {}
            skip_http_whitespace() => {}
            collect_parameter_name() => {}
            lowercase_parameter_name() => {}
            handle_parameter_name_delimiter() => {}
            check_parameter_value_available() => {}
            collect_parameter_value() => {}
            trim_unquoted_parameter_value() => {}
            continue_after_empty_unquoted_value() => {}
            store_parameter_if_valid() => {}
        }}
    }
}

} // verus!

// Earlier, unfinished direct parser model, kept separately from the automaton's helpers.
// TODO: Reconcile this draft with MimeParseAutomaton before using it in a general contract.
pub mod legacy {
use vstd::prelude::*;
use super::MimeView;

verus! {

// https://mimesniff.spec.whatwg.org/#parsing-a-mime-type
// To parse a MIME type, given a string input, run these steps: 
pub open spec fn parse_mime_type_spec(input: Seq<char>) -> Option<MimeView> {
    // 1. Remove any leading and trailing HTTP whitespace from input. 
    let input = remove_http_whitespace(input);
    // 2. Let position be a position variable for input, initially pointing at the start of input.
    // 3. Let type be the result of collecting a sequence of code points that are not U+002F (/) from input, given position. 
    let (type_, position) = collect_a_sequence_of_code_points(input, |c: char| c != '/', 0);
    // 4. If type is the empty string or does not solely contain HTTP token code points, then return failure. 
    if type_.len() == 0 || !solely_contains_http_token_code_points(type_) {
        None
    // 5. If position is past the end of input, then return failure. 
    } else if position >= input.len() {
        None
    } else {
        // 6. Advance position by 1. (This skips past U+002F (/).) 
        let position = position + 1;
        // 7. Let subtype be the result of collecting a sequence of code points that are not U+003B (;) from input, given position. 
        let (subtype, position) = collect_a_sequence_of_code_points(input, |c: char| c != ';', position);
        // 8. Remove any trailing HTTP whitespace from subtype. 
        let subtype = remove_http_whitespace(subtype);
        // 9. If subtype is the empty string or does not solely contain HTTP token code points, then return failure. 
        if subtype.len() == 0 || !solely_contains_http_token_code_points(subtype) {
            None
        } else {
            // Let mimeType be a new MIME type record whose type is type, in ASCII lowercase, and subtype is subtype, in ASCII lowercase. 
            let mime_type = MimeView {
                type_: type_,
                subtype: subtype,
                suffix: None, // TODO:
                params: Map::empty(),
            };
            Some(mime_type) //TODO:
        }
    }

}


pub open spec fn solely_contains_http_token_code_points(input: Seq<char>) -> bool {
    forall |i: int| 0 <= i < input.len() ==> #[trigger] is_http_token_code_point(input[i])
}
// https://mimesniff.spec.whatwg.org/#http-token-code-point
// An HTTP token code point is U+0021 (!), U+0023 (#), U+0024 ($), U+0025 (%), U+0026 (&), U+0027 ('), U+002A (*), U+002B (+), 
// U+002D (-), U+002E (.), U+005E (^), U+005F (_), U+0060 (`), U+007C (|), U+007E (~), or an ASCII alphanumeric.
pub open spec fn is_http_token_code_point(c: char) -> bool {
    ||| (c == '\u{0021}') // !
    ||| (c == '\u{0023}') // #
    ||| (c == '\u{0024}') // $
    ||| (c == '\u{0025}') // %
    ||| (c == '\u{0026}') // &
    ||| (c == '\u{0027}') // '
    ||| (c == '\u{002A}') // *
    ||| (c == '\u{002B}') // +
    ||| (c == '\u{002D}') // -
    ||| (c == '\u{002E}') // .
    ||| (c == '\u{005E}') // ^
    ||| (c == '\u{005F}') // _
    ||| (c == '\u{0060}') // `
    ||| (c == '\u{007C}') // |
    ||| (c == '\u{007E}') // ~
    ||| (is_ascii_alphanumeric_(c))
}
pub open spec fn is_ascii_alphanumeric_(c: char) -> bool {
    ('0' <= c && c <= '9')
        || ('A' <= c && c <= 'Z')
        || ('a' <= c && c <= 'z')
}

// https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points
// To collect a sequence of code points meeting a condition condition from a string input, 
// given a position variable position tracking the position of the calling algorithm within input:
pub open spec fn collect_a_sequence_of_code_points_helper(
    input: Seq<char>,
    condition: spec_fn(char) -> bool,
    result: Seq<char>,
    position: int,
) -> (Seq<char>, int)
    recommends
        0 <= position <= input.len(),
    decreases input.len() - position,
{
    // 2. While position doesn't point past the end of input and the code point at position within input meets the condition condition: 
    if position < input.len() && condition(input[position]) {
        collect_a_sequence_of_code_points_helper(
            input,
            condition,
            // 2.1 Append that code point to the end of result. 
            result.push(input[position]),
            // 2.2 Advance position by 1.
            position + 1,
        )
    } else {
        // 3. Return result. 
        (result, position)
    }
}
pub open spec fn collect_a_sequence_of_code_points(input: Seq<char>, condition: spec_fn(char) -> bool, position: int) -> (Seq<char>, int)
    recommends 0 <= position <= input.len(),
{
    collect_a_sequence_of_code_points_helper(
        input,
        condition,
        // 1. Let result be the empty string. 
        Seq::empty(),
        position,
    )
}

// An HTTP tab or space is U+0009 TAB or U+0020 SPACE.
// HTTP whitespace is U+000A LF, U+000D CR, or an HTTP tab or space. 
pub open spec fn is_http_whitespace(c: char) -> bool {
    ||| c == '\u{000A}' // LF
    ||| c == '\u{000D}' // CR
    ||| c == '\u{0009}' // TAB
    ||| c == '\u{0020}' // SPACE
}
pub open spec fn remove_http_whitespace(input: Seq<char>) -> Seq<char>
    decreases input.len(),
{
    if input.len() == 0 {
        input
    } else if is_http_whitespace(input[0]) {
        // remove prefix
        remove_http_whitespace(
            input.subrange(1, input.len() as int),
        )
    } else if is_http_whitespace(input[input.len() - 1]) {
        // remove suffix
        remove_http_whitespace(
            input.subrange(0, input.len() as int - 1),
        )
    } else {
        input
    }
}

} // verus!
}
