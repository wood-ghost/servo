/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! CSS units and the numeric types accepted as MQ4 feature values.
//! <https://drafts.csswg.org/css-values-4/#numeric-types>
use super::calculation;
use super::device::MediaDeviceView;
use super::math;
use super::tokens::Token;
use vstd::prelude::*;
verus! {

pub(super) enum Kind {
    Integer,
    Number,
    Length,
    Resolution,
}

pub(super) struct Value {
    pub kind: Kind,
    pub n: real,
}

pub(super) open spec fn trim(t: Seq<Token>) -> Seq<Token>
    decreases t.len(),
{
    if t.len() == 0 {
        t
    } else if t.first() == Token::Space {
        trim(t.skip(1))
    } else if t.last() == Token::Space {
        trim(t.drop_last())
    } else {
        t
    }
}

pub(super) closed spec fn length_px(u: Seq<char>, d: MediaDeviceView) -> Option<real> {
    let wh = if "sv"@.is_prefix_of(u) || "cq"@.is_prefix_of(u) {
        d.small_viewport_px
    } else if "dv"@.is_prefix_of(u) {
        d.dynamic_viewport_px
    } else {
        d.large_viewport_px
    };
    let vp = if "sv"@.is_prefix_of(u) || "lv"@.is_prefix_of(u) || "dv"@.is_prefix_of(u)
        || "cq"@.is_prefix_of(u) {
        u.skip(2)
    } else if u.len() > 0 && u[0] == 'v' {
        u.skip(1)
    } else {
        Seq::empty()
    };
    if u == "px"@ {
        Some(1real)
    } else if u == "in"@ {
        Some(96real)
    } else if u == "cm"@ {
        Some(9600real / 254real)
    } else if u == "mm"@ {
        Some(960real / 254real)
    } else if u == "q"@ {
        Some(240real / 254real)
    } else if u == "pt"@ {
        Some(96real / 72real)
    } else if u == "pc"@ {
        Some(16real)
    } else if u == "em"@ || u == "rem"@ {
        Some(d.em_px)
    } else if u == "ex"@ || u == "rex"@ {
        Some(d.ex_px)
    } else if u == "ch"@ || u == "rch"@ {
        Some(d.ch_px)
    } else if u == "ic"@ || u == "ric"@ {
        Some(d.ic_px)
    } else if u == "cap"@ || u == "rcap"@ {
        Some(d.cap_px)
    } else if u == "lh"@ || u == "rlh"@ {
        Some(d.lh_px)
    } else if vp == "w"@ || vp == "i"@ && !d.vertical_writing || vp == "b"@ && d.vertical_writing {
        Some(wh.0 / 100real)
    } else if vp == "h"@ || vp == "b"@ && !d.vertical_writing || vp == "i"@ && d.vertical_writing {
        Some(wh.1 / 100real)
    } else if vp == "min"@ {
        Some(
            (if wh.0 < wh.1 {
                wh.0
            } else {
                wh.1
            }) / 100real,
        )
    } else if vp == "max"@ {
        Some(
            (if wh.0 > wh.1 {
                wh.0
            } else {
                wh.1
            }) / 100real,
        )
    } else {
        None
    }
}

pub(super) closed spec fn numeric_value(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> Option<
    Value,
> {
    let t = trim(t);
    if t.len() != 1 {
        None
    } else {
        match t[0] {
            Token::Number(n, integer) => Some(
                Value {
                    n,
                    kind: if integer {
                        Kind::Integer
                    } else {
                        Kind::Number
                    },
                },
            ),
            Token::Dimension(n, u) => if let Some(px) = length_px(u, d) {
                Some(Value { n: n * px, kind: Kind::Length })
            } else {
                match calculation::dimension(n, u, d) {
                    Some((dim, x)) if dim == calculation::resolution() => Some(
                        Value { kind: Kind::Resolution, n: math::finite(x, d.numeric_limit) },
                    ),
                    _ => None,
                }
            },
            Token::Function(_, _) => match calculation::eval(t, d, fuel) {
                Some((dim, x)) if dim == calculation::scalar() || dim == calculation::length()
                    || dim == calculation::resolution() => Some(
                    Value {
                        kind: if dim == calculation::scalar() {
                            Kind::Number
                        } else if dim == calculation::length() {
                            Kind::Length
                        } else {
                            Kind::Resolution
                        },
                        n: math::finite(x, d.numeric_limit),
                    },
                ),
                _ => None,
            },
            _ => None,
        }
    }
}

pub(super) closed spec fn integer_value(t: Seq<Token>, d: MediaDeviceView, fuel: nat) -> Option<
    int,
> {
    let t = trim(t);
    match numeric_value(t, d, fuel) {
        Some(v) if v.kind == Kind::Integer || v.kind == Kind::Number && t.len() == 1
            && t[0] is Function => Some((v.n + 0.5real).floor()),
        _ => None,
    }
}

pub(super) proof fn literal_px(n: real, d: MediaDeviceView)
    ensures
        numeric_value(seq![Token::Dimension(n, "px"@)], d, 12) == Some(
            Value { kind: Kind::Length, n },
        ),
{
}

} // verus!
