//! MIME parser model and specifications.

use vstd::prelude::*;
use verus_state_machines_macros::{case_on_next, state_machine};

verus! {

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

// https://fetch.spec.whatwg.org/#collect-an-http-quoted-string
// TODO: Define using the quoted-string automaton.
// Models extract-value = true for MIME parsing.
// Returns (extracted value, updated position); currently a dummy implementation.
// Intended entry: 0 <= position < input.len() and input[position] == '\u{0022}'.
pub open spec fn collect_http_quoted_string(
    input: Seq<char>,
    position: int,
) -> (Seq<char>, int) {
    // TODO: Replace this stub with the Fetch quoted-string automaton.
    // This only skips the opening quote and returns an empty value; it does not parse quotes.
    if 0 <= position < input.len() {
        (Seq::empty(), position + 1)
    } else {
        (Seq::empty(), position)
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
                    let (parameter_value, after_quote) =
                        collect_http_quoted_string(pre.input, pre.position);
                    // 11.8.2 Collect a sequence of code points that are not U+003B (;) from input, given position
                    let (_, position) = collect_code_points(
                        pre.input,
                        after_quote,
                        |c: char| c != '\u{003B}',
                    );
                    update parameter_value = parameter_value;
                    update position = position;
                    update state = MimeParseState::QuotedParameterValueCollected;
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
                let (_, after_quote) = collect_http_quoted_string(pre.input, pre.position);
                collect_code_points_position_bounds(
                    pre.input, after_quote, |c: char| c != '\u{003B}',
                );
            }
            // TODO: Recheck this proof against the real quoted-string automaton, not the stub.
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
