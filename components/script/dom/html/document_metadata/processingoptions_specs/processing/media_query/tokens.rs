/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! CSS Syntax component values for the MQ4 grammar. Unused token kinds need only
//! retain their boundaries and whether they are legal in `<general-enclosed>`.
//! <https://drafts.csswg.org/css-syntax-3/#tokenization>
use net_traits::mime_classifier_specs::mime_api::ascii_lowercase;
use vstd::arithmetic::power::pow;
use vstd::prelude::*;

verus! {

pub(super) enum Token {
    Space,
    Ident(Seq<char>),
    Number(real, bool),
    Dimension(real, Seq<char>),
    Percentage(real),
    Delim(char),
    Block(char, Seq<Token>),
    Function(Seq<char>, Seq<Token>),
    Other,
    Bad,
}

enum Lex {
    Atom(Token),
    Open(char, Option<Seq<char>>),
    Close(char),
    Comment,
}

spec fn preprocess(s: Seq<char>, i: int) -> Seq<char>
    decreases s.len() - i,
{
    if i < 0 || i >= s.len() {
        Seq::empty()
    } else {
        let c = s[i];
        let n = if c == '\r' && i + 1 < s.len() && s[i + 1] == '\n' {
            2int
        } else {
            1int
        };
        seq![
            if c == '\r' || c == '\u{000c}' {
                '\n'
            } else if c == '\0' {
                '\u{fffd}'
            } else {
                c
            },
        ] + preprocess(s, i + n)
    }
}

spec fn at(s: Seq<char>, i: int) -> char {
    if 0 <= i < s.len() {
        s[i]
    } else {
        '\0'
    }  // NUL was replaced in preprocessing.
}

spec fn ws(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}

spec fn digit(c: char) -> bool {
    '0' <= c <= '9'
}

spec fn hex(c: char) -> int {
    if digit(c) {
        c as int - '0' as int
    } else if 'a' <= c <= 'f' {
        c as int - 'a' as int + 10
    } else if 'A' <= c <= 'F' {
        c as int - 'A' as int + 10
    } else {
        -1
    }
}

spec fn name_start(c: char) -> bool {
    let u = c as int;
    'a' <= c <= 'z' || 'A' <= c <= 'Z' || c == '_' || u == 0xb7 || 0xc0 <= u <= 0xd6 || 0xd8 <= u
        <= 0xf6 || 0xf8 <= u <= 0x37d || 0x37f <= u <= 0x1fff || 0x200c <= u <= 0x200d || 0x203f
        <= u <= 0x2040 || 0x2070 <= u <= 0x218f || 0x2c00 <= u <= 0x2fef || 0x3001 <= u <= 0xd7ff
        || 0xf900 <= u <= 0xfdcf || 0xfdf0 <= u <= 0xfffd || u >= 0x10000
}

spec fn name_char(c: char) -> bool {
    name_start(c) || digit(c) || c == '-'
}

spec fn escape(s: Seq<char>, i: int) -> bool {
    at(s, i) == '\\' && at(s, i + 1) != '\n'
}

spec fn starts_ident(s: Seq<char>, i: int) -> bool {
    name_start(at(s, i)) || escape(s, i) || (at(s, i) == '-' && (name_start(at(s, i + 1)) || at(
        s,
        i + 1,
    ) == '-' || escape(s, i + 1)))
}

spec fn run(s: Seq<char>, i: int, p: spec_fn(char) -> bool) -> int
    decreases s.len() - i,
{
    if 0 <= i < s.len() && p(s[i]) {
        run(s, i + 1, p)
    } else {
        i
    }
}

spec fn digits_value(s: Seq<char>, radix: int) -> int
    decreases s.len(),
{
    if s.len() == 0 {
        0
    } else {
        digits_value(s.drop_last(), radix) * radix + hex(s.last())
    }
}

spec fn escaped(s: Seq<char>, i: int) -> (char, int) {
    let j = i + 1;
    if hex(at(s, j)) >= 0 {
        let available = run(s, j, |c: char| hex(c) >= 0);
        let end = if available > j + 6 {
            j + 6
        } else {
            available
        };
        let v = digits_value(s.subrange(j, end), 16);
        (
            if v == 0 || 0xd800 <= v <= 0xdfff || v > 0x10ffff {
                '\u{fffd}'
            } else {
                (v as u32) as char
            },
            if ws(at(s, end)) {
                end + 1
            } else {
                end
            },
        )
    } else {
        (
            if j < s.len() {
                at(s, j)
            } else {
                '\u{fffd}'
            },
            j + 1,
        )
    }
}

spec fn name(s: Seq<char>, i: int) -> (Seq<char>, int)
    decreases
            if i < s.len() {
                s.len() - i
            } else {
                0int
            },
{
    if !(0 <= i < s.len()) {
        (Seq::empty(), i)
    } else if name_char(s[i]) || escape(s, i) {
        let (c, j) = if s[i] == '\\' {
            escaped(s, i)
        } else {
            (s[i], i + 1)
        };
        let (tail, end) = name(
            s,
            if j > i {
                j
            } else {
                i + 1
            },
        );
        (seq![c] + tail, end)
    } else {
        (Seq::empty(), i)
    }
}

spec fn starts_number(s: Seq<char>, i: int) -> bool {
    let j = if at(s, i) == '+' || at(s, i) == '-' {
        i + 1
    } else {
        i
    };
    digit(at(s, j)) || (at(s, j) == '.' && digit(at(s, j + 1)))
}

spec fn power_ten(n: int) -> real {
    if n >= 0 {
        pow(10, n as nat) as real
    } else {
        1real / (pow(10, (-n) as nat) as real)
    }
}

spec fn numeric(s: Seq<char>, i: int) -> (Lex, int) {
    let j = if at(s, i) == '+' || at(s, i) == '-' {
        i + 1
    } else {
        i
    };
    let dot = run(s, j, |c: char| digit(c));
    let frac = at(s, dot) == '.' && digit(at(s, dot + 1));
    let e = if frac {
        run(s, dot + 1, |c: char| digit(c))
    } else {
        dot
    };
    let exp_start = if at(s, e + 1) == '+' || at(s, e + 1) == '-' {
        e + 2
    } else {
        e + 1
    };
    let exp = (at(s, e) == 'e' || at(s, e) == 'E') && digit(at(s, exp_start));
    let end = if exp {
        run(s, exp_start, |c: char| digit(c))
    } else {
        e
    };
    let exponent = if exp {
        digits_value(s.subrange(exp_start, end), 10) * (if at(s, e + 1) == '-' {
            -1int
        } else {
            1int
        })
    } else {
        0
    };
    let mantissa = digits_value(s.subrange(j, dot), 10) as real + if frac {
        digits_value(s.subrange(dot + 1, e), 10) as real * power_ten(dot + 1 - e)
    } else {
        0real
    };
    let v = (if at(s, i) == '-' {
        -mantissa
    } else {
        mantissa
    }) * power_ten(exponent);
    if starts_ident(s, end) {
        let (unit, next) = name(s, end);
        (Lex::Atom(Token::Dimension(v, ascii_lowercase(unit))), next)
    } else if at(s, end) == '%' {
        (Lex::Atom(Token::Percentage(v)), end + 1)
    } else {
        (Lex::Atom(Token::Number(v, !exp && !frac)), end)
    }
}

spec fn comment_end(s: Seq<char>, i: int) -> int
    decreases s.len() - i,
{
    if i >= s.len() {
        i
    } else if at(s, i) == '*' && at(s, i + 1) == '/' {
        i + 2
    } else {
        comment_end(s, i + 1)
    }
}

spec fn string_end(s: Seq<char>, i: int, quote: char) -> (Lex, int)
    decreases
            if i < s.len() {
                s.len() - i
            } else {
                0int
            },
{
    if i >= s.len() {
        (Lex::Atom(Token::Other), i)
    } else if at(s, i) == quote {
        (Lex::Atom(Token::Other), i + 1)
    } else if at(s, i) == '\n' {
        (Lex::Atom(Token::Bad), i)
    } else if at(s, i) == '\\' {
        let end = if at(s, i + 1) == '\n' {
            i + 2
        } else {
            escaped(s, i).1
        };
        string_end(
            s,
            if end > i {
                end
            } else {
                i + 1
            },
            quote,
        )
    } else {
        string_end(s, i + 1, quote)
    }
}

// Unquoted url(...) is a single CSS token; even a bad-url consumes through the
// next unescaped ')' (commas and parentheses inside it are not list separators).
spec fn url_end(s: Seq<char>, i: int, bad: bool) -> (Lex, int)
    decreases
            if i < s.len() {
                s.len() - i
            } else {
                0int
            },
{
    let c = at(s, i);
    if i >= s.len() || c == ')' {
        (
            Lex::Atom(
                if bad {
                    Token::Bad
                } else {
                    Token::Other
                },
            ),
            i + 1,
        )
    } else if escape(s, i) {
        let j = escaped(s, i).1;
        url_end(
            s,
            if j > i {
                j
            } else {
                i + 1
            },
            bad,
        )
    } else if ws(c) {
        let j = run(s, i + 1, |c: char| ws(c));
        url_end(
            s,
            if j > i {
                j
            } else {
                i + 1
            },
            bad || (j < s.len() && at(s, j) != ')'),
        )
    } else {
        url_end(
            s,
            i + 1,
            bad || c == '(' || c == '"' || c == '\'' || c == '\\' || c as int <= 8 || c as int == 11
                || 14 <= c as int <= 31 || c as int == 127,
        )
    }
}

spec fn lex(s: Seq<char>, i: int) -> (Lex, int) {
    let c = at(s, i);
    if ws(c) {
        (Lex::Atom(Token::Space), run(s, i + 1, |c: char| ws(c)))
    } else if c == '/' && at(s, i + 1) == '*' {
        (Lex::Comment, comment_end(s, i + 2))
    } else if c == '"' || c == '\'' {
        string_end(s, i + 1, c)
    } else if starts_number(s, i) {
        numeric(s, i)
    } else if starts_ident(s, i) {
        let (raw, j) = name(s, i);
        let id = ascii_lowercase(raw);
        if at(s, j) != '(' {
            (Lex::Atom(Token::Ident(id)), j)
        } else {
            let k = run(s, j + 1, |c: char| ws(c));
            if id == "url"@ && at(s, k) != '"' && at(s, k) != '\'' {
                url_end(s, k, false)
            } else {
                (Lex::Open(')', Some(id)), j + 1)
            }
        }
    } else if (c == '@' && starts_ident(s, i + 1)) || (c == '#' && (name_char(at(s, i + 1))
        || escape(s, i + 1))) {
        (Lex::Atom(Token::Other), name(s, i + 1).1)
    } else if c == '(' || c == '[' || c == '{' {
        (
            Lex::Open(
                if c == '(' {
                    ')'
                } else if c == '[' {
                    ']'
                } else {
                    '}'
                },
                None,
            ),
            i + 1,
        )
    } else if c == ')' || c == ']' || c == '}' {
        (Lex::Close(c), i + 1)
    } else {
        (Lex::Atom(Token::Delim(c)), i + 1)
    }
}

struct Frame {
    parent: Seq<Token>,
    close: char,
    function: Option<Seq<char>>,
}

spec fn close(f: Frame, contents: Seq<Token>) -> Seq<Token> {
    f.parent.push(
        match f.function {
            Some(n) => Token::Function(n, contents),
            None => Token::Block(f.close, contents),
        },
    )
}

spec fn components(s: Seq<char>, i: int, out: Seq<Token>, stack: Seq<Frame>) -> Seq<Token>
    decreases
            if i < s.len() {
                s.len() - i
            } else {
                0int
            },
            stack.len(),
{
    if i >= s.len() {
        if stack.len() == 0 {
            out
        } else {
            components(s, i, close(stack.last(), out), stack.drop_last())
        }
    } else {
        let (tok, end) = lex(s, i);
        let j = if end > i {
            end
        } else {
            i + 1
        };
        match tok {
            Lex::Comment => components(s, j, out, stack),
            Lex::Atom(t) => components(s, j, out.push(t), stack),
            Lex::Open(close, function) => components(
                s,
                j,
                Seq::empty(),
                stack.push(Frame { parent: out, close, function }),
            ),
            Lex::Close(c) => if stack.len() > 0 && stack.last().close == c {
                components(s, j, close(stack.last(), out), stack.drop_last())
            } else {
                components(s, j, out.push(Token::Bad), stack)
            },
        }
    }
}

pub(super) closed spec fn tokenize(s: Seq<char>) -> Seq<Token> {
    components(preprocess(s, 0), 0, Seq::empty(), Seq::empty())
}

// Exercise CSS escape consumption as well as case folding, rather than testing
// the query grammar only with hand-built tokens.
pub(super) proof fn escaped_all_token()
    ensures
        tokenize("\\61 LL"@) == seq![Token::Ident("all"@)],
{
    let s = "\\61 LL"@;
    reveal_with_fuel(preprocess, 8);
    assert(preprocess(s, 0) =~= s);
    reveal_with_fuel(run, 4);
    reveal_with_fuel(digits_value, 4);
    assert(escaped(s, 0) == ('a', 4int));
    reveal_with_fuel(name, 5);
    assert(name(s, 0).0 =~= "aLL"@);
    assert(ascii_lowercase("aLL"@) =~= "all"@);
    assert(lex(s, 0) == (Lex::Atom(Token::Ident("all"@)), 6int));
    reveal_with_fuel(components, 2);
    assert(tokenize(s) =~= seq![Token::Ident("all"@)]);
}

proof fn numeric_token_regression() {
    let s = ".5e2PX"@;
    reveal_with_fuel(run, 4);
    reveal_with_fuel(name, 4);
    reveal_with_fuel(digits_value, 4);
    reveal_with_fuel(pow, 4);
    assert(s.subrange(1, 2) =~= seq!['5']);
    assert(s.subrange(3, 4) =~= seq!['2']);
    assert(s.subrange(0, 0) =~= Seq::empty());
    assert(run(s, 0, |c: char| digit(c)) == 0);
    assert(run(s, 1, |c: char| digit(c)) == 2);
    assert(run(s, 3, |c: char| digit(c)) == 4);
    assert(digits_value(s.subrange(1, 2), 10) == 5);
    assert(digits_value(s.subrange(3, 4), 10) == 2);
    assert(pow(10, 1) == 10);
    assert(power_ten(-1) == 1real / 10real);
    assert(power_ten(2) == 100real);
    assert(name(s, 4).0 =~= "PX"@);
    assert(name(s, 4).1 == 6);
    assert(ascii_lowercase("PX"@) =~= "px"@);
    assert(5real * power_ten(-1) * power_ten(2) == 50real) by (nonlinear_arith)
        requires
            power_ten(-1) == 1real / 10real,
            power_ten(2) == 100real,
    ;
    assert(starts_number(s, 0) && starts_ident(s, 4));
    assert(numeric(s, 0) == (Lex::Atom(Token::Dimension(50real, "px"@)), 6int));
    assert(lex(s, 0) == (Lex::Atom(Token::Dimension(50real, "px"@)), 6int));
}

} // verus!
