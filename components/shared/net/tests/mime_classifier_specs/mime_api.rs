/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! External MIME API regressions, with values observed in ordinary Rust.
//! All API cases require successful verification, including pending contracts.
//! Failure notes describe the 2026-10-08 run in README.md: "support" failures
//! stop before proof checking; "proof" failures are unproved obligations.

#[macro_use]
#[allow(dead_code)] // The error-expectation helpers are exercised in mime_parse.rs.
mod common;

// Accessors and cloning. Constants isolate these cases from parsing requirements.

// FIXME (support): Verus 0.2026.10.05.4558d3d.dirty rejects auto_reveal_strlit;
// this case needs a verifier build with literal auto-reveal support.
test_verify_one_file! {
    #[test] mime_constant_components verus_code! {
        use vstd::prelude::*;

        #[verifier::auto_reveal_strlit]
        fn test() {
            let mime = mime::TEXT_PLAIN_UTF_8;
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "plain"@);
            assert(suffix.is_none());
            assert(essence@ =~= "text/plain"@);
            let ghost view = crate::mime_api::view(&mime);
            assert(view.params =~= map!["charset"@ => "utf-8"@]);
        }
    } => Ok(())
}

// FIXME (support): `mime::IMAGE_SVG` lacks an external constant specification.
test_verify_one_file! {
    #[test] mime_constant_suffix verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::IMAGE_SVG;
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix().unwrap().as_str();
            let essence = mime.essence_str();
            assert(type_@ == "image"@);
            assert(subtype@ == "svg"@);
            assert(suffix@ == "xml"@);
            assert(essence@ == "image/svg+xml"@);
        }
    } => Ok(())
}

// FIXME (support): Verus 0.2026.10.05.4558d3d.dirty rejects auto_reveal_strlit;
// this case needs a verifier build with literal auto-reveal support.
test_verify_one_file! {
    #[test] mime_clone_constant_components verus_code! {
        use vstd::prelude::*;

        #[verifier::auto_reveal_strlit]
        fn test() {
            let original = mime::TEXT_PLAIN_UTF_8;
            let cloned = original.clone();
            let type_ = cloned.type_().as_str();
            let subtype = cloned.subtype().as_str();
            let suffix = cloned.suffix();
            let essence = cloned.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "plain"@);
            assert(suffix.is_none());
            assert(essence@ =~= "text/plain"@);
            let ghost view = crate::mime_api::view(&cloned);
            assert(view.params =~= map!["charset"@ => "utf-8"@]);
        }
    } => Ok(())
}

// FIXME (proof): Parsing/unwrap requirements and the cloned component assertions
// are unproved; cloning cannot supply the missing facts about the parsed input.
test_verify_one_file! {
    #[test] mime_clone_dynamic_components verus_code! {
        use vstd::prelude::*;

        fn test() {
            let original = "image/svg+xml;foo=AbC".parse::<mime::Mime>().unwrap();
            let cloned = original.clone();
            let type_ = cloned.type_().as_str();
            let subtype = cloned.subtype().as_str();
            let suffix = cloned.suffix().unwrap().as_str();
            let essence = cloned.essence_str();
            assert(type_@ == "image"@);
            assert(subtype@ == "svg"@);
            assert(suffix@ == "xml"@);
            assert(essence@ == "image/svg+xml"@);
            let ghost view = crate::mime_api::view(&cloned);
            assert(view.params =~= map!["foo"@ => "AbC"@]);
        }
    } => Ok(())
}

// FIXME (proof): `cloned_text@ == "text"@` is unproved after `Name::clone`.
test_verify_one_file! {
    #[test] name_clone_and_copy verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let copied = name;
            let cloned = name.clone();
            let original_text = name.as_str();
            let copied_text = copied.as_str();
            let cloned_text = cloned.as_str();
            assert(original_text@ == "text"@);
            assert(copied_text@ == "text"@);
            assert(cloned_text@ == "text"@);
        }
    } => Ok(())
}

// Parameter lookup and iterator state, including duplicates and exhaustion.

// FIXME (support): `Mime::get_param` lacks a specification; absence is not checked.
test_verify_one_file! {
    #[test] get_param_absent verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN;
            let charset = mime.get_param("charset");
            let unknown = mime.get_param("baz");
            assert(charset.is_none());
            assert(unknown.is_none());
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` lacks a specification; lookup is not checked.
test_verify_one_file! {
    #[test] get_param_present_constant verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN_UTF_8;
            let value = mime.get_param("CHARSET").unwrap().as_str();
            assert(value@ == "utf-8"@);
        }
    } => Ok(())
}

// FIXME (support): Both `Mime::get_param` and the `mime::CHARSET` constant need specs.
test_verify_one_file! {
    #[test] get_param_string_and_name verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let by_string = mime.get_param("CHARSET").unwrap().as_str();
            let by_name = mime.get_param(mime::CHARSET).unwrap().as_str();
            let boundary = mime.get_param("BOUNDARY").unwrap().as_str();
            assert(by_string@ == "base64"@);
            assert(by_name@ == "base64"@);
            assert(boundary@ == "AbC"@);
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` blocks translation before first-match checking.
test_verify_one_file! {
    #[test] get_param_first_duplicate verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let first = mime.get_param("foo").unwrap().as_str();
            let absent = mime.get_param("missing");
            assert(first@ == "text"@);
            assert(absent.is_none());
        }
    } => Ok(())
}

// FIXME (support): `Params`, `Mime::params`, and `Params::size_hint` need specs
// before the empty-iterator and exhaustion assertions can be checked.
test_verify_one_file! {
    #[test] params_empty_next_and_size_hint verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN;
            let mut params = mime.params();
            let before = params.size_hint();
            let first = params.next();
            let after = params.size_hint();
            let again = params.next();
            assert(before == (0, Some(0)));
            assert(first.is_none());
            assert(after == (0, Some(0)));
            assert(again.is_none());
        }
    } => Ok(())
}

// FIXME (support): `Params`, `Mime::params`, and `Params::size_hint` need specs
// before the singleton's state changes and exhaustion can be checked.
test_verify_one_file! {
    #[test] params_utf8_advances_and_exhausts verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN_UTF_8;
            let mut params = mime.params();
            let before = params.size_hint();
            assert(before == (1, Some(1)));
            let pair = params.next().unwrap();
            let name = pair.0.as_str();
            let value = pair.1.as_str();
            assert(name@ == "charset"@);
            assert(value@ == "utf-8"@);
            let after = params.size_hint();
            let exhausted = params.next();
            let again = params.next();
            assert(after == (0, Some(0)));
            assert(exhausted.is_none());
            assert(again.is_none());
        }
    } => Ok(())
}

// FIXME (support): `Params`, `Mime::params`, and `Params::size_hint` need specs;
// the ordered duplicate entries and remaining-length assertions are not reached.
test_verify_one_file! {
    #[test] params_order_duplicates_and_size_hint verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/html;foo=first;FOO=second;bar=last".parse::<mime::Mime>().unwrap();
            let mut params = mime.params();
            let before = params.size_hint();
            assert(before == (3, Some(3)));
            let first = params.next().unwrap();
            let first_name = first.0.as_str();
            let first_value = first.1.as_str();
            let after_first = params.size_hint();
            assert(first_name@ == "foo"@);
            assert(first_value@ == "first"@);
            assert(after_first == (2, Some(2)));
            let second = params.next().unwrap();
            let second_name = second.0.as_str();
            let second_value = second.1.as_str();
            let after_second = params.size_hint();
            assert(second_name@ == "foo"@);
            assert(second_value@ == "second"@);
            assert(after_second == (1, Some(1)));
            let third = params.next().unwrap();
            let third_name = third.0.as_str();
            let third_value = third.1.as_str();
            let after_third = params.size_hint();
            let exhausted = params.next();
            let again = params.next();
            assert(third_name@ == "bar"@);
            assert(third_value@ == "last"@);
            assert(after_third == (0, Some(0)));
            assert(exhausted.is_none());
            assert(again.is_none());
        }
    } => Ok(())
}

// FIXME (support): `Params` and `Mime::params` block translation before formatting.
test_verify_one_file! {
    #[test] params_debug verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN;
            let params = mime.params();
            let rendered = format!("{:?}", params);
            assert(rendered@ == "Params"@);
        }
    } => Ok(())
}

// Equality: bind the executable comparison result, then check the Rust oracle.

test_verify_one_file! {
    #[test] mime_eq_constants verus_code! {
        use vstd::prelude::*;

        fn test() {
            let plain = mime::TEXT_PLAIN;
            let other_plain = mime::TEXT_PLAIN;
            let html = mime::TEXT_HTML;
            let same = plain == other_plain;
            let different = plain == html;
            assert(same);
            assert(!different);
        }
    } => Ok(())
}

// FIXME (proof): The MIME/string comparison contracts do not establish the
// expected forward, reverse, or mismatch results.
test_verify_one_file! {
    #[test] mime_eq_strings_both_orders verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN;
            let forward = mime == "TEXT/PLAIN";
            let reverse = "TEXT/PLAIN" == mime;
            let mismatch = mime == "text/html";
            assert(forward);
            assert(reverse);
            assert(!mismatch);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and equality with the constant are
// unproved until the parsed MIME's result is specified.
test_verify_one_file! {
    #[test] mime_eq_parsed_and_constant verus_code! {
        use vstd::prelude::*;

        fn test() {
            let parsed = "TEXT/PLAIN".parse::<mime::Mime>().unwrap();
            let constant = mime::TEXT_PLAIN;
            let forward = parsed == constant;
            let reverse = constant == parsed;
            assert(forward);
            assert(reverse);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the equality assertion are unproved.
// Fidelity: the observed Rust equality is not equality of complete parameter maps.
test_verify_one_file! {
    #[test] mime_eq_parameter_values_pinned_behavior verus_code! {
        use vstd::prelude::*;

        fn test() {
            let left = "text/plain;foo=first".parse::<mime::Mime>().unwrap();
            let right = "text/plain;foo=second".parse::<mime::Mime>().unwrap();
            // Observed in mime 0.3.17: different parameter values compare equal.
            let forward = left == right;
            let reverse = right == left;
            assert(forward);
            assert(reverse);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and inequality assertions are unproved.
// Fidelity: an unordered parameter map cannot capture the observed order effect.
test_verify_one_file! {
    #[test] mime_eq_parameter_names_and_order verus_code! {
        use vstd::prelude::*;

        fn test() {
            let foo = "text/plain;foo=first".parse::<mime::Mime>().unwrap();
            let bar = "text/plain;bar=first".parse::<mime::Mime>().unwrap();
            let different_names = foo == bar;
            assert(!different_names);
            let left = "text/plain;foo=first;bar=last".parse::<mime::Mime>().unwrap();
            let right = "text/plain;bar=last;foo=first".parse::<mime::Mime>().unwrap();
            let forward = left == right;
            let reverse = right == left;
            assert(!forward);
            assert(!reverse);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and the equality assertion are unproved.
// Fidelity: Rust ignores the extra parameter here, unlike equality of full views.
test_verify_one_file! {
    #[test] mime_eq_extra_parameters_pinned_behavior verus_code! {
        use vstd::prelude::*;

        fn test() {
            let left = "text/plain;foo=first".parse::<mime::Mime>().unwrap();
            let right = "text/plain;foo=first;bar=last".parse::<mime::Mime>().unwrap();
            let forward = left == right;
            let reverse = right == left;
            assert(forward);
            assert(reverse);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and equality of the two parsed values
// are unproved with the current general parsing contract.
test_verify_one_file! {
    #[test] mime_eq_quoted_and_unquoted verus_code! {
        use vstd::prelude::*;

        fn test() {
            let left = "text/plain;foo=first".parse::<mime::Mime>().unwrap();
            let right = r#"text/plain;foo="first""#.parse::<mime::Mime>().unwrap();
            let forward = left == right;
            let reverse = right == left;
            assert(forward);
            assert(reverse);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and both comparison assertions fail.
// Fidelity: symmetric view equality cannot model the measured Rust asymmetry.
test_verify_one_file! {
    #[test] mime_eq_utf8_fast_path_asymmetry verus_code! {
        use vstd::prelude::*;

        fn test() {
            let utf8 = mime::TEXT_PLAIN_UTF_8;
            let custom = "text/plain; charset=abcde".parse::<mime::Mime>().unwrap();
            // These unequal directions are the actual pinned Rust outcomes.
            let forward = utf8 == custom;
            let reverse = custom == utf8;
            assert(!forward);
            assert(reverse);
        }
    } => Ok(())
}

// FIXME (proof): The Name/string comparison contracts do not establish the
// expected case-insensitive forward, reverse, or mismatch results.
test_verify_one_file! {
    #[test] name_eq_strings_both_orders verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let forward = name == "TEXT";
            let reverse = "TEXT" == name;
            let mismatch = name == "image";
            assert(forward);
            assert(reverse);
            assert(!mismatch);
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` blocks translation before the case-sensitive
// Name/string comparisons can be verified.
test_verify_one_file! {
    #[test] name_eq_case_sensitive_parameter_value verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let name = mime.get_param("boundary").unwrap();
            let same = name == "AbC";
            let same_reverse = "AbC" == name;
            let changed_case = name == "abc";
            let changed_reverse = "abc" == name;
            assert(same);
            assert(same_reverse);
            assert(!changed_case);
            assert(!changed_reverse);
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` blocks translation before charset-value
// case-insensitive comparisons can be verified.
test_verify_one_file! {
    #[test] name_eq_charset_value_ignores_case verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let name = mime.get_param("charset").unwrap();
            let forward = name == "bAsE64";
            let reverse = "bAsE64" == name;
            assert(forward);
            assert(reverse);
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` blocks translation before the equality checks.
// Fidelity: text-only `name_identity` equality omits the observed sensitivity flag.
test_verify_one_file! {
    #[test] name_eq_same_text_different_sensitivity verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let value = mime.get_param("foo").unwrap();
            let constant = mime::TEXT;
            let value_text = value.as_str();
            let constant_text = constant.as_str();
            let forward = value == constant;
            let reverse = constant == value;
            assert(value_text@ == "text"@);
            assert(constant_text@ == "text"@);
            assert(!forward);
            assert(!reverse);
        }
    } => Ok(())
}

// Conversion, formatting, and ordering return concrete observations as well.

// FIXME (support): `<Mime as AsRef<str>>::as_ref` lacks an external specification.
test_verify_one_file! {
    #[test] mime_as_ref_preserves_serialization verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN_UTF_8;
            let source: &str = mime.as_ref();
            assert(source@ == "text/plain; charset=utf-8"@);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] name_as_str verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let text = name.as_str();
            assert(text@ == "text"@);
        }
    } => Ok(())
}

// FIXME (support): `<Name as AsRef<str>>::as_ref` lacks an external specification.
test_verify_one_file! {
    #[test] name_as_ref verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let text: &str = name.as_ref();
            assert(text@ == "text"@);
        }
    } => Ok(())
}

// FIXME (proof): The Name-to-str conversion does not establish `text@ == "text"@`.
test_verify_one_file! {
    #[test] name_into_str verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let text: &str = name.into();
            assert(text@ == "text"@);
        }
    } => Ok(())
}

// FIXME (proof): `to_string` does not establish the expected serialized MIME text.
test_verify_one_file! {
    #[test] mime_display verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN_UTF_8;
            let rendered = mime.to_string();
            assert(rendered@ == "text/plain; charset=utf-8"@);
        }
    } => Ok(())
}

// FIXME (proof): The Debug formatting precondition and quoted output are unproved.
test_verify_one_file! {
    #[test] mime_debug verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mime = mime::TEXT_PLAIN_UTF_8;
            let rendered = format!("{:?}", mime);
            assert(rendered@ == "\"text/plain; charset=utf-8\""@);
        }
    } => Ok(())
}

// FIXME (proof): `to_string` does not establish the expected Name text.
test_verify_one_file! {
    #[test] name_display verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let rendered = name.to_string();
            assert(rendered@ == "text"@);
        }
    } => Ok(())
}

// FIXME (proof): The Debug formatting precondition and quoted Name text are unproved.
test_verify_one_file! {
    #[test] name_debug verus_code! {
        use vstd::prelude::*;

        fn test() {
            let name = mime::TEXT;
            let rendered = format!("{:?}", name);
            assert(rendered@ == "\"text\""@);
        }
    } => Ok(())
}

// FIXME (proof): Current Ord/PartialOrd contracts do not establish any of the
// three expected ordering results, including self-comparison.
test_verify_one_file! {
    #[test] mime_order_constants verus_code! {
        use vstd::prelude::*;
        use std::cmp::Ordering;

        fn test() {
            let html = mime::TEXT_HTML;
            let plain = mime::TEXT_PLAIN;
            let cmp = html.cmp(&plain);
            let partial = html.partial_cmp(&plain);
            let same = html.cmp(&html);
            assert(cmp == Ordering::Less);
            assert(partial == Some(Ordering::Less));
            assert(same == Ordering::Equal);
        }
    } => Ok(())
}

// FIXME (proof): Parse/unwrap requirements and serialization-based ordering
// assertions are unproved with the current parsing and ordering contracts.
test_verify_one_file! {
    #[test] mime_order_uses_serialization verus_code! {
        use vstd::prelude::*;
        use std::cmp::Ordering;

        fn test() {
            let bare = "text/plain;foo=first".parse::<mime::Mime>().unwrap();
            let quoted = r#"text/plain;foo="first""#.parse::<mime::Mime>().unwrap();
            let cmp = bare.cmp(&quoted);
            let partial = bare.partial_cmp(&quoted);
            assert(cmp == Ordering::Greater);
            assert(partial == Some(Ordering::Greater));
        }
    } => Ok(())
}

// FIXME (proof): Current Name Ord/PartialOrd contracts do not establish the
// expected ordering results, including self-comparison.
test_verify_one_file! {
    #[test] name_order_constants verus_code! {
        use vstd::prelude::*;
        use std::cmp::Ordering;

        fn test() {
            let image = mime::IMAGE;
            let text = mime::TEXT;
            let cmp = image.cmp(&text);
            let partial = image.partial_cmp(&text);
            let same = text.cmp(&text);
            assert(cmp == Ordering::Less);
            assert(partial == Some(Ordering::Less));
            assert(same == Ordering::Equal);
        }
    } => Ok(())
}

// FIXME (support): `Mime::get_param` blocks translation before sensitivity-aware
// ordering can be checked.
test_verify_one_file! {
    #[test] name_order_includes_sensitivity verus_code! {
        use vstd::prelude::*;
        use std::cmp::Ordering;

        fn test() {
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let value = mime.get_param("foo").unwrap();
            let constant = mime::TEXT;
            let cmp = value.cmp(&constant);
            let partial = value.partial_cmp(&constant);
            assert(cmp == Ordering::Less);
            assert(partial == Some(Ordering::Less));
        }
    } => Ok(())
}

// Record bytes rather than asserting unstable DefaultHasher numeric digests.

// FIXME (support): `<Mime as Hash>::hash` lacks a specification for its writes.
test_verify_one_file! {
    #[test] mime_hash_writes_source_bytes verus_code! {
        use vstd::prelude::*;
        use std::hash::{Hash, Hasher};

        struct Recorder { bytes: Vec<u8> }

        impl Hasher for Recorder {
            fn finish(&self) -> u64 { 0 }
            fn write(&mut self, bytes: &[u8]) {
                self.bytes.extend_from_slice(bytes);
            }
        }

        fn test() {
            let plain = mime::TEXT_PLAIN;
            let mut first = Recorder { bytes: Vec::new() };
            plain.hash(&mut first);
            assert(first.bytes@ == b"text/plain"@);
            let utf8 = mime::TEXT_PLAIN_UTF_8;
            let mut second = Recorder { bytes: Vec::new() };
            utf8.hash(&mut second);
            assert(second.bytes@ == b"text/plain; charset=utf-8"@);
        }
    } => Ok(())
}

// FIXME (support): `<Name as Hash>::hash` and `Mime::get_param` lack specs;
// neither recorded-byte assertion reaches proof checking.
test_verify_one_file! {
    #[test] name_hash_includes_sensitivity verus_code! {
        use vstd::prelude::*;
        use std::hash::{Hash, Hasher};

        struct Recorder { bytes: Vec<u8> }

        impl Hasher for Recorder {
            fn finish(&self) -> u64 { 0 }
            fn write(&mut self, bytes: &[u8]) {
                self.bytes.extend_from_slice(bytes);
            }
        }

        fn test() {
            let constant = mime::TEXT;
            let mut first = Recorder { bytes: Vec::new() };
            constant.hash(&mut first);
            assert(first.bytes@ == b"text\xff\x01"@);
            let mime = "text/plain;CHARSET=BASE64;boundary=AbC;foo=text;FOO=second"
                .parse::<mime::Mime>().unwrap();
            let value = mime.get_param("foo").unwrap();
            let mut second = Recorder { bytes: Vec::new() };
            value.hash(&mut second);
            assert(second.bytes@ == b"text\xff\x00"@);
        }
    } => Ok(())
}

// Direct FromStr, errors, and the MIME-list iterator's public API.

// FIXME (proof): `from_str_requires`, the Ok guarantee for unwrap, and the
// general parsed-component relation are unfinished.
test_verify_one_file! {
    #[test] mime_from_str_direct verus_code! {
        use vstd::prelude::*;
        use std::str::FromStr;

        fn test() {
            let mime = mime::Mime::from_str("TEXT/PLAIN").unwrap();
            let type_ = mime.type_().as_str();
            let subtype = mime.subtype().as_str();
            let suffix = mime.suffix();
            let essence = mime.essence_str();
            assert(type_@ == "text"@);
            assert(subtype@ == "plain"@);
            assert(suffix.is_none());
            assert(essence@ == "text/plain"@);
            let ghost view = crate::mime_api::view(&mime);
            assert(view.params =~= map![]);
        }
    } => Ok(())
}

// FIXME (proof): The FromStr precondition is unproved and its contract does not
// establish Err for this input.
test_verify_one_file! {
    #[test] mime_from_str_direct_rejects verus_code! {
        use vstd::prelude::*;
        use std::str::FromStr;

        fn test() {
            let result = mime::Mime::from_str("text");
            assert(result.is_err());
        }
    } => Ok(())
}

// FIXME (proof): FromStr/unwrap_err requirements and the rendered error-message
// assertions are unproved.
test_verify_one_file! {
    #[test] error_display verus_code! {
        use vstd::prelude::*;
        use std::str::FromStr;

        fn test() {
            let missing_slash = mime::Mime::from_str("text").unwrap_err();
            let missing_text = missing_slash.to_string();
            assert(missing_text@ == "mime parse error: a slash (/) was missing between the type and subtype"@);
            let invalid_token = mime::Mime::from_str("te xt/html").unwrap_err();
            let invalid_text = invalid_token.to_string();
            assert(invalid_text@ == "mime parse error: an invalid token was encountered, 20 at position 2"@);
        }
    } => Ok(())
}

// FIXME (proof): FromStr/unwrap_err and Debug formatting preconditions fail;
// the rendered diagnostic assertion is also unproved.
test_verify_one_file! {
    #[test] error_debug verus_code! {
        use vstd::prelude::*;
        use std::str::FromStr;

        fn test() {
            let error = mime::Mime::from_str("te xt/html").unwrap_err();
            let rendered = format!("{:?}", error);
            assert(rendered@ == "FromStrError { inner: InvalidToken { pos: 2, byte: 32 } }"@);
        }
    } => Ok(())
}

// FIXME (support): The `std::error::Error` trait is not declared to Verus.
test_verify_one_file! {
    #[test] error_source_is_none verus_code! {
        use vstd::prelude::*;
        use std::error::Error;
        use std::str::FromStr;

        fn test() {
            let error = mime::Mime::from_str("text").unwrap_err();
            let source = error.source();
            assert(source.is_none());
        }
    } => Ok(())
}

// FIXME (support): `MimeIter`, `MimeIter::new`, and its default `size_hint` need
// specs before the item and exhaustion assertions can be checked.
test_verify_one_file! {
    #[test] mime_iter_empty_and_single verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mut empty = mime::MimeIter::new("");
            let hint = empty.size_hint();
            let empty_first = empty.next();
            let empty_again = empty.next();
            assert(hint == (0, None));
            assert(empty_first.is_none());
            assert(empty_again.is_none());
            let mut single = mime::MimeIter::new("text/plain");
            let mime = single.next().unwrap().unwrap();
            let essence = mime.essence_str();
            let exhausted = single.next();
            let again = single.next();
            assert(essence@ == "text/plain"@);
            assert(exhausted.is_none());
            assert(again.is_none());
        }
    } => Ok(())
}

// FIXME (support): `MimeIter` and `MimeIter::new` block translation before
// advancement and cloning of iterator state can be checked.
test_verify_one_file! {
    #[test] mime_iter_advancement_and_clone verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mut iter = mime::MimeIter::new("text/plain, image/png");
            let first = iter.next().unwrap().unwrap();
            let first_essence = first.essence_str();
            assert(first_essence@ == "text/plain"@);
            let mut cloned = iter.clone();
            let second = iter.next().unwrap().unwrap();
            let cloned_second = cloned.next().unwrap().unwrap();
            let second_essence = second.essence_str();
            let cloned_essence = cloned_second.essence_str();
            let exhausted = iter.next();
            let clone_exhausted = cloned.next();
            assert(second_essence@ == "image/png"@);
            assert(cloned_essence@ == "image/png"@);
            assert(exhausted.is_none());
            assert(clone_exhausted.is_none());
        }
    } => Ok(())
}

// FIXME (support): `MimeIter` and `MimeIter::new` block translation before
// the error-slice, continuation, and incomplete-input results can be checked.
test_verify_one_file! {
    #[test] mime_iter_invalid_prefix_and_incomplete_input verus_code! {
        use vstd::prelude::*;

        fn test() {
            let mut iter = mime::MimeIter::new("bad, text/plain");
            let invalid = iter.next().unwrap().unwrap_err();
            let valid = iter.next().unwrap().unwrap();
            let essence = valid.essence_str();
            let exhausted = iter.next();
            assert(invalid@ == "bad"@);
            assert(essence@ == "text/plain"@);
            assert(exhausted.is_none());
            let mut incomplete = mime::MimeIter::new("text");
            let first = incomplete.next();
            let again = incomplete.next();
            assert(first.is_none());
            assert(again.is_none());
        }
    } => Ok(())
}

// FIXME (support): `MimeIter` and `MimeIter::new` block translation before Debug.
test_verify_one_file! {
    #[test] mime_iter_debug verus_code! {
        use vstd::prelude::*;

        fn test() {
            let iter = mime::MimeIter::new("text/plain");
            let rendered = format!("{:?}", iter);
            assert(rendered@ == "MimeIter { pos: 0, source: \"text/plain\" }"@);
        }
    } => Ok(())
}
