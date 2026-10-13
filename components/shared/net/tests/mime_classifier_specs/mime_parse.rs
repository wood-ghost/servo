/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Verification regressions for the external MIME parsing contract.
//! Expected values were checked by ordinary Rust execution of mime 0.3.17.
//! The parsing specifications are unfinished: the MIME cases require Ok(()).
//! Bind executable accessor results before using them in proof-mode assertions.
//! Compare string contents through their views, not logical reference equality.
//! Failure notes describe the 2026-10-08 run in README.md: "support" failures
//! stop before proof checking; "proof" failures are unproved obligations.
//! "Fidelity" notes record known Rust/model differences; these still require Ok(()).

#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] harness_verifies_assertion verus_code! {
        use vstd::prelude::*;

        fn test() {
            assert(1 + 1 == 2);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] harness_detects_assertion_failure verus_code! {
        use vstd::prelude::*;

        fn test() {
            assert(false);
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] harness_distinguishes_type_error verus_code! {
        use vstd::prelude::*;

        fn test() {
            let _: u8 = false;
        }
    } => Err(err) => assert_rust_error(err, "E0308")
}

// Pure-model cases for suffix splitting, independent of the unfinished parser run.
// Use character sequences directly, matching the helper's input type.
// Cases checked against ordinary Rust mime 0.3.17 on 2026-10-12.
test_verify_one_file! {
    #[test] model_subtype_and_suffix_boundaries verus_code! {
        use vstd::prelude::*;
        use crate::mime_api::mime_parse_subtype_and_suffix;

        proof fn test() {
            let empty = Seq::<char>::empty();
            empty.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(empty) == (empty, None));

            let plain = seq!['h', 't', 'm', 'l'];
            plain.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(plain) == (plain, None));

            let wildcard = seq!['*'];
            wildcard.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(wildcard) == (wildcard, None));

            let structured = seq!['s', 'v', 'g', '+', 'x', 'm', 'l'];
            structured.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(structured)
                =~~= (seq!['s', 'v', 'g'], Some(seq!['x', 'm', 'l'])));

            let multiple = seq!['a', '+', 'b', '+', 'c'];
            multiple.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(multiple)
                =~~= (seq!['a', '+', 'b'], Some(seq!['c'])));

            let leading = seq!['+', 'j', 's', 'o', 'n'];
            leading.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(leading) == (leading, None));

            let plus = seq!['+'];
            plus.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(plus) == (plus, None));

            let leading_and_later = seq!['+', 'j', 's', 'o', 'n', '+', 'x', 'm', 'l'];
            leading_and_later.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(leading_and_later)
                =~~= (seq!['+', 'j', 's', 'o', 'n'], Some(seq!['x', 'm', 'l'])));

            let trailing = seq!['h', 't', 'm', 'l', '+'];
            trailing.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(trailing)
                =~~= (seq!['h', 't', 'm', 'l'], Some(seq![])));

            let consecutive = seq!['a', '+', '+'];
            consecutive.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(consecutive)
                =~~= (seq!['a', '+'], Some(seq![])));

            let two_pluses = seq!['+', '+'];
            two_pluses.index_of_last_ensures('+');
            assert(mime_parse_subtype_and_suffix(two_pluses)
                =~~= (seq!['+'], Some(seq![])));
        }
    } => Ok(())
}

// Execute the parent model on known character sequences. The executable client
// below constrains only its input contents, not the result of parsing.
test_verify_one_file! {
    #[test] model_mime_parse_final_state_plain verus_code! {
        use vstd::prelude::*;
        use crate::mime_api::{
            MimeView, MimeParseAutomaton, MimeParseState, ascii_lowercase, collect_code_points,
            mime_parse_done_view, mime_parse_final_state, mime_parse_initialize_properties, mime_parse_run,
            mime_parse_run_one_step,
        };

        proof fn plain_model()
            ensures
                mime_parse_final_state(seq!['t', 'e', 'x', 't', '/', 'h', 't', 'm', 'l']).state
                    == MimeParseState::Done,
                mime_parse_done_view(mime_parse_final_state(
                    seq!['t', 'e', 'x', 't', '/', 'h', 't', 'm', 'l'],
                )) == (MimeView {
                    type_: seq!['t', 'e', 'x', 't'],
                    subtype: seq!['h', 't', 'm', 'l'],
                    suffix: None,
                    params: seq![],
                }),
        {
            hide(mime_parse_run);
            hide(collect_code_points);
            reveal(MimeParseAutomaton::State::next);
            reveal(MimeParseAutomaton::State::next_by);
            assert(collect_code_points(
                seq!['t', 'e', 'x', 't', '/', 'h', 't', 'm', 'l'],
                0, |c: char| c != '/',
            ) =~~= (seq!['t', 'e', 'x', 't'], 4int)) by (compute_only);
            assert(collect_code_points(
                seq!['t', 'e', 'x', 't', '/', 'h', 't', 'm', 'l'],
                5, |c: char| c != ';',
            ) =~~= (seq!['h', 't', 'm', 'l'], 9int)) by (compute_only);
            assert(ascii_lowercase(seq!['t', 'e', 'x', 't']) =~= seq!['t', 'e', 'x', 't']) by (compute);
            assert(ascii_lowercase(seq!['h', 't', 'm', 'l']) =~= seq!['h', 't', 'm', 'l']) by (compute);
            let input = seq!['t', 'e', 'x', 't', '/', 'h', 't', 'm', 'l'];
            let initial = MimeParseAutomaton::take_step::initialize(input);
            mime_parse_initialize_properties(input, initial);
            let s = mime_parse_run_one_step(initial);
            let s = mime_parse_run_one_step(s);
            assert(s.type_ =~= seq!['t', 'e', 'x', 't']);
            assert(s.position == 4);
            let s = mime_parse_run_one_step(s);
            let s = mime_parse_run_one_step(s);
            assert(s.subtype =~= seq!['h', 't', 'm', 'l']);
            assert(s.position == 9);
            let s = mime_parse_run_one_step(s);
            let s = mime_parse_run_one_step(s);
            let result = mime_parse_run_one_step(s);
            assert(result.state == MimeParseState::Done);
            assert(result.type_ =~= seq!['t', 'e', 'x', 't']);
            assert(result.subtype =~= seq!['h', 't', 'm', 'l']);
            assert(result.parameters =~= seq![]);
            result.subtype.index_of_last_ensures('+');
        }

        fn test(input: &str)
            requires input@ == seq!['t', 'e', 'x', 't', '/', 'h', 't', 'm', 'l'],
        {
            proof { plain_model(); }
            let parsed = input.parse::<mime::Mime>().unwrap();
            let type_ = parsed.type_().as_str();
            let subtype = parsed.subtype().as_str();
            let suffix = parsed.suffix();
            assert(type_@ == seq!['t', 'e', 'x', 't']);
            assert(subtype@ == seq!['h', 't', 'm', 'l']);
            assert(suffix.is_none());
            assert(crate::mime_api::view(&parsed).params =~= seq![]);
        }
    } => Ok(())
}

// The lowercase and uppercase SVG headers agree between the current automaton
// and Rust mime 0.3.17. This isolates suffix splitting and normalization from
// the intentionally failing charset-value regression below.
test_verify_one_file! {
    #[test] model_mime_parse_final_state_svg verus_code! {
        use vstd::prelude::*;
        use crate::mime_api::{
            MimeView, MimeParseAutomaton, MimeParseState, ascii_lowercase,
            collect_code_points, mime_parse_done_view, mime_parse_final_state,
            mime_parse_initialize_properties, mime_parse_run, mime_parse_run_one_step,
        };

        spec fn svg_input(uppercase: bool) -> Seq<char> {
            if uppercase {
                seq!['I', 'M', 'A', 'G', 'E', '/', 'S', 'V', 'G', '+', 'X', 'M', 'L']
            } else {
                seq!['i', 'm', 'a', 'g', 'e', '/', 's', 'v', 'g', '+', 'x', 'm', 'l']
            }
        }

        spec fn expected_svg() -> MimeView {
            MimeView {
                type_: seq!['i', 'm', 'a', 'g', 'e'],
                subtype: seq!['s', 'v', 'g'],
                suffix: Some(seq!['x', 'm', 'l']),
                params: seq![],
            }
        }

        proof fn svg_model(input: Seq<char>)
            requires input == svg_input(false) || input == svg_input(true),
            ensures
                mime_parse_final_state(input).state == MimeParseState::Done,
                mime_parse_done_view(mime_parse_final_state(input)) == expected_svg(),
        {
            hide(mime_parse_run);
            hide(collect_code_points);
            reveal(MimeParseAutomaton::State::next);
            reveal(MimeParseAutomaton::State::next_by);
            assert(collect_code_points(svg_input(false), 0, |c: char| c != '/')
                =~~= (seq!['i', 'm', 'a', 'g', 'e'], 5int)) by (compute_only);
            assert(collect_code_points(svg_input(true), 0, |c: char| c != '/')
                =~~= (seq!['I', 'M', 'A', 'G', 'E'], 5int)) by (compute_only);
            assert(collect_code_points(svg_input(false), 6, |c: char| c != ';')
                =~~= (seq!['s', 'v', 'g', '+', 'x', 'm', 'l'], 13int)) by (compute_only);
            assert(collect_code_points(svg_input(true), 6, |c: char| c != ';')
                =~~= (seq!['S', 'V', 'G', '+', 'X', 'M', 'L'], 13int)) by (compute_only);
            assert(ascii_lowercase(seq!['i', 'm', 'a', 'g', 'e'])
                =~= seq!['i', 'm', 'a', 'g', 'e']) by (compute);
            assert(ascii_lowercase(seq!['I', 'M', 'A', 'G', 'E'])
                =~= seq!['i', 'm', 'a', 'g', 'e']) by (compute);
            assert(ascii_lowercase(seq!['s', 'v', 'g', '+', 'x', 'm', 'l'])
                =~= seq!['s', 'v', 'g', '+', 'x', 'm', 'l']) by (compute);
            assert(ascii_lowercase(seq!['S', 'V', 'G', '+', 'X', 'M', 'L'])
                =~= seq!['s', 'v', 'g', '+', 'x', 'm', 'l']) by (compute);
            let initial = MimeParseAutomaton::take_step::initialize(input);
            mime_parse_initialize_properties(input, initial);
            let s = mime_parse_run_one_step(initial);
            let s = mime_parse_run_one_step(s);
            assert(s.position == 5);
            assert(s.type_ == seq!['i', 'm', 'a', 'g', 'e']
                || s.type_ == seq!['I', 'M', 'A', 'G', 'E']);
            let s = mime_parse_run_one_step(s);
            let s = mime_parse_run_one_step(s);
            assert(s.position == 13);
            assert(s.subtype == seq!['s', 'v', 'g', '+', 'x', 'm', 'l']
                || s.subtype == seq!['S', 'V', 'G', '+', 'X', 'M', 'L']);
            let s = mime_parse_run_one_step(s);
            let s = mime_parse_run_one_step(s);
            let result = mime_parse_run_one_step(s);
            assert(result.state == MimeParseState::Done);
            assert(result.type_ =~= seq!['i', 'm', 'a', 'g', 'e']);
            assert(result.subtype =~= seq!['s', 'v', 'g', '+', 'x', 'm', 'l']);
            assert(result.parameters =~= seq![]);
            result.subtype.index_of_last_ensures('+');
            assert(mime_parse_done_view(result).subtype =~= seq!['s', 'v', 'g']);
            assert(mime_parse_done_view(result).suffix =~~= Some(seq!['x', 'm', 'l']));
        }

        fn test(input: &str)
            requires input@ == svg_input(false) || input@ == svg_input(true),
        {
            proof { svg_model(input@); }
            let parsed = input.parse::<mime::Mime>().unwrap();
            let type_ = parsed.type_().as_str();
            let subtype = parsed.subtype().as_str();
            let suffix = parsed.suffix().unwrap().as_str();
            assert(type_@ == seq!['i', 'm', 'a', 'g', 'e']);
            assert(subtype@ == seq!['s', 'v', 'g']);
            assert(suffix@ == seq!['x', 'm', 'l']);
            assert(crate::mime_api::view(&parsed).params =~= seq![]);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] model_mime_parse_final_state_failures verus_code! {
        use vstd::prelude::*;
        use crate::mime_api::{
            MimeParseAutomaton, MimeParseState, collect_code_points, mime_parse_final_state,
            mime_parse_initialize_properties, mime_parse_run, mime_parse_run_one_step,
        };

        proof fn check_failure(input: Seq<char>)
            requires input == seq![] || input == seq!['t', 'e', 'x', 't']
                || input == seq!['/', 'h', 't', 'm', 'l'],
            ensures mime_parse_final_state(input).state == MimeParseState::Failure,
        {
            hide(mime_parse_run);
            reveal_with_fuel(collect_code_points, 12);
            reveal(MimeParseAutomaton::State::next);
            reveal(MimeParseAutomaton::State::next_by);
            let initial = MimeParseAutomaton::take_step::initialize(input);
            mime_parse_initialize_properties(input, initial);
            let s = mime_parse_run_one_step(initial);
            let s = mime_parse_run_one_step(s);
            let result = mime_parse_run_one_step(s);
            assert(result.state == MimeParseState::Failure);
        }

        proof fn test() {
            check_failure(seq![]);
            check_failure(seq!['t', 'e', 'x', 't']);
            check_failure(seq!['/', 'h', 't', 'm', 'l']);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] model_mime_parse_final_state_parameters verus_code! {
        use vstd::prelude::*;
        broadcast use crate::mime_api::mime_parse_step_correct;
        use crate::mime_api::{
            MimeParseAutomaton, MimeParseState, ascii_lowercase, collect_code_points,
            is_http_whitespace, mime_parse_final_state,
            mime_parse_initialize_properties, mime_parse_run, mime_parse_run_one_step, mime_parse_step,
            mime_parse_terminal,
        };

        spec fn sample_input() -> Seq<char> {
            seq!['t', '/', 's', ';', 'a', '=', 'b', ';', 'c', '=', 'd']
        }

        proof fn four_steps(pre: MimeParseAutomaton::State) -> (post: MimeParseAutomaton::State)
            requires pre.invariant(),
            ensures
                post == mime_parse_step(mime_parse_step(mime_parse_step(mime_parse_step(pre)))),
                post.invariant(),
                mime_parse_run(pre) == mime_parse_run(post),
                mime_parse_terminal(post.state) ==> mime_parse_run(pre) == post,
        {
            hide(mime_parse_run);
            let s = mime_parse_run_one_step(pre);
            let s = mime_parse_run_one_step(s);
            let s = mime_parse_run_one_step(s);
            mime_parse_run_one_step(s)
        }

        proof fn test() {
            hide(mime_parse_run);
            hide(collect_code_points);
            hide(ascii_lowercase);
            reveal(MimeParseAutomaton::State::next);
            reveal(MimeParseAutomaton::State::next_by);
            assert(collect_code_points(sample_input(), 0, |c: char| c != '/')
                =~~= (seq!['t'], 1int)) by (compute_only);
            assert(collect_code_points(sample_input(), 2, |c: char| c != ';')
                =~~= (seq!['s'], 3int)) by (compute_only);
            assert(collect_code_points(sample_input(), 4, |c: char| is_http_whitespace(c))
                =~~= (seq![], 4int)) by (compute_only);
            assert(collect_code_points(sample_input(), 8, |c: char| is_http_whitespace(c))
                =~~= (seq![], 8int)) by (compute_only);
            assert(collect_code_points(sample_input(), 4, |c: char| c != ';' && c != '=')
                =~~= (seq!['a'], 5int)) by (compute_only);
            assert(collect_code_points(sample_input(), 8, |c: char| c != ';' && c != '=')
                =~~= (seq!['c'], 9int)) by (compute_only);
            assert(collect_code_points(sample_input(), 6, |c: char| c != ';')
                =~~= (seq!['b'], 7int)) by (compute_only);
            assert(collect_code_points(sample_input(), 10, |c: char| c != ';')
                =~~= (seq!['d'], 11int)) by (compute_only);
            assert(ascii_lowercase(seq!['t']) =~= seq!['t']) by (compute);
            assert(ascii_lowercase(seq!['s']) =~= seq!['s']) by (compute);
            assert(ascii_lowercase(seq!['a']) =~= seq!['a']) by (compute);
            assert(ascii_lowercase(seq!['c']) =~= seq!['c']) by (compute);
            let input = sample_input();
            let initial = MimeParseAutomaton::take_step::initialize(input);
            mime_parse_initialize_properties(input, initial);
            let s = four_steps(initial);
            assert(s.state == MimeParseState::SubtypeCollected);
            assert(s.position == 3);
            assert(s.subtype =~= seq!['s']);
            let s = four_steps(s);
            assert(s.state == MimeParseState::SemicolonSkipped);
            assert(s.position == 4);
            let s = four_steps(s);
            assert(s.state == MimeParseState::AwaitingParameterValue);
            assert(s.position == 6);
            let s = four_steps(s);
            assert(s.state == MimeParseState::LoopHead);
            assert(s.parameters =~~= seq![(seq!['a'], seq!['b'])]);
            let s = four_steps(s);
            assert(s.state == MimeParseState::ParameterNameLowercased);
            assert(s.position == 9);
            let s = four_steps(s);
            assert(s.state == MimeParseState::UnquotedParameterValueTrimmed);
            assert(s.position == 11);
            assert(s.parameter_name =~= seq!['c']);
            assert(s.parameter_value =~= seq!['d']);
            assert(s.parameters =~~= seq![(seq!['a'], seq!['b'])]);
            assert(s.parameters[0].0 != s.parameter_name) by {
                assert(s.parameters[0].0[0] == 'a');
                assert(s.parameter_name[0] == 'c');
            }
            let result = four_steps(s);
            assert(result.state == MimeParseState::Done);
            assert(result.position == input.len());
            assert(result.type_ =~= seq!['t']);
            assert(result.subtype =~= seq!['s']);
            assert(result.parameters =~~= seq![(seq!['a'], seq!['b']), (seq!['c'], seq!['d'])]);
            assert(mime_parse_final_state(input) == result);
        }
    } => Ok(())
}

// FIXME: (proof): `mime_parse_requires`, the Ok guarantee for unwrap, and the
// general component relation are unfinished; the component assertions fail.
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

// FIXME: (fidelity, 2026-10-13): Rust lowercases charset values to "utf-8";
// the current automaton stores the original "UTF-8" value. Keep the Rust result
// and Ok(()) expectation so this remains a failing regression until corrected.
test_verify_one_file! {
    #[test] parse_normalized_suffix_and_charset verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "IMAGE/SVG+XML;CHARSET=UTF-8".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix().unwrap().as_str();
            let essence = mime.essence_str();
            assert(type_@ == "image"@);
            assert(subtype@ == "svg"@);
            assert(suffix@ == "xml"@);
            assert(essence@ == "image/svg+xml"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("charset"@, "utf-8"@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and the lowercase-result component
// assertions are unproved under the current general parsing contract.
test_verify_one_file! {
    #[test] parse_type_and_subtype_in_ascii_lowercase verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "TEXT/HTML".parse::<mime::Mime>().unwrap();
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

// FIXME: (proof): Parse/unwrap and suffix unwrap requirements are unproved;
// the contract does not establish the base-subtype/suffix decomposition.
test_verify_one_file! {
    #[test] parse_structured_suffix verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "image/svg+xml".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix().unwrap().as_str();
            let essence = mime.essence_str();
            assert(type_@ == "image"@);
            assert(subtype@ == "svg"@);
            assert(suffix@ == "xml"@);
            assert(essence@ == "image/svg+xml"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and component assertions are unproved,
// including lowercase parameter names with case-preserved non-charset values.
test_verify_one_file! {
    #[test] parse_parameter_names_in_lowercase_preserving_other_values verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "multipart/form-data;BOUNDARY=AbC123;Foo=MiXeD".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "multipart"@);
            assert(subtype@ == "form-data"@);
            assert(suffix.is_none());
            assert(essence@ == "multipart/form-data"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("boundary"@, "AbC123"@), ("foo"@, "MiXeD"@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and the component/parameter-sequence
// assertions are unproved; the general parsing-result relation is unfinished.
test_verify_one_file! {
    #[test] parse_multiple_unquoted_parameters verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/html;foo=ab;bar=cd".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, "ab"@), ("bar"@, "cd"@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and the returned components are unproved;
// FromStr is not yet connected to the quoted-value model.
test_verify_one_file! {
    #[test] parse_quoted_parameter verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = r#"text/html;foo="ab""#.parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, "ab"@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and the component assertions fail;
// FromStr does not yet establish the observed one-quote parameter value.
test_verify_one_file! {
    #[test] parse_triple_quote_as_one_quote verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = r#"text/html;foo=""""#.parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, "\""@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and the component assertions fail;
// the contract does not establish preservation of the quoted backslash.
test_verify_one_file! {
    #[test] parse_backslash_as_ordinary_quoted_data verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = r#"text/html;foo="a\b""#.parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, r"a\b"@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and the result sequence are unproved;
// FromStr does not yet relate quoted scanning and subsequent parameters to its result.
test_verify_one_file! {
    #[test] parse_quoted_semicolon_and_following_parameter verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = r#"text/html;foo="a;b";bar=c"#.parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, "a;b"@), ("bar"@, "c"@)]);
        }
    } => Ok(())
}

// FIXME: (fidelity, 2026-10-13): Rust retains this Unicode quoted value. The
// automaton's parent value check rejects code points above U+00FF and drops foo.
// Keep the Rust value and Ok(()) expectation for the later manual correction.
test_verify_one_file! {
    #[test] parse_unicode_quoted_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = r#"text/html;foo="é🦀""#.parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, "é🦀"@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and both parameter values are unproved;
// the general FromStr result relation is unfinished.
test_verify_one_file! {
    #[test] parse_parameter_after_unicode_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = r#"text/html;pre="é";foo="ab""#.parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("pre"@, "é"@), ("foo"@, "ab"@)]);
        }
    } => Ok(())
}

// FIXME: (fidelity, 2026-10-13): Rust retains both duplicate entries in order;
// the automaton's name_is_absent check discards the second one. Keep both entries
// in the expected view and require Ok(()). Parameter-entry/API bridging is also pending.
test_verify_one_file! {
    #[test] parse_duplicate_parameter_lookup verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/html;foo=first;FOO=second".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            // Rust params() retains both entries; get_param observes the first.
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~~= seq![("foo"@, "first"@), ("foo"@, "second"@)]);
            let first = mime.get_param("foo").unwrap().as_str();
            assert(first@ == "first"@);
        }
    } => Ok(())
}

// FIXME: (fidelity, 2026-10-13): Rust retains ("foo", "") at EOF; the current
// automaton reaches Done without storing that entry. Keep the Rust expectation.
test_verify_one_file! {
    #[test] parse_empty_unquoted_value_at_end verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/html;foo=".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "html"@);
            assert(suffix.is_none());
            assert(essence@ == "text/html"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![("foo"@, ""@)]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap requirements and components are unproved, including
// the literal leading '+' subtype and absence of a separate suffix.
test_verify_one_file! {
    #[test] parse_leading_plus_without_suffix verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "application/+json".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "application"@);
            assert(subtype@ == "+json"@);
            assert(suffix.is_none());
            assert(essence@ == "application/+json"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![]);
        }
    } => Ok(())
}

// FIXME: (proof): Parse/unwrap and suffix unwrap requirements are unproved;
// the contract does not establish splitting at the last non-leading '+'.
test_verify_one_file! {
    #[test] parse_suffix_after_last_plus verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "application/a+b+c".parse::<mime::Mime>().unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix().unwrap().as_str();
            let essence = mime.essence_str();
            assert(type_@ == "application"@);
            assert(subtype@ == "a+b"@);
            assert(suffix@ == "c"@);
            assert(essence@ == "application/a+b+c"@);
            let ghost mime_view = crate::mime_api::view(&mime);
            assert(mime_view.params =~= seq![]);
        }
    } => Ok(())
}

// FIXME: (proof): Each parse precondition and Err assertion is unproved;
// the current contract does not characterize these malformed inputs.
test_verify_one_file! {
    #[test] rejects_missing_or_invalid_type verus_code! {
        use vstd::prelude::*;

        fn test() {
            let empty = "".parse::<mime::Mime>();
            assert(empty.is_err());
            let no_slash = "text".parse::<mime::Mime>();
            assert(no_slash.is_err());
            let no_type = "/html".parse::<mime::Mime>();
            assert(no_type.is_err());
            let invalid_token = "te xt/html".parse::<mime::Mime>();
            assert(invalid_token.is_err());
        }
    } => Ok(())
}

// FIXME: (fidelity, 2026-10-13): Rust rejects these incomplete parameters.
// The automaton instead skips them and reaches Done (retaining bar=baz in the
// last case). These assertions deliberately remain required to verify successfully.
test_verify_one_file! {
    #[test] rejects_incomplete_parameters verus_code! {
        use vstd::prelude::*;

        fn test() {
            let no_equal = "text/html;foo".parse::<mime::Mime>();
            assert(no_equal.is_err());
            let no_equal_before_semicolon = "text/html;foo;".parse::<mime::Mime>();
            assert(no_equal_before_semicolon.is_err());
            let empty_before_semicolon = "text/html;foo=;bar=baz".parse::<mime::Mime>();
            assert(empty_before_semicolon.is_err());
        }
    } => Ok(())
}

// FIXME: (proof): The parse precondition and Err assertion fail; the FromStr
// contract does not yet expose the quoted model's missing-quote outcome.
test_verify_one_file! {
    #[test] rejects_empty_quoted_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let result = r#"text/html;foo="""#.parse::<mime::Mime>();
            assert(result.is_err());
        }
    } => Ok(())
}

// FIXME: (proof): The parse precondition and Err assertion are unproved;
// the contract does not establish rejection of this unterminated value.
test_verify_one_file! {
    #[test] rejects_unterminated_quoted_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let result = r#"text/html;foo="ab"#.parse::<mime::Mime>();
            assert(result.is_err());
        }
    } => Ok(())
}

// FIXME: (proof): The parse precondition and Err assertion are unproved;
// the contract does not capture closing at this quote and rejecting trailing text.
test_verify_one_file! {
    #[test] rejects_backslash_quote_followed_by_text verus_code! {
        use vstd::prelude::*;

        fn test() {
            let result = r#"text/html;foo="a\"b""#.parse::<mime::Mime>();
            assert(result.is_err());
        }
    } => Ok(())
}

// FIXME: (proof): The parse precondition and Err assertion are unproved;
// the current result contract does not characterize post-quote validation.
test_verify_one_file! {
    #[test] rejects_junk_after_quoted_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let result = r#"text/html;foo="ab"C"#.parse::<mime::Mime>();
            assert(result.is_err());
        }
    } => Ok(())
}

// FIXME: (proof): The parse precondition and Err assertion are unproved;
// FromStr does not yet establish rejection of TAB in a quoted value.
test_verify_one_file! {
    #[test] rejects_tab_in_quoted_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let result = "text/html;foo=\"a\tb\"".parse::<mime::Mime>();
            assert(result.is_err());
        }
    } => Ok(())
}

// FIXME: (proof): The parse precondition and Err assertion are unproved;
// FromStr does not yet establish rejection of DEL in a quoted value.
test_verify_one_file! {
    #[test] rejects_del_in_quoted_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let result = "text/html;foo=\"a\u{007F}b\"".parse::<mime::Mime>();
            assert(result.is_err());
        }
    } => Ok(())
}
