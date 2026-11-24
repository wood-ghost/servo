/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! Mathematical semantics of CSS Values 4 calculations, including intermediate
//! signed zero, infinities and NaN. Analytic functions below use mathematical
//! definitions (series limits, roots and principal inverses); the regression
//! proofs do not establish the real-analysis existence and uniqueness theorems.
//! Finite operations use ideal reals. The
//! document supplies the UA's representable limit for a top-level infinity.
//! <https://drafts.csswg.org/css-values-4/#calc-internal>
use vstd::prelude::*;
verus! {

pub(super) enum Num {
    F(real),
    NegativeZero,
    // The sign bit is true for negative infinity.
    Infinity(bool),
    Nan,
}

spec fn abs(x: real) -> real {
    if x < 0real {
        -x
    } else {
        x
    }
}

spec fn zero(x: Num) -> bool {
    x == Num::NegativeZero || x == Num::F(0real)
}

spec fn negative(x: Num) -> bool {
    match x {
        Num::F(n) => n < 0real,
        Num::NegativeZero | Num::Infinity(true) => true,
        _ => false,
    }
}

spec fn signed_zero(neg: bool) -> Num {
    if neg {
        Num::NegativeZero
    } else {
        Num::F(0real)
    }
}

pub(super) closed spec fn neg(x: Num) -> Num {
    match x {
        Num::F(n) => if n == 0real {
            Num::NegativeZero
        } else {
            Num::F(-n)
        },
        Num::NegativeZero => Num::F(0real),
        Num::Infinity(b) => Num::Infinity(!b),
        Num::Nan => Num::Nan,
    }
}

pub(super) closed spec fn add(x: Num, y: Num) -> Num {
    match (x, y) {
        (Num::Nan, _) | (_, Num::Nan) => Num::Nan,
        (Num::Infinity(a), Num::Infinity(b)) if a != b => Num::Nan,
        (Num::Infinity(_), _) => x,
        (_, Num::Infinity(_)) => y,
        (Num::NegativeZero, _) => y,
        (_, Num::NegativeZero) => x,
        (Num::F(a), Num::F(b)) => Num::F(a + b),
    }
}

pub(super) closed spec fn mul(x: Num, y: Num) -> Num {
    let n = negative(x) != negative(y);
    if x == Num::Nan || y == Num::Nan {
        Num::Nan
    } else if x is Infinity || y is Infinity {
        if zero(x) || zero(y) {
            Num::Nan
        } else {
            Num::Infinity(n)
        }
    } else if zero(x) || zero(y) {
        signed_zero(n)
    } else {
        Num::F(x->F_0 * y->F_0)
    }
}

pub(super) closed spec fn div(x: Num, y: Num) -> Num {
    let n = negative(x) != negative(y);
    if x == Num::Nan || y == Num::Nan || (zero(x) && zero(y)) || (x is Infinity && y is Infinity) {
        Num::Nan
    } else if x is Infinity || zero(y) {
        Num::Infinity(n)
    } else if zero(x) || y is Infinity {
        signed_zero(n)
    } else {
        Num::F(x->F_0 / y->F_0)
    }
}

pub(super) closed spec fn extreme(x: Num, y: Num, minimum: bool) -> Num {
    if x == Num::Nan || y == Num::Nan {
        Num::Nan
    } else {
        let lt = if x == y {
            false
        } else if x == Num::Infinity(true) || y == Num::Infinity(false) {
            true
        } else if x is Infinity || y is Infinity {
            false
        } else if x == Num::NegativeZero {
            !negative(y)
        } else if y == Num::NegativeZero {
            negative(x)
        } else {
            x->F_0 < y->F_0
        };
        if lt == minimum {
            x
        } else {
            y
        }
    }
}

pub(super) closed spec fn finite(x: Num, limit: real) -> real {
    match x {
        Num::Infinity(false) => limit,
        Num::Infinity(true) => -limit,
        Num::F(n) => n,
        _ => 0real,
    }
}

// Exact real functions by their mathematical definitions. Each choice specifies
// a unique real on the domain of the caller (not an oracle for CSS evaluation).
spec fn term(x: real, k: nat) -> real
    decreases k,
{
    if k == 0 {
        1real
    } else {
        term(x, (k - 1) as nat) * x / (k as real)
    }
}

spec fn partial(x: real, k: nat, sine: bool) -> real
    decreases k,
{
    let v = if !sine {
        term(x, k)
    } else if k % 2 == 0 {
        0real
    } else if k % 4 == 1 {
        term(x, k)
    } else {
        -term(x, k)
    };
    if k == 0 {
        v
    } else {
        partial(x, (k - 1) as nat, sine) + v
    }
}

spec fn tail_within(x: real, sine: bool, r: real, k: nat, n: nat) -> bool {
    forall|j: nat| j >= n ==> abs(#[trigger] partial(x, j, sine) - r) < 1real / (k as real)
}

spec fn eventually_within(x: real, sine: bool, r: real, k: nat) -> bool {
    exists|n: nat| #[trigger] tail_within(x, sine, r, k, n)
}

spec fn converges(x: real, sine: bool, r: real) -> bool {
    forall|k: nat| k > 0 ==> #[trigger] eventually_within(x, sine, r, k)
}

spec fn series(x: real, sine: bool) -> real {
    choose|r: real| converges(x, sine, r)
}

pub(super) closed spec fn pi() -> real {
    choose|x: real| 3real < x < 4real && series(x, true) == 0real
}

pub(super) closed spec fn euler() -> real {
    series(1real, false)
}

spec fn ln(x: real) -> real {
    choose|r: real| series(r, false) == x
}

spec fn square(x: real) -> real {
    x * x
}

spec fn root(x: real) -> real {
    choose|r: real| r >= 0real && #[trigger] square(r) == x
}

spec fn asin_real(x: real) -> real {
    choose|r: real| -pi() / 2real <= r <= pi() / 2real && series(r, true) == x
}

spec fn atan_real(x: real) -> real {
    asin_real(x / root(1real + x * x))
}

pub(super) closed spec fn unary(name: Seq<char>, x: Num) -> Num {
    if x == Num::Nan {
        Num::Nan
    } else if name == "abs"@ {
        if negative(x) {
            neg(x)
        } else {
            x
        }
    } else if name == "sign"@ {
        if zero(x) {
            x
        } else {
            Num::F(
                if negative(x) {
                    -1real
                } else {
                    1real
                },
            )
        }
    } else if name == "exp"@ {
        match x {
            Num::Infinity(true) => Num::F(0real),
            Num::Infinity(false) => x,
            Num::F(n) => Num::F(series(n, false)),
            _ => Num::F(1real),
        }
    } else if name == "log"@ {
        if zero(x) {
            Num::Infinity(true)
        } else if negative(x) {
            Num::Nan
        } else if x is Infinity {
            x
        } else {
            Num::F(ln(x->F_0))
        }
    } else if name == "sqrt"@ {
        if zero(x) {
            x
        } else if negative(x) {
            Num::Nan
        } else if x is Infinity {
            x
        } else {
            Num::F(root(x->F_0))
        }
    } else if name == "atan"@ {
        if zero(x) {
            x
        } else if x is Infinity {
            Num::F(
                if negative(x) {
                    -pi() / 2real
                } else {
                    pi() / 2real
                },
            )
        } else {
            Num::F(atan_real(x->F_0))
        }
    } else if x is Infinity {
        Num::Nan
    } else {
        let n = finite(x, 0real);
        if name == "asin"@ || name == "acos"@ {
            if abs(n) > 1real {
                Num::Nan
            } else if name == "asin"@ {
                if zero(x) {
                    x
                } else {
                    Num::F(asin_real(n))
                }
            } else {
                Num::F(pi() / 2real - asin_real(n))
            }
        } else if name == "sin"@ {
            if zero(x) {
                x
            } else {
                Num::F(series(n, true))
            }
        } else if name == "cos"@ {
            Num::F(series(n + pi() / 2real, true))
        } else if name == "tan"@ {
            if zero(x) {
                x
            } else {
                div(Num::F(series(n, true)), Num::F(series(n + pi() / 2real, true)))
            }
        } else {
            Num::Nan
        }
    }
}

pub(super) closed spec fn log_base(x: Num, b: Num) -> Num {
    if b == Num::Nan || b == Num::F(1real) || negative(b) && !zero(b) {
        Num::Nan
    } else if x == Num::F(1real) {
        Num::F(0real)
    } else if zero(x) {
        Num::Infinity(true)
    } else if x == Num::Infinity(false) {
        x
    } else {
        div(unary("log"@, x), unary("log"@, b))
    }
}

pub(super) closed spec fn power(a: Num, b: Num) -> Num {
    if a == Num::Nan || b == Num::Nan {
        Num::Nan
    } else if zero(b) {
        Num::F(1real)
    } else if b is Infinity {
        if a is F && abs(a->F_0) == 1real {
            Num::Nan
        } else if (a is Infinity || a is F && abs(a->F_0) > 1real) != negative(b) {
            Num::Infinity(false)
        } else {
            Num::F(0real)
        }
    } else {
        let y = b->F_0;
        let odd = y == y.floor() as real && y.floor() % 2 == 1;
        let n = negative(a) && odd;
        if zero(a) || a is Infinity {
            if zero(a) == (y < 0real) {
                Num::Infinity(n)
            } else {
                signed_zero(n)
            }
        } else if negative(a) && y != y.floor() as real {
            Num::Nan
        } else {
            let r = series(y * ln(abs(a->F_0)), false);
            Num::F(
                if n {
                    -r
                } else {
                    r
                },
            )
        }
    }
}

pub(super) closed spec fn atan2(y: Num, x: Num) -> Num {
    if x == Num::Nan || y == Num::Nan {
        Num::Nan
    } else {
        let a = if x is Infinity && y is Infinity {
            if negative(x) {
                3real * pi() / 4real
            } else {
                pi() / 4real
            }
        } else if zero(y) || x is Infinity {
            if negative(x) {
                pi()
            } else {
                0real
            }
        } else if zero(x) || y is Infinity {
            pi() / 2real
        } else {
            let v = atan_real(abs(y->F_0 / x->F_0));
            if negative(x) {
                pi() - v
            } else {
                v
            }
        };
        if negative(y) {
            neg(Num::F(a))
        } else {
            Num::F(a)
        }
    }
}

// nearest/up/down/to-zero round to a multiple of the (absolute) step.
pub(super) closed spec fn rounded(a: Num, b: Num, strategy: Seq<char>) -> Num {
    if zero(b) || a == Num::Nan || b == Num::Nan || a is Infinity && b is Infinity {
        Num::Nan
    } else if a is Infinity || zero(a) {
        a
    } else if b is Infinity {
        if strategy == "down"@ && negative(a) {
            Num::Infinity(true)
        } else if strategy == "up"@ && !negative(a) {
            Num::Infinity(false)
        } else {
            signed_zero(negative(a))
        }
    } else {
        let v = a->F_0;
        let step = abs(b->F_0);
        let q = v / step;
        let down = q.floor() as real;
        let result = if down == q || strategy == "down"@ || strategy == "to-zero"@ && v > 0real {
            down
        } else if strategy == "up"@ || strategy == "to-zero"@ {
            down + 1real
        } else {
            (q + 0.5real).floor() as real
        };
        if result == 0real {
            signed_zero(v < 0real)
        } else {
            Num::F(result * step)
        }
    }
}

pub(super) closed spec fn remainder(a: Num, b: Num, modulus: bool) -> Num {
    if a == Num::Nan || b == Num::Nan || zero(b) || a is Infinity {
        Num::Nan
    } else if b is Infinity {
        if modulus && negative(a) != negative(b) {
            Num::Nan
        } else {
            a
        }
    } else {
        let n = finite(a, 0real);
        let step = b->F_0;
        let q = n / step;
        let v = n - step * (if modulus || q >= 0real {
            q.floor()
        } else {
            -(-q).floor()
        }) as real;
        if v == 0real {
            signed_zero(
                if modulus {
                    negative(b)
                } else {
                    negative(a)
                },
            )
        } else {
            Num::F(v)
        }
    }
}

proof fn arithmetic_regressions() {
    assert("nearest"@ != "up"@ && "nearest"@ != "down"@ && "nearest"@ != "to-zero"@);
    assert((2.5real + 0.5real).floor() == 3);
    assert(rounded(Num::F(2.5real), Num::F(1real), "nearest"@) == Num::F(3real));
    assert(remainder(Num::F(-3real), Num::F(2real), true) == Num::F(1real));
    assert(remainder(Num::F(-3real), Num::F(2real), false) == Num::F(-1real));
    assert(div(Num::F(1real), mul(Num::F(-1real), Num::F(0real))) == Num::Infinity(true));
    assert(power(Num::Nan, Num::F(0real)) == Num::Nan);
}

} // verus!
