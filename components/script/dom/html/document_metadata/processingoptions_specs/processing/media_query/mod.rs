/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! CSS Media Queries Level 4, sections 2–3: parse a list of component values,
//! recover at top-level commas, and evaluate media types and nested conditions.
//! <https://drafts.csswg.org/mediaqueries-4/#mq-syntax>
//! Feature coverage follows the tables in sections 4–7 and Appendix A of
//! <https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/>. Numeric evaluation
//! uses mathematical reals (see `values` and `math`); correspondence to
//! Stylo's device snapshots and its finite precision is a separate obligation.
#![verifier::auto_reveal_strlit]

mod calculation;
pub mod device;
mod features;
mod math;
mod tokens;
mod values;
pub use self::device::MediaDeviceView;
use self::tokens::{Token, tokenize};
use vstd::prelude::*;

verus! {

// CSS three-valued logic. Syntactically invalid input is a separate case:
// `not unknown-type` matches, but negating an invalid query does not repair it.
enum Truth {
    Yes,
    No,
    Unknown,
}

spec fn truth(b: bool) -> Truth {
    if b {
        Truth::Yes
    } else {
        Truth::No
    }
}

spec fn negate(a: Truth) -> Truth {
    match a {
        Truth::Yes => Truth::No,
        Truth::No => Truth::Yes,
        Truth::Unknown => Truth::Unknown,
    }
}

spec fn and(a: Truth, b: Truth) -> Truth {
    if a == Truth::No || b == Truth::No {
        Truth::No
    } else if a == b {
        a
    } else {
        Truth::Unknown
    }
}

spec fn or(a: Truth, b: Truth) -> Truth {
    negate(and(negate(a), negate(b)))
}

spec fn without_spaces(t: Seq<Token>) -> Seq<Token> {
    t.filter(|x: Token| x != Token::Space)
}

// Unmatched closers, bad-string, and bad-url tokens cannot occur in <any-value>.
spec fn any_value(t: Seq<Token>, fuel: nat) -> bool
    decreases fuel,
{
    t.len() == 0 || fuel > 0 && {
        let ok = match t.first() {
            Token::Bad => false,
            Token::Function(_, inner) | Token::Block(_, inner) => any_value(
                inner,
                (fuel - 1) as nat,
            ),
            _ => true,
        };
        ok && any_value(t.skip(1), (fuel - 1) as nat)
    }
}

// One level consumes at least one component (or a connective). The initial
// budget below exceeds three calls per source code point; no fixed depth cap.
spec fn atom(t: Token, d: MediaDeviceView, fuel: nat) -> Option<Truth>
    decreases fuel, 1int,
{
    if fuel == 0 {
        None
    } else {
        match t {
            Token::Block(')', inner) => match condition(
                without_spaces(inner),
                true,
                d,
                (fuel - 1) as nat,
            ) {
                Some(v) => Some(v),
                None => match features::feature(inner, d, fuel) {
                    Some(b) => Some(truth(b)),
                    None => if any_value(inner, fuel) {
                        Some(Truth::Unknown)
                    } else {
                        None
                    },
                },
            },
            Token::Function(_, inner) if any_value(inner, fuel) => Some(Truth::Unknown),
            _ => None,
        }
    }
}

spec fn chain(t: Seq<Token>, conjunction: bool, d: MediaDeviceView, fuel: nat) -> Option<Truth>
    decreases fuel, 2int,
{
    if fuel == 0 || t.len() == 0 {
        None
    } else {
        match atom(t.first(), d, fuel) {
            None => None,
            Some(a) => if t.len() == 1 {
                Some(a)
            } else if t.len() < 3 || t[1] != Token::Ident(
                if conjunction {
                    "and"@
                } else {
                    "or"@
                },
            ) {
                None
            } else {
                match chain(t.skip(2), conjunction, d, (fuel - 1) as nat) {
                    None => None,
                    Some(b) => Some(
                        if conjunction {
                            and(a, b)
                        } else {
                            or(a, b)
                        },
                    ),
                }
            },
        }
    }
}

spec fn condition(t: Seq<Token>, allow_or: bool, d: MediaDeviceView, fuel: nat) -> Option<Truth>
    decreases fuel, 3int,
{
    if fuel == 0 || t.len() == 0 {
        None
    } else if t.first() == Token::Ident("not"@) {
        if t.len() == 2 {
            match atom(t[1], d, fuel) {
                Some(v) => Some(negate(v)),
                None => None,
            }
        } else {
            None
        }
    } else if t.len() > 1 && t[1] == Token::Ident("or"@) {
        if allow_or {
            chain(t, false, d, fuel)
        } else {
            None
        }
    } else {
        chain(t, true, d, fuel)
    }
}

spec fn query(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> bool {
    let t = without_spaces(t);
    let neg = t.len() > 0 && t[0] == Token::Ident("not"@);
    // A leading `not` before a block/function starts a condition, not a media type.
    if t.len() > 0 && (!(t[0] is Ident) || neg && t.len() > 1 && !(t[1] is Ident)) {
        condition(t, true, d, fuel) == Some(Truth::Yes)
    } else {
        let i = if neg || (t.len() > 0 && t[0] == Token::Ident("only"@)) {
            1int
        } else {
            0int
        };
        if i >= t.len() {
            false
        } else {
            match t[i] {
                Token::Ident(n) if n != "only"@ && n != "not"@ && n != "and"@ && n != "or"@ && n
                    != "layer"@ => {
                    let ty = truth(
                        n == "all"@ || (n == "print"@ && d.print) || (n == "screen"@ && !d.print),
                    );
                    let tail = if i + 1 == t.len() {
                        Some(Truth::Yes)
                    } else if t[i + 1] == Token::Ident("and"@) {
                        condition(t.skip(i + 2), false, d, fuel)
                    } else {
                        None
                    };
                    match tail {
                        None => false,
                        Some(v) => {
                            let result = and(ty, v);
                            (if neg {
                                negate(result)
                            } else {
                                result
                            }) == Truth::Yes
                        },
                    }
                },
                _ => false,
            }
        }
    }
}

spec fn member_matches(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> bool
    decreases t.len(),
{
    match t.index_of_first(Token::Delim(',')) {
        Some(i) if 0 <= i < t.len() => query(t.take(i), d, fuel) || member_matches(
            t.skip(i + 1),
            d,
            fuel,
        ),
        _ => query(t, d, fuel),
    }
}

pub closed spec fn matches(media: Seq<char>, d: MediaDeviceView) -> bool {
    let t = tokenize(media);
    without_spaces(t).len() == 0 || member_matches(t, d, 3 * media.len() + 3)
}

// Regression checks on the matching rules. Unknown must survive negation, unlike
// an unrecognised media *type*, which is a valid type whose value is false.
proof fn mq4_conditions(d: MediaDeviceView) {
    reveal_with_fuel(Seq::filter, 4);
    let all = Token::Ident("all"@);
    assert(without_spaces(seq![all]) =~= seq![all]);
    let screen = Token::Ident("screen"@);
    assert(without_spaces(seq![screen]) =~= seq![screen]);
    let not_unknown = seq![Token::Ident("not"@), Token::Ident("unknown-type"@)];
    assert(without_spaces(not_unknown) =~= not_unknown);
    assert(query(seq![all], d, 8));
    tokens::escaped_all_token();
    seq![all].index_of_first_ensures(Token::Delim(','));
    assert(matches("\\61 LL"@, d));
    assert(query(seq![Token::Ident("screen"@)], d, 8) == !d.print);
    assert(query(seq![Token::Ident("not"@), Token::Ident("unknown-type"@)], d, 8));
    assert(negate(Truth::Unknown) == Truth::Unknown);
    assert(or(Truth::Unknown, Truth::Yes) == Truth::Yes);
    assert(and(Truth::Unknown, Truth::No) == Truth::No);
    // An invalid member does not discard a valid following query.
    let invalid = Token::Delim('!');
    let list = seq![invalid, Token::Delim(','), all];
    list.index_of_first_ensures(Token::Delim(','));
    assert(list[1] == Token::Delim(','));
    assert(list.index_of_first(Token::Delim(',')) == Some(1int));
    assert(list.skip(2) =~= seq![all]);
    reveal_with_fuel(member_matches, 2);
    assert(member_matches(list, d, 8));
}

proof fn nested_unknown_or_grid(d: MediaDeviceView) {
    // future(a,b) is one general-enclosed component; its comma is not a list delimiter.
    let future = Token::Function("future"@, seq![Token::Delim(',')]);
    let grid = seq![Token::Ident("grid"@)];
    let block = Token::Block(')', grid);
    let mixed = seq![future, Token::Ident("or"@), block];
    reveal_with_fuel(Seq::filter, 4);
    assert(without_spaces(grid) =~= grid);
    assert(without_spaces(mixed) =~= mixed);
    assert(mixed.skip(2) =~= seq![block]);
    reveal_with_fuel(any_value, 2);
    reveal_with_fuel(chain, 3);
    reveal_with_fuel(atom, 3);
    reveal_with_fuel(condition, 3);
    features::boolean_grid(d, 11);
    assert(atom(grid.first(), d, 10) == None);
    assert(condition(grid, true, d, 10) == None);
    assert(atom(block, d, 11) == Some(truth(d.grid)));
    assert(atom(future, d, 12) == Some(Truth::Unknown));
    assert(chain(seq![block], false, d, 11) == Some(truth(d.grid)));
    assert(condition(mixed, true, d, 12) == Some(or(Truth::Unknown, truth(d.grid))));
    assert(query(mixed, d, 12) == d.grid);
}

} // verus!
