//! The floating point value of l3fp and its exactly rounded operations (l3fp-basics, l3fp-round, l3fp-logic):
//! `+ - * /` and `sqrt` round to 16 digits, ties to even, as `\__fp_round_digit:Nw` with `\__fp_round_to_nearest:NNN`
//! achieves one digit at a time, so exact `u128` arithmetic gives the same digits (the oracle `kit/l3fp_oracle.sh` holds
//! this against pdflatex). What the TeX answers with a flag or a message (invalid operation, division by zero, overflow,
//! underflow) is a [`Decline`]: the raw macro reports it.

/// `10^n` for the mantissa and its neighbours.
pub(super) const POW10: [u64; 20] = {
  let mut t = [1u64; 20];
  let mut i = 1;
  while i < 20 {
    t[i] = t[i - 1] * 10;
    i += 1;
  }
  t
};
/// `\c__fp_max_exponent_int`, `\c__fp_minus_min_exponent_int`
pub(super) const MAX_EXPONENT: i64 = 10000;
/// `\c__fp_max_exp_exponent_int`
pub(super) const MAX_EXP_EXPONENT: i64 = 5;

/// Why the native path gave way to the raw macro. Reasons are static, so a fallback costs no allocation; the census
/// (`LATEXML_L3FP_TRACE`) groups by them.
#[derive(Clone, Copy)]
pub struct Decline(pub &'static str);
pub(super) type R<T> = Result<T, Decline>;

/// A sign digit as the TeX stores it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sign {
  /// `0`
  Pos,
  /// `2`
  Neg,
}

impl Sign {
  pub(super) fn flip(self) -> Sign {
    match self {
      Sign::Pos => Sign::Neg,
      Sign::Neg => Sign::Pos,
    }
  }
}

/// The mark of a special value: `\s__fp_exact`, or the `\s__fp_underflow` a `\__fp_zero_fp:N` zero carries (`round(0.4)`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tag {
  /// `\s__fp_exact`
  Exact,
  /// `\s__fp_underflow`
  Underflow,
}

/// A floating point number: `\s__fp \__fp_chk:w <kind> <sign> ...`, without the `nan` the native path never holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fp {
  /// kind 0
  Zero(Sign, Tag),
  /// kind 2, `\s__fp_exact`
  Inf(Sign),
  /// kind 1: `0.mant * 10^exp`, the mantissa of exactly 16 digits
  Normal {
    /// the sign
    sign: Sign,
    /// the exponent, `|exp| <= 10000`
    exp:  i32,
    /// `10^15 <= mant < 10^16`
    mant: u64,
  },
}

/// `\c_zero_fp`
pub(super) const ZERO: Fp = Fp::Zero(Sign::Pos, Tag::Exact);
/// `\c_minus_zero_fp`
pub(super) const MINUS_ZERO: Fp = Fp::Zero(Sign::Neg, Tag::Exact);
/// `\c_inf_fp`
pub(super) const INF: Fp = Fp::Inf(Sign::Pos);
/// `\c_minus_inf_fp`
pub(super) const MINUS_INF: Fp = Fp::Inf(Sign::Neg);
/// `\c_one_fp`
pub(super) const ONE: Fp = Fp::Normal {
  sign: Sign::Pos,
  exp:  1,
  mant: POW10[15],
};

/// `\__fp_sanitize:Nw` of a non-zero result: out of range is an overflow's or an underflow's flag, for the raw macro.
pub(super) fn normal(sign: Sign, exp: i64, mant: u64) -> R<Fp> {
  if exp > MAX_EXPONENT {
    return Err(Decline("overflow"));
  }
  if exp < -MAX_EXPONENT {
    return Err(Decline("underflow"));
  }
  debug_assert!((POW10[15]..POW10[16]).contains(&mant));
  Ok(Fp::Normal { sign, exp: exp as i32, mant })
}

/// The sign digit, with `Pos` for a positive-or-zero sign.
impl Fp {
  /// `0`, `1`, `2`: the kind the TeX tests.
  pub(super) fn kind(self) -> u8 {
    match self {
      Fp::Zero(..) => 0,
      Fp::Normal { .. } => 1,
      Fp::Inf(..) => 2,
    }
  }

  pub(super) fn sign(self) -> Sign {
    match self {
      Fp::Zero(s, _) | Fp::Inf(s) => s,
      Fp::Normal { sign, .. } => sign,
    }
  }

  /// The same value with another sign (`\__fp_set_sign_o:w`; a zero keeps its mark).
  pub(super) fn with_sign(self, s: Sign) -> Fp {
    match self {
      Fp::Zero(_, t) => Fp::Zero(s, t),
      Fp::Inf(_) => Fp::Inf(s),
      Fp::Normal { exp, mant, .. } => Fp::Normal { sign: s, exp, mant },
    }
  }

  pub(super) fn is_zero(self) -> bool { matches!(self, Fp::Zero(..)) }
}

/// Round a number of `len` decimal digits to 16 (`\__fp_round:NNN`: nearest, a tie to the even digit), `sticky` standing
/// for digits below it; the digits and the carry into a 17th.
#[inline]
fn round_to_16(v: u128, len: u32, sticky: bool) -> (u64, i64) {
  debug_assert!(len >= 16);
  let d = len - 16;
  if d == 0 {
    debug_assert!(!sticky);
    return (v as u64, 0);
  }
  let p = 10u128.pow(d);
  let mut q = (v / p) as u64;
  let r = v % p;
  let half = p / 2;
  if r > half || (r == half && (sticky || q % 2 == 1)) {
    q += 1;
  }
  if q == POW10[16] {
    (POW10[15], 1)
  } else {
    (q, 0)
  }
}

/// `\__fp_add_npos_o:NnwNnw`
fn add_npos(sign: Sign, e1: i64, m1: u64, e2: i64, m2: u64) -> R<Fp> {
  let (e, big, small, s) = if e1 > e2 {
    (e1, m1, m2, e1 - e2)
  } else {
    (e2, m2, m1, e2 - e1)
  };
  // past 20 digits the small one is far below the rounding digit of `big`
  if s > 20 {
    return normal(sign, e, big);
  }
  let v = big as u128 * 10u128.pow(s as u32) + small as u128;
  let len = 16 + s as u32 + u32::from(v >= 10u128.pow(16 + s as u32));
  let (mant, carry) = round_to_16(v, len, false);
  normal(sign, e + i64::from(len - 16 - s as u32) + carry, mant)
}

/// `\__fp_sub_npos_o:NnwNnw`
fn sub_npos(sign: Sign, e1: i64, m1: u64, e2: i64, m2: u64) -> R<Fp> {
  let (sign, e1, m1, e2, m2) = match (e1, m1).cmp(&(e2, m2)) {
    std::cmp::Ordering::Equal => return Ok(ZERO),
    std::cmp::Ordering::Greater => (sign, e1, m1, e2, m2),
    std::cmp::Ordering::Less => (sign.flip(), e2, m2, e1, m1),
  };
  let s = e1 - e2;
  // past 21 digits `m2` is far below half a unit of the result, even one digit shorter
  if s > 21 {
    return normal(sign, e1, m1);
  }
  let v = m1 as u128 * 10u128.pow(s as u32) - m2 as u128;
  let len = super::fixed::decimal_len(v);
  if len >= 16 {
    let (mant, carry) = round_to_16(v, len, false);
    normal(sign, e1 - (16 + s) + i64::from(len) + carry, mant)
  } else {
    let mant = v as u64 * POW10[(16 - len) as usize];
    normal(sign, e1 - (16 + s) + i64::from(len), mant)
  }
}

/// `\__fp_+_o:ww`, with `b`'s sign already flipped for a `-` (`\__fp_-_o:ww`).
pub(super) fn add(a: Fp, b: Fp) -> R<Fp> {
  match (a, b) {
    (Fp::Zero(sa, _), Fp::Zero(sb, _)) => {
      // \__fp_add_zeros_o:Nww
      Ok(if sa == Sign::Neg && sb == Sign::Pos {
        b
      } else {
        a
      })
    },
    (Fp::Inf(sa), Fp::Inf(sb)) => {
      if sa == sb {
        Ok(a)
      } else {
        Err(Decline("invalid: inf - inf"))
      }
    },
    (Fp::Normal { sign: sa, exp: ea, mant: ma }, Fp::Normal { sign: sb, exp: eb, mant: mb }) => {
      if sa == sb {
        add_npos(sa, i64::from(ea), ma, i64::from(eb), mb)
      } else {
        sub_npos(sa, i64::from(ea), ma, i64::from(eb), mb)
      }
    },
    _ => Ok(if a.kind() > b.kind() { a } else { b }),
  }
}

/// `\__fp_-_o:ww`
pub(super) fn sub(a: Fp, b: Fp) -> R<Fp> {
  // a's sign and b's flipped: `\__fp_add_return_ii_o:Nww` returns the flipped second
  add(a, b.with_sign(b.sign().flip()))
}

fn product_sign(a: Fp, b: Fp) -> Sign {
  if a.sign() == b.sign() {
    Sign::Pos
  } else {
    Sign::Neg
  }
}

/// `\__fp_*_o:ww`
pub(super) fn mul(a: Fp, b: Fp) -> R<Fp> {
  let sign = product_sign(a, b);
  match (a, b) {
    (Fp::Normal { exp: ea, mant: ma, .. }, Fp::Normal { exp: eb, mant: mb, .. }) => {
      let v = u128::from(ma) * u128::from(mb);
      let len = 31 + u32::from(v >= 10u128.pow(31));
      let (mant, carry) = round_to_16(v, len, false);
      normal(
        sign,
        i64::from(ea) + i64::from(eb) + i64::from(len) - 32 + carry,
        mant,
      )
    },
    (Fp::Zero(..), Fp::Inf(_)) | (Fp::Inf(_), Fp::Zero(..)) => Err(Decline("invalid: 0 * inf")),
    (Fp::Inf(_), _) | (_, Fp::Inf(_)) => Ok(if sign == Sign::Pos { INF } else { MINUS_INF }),
    _ => Ok(if sign == Sign::Pos { ZERO } else { MINUS_ZERO }),
  }
}

/// `\__fp_/_o:ww`
pub(super) fn div(a: Fp, b: Fp) -> R<Fp> {
  let sign = product_sign(a, b);
  match (a, b) {
    (Fp::Normal { exp: ea, mant: ma, .. }, Fp::Normal { exp: eb, mant: mb, .. }) => {
      let n = u128::from(ma) * 10u128.pow(18);
      let d = u128::from(mb);
      let q = n / d;
      let sticky = !n.is_multiple_of(d);
      // `ma >= mb`: 19 digits, the exponent's +1
      let len = 18 + u32::from(q >= 10u128.pow(18));
      let (mant, carry) = round_to_16(q, len, sticky);
      normal(
        sign,
        i64::from(ea) - i64::from(eb) + i64::from(len) - 18 + carry,
        mant,
      )
    },
    (Fp::Zero(..), Fp::Zero(..)) | (Fp::Inf(_), Fp::Inf(_)) => {
      Err(Decline("invalid: 0/0 or inf/inf"))
    },
    (Fp::Normal { .. }, Fp::Zero(..)) => Err(Decline("division by zero")),
    (Fp::Inf(_), _) => Ok(if sign == Sign::Pos { INF } else { MINUS_INF }),
    _ => Ok(if sign == Sign::Pos { ZERO } else { MINUS_ZERO }),
  }
}

/// `floor(sqrt(v))`, by Newton's method on integers.
fn isqrt(v: u128) -> u128 {
  if v < 2 {
    return v;
  }
  let mut x = 1u128 << (v.ilog2() / 2 + 1);
  loop {
    let y = (x + v / x) / 2;
    if y >= x {
      return x;
    }
    x = y;
  }
}

/// `\__fp_sqrt_o:w`
pub(super) fn sqrt(a: Fp) -> R<Fp> {
  match a {
    Fp::Zero(..) => Ok(a),
    Fp::Inf(Sign::Pos) => Ok(a),
    Fp::Inf(Sign::Neg) | Fp::Normal { sign: Sign::Neg, .. } => {
      Err(Decline("invalid: sqrt of a negative"))
    },
    Fp::Normal { exp, mant, .. } => {
      let exp = i64::from(exp);
      let odd = exp.rem_euclid(2) == 1;
      let v = u128::from(mant) * 10u128.pow(if odd { 19 } else { 20 });
      let r = isqrt(v);
      let sticky = r * r != v;
      let (mant, carry) = round_to_16(r, 18, sticky);
      let e = if odd { (exp + 1) / 2 } else { exp / 2 };
      normal(Sign::Pos, e + carry, mant)
    },
  }
}

/// `\__fp_bcmp:ww`: the sign of `a - b`; -0 equals 0.
pub(super) fn compare(a: Fp, b: Fp) -> i32 {
  let magnitude = |x: Fp| -> (u8, i64, u64) {
    match x {
      Fp::Normal { exp, mant, .. } => (1, i64::from(exp), mant),
      other => (other.kind(), 0, 0),
    }
  };
  if a.is_zero() && b.is_zero() {
    return 0;
  }
  if a.sign() != b.sign() {
    return if a.sign() == Sign::Neg { -1 } else { 1 };
  }
  let c = match magnitude(a).cmp(&magnitude(b)) {
    std::cmp::Ordering::Less => -1,
    std::cmp::Ordering::Equal => 0,
    std::cmp::Ordering::Greater => 1,
  };
  if a.sign() == Sign::Neg { -c } else { c }
}

/// How a `round` family function rounds (`\__fp_round_to_...:NNN`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
  /// `trunc`
  Zero,
  /// `floor`
  Ninf,
  /// `ceil`
  Pinf,
  /// `round`
  Nearest,
  /// `round(x,n,0)`
  NearestZero,
  /// `round(x,n,-1)`
  NearestNinf,
  /// `round(x,n,1)`
  NearestPinf,
}

/// The mode's macro on the sign, the last digit kept and the rounding digit (`\__fp_round_digit:Nw`).
fn round_up(mode: Mode, sign: Sign, last: u64, digit: u64) -> bool {
  match mode {
    Mode::Zero => false,
    Mode::Ninf => sign == Sign::Neg && digit > 0,
    Mode::Pinf => sign == Sign::Pos && digit > 0,
    Mode::Nearest => digit > 5 || (digit == 5 && last % 2 == 1),
    Mode::NearestZero => digit > 5,
    Mode::NearestNinf => digit > 5 || (digit == 5 && sign == Sign::Neg),
    Mode::NearestPinf => digit > 5 || (digit == 5 && sign == Sign::Pos),
  }
}

/// `\__fp_round:Nwn`, at `places` decimal places.
pub(super) fn round(mode: Mode, x: Fp, places: i64) -> R<Fp> {
  let Fp::Normal { sign, exp, mant } = x else {
    return Ok(x);
  };
  let exp = i64::from(exp);
  // \__fp_decimate:nNnnnn { \c__fp_prec_int - #4 - #2 }
  let q = 16 - exp - places;
  if q <= 0 {
    return Ok(x);
  }
  let (kept, digit) = if q > 16 {
    // \__fp_decimate_tiny:Nnnnn
    (0u64, 1u64)
  } else {
    let p = POW10[q as usize];
    let (k, rest) = (mant / p, mant % p);
    let first = rest / (p / 10);
    let tail = rest % (p / 10);
    let d = if tail > 0 && (first == 0 || first == 5) {
      first + 1
    } else {
      first
    };
    (k, d)
  };
  let up = round_up(mode, sign, kept % 10, digit);
  let k = kept + u64::from(up);
  if k == 0 {
    // \__fp_round_special:NwwNnn
    return Ok(Fp::Zero(sign, Tag::Underflow));
  }
  if kept == 0 {
    // the digit 1 alone: `1 - #6`
    return normal(sign, 1 - places, POW10[15]);
  }
  // Kept digits number 16 - q; a carry past them is `1 + #5`.
  let width = 16 - q;
  if k == POW10[width as usize] {
    return normal(sign, 1 + exp, POW10[15]);
  }
  normal(sign, exp, k * POW10[q as usize])
}

/// The value of an exact small integer: `\__fp_small_int:wTF`, its magnitude capped at `10^8`.
pub(super) fn small_int(x: Fp) -> Option<i64> {
  match x {
    Fp::Zero(..) => Some(0),
    Fp::Inf(s) => Some(if s == Sign::Neg {
      -100_000_000
    } else {
      100_000_000
    }),
    Fp::Normal { sign, exp, mant } => {
      let exp = i64::from(exp);
      let q = 16 - exp;
      let (high, low, rest) = if q <= 0 {
        (mant / POW10[8], mant % POW10[8], 0)
      } else if q > 16 {
        return None;
      } else {
        let p = POW10[q as usize];
        let k = mant / p;
        (k / POW10[8], k % POW10[8], mant % p)
      };
      if rest != 0 {
        return None;
      }
      let v = if high > 0 { 100_000_000 } else { low as i64 };
      Some(if sign == Sign::Neg { -v } else { v })
    },
  }
}

/// The 16 digits of a normal number, as the four blocks of its form.
pub(super) fn blocks(mant: u64) -> [u32; 4] {
  [
    (mant / POW10[12]) as u32,
    (mant / POW10[8] % 10000) as u32,
    (mant / POW10[4] % 10000) as u32,
    (mant % 10000) as u32,
  ]
}

/// `\__fp_parse:n`'s reading of an integer, for a register's value (`#1 e #3`) and for `logb` (`\__fp_logb_aux_o:w`).
pub(super) fn from_int(v: i64, ten_exp: i64) -> R<Fp> {
  if v == 0 {
    return Ok(ZERO);
  }
  let sign = if v < 0 { Sign::Neg } else { Sign::Pos };
  let m = v.unsigned_abs();
  let len = (m.ilog10() + 1) as i64;
  if len > 16 {
    return Err(Decline("an integer past 16 digits"));
  }
  normal(sign, len + ten_exp, m * POW10[(16 - len) as usize])
}

#[cfg(test)]
mod tests {
  use super::*;

  fn f(sign: Sign, exp: i32, mant: u64) -> Fp { Fp::Normal { sign, exp, mant } }

  #[test]
  fn add_rounds_a_tie_to_even() {
    // 1 + 5e-16 is a tie below 1.000000000000000|5: the even 16th digit stays
    let one = f(Sign::Pos, 1, 1000000000000000);
    let half = f(Sign::Pos, -15, 5000000000000000);
    assert!(add(one, half).ok() == Some(one));
    // 1.000000000000001 + 5e-16 is the tie the other way: the odd digit goes up
    let next = f(Sign::Pos, 1, 1000000000000001);
    assert!(add(next, half).ok() == Some(f(Sign::Pos, 1, 1000000000000002)));
  }

  #[test]
  fn sub_cancels_to_the_exact_zero() {
    let x = f(Sign::Pos, 3, 1234567890123456);
    assert!(sub(x, x).ok() == Some(ZERO));
  }

  #[test]
  fn sqrt_of_an_exact_square_and_of_two() {
    let four = f(Sign::Pos, 1, 4000000000000000);
    assert!(sqrt(four).ok() == Some(f(Sign::Pos, 1, 2000000000000000)));
    let two = f(Sign::Pos, 1, 2000000000000000);
    assert!(sqrt(two).ok() == Some(f(Sign::Pos, 1, 1414213562373095)));
  }

  #[test]
  fn round_keeps_the_special_zero_and_the_carry() {
    let x = f(Sign::Pos, 0, 4000000000000000);
    assert!(round(Mode::Nearest, x, 0).ok() == Some(Fp::Zero(Sign::Pos, Tag::Underflow)));
    let nine = f(Sign::Pos, 1, 9500000000000000);
    assert!(round(Mode::Nearest, nine, 0).ok() == Some(f(Sign::Pos, 2, 1000000000000000)));
  }
}
