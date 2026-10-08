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

// FIXME (proof): `mime_parse_requires`, the Ok guarantee for unwrap, and the
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
            assert(mime_view.params =~= map![]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and suffix presence are unproved;
// the contract also lacks the normalized component and charset guarantees.
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
            assert(mime_view.params =~= map!["charset"@ => "utf-8"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the lowercase-result component
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
            assert(mime_view.params =~= map![]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap and suffix unwrap requirements are unproved;
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
            assert(mime_view.params =~= map![]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and component assertions are unproved,
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
            assert(mime_view.params =~= map!["boundary"@ => "AbC123"@, "foo"@ => "MiXeD"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the component/parameter-map
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
            assert(mime_view.params =~= map!["foo"@ => "ab"@, "bar"@ => "cd"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the returned components are unproved;
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
            assert(mime_view.params =~= map!["foo"@ => "ab"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the component assertions fail;
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
            assert(mime_view.params =~= map!["foo"@ => "\""@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the component assertions fail;
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
            assert(mime_view.params =~= map!["foo"@ => r"a\b"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the result map are unproved;
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
            assert(mime_view.params =~= map!["foo"@ => "a;b"@, "bar"@ => "c"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and component assertions are unproved,
// including the Unicode parameter value in the returned view.
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
            assert(mime_view.params =~= map!["foo"@ => "é🦀"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and both parameter values are unproved;
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
            assert(mime_view.params =~= map!["pre"@ => "é"@, "foo"@ => "ab"@]);
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` lacks a specification, so this test stops
// before proof checking the parser result and first-duplicate lookup.
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
            // A Map cannot express the complete duplicate-bearing iterator.
            let first = mime.get_param("foo").unwrap().as_str();
            assert(first@ == "first"@);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and component assertions fail;
// the contract does not establish the observed empty unquoted value at EOF.
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
            assert(mime_view.params =~= map!["foo"@ => ""@]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and components are unproved, including
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
            assert(mime_view.params =~= map![]);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap and suffix unwrap requirements are unproved;
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
            assert(mime_view.params =~= map![]);
        }
    } => Ok(())
}

// FIXME (proof): Each parse precondition and Err assertion is unproved;
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

// FIXME (proof): Parse preconditions and both Err assertions are unproved;
// incomplete-parameter rejection is absent from the current result contract.
test_verify_one_file! {
    #[test] rejects_incomplete_parameters verus_code! {
        use vstd::prelude::*;

        fn test() {
            let no_equal = "text/html;foo".parse::<mime::Mime>();
            assert(no_equal.is_err());
            let empty_before_semicolon = "text/html;foo=;bar=baz".parse::<mime::Mime>();
            assert(empty_before_semicolon.is_err());
        }
    } => Ok(())
}

// FIXME (proof): The parse precondition and Err assertion fail; the FromStr
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

// FIXME (proof): The parse precondition and Err assertion are unproved;
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

// FIXME (proof): The parse precondition and Err assertion are unproved;
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

// FIXME (proof): The parse precondition and Err assertion are unproved;
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

// FIXME (proof): The parse precondition and Err assertion are unproved;
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

// FIXME (proof): The parse precondition and Err assertion are unproved;
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
