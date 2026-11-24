/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! CSS Values 4 math functions and dimensional type checking. Intermediate
//! dimensions can cancel (e.g. 1in / 1px); a final feature still checks its type.
//! Percentages in media queries have no length basis. No percentage hint is applied.
//! <https://drafts.csswg.org/css-values-4/#calc-type-checking>
use super::device::MediaDeviceView;
use super::math::{self, Num};
use super::tokens::Token;
use super::values::{length_px, trim};
use vstd::prelude::*;
verus! {

// Exponents of length, angle, time, frequency, resolution, flex, percentage.
pub(super) type Dim = (int, int, int, int, int, int, int);

type Typed = (Dim, Num);

pub(super) open spec fn scalar() -> Dim {
    (0, 0, 0, 0, 0, 0, 0)
}

pub(super) open spec fn length() -> Dim {
    (1, 0, 0, 0, 0, 0, 0)
}

pub(super) open spec fn resolution() -> Dim {
    (0, 0, 0, 0, 1, 0, 0)
}

spec fn angle() -> Dim {
    (0, 1, 0, 0, 0, 0, 0)
}

spec fn product(a: Dim, b: Dim, s: int) -> Dim {
    (
        a.0 + s * b.0,
        a.1 + s * b.1,
        a.2 + s * b.2,
        a.3 + s * b.3,
        a.4 + s * b.4,
        a.5 + s * b.5,
        a.6 + s * b.6,
    )
}

pub(super) closed spec fn dimension(n: real, u: Seq<char>, d: MediaDeviceView) -> Option<Typed> {
    if let Some(k) = length_px(u, d) {
        Some((length(), Num::F(n * k)))
    } else if u == "dppx"@ || u == "x"@ || u == "dpi"@ || u == "dpcm"@ {
        Some(
            (
                resolution(),
                Num::F(
                    if u == "dpi"@ {
                        n / 96real
                    } else if u == "dpcm"@ {
                        n * 254real / 9600real
                    } else {
                        n
                    },
                ),
            ),
        )
    } else if u == "rad"@ || u == "deg"@ || u == "turn"@ || u == "grad"@ {
        Some(
            (
                angle(),
                Num::F(
                    if u == "rad"@ {
                        n
                    } else if u == "deg"@ {
                        n * math::pi() / 180real
                    } else if u == "grad"@ {
                        n * math::pi() / 200real
                    } else {
                        n * 2real * math::pi()
                    },
                ),
            ),
        )
    } else if u == "s"@ || u == "ms"@ {
        Some(
            (
                (0, 0, 1, 0, 0, 0, 0),
                Num::F(
                    if u == "ms"@ {
                        n / 1000real
                    } else {
                        n
                    },
                ),
            ),
        )
    } else if u == "hz"@ || u == "khz"@ {
        Some(
            (
                (0, 0, 0, 1, 0, 0, 0),
                Num::F(
                    if u == "khz"@ {
                        n * 1000real
                    } else {
                        n
                    },
                ),
            ),
        )
    } else if u == "fr"@ {
        Some(((0, 0, 0, 0, 0, 1, 0), Num::F(n)))
    } else {
        None
    }
}

spec fn last_op(t: Seq<Token>, sum: bool) -> Option<int>
    decreases t.len(),
{
    if t.len() == 0 {
        None
    } else {
        let x = t.last();
        if (sum && (x == Token::Delim('+') || x == Token::Delim('-'))) || (!sum && (x
            == Token::Delim('*') || x == Token::Delim('/'))) {
            Some(t.len() - 1)
        } else {
            last_op(t.drop_last(), sum)
        }
    }
}

spec fn arguments(t: Seq<Token>) -> Seq<Seq<Token>>
    decreases t.len(),
{
    match t.index_of_first(Token::Delim(',')) {
        Some(i) if 0 <= i < t.len() => seq![trim(t.take(i))] + arguments(t.skip(i + 1)),
        _ => seq![trim(t)],
    }
}

spec fn fold(name: Seq<char>, a: Seq<Typed>) -> Option<Typed>
    decreases a.len(),
{
    if a.len() == 0 {
        None
    } else {
        let (k, x) = a.first();
        if a.len() == 1 {
            Some(
                (
                    k,
                    if name == "hypot"@ {
                        math::mul(x, x)
                    } else {
                        x
                    },
                ),
            )
        } else {
            match fold(name, a.skip(1)) {
                Some((l, y)) if k == l => Some(
                    (
                        k,
                        if name == "hypot"@ {
                            math::add(math::mul(x, x), y)
                        } else {
                            math::extreme(x, y, name == "min"@)
                        },
                    ),
                ),
                _ => None,
            }
        }
    }
}

spec fn round_value(args: Seq<Typed>, strategy: Seq<char>, d: MediaDeviceView) -> Option<Typed> {
    if !(1 <= args.len() <= 2) {
        None
    } else {
        let (k, a) = args[0];
        if args.len() == 2 && args[1].0 != k {
            None
        } else if strategy == "line-width"@ && k != length() {
            None
        } else if args.len() == 1 && k != scalar() && strategy != "line-width"@ {
            None
        } else if strategy == "line-width"@ {
            let px = d.css_px_per_device_pixel;
            let v = math::finite(a, d.numeric_limit);
            let r = if args.len() == 1 {
                v
            } else {
                let b = args[1].1;
                let rounded = math::rounded(a, b, "nearest"@);
                let q = math::finite(rounded, d.numeric_limit);
                if rounded != Num::Nan && v != 0real && q == 0real {
                    let step = math::finite(math::unary("abs"@, b), d.numeric_limit);
                    if v < 0real {
                        -step
                    } else {
                        step
                    }
                } else {
                    q
                }
            };
            let count = if r < 0real {
                -r / px
            } else {
                r / px
            };
            let snapped = if count > 0real && count < 1real {
                1real
            } else {
                count.floor() as real
            };
            Some(
                (
                    k,
                    Num::F(
                        if r < 0real {
                            -snapped * px
                        } else {
                            snapped * px
                        },
                    ),
                ),
            )
        } else {
            Some(
                (
                    k,
                    math::rounded(
                        a,
                        if args.len() == 2 {
                            args[1].1
                        } else {
                            Num::F(1real)
                        },
                        strategy,
                    ),
                ),
            )
        }
    }
}

spec fn call(n: Seq<char>, a: Seq<Typed>) -> Option<Typed> {
    if a.len() == 0 {
        None
    } else {
        let (k, x) = a[0];
        if n == "min"@ || n == "max"@ || n == "hypot"@ {
            match fold(n, a) {
                Some((l, y)) => Some(
                    (
                        l,
                        if n == "hypot"@ {
                            math::unary("sqrt"@, y)
                        } else {
                            y
                        },
                    ),
                ),
                None => None,
            }
        } else if a.len() == 1 {
            if n == "calc"@ {
                Some((k, x))
            } else if n == "abs"@ || n == "sign"@ {
                Some(
                    (
                        if n == "abs"@ {
                            k
                        } else {
                            scalar()
                        },
                        math::unary(n, x),
                    ),
                )
            } else if (n == "sin"@ || n == "cos"@ || n == "tan"@) && (k == scalar() || k
                == angle()) {
                Some((scalar(), math::unary(n, x)))
            } else if (n == "asin"@ || n == "acos"@ || n == "atan"@) && k == scalar() {
                Some((angle(), math::unary(n, x)))
            } else if (n == "sqrt"@ || n == "log"@ || n == "exp"@) && k == scalar() {
                Some((k, math::unary(n, x)))
            } else {
                None
            }
        } else if a.len() == 2 && a[1].0 == k {
            let y = a[1].1;
            if n == "atan2"@ {
                Some((angle(), math::atan2(x, y)))
            } else if n == "pow"@ && k == scalar() {
                Some((k, math::power(x, y)))
            } else if n == "log"@ && k == scalar() {
                Some((k, math::log_base(x, y)))
            } else if n == "mod"@ || n == "rem"@ {
                Some((k, math::remainder(x, y, n == "mod"@)))
            } else {
                None
            }
        } else {
            None
        }
    }
}

spec fn eval_args(a: Seq<Seq<Token>>, d: MediaDeviceView, fuel: nat) -> Option<Seq<Typed>>
    decreases fuel, 0int,
{
    if a.len() == 0 {
        Some(Seq::empty())
    } else if fuel == 0 {
        None
    } else {
        match (eval(a.first(), d, (fuel - 1) as nat), eval_args(a.skip(1), d, (fuel - 1) as nat)) {
            (Some(x), Some(tail)) => Some(seq![x] + tail),
            _ => None,
        }
    }
}

pub(super) closed spec fn eval(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> Option<Typed>
    decreases fuel, 1int,
{
    let t = trim(t);
    if fuel == 0 || t.len() == 0 {
        None
    } else if t.len() == 1 {
        let f = (fuel - 1) as nat;
        match t.first() {
            Token::Number(n, _) => Some((scalar(), Num::F(n))),
            Token::Dimension(n, u) => dimension(n, u, d),
            Token::Percentage(n) => Some(((0, 0, 0, 0, 0, 0, 1), Num::F(n))),
            Token::Ident(n) => if n == "pi"@ {
                Some((scalar(), Num::F(math::pi())))
            } else if n == "e"@ {
                Some((scalar(), Num::F(math::euler())))
            } else if n == "infinity"@ || n == "-infinity"@ {
                Some((scalar(), Num::Infinity(n == "-infinity"@)))
            } else if n == "nan"@ {
                Some((scalar(), Num::Nan))
            } else {
                None
            },
            Token::Block(')', inner) => eval(inner, d, f),
            Token::Function(name, inner) => {
                let a = arguments(inner);
                if name == "clamp"@ && a.len() == 3 {
                    if let Some((k, mid)) = eval(a[1], d, f) {
                        let none = seq![Token::Ident("none"@)];
                        let lo = if a[0] == none {
                            Some((k, Num::Infinity(true)))
                        } else {
                            eval(a[0], d, f)
                        };
                        let hi = if a[2] == none {
                            Some((k, Num::Infinity(false)))
                        } else {
                            eval(a[2], d, f)
                        };
                        match (lo, hi) {
                            (Some((l, x)), Some((h, y))) if l == k && h == k => Some(
                                (k, math::extreme(x, math::extreme(mid, y, true), false)),
                            ),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else if name == "round"@ && a.len() > 0 {
                    let strategy = match ident_arg(a[0]) {
                        Some(s) if s == "up"@ || s == "down"@ || s == "nearest"@ || s == "to-zero"@
                            || s == "line-width"@ => Some(s),
                        _ => None,
                    };
                    let (s, rest) = match strategy {
                        Some(s) => (s, a.skip(1)),
                        None => ("nearest"@, a),
                    };
                    match eval_args(rest, d, f) {
                        Some(args) => round_value(args, s, d),
                        None => None,
                    }
                } else {
                    match eval_args(a, d, f) {
                        Some(args) => call(name, args),
                        None => None,
                    }
                }
            },
            _ => None,
        }
    } else {
        let op = match last_op(t, true) {
            Some(i) => Some(i),
            None => last_op(t, false),
        };
        match op {
            Some(i) if 0 < i < t.len() - 1 => match t[i] {
                Token::Delim(c) if c != '+' && c != '-' || (t[i - 1] == Token::Space && t[i + 1]
                    == Token::Space) => match (
                    eval(t.take(i), d, (fuel - 1) as nat),
                    eval(t.skip(i + 1), d, (fuel - 1) as nat),
                ) {
                    (Some((k, x)), Some((l, y))) => if c == '*' {
                        Some((product(k, l, 1), math::mul(x, y)))
                    } else if c == '/' {
                        Some((product(k, l, -1), math::div(x, y)))
                    } else if k != l {
                        None
                    } else {
                        Some(
                            (
                                k,
                                math::add(
                                    x,
                                    if c == '-' {
                                        math::neg(y)
                                    } else {
                                        y
                                    },
                                ),
                            ),
                        )
                    },
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }
}

spec fn ident_arg(t: Seq<Token>) -> Option<Seq<char>> {
    if t.len() == 1 {
        match t[0] {
            Token::Ident(n) => Some(n),
            _ => None,
        }
    } else {
        None
    }
}

} // verus!
