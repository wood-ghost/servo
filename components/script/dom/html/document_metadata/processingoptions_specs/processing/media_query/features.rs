/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! MQ4 sections 4–7 and Appendix A. None is an unknown feature/expression,
//! not a known feature which is false on this device.
use super::device::*;
use super::tokens::Token;
use super::values::{Kind, integer_value, numeric_value, trim};
use vstd::prelude::*;
verus! {

enum Domain {
    Length,
    Ratio,
    Resolution,
    Integer,
}

enum Magnitude {
    Finite(real),
    Infinite,
}

enum Op {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}

spec fn reverse(op: Op) -> Op {
    match op {
        Op::Lt => Op::Gt,
        Op::Le => Op::Ge,
        Op::Eq => Op::Eq,
        Op::Ge => Op::Le,
        Op::Gt => Op::Lt,
    }
}

spec fn compare(a: Magnitude, op: Op, b: Magnitude) -> bool {
    let lt = match (a, b) {
        (Magnitude::Finite(x), Magnitude::Finite(y)) => x < y,
        (Magnitude::Finite(_), Magnitude::Infinite) => true,
        _ => false,
    };
    match op {
        Op::Lt => lt,
        Op::Le => lt || a == b,
        Op::Eq => a == b,
        Op::Ge => !lt,
        Op::Gt => !lt && a != b,
    }
}

spec fn ratio(a: real, b: real) -> Magnitude {
    if b == 0real {
        Magnitude::Infinite
    } else {
        Magnitude::Finite(a / b)
    }
}

spec fn range_feature(n: Seq<char>, d: MediaDeviceView) -> Option<(Domain, Option<Magnitude>)> {
    let device = n == "device-width"@ || n == "device-height"@ || n == "device-aspect-ratio"@;
    let wh = if device {
        d.device_size
    } else {
        d.viewport
    };
    if n == "width"@ || n == "device-width"@ || n == "height"@ || n == "device-height"@ {
        Some(
            (
                Domain::Length,
                match wh {
                    None => None,
                    Some((w, h)) => Some(
                        Magnitude::Finite(
                            if n == "width"@ || n == "device-width"@ {
                                w
                            } else {
                                h
                            },
                        ),
                    ),
                },
            ),
        )
    } else if n == "aspect-ratio"@ || n == "device-aspect-ratio"@ {
        Some(
            (
                Domain::Ratio,
                match wh {
                    None => None,
                    Some((w, h)) => Some(ratio(w, h)),
                },
            ),
        )
    } else if n == "resolution"@ {
        Some(
            (
                Domain::Resolution,
                match d.resolution {
                    None => None,
                    Some(Resolution::Dppx(x)) => Some(Magnitude::Finite(x)),
                    Some(Resolution::Infinite) => Some(Magnitude::Infinite),
                },
            ),
        )
    } else if n == "color"@ || n == "color-index"@ || n == "monochrome"@ {
        Some(
            (
                Domain::Integer,
                Some(
                    Magnitude::Finite(
                        (if n == "color"@ {
                            d.color_bits
                        } else if n == "color-index"@ {
                            d.color_index
                        } else {
                            d.monochrome_bits
                        }) as real,
                    ),
                ),
            ),
        )
    } else {
        None
    }
}

// CSS range restrictions reject out-of-range literals but clamp math-function
// results at computed-value time. Each ratio component is a nonnegative number.
spec fn ratio_component(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> Option<real> {
    let t = trim(t);
    match numeric_value(t, d, fuel) {
        Some(x) if x.kind == Kind::Number || x.kind == Kind::Integer => if x.n >= 0real {
            Some(x.n)
        } else if t.len() == 1 && t[0] is Function {
            Some(0real)
        } else {
            None
        },
        _ => None,
    }
}

spec fn magnitude(t: Seq<Token>, kind: Domain, d: MediaDeviceView, fuel: nat) -> Option<Magnitude> {
    let t = trim(t);
    if kind == Domain::Resolution && t == seq![Token::Ident("infinite"@)] {
        Some(Magnitude::Infinite)
    } else if kind == Domain::Integer {
        match integer_value(t, d, fuel) {
            Some(i) => Some(Magnitude::Finite(i as real)),
            None => None,
        }
    } else if kind == Domain::Ratio {
        let slash = t.index_of_first(Token::Delim('/'));
        let (a, b) = match slash {
            Some(i) => (t.take(i), t.skip(i + 1)),
            None => (t, seq![Token::Number(1real, true)]),
        };
        match (ratio_component(a, d, fuel), ratio_component(b, d, fuel)) {
            (Some(x), Some(y)) => Some(ratio(x, y)),
            _ => None,
        }
    } else {
        match numeric_value(t, d, fuel) {
            Some(v) if (kind == Domain::Resolution && v.kind == Kind::Resolution) || (kind
                == Domain::Length && (v.kind == Kind::Length || t.len() == 1 && t[0] is Number
                && v.n == 0real)) => Some(Magnitude::Finite(v.n)),
            _ => None,
        }
    }
}

// Without a keyword this is the boolean context (any matching value other than none/zero).
spec fn discrete(n: Seq<char>, k: Option<Seq<char>>, d: MediaDeviceView) -> Option<bool> {
    let keyword = match k {
        Some(k) => k,
        None => Seq::empty(),
    };
    let any = k is None;
    if n == "orientation"@ && (any || keyword == "portrait"@ || keyword == "landscape"@) {
        Some(
            match d.viewport {
                None => false,
                Some((w, h)) => any || (keyword == "portrait"@) == (h >= w),
            },
        )
    } else if n == "scan"@ && (any || keyword == "progressive"@ || keyword == "interlace"@) {
        Some(
            match d.scan {
                None => false,
                Some(scan) => any || (keyword == "progressive"@) == (scan == Scan::Progressive),
            },
        )
    } else if n == "update"@ && (any || keyword == "none"@ || keyword == "slow"@ || keyword
        == "fast"@) {
        Some(
            if any {
                d.update != Update::None
            } else {
                keyword == match d.update {
                    Update::None => "none"@,
                    Update::Slow => "slow"@,
                    Update::Fast => "fast"@,
                }
            },
        )
    } else if n == "overflow-block"@ && (any || keyword == "none"@ || keyword == "scroll"@
        || keyword == "paged"@) {
        Some(
            if any {
                d.overflow_block != Overflow::None
            } else {
                keyword == match d.overflow_block {
                    Overflow::None => "none"@,
                    Overflow::Scroll => "scroll"@,
                    Overflow::Paged => "paged"@,
                }
            },
        )
    } else if n == "overflow-inline"@ && (any || keyword == "none"@ || keyword == "scroll"@) {
        Some(
            if any || keyword == "scroll"@ {
                d.inline_scroll
            } else {
                !d.inline_scroll
            },
        )
    } else if n == "color-gamut"@ && (any || keyword == "srgb"@ || keyword == "p3"@ || keyword
        == "rec2020"@) {
        Some(
            if any || keyword == "srgb"@ {
                d.gamut != Gamut::BelowSrgb
            } else if keyword == "p3"@ {
                d.gamut == Gamut::P3 || d.gamut == Gamut::Rec2020
            } else {
                d.gamut == Gamut::Rec2020
            },
        )
    } else if (n == "pointer"@ || n == "any-pointer"@) && (any || keyword == "none"@ || keyword
        == "fine"@ || keyword == "coarse"@) {
        let fine = d.pointer == Pointer::Fine || (n == "any-pointer"@ && d.additional_fine);
        let coarse = d.pointer == Pointer::Coarse || (n == "any-pointer"@ && d.additional_coarse);
        Some(
            if any {
                fine || coarse
            } else if keyword == "fine"@ {
                fine
            } else if keyword == "coarse"@ {
                coarse
            } else {
                !fine && !coarse
            },
        )
    } else if (n == "hover"@ || n == "any-hover"@) && (any || keyword == "none"@ || keyword
        == "hover"@) {
        let hover = d.hover || (n == "any-hover"@ && d.additional_hover);
        Some(
            if any || keyword == "hover"@ {
                hover
            } else {
                !hover
            },
        )
    } else {
        None
    }
}

spec fn ident(t: Seq<Token>) -> Option<Seq<char>> {
    let t = trim(t);
    if t.len() == 1 {
        match t[0] {
            Token::Ident(id) => Some(id),
            _ => None,
        }
    } else {
        None
    }
}

spec fn range_test(n: Seq<char>, op: Op, t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> Option<
    bool,
> {
    match range_feature(n, d) {
        Some((kind, actual)) => match magnitude(t, kind, d, fuel) {
            Some(v) => Some(
                match actual {
                    Some(a) => compare(a, op, v),
                    None => false,
                },
            ),
            None => None,
        },
        None => None,
    }
}

spec fn first_cmp(t: Seq<Token>, i: int) -> Option<(int, Op, int)>
    decreases t.len() - i,
{
    if !(0 <= i < t.len()) {
        None
    } else {
        let eq = i + 1 < t.len() && t[i + 1] == Token::Delim('=');
        match t[i] {
            Token::Delim('<') => Some(
                (
                    i,
                    if eq {
                        Op::Le
                    } else {
                        Op::Lt
                    },
                    if eq {
                        i + 2
                    } else {
                        i + 1
                    },
                ),
            ),
            Token::Delim('>') => Some(
                (
                    i,
                    if eq {
                        Op::Ge
                    } else {
                        Op::Gt
                    },
                    if eq {
                        i + 2
                    } else {
                        i + 1
                    },
                ),
            ),
            Token::Delim('=') => Some((i, Op::Eq, i + 1)),
            _ => first_cmp(t, i + 1),
        }
    }
}

pub(super) closed spec fn feature(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> Option<bool> {
    if let Some(n) = ident(t) {
        match range_feature(n, d) {
            Some((_, x)) => Some(
                match x {
                    Some(v) => compare(v, Op::Gt, Magnitude::Finite(0real)),
                    None => false,
                },
            ),
            None if n == "grid"@ => Some(d.grid),
            None => discrete(n, None, d),
        }
    } else if let Some(i) = t.index_of_first(Token::Delim(':')) {
        if let Some(n) = ident(t.take(i)) {
            let val = trim(t.skip(i + 1));
            if n == "grid"@ {
                match integer_value(val, d, fuel) {
                    Some(v) if v == 0 || v == 1 => Some(d.grid == (v == 1)),
                    Some(v) if val.len() == 1 && val[0] is Function => Some(d.grid == (v > 0)),
                    _ => None,
                }
            } else if "min-"@.is_prefix_of(n) || "max-"@.is_prefix_of(n) {
                range_test(
                    n.skip(4),
                    if n[1] == 'i' {
                        Op::Ge
                    } else {
                        Op::Le
                    },
                    val,
                    d,
                    fuel,
                )
            } else if range_feature(n, d) is Some {
                range_test(n, Op::Eq, val, d, fuel)
            } else if let Some(k) = ident(val) {
                discrete(n, Some(k), d)
            } else {
                None
            }
        } else {
            None
        }
    } else if let Some((i, op, j)) = first_cmp(t, 0) {
        let left = t.take(i);
        if let Some((k, op2, l)) = first_cmp(t, j) {
            let inc = op == Op::Lt || op == Op::Le;
            let dec = op == Op::Gt || op == Op::Ge;
            if (inc && (op2 == Op::Lt || op2 == Op::Le)) || (dec && (op2 == Op::Gt || op2
                == Op::Ge)) {
                if let Some(n) = ident(t.subrange(j, k)) {
                    match (
                        range_test(n, reverse(op), left, d, fuel),
                        range_test(n, op2, t.skip(l), d, fuel),
                    ) {
                        (Some(a), Some(b)) => Some(a && b),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let right = t.skip(j);
            let lhs = match ident(left) {
                Some(n) => range_test(n, op, right, d, fuel),
                None => None,
            };
            if lhs is Some {
                lhs
            } else {
                match ident(right) {
                    Some(n) => range_test(n, reverse(op), left, d, fuel),
                    None => None,
                }
            }
        }
    } else {
        None
    }
}

// Equality at the upper end and strictness at the lower end of a chained range.
proof fn width_range_regression(d: MediaDeviceView, w: real, h: real)
    requires
        d.viewport == Some((w, h)),
{
    let width = Token::Ident("width"@);
    let px = Token::Dimension(600real, "px"@);
    let t = seq![
        px,
        Token::Delim('<'),
        width,
        Token::Delim('<'),
        Token::Delim('='),
        Token::Dimension(900real, "px"@),
    ];
    t.index_of_first_ensures(Token::Delim(':'));
    assert forall|j: int| 0 <= j < t.len() implies #[trigger] t[j] != Token::Delim(':') by {};
    reveal_with_fuel(first_cmp, 4);
    assert(first_cmp(t, 0) == Some((1int, Op::Lt, 2int)));
    assert(first_cmp(t, 2) == Some((3int, Op::Le, 5int)));
    assert(t.take(1) =~= seq![px]);
    assert(t.subrange(2, 3) =~= seq![width]);
    assert(t.skip(5) =~= seq![Token::Dimension(900real, "px"@)]);
    super::values::literal_px(600real, d);
    super::values::literal_px(900real, d);
    assert(feature(t, d, 12) == Some(600real < w <= 900real));
}

pub(super) proof fn boolean_grid(d: MediaDeviceView, fuel: nat)
    ensures
        feature(seq![Token::Ident("grid"@)], d, fuel) == Some(d.grid),
{
}

} // verus!
