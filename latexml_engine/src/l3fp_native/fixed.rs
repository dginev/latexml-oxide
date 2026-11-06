//! l3fp-extended's fixed-point blocks and extended-precision numbers, one operation for one macro
//! (expl3-code.tex:19257-19705): a [`Fixed`] is six decimal blocks, the first with a weight of one, and an [`Ep`] a
//! fixed with a decimal exponent. These results are NOT correctly rounded, and that is the point: every function built
//! on them (`exp`, `ln`, `^`, `sin`, ...) truncates where the TeX truncates. `\numexpr`'s rounding division is [`rdiv`].

use super::num::POW10;

/// Six blocks of four digits (the first may have five): `{10000}{0000}{0000}{0000}{0000}{0000}` is 1.
pub(super) type Fixed = [i64; 6];
/// `\c__fp_one_fixed_tl`
pub(super) const ONE_FIXED: Fixed = [10000, 0, 0, 0, 0, 0];
/// An extended number `<exp>,<fixed>`.
pub(super) type Ep = (i64, Fixed);

const LEADING: i64 = -50000;
const MIDDLE: i64 = 50000 * 9999;
const TRAILING: i64 = 50000 * 10000;
const BIG_LEADING: i64 = -152374;
const BIG_MIDDLE: i64 = 152374 * 9999;
const BIG_TRAILING: i64 = 152374 * 10000;

/// `\numexpr a/b\relax`: the quotient rounded to nearest, a tie away from zero (eTeX's `x_over_n`-free division).
#[inline]
pub(super) fn rdiv(a: i64, b: i64) -> i64 {
  let (ua, ub) = (a.unsigned_abs(), b.unsigned_abs());
  let q = ((2 * ua + ub) / (2 * ub)) as i64;
  if (a < 0) != (b < 0) { -q } else { q }
}

/// The `#1#2` of two blocks read as one number: the digits side by side, so a negative first block keeps its minus
/// in front of them.
#[inline]
pub(super) fn concat(high: i64, low: i64) -> i64 {
  if high < 0 {
    high * 10000 - low
  } else {
    high * 10000 + low
  }
}

/// The carry and the last four digits of a shifted block (`\__fp_pack:NNNNNw`, `\__fp_pack_big:NNNNNNw`,
/// `\__fp_pack_Bigg:NNNNNNw`): the shift keeps the value positive, so these are its leading and trailing digits.
#[inline]
fn pack(v: i64) -> (i64, i64) {
  debug_assert!(v >= 0, "a shifted block went negative: {v}");
  (v / 10000, v % 10000)
}

/// `\__fp_fixed_div_myriad:wn`
pub(super) fn div_myriad(a: &Fixed) -> Fixed {
  let (c, l) = pack(TRAILING + a[0]);
  [LEADING + c, l, a[1], a[2], a[3], a[4]]
}

/// `\__fp_fixed_mul_short:wwn`: by a number of three blocks.
pub(super) fn mul_short(a: &Fixed, b: [i64; 3]) -> Fixed {
  let [a1, a2, a3, a4, a5, a6] = *a;
  let [b1, b2, b3] = b;
  let (c, l6) = pack(
    TRAILING + a4 * b3 + a5 * b2 + a6 * b1 + rdiv(a5 * b3 + a6 * b2 + rdiv(a6 * b3, 10000), 10000),
  );
  let (c, l5) = pack(MIDDLE + a3 * b3 + a4 * b2 + a5 * b1 + c);
  let (c, l4) = pack(MIDDLE + a2 * b3 + a3 * b2 + a4 * b1 + c);
  let (c, l3) = pack(MIDDLE + a1 * b3 + a2 * b2 + a3 * b1 + c);
  let (c, l2) = pack(MIDDLE + a1 * b2 + a2 * b1 + c);
  [LEADING + a1 * b1 + c, l2, l3, l4, l5, l6]
}

/// `\__fp_fixed_div_int:wwN`: by a small integer, by short division with each quotient digit one too low and the
/// final correction `+ 2` of `\__fp_fixed_div_int_auxii:wnn`.
pub(super) fn div_int(a: &Fixed, n: i64) -> Fixed {
  let mut q = [0i64; 6];
  let mut r = 0;
  for i in 0..6 {
    let v = if i == 0 { a[0] } else { r * 10000 + a[i] };
    q[i] = rdiv(v, n) - 1;
    r = v - q[i] * n;
  }
  let v6 = 9999 + q[5] + 2;
  let v5 = 9999 + q[4] + v6 / 10000;
  let v4 = 9999 + q[3] + v5 / 10000;
  let v3 = 9999 + q[2] + v4 / 10000;
  let v2 = 9999 + q[1] + v3 / 10000;
  [
    -1 + q[0] + v2 / 10000,
    v2 % 10000,
    v3 % 10000,
    v4 % 10000,
    v5 % 10000,
    v6 % 10000,
  ]
}

/// `\__fp_fixed_add_one:wN`
#[inline]
pub(super) fn add_one(a: &Fixed) -> Fixed {
  let mut r = *a;
  r[0] += 10000;
  r
}

fn add_or_sub(a: &Fixed, b: &Fixed, s: i64) -> Fixed {
  let l3 = 200_000_000 + s * concat(b[4], b[5]) + concat(a[4], a[5]);
  let l2 = 199_999_998 + concat(a[2], a[3]) + s * concat(b[2], b[3]) + l3 / 100_000_000;
  let l1 = 999_999_998 + concat(a[0], a[1]) + s * concat(b[0], b[1]) + l2 / 100_000_000;
  debug_assert!(
    l1 >= 1_000_000_000,
    "a fixed subtraction went negative: {l1}"
  );
  [
    (l1 - 1_000_000_000) / 10000,
    l1 % 10000,
    (l2 / 10000) % 10000,
    l2 % 10000,
    (l3 / 10000) % 10000,
    l3 % 10000,
  ]
}

/// `\__fp_fixed_add:wwn`
pub(super) fn add(a: &Fixed, b: &Fixed) -> Fixed { add_or_sub(a, b, 1) }
/// `\__fp_fixed_sub:wwn`
pub(super) fn sub(a: &Fixed, b: &Fixed) -> Fixed { add_or_sub(a, b, -1) }

/// `\__fp_fixed_mul:wwn`
pub(super) fn mul(a: &Fixed, b: &Fixed) -> Fixed {
  let [a1, a2, a3, a4, a5, a6] = *a;
  let [b1, b2, b3, b4, b5, b6] = *b;
  let (c, l6) = pack(
    TRAILING
      + a2 * b4
      + a3 * b3
      + a4 * b2
      + rdiv(
        a3 * b4 + a4 * b3 + a5 * b2 + a6 * b1 + a1 * b6 + a2 * b5,
        10000,
      )
      + a5 * b1
      + a1 * b5,
  );
  let (c, l5) = pack(MIDDLE + a1 * b4 + a2 * b3 + a3 * b2 + a4 * b1 + c);
  let (c, l4) = pack(MIDDLE + a1 * b3 + a2 * b2 + a3 * b1 + c);
  let (c, l3) = pack(MIDDLE + a1 * b2 + a2 * b1 + c);
  let (c, l2) = pack(MIDDLE + a1 * b1 + c);
  [LEADING + c, l2, l3, l4, l5, l6]
}

/// `\__fp_fixed_mul_add:Nwnnnwnnn` with its sign `s`: `c + s*a*b`.
fn mul_add_signed(a: &Fixed, b: &Fixed, c: &Fixed, s: i64) -> Fixed {
  let [a1, a2, a3, a4, a5, a6] = *a;
  let [b1, b2, b3, b4, b5, b6] = *b;
  let (carry, l6) = pack(
    BIG_TRAILING
      + s * (a5 * b1 + a4 * b2 + a3 * b3 + a2 * b4 + a1 * b5)
      + s
        * rdiv(
          a6 * b1 + a5 * b2 + a4 * b3 + a3 * b4 + a2 * b5 + a1 * b6,
          10000,
        )
      + concat(c[4], c[5]),
  );
  let (carry, l5) = pack(BIG_MIDDLE + s * (a1 * b4 + a2 * b3 + a3 * b2 + a4 * b1) + carry);
  let (carry, l4) =
    pack(BIG_MIDDLE + s * (a1 * b3 + a2 * b2 + a3 * b1) + concat(c[2], c[3]) + carry);
  let (carry, l3) = pack(BIG_MIDDLE + s * (a1 * b2 + a2 * b1) + carry);
  let (carry, l2) = pack(BIG_MIDDLE + concat(c[0], c[1]) + s * (a1 * b1) + carry);
  [BIG_LEADING + carry, l2, l3, l4, l5, l6]
}

/// `\__fp_fixed_mul_sub_back:wwwn`: `c - a*b`.
pub(super) fn mul_sub_back(a: &Fixed, b: &Fixed, c: &Fixed) -> Fixed { mul_add_signed(a, b, c, -1) }

/// `\__fp_ep_to_fixed:wwn`: `<exp>,<fixed>` read as a fixed, its digits shifted by the exponent, those past the sixth
/// block gone. The digits stand in a string in the TeX, so a first block of twelve digits is one number here.
pub(super) fn ep_to_fixed(ep: &Ep) -> Fixed {
  let (e, f) = ep;
  let zeros = (4 - (*e).max(-32)).max(0) as usize;
  let mut digits = [0u8; 80];
  let mut n = zeros;
  n += push_digits(&mut digits[n..], f[0], 8);
  for block in &f[1..] {
    n += push_digits(&mut digits[n..], *block, 4);
  }
  // `0000 !` and the blocks past the sixth: all zero or dropped
  let read = |from: usize, to: usize| -> i64 {
    digits[from..to]
      .iter()
      .fold(0, |v, d| v * 10 + i64::from(*d))
  };
  debug_assert!(n + 4 >= 32);
  [
    read(0, 12),
    read(12, 16),
    read(16, 20),
    read(20, 24),
    read(24, 28),
    read(28, 32),
  ]
}

/// Write `v` as `width` decimal digits at the start of `out`; the number written.
fn push_digits(out: &mut [u8], mut v: i64, width: usize) -> usize {
  for i in (0..width).rev() {
    out[i] = (v % 10) as u8;
    v /= 10;
  }
  width
}

/// The digits of a fixed as the TeX lays them out (`1 0000 0000 + #1`, then the blocks): 28 digits, in a `u128`.
#[inline]
pub(super) fn fixed_digits(f: &Fixed) -> u128 {
  let mut v = f[0] as u128;
  for block in &f[1..] {
    v = v * 10000 + *block as u128;
  }
  v
}

/// `\__fp_ep_to_ep:wwN`: normalized, with a first block of four digits whose first is not zero; a zero number takes
/// the exponent `- 2 * \c__fp_max_exponent_int` and the blocks `{1000}{0000}...` of `\__fp_ep_to_ep_zero:ww`.
pub(super) fn ep_to_ep(ep: &Ep) -> Ep {
  let (e, f) = ep;
  let v = fixed_digits(f);
  if v == 0 {
    // 28 leading zeros counted down, and then the zero's mark
    return (e + 4 - 28 - 20000, [1000, 0, 0, 0, 0, 0]);
  }
  let zeros = 28 - decimal_len(v) as i64;
  // the digits from the first non-zero one, padded with zeros; past 24 digits they are dropped
  let shifted = if zeros >= 4 {
    v * 10u128.pow(zeros as u32 - 4)
  } else {
    v / 10u128.pow(4 - zeros as u32)
  };
  let blocks = shifted;
  (e + 4 - zeros, split_blocks(blocks))
}

/// The number of decimal digits of `v`, at least 1.
#[inline]
pub(super) fn decimal_len(v: u128) -> u32 {
  if v == 0 {
    return 1;
  }
  v.ilog10() + 1
}

/// A number of at most 24 digits as six blocks of four.
fn split_blocks(mut v: u128) -> Fixed {
  let mut out = [0i64; 6];
  for slot in out.iter_mut().rev() {
    *slot = (v % 10000) as i64;
    v /= 10000;
  }
  out
}

/// `\__fp_ep_compare:wwww`: -1, 0 or 1.
pub(super) fn ep_compare(a: &Ep, b: &Ep) -> i32 {
  let key = |ep: &Ep| {
    (
      ep.0,
      concat(ep.1[0], ep.1[1]),
      concat(ep.1[2], ep.1[3]),
      concat(ep.1[4], ep.1[5]),
    )
  };
  match key(a).cmp(&key(b)) {
    std::cmp::Ordering::Less => -1,
    std::cmp::Ordering::Equal => 0,
    std::cmp::Ordering::Greater => 1,
  }
}

/// `\__fp_ep_mul_raw:wwwwN`
pub(super) fn ep_mul_raw(a: &Ep, b: &Ep) -> Ep { (a.0 + b.0, mul(&a.1, &b.1)) }

/// `\__fp_ep_mul:wwwwn`
pub(super) fn ep_mul(a: &Ep, b: &Ep) -> Ep {
  let b = ep_to_ep(b);
  let a = ep_to_ep(a);
  ep_mul_raw(&a, &b)
}

/// `\__fp_ep_div:wwwwn`: `a/b`, by an estimate of `1/b` of `\__fp_ep_div_estiii` refined by `\__fp_ep_div_epsii`.
pub(super) fn ep_div(a: &Ep, b: &Ep) -> Ep {
  let a = ep_to_ep(a);
  let (be, bf) = ep_to_ep(b);
  let q0 = rdiv(1_000_000_000, bf[0] + 1);
  let exp = a.0 - be + 1;
  let x =
    1_000_000_000 - 1750 + q0 * 1000 + (rdiv(1_000_000_000, bf[0]) - q0) * (1000 - rdiv(bf[1], 10));
  // `1#1#2#3#4#5#6`: ten digits, the 1 dropped
  let digits = x - 1_000_000_000;
  debug_assert!((0..1_000_000_000).contains(&digits));
  let d1 = digits / 100_000_000; // #1
  let d2 = (digits / 10000) % 10000; // #2#3#4#5
  let d3 = digits % 10000; // #6
  let product = mul_short(&bf, [d1, d2, d3]);
  // `\__fp_ep_div_epsi:wnNNNNNn`: 1 - product; the product's first block, 1, is not read
  let l3 = 200_000_000 - concat(product[4], product[5]);
  let l2 = 199_999_998 - concat(product[2], product[3]) + l3 / 100_000_000;
  let l1 = 19998 - product[1] + l2 / 100_000_000;
  debug_assert!((10000..20000).contains(&l1));
  let eps: Fixed = [
    0,
    l1 - 10000,
    (l2 / 10000) % 10000,
    l2 % 10000,
    (l3 / 10000) % 10000,
    l3 % 10000,
  ];
  let y_abcd = digits / 100000; // #1#2#3#4
  let y_efgh = (digits / 10) % 10000; // #5 and the first three of #6
  let y_i = digits % 10; // the last of #6
  let mut one_plus_eps = eps;
  one_plus_eps[0] = 10000;
  let r = mul(&add_one(&mul(&eps, &eps)), &one_plus_eps);
  let d = div_myriad(&mul_short(&eps, [y_abcd, y_efgh, y_i * 1000]));
  let inverse = add(&mul(&d, &r), &[y_abcd, y_efgh, y_i * 1000, 0, 0, 0]);
  (exp, mul(&inverse, &a.1))
}

/// `\__fp_ep_isqrt:wwn`: `1/sqrt(x)`: an estimate by Newton's method on integers, then three refinements of
/// `\__fp_ep_isqrt_epsii:wwN`.
pub(super) fn ep_isqrt(ep: &Ep) -> Ep {
  let (e, x) = ep_to_ep(ep);
  let odd = e.rem_euclid(2) == 1;
  let (exp, start) = if odd {
    (rdiv(1 - e, 2), 535)
  } else {
    (1 - rdiv(e, 2), 168)
  };
  let numerator: i64 = if odd { 1_005_000_000 } else { 100_500_000 };
  let (mut y, mut prev) = (start, 0);
  while y != prev {
    let next = rdiv(y + rdiv(numerator, y * x[0]), 2);
    prev = y;
    y = next;
  }
  let v = 10_000_000 + y * (if odd { y } else { y * 10 }) * 5;
  let w = 10000 + y;
  let digits = v - 10_000_000;
  let (v1, v2, v3) = (digits / 1_000_000, (digits / 100) % 10000, digits % 100);
  let epsilon = mul_short(&x, [v1, v2, v3 * 100]);
  let one_and_half: Fixed = [15000, 0, 0, 0, 0, 0];
  let mut s = sub(&one_and_half, &epsilon);
  for _ in 0..3 {
    let square = mul(&s, &s);
    s = mul(&mul_sub_back(&square, &epsilon, &one_and_half), &s);
  }
  let (w1, w2) = (w % 10000 / 1000, w % 1000 * 10);
  (exp, mul_short(&s, [w1, w2, 0]))
}

/// `\__fp_fixed_to_float_o:wN`: the exponent it adds (`\c__fp_block_int` less the leading zeros) and the sixteen digits
/// rounded on the seventeenth alone (`\__fp_fixed_to_float_pack:ww`, a half going up), or `None` for a zero, the
/// `- 2 * \c__fp_max_exponent_int` of `\__fp_fixed_to_float_zero:w` the caller's sanitize turns into an underflow.
pub(super) fn to_float(f: &Fixed) -> Option<(i64, u64)> {
  let v = fixed_digits(f);
  if v == 0 {
    return None;
  }
  let n = decimal_len(v);
  debug_assert!(n <= 28, "a fixed's first block overflowed: {v}");
  let mut exp = 4 - (28 - i64::from(n));
  let (mut mant, next) = if n >= 17 {
    let scaled = v / 10u128.pow(n - 17);
    ((scaled / 10) as u64, (scaled % 10) as u64)
  } else {
    ((v * 10u128.pow(16 - n)) as u64, 0)
  };
  if next > 4 {
    mant += 1;
    if mant == POW10[16] {
      mant = POW10[15];
      exp += 1;
    }
  }
  Some((exp, mant))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn isqrt_of_three_quarters_and_of_seven_and_a_half() {
    let (e, f) = ep_isqrt(&(0, [7500, 0, 0, 0, 0, 0]));
    let (e2, f2) = ep_isqrt(&(1, [7500, 0, 0, 0, 0, 0]));
    assert_eq!((e, f[0]), (1, 1154));
    assert_eq!((e2, f2[0]), (0, 3651));
  }
}
