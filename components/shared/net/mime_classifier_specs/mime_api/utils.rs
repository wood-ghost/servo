//! Character and sequence utilities for MIME parsing specifications.

use vstd::prelude::*;

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

// mime 0.3.17 checks each quoted byte with `c > 31 && c != 127`.
// For a Rust string, every UTF-8 byte of a non-ASCII scalar satisfies that rule.
// Thus only ASCII controls and DEL are invalid in this character-indexed model.
pub open spec fn is_rust_quoted_value_char(c: char) -> bool {
    c >= '\u{0020}' && c != '\u{007F}'
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

} // verus!
