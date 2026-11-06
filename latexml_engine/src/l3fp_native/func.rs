//! l3fp-expo and l3fp-trig, block for block (expl3-code.tex:19706-21376). None of these is correctly rounded: each
//! Taylor loop, table and truncation below is the TeX's, because a numerically better answer is a wrong one here. A
//! comment names the macro a piece ports; the blocks are [`Fixed`]. What the TeX reports (`ln(0)`, `asin(2)`, an
//! overflow) is a [`Decline`], for the raw macro.

use super::{
  fixed::{
    self, Ep, Fixed, ONE_FIXED, concat, ep_div, ep_mul, ep_mul_raw, ep_to_ep, ep_to_fixed, mul,
    mul_sub_back, rdiv, sub,
  },
  num::{Decline, Fp, MINUS_ZERO, Mode, ONE, POW10, R, Sign, ZERO, normal, small_int},
  tables::{EXP_TABLE, TRIG_TABLE},
};

/// Whether an angle is in radians or degrees (`\use_i:nn`, `\use_ii:nn`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Unit {
  /// `sin`
  Rad,
  /// `sind`
  Deg,
}

/// The digits of a 16-digit mantissa.
fn mant_digits(mant: u64) -> [u8; 16] {
  let mut out = [0u8; 16];
  let mut v = mant;
  for slot in out.iter_mut().rev() {
    *slot = (v % 10) as u8;
    v /= 10;
  }
  out
}

/// The first 24 of `digits` (zero-padded) as six blocks (`\__fp_pack_twice_four:wNNNNNNNN` three times).
fn blocks_of(digits: &[u8]) -> Fixed {
  let mut out = [0i64; 6];
  for (i, slot) in out.iter_mut().enumerate() {
    for j in 0..4 {
      *slot = *slot * 10 + i64::from(digits.get(i * 4 + j).copied().unwrap_or(0));
    }
  }
  out
}

/// The blocks of a normal number with two zero blocks more: `#8{0000}{0000}`.
fn fixed_of_mant(mant: u64) -> Fixed {
  let b = super::num::blocks(mant);
  [
    i64::from(b[0]),
    i64::from(b[1]),
    i64::from(b[2]),
    i64::from(b[3]),
    0,
    0,
  ]
}

/// `\__fp_sanitize:Nw` with what `\__fp_ep_to_float_o:wwN` adds: the exponent `base` and the float of `f`.
fn float_of(sign: Sign, base: i64, f: &Fixed) -> R<Fp> {
  let Some((e, mant)) = fixed::to_float(f) else {
    // \__fp_fixed_to_float_zero:w: `- 2 * \c__fp_max_exponent_int`, an underflow's flag
    return Err(Decline("underflow"));
  };
  normal(sign, base + e, mant)
}

/// `\__fp_ep_inv_to_float_o:wwN`
fn inv_float_of(sign: Sign, ep: &Ep) -> R<Fp> {
  let one: Ep = (1, [1000, 0, 0, 0, 0, 0]);
  let (e, f) = ep_div(&one, ep);
  float_of(sign, e, &f)
}

// ---------------------------------------------------------------- exp

/// `\__fp_exp_Taylor_loop:www`: `x` the fraction's fixed.
fn exp_taylor(x: &Fixed) -> Fixed {
  let mut y = *x;
  for n in (2..=10).rev() {
    y = mul(&fixed::add_one(&fixed::div_int(&y, n)), x);
  }
  fixed::add_one(&y)
}

/// `\__fp_exp_large:NwN` from `level` over `digits`, then `\__fp_exp_large_after:wwn` on the rest: the exponent the table's
/// rows add and the mantissa `exp(x)` of the digits.
fn exp_large(level: i64, digits: &[u8]) -> R<(i64, Fixed)> {
  let mut accum = ONE_FIXED;
  let mut exponent = 0;
  let mut pos = 0;
  let mut l = level;
  loop {
    let d = i64::from(digits.get(pos).copied().ok_or(Decline("digits ran out"))?);
    pos += 1;
    if d != 0 {
      // \__fp_exp_intarray:w: item (36*l + 4*d) - 3 is the exponent, the next three the mantissa
      let row = (9 * l + d - 1) as usize;
      let [e, a, b, c] = *EXP_TABLE.get(row).ok_or(Decline("exp table"))?;
      let (a, b, c) = (a - 100_000_000, b - 100_000_000, c - 100_000_000);
      let t: Fixed = [
        a / 10000,
        a % 10000,
        b / 10000,
        b % 10000,
        c / 10000,
        c % 10000,
      ];
      accum = mul(&t, &accum);
      exponent += e;
    }
    if l == 0 {
      break;
    }
    l -= 1;
  }
  // `0 #2 0000 0000`: the fraction's digits after the zero
  let mut rest = [0u8; 40];
  let tail = &digits[pos..];
  let n = tail.len().min(rest.len() - 1);
  rest[1..=n].copy_from_slice(&tail[..n]);
  let x = blocks_of(&rest);
  Ok((exponent, mul(&exp_taylor(&x), &accum)))
}

/// The float of an `exp`-family result: `#1 \__fp_int_eval:w <exponent> #1 <float function>`.
fn exp_result(negate: bool, exponent: i64, f: &Fixed) -> R<Fp> {
  if negate {
    // \__fp_fixed_inv_to_float_o:wN, its exponent less the table's
    let (e, g) = ep_div(&(1, [1000, 0, 0, 0, 0, 0]), &(0, *f));
    let Some((fe, mant)) = fixed::to_float(&g) else {
      return Err(Decline("underflow"));
    };
    normal(Sign::Pos, e + fe - exponent, mant)
  } else {
    float_of(Sign::Pos, exponent, f)
  }
}

/// `\__fp_exp_o:w`
pub(super) fn exp(x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(..) => Ok(ONE),
    Fp::Inf(Sign::Pos) => Ok(super::num::INF),
    Fp::Inf(Sign::Neg) => Ok(ZERO),
    Fp::Normal { sign, exp: e, mant } => {
      let e = i64::from(e);
      if e > super::num::MAX_EXP_EXPONENT {
        return Err(Decline(if sign == Sign::Pos {
          "overflow"
        } else {
          "underflow"
        }));
      }
      let m = mant_digits(mant);
      let negate = sign == Sign::Neg;
      if e < 0 {
        // \__fp_decimate:nNnnnn { - #4 }, then `#2#3#4 0000 0000`
        let zeros = if -e > 16 { 17 } else { (-e) as usize };
        let mut digits = [0u8; 48];
        let keep = 16.min(digits.len() - zeros);
        digits[zeros..zeros + keep].copy_from_slice(&m[..keep]);
        let f = exp_taylor(&blocks_of(&digits));
        return exp_result(negate, 0, &f);
      }
      // `\int_value:w #3 #4`: the first digit at level e; digits after the (e+1)st, and 00000
      let mut digits = [0u8; 30];
      digits[..16].copy_from_slice(&m);
      let (exponent, f) = exp_large(e, &digits[..21])?;
      exp_result(negate, exponent, &f)
    },
  }
}

// ---------------------------------------------------------------- ln

/// `\c__fp_ln_<n>_fixed_tl`
fn ln_constant(n: i64) -> Fixed {
  match n {
    1 => [0, 0, 0, 0, 0, 0],
    2 => [6931, 4718, 559, 9453, 941, 7232],
    3 => [10986, 1228, 8668, 1096, 9139, 5245],
    4 => [13862, 9436, 1119, 8906, 1883, 4464],
    6 => [17917, 5946, 9228, 550, 81, 2477],
    7 => [19459, 1014, 9055, 3133, 510, 5353],
    8 => [20794, 4154, 1679, 8359, 2825, 1696],
    9 => [21972, 2457, 7336, 2193, 8279, 490],
    _ => LN_TEN,
  }
}
/// `\c__fp_ln_x_fixed_tl`
const LN_TEN: Fixed = [23025, 8509, 2994, 456, 8401, 7991];

/// `\__fp_div_significand_calc:wwnnnnnnn` on the seven-digit `v` (`1 d q`), the dividend `b1 b3 b4` and the divisor
/// `y`: the remainder's four blocks. Exact arithmetic with the shifts of `\__fp_pack_Bigg:NNNNNNw`.
fn div_calc(v: i64, b: [i64; 3], y: &[i64; 4]) -> [i64; 4] {
  const BL: i64 = -200000;
  const BM: i64 = 200000 * 9999;
  const BT: i64 = 200000 * 10000;
  let d = (v / 100000) % 10;
  let q = v % 100000;
  let big = d == 1;
  let r4 = BT - q * y[3];
  let r3 = BM + b[2] - q * y[2] - if big { y[3] * 10 } else { 0 } + r4 / 10000;
  let r2 = BM + b[1] - q * y[1] - if big { y[2] * 10 } else { 0 } + r3 / 10000;
  let r1 = BL + b[0] - q * y[0] - if big { concat(y[0], y[1]) * 10 } else { 0 } + r2 / 10000;
  [r1, r2 % 10000, r3 % 10000, r4 % 10000]
}

/// `\__fp_ln_significand:NNNNnnnN`: `-ln` of the mantissa, as a fixed (`\__fp_ln_x_ii:wnnnn` to `\__fp_ln_c:NwNw`).
fn ln_significand(mant: u64) -> Fixed {
  let b = super::num::blocks(mant);
  let [b1, b2, b3, b4] = [
    i64::from(b[0]),
    i64::from(b[1]),
    i64::from(b[2]),
    i64::from(b[3]),
  ];
  let (d1, d2) = (b1 / 1000, (b1 / 100) % 10);
  let factor = match d1 {
    1 => {
      if d2 < 4 {
        10 - d2
      } else {
        6
      }
    },
    2 => 4,
    3 => 3,
    4..=6 => 2,
    _ => 1,
  };
  // \__fp_ln_x_ii:wnnnn: X = factor * mantissa, and 1 + X
  let low = 1_000_000_000 + factor * concat(b3, b4);
  let high = 99_999_990 + factor * concat(b1, b2) + low / 100_000_000;
  let high5 = high / 10000;
  let y_int = high5 + 1;
  let y = [high5, high % 10000, (low / 10000) % 10000, low % 10000];
  // \__fp_ln_x_iv:wnnnnnnnn: 2/(1+X), six quotient digits by \__fp_div_significand_calc
  let mut vals = [0i64; 6];
  let v1 = 999_999 + rdiv(200_000_000, y_int);
  let mut r = div_calc(v1, [200_000_000, 0, 0], &y);
  vals[0] = v1;
  for val in vals.iter_mut().take(5).skip(1) {
    let dividend = concat(r[0], r[1]);
    let v = 999_999 + rdiv(dividend, y_int);
    r = div_calc(v, [dividend, r[2], r[3]], &y);
    *val = v;
  }
  vals[5] = 1_000_000 + rdiv(concat(r[0], r[1]), y_int);
  // the carries of \__fp_div_significand_pack:NNN, from the last quotient up
  let mut t = [0i64; 6];
  let mut carry = 0;
  for i in (0..6).rev() {
    let v = vals[i] + carry;
    t[i] = v % 10000;
    carry = v / 10000 - 100;
  }
  let t0 = carry;
  // \__fp_ln_div_after:Nw
  let (plus, t) = if t0 == 0 {
    (true, [
      9999 - t[0],
      9999 - t[1],
      9999 - t[2],
      9999 - t[3],
      9999 - t[4],
      10000 - t[5],
    ])
  } else {
    (false, t)
  };
  // \__fp_ln_t_large:NNw: t^2 ...
  let [t1, t2, t3, t4, t5, t6] = t;
  let s5 = 100_000_000
    + 2 * t1 * t5
    + 2 * t2 * t4
    + t3 * t3
    + rdiv(2 * t1 * t6 + 2 * t2 * t5 + 2 * t3 * t4, 10000);
  let s4 = 99_990_000 + 2 * t1 * t4 + 2 * t2 * t3 + s5 / 10000;
  let s3 = 99_990_000 + 2 * t1 * t3 + t2 * t2 + s4 / 10000;
  let s2 = 99_990_000 + 2 * t1 * t2 + s3 / 10000;
  let s1 = 99_990_000 + t1 * t1 + s2 / 10000;
  // \__fp_ln_square_t_after:w `1 0 #1#2#3 #4`
  let square: Fixed = [
    (s1 - 100_000_000) / 10000,
    s1 % 10000,
    s2 % 10000,
    s3 % 10000,
    s4 % 10000,
    s5 % 10000,
  ];
  // ... and 2t (\__fp_ln_twice_t_pack:Nw)
  let d6 = 10000 + 2 * t6;
  let d5 = 9999 + 2 * t5 + d6 / 10000;
  let d4 = 9999 + 2 * t4 + d5 / 10000;
  let d3 = 9999 + 2 * t3 + d4 / 10000;
  let d2 = 9999 + 2 * t2 + d3 / 10000;
  let d1 = -1 + 2 * t1 + d2 / 10000;
  let twice: Fixed = [
    d1,
    d2 % 10000,
    d3 % 10000,
    d4 % 10000,
    d5 % 10000,
    d6 % 10000,
  ];
  // \__fp_ln_Taylor_loop:www
  let mut s: Fixed = [0; 6];
  let mut n = 21;
  while n != 1 {
    s = mul(&fixed::add(&fixed::div_int(&ONE_FIXED, n), &s), &square);
    n -= 2;
  }
  let series = mul(&fixed::add_one(&s), &twice);
  // \__fp_ln_c:NwNw
  let constant = ln_constant(factor);
  if plus {
    sub(&constant, &series)
  } else {
    fixed::add(&constant, &series)
  }
}

/// `\__fp_ln_o:w`
pub(super) fn ln(x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(..) => Err(Decline("division by zero: ln(0)")),
    Fp::Inf(Sign::Pos) => Ok(x),
    Fp::Inf(Sign::Neg) | Fp::Normal { sign: Sign::Neg, .. } => {
      Err(Decline("invalid: ln of a negative"))
    },
    Fp::Normal { exp: e, mant, .. } => {
      let e = i64::from(e);
      let sign = if e < 1 { Sign::Neg } else { Sign::Pos };
      let l = ln_significand(mant);
      // \__fp_ln_exponent:wn
      match e {
        0 => float_of(sign, 0, &l),
        1 => float_of(sign, 0, &sub(&LN_TEN, &l)),
        _ => {
          let n = e.abs();
          let product = mul(&LN_TEN, &[n, 0, 0, 0, 0, 0]);
          let shifted: Fixed = [0, l[0], l[1], l[2], l[3], l[4]];
          let f = if e > 0 {
            sub(&product, &shifted)
          } else {
            fixed::add(&product, &shifted)
          };
          float_of(sign, 4, &f)
        },
      }
    },
  }
}

// ---------------------------------------------------------------- ^

/// `\__fp_pow_exponent:Nwnnnnnw`: `s * (n ln(10) - l)`.
fn pow_exponent(l: &Fixed, e: i64) -> Fixed {
  let s = if e > 0 { 1 } else { -1 };
  const M: i64 = 50000 * 9999;
  let v6 = 50000 * 10000 + s * e * 8401 - s * l[4] + s * rdiv(e * 7991 - l[5], 10000);
  let v5 = M + s * e * 456 - s * l[3] + v6 / 10000;
  let v4 = M + s * e * 2994 - s * l[2] + v5 / 10000;
  let v3 = M + s * e * 8509 - s * l[1] + v4 / 10000;
  let v2 = M + s * e * 23025 - s * l[0] + v3 / 10000;
  [
    -50000 + v2 / 10000,
    v2 % 10000,
    v3 % 10000,
    v4 % 10000,
    v5 % 10000,
    v6 % 10000,
  ]
}

/// `\__fp_pow_npos_o:Nww`: `|a|^b`, both normal.
fn pow_npos(ea: i64, ma: u64, sb: Sign, eb: i64, mb: u64) -> R<Fp> {
  let negate = sb != if ea > 0 { Sign::Pos } else { Sign::Neg };
  let l = ln_significand(ma);
  let exponent = pow_exponent(&l, ea);
  let product = mul(&exponent, &fixed_of_mant(mb));
  // \__fp_pow_B:wwN: `1` and five digits, the blocks, zeros; shifted by b's exponent
  let mut digits = [0u8; 80];
  write_digits(&mut digits[..5], product[0]);
  for (i, block) in product[1..].iter().enumerate() {
    write_digits(&mut digits[5 + i * 4..9 + i * 4], *block);
  }
  if eb < 0 {
    // \__fp_pow_C_neg:w: the zeros in front. Past 40 of them the digits are beyond what `\__fp_exp_large:NwN` reads.
    let zeros = eb.unsigned_abs().min(40) as usize;
    let mut shifted = [0u8; 120];
    shifted[zeros..zeros + 60].copy_from_slice(&digits[..60]);
    return pow_finish(negate, &shifted);
  }
  // \__fp_pow_C_pos:w: `eb` leading digits must be zero, and \__fp_pow_B:wwN stops at 22
  if eb >= 22 || digits[..eb as usize].iter().any(|d| *d != 0) {
    return Err(Decline(if negate { "underflow" } else { "overflow" }));
  }
  pow_finish(negate, &digits[eb as usize..])
}

/// `v` as digits filling `out`.
fn write_digits(out: &mut [u8], mut v: i64) {
  for slot in out.iter_mut().rev() {
    *slot = (v % 10) as u8;
    v /= 10;
  }
}

/// `\__fp_pow_C_pack:w`: `\__fp_exp_large:NwN` from level 5 over `digits`.
fn pow_finish(negate: bool, digits: &[u8]) -> R<Fp> {
  let (exponent, f) = exp_large(5, digits)?;
  exp_result(negate, exponent, &f)
}

/// `\__fp_^_o:ww`
pub(super) fn pow(a: Fp, b: Fp) -> R<Fp> {
  if b.is_zero() {
    return Ok(ONE);
  }
  let magnitude = pow_magnitude(a, b)?;
  if a.sign() == Sign::Pos {
    return Ok(magnitude);
  }
  // \__fp_pow_neg:www, \__fp_pow_neg_case:w
  match pow_neg_case(b) {
    -1 => Ok(magnitude),
    0 => Ok(magnitude.with_sign(magnitude.sign().flip())),
    _ => {
      if matches!(magnitude, Fp::Normal { .. } | Fp::Inf(_)) {
        Err(Decline("invalid: a negative base's fractional power"))
      } else {
        Ok(magnitude)
      }
    },
  }
}

fn pow_magnitude(a: Fp, b: Fp) -> R<Fp> {
  match a {
    Fp::Normal { exp: ea, mant: ma, .. } => {
      let ea = i64::from(ea);
      if ea == 1 && ma == POW10[15] {
        return Ok(ONE);
      }
      match b {
        Fp::Normal { sign, exp: eb, mant: mb } => pow_npos(ea, ma, sign, i64::from(eb), mb),
        Fp::Inf(sb) => {
          let grows = ea > 0;
          Ok(if grows == (sb == Sign::Pos) {
            super::num::INF
          } else {
            ZERO
          })
        },
        Fp::Zero(..) => Ok(ONE),
      }
    },
    // \__fp_pow_zero_or_inf:ww
    _ => {
      let bneg = b.sign() == Sign::Neg;
      let akind_zero = a.is_zero();
      if (akind_zero && !bneg) || (!akind_zero && bneg) {
        return Ok(ZERO);
      }
      if !akind_zero {
        return Ok(super::num::INF);
      }
      if matches!(b, Fp::Inf(_)) {
        return Ok(super::num::INF);
      }
      Err(Decline("division by zero: 0 to a negative power"))
    },
  }
}

/// `\__fp_pow_neg_case:w`: -1 an even integer (or a zero, an infinity), 0 an odd one, 1 not an integer.
fn pow_neg_case(b: Fp) -> i32 {
  match b {
    Fp::Zero(..) | Fp::Inf(_) => -1,
    Fp::Normal { exp, mant, .. } => {
      let e = i64::from(exp);
      if e > 16 {
        return -1;
      }
      let q = 16 - e;
      if q > 16 {
        return 1;
      }
      let p = POW10[q as usize];
      if mant % p != 0 {
        return 1;
      }
      if (mant / p) % 2 == 1 { 0 } else { -1 }
    },
  }
}

// ---------------------------------------------------------------- fact

/// `\__fp_fact_o:w`
pub(super) fn fact(x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(..) => Ok(ONE),
    Fp::Inf(Sign::Pos) => Ok(x),
    Fp::Inf(Sign::Neg) | Fp::Normal { sign: Sign::Neg, .. } => {
      Err(Decline("invalid: fact of a negative"))
    },
    Fp::Normal { .. } => {
      let n = small_int(x).ok_or(Decline("invalid: fact of a non-integer"))?;
      if n > 3248 {
        return Err(Decline("overflow"));
      }
      // \__fp_fact_loop_o:w
      let mut n = n;
      let mut acc: Ep = (4, [1, 0, 0, 0, 0, 0]);
      while n >= 12 {
        acc = ep_mul(&(4, [n * (n - 1), 0, 0, 0, 0, 0]), &acc);
        n -= 2;
      }
      const SMALL: [i64; 12] = [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
      ];
      let (e, f) = ep_mul(&(4, [SMALL[n as usize], 0, 0, 0, 0, 0]), &acc);
      float_of(Sign::Pos, e, &f)
    },
  }
}

// ---------------------------------------------------------------- the reduction of an angle

const MINI_PI: Fixed = [7853, 9816, 3397, 4483, 961, 5661];

/// `\__fp_trig_small:ww`: the octant so far, the fixed of the reduced angle, and its extended form.
fn trig_small(octant: i64, ep: Ep) -> (i64, Fixed, Ep) { (octant, ep_to_fixed(&ep), ep) }

/// `\__fp_trigd_small:ww`
fn trigd_small(octant: i64, ep: Ep) -> (i64, Fixed, Ep) {
  let degree: Ep = (-1, [1745, 3292, 5199, 4329, 5769, 2369]);
  trig_small(octant, ep_mul_raw(&degree, &ep))
}

/// `\__fp_trigd_large:ww`: an angle in degrees reduced modulo 360 by digits (`(x-4)/9*9`).
fn trigd_large(octant: i64, e: i64, f: &Fixed) -> (i64, Fixed, Ep) {
  // zeros, the 24 digits, `0000 0000 0000`
  let zeros = (22 - e).max(0) as usize;
  let mut s = [0u8; 80];
  let mut n = zeros;
  for block in f {
    let mut v = *block;
    for i in (0..4).rev() {
      s[n + i] = (v % 10) as u8;
      v /= 10;
    }
    n += 4;
  }
  let read = |from: usize, to: usize| s[from..to].iter().fold(0i64, |v, d| v * 10 + i64::from(*d));
  // \__fp_trigd_large_auxi:nnnnwNNNN
  let (a1, a2) = (read(0, 8), read(8, 16));
  let (a3, a4_d1, a4_rest) = (read(16, 20), read(20, 21), read(21, 24));
  let first = a1 + a2 - rdiv(a1 + a2 - 4, 9) * 9;
  // `#1` of auxii is first and the next four digits side by side; `#2` one digit more
  let v = (first * 10000 + a3) * 10 + a4_d1;
  let k = rdiv(v - 4, 9);
  let mut octant = octant + k * 2;
  let reduced = v - k * 9;
  let head = reduced * 1000 + a4_rest;
  let x: Fixed = [head, read(24, 28), read(28, 32), read(32, 36), 0, 0];
  let x = if head < 4500 {
    x
  } else {
    octant += 1;
    sub(&[9000, 0, 0, 0, 0, 0], &x)
  };
  trigd_small(octant, (2, x))
}

/// `\__fp_trig_large:ww`: an angle in radians, by its digits against those of `\c__fp_trig_intarray`.
fn trig_large(octant: i64, e: i64, f: &Fixed) -> R<(i64, Fixed, Ep)> {
  let k = rdiv(e - 4, 8);
  let eaten = e - k * 8;
  let start = k as usize;
  // nine items, each without its leading 1; the first `eaten` digits go
  let mut digits = [0u8; 72];
  for i in 0..9 {
    let item = *TRIG_TABLE
      .get(start + i)
      .ok_or(Decline("an angle past the trig table"))?
      - 100_000_000;
    let mut v = item;
    for j in (0..8).rev() {
      digits[i * 8 + j] = (v % 10) as u8;
      v /= 10;
    }
  }
  let mut p = [0i64; 16];
  for (i, slot) in p.iter_mut().enumerate() {
    let at = eaten as usize + i * 4;
    *slot = digits[at..at + 4]
      .iter()
      .fold(0, |v, d| v * 10 + i64::from(*d));
  }
  // \__fp_trig_large_auxvi:wnnnnnnnn: m * p, thirteen blocks
  const M: i64 = 50000 * 9999;
  let m = [f[0], f[1], f[2], f[3]];
  let mut low = [0i64; 13];
  let mut carry = 0;
  for i in (0..13).rev() {
    let base = if i == 12 { 50000 * 10000 } else { M };
    let v = base + m[0] * p[i + 3] + m[1] * p[i + 2] + m[2] * p[i + 1] + m[3] * p[i] + carry;
    low[i] = v % 10000;
    carry = v / 10000;
  }
  // \__fp_trig_large_auxvii:w: the first three digits name the octant, \__fp_trig_large_auxviii:ww
  let first3 = low[0] / 10;
  let q = rdiv(first3 - 62, 125);
  let octant = octant + q;
  let s: i64 = if q.rem_euclid(2) == 1 { -1 } else { 1 };
  // \__fp_trig_large_auxx:wNNNNN: 8 * the fraction's digits (or their complement), 52 digits
  let mut d = [0i64; 13];
  let mut carry = 0;
  // the digit stream starts with the three digits again, so this runs over the same 13 blocks
  for i in (0..13).rev() {
    let base = if i == 12 { 50000 * 10000 } else { M };
    let v = base + s * 8 * low[i] + carry;
    d[i] = v % 10000;
    carry = v / 10000;
  }
  // \__fp_trig_large_auxxi:w: \__fp_ep_to_ep_loop:N over them
  let mut v: u128 = 0;
  let mut zeros: i64 = 0;
  let mut seen = false;
  let mut count = 0;
  for block in &d {
    for j in (0..4).rev() {
      let digit = (block / 10i64.pow(j)) % 10;
      if !seen && digit == 0 {
        zeros += 1;
        continue;
      }
      seen = true;
      if count < 24 {
        v = v * 10 + digit as u128;
        count += 1;
      }
    }
  }
  if !seen || count < 24 {
    // the sign would be read as a digit: what the TeX does then is not a number
    return Err(Decline("an angle within 1e-28 of a multiple of pi/4"));
  }
  let mut blocks = [0i64; 6];
  for slot in blocks.iter_mut().rev() {
    *slot = (v % 10000) as i64;
    v /= 10000;
  }
  let product = ep_mul_raw(&(-zeros, blocks), &(0, MINI_PI));
  Ok(trig_small(octant, product))
}

/// `\__fp_trig:NNNNNwn`'s reduction: octant base, then the fixed and the extended form of the angle.
fn reduce(unit: Unit, base: i64, exp: i32, mant: u64) -> R<(i64, Fixed, Ep)> {
  let e = i64::from(exp);
  let f = fixed_of_mant(mant);
  let threshold = if unit == Unit::Rad { 0 } else { 1 };
  Ok(match (e > threshold, unit) {
    (true, Unit::Rad) => trig_large(base, e, &f)?,
    (true, Unit::Deg) => trigd_large(base, e, &f),
    (false, Unit::Rad) => trig_small(base, (e, f)),
    (false, Unit::Deg) => trigd_small(base, (e, f)),
  })
}

const SIN_COEFFS_COS: [Fixed; 9] = [
  [0, 0, 0, 477, 9477, 3324],
  [0, 0, 11, 4707, 4559, 7730],
  [0, 0, 2087, 6756, 9878, 6810],
  [0, 27, 5573, 1922, 3985, 8907],
  [0, 2480, 1587, 3015, 8730, 1587],
  [13, 8888, 8888, 8888, 8888, 8889],
  [416, 6666, 6666, 6666, 6666, 6667],
  [5000, 0, 0, 0, 0, 0],
  [10000, 0, 0, 0, 0, 0],
];
const SIN_COEFFS_SIN: [Fixed; 8] = [
  [0, 0, 0, 7647, 1637, 3182],
  [0, 0, 160, 5904, 3836, 8216],
  [0, 2, 5052, 1083, 8544, 1719],
  [0, 275, 5731, 9223, 9858, 9065],
  [1, 9841, 2698, 4126, 9841, 2698],
  [83, 3333, 3333, 3333, 3333, 3333],
  [1666, 6666, 6666, 6666, 6666, 6667],
  [10000, 0, 0, 0, 0, 0],
];

/// `\__fp_sin_series_aux_o:NNnwww`'s two series: the extended result.
fn sin_series(octant: i64, x: &Fixed, ep: &Ep) -> Ep {
  let square = mul(x, x);
  if rdiv(octant, 2).rem_euclid(2) == 1 {
    let mut v = mul_sub_back(&[0, 0, 0, 1, 5619, 2070], &square, &SIN_COEFFS_COS[0]);
    for c in &SIN_COEFFS_COS[1..] {
      v = mul_sub_back(&v, &square, c);
    }
    (0, v)
  } else {
    let mut v = mul_sub_back(&[0, 0, 0, 28, 1145, 7254], &square, &SIN_COEFFS_SIN[0]);
    for c in &SIN_COEFFS_SIN[1..] {
      v = mul_sub_back(&v, &square, c);
    }
    ep_mul(&(0, v), ep)
  }
}

/// The sign of `\__fp_sin_series_o:NNwwww`.
fn sin_sign(octant: i64, sign: Sign) -> Sign {
  if rdiv(octant + 2, 4).rem_euclid(2) == 1 {
    sign
  } else {
    sign.flip()
  }
}

/// `sin`, `cos`, `csc`, `sec` (`\__fp_sin_o:w` and its siblings).
pub(super) fn sin_like(unit: Unit, x: Fp, kind: SinKind) -> R<Fp> {
  let (base, inverse) = match kind {
    SinKind::Sin => (0, false),
    SinKind::Cos => (2, false),
    SinKind::Csc => (0, true),
    SinKind::Sec => (2, true),
  };
  match x {
    Fp::Zero(..) => match kind {
      SinKind::Sin => Ok(x),
      SinKind::Cos | SinKind::Sec => Ok(ONE),
      SinKind::Csc => Err(Decline("division by zero: csc(0)")),
    },
    Fp::Inf(_) => Err(Decline("invalid: a trigonometric function of inf")),
    Fp::Normal { sign, exp, mant } => {
      let (octant, f, ep) = reduce(unit, base, exp, mant)?;
      let sign = sin_sign(
        octant,
        if matches!(kind, SinKind::Sin | SinKind::Csc) {
          sign
        } else {
          Sign::Pos
        },
      );
      let r = sin_series(octant, &f, &ep);
      if inverse {
        inv_float_of(sign, &r)
      } else {
        float_of(sign, r.0, &r.1)
      }
    },
  }
}

/// Which of the sine family.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SinKind {
  /// `\__fp_sin_o:w`
  Sin,
  /// `\__fp_cos_o:w`
  Cos,
  /// `\__fp_csc_o:w`
  Csc,
  /// `\__fp_sec_o:w`
  Sec,
}

const TAN_NUM: [Fixed; 5] = [
  [0, 159, 6080, 274, 5257, 6472],
  [2, 4571, 2320, 157, 2558, 8481],
  [115, 5830, 7533, 5397, 3168, 2147],
  [1929, 8245, 6140, 3508, 7719, 2982],
  [10000, 0, 0, 0, 0, 0],
];
const TAN_DEN: [Fixed; 5] = [
  [0, 2343, 7175, 1399, 6151, 7670],
  [19, 2638, 4588, 9232, 8861, 3691],
  [536, 6357, 691, 4344, 6852, 4252],
  [5263, 1578, 9473, 6842, 1052, 6315],
  [10000, 0, 0, 0, 0, 0],
];

/// `\__fp_tan_o:w`, `\__fp_cot_o:w` (`\__fp_tan_series_o:NNwwww`).
pub(super) fn tan_like(unit: Unit, x: Fp, cot: bool) -> R<Fp> {
  match x {
    Fp::Zero(..) => {
      if cot {
        Err(Decline("division by zero: cot(0)"))
      } else {
        Ok(x)
      }
    },
    Fp::Inf(_) => Err(Decline("invalid: a trigonometric function of inf")),
    Fp::Normal { sign, exp, mant } => {
      let (first, base) = if cot { (Sign::Neg, 3) } else { (Sign::Pos, 1) };
      let (octant, f, ep) = reduce(unit, base, exp, mant)?;
      let square = mul(&f, &f);
      // the sign: `\if_meaning:w #1#2 2 \else: 0 \fi:`, reversed where `#3 / 2` is odd
      let same = first == sign;
      let mut neg = same;
      if rdiv(octant, 2).rem_euclid(2) == 1 {
        neg = !neg;
      }
      let result_sign = if neg { Sign::Neg } else { Sign::Pos };
      let mut n = mul_sub_back(&[0, 0, 1527, 3493, 856, 7059], &square, &TAN_NUM[0]);
      for c in &TAN_NUM[1..] {
        n = mul_sub_back(&n, &square, c);
      }
      let numerator = ep_mul(&(0, n), &ep);
      let mut d = mul_sub_back(&[0, 7, 258, 681, 9408, 4706], &square, &TAN_DEN[0]);
      for c in &TAN_DEN[1..] {
        d = mul_sub_back(&d, &square, c);
      }
      let denominator: Ep = (0, d);
      let (e, r) = if rdiv(octant - 1, 2).rem_euclid(2) == 1 {
        ep_div(&denominator, &numerator)
      } else {
        ep_div(&numerator, &denominator)
      };
      float_of(result_sign, e, &r)
    },
  }
}

// ---------------------------------------------------------------- the inverses

/// `\__fp_atan_combine_o:NwwwwwN`
fn atan_combine(sign: Sign, octant: i64, s: &Fixed, x: &Fixed, ep: &Ep, unit: Unit) -> R<Fp> {
  let finish = |base: i64, f: &Fixed| -> R<Fp> {
    match unit {
      Unit::Rad => float_of(sign, base, f),
      // \__fp_fixed_to_float_rad_o:wN
      Unit::Deg => {
        let g = mul(f, &[5729, 5779, 5130, 8232, 876, 7981]);
        float_of(sign, base + 2, &g)
      },
    }
  };
  if octant == 0 {
    finish(ep.0, &mul(s, &ep.1))
  } else {
    let p = mul(s, x);
    // \__fp_atan_combine_aux:ww
    let q = fixed::mul_short(&MINI_PI, [rdiv(octant, 2), 0, 0]);
    let f = if octant.rem_euclid(2) == 1 {
      sub(&q, &p)
    } else {
      fixed::add(&q, &p)
    };
    finish(0, &f)
  }
}

/// `\__fp_atan_auxi:ww`, `\__fp_atan_Taylor_loop:www`
fn atan_series(ep: &Ep) -> (Fixed, Fixed) {
  let x = ep_to_fixed(ep);
  let square = mul(&x, &x);
  let mut s: Fixed = [0; 6];
  let mut n = 39;
  while n != -1 {
    s = mul_sub_back(&s, &square, &fixed::div_int(&ONE_FIXED, n));
    n -= 2;
  }
  (s, x)
}

/// `\__fp_atan_test_o:NwwNwwN`: `atan2` of two extended numbers with signs.
fn atan_test(s1: Sign, y: &Ep, s2: Sign, x: &Ep, unit: Unit) -> R<Fp> {
  let reversed = fixed::ep_compare(y, x) > 0;
  let (num, den) = if reversed { (x, y) } else { (y, x) };
  // \__fp_atan_div:wnwwnw
  let scale = match den.0 - num.0 {
    0 => 100,
    1 => 10,
    _ => 1,
  };
  let near = 41421 * den.1[0] < num.1[0] * 1000 * scale;
  let (mut octant, quotient) = if near {
    // \__fp_atan_near:wwwn, \__fp_atan_near_aux:wwn
    let a = ep_to_fixed(&(num.0 - den.0, num.1));
    let sum = fixed::add(&a, &den.1);
    let diff = sub(&den.1, &a);
    (1, ep_div(&(0, diff), &(0, sum)))
  } else {
    (0, ep_div(num, den))
  };
  let (s, xfix) = atan_series(&quotient);
  if reversed {
    octant = 3 - octant;
  }
  if s2 == Sign::Neg {
    octant = 7 - octant;
  }
  atan_combine(s1, octant, &s, &xfix, &quotient, unit)
}

/// `\__fp_atan_inf_o:NNNw`
fn atan_inf(sign: Sign, octant: i64, x_sign: Sign, unit: Unit) -> R<Fp> {
  let octant = if x_sign == Sign::Neg {
    7 - octant
  } else {
    octant
  };
  atan_combine(sign, octant, &ONE_FIXED, &[0; 6], &(0, [0; 6]), unit)
}

/// `\__fp_atanii_o:Nww`: `atan(y, x)`.
pub(super) fn atan2(unit: Unit, y: Fp, x: Fp) -> R<Fp> {
  match (y, x) {
    (Fp::Normal { sign: sy, exp: ey, mant: my }, Fp::Normal { sign: sx, exp: ex, mant: mx }) => {
      let py: Ep = (i64::from(ey), fixed_of_mant(my));
      let px: Ep = (i64::from(ex), fixed_of_mant(mx));
      atan_test(sy, &py, sx, &px, unit)
    },
    _ => {
      let octant = match y.kind().cmp(&x.kind()) {
        std::cmp::Ordering::Equal => 2,
        std::cmp::Ordering::Greater => 4,
        std::cmp::Ordering::Less => 0,
      };
      atan_inf(y.sign(), octant, x.sign(), unit)
    },
  }
}

/// `\__fp_asin_auxi_o:NnNww`: the fixed `sqrt(1-x^2)` by `\__fp_ep_isqrt:wwn`, then `atan`.
fn asin_auxi(unit: Unit, reverse: bool, sign: Sign, ep: &Ep) -> R<Fp> {
  let x = ep_to_fixed(ep);
  // \__fp_asin_isqrt:wn
  let diff = sub(&ONE_FIXED, &x);
  let sum = fixed::add_one(&x);
  let product = ep_mul(&(0, sum), &(0, diff));
  let isqrt = fixed::ep_isqrt(&product);
  let y = ep_to_ep(&ep_mul(&isqrt, ep));
  let one: Ep = (1, [1000, 0, 0, 0, 0, 0]);
  if reverse {
    atan_test(Sign::Pos, &one, sign, &y, unit)
  } else {
    atan_test(sign, &y, Sign::Pos, &one, unit)
  }
}

/// `\__fp_asin_normal_o:NfwNnnnnw`: `|x| <= 1`, or the invalid operation's report.
fn asin_normal(unit: Unit, reverse: bool, x: Fp) -> R<Fp> {
  let Fp::Normal { sign, exp, mant } = x else {
    return Err(Decline("asin of a special value"));
  };
  if exp >= 1 && !(exp == 1 && mant == POW10[15]) {
    return Err(Decline("invalid: asin or acos outside [-1, 1]"));
  }
  asin_auxi(unit, reverse, sign, &(i64::from(exp), fixed_of_mant(mant)))
}

/// `\__fp_asin_o:w`
pub(super) fn asin(unit: Unit, x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(..) => Ok(x),
    Fp::Inf(_) => Err(Decline("invalid: asin of inf")),
    Fp::Normal { .. } => asin_normal(unit, false, x),
  }
}

/// `\__fp_acos_o:w`
pub(super) fn acos(unit: Unit, x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(s, _) => atan_inf(Sign::Pos, 4, s, unit),
    Fp::Inf(_) => Err(Decline("invalid: acos of inf")),
    Fp::Normal { .. } => asin_normal(unit, true, x),
  }
}

/// `\__fp_acsc_normal_o:NfwNnw`: `asin(1/x)`.
fn acsc_normal(unit: Unit, reverse: bool, x: Fp) -> R<Fp> {
  let Fp::Normal { sign, exp, mant } = x else {
    return Err(Decline("acsc of a special value"));
  };
  if exp < 1 {
    return Err(Decline("invalid: acsc or asec inside (-1, 1)"));
  }
  let q = ep_div(
    &(1, [1000, 0, 0, 0, 0, 0]),
    &(i64::from(exp), fixed_of_mant(mant)),
  );
  asin_auxi(unit, reverse, sign, &q)
}

/// `\__fp_acsc_o:w`
pub(super) fn acsc(unit: Unit, x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(..) => Err(Decline("invalid: acsc(0)")),
    Fp::Inf(Sign::Pos) => Ok(ZERO),
    Fp::Inf(Sign::Neg) => Ok(MINUS_ZERO),
    Fp::Normal { .. } => acsc_normal(unit, false, x),
  }
}

/// `\__fp_asec_o:w`
pub(super) fn asec(unit: Unit, x: Fp) -> R<Fp> {
  match x {
    Fp::Zero(..) => Err(Decline("invalid: asec(0)")),
    Fp::Inf(s) => atan_inf(Sign::Pos, 4, s, unit),
    Fp::Normal { .. } => acsc_normal(unit, true, x),
  }
}

/// The rounding word's mode (`\__fp_round:Nwww`).
pub(super) fn nearest_mode(third: Fp) -> Mode {
  match third {
    Fp::Zero(..) => Mode::NearestZero,
    other => {
      if other.sign() == Sign::Neg {
        Mode::NearestNinf
      } else {
        Mode::NearestPinf
      }
    },
  }
}
