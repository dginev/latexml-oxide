//! A native `\__fp_parse:n` (expl3-code.tex:17313), the one door of l3fp: `\fp_set:Nn`, `\fp_eval:n`, `\fp_compare:nNnTF`,
//! xfp's `\fpeval`, siunitx and l3draw all come through it. It takes one `\fp_set:Nn` from about 40k token reads to a few
//! hundred (`pgf-interference`'s pixel loop, docs/performance/PERFORMANCE.md).
//!
//! **The contract.** The expansion is exactly the tokens the macro's f-expansion yields,
//! `\s__fp \__fp_chk:w <kind> <sign> {<exp>}{<b1>}{<b2>}{<b3>}{<b4>} \__fp_sep:`, returned as a list through the ordinary
//! path of a macro call (`invoke_expansion`, whose `inherit_field_prov` a printed `\@title` needs). On anything the
//! native path does not handle with certainty (`Decline`: an unknown token, a tuple, `nan`, `rand`, a flag or a
//! message of any kind), the call expands to the raw macro on the original argument tokens, untouched; `parse.rs` says
//! why this can have no side effect before then. So the behaviour cannot change, only the cost.
//!
//! **The arithmetic is a port.** `+ - * /` and `sqrt` round correctly in the TeX, so exact integers give the same digits
//! (`num.rs`). `exp`, `ln`, `^` and the trigonometric functions do not, so `func.rs` and `fixed.rs` translate their block
//! operations and tables one for one, truncations included. No `f64` comes near a value.
//!
//! `LATEXML_L3FP_NATIVE=0` keeps the raw macro. `LATEXML_L3FP_CHECK=1` runs both on every call and reports an Error naming
//! the argument if the token lists differ (`kit/l3fp_oracle.sh` holds the arithmetic against pdflatex as well).
//! `LATEXML_L3FP_TRACE=1` counts answers and fallbacks by reason, printed on stderr at the thread's end.
//!
//! Installed at the format's end, after the dump's apply: the dump does not change, and a later raw load of
//! `expl3-code.tex` simply wins. Only over the l3fp this port was made from (`native_applies`): another year's
//! `\__fp_parse:n` keeps its own macro. OXIDIZED_DESIGN_DIVERGENCES #973.

mod fixed;
mod func;
mod num;
mod parse;
mod tables;

use std::{cell::RefCell, collections::HashMap};

use num::{Decline, Fp, Sign, Tag};
use once_cell::sync::Lazy;

use crate::prelude::*;

/// The private name of the raw `\__fp_parse:n`, whatever that year's definition.
const RAW: &str = "\\lx@fp@parse@raw:n";

/// `\ExplFileDate`s whose l3fp this module ports and has been held against, and whether `\__fp_parse:n` wraps its work in
/// `\group_align_safe_begin:` there: expl3-code.tex of 2025-11-06 (TL 2025) and 2026-01-19, whose l3fp differs in only that
/// (`\__fp_parse:n`, `\__fp_parse_o:n`). An `&` in the argument is safe here anyway: the argument is read as a macro's.
const PORTED: &[(&str, bool)] = &[("2025-11-06", false), ("2026-01-19", true)];

static CHECK: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_L3FP_CHECK").as_deref() == Ok("1"));
static TRACE: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_L3FP_TRACE").as_deref() == Ok("1"));

thread_local! {
  /// This thread's choice of the native path, consulted before `LATEXML_L3FP_NATIVE`: a test sets it instead of
  /// mutating the process environment (as `pgfkeys_code_tex::set_native_override`).
  static NATIVE_OVERRIDE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
  /// The same for `LATEXML_L3FP_CHECK`.
  static CHECK_OVERRIDE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
  /// The census of `LATEXML_L3FP_TRACE=1`. An ordinary `thread_local!` for its destructor, which prints it.
  static CENSUS: RefCell<Census> = RefCell::new(Census::default());
}

/// Force the native path on or off for conversions on this thread (`None`: `LATEXML_L3FP_NATIVE` decides, default on).
pub fn set_native_override(native: Option<bool>) { NATIVE_OVERRIDE.with(|o| o.set(native)); }

/// Force the self-check on or off for conversions on this thread (`None`: `LATEXML_L3FP_CHECK` decides, default off).
pub fn set_check_override(check: Option<bool>) { CHECK_OVERRIDE.with(|o| o.set(check)); }

fn check_enabled() -> bool { CHECK_OVERRIDE.with(std::cell::Cell::get).unwrap_or(*CHECK) }

fn native_enabled() -> bool {
  NATIVE_OVERRIDE
    .with(std::cell::Cell::get)
    .unwrap_or_else(|| std::env::var("LATEXML_L3FP_NATIVE").as_deref() != Ok("0"))
}

/// Answers and fallbacks by reason (`LATEXML_L3FP_TRACE=1`).
#[derive(Default)]
struct Census {
  native:   u64,
  fallback: HashMap<String, u64>,
  sample:   HashMap<String, String>,
}

impl Drop for Census {
  fn drop(&mut self) {
    if self.native == 0 && self.fallback.is_empty() {
      return;
    }
    eprintln!("L3FP native {}", self.native);
    for (reason, n) in &self.fallback {
      eprintln!("L3FP fallback {n} {reason}");
      eprintln!(
        "L3FP sample {reason} <- {}",
        self.sample.get(reason).map_or("", String::as_str)
      );
    }
  }
}

fn note_native() { CENSUS.with(|c| c.borrow_mut().native += 1); }

fn note_fallback(decline: Decline, at: Option<Token>, via: Option<Token>, arg: &[Token]) {
  let at = at.map(|t| t.to_string()).unwrap_or_default();
  let via = via.map(|t| format!(" via {t}")).unwrap_or_default();
  let key = format!("{} {at}{via}", decline.0);
  CENSUS.with(|c| {
    let mut census = c.borrow_mut();
    *census.fallback.entry(key.clone()).or_insert(0) += 1;
    census.sample.entry(key).or_insert_with(|| {
      Tokens::new(arg.to_vec())
        .to_string()
        .replace('\n', " ")
        .chars()
        .take(160)
        .collect()
    });
  });
}

/// A digit, or a `-`, of a number's form.
fn other_digits(out: &mut Vec<Token>, v: i64, width: usize) {
  let mut buf = [b'0'; 20];
  let mut n = 0;
  let mut u = v.unsigned_abs();
  while u > 0 || n == 0 {
    buf[n] = b'0' + (u % 10) as u8;
    u /= 10;
    n += 1;
  }
  n = n.max(width);
  if v < 0 {
    out.push(T_OTHER!("-"));
  }
  for i in (0..n).rev() {
    out.push(digit_token(buf[i]));
  }
}

#[inline]
fn digit_token(d: u8) -> Token {
  match d {
    b'0' => T_OTHER!("0"),
    b'1' => T_OTHER!("1"),
    b'2' => T_OTHER!("2"),
    b'3' => T_OTHER!("3"),
    b'4' => T_OTHER!("4"),
    b'5' => T_OTHER!("5"),
    b'6' => T_OTHER!("6"),
    b'7' => T_OTHER!("7"),
    b'8' => T_OTHER!("8"),
    _ => T_OTHER!("9"),
  }
}

/// The form of a value, as `\__fp_exp_after_o:w` leaves it.
fn fp_tokens(x: Fp) -> Vec<Token> {
  let mut out = Vec::with_capacity(36);
  out.push(T_CS!("\\s__fp"));
  out.push(T_CS!("\\__fp_chk:w"));
  let sign_digit = |s: Sign| {
    if s == Sign::Neg {
      T_OTHER!("2")
    } else {
      T_OTHER!("0")
    }
  };
  match x {
    Fp::Zero(s, tag) => {
      out.push(T_OTHER!("0"));
      out.push(sign_digit(s));
      out.push(if tag == Tag::Exact {
        T_CS!("\\s__fp_exact")
      } else {
        T_CS!("\\s__fp_underflow")
      });
    },
    Fp::Inf(s) => {
      out.push(T_OTHER!("2"));
      out.push(sign_digit(s));
      out.push(T_CS!("\\s__fp_exact"));
    },
    Fp::Normal { sign, exp, mant } => {
      out.push(T_OTHER!("1"));
      out.push(sign_digit(sign));
      out.push(T_BEGIN!());
      other_digits(&mut out, i64::from(exp), 0);
      out.push(T_END!());
      for b in num::blocks(mant) {
        out.push(T_BEGIN!());
        other_digits(&mut out, i64::from(b), 4);
        out.push(T_END!());
      }
    },
  }
  out.push(T_CS!("\\__fp_sep:"));
  out
}

/// `\__fp_to_decimal:w`: the digits, no exponent, no trailing zeros. An infinity is that function's invalid operation.
fn to_decimal(x: Fp) -> num::R<String> {
  let (sign, exp, mant) = match x {
    Fp::Zero(s, _) => return Ok(String::from(if s == Sign::Neg { "-0" } else { "0" })),
    Fp::Inf(_) => return Err(Decline("invalid: the decimal of inf")),
    Fp::Normal { sign, exp, mant } => (sign, i64::from(exp), mant),
  };
  let mut out = String::with_capacity(24);
  if sign == Sign::Neg {
    out.push('-');
  }
  let digits = mant.to_string();
  if exp >= 16 {
    // \__fp_to_decimal_huge:wnnnn
    out.push_str(&digits);
    for _ in 16..exp {
      out.push('0');
    }
  } else {
    if exp > 0 {
      out.push_str(&digits[..exp as usize]);
      out.push('.');
      out.push_str(&digits[exp as usize..]);
    } else {
      out.push_str("0.");
      for _ in exp..0 {
        out.push('0');
      }
      out.push_str(&digits);
    }
    // \__fp_trim_zeros:w
    let trimmed = out.trim_end_matches('0').trim_end_matches('.').len();
    out.truncate(trimmed);
  }
  Ok(out)
}

/// The body of `\__fp_parse:n` (expl3-code.tex:17313-17322), parameter numbered.
fn expected_body(align_safe: bool) -> Vec<Token> {
  let cs = |s: &str| T_CS!(s);
  let mut body = vec![cs("\\exp:w")];
  if align_safe {
    body.extend([
      cs("\\group_align_safe_begin:"),
      cs("\\exp_after:wN"),
      cs("\\group_align_safe_end:"),
    ]);
  }
  body.extend([
    cs("\\exp_after:wN"),
    cs("\\__fp_parse_after:ww"),
    cs("\\exp:w"),
    cs("\\__fp_parse_operand:Nw"),
    cs("\\c__fp_prec_end_int"),
    cs("\\__fp_parse_expand:w"),
    Token::new("1", Catcode::ARG),
    cs("\\s__fp_expr_mark"),
    cs("\\__fp_parse_infix_end:N"),
    cs("\\s__fp_expr_stop"),
    cs("\\exp_end:"),
  ]);
  body
}

/// Is the current l3fp the one this module ports: the expl3 date, and the door's own body?
fn native_applies() -> Result<bool> {
  let Some(defn) = lookup_definition(&T_CS!("\\__fp_parse:n"))? else {
    return Ok(false);
  };
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return Ok(false);
  };
  let Some(date) = lookup_definition(&T_CS!("\\ExplLoaderFileDate"))? else {
    return Ok(false);
  };
  let Some(ExpansionBody::Tokens(date)) = date.get_expansion() else {
    return Ok(false);
  };
  let date = date.to_string();
  Ok(PORTED.iter().any(|(d, align_safe)| {
    *d == date && body.unlist_ref() == expected_body(*align_safe).as_slice()
  }))
}

/// The raw macro over the original argument.
fn raw_call(arg: &[Token]) -> Tokens {
  let mut toks = Vec::with_capacity(arg.len() + 3);
  toks.push(T_CS!(RAW));
  toks.push(T_BEGIN!());
  toks.extend_from_slice(arg);
  toks.push(T_END!());
  Tokens::new(toks)
}

/// `LATEXML_L3FP_CHECK=1`: the raw macro's own f-expansion (`\exp_not:f`) against the native answer. Returns the raw one.
fn checked(arg: &[Token], native: &[Token]) -> Result<Tokens> {
  let mut toks = vec![T_CS!("\\exp_not:f"), T_BEGIN!()];
  toks.extend(raw_call(arg).unlist());
  toks.push(T_END!());
  // as an `\edef` reads it: `\exp_not:f` holds the value back, whose `\__fp_chk:w` a full expansion would run
  let raw = do_expand_partially(Tokens::new(toks))?;
  if raw.unlist_ref() != native {
    let message = s!(
      "native and raw l3fp differ on {{{}}}: native {} raw {}",
      Tokens::new(arg.to_vec()),
      Tokens::new(native.to_vec()),
      raw
    );
    Error!("l3fp", "native-mismatch", message);
  }
  Ok(raw)
}

/// `\__fp_parse:n`'s native body.
fn parse_call(arg: Tokens) -> Result<Tokens> {
  let tokens = arg.unlist_ref();
  let mut parser = parse::Parser::new(tokens, 0);
  match parser.parse() {
    Ok(x) => {
      if *TRACE {
        note_native();
      }
      let form = fp_tokens(x);
      if check_enabled() {
        return checked(tokens, &form);
      }
      Ok(Tokens::new(form))
    },
    Err(decline) => {
      if *TRACE {
        note_fallback(decline, parser.at, parser.via, tokens);
      }
      Ok(raw_call(tokens))
    },
  }
}

/// Install the native `\__fp_parse:n` over the format's, its raw definition kept under [`RAW`].
pub(crate) fn install() -> Result<()> {
  if !native_enabled() || !native_applies()? {
    return Ok(());
  }
  let_i(&T_CS!(RAW), &T_CS!("\\__fp_parse:n"), None);
  DefMacro!("\\__fp_parse:n{}", sub[(arg)] { parse_call(arg) });
  Ok(())
}
