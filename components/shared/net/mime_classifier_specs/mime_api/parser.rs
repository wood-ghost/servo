//! MIME parser model and specifications.
//!
//! The external FromStr/str::parse contracts are declared in `mime.rs`.
//! Quoted-string collection and its proofs are defined in `http_quoted_string.rs`.
//! The automata and their proofs supply models; their general correspondence
//! with the external parser remains an explicit TODO.
//! Parent-parser execution follows the state-machine transitions, with a proved
//! terminating run and completed-trace witness.

use verus_state_machines_macros::{case_on_next, state_machine};
use vstd::prelude::*;

use super::http_quoted_string::{
    collect_http_quoted_string, collect_http_quoted_string_properties,
};
use super::mime::{MimeView, essence_str_view};
use super::utils::{
    ascii_lowercase, collect_code_points, is_http_quoted_string_token_code_point,
    is_http_token_code_point, is_http_whitespace, remove_http_whitespace,
    remove_trailing_http_whitespace,
};

verus! {

// Currently no input requirements are checked.
// TODO: Add further requirements later.
pub open spec fn mime_parse_requires(input: Seq<char>) -> bool {
    true
}

// Parser-contract structure:
// input -> completed MimeParseAutomaton state -> Done/MimeView or Failure.
//
// The runner below selects states through the generated initialize and next relations.
// mime_parse_final_state_properties proves that it reaches a terminal state along
// a completed trace. No arbitrary terminal state or fixed fuel limit is assumed.
// TODO: Reconcile the remaining WHATWG-shaped transitions (e.g. duplicate-name
// filtering) with mime 0.3.17 before establishing general library correspondence.
#[verifier::opaque]
pub open spec fn mime_parse_final_state(input: Seq<char>) -> MimeParseAutomaton::State {
    // Reuse the DSL's initialization relation as the single source of field values.
    // The generated initializer supplies a witness. mime_parse_initialize_properties
    // proves that every state satisfying the relation gives this same run.
    let initial = choose|state: MimeParseAutomaton::State|
        MimeParseAutomaton::State::initialize(state, input);
    mime_parse_run(initial)
}

// Split the state's collected subtype according to mime 0.3.17's rules:
// https://docs.rs/mime/0.3.17/src/mime/parse.rs.html#137-155
// For example, "svg+xml" becomes ("svg", Some("xml")); "html" has no suffix.
// Only the last non-leading '+' is a separator. A trailing '+' gives Some(empty),
// while a leading '+' alone stays in the subtype. Normalization is done by the parent.
pub open spec fn mime_parse_subtype_and_suffix(
    subtype: Seq<char>,
) -> (Seq<char>, Option<Seq<char>>) {
    match subtype.index_of_last('+') {
        Some(index) if index > 0 => (
            subtype.subrange(0, index),
            Some(subtype.subrange(index + 1, subtype.len() as int)),
        ),
        _ => (subtype, None),
    }
}

// Project a successful state into the shared view. Parameters are already an
// ordered sequence; an empty sequence represents a successful parse with no parameters.
pub open spec fn mime_parse_done_view(final_state: MimeParseAutomaton::State) -> MimeView
    recommends final_state.state == MimeParseState::Done,
{
    let (subtype, suffix) = mime_parse_subtype_and_suffix(final_state.subtype);
    MimeView {
        type_: final_state.type_,
        subtype,
        suffix,
        params: final_state.parameters,
    }
}

// A parameter-free MIME string: nonempty HTTP-token components separated by one
// slash, with no surrounding whitespace. This is a syntactic success criterion,
// not a restriction on the external parsing contract or on the automaton.
pub open spec fn mime_parse_plain_input(input: Seq<char>) -> bool {
    let slash = input.index_of('/');
    &&& input.contains('/')
    &&& 0 < slash < input.len() - 1
    &&& forall|i: int| #![auto] 0 <= i < input.len() && i != slash
        ==> is_http_token_code_point(input[i])
}

pub open spec fn mime_parse_plain_view(input: Seq<char>) -> MimeView
    recommends mime_parse_plain_input(input),
{
    let slash = input.index_of('/');
    let type_ = ascii_lowercase(input.subrange(0, slash));
    let full_subtype = ascii_lowercase(input.subrange(slash + 1, input.len() as int));
    let (subtype, suffix) = mime_parse_subtype_and_suffix(full_subtype);
    MimeView { type_, subtype, suffix, params: Seq::empty() }
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

        // The value-availability transition establishes the collector's entry guard.
        #[invariant]
        pub fn ready_to_parse_value_in_bounds(&self) -> bool {
            self.state == MimeParseState::ReadyToParseParameterValue
                ==> self.position < self.input.len()
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
        fn continue_after_empty_unquoted_value_inductive(pre: Self, post: Self) {}

        #[inductive(store_parameter_if_valid)]
        fn store_parameter_if_valid_inductive(pre: Self, post: Self) {}
    }
}

// Execution of the parent parser using the generated state-machine relations.

// Initialization determines a unique state and establishes the invariant.
// The supplied initial state is also a witness that the choice in final_state exists.
pub proof fn mime_parse_initialize_properties(
    input: Seq<char>,
    initial: MimeParseAutomaton::State,
)
    requires MimeParseAutomaton::State::initialize(initial, input),
    ensures
        initial.invariant(),
        forall|state: MimeParseAutomaton::State|
            #[trigger] MimeParseAutomaton::State::initialize(state, input) ==> state == initial,
        mime_parse_final_state(input) == mime_parse_run(initial),
{
    hide(mime_parse_run);
    reveal(mime_parse_final_state);
}

pub open spec fn mime_parse_terminal(state: MimeParseState) -> bool {
    state == MimeParseState::Done || state == MimeParseState::Failure
}

// Select a successor using the DSL's transition relation. Existence, uniqueness,
// invariant preservation, and progress are proved below for valid nonterminal states.
pub open spec fn mime_parse_step(pre: MimeParseAutomaton::State) -> MimeParseAutomaton::State {
    if mime_parse_terminal(pre.state) {
        pre
    } else {
        choose|post: MimeParseAutomaton::State| MimeParseAutomaton::State::next(pre, post)
    }
}

// At a fixed cursor position, phases descend toward LoopHead or a terminal state.
// LoopHead can restart the parameter phases only by advancing the cursor.
pub open spec fn mime_parse_phase_rank(state: MimeParseState) -> nat {
    match state {
        MimeParseState::Init => 17,
        MimeParseState::HttpWhitespaceRemoved => 16,
        MimeParseState::TypeCollected => 15,
        MimeParseState::AwaitingSubtype => 14,
        MimeParseState::SubtypeCollected => 13,
        MimeParseState::SubtypeValidated => 12,
        MimeParseState::AwaitingParameters => 11,
        MimeParseState::SemicolonSkipped => 10,
        MimeParseState::HttpWhitespaceSkipped => 9,
        MimeParseState::ParameterNameCollected => 8,
        MimeParseState::ParameterNameLowercased => 7,
        MimeParseState::AwaitingParameterValue => 6,
        MimeParseState::ReadyToParseParameterValue => 5,
        MimeParseState::UnquotedParameterValueCollected => 4,
        MimeParseState::UnquotedParameterValueTrimmed | MimeParseState::QuotedParameterValueCollected => 3,
        MimeParseState::LoopHead => 1,
        _ => 0,
    }
}

// Select an enabled generated proof helper. All field updates remain in the DSL.
pub proof fn mime_parse_next_exists(pre: MimeParseAutomaton::State) -> (post: MimeParseAutomaton::State)
    requires pre.invariant(), !mime_parse_terminal(pre.state),
    ensures
        MimeParseAutomaton::State::next(pre, post),
        post.invariant(),
{
    match pre.state {
        MimeParseState::Init => {
            let post = MimeParseAutomaton::take_step::remove_whitespace(pre);
            MimeParseAutomaton::show::remove_whitespace(pre, post);
            post
        },
        MimeParseState::HttpWhitespaceRemoved => {
            let post = MimeParseAutomaton::take_step::collect_type(pre);
            MimeParseAutomaton::show::collect_type(pre, post);
            post
        },
        MimeParseState::TypeCollected => {
            let post = MimeParseAutomaton::take_step::validate_type_and_skip_slash(pre);
            MimeParseAutomaton::show::validate_type_and_skip_slash(pre, post);
            post
        },
        MimeParseState::AwaitingSubtype => {
            let post = MimeParseAutomaton::take_step::collect_subtype(pre);
            MimeParseAutomaton::show::collect_subtype(pre, post);
            post
        },
        MimeParseState::SubtypeCollected => {
            let post = MimeParseAutomaton::take_step::trim_and_validate_subtype(pre);
            MimeParseAutomaton::show::trim_and_validate_subtype(pre, post);
            post
        },
        MimeParseState::SubtypeValidated => {
            let post = MimeParseAutomaton::take_step::create_mime_record(pre);
            MimeParseAutomaton::show::create_mime_record(pre, post);
            post
        },
        MimeParseState::AwaitingParameters => {
            let post = MimeParseAutomaton::take_step::enter_loop(pre);
            MimeParseAutomaton::show::enter_loop(pre, post);
            post
        },
        MimeParseState::LoopHead => {
            let post = MimeParseAutomaton::take_step::check_loop_continuation(pre);
            MimeParseAutomaton::show::check_loop_continuation(pre, post);
            post
        },
        MimeParseState::SemicolonSkipped => {
            let post = MimeParseAutomaton::take_step::skip_http_whitespace(pre);
            MimeParseAutomaton::show::skip_http_whitespace(pre, post);
            post
        },
        MimeParseState::HttpWhitespaceSkipped => {
            let post = MimeParseAutomaton::take_step::collect_parameter_name(pre);
            MimeParseAutomaton::show::collect_parameter_name(pre, post);
            post
        },
        MimeParseState::ParameterNameCollected => {
            let post = MimeParseAutomaton::take_step::lowercase_parameter_name(pre);
            MimeParseAutomaton::show::lowercase_parameter_name(pre, post);
            post
        },
        MimeParseState::ParameterNameLowercased => {
            let post = MimeParseAutomaton::take_step::handle_parameter_name_delimiter(pre);
            MimeParseAutomaton::show::handle_parameter_name_delimiter(pre, post);
            post
        },
        MimeParseState::AwaitingParameterValue => {
            let post = MimeParseAutomaton::take_step::check_parameter_value_available(pre);
            MimeParseAutomaton::show::check_parameter_value_available(pre, post);
            post
        },
        MimeParseState::ReadyToParseParameterValue => {
            let post = MimeParseAutomaton::take_step::collect_parameter_value(pre);
            MimeParseAutomaton::show::collect_parameter_value(pre, post);
            post
        },
        MimeParseState::UnquotedParameterValueCollected => {
            let post = MimeParseAutomaton::take_step::trim_unquoted_parameter_value(pre);
            MimeParseAutomaton::show::trim_unquoted_parameter_value(pre, post);
            post
        },
        MimeParseState::UnquotedParameterValueTrimmed => {
            if pre.parameter_value.len() == 0 {
                let post = MimeParseAutomaton::take_step::continue_after_empty_unquoted_value(pre);
                MimeParseAutomaton::show::continue_after_empty_unquoted_value(pre, post);
                post
            } else {
                let post = MimeParseAutomaton::take_step::store_parameter_if_valid(pre);
                MimeParseAutomaton::show::store_parameter_if_valid(pre, post);
                post
            }
        },
        MimeParseState::QuotedParameterValueCollected => {
            let post = MimeParseAutomaton::take_step::store_parameter_if_valid(pre);
            MimeParseAutomaton::show::store_parameter_if_valid(pre, post);
            post
        },
        _ => pre,
    }
}

// Prove progress from the generated transition relations, not from copied updates.
pub proof fn mime_parse_next_properties(
    pre: MimeParseAutomaton::State,
    post: MimeParseAutomaton::State,
)
    requires pre.invariant(), MimeParseAutomaton::State::next(pre, post),
    ensures
        post.invariant(),
        !mime_parse_terminal(pre.state),
        post.state != MimeParseState::Init,
        pre.state != MimeParseState::Init ==> {
            &&& post.input == pre.input
            &&& pre.position <= post.position
            &&& pre.position == post.position ==>
                mime_parse_phase_rank(post.state) < mime_parse_phase_rank(pre.state)
        },
{
    case_on_next! { pre, post, MimeParseAutomaton => {
        remove_whitespace() => {}
        collect_type() => {
            collect_code_points_position_bounds(pre.input, 0, |c: char| c != '\u{002F}');
        }
        validate_type_and_skip_slash() => {}
        collect_subtype() => {
            collect_code_points_position_bounds(pre.input, pre.position, |c: char| c != '\u{003B}');
        }
        trim_and_validate_subtype() => {}
        create_mime_record() => {}
        enter_loop() => {}
        check_loop_continuation() => {}
        skip_http_whitespace() => {
            collect_code_points_position_bounds(pre.input, pre.position, |c: char| is_http_whitespace(c));
        }
        collect_parameter_name() => {
            collect_code_points_position_bounds(pre.input, pre.position, |c: char| c != '\u{003B}' && c != '\u{003D}');
        }
        lowercase_parameter_name() => {}
        handle_parameter_name_delimiter() => {}
        check_parameter_value_available() => {}
        collect_parameter_value() => {
            if pre.input[pre.position] == '\u{0022}' {
                collect_http_quoted_string_properties(pre.input, pre.position, true);
                if let Ok((_, after_quote)) = collect_http_quoted_string(pre.input, pre.position, true) {
                    collect_code_points_position_bounds(pre.input, after_quote, |c: char| c == '\u{0020}');
                }
            } else {
                collect_code_points_position_bounds(pre.input, pre.position, |c: char| c != '\u{003B}');
            }
        }
        trim_unquoted_parameter_value() => {}
        continue_after_empty_unquoted_value() => {}
        store_parameter_if_valid() => {}
    }}
}

pub proof fn mime_parse_next_is_deterministic(
    pre: MimeParseAutomaton::State,
    left: MimeParseAutomaton::State,
    right: MimeParseAutomaton::State,
)
    requires
        MimeParseAutomaton::State::next(pre, left),
        MimeParseAutomaton::State::next(pre, right),
    ensures left == right,
{
    reveal(MimeParseAutomaton::State::next);
    reveal(MimeParseAutomaton::State::next_by);
}

pub broadcast proof fn mime_parse_step_correct(pre: MimeParseAutomaton::State)
    requires pre.invariant(), !mime_parse_terminal(pre.state),
    ensures
        MimeParseAutomaton::State::next(pre, #[trigger] mime_parse_step(pre)),
        mime_parse_step(pre).invariant(),
        mime_parse_step(pre).state != MimeParseState::Init,
        pre.state != MimeParseState::Init ==> {
            let post = mime_parse_step(pre);
            &&& post.input == pre.input
            &&& pre.position <= post.position
            &&& pre.position == post.position ==>
                mime_parse_phase_rank(post.state) < mime_parse_phase_rank(pre.state)
        },
{
    mime_parse_next_exists(pre);
    let post = mime_parse_step(pre);
    mime_parse_next_properties(pre, post);
}

// The first measure component handles the one-time input trimming. Afterwards,
// input is fixed and either the cursor advances or the phase rank decreases.
pub open spec fn mime_parse_run(pre: MimeParseAutomaton::State) -> MimeParseAutomaton::State
    decreases
        (if pre.state == MimeParseState::Init { 1nat } else { 0nat }),
        pre.input.len() - pre.position,
        mime_parse_phase_rank(pre.state),
{
    decreases_by(mime_parse_run_decreases);
    // Totalize on malformed internal states. The initial state satisfies the
    // invariant and step_correct preserves it, so valid runs never take this fallback.
    if !pre.invariant() || mime_parse_terminal(pre.state) {
        pre
    } else {
        mime_parse_run(mime_parse_step(pre))
    }
}

#[verifier::decreases_by]
proof fn mime_parse_run_decreases(pre: MimeParseAutomaton::State) {
    if pre.invariant() && !mime_parse_terminal(pre.state) {
        mime_parse_step_correct(pre);
    }
}

// Expose one step to clients without unfolding the recursive runner many times.
// On a terminal state, step is the identity and the final result stays the same.
pub proof fn mime_parse_run_one_step(pre: MimeParseAutomaton::State) -> (post: MimeParseAutomaton::State)
    requires pre.invariant(),
    ensures
        post == mime_parse_step(pre),
        post.invariant(),
        !mime_parse_terminal(pre.state) ==> MimeParseAutomaton::State::next(pre, post),
        mime_parse_run(pre) == mime_parse_run(post),
        mime_parse_terminal(post.state) ==> mime_parse_run(pre) == post,
{
    if !mime_parse_terminal(pre.state) {
        mime_parse_step_correct(pre);
    }
    mime_parse_step(pre)
}

pub open spec fn mime_parse_steps(trace: Seq<MimeParseAutomaton::State>) -> bool {
    forall|i: int| 0 <= i && i + 1 < trace.len() ==>
        #[trigger] MimeParseAutomaton::State::next(trace[i], trace[i + 1])
}

pub proof fn mime_parse_step_at(trace: Seq<MimeParseAutomaton::State>, i: int)
    requires mime_parse_steps(trace), 0 <= i, i + 1 < trace.len(),
    ensures MimeParseAutomaton::State::next(trace[i], trace[i + 1]),
{
    reveal(mime_parse_steps);
}

pub open spec fn mime_parse_complete_trace(
    input: Seq<char>, trace: Seq<MimeParseAutomaton::State>,
) -> bool {
    &&& trace.len() > 0
    &&& MimeParseAutomaton::State::initialize(trace.first(), input)
    &&& mime_parse_steps(trace)
    &&& mime_parse_terminal(trace.last().state)
}

// Construct a real execution witness for the runner, including failure paths.
pub proof fn mime_parse_run_trace(pre: MimeParseAutomaton::State) -> (trace: Seq<MimeParseAutomaton::State>)
    requires pre.invariant(),
    ensures
        trace.len() > 0,
        trace.first() == pre,
        mime_parse_steps(trace),
        trace.last() == mime_parse_run(pre),
        trace.last().invariant(),
        mime_parse_terminal(trace.last().state),
    decreases
        (if pre.state == MimeParseState::Init { 1nat } else { 0nat }),
        pre.input.len() - pre.position,
        mime_parse_phase_rank(pre.state),
{
    if mime_parse_terminal(pre.state) {
        seq![pre]
    } else {
        mime_parse_step_correct(pre);
        let post = mime_parse_step(pre);
        let rest = mime_parse_run_trace(post);
        let trace = seq![pre] + rest;
        assert forall|i: int| 0 <= i && i + 1 < trace.len() implies
            #[trigger] MimeParseAutomaton::State::next(trace[i], trace[i + 1]) by {
            if i > 0 {
                mime_parse_step_at(rest, i - 1);
            }
        }
        trace
    }
}

pub proof fn mime_parse_final_state_properties(input: Seq<char>)
    ensures
        mime_parse_final_state(input).invariant(),
        mime_parse_terminal(mime_parse_final_state(input).state),
        exists|trace: Seq<MimeParseAutomaton::State>|
            #[trigger] mime_parse_complete_trace(input, trace)
                && trace.last() == mime_parse_final_state(input),
{
    let initial = MimeParseAutomaton::take_step::initialize(input);
    mime_parse_initialize_properties(input, initial);
    let trace = mime_parse_run_trace(initial);
    // Expose the constructed trace as the existential postcondition's witness.
    assert(mime_parse_complete_trace(input, trace));
}

// Prove collection of any range whose characters meet the condition, stopping
// at the first nonmatching character or EOF. No literal-dependent fuel is needed.
pub proof fn collect_code_points_range(
    input: Seq<char>,
    start: int,
    end: int,
    condition: spec_fn(char) -> bool,
)
    requires
        0 <= start <= end <= input.len(),
        forall|i: int| start <= i < end ==> #[trigger] condition(input[i]),
        end < input.len() ==> !condition(input[end]),
    ensures
        collect_code_points(input, start, condition) == (input.subrange(start, end), end),
    decreases end - start,
{
    if start < end {
        collect_code_points_range(input, start + 1, end, condition);
    }
}

// The parsing contract introduces final_state(input), triggering this theorem
// for any input satisfying the syntax predicate, regardless of its length.
pub broadcast proof fn lemma_mime_parse_plain(input: Seq<char>)
    requires mime_parse_plain_input(input),
    ensures
        #![trigger mime_parse_final_state(input)]
        mime_parse_final_state(input).state == MimeParseState::Done,
        mime_parse_final_state(input).type_ == ascii_lowercase(input.subrange(0, input.index_of('/'))),
        mime_parse_final_state(input).subtype
            == ascii_lowercase(input.subrange(input.index_of('/') + 1, input.len() as int)),
        mime_parse_final_state(input).parameters == Seq::<(Seq<char>, Seq<char>)>::empty(),
        mime_parse_done_view(mime_parse_final_state(input)) == mime_parse_plain_view(input),
{
    hide(mime_parse_run);
    hide(collect_code_points);
    reveal(MimeParseAutomaton::State::next);
    reveal(MimeParseAutomaton::State::next_by);
    let slash = input.index_of('/');
    collect_code_points_range(input, 0, slash, |c: char| c != '/');
    collect_code_points_range(input, slash + 1, input.len() as int, |c: char| c != ';');
    // assert(remove_http_whitespace(input) == input);
    let initial = MimeParseAutomaton::take_step::initialize(input);
    mime_parse_initialize_properties(input, initial);
    let s = mime_parse_run_one_step(initial);
    let s = mime_parse_run_one_step(s);
    let s = mime_parse_run_one_step(s);
    let s = mime_parse_run_one_step(s);
    let s = mime_parse_run_one_step(s);
    let s = mime_parse_run_one_step(s);
    mime_parse_run_one_step(s);
}

// Expose the existing last-plus theorem when a parsed subtype is projected.
pub broadcast proof fn lemma_mime_parse_suffix_index(subtype: Seq<char>)
    ensures
        #![trigger mime_parse_subtype_and_suffix(subtype)]
        match subtype.index_of_last('+') {
            Some(index) => {
                &&& 0 <= index < subtype.len()
                &&& subtype[index] == '+'
                &&& forall|j: int| index < j < subtype.len() ==> subtype[j] != '+'
            },
            None => !subtype.contains('+'),
        },
{
    subtype.index_of_last_ensures('+');
}

// A client's expected normalized type supplies a separator position. Check the
// input's separator, prefix contents, and token conditions, then apply the
// general parser theorem without unfolding the runner at the call site.
pub proof fn lemma_mime_parse_plain_type_hint(input: Seq<char>, type_hint: &str)
    requires
        0 < type_hint@.len() < input.len() - 1,
        input[type_hint@.len() as int] == '/',
        ascii_lowercase(input.subrange(0, type_hint@.len() as int)) =~= type_hint@,
        forall|i: int| #![auto] 0 <= i < input.len() && i != type_hint@.len()
            ==> is_http_token_code_point(input[i]),
    ensures
        mime_parse_plain_input(input),
        input.index_of('/') == type_hint@.len(),
        mime_parse_final_state(input).state == MimeParseState::Done,
        mime_parse_final_state(input).type_ == type_hint@,
        mime_parse_done_view(mime_parse_final_state(input)) == mime_parse_plain_view(input),
{
    lemma_mime_parse_plain(input);
}

// Use the supported sequence-index trigger. Separator/suffix hints introduce
// the relevant indexes; this bridge does not invent an index witness. vstd
// already enables the equivalent axiom, so the group below does not repeat it.
pub broadcast proof fn lemma_mime_parse_strlit_index(s: &str, i: int)
    ensures
        strslice_get_char(s, i) == #[trigger] s@[i],
{
    vstd::string::axiom_str_literal_get_char(s, i);
}

// Relate normalized components to literal views by their length and contents.
pub broadcast proof fn lemma_mime_parse_lowercase_str(input: Seq<char>, expected: &str)
    requires
        input.len() == expected@.len(),
        forall|i: int| 0 <= i < input.len()
            ==> ascii_lowercase(input)[i] == #[trigger] expected@[i],
    ensures
        #![trigger ascii_lowercase(input), expected@]
        ascii_lowercase(input) == expected@,
{
}

// Keep essence equality instantiation tied to a parsed plain-MIME view rather
// than every MIME essence comparison in the classifier.
pub broadcast proof fn lemma_mime_parse_plain_essence_str(input: Seq<char>, expected: &str)
    requires
        mime_parse_plain_input(input),
        essence_str_view(&mime_parse_plain_view(input)).len() == expected@.len(),
        forall|i: int| 0 <= i < expected@.len() ==>
            essence_str_view(&mime_parse_plain_view(input))[i] == #[trigger] expected@[i],
    ensures
        #![trigger mime_parse_plain_view(input), expected@]
        essence_str_view(&mime_parse_plain_view(input)) == expected@,
{
}

// Normalization cannot introduce a suffix separator into a plus-free input.
pub broadcast proof fn lemma_mime_parse_plain_no_plus(input: Seq<char>)
    requires mime_parse_plain_input(input), !input.contains('+'),
    ensures
        #![trigger mime_parse_plain_view(input)]
        mime_parse_plain_view(input).suffix == None,
        mime_parse_plain_view(input).subtype
            == ascii_lowercase(input.subrange(input.index_of('/') + 1, input.len() as int)),
{
    let slash = input.index_of('/');
    let subtype = ascii_lowercase(input.subrange(slash + 1, input.len() as int));
    subtype.index_of_last_ensures('+');
}

// A plus in the input's subtype remains at the corresponding position after
// normalization. Supply that witness to the last-plus projection.
pub proof fn lemma_mime_parse_plain_plus(input: Seq<char>, i: int)
    requires
        mime_parse_plain_input(input),
        input.index_of('/') < i < input.len(),
        input[i] == '+',
    ensures
        ascii_lowercase(input.subrange(input.index_of('/') + 1, input.len() as int))[
            i - input.index_of('/') - 1
        ] == '+',
        ascii_lowercase(input.subrange(input.index_of('/') + 1, input.len() as int))
            .contains('+'),
        input.index_of('/') + 1 < i ==> mime_parse_plain_view(input).suffix.is_some(),
{
    let slash = input.index_of('/');
    let subtype = ascii_lowercase(input.subrange(slash + 1, input.len() as int));
    subtype.index_of_last_ensures('+');
}

// The expected base subtype supplies a candidate non-leading plus position.
pub proof fn lemma_mime_parse_plain_suffix_hint(input: Seq<char>, subtype_hint: &str)
    requires
        mime_parse_plain_input(input),
        subtype_hint@.len() > 0,
        input.index_of('/') + 1 + subtype_hint@.len() < input.len(),
        input[input.index_of('/') + 1 + subtype_hint@.len()] == '+',
    ensures
        mime_parse_plain_view(input).suffix.is_some(),
        ascii_lowercase(input.subrange(input.index_of('/') + 1, input.len() as int))[
            subtype_hint@.len() as int
        ] == '+',
{
    lemma_mime_parse_plain_plus(input, input.index_of('/') + 1 + subtype_hint@.len());
}

pub broadcast group mime_parse_lemmas {
    lemma_mime_parse_plain,
    lemma_mime_parse_suffix_index,
    lemma_mime_parse_lowercase_str,
    lemma_mime_parse_plain_essence_str,
    lemma_mime_parse_plain_no_plus,
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
