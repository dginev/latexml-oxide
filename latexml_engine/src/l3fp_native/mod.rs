//! A native answer to `\__fp_parse:n` and `\__fp_parse_o:n` (expl3-code.tex:17313-17339), the doors of l3fp: `\fp_set:Nn`,
//! `\fp_eval:n`, `\fp_compare:nNnTF`, xfp's `\fpeval`, siunitx and l3draw all come through them. It takes one `\fp_set:Nn`
//! from about 40k token reads to a few hundred (`pgf-interference`'s pixel loop, docs/performance/PERFORMANCE.md).
//!
//! **The hook.** Both doors stay the format's own macros. Their expansion is
//! `… \exp:w \__fp_parse_operand:Nw \c__fp_prec_end_int \__fp_parse_expand:w <expression> \s__fp_expr_mark
//! \__fp_parse_infix_end:N \s__fp_expr_stop …`, the only such call. `\__fp_parse_operand:Nw` keeps its body too, and carries
//! a native call (`operand_call`, `SpecialCall::Native`): when the gullet expands it in full and the input is that
//! shape, the expression is read from the pushback without consuming it. The answer leaves what the macro's own run
//! leaves for `\__fp_parse_after:ww`, `\exp_end_continue_f:w <fp>@ \__fp_parse_infix_end:N` (`\s__fp_expr_stop` still on the
//! input), through the ordinary path of a macro's expansion (`invoke_expansion`, whose `inherit_field_prov` a printed
//! `\@title` needs). On any other call, and on anything the native path does not handle with certainty (`Decline`: an
//! unknown token, a tuple, `nan`, `rand`, a flag or a message of any kind), nothing has happened and the macro runs as
//! defined over the input as it stands. So `\meaning`, the prefixes and a one-step `\exp_after:wN` of either door and of
//! this macro are the format's, and a tuple, a flag or a message is the macro's own.
//!
//! **While it may answer.** Only while l3fp is what this module ports: the meanings of the `\__fp_…`, `\c__fp_…`, `\s__fp…`
//! and `\c_…_fp` names are watched ([`watch_meanings`]), and an assignment that changes one, however local, ends the native
//! path for the rest of the conversion. A document's new word (`\fp_new_variable:n`) has no meaning to change: the parser
//! declines any word it does not know. So a control sequence the parser knows by name means what it did at the install.
//!
//! **The arithmetic is a port.** `+ - * /` and `sqrt` round correctly in the TeX, so exact integers give the same digits
//! (`num.rs`). `exp`, `ln`, `^` and the trigonometric functions do not, so `func.rs` and `fixed.rs` translate their block
//! operations and tables one for one, truncations included. No `f64` comes near a value.
//!
//! `LATEXML_L3FP_NATIVE=0` installs nothing. `LATEXML_L3FP_CHECK=1` runs the macro's own run on every answer and reports an
//! Error naming the argument if the tokens differ; it compares tokens only, not what a run reports (`kit/l3fp_oracle.sh`
//! holds the arithmetic against pdflatex as well). `LATEXML_L3FP_TRACE=1` counts answers and fallbacks by reason, printed on
//! stderr at the thread's end.
//!
//! Installed at the format's end, after the dump's apply: the dump does not change, and a later raw load of
//! `expl3-code.tex` simply wins (and trips the watch). Only over the l3fp this port was made from (`native_applies`): another
//! year's l3fp stays entirely the macros'. No code here relies on unwinding (`maxperf` aborts on a panic). OXIDIZED_DESIGN_DIVERGENCES #977.

mod fixed;
mod func;
mod num;
mod parse;
mod tables;

use std::{
  cell::{Cell, RefCell},
  collections::HashMap,
};

use latexml_core::definition::expandable::{SpecialCall, set_native_call};
use num::{Decline, Fp, Sign, Tag};
use once_cell::sync::Lazy;

use crate::prelude::*;

/// `\ExplFileDate`s whose l3fp this module ports and has been held against, and whether `\__fp_parse:n` wraps its work in
/// `\group_align_safe_begin:` there: expl3-code.tex of 2025-11-06 (TL 2025) and 2026-01-19, whose l3fp differs in only that
/// (`\__fp_parse:n`, `\__fp_parse_o:n`).
const PORTED: &[(&str, bool)] = &[("2025-11-06", false), ("2026-01-19", true)];

/// The levels of expansion recursion the macro's own run over the deepest expression the parser answers may take
/// (`parse::MAX_DEPTH`): with fewer left it runs and reports, as it would.
const RAW_DEPTH_RESERVE: usize = 6000;

/// What l3fp assigns itself as it runs, or what only a flag's path reaches, which the parser declines: not watched. The
/// scratch macro, a user function's argument reader, and what `\fp_trap:nn` sets (expl3-code.tex:16058-16134).
const NOT_WATCHED: &[&str] = &[
  "\\__fp_tmp:w",
  "\\__fp_function_arg_o:w",
  "\\__fp_invalid_operation:nnw",
  "\\__fp_invalid_operation_o:Nww",
  "\\__fp_invalid_operation_tl_o:ff",
  "\\__fp_division_by_zero_o:Nnw",
  "\\__fp_division_by_zero_o:NNww",
  "\\__fp_overflow:w",
  "\\__fp_underflow:w",
];

/// Is `name` one of l3fp's own control sequences whose meaning the answers depend on? The marks `\s__fp…` are `\relax` by
/// meaning (`\scan_new:N`); any other is defined otherwise, so a `\relax` on it comes from a `\csname` over a word still to
/// define. (Asked of every meaning assignment: the first test is by one byte.)
fn watched_name(name: &str) -> WatchedName {
  let family = match name.as_bytes().get(1) {
    Some(b'_') => name.starts_with("\\__fp_"),
    Some(b's') => {
      if name.starts_with("\\s__fp") {
        return WatchedName::Yes;
      }
      false
    },
    Some(b'c') => {
      name.starts_with("\\c_") && (name.starts_with("\\c__fp_") || name.ends_with("_fp"))
    },
    _ => false,
  };
  if family && !NOT_WATCHED.contains(&name) {
    WatchedName::RelaxIsUnset
  } else {
    WatchedName::No
  }
}

static CHECK: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_L3FP_CHECK").as_deref() == Ok("1"));
static TRACE: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_L3FP_TRACE").as_deref() == Ok("1"));

thread_local! {
  /// This thread's choice of the native path, consulted before `LATEXML_L3FP_NATIVE`: a test sets it instead of
  /// mutating the process environment (as `pgfkeys_code_tex::set_native_override`).
  static NATIVE_OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
  /// The same for `LATEXML_L3FP_CHECK`.
  static CHECK_OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
  /// The census of `LATEXML_L3FP_TRACE=1`. An ordinary `thread_local!` for its destructor, which prints it.
  static CENSUS: RefCell<Census> = RefCell::new(Census::default());
  /// How many doors this conversion has answered natively, and how many it left to the macro ([`native_counts`]).
  static COUNTS: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
  /// This conversion's format took the native call.
  static INSTALLED: Cell<bool> = const { Cell::new(false) };
  /// The self-check's own run of the macro is going on: it is the macro's, all of it.
  static CHECKING: Cell<bool> = const { Cell::new(false) };
}

/// How many doors this thread's last conversion answered natively and how many it left to the macro, what a guard asserts so
/// as not to hold the macro against itself; `None` if nothing was installed: the switch, or a TeX tree's l3fp not ported.
pub fn native_counts() -> Option<(u64, u64)> {
  INSTALLED.with(Cell::get).then(|| COUNTS.with(Cell::get))
}

/// Force the native path on or off for conversions on this thread (`None`: `LATEXML_L3FP_NATIVE` decides, default on).
pub fn set_native_override(native: Option<bool>) { NATIVE_OVERRIDE.with(|o| o.set(native)); }

/// Force the self-check on or off for conversions on this thread (`None`: `LATEXML_L3FP_CHECK` decides, default off).
pub fn set_check_override(check: Option<bool>) { CHECK_OVERRIDE.with(|o| o.set(check)); }

fn check_enabled() -> bool { CHECK_OVERRIDE.with(Cell::get).unwrap_or(*CHECK) }

fn native_enabled() -> bool {
  NATIVE_OVERRIDE
    .with(Cell::get)
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

/// `input`: the pushback's slice of the argument, its first token last.
fn note_fallback(decline: Decline, at: Option<Token>, via: Option<Token>, input: &[Token]) {
  let at = at.map(|t| t.to_string()).unwrap_or_default();
  let via = via.map(|t| format!(" via {t}")).unwrap_or_default();
  let key = format!("{} {at}{via}", decline.0);
  CENSUS.with(|c| {
    let mut census = c.borrow_mut();
    *census.fallback.entry(key.clone()).or_insert(0) += 1;
    census.sample.entry(key).or_insert_with(|| {
      Tokens::new(input.iter().rev().copied().collect())
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

/// The form of a value, as `\__fp_exp_after_o:w` leaves it, onto `out`.
fn fp_tokens(x: Fp, out: &mut Vec<Token>) {
  out.reserve(36);
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
      other_digits(out, i64::from(exp), 0);
      out.push(T_END!());
      for b in num::blocks(mant) {
        out.push(T_BEGIN!());
        other_digits(out, i64::from(b), 4);
        out.push(T_END!());
      }
    },
  }
  out.push(T_CS!("\\__fp_sep:"));
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
fn expected_door(align_safe: bool) -> Vec<Token> {
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

/// The body of `\__fp_parse_operand:Nw` (expl3-code.tex:17340).
fn expected_operand() -> Vec<Token> {
  let cs = |s: &str| T_CS!(s);
  let prec = Token::new("1", Catcode::ARG);
  vec![
    cs("\\exp_end_continue_f:w"),
    cs("\\exp_after:wN"),
    cs("\\__fp_parse_continue:NwN"),
    cs("\\exp_after:wN"),
    prec,
    cs("\\exp:w"),
    cs("\\exp_end_continue_f:w"),
    cs("\\exp_after:wN"),
    cs("\\__fp_parse_one:Nw"),
    cs("\\exp_after:wN"),
    prec,
    cs("\\exp:w"),
  ]
}

/// The body of `\__fp_parse_infix_end:N` (expl3-code.tex:17482).
fn expected_infix_end() -> Vec<Token> {
  vec![
    T_LETTER!("@"),
    T_CS!("\\use_none:n"),
    T_CS!("\\__fp_parse_infix_end:N"),
  ]
}

/// The body of the macro `cs`, as stored.
fn body_of(cs: &Token) -> Result<Option<Tokens>> {
  Ok(match lookup_definition(cs)? {
    Some(defn) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
      _ => None,
    },
    None => None,
  })
}

/// Is the current l3fp the one this module ports: the expl3 date, and the bodies of the door and of what the answer stands
/// for?
fn native_applies() -> Result<bool> {
  let (Some(door), Some(operand), Some(infix_end), Some(date)) = (
    body_of(&T_CS!("\\__fp_parse:n"))?,
    body_of(&T_CS!("\\__fp_parse_operand:Nw"))?,
    body_of(&T_CS!("\\__fp_parse_infix_end:N"))?,
    body_of(&T_CS!("\\ExplLoaderFileDate"))?,
  ) else {
    return Ok(false);
  };
  let date = date.to_string();
  Ok(
    operand.unlist_ref() == expected_operand().as_slice()
      && infix_end.unlist_ref() == expected_infix_end().as_slice()
      && PORTED.iter().any(|(d, align_safe)| {
        *d == date && door.unlist_ref() == expected_door(*align_safe).as_slice()
      }),
  )
}

/// The door's shape on the input: the index of its `\s__fp_expr_mark`, or `None`. The argument is what lies between, its
/// braces as they are. `\s__fp_expr_mark` in the argument, or `\s__fp_expr_stop` not next, is no door of the macro's.
fn door_mark(input: &[Token]) -> Option<usize> {
  let mut depth = 0u32;
  let mut i = input.len().checked_sub(2)?;
  while i > 0 {
    i -= 1;
    let t = input[i];
    match t.code {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.checked_sub(1)?,
      Catcode::CS if depth == 0 && t == T_CS!("\\s__fp_expr_mark") => {
        return (i >= 2
          && input[i - 1] == T_CS!("\\__fp_parse_infix_end:N")
          && input[i - 2] == T_CS!("\\s__fp_expr_stop"))
        .then_some(i);
      },
      _ => {},
    }
  }
  None
}

/// `LATEXML_L3FP_CHECK=1`: the macro's own run over the same input, what it leaves against the native answer. Returns what
/// the macro's run leaves, `\s__fp_expr_stop` included, and an Error if that differs.
fn checked(input: &[Token], native: &[Token]) -> Result<Vec<Token>> {
  let mut toks = vec![
    T_CS!("\\exp_after:wN"),
    T_CS!("\\exp_not:n"),
    T_CS!("\\exp_after:wN"),
    T_BEGIN!(),
    T_CS!("\\exp:w"),
    T_CS!("\\__fp_parse_operand:Nw"),
  ];
  toks.extend(input.iter().rev().copied());
  toks.push(T_END!());
  CHECKING.with(|c| c.set(true));
  let raw = do_expand_partially(Tokens::new(toks));
  CHECKING.with(|c| c.set(false));
  let raw = raw?.unlist();
  let stop = T_CS!("\\s__fp_expr_stop");
  if raw.split_last() != Some((&stop, native)) {
    let message = s!(
      "native and raw l3fp differ on {{{}}}: native {} raw {}",
      Tokens::new(input.iter().rev().copied().collect()),
      Tokens::new(native.to_vec()),
      Tokens::new(raw.clone())
    );
    Error!("l3fp", "native-mismatch", message);
  }
  Ok(raw)
}

/// `\__fp_parse_operand:Nw`'s native call (`SpecialCall::Native`). Every other caller passes another precedence:
/// one comparison, and it runs as defined.
fn operand_call() -> Result<Option<Tokens>> {
  let at_door = with_pushback(|input| {
    matches!(input, [.., expand, prec]
      if *prec == T_CS!("\\c__fp_prec_end_int") && *expand == T_CS!("\\__fp_parse_expand:w"))
  });
  if at_door != Some(true) || CHECKING.with(Cell::get) {
    return Ok(None);
  }
  // With the macro's own run near the recursion limit, what it reports is the answer.
  let changed = watched_meaning_changed();
  if changed.is_some() || expand_depth_left() < RAW_DEPTH_RESERVE {
    left_to_the_macro();
    if *TRACE {
      let at = changed.map(|name| with(name, |cs| T_CS!(cs)));
      note_fallback(
        Decline("a name of l3fp changed, or near the recursion limit"),
        at,
        None,
        &[],
      );
    }
    return Ok(None);
  }
  let length = pushback_len();
  let read = with_pushback(|input| {
    let mark = door_mark(input)?;
    Some((
      mark,
      parse::Parser::over_input(&input[mark + 1..input.len() - 2]),
    ))
  })
  .flatten();
  let Some((mark, mut parser)) = read else {
    left_to_the_macro();
    if *TRACE {
      note_fallback(Decline("no door's marks on the input"), None, None, &[]);
    }
    return Ok(None);
  };
  let value = parser.parse();
  let answer = match value {
    Ok(x) if pushback_len() == length => x,
    Ok(_) => {
      left_to_the_macro();
      return Ok(None);
    },
    Err(decline) => {
      left_to_the_macro();
      if *TRACE {
        with_pushback(|input| {
          note_fallback(
            decline,
            parser.at,
            parser.via,
            &input[mark + 1..input.len() - 2],
          );
        });
      }
      return Ok(None);
    },
  };
  COUNTS.with(|c| c.set((c.get().0 + 1, c.get().1)));
  if *TRACE {
    note_native();
  }
  let mut out = vec![T_CS!("\\exp_end_continue_f:w")];
  fp_tokens(answer, &mut out);
  out.push(T_LETTER!("@"));
  out.push(T_CS!("\\__fp_parse_infix_end:N"));
  if check_enabled() {
    let input = with_pushback(|input| input[mark - 2..].to_vec()).unwrap_or_default();
    let raw = checked(&input, &out[1..])?;
    out.truncate(1);
    out.extend(raw);
    consume_pushback_to(mark - 2);
  } else {
    consume_pushback_to(mark - 1);
  }
  Ok(Some(Tokens::new(out)))
}

fn left_to_the_macro() { COUNTS.with(|c| c.set((c.get().0, c.get().1 + 1))); }

/// Give `\__fp_parse_operand:Nw` its native call, its definition as it is, and watch l3fp's meanings from now on.
pub(crate) fn install() -> Result<()> {
  COUNTS.with(|c| c.set((0, 0)));
  INSTALLED.with(|c| c.set(false));
  if !native_enabled() || !native_applies()? {
    return Ok(());
  }
  let operand = T_CS!("\\__fp_parse_operand:Nw");
  if let Some(Stored::Expandable(defn)) = lookup_meaning(&operand) {
    let mut hooked = (*defn).clone();
    hooked.special_call = SpecialCall::Native;
    set_native_call(operand_call);
    assign_meaning(
      &operand,
      Stored::Expandable(Rc::new(hooked)),
      Some(Scope::Global),
    );
    watch_meanings(watched_name);
    INSTALLED.with(|c| c.set(true));
  }
  Ok(())
}
