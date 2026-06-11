//! siunitx.sty — SI units and number formatting
//! Perl: siunitx.sty.ltxml (1817 lines)
//!
//! Full semantic port: number parsing, formatting with XMDual semantics,
//! unit parsing/formatting, options system, table columns.
use crate::{prelude::*, xmath_helpers::*};

/// Read the control-sequence argument of a siunitx `\Declare…` primitive,
/// handling BOTH the bare form `\DeclareSIPrefix \yocto {…}{…}` AND the
/// braced `m`-arg form `\DeclareSIPrefix{\million}{…}{…}` (Perl uses the
/// `DefToken` parameter type, which accepts either). A plain
/// `gullet::read_token()` reads `{` (catcode BEGIN) for the braced form, so
/// the real cs (`\million`) was never registered and stayed undefined —
/// witness 1811.03510 (`\DeclareSIPrefix{\million}{\text{M}}{2}` then
/// `\SI{185}{\million rays/s}`). `read_arg` strips the optional braces and
/// yields the single cs token in both cases.
fn read_si_declare_cs() -> Result<Token> {
  let toks = read_arg(ExpansionLevel::Off)?;
  Ok(
    toks
      .unlist_ref()
      .first()
      .copied()
      .unwrap_or_else(|| T_CS!("\\relax")),
  )
}

/// Structured error emission for siunitx (parallels
/// `latexml_post::diag::log_post_error!` and the engine `Error!` macro).
///
/// Why not `latexml_core::Error!`: the full macro early-returns
/// `Err(LatexmlError)` on max-errors / runaway-loop, but the siunitx
/// parsing helpers below return `Option<…>` / `Tokens` / `Vec<…>`,
/// not `Result<_, LatexmlError>`, so the early-return would type-mismatch.
/// The thin emit-only macro keeps the harness `Error:<class>:<object>`
/// format contract while staying type-compatible.
macro_rules! six_log_error {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    log::error!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {
    log::error!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

//======================================================================
// Perl siunitx.sty.ltxml L1747-1817: six_load_compat1 — RawTeX block of
// DeclareSIPrePower / DeclareSIUnit calls fired when the user opts in
// via `\usepackage[version-1-compatibility]{siunitx}` or any
// `\usepackage[alsoload=<x>]{siunitx}` (e.g. `alsoload=synchem` for
// `\Molar`). Copied verbatim from the Perl source so the v2-by-default
// engine still recognises legacy aliases used by older arXiv papers.
const SIX_LOAD_COMPAT1: &str = r"
\DeclareSIPrePower \Square  { 2 }
\DeclareSIPrePower \ssquare { 2 }
\DeclareSIUnit \BAR   { \bar }
\DeclareSIUnit \bbar  { \bar }
\DeclareSIUnit \Day   { \day }
\DeclareSIUnit \dday  { \day }
\DeclareSIUnit \Gray  { \gray }
\DeclareSIUnit \ggray { \gray }
\DeclareSIUnit \atomicmass { \atomicmassunit }
\DeclareSIUnit \arcmin     { \arcminute }
\DeclareSIUnit \arcsec     { \arcsecond }
\DeclareSIUnit \are      { a }
\DeclareSIUnit \curie    { Ci }
\DeclareSIUnit \gal      { Gal }
\DeclareSIUnit \millibar { \milli \bar }
\DeclareSIUnit \rad      { rad }
\DeclareSIUnit \rem      { rem }
\DeclareSIUnit \roentgen { R }
\DeclareSIUnit \micA   { \micro \ampere }
\DeclareSIUnit \micmol { \micro \mole   }
\DeclareSIUnit \micl   { \micro \litre  }
\DeclareSIUnit \micL   { \micro \liter  }
\DeclareSIUnit \nanog  { \nano  \gram   }
\DeclareSIUnit \micg   { \micro \gram   }
\DeclareSIUnit \picm   { \pico  \metre  }
\DeclareSIUnit \micm   { \micro \metre  }
\DeclareSIUnit \Sec    { \second }
\DeclareSIUnit \mics   { \micro \second }
\DeclareSIUnit \cmc    { \centi \metre \cubed }
\DeclareSIUnit \dmc    { \deci  \metre \cubed }
\DeclareSIUnit \cms    { \centi \metre \squared }
\DeclareSIUnit \centimetrecubed   { \centi \metre \cubed }
\DeclareSIUnit \centimetresquared { \centi \metre \squared }
\DeclareSIUnit \cubiccentimetre   { \centi \metre \cubed }
\DeclareSIUnit \cubicdecimetre    { \deci \metre \cubed }
\DeclareSIUnit \squarecentimetre  { \centi \metre \squared }
\DeclareSIUnit \squaremetre       { \metre \squared }
\DeclareSIUnit \squarekilometre   { \kilo \metre \squared }
\DeclareSIUnit \parsec    { pc }
\DeclareSIUnit \lightyear { ly }
\DeclareSIUnit \gmol  { g  \text { - } mol }
\DeclareSIUnit \kgmol { kg \text { - } mol }
\DeclareSIUnit \lbmol { lb \text { - } mol }
\DeclareSIUnit \molar { \mole \per \cubic \deci \metre }
\DeclareSIUnit \Molar { \textsc { m } }
\DeclareSIUnit \torr  { Torr }
\DeclareSIUnit \gon    { gon }
\DeclareSIUnit \clight { \text { \ensuremath { c } } }
\DeclareSIUnit \micron    { \micro \metre }
\DeclareSIUnit \mrad      { \milli \rad }
\DeclareSIUnit \gauss     { G }
\DeclareSIUnit \eVperc    { \eV \per \clight }
\DeclareSIUnit \nanobarn  { \nano \barn }
\DeclareSIUnit \picobarn  { \pico \barn }
\DeclareSIUnit \femtobarn { \femto \barn }
\DeclareSIUnit \attobarn  { \atto \barn }
\DeclareSIUnit \zeptobarn { \zepto \barn }
\DeclareSIUnit \yoctobarn { \yocto \barn }
\DeclareSIUnit \nb        { \nano \barn }
\DeclareSIUnit \pb        { \pico \barn }
\DeclareSIUnit \fb        { \femto \barn }
\DeclareSIUnit \ab        { \atto \barn }
\DeclareSIUnit \zb        { \zepto \barn }
\DeclareSIUnit \yb        { \yocto \barn }
";

//======================================================================
// SIX keyvals helpers
//======================================================================

/// `six_pin!("key")` — concatenates `"SIX_"` + `key` at compile time,
/// then caches the interned `SymStr` per call site. Every subsequent
/// lookup on the same thread is a `OnceCell` load + u32 compare — no
/// `format!()` allocation, no hash probe into the arena.
///
/// siunitx reads its options on every `\num` / `\SI` / `\ang` expansion
/// and `format!("SIX_{key}")` was one of the measurable chunks of the
/// siunitx critical path. This macro replaces the `&str`-based `six_get`
/// family for the (41 of 52) call sites that pass a string literal.
macro_rules! six_pin {
  ($key:literal) => {{
    std::thread_local! {
      static CACHED: std::cell::OnceCell<::latexml_core::common::arena::SymStr>
        = const { std::cell::OnceCell::new() };
    }
    CACHED
      .with(|c| *c.get_or_init(|| ::latexml_core::common::arena::pin_static(concat!("SIX_", $key))))
  }};
}

/// SymStr-keyed six_get: skip the per-call `format!` + `arena::pin`
/// that the `&str` variant pays.
fn six_get_sym(key: SymStr) -> Option<Stored> { with_value_sym(key, |v| v.cloned()) }

fn six_get_tokens_sym(key: SymStr) -> Tokens {
  match six_get_sym(key) {
    Some(Stored::Tokens(t)) => t,
    Some(Stored::String(s)) => {
      let txt = with(s, |t| t.to_string());
      Tokenize!(&txt)
    },
    _ => Tokens::default(),
  }
}

fn six_get_bool_sym(key: SymStr) -> bool {
  match six_get_sym(key) {
    Some(Stored::String(s)) => with(s, |txt| txt.trim() == "true"),
    Some(Stored::Tokens(t)) => t.to_string().trim() == "true",
    Some(Stored::Bool(b)) => b,
    _ => false,
  }
}

fn six_get_choice_sym(key: SymStr) -> String {
  match six_get_sym(key) {
    Some(Stored::String(s)) => with(s, |txt| txt.trim().to_string()),
    Some(Stored::Tokens(t)) => t.to_string().trim().to_string(),
    Some(v) => v.to_string().trim().to_string(),
    None => String::new(),
  }
}

// six_get(&str) removed — all callers now use six_get_sym + six_pin!
// which skips the per-call format!(\"SIX_{key}\") and arena::pin.

/// Build raw keyvals content (without brackets) for passing to i_wrap
fn make_kv_content(kv: &[(&str, Tokens)]) -> Option<Tokens> {
  if kv.is_empty() {
    return None;
  }
  let mut tks = Vec::new();
  for (i, (key, value)) in kv.iter().enumerate() {
    if i > 0 {
      tks.push(T_OTHER!(","));
    }
    tks.extend(ExplodeText!(key));
    tks.push(T_OTHER!("="));
    tks.push(T_BEGIN!());
    tks.extend_from_slice(value.unlist_ref());
    tks.push(T_END!());
  }
  Some(Tokens::new(tks))
}

/// Perl: six_get_op — look up an option, wrap in \text{} and I_wrap
fn six_get_op_sym(kv: &[(&str, Tokens)], key: SymStr) -> Tokens {
  let text = six_get_tokens_sym(key);
  if text.is_empty() {
    i_wrap(make_kv_content(kv), Tokens::default())
  } else {
    let mut tks = vec![T_CS!("\\text"), T_BEGIN!()];
    tks.extend(text.unlist());
    tks.push(T_END!());
    i_wrap(make_kv_content(kv), Tokens::new(tks))
  }
}

/// Perl: six_setup — assign all keyvals to SIX_key state values
fn six_setup(kv: &KeyVals) {
  for (key, value) in kv.get_pairs() {
    let key_str = key.clone();
    match value {
      ArgWrap::Tokens(t) => {
        if t.is_empty() {
          // Bare flag (e.g., "parse-numbers" without "= true") → set "true"
          assign_value(&format!("SIX_{key_str}"), Stored::from("true"), None);
        } else {
          assign_value(&format!("SIX_{key_str}"), Stored::Tokens(t.clone()), None);
        }
      },
      ArgWrap::KV(_) => {
        assign_value(
          &format!("SIX_{key_str}"),
          Stored::from(value.to_string()),
          None,
        );
      },
      _ => {
        // Empty value (boolean flag) or other
        let s = value.to_string();
        if s.is_empty() {
          assign_value(&format!("SIX_{key_str}"), Stored::from("true"), None);
        } else {
          assign_value(&format!("SIX_{key_str}"), Stored::from(s), None);
        }
      },
    }
  }
}

/// Perl: six_begin_processing — bgroup + apply keyvals + redefine input-protect-tokens
fn six_begin_processing(kv: Option<&KeyVals>) {
  bgroup();
  if let Some(kv) = kv {
    six_setup(kv);
  }
  // Perl L98-100: `Let($token, T_OTHER($name))` — make each input-protect
  // CS *let-equal to a non-expandable character token* of its own name, so a
  // later `Expand($expr)` in `\num` leaves the CS in place (instead of
  // expanding it to its `\def` body) and it then matches the `input-symbols`
  // list, which holds the same CS. This MUST be a let-to-char (`Stored::Token`),
  // NOT an expandable macro: an expandable redefinition still expands (e.g.
  // `\def\odd{\xi}` → `\odd` → `\xi`) and the now-bare `\xi` fails to match
  // `input-symbols={\odd}`, yielding `Not matched in \num: \xi`. Witness:
  // si.tex L98-100 `\num[input-symbols=\odd, input-protect-tokens=\odd]{3\odd}`.
  if let Some(Stored::Tokens(protect)) = six_get_sym(six_pin!("input-protect-tokens")) {
    // A control sequence token has catcode CS (or ACTIVE) AFTER tokenization —
    // NOT ESCAPE (catcode 0, the pre-tokenization backslash *character*). The
    // prior `== Catcode::ESCAPE` guard was therefore always false, so the
    // protect-token redefinition never fired and `input-protect-tokens` was a
    // silent no-op (the root cause of the `\xi` long tail). Perl
    // (`six_begin_processing` L98-100) applies `Let` to every token in the
    // list unconditionally; restrict to CS/active so `getCSName`/name-trim is
    // well-defined.
    for token in protect.unlist() {
      if token.get_catcode().is_active_or_cs() {
        let name = token.to_string();
        let other_name = name.trim_start_matches('\\');
        assign_meaning(&token, Stored::Token(T_OTHER!(other_name)), None);
      }
    }
  }
}

/// Perl: six_end_processing — egroup
fn six_end_processing() { let _ = egroup(); }

//======================================================================
// Number parsing — six_match_* functions
//======================================================================

/// Parsed number structure (mirrors Perl hash)
#[derive(Clone, Debug)]
enum SixNumber {
  Simple {
    sign:     Option<Tokens>,
    integer:  Option<Tokens>,
    decimal:  Option<Tokens>,
    fraction: Option<Tokens>,
  },
  Operator {
    operator:   String,
    arg1:       Option<Box<SixNumber>>,
    arg2:       Option<Box<SixNumber>>,
    sign:       Option<Tokens>,
    symbol:     Option<Tokens>,
    comparator: Option<Tokens>,
  },
}

impl SixNumber {
  fn simple(
    sign: Option<Tokens>,
    integer: Option<Tokens>,
    decimal: Option<Tokens>,
    fraction: Option<Tokens>,
  ) -> Self {
    SixNumber::Simple {
      sign,
      integer,
      decimal,
      fraction,
    }
  }

  fn get_sign(&self) -> Option<&Tokens> {
    match self {
      SixNumber::Simple { sign, .. } | SixNumber::Operator { sign, .. } => sign.as_ref(),
    }
  }

  fn set_sign(&mut self, new_sign: Option<Tokens>) {
    match self {
      SixNumber::Simple { sign, .. } | SixNumber::Operator { sign, .. } => *sign = new_sign,
    }
  }

  fn get_integer(&self) -> Option<&Tokens> {
    match self {
      SixNumber::Simple { integer, .. } => integer.as_ref(),
      _ => None,
    }
  }

  fn get_fraction(&self) -> Option<&Tokens> {
    match self {
      SixNumber::Simple { fraction, .. } => fraction.as_ref(),
      _ => None,
    }
  }

  fn is_operator(&self) -> bool { matches!(self, SixNumber::Operator { .. }) }
}

/// Perl: six_match_keys — match and remove leading tokens matching sixkeys
fn six_match_keys(tokens: &mut Vec<Token>, keys: &[SymStr]) -> Option<Tokens> {
  let mut tomatch: Vec<Token> = vec![T_SPACE!()];
  for key in keys {
    if let Some(Stored::Tokens(toks)) = six_get_sym(*key) {
      tomatch.extend(toks.unlist());
    }
  }

  // Match by Token-text equality against `tomatch`. For a typical
  // `tomatch` of a few dozen tokens and input of a few dozen tokens
  // this is O(n*m) — still fast because `Token` eq is a `SymStr` u32
  // compare, not a string compare.
  let mut matched = Vec::new();
  let mut consumed = 0;
  for t in tokens.iter() {
    if !tomatch.iter().any(|m| t == m) {
      break;
    }
    consumed += 1;
    if t.get_catcode() != Catcode::SPACE {
      if t.get_catcode() == Catcode::ESCAPE {
        matched.push(*t);
        matched.push(T_SPACE!());
      } else {
        matched.push(*t);
      }
    }
  }
  // Single drain at the end — the prior `tokens.remove(0)` per match
  // was O(n) each call (memmove of the tail on every consumed token).
  if consumed > 0 {
    tokens.drain(..consumed);
  }
  if matched.is_empty() {
    None
  } else {
    Some(Tokens::new(matched))
  }
}

/// Perl `six_match1($token, @keys)`: non-consuming test of whether a *single*
/// leading token belongs to any of the named option lists (used by the column
/// pre-peel loop to decide whether a leading control sequence is an
/// input-symbol/comparator/protect-token — which stays for number matching —
/// or surrounding formatting material — which is peeled into `pre`).
fn six_token_matches_keys(tok: &Token, keys: &[SymStr]) -> bool {
  for key in keys {
    if let Some(Stored::Tokens(toks)) = six_get_sym(*key)
      && toks.unlist().iter().any(|m| tok == m)
    {
      return true;
    }
  }
  false
}

fn six_match_sign(tokens: &mut Vec<Token>) -> Option<Tokens> {
  six_match_keys(tokens, &[six_pin!("input-signs")])
}

fn six_match_simplenumber(tokens: &mut Vec<Token>) -> Option<SixNumber> {
  let sign = six_match_sign(tokens);
  let integer = six_match_keys(tokens, &[
    six_pin!("input-digits"),
    six_pin!("input-symbols"),
  ]);
  let (decimal, fraction) =
    if six_match_keys(tokens, &[six_pin!("input-decimal-markers")]).is_some() {
      (
        Some(Tokens::default()),
        six_match_keys(tokens, &[
          six_pin!("input-digits"),
          six_pin!("input-symbols"),
        ]),
      )
    } else {
      (None, None)
    };

  if sign.is_some() || integer.is_some() || decimal.is_some() || fraction.is_some() {
    Some(SixNumber::simple(sign, integer, decimal, fraction))
  } else {
    None
  }
}

fn six_match_uncertainnumber(tokens: &mut Vec<Token>) -> Option<SixNumber> {
  let number = six_match_simplenumber(tokens)?;

  if let Some(sign) = six_match_keys(tokens, &[six_pin!("input-uncertainty-signs")]) {
    let int = six_match_keys(tokens, &[
      six_pin!("input-digits"),
      six_pin!("input-symbols"),
    ]);
    let (dec, frac) = if six_match_keys(tokens, &[six_pin!("input-decimal-markers")]).is_some() {
      (
        Some(Tokens::default()),
        six_match_keys(tokens, &[
          six_pin!("input-digits"),
          six_pin!("input-symbols"),
        ]),
      )
    } else {
      (None, None)
    };

    if six_match_keys(tokens, &[
      six_pin!("input-decimal-markers"),
      six_pin!("input-complex-roots"),
    ])
    .is_some()
    {
      return Some(number);
    }

    let uncertainty = SixNumber::simple(Some(sign), int, dec, frac);
    return Some(SixNumber::Operator {
      operator:   "uncertain".to_string(),
      arg1:       Some(Box::new(number)),
      arg2:       Some(Box::new(uncertainty)),
      sign:       None,
      symbol:     None,
      comparator: None,
    });
  }

  if six_match_keys(tokens, &[six_pin!("input-open-uncertainty")]).is_some() {
    let int = six_match_keys(tokens, &[
      six_pin!("input-digits"),
      six_pin!("input-symbols"),
    ]);
    six_match_keys(tokens, &[six_pin!("input-close-uncertainty")]);
    let uncertainty = SixNumber::simple(None, int, None, None);
    return Some(SixNumber::Operator {
      operator:   "uncertain".to_string(),
      arg1:       Some(Box::new(number)),
      arg2:       Some(Box::new(uncertainty)),
      sign:       None,
      symbol:     None,
      comparator: None,
    });
  }

  Some(number)
}

fn six_match_complexnumber(tokens: &mut Vec<Token>) -> Option<SixNumber> {
  // Perl siunitx.sty.ltxml:253 `my $number = six_match_uncertainnumber($tokens)`
  // — `$number` MAY be undef. Do NOT early-return here: a pure-imaginary input
  // like `\num{i}` (the imaginary unit alone, no preceding number) has no
  // uncertain-number but must still match the `input-complex-roots` key below.
  let number_opt = six_match_uncertainnumber(tokens);

  if let Some(i) = six_match_keys(tokens, &[six_pin!("input-complex-roots")]) {
    // pure imaginary! Perl L255-256: $sign = $$number{sign}; $$number{sign}=undef
    // (make the sign "infix"); $number = {complex, symbol=>i, sign, arg2=>$number}.
    // $number may be undef (`\num{i}`) → no sign, no arg2.
    let (sign, arg2) = match number_opt {
      Some(mut n) => {
        let s = n.get_sign().cloned();
        n.set_sign(None);
        (s, Some(Box::new(n)))
      },
      None => (None, None),
    };
    return Some(SixNumber::Operator {
      operator: "complex".to_string(),
      arg1: None,
      arg2,
      sign,
      symbol: Some(i),
      comparator: None,
    });
  }

  // Past the pure-imaginary case the remaining forms (`a ± b i`) all operate on
  // a real preceding number; if there wasn't one, Perl returns the undef
  // `$number` and the caller reports the "Not matched" error.
  let number = number_opt?;

  if let Some(sign) = six_match_sign(tokens) {
    if let Some(i) = six_match_keys(tokens, &[six_pin!("input-complex-roots")]) {
      if let Some(imag) = six_match_uncertainnumber(tokens) {
        return Some(SixNumber::Operator {
          operator:   "complex".to_string(),
          arg1:       Some(Box::new(number)),
          arg2:       Some(Box::new(imag)),
          sign:       Some(sign),
          symbol:     Some(i),
          comparator: None,
        });
      }
    } else if let Some(imag) = six_match_uncertainnumber(tokens) {
      if let Some(i) = six_match_keys(tokens, &[six_pin!("input-complex-roots")]) {
        return Some(SixNumber::Operator {
          operator:   "complex".to_string(),
          arg1:       Some(Box::new(number)),
          arg2:       Some(Box::new(imag)),
          sign:       Some(sign),
          symbol:     Some(i),
          comparator: None,
        });
      }
      // Imaginary part matched but no `input-complex-roots` key — incomplete
      // complex form. Perl siunitx.sty.ltxml:266 — Error('unexpected',
      // 'sign', undef, "expected to find complex number")
      six_log_error!("unexpected", "sign", "expected to find complex number");
    } else {
      // Perl siunitx.sty.ltxml:266 — same Error site (no imaginary part
      // followed the matched sign).
      six_log_error!("unexpected", "sign", "expected to find complex number");
    }
  }

  Some(number)
}

fn six_match_scinumber(tokens: &mut Vec<Token>) -> Option<SixNumber> {
  // Perl siunitx.sty.ltxml:270-279 — `$number` may be undef and the
  // exponent-marker still matches. `\SI{e12}{G}` is a valid no-mantissa
  // form that defaults to `10^12`. Without this branch our `?` short-
  // circuited on a missing mantissa and the exponent-marker leaked into
  // the leftover-tokens "Not matched in \num" error path. Witnesses:
  // arXiv:2509.14043 / 2509.16675 — `\SIrange{e12}{e15}{G}` and
  // `\SI{4e13}{G}` astro-physics magnetic-field strings.
  let number = six_match_complexnumber(tokens);

  if six_match_keys(tokens, &[six_pin!("input-exponent-markers")]).is_some() {
    let sign = six_match_sign(tokens);
    let exp = six_match_keys(tokens, &[
      six_pin!("input-digits"),
      six_pin!("input-symbols"),
    ]);
    let exp_number = SixNumber::simple(sign, exp, None, None);
    return Some(SixNumber::Operator {
      operator:   "exponent".to_string(),
      arg1:       number.map(Box::new),
      arg2:       Some(Box::new(exp_number)),
      sign:       None,
      symbol:     None,
      comparator: None,
    });
  }

  number
}

fn six_match_compoundnumber(tokens: &mut Vec<Token>) -> Option<SixNumber> {
  if let Some(comp) = six_match_keys(tokens, &[six_pin!("input-comparators")]) {
    return Some(SixNumber::Operator {
      operator:   "comparator".to_string(),
      arg1:       six_match_number(tokens).map(Box::new),
      arg2:       None,
      sign:       None,
      symbol:     None,
      comparator: Some(comp),
    });
  }

  let mut number = six_match_scinumber(tokens)?;
  loop {
    if six_match_keys(tokens, &[six_pin!("input-product")]).is_some() {
      let rhs = six_match_scinumber(tokens);
      number = SixNumber::Operator {
        operator:   "product".to_string(),
        arg1:       Some(Box::new(number)),
        arg2:       rhs.map(Box::new),
        sign:       None,
        symbol:     None,
        comparator: None,
      };
    } else if six_match_keys(tokens, &[six_pin!("input-quotient")]).is_some() {
      let rhs = six_match_scinumber(tokens);
      number = SixNumber::Operator {
        operator:   "quotient".to_string(),
        arg1:       Some(Box::new(number)),
        arg2:       rhs.map(Box::new),
        sign:       None,
        symbol:     None,
        comparator: None,
      };
    } else {
      return Some(number);
    }
  }
}

fn six_match_number(tokens: &mut Vec<Token>) -> Option<SixNumber> {
  six_match_compoundnumber(tokens)
}

//======================================================================
// Math ligatures
//======================================================================

fn six_apply_mathligatures(tokens: Vec<Token>) -> Vec<Token> {
  let mut result = Vec::with_capacity(tokens.len());
  let mut iter = tokens.into_iter().peekable();
  // Pre-intern the ligature trigger chars so the per-token dispatch
  // below compares SymStr (u32 equality) instead of allocating
  // `t.to_string()` to compare with "+" / ">" / "<" / etc.
  let plus = pin!("+");
  let minus = pin!("-");
  let gt = pin!(">");
  let lt = pin!("<");
  let eq = pin!("=");
  while let Some(t) = iter.next() {
    if t.get_catcode() == Catcode::COMMENT {
      continue;
    }
    if t.text == plus {
      if iter.peek().is_some_and(|n| n.text == minus) {
        iter.next();
        result.push(T_CS!("\\pm"));
      } else {
        result.push(t);
      }
    } else if t.text == gt {
      if let Some(ns) = iter.peek().map(|n| n.text) {
        if ns == eq {
          iter.next();
          result.push(T_CS!("\\ge"));
        } else if ns == gt {
          iter.next();
          result.push(T_CS!("\\gg"));
        } else {
          result.push(t);
        }
      } else {
        result.push(t);
      }
    } else if t.text == lt {
      if let Some(ns) = iter.peek().map(|n| n.text) {
        if ns == eq {
          iter.next();
          result.push(T_CS!("\\le"));
        } else if ns == lt {
          iter.next();
          result.push(T_CS!("\\ll"));
        } else {
          result.push(t);
        }
      } else {
        result.push(t);
      }
    } else {
      result.push(t);
    }
  }
  result
}

//======================================================================
// Number post-processing
//======================================================================

fn six_postprocess(number: Option<SixNumber>) -> Option<SixNumber> {
  number.map(six_postprocess_aux)
}

fn six_postprocess_aux(mut number: SixNumber) -> SixNumber {
  match &mut number {
    SixNumber::Operator { arg1, arg2, .. } => {
      *arg1 = arg1.take().map(|n| Box::new(six_postprocess_aux(*n)));
      *arg2 = arg2.take().map(|n| Box::new(six_postprocess_aux(*n)));
    },
    SixNumber::Simple {
      decimal,
      fraction,
      integer,
      sign,
      ..
    } => {
      if six_get_bool_sym(six_pin!("add-decimal-zero")) && decimal.is_some() && fraction.is_none() {
        *fraction = Some(Tokens::new(vec![T_OTHER!("0")]));
      }
      if six_get_bool_sym(six_pin!("add-integer-zero")) && decimal.is_some() && integer.is_none() {
        *integer = Some(Tokens::new(vec![T_OTHER!("0")]));
      }
      if sign.is_none()
        && let Some(Stored::Tokens(s)) = six_get_sym(six_pin!("explicit-sign"))
        && !s.is_empty()
      {
        *sign = Some(s);
      }
    },
  }
  number
}

//======================================================================
// Top-level number parsing
//======================================================================

enum SixParseResult {
  Parsed(SixNumber),
  Raw(Tokens),
  /// An EMPTY component between `;` separators in `\ang` / `\numlist` etc.
  /// Perl represents these as `undef` in `six_parse_numbers`'s result list —
  /// e.g. `\ang{;;1.0}` (empty degrees, empty minutes, 1.0 seconds) yields
  /// `(undef, undef, <1.0>)`. The component is skipped at format time.
  Empty,
}

fn six_parse_number(expr: &Tokens) -> SixParseResult {
  if six_get_bool_sym(six_pin!("parse-numbers")) {
    let expanded = do_expand_partially(expr.clone()).unwrap_or_else(|_| expr.clone());
    let mut tokens = six_apply_mathligatures(expanded.unlist());
    let result = six_postprocess(six_match_number(&mut tokens));
    if !tokens.is_empty() {
      // Perl siunitx.sty.ltxml:410 — Error('unexpected', $$tokens[0],
      //   $gullet, "Not matched in \\num: ...")
      let leftover = Tokens::new(tokens.clone()).to_string();
      let first_obj = tokens.first().map(|t| t.to_string()).unwrap_or_default();
      six_log_error!(
        "unexpected",
        first_obj,
        "Not matched in \\num: {}",
        leftover
      );
      return SixParseResult::Raw(expr.clone());
    }
    match result {
      Some(n) => SixParseResult::Parsed(n),
      None => SixParseResult::Raw(expr.clone()),
    }
  } else {
    SixParseResult::Raw(expr.clone())
  }
}

fn six_parse_numbers(expr: &Tokens) -> Vec<SixParseResult> {
  if six_get_bool_sym(six_pin!("parse-numbers")) {
    let expanded = do_expand_partially(expr.clone()).unwrap_or_else(|_| expr.clone());
    let mut tokens = six_apply_mathligatures(expanded.unlist());
    let mut results = Vec::new();
    loop {
      // Perl siunitx.sty.ltxml:six_parse_numbers ALWAYS pushes the result —
      // `undef` (our `Empty`) for an empty component between `;` separators —
      // then consumes the `;` and continues. Breaking on `None` (the previous
      // Rust behavior) made `\ang{;;1.0}` (empty;empty;1.0) leave `;;1.0`
      // unconsumed → spurious "Not matched in \num: ;;1.0". The trailing `;`
      // check still guarantees progress (each iteration either matches a
      // number or consumes one `;`). Witness: 2007.08215.
      let result = six_postprocess(six_match_number(&mut tokens));
      results.push(match result {
        Some(n) => SixParseResult::Parsed(n),
        None => SixParseResult::Empty,
      });
      if tokens.first().is_some_and(|t| t.text == pin!(";")) {
        tokens.remove(0);
      } else {
        break;
      }
    }
    if !tokens.is_empty() {
      // Perl siunitx.sty.ltxml:430 — Error('unexpected', $$tokens[0],
      //   $gullet, "Not matched in \\num: ...")
      let leftover = Tokens::new(tokens.clone()).to_string();
      let first_obj = tokens.first().map(|t| t.to_string()).unwrap_or_default();
      six_log_error!(
        "unexpected",
        first_obj,
        "Not matched in \\num: {}",
        leftover
      );
      return vec![SixParseResult::Raw(expr.clone())];
    }
    results
  } else {
    let mut results = Vec::new();
    let mut current = Vec::new();
    for t in expr.unlist_ref() {
      if t.text == pin!(";") {
        results.push(SixParseResult::Raw(Tokens::new(current)));
        current = Vec::new();
      } else {
        current.push(*t);
      }
    }
    if !current.is_empty() {
      results.push(SixParseResult::Raw(Tokens::new(current)));
    }
    results
  }
}

//======================================================================
// Number formatting
//======================================================================

fn six_number_string(number: &SixNumber) -> String {
  match number {
    SixNumber::Operator {
      operator,
      arg1,
      arg2,
      sign,
      comparator,
      ..
    } => match operator.as_str() {
      "uncertain" => {
        let a1 = arg1
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        if let Some(a2) = arg2 {
          format!("{}({})", a1, six_number_string(a2))
        } else {
          a1
        }
      },
      "comparator" => {
        let comp = comparator
          .as_ref()
          .map(|t| t.to_string())
          .unwrap_or_default();
        let a1 = arg1
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        format!("{}{}", comp, a1)
      },
      "product" => {
        let a1 = arg1
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        let a2 = arg2
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        format!("{}*{}", a1, a2)
      },
      "quotient" => {
        let a1 = arg1
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        let a2 = arg2
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        format!("{}/{}", a1, a2)
      },
      "complex" => {
        let a1 = arg1
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        let a2 = arg2
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        let s = sign
          .as_ref()
          .map(|t| t.to_string())
          .unwrap_or_else(|| "+".to_string());
        format!("{}{}{}\u{2148}", a1, s, a2)
      },
      "exponent" => {
        let a1 = arg1
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        let a2 = arg2
          .as_ref()
          .map(|n| six_number_string(n))
          .unwrap_or_default();
        format!("{}E{}", a1, a2)
      },
      _ => String::new(),
    },
    SixNumber::Simple { sign, integer, fraction, .. } => {
      let s = sign.as_ref().map(|t| t.to_string()).unwrap_or_default();
      let i = integer.as_ref().map(|t| t.to_string()).unwrap_or_default();
      let f = fraction
        .as_ref()
        .map(|t| format!(".{}", t))
        .unwrap_or_default();
      format!("{}{}{}", s, i, f)
    },
  }
}

fn six_groupdigits(digits: &Tokens, direction: i32) -> Tokens {
  let min: usize = six_get_choice_sym(six_pin!("group-minimum-digits"))
    .parse()
    .unwrap_or(5);
  if min > digits.len() {
    return digits.clone();
  }
  let digs: Vec<Token> = digits.unlist_ref().clone();
  let sep = six_get_tokens_sym(six_pin!("group-separator"));
  let g = 3usize;
  let mut result = Vec::new();

  if direction > 0 {
    let mut remaining = digs;
    while !remaining.is_empty() {
      let mut chunk = Vec::new();
      for _ in 0..g {
        if remaining.is_empty() {
          break;
        }
        chunk.insert(0, remaining.pop().unwrap());
      }
      if !result.is_empty() {
        result.splice(0..0, sep.unlist_ref().iter().copied());
      }
      result.splice(0..0, chunk);
    }
  } else {
    let mut remaining = digs;
    while !remaining.is_empty() {
      if !result.is_empty() {
        result.extend_from_slice(sep.unlist_ref());
      }
      for _ in 0..g {
        if remaining.is_empty() {
          break;
        }
        result.push(remaining.remove(0));
      }
    }
  }

  Tokens::new(result)
}

fn six_format_simplenumber(number: &SixNumber) -> Tokens {
  if let SixNumber::Simple {
    sign,
    integer,
    fraction,
    decimal,
  } = number
  {
    let grouping = six_get_choice_sym(six_pin!("group-digits"));
    let mut tokens = Vec::new();
    let mut trailer = Vec::new();

    if let Some(s) = sign {
      let s_str = s.to_string();
      if s_str.contains('-') {
        if let Some(Stored::Tokens(c)) = six_get_sym(six_pin!("negative-color")) {
          tokens.push(T_BEGIN!());
          tokens.push(T_CS!("\\color"));
          tokens.push(T_BEGIN!());
          tokens.extend(c.unlist());
          tokens.push(T_END!());
          trailer.insert(0, T_END!());
        }
        if six_get_bool_sym(six_pin!("bracket-negative-numbers")) {
          tokens.extend(six_get_tokens_sym(six_pin!("open-bracket")).unlist());
          let cb = six_get_tokens_sym(six_pin!("close-bracket"));
          trailer.splice(0..0, cb.unlist());
        } else {
          tokens.extend_from_slice(s.unlist_ref());
        }
      } else if s_str.contains('+') && !six_get_bool_sym(six_pin!("retain-explicit-plus")) {
        // drop +
      } else {
        tokens.extend_from_slice(s.unlist_ref());
      }
    }

    if let Some(int) = integer {
      let i = if grouping == "true" || grouping == "integer" {
        six_groupdigits(int, 1)
      } else {
        int.clone()
      };
      tokens.extend(i.unlist());
    }

    let has_decimal = decimal.is_some();
    let has_fraction = fraction.is_some();
    if six_get_bool_sym(six_pin!("copy-decimal-marker")) {
      if has_decimal && let Some(d) = decimal {
        if !d.is_empty() {
          tokens.extend_from_slice(d.unlist_ref());
        } else {
          tokens.extend(six_get_tokens_sym(six_pin!("output-decimal-marker")).unlist());
        }
      }
    } else if has_fraction || has_decimal {
      tokens.extend(six_get_tokens_sym(six_pin!("output-decimal-marker")).unlist());
    }

    if let Some(frac) = fraction {
      let f = if grouping == "true" || grouping == "decimal" {
        six_groupdigits(frac, -1)
      } else {
        frac.clone()
      };
      tokens.extend(f.unlist());
    }

    tokens.extend(trailer);

    let meaning = six_number_string(number);
    let meaning_toks = Tokenize!(&meaning);
    let symbol = i_symbol(
      &[("role", Tokenize!("NUMBER")), ("meaning", meaning_toks)],
      None,
    );
    let wrapped = i_wrap(None, Tokens::new(tokens));
    i_dual(
      &[("revert_as", Tokenize!("presentation"))],
      symbol,
      wrapped,
      vec![],
    )
    .unwrap_or_default()
  } else {
    Tokens::default()
  }
}

fn six_format_infix(
  op: Tokens,
  left: Option<Tokens>,
  right: Option<Tokens>,
  args: Vec<Tokens>,
) -> Tokens {
  let n = args.len();
  if n < 1 {
    return Tokens::default();
  }
  if n == 1 {
    return args.into_iter().next().unwrap();
  }

  // Perl: I_apply({}, map { I_arg($_) } 1..n+1)
  // First arg is the operator, rest are operands
  let content = i_apply(
    &[],
    Tokens::new(vec![i_arg("1")]),
    (2..=n + 1)
      .map(|i| Tokens::new(vec![i_arg(&i.to_string())]))
      .collect(),
  );

  let mut pres = Vec::new();
  if let Some(l) = &left {
    pres.extend_from_slice(l.unlist_ref());
  }
  pres.push(i_arg("2"));
  for i in 3..=n + 1 {
    pres.push(i_arg("1"));
    pres.push(i_arg(&i.to_string()));
  }
  if let Some(r) = &right {
    pres.extend_from_slice(r.unlist_ref());
  }

  let mut all_args = vec![op];
  all_args.extend(args);

  i_dual(
    &[("revert_as", Tokenize!("presentation"))],
    content,
    Tokens::new(pres),
    all_args,
  )
  .unwrap_or_default()
}

fn six_format_number(number: &SixParseResult, bracket: i32) -> Tokens {
  match number {
    SixParseResult::Raw(toks) => i_wrap(None, toks.clone()),
    SixParseResult::Parsed(num) => six_format_number_inner(num, bracket),
    SixParseResult::Empty => Tokens::default(),
  }
}

fn six_format_number_inner(number: &SixNumber, bracket: i32) -> Tokens {
  let bracket = if bracket > 0 && six_get_bool_sym(six_pin!("bracket-numbers")) {
    bracket
  } else {
    0
  };

  match number {
    SixNumber::Simple { .. } => six_format_simplenumber(number),
    SixNumber::Operator { operator, arg1, arg2, .. } => match operator.as_str() {
      "uncertain" => {
        let a1 = arg1.as_deref();
        let a2 = arg2.as_deref();
        if a2.is_none() || six_get_bool_sym(six_pin!("omit-uncertainty")) {
          return a1
            .map(|n| six_format_number_inner(n, 0))
            .unwrap_or_default();
        }
        let fa1 = a1
          .map(|n| six_format_number_inner(n, 0))
          .unwrap_or_default();
        let fa2 = a2
          .map(|n| six_format_number_inner(n, 0))
          .unwrap_or_default();
        if six_get_bool_sym(six_pin!("separate-uncertainty")) {
          six_format_infix(Tokens::new(vec![T_CS!("\\pm")]), None, None, vec![fa1, fa2])
        } else {
          let open = six_get_tokens_sym(six_pin!("output-open-uncertainty"));
          let close = six_get_tokens_sym(six_pin!("output-close-uncertainty"));
          let mut tks = fa1.unlist();
          tks.extend(open.unlist());
          tks.extend(fa2.unlist());
          tks.extend(close.unlist());
          Tokens::new(tks)
        }
      },
      "complex" => {
        if let SixNumber::Operator { arg1, arg2, sign, symbol, .. } = number {
          let real = arg1.as_deref().map(|n| six_format_number_inner(n, 0));
          let imag = arg2.as_deref().map(|n| six_format_number_inner(n, 0));
          let i_tok = if six_get_bool_sym(six_pin!("copy-complex-root")) {
            symbol
              .clone()
              .unwrap_or_else(|| six_get_tokens_sym(six_pin!("output-complex-root")))
          } else {
            six_get_tokens_sym(six_pin!("output-complex-root"))
          };

          let mut result = Vec::new();
          if let Some(r) = &real {
            result.extend_from_slice(r.unlist_ref());
          }
          if let Some(s) = sign {
            result.extend_from_slice(s.unlist_ref());
          }
          if let Some(im) = &imag {
            result.extend_from_slice(im.unlist_ref());
          }
          result.extend(i_tok.unlist());
          Tokens::new(result)
        } else {
          Tokens::default()
        }
      },
      "exponent" => {
        let a1 = arg1.as_deref();
        let a2 = arg2.as_deref();

        if let Some(a2) = a2 {
          let has_content = a2.get_integer().is_some_and(|i| !i.is_empty())
            || a2.get_fraction().is_some_and(|f| !f.is_empty());
          if !six_get_bool_sym(six_pin!("retain-zero-exponent")) && !has_content {
            return a1
              .map(|n| six_format_number_inner(n, 0))
              .unwrap_or_default();
          }
        }

        let fa1 = a1
          .map(|n| six_format_number_inner(n, 1))
          .unwrap_or_default();
        let base = six_get_tokens_sym(six_pin!("exponent-base"));
        let fa2 = a2
          .map(|n| six_format_number_inner(n, 0))
          .unwrap_or_default();
        let power = i_superscript(&[("operator_meaning", Tokenize!("power"))], base, fa2);

        let has_mantissa = a1.is_some_and(|n| {
          n.get_integer().is_some() || n.get_fraction().is_some() || n.is_operator()
        });
        let times = if has_mantissa {
          six_get_op_sym(
            &[
              ("role", Tokenize!("MULOP")),
              ("meaning", Tokenize!("times")),
            ],
            six_pin!("exponent-product"),
          )
        } else {
          Tokens::new(vec![T_CS!("\\lx@InvisibleTimes")])
        };

        six_format_infix(
          times,
          if bracket > 1 {
            Some(six_get_tokens_sym(six_pin!("open-bracket")))
          } else {
            None
          },
          if bracket > 1 {
            Some(six_get_tokens_sym(six_pin!("close-bracket")))
          } else {
            None
          },
          vec![fa1, power],
        )
      },
      "comparator" => {
        if let SixNumber::Operator { comparator, .. } = number {
          let comp = comparator.clone().unwrap_or_default();
          let a1 = arg1
            .as_deref()
            .map(|n| six_format_number_inner(n, 0))
            .unwrap_or_default();
          let mut tks = comp.unlist();
          tks.extend(a1.unlist());
          i_wrap(None, Tokens::new(tks))
        } else {
          Tokens::default()
        }
      },
      "product" => {
        let fa1 = arg1
          .as_deref()
          .map(|n| six_format_number_inner(n, 1))
          .unwrap_or_default();
        let fa2 = arg2
          .as_deref()
          .map(|n| six_format_number_inner(n, 1))
          .unwrap_or_default();
        let times = six_get_op_sym(
          &[
            ("role", Tokenize!("MULOP")),
            ("meaning", Tokenize!("times")),
          ],
          six_pin!("output-product"),
        );
        six_format_infix(times, None, None, vec![fa1, fa2])
      },
      "quotient" => {
        let fa1 = arg1
          .as_deref()
          .map(|n| six_format_number_inner(n, 1))
          .unwrap_or_default();
        let fa2 = arg2
          .as_deref()
          .map(|n| six_format_number_inner(n, 2))
          .unwrap_or_default();
        if six_get_choice_sym(six_pin!("quotient-mode")) == "fraction" {
          let frac = six_get_tokens_sym(six_pin!("fraction-function"));
          let mut tks = frac.unlist();
          tks.push(T_BEGIN!());
          tks.extend(fa1.unlist());
          tks.push(T_END!());
          tks.push(T_BEGIN!());
          tks.extend(fa2.unlist());
          tks.push(T_END!());
          Tokens::new(tks)
        } else {
          let div = six_get_op_sym(
            &[
              ("role", Tokenize!("MULOP")),
              ("meaning", Tokenize!("divide")),
            ],
            six_pin!("output-quotient"),
          );
          six_format_infix(div, None, None, vec![fa1, fa2])
        }
      },
      other => {
        // Perl siunitx.sty.ltxml:642 — Error('unexpected', $op, undef,
        //   "Unrecognized operator $op in siunitx number")
        six_log_error!(
          "unexpected",
          other,
          "Unrecognized operator {} in siunitx number",
          other
        );
        Tokens::default()
      },
    },
  }
}

fn six_format_range(bracketed: bool, first: Tokens, last: Tokens) -> Tokens {
  let mut range_pres = Vec::new();
  range_pres.push(i_arg("1"));
  range_pres
    .extend(six_get_op_sym(&[("role", Tokenize!("PUNCT"))], six_pin!("range-phrase")).unlist());
  range_pres.push(i_arg("2"));

  if bracketed {
    // Perl six_format_range: `unshift(@range, six_get_op({role=>'OPEN'},
    // 'open-bracket'))` prepends the ENTIRE open-bracket op. The earlier
    // Rust port did `range_pres.insert(0, open.unlist().remove(0))`, which
    // kept only the op's FIRST token and dropped the rest — corrupting the
    // bracketed presentation so the dual lost its first argument
    // (`range@([], 4)` instead of `range@(2, 4)`). Prepend the whole op,
    // matching Perl. Witness: si.tex `\SIrange[range-units=brackets]{2}{4}
    // {\degreeCelsius}`.
    let mut bracketed_pres =
      six_get_op_sym(&[("role", Tokenize!("OPEN"))], six_pin!("open-bracket")).unlist();
    bracketed_pres.append(&mut range_pres);
    range_pres = bracketed_pres;
    range_pres
      .extend(six_get_op_sym(&[("role", Tokenize!("CLOSE"))], six_pin!("close-bracket")).unlist());
  }

  let content = i_apply(
    &[],
    i_symbol(&[("meaning", Tokenize!("range"))], None),
    vec![Tokens::new(vec![i_arg("1")]), Tokens::new(vec![i_arg("2")])],
  );

  i_dual(&[], content, Tokens::new(range_pres), vec![first, last]).unwrap_or_default()
}

fn six_format_list(bracketed: bool, items: Vec<Tokens>) -> Tokens {
  let n = items.len();
  if n == 0 {
    return Tokens::default();
  }
  if n == 1 {
    return items.into_iter().next().unwrap();
  }

  let mut list_pres = Vec::new();
  if n == 2 {
    list_pres.push(i_arg("1"));
    list_pres.extend(
      six_get_op_sym(
        &[("role", Tokenize!("PUNCT"))],
        six_pin!("list-pair-separator"),
      )
      .unlist(),
    );
    list_pres.push(i_arg("2"));
  } else {
    list_pres.push(i_arg("1"));
    for i in 2..n {
      list_pres.extend(
        six_get_op_sym(&[("role", Tokenize!("PUNCT"))], six_pin!("list-separator")).unlist(),
      );
      list_pres.push(i_arg(&i.to_string()));
    }
    list_pres.extend(
      six_get_op_sym(
        &[("role", Tokenize!("PUNCT"))],
        six_pin!("list-final-separator"),
      )
      .unlist(),
    );
    list_pres.push(i_arg(&n.to_string()));
  }

  if n > 1 && bracketed {
    list_pres.splice(
      0..0,
      six_get_op_sym(&[("role", Tokenize!("OPEN"))], six_pin!("open-bracket")).unlist(),
    );
    list_pres
      .extend(six_get_op_sym(&[("role", Tokenize!("CLOSE"))], six_pin!("close-bracket")).unlist());
  }

  let content = i_apply(
    &[],
    i_symbol(&[("meaning", Tokenize!("list"))], None),
    (1..=n)
      .map(|i| Tokens::new(vec![i_arg(&i.to_string())]))
      .collect(),
  );

  i_dual(&[], content, Tokens::new(list_pres), items).unwrap_or_default()
}

/// Perl siunitx.sty.ltxml L1379-1399 `DefColumnType('S'|'s' Optional, …)`: add
/// an alignment column whose `before`/`after` wrap each cell as
/// `{ \lx@si@column@prep[kv] <parse> <cell> \lx@si@column@end }`, routing the
/// cell through the SI parser so table numbers/units render in MATH mode (like
/// `\num`/`\si`). `parse_cs` is `\lx@SI@column@parse` (number parse, the `S`
/// column) or `\lx@si@column@parse` (unit parse, the lowercase `s` column).
/// The Rust S/s columns used to be stubs (default Cell, no before/after) →
/// cells rendered as bare text, never `<ltx:Math>`. Witness 1909.01486
/// (siunitx `S[table-format=…]` tables: RUST 303 Math vs PERL 578; table@671
/// RUST 6 vs PERL 174).
fn add_si_column(kv: Option<Tokens>, parse_cs: &str) {
  let mut before: Vec<Token> = vec![T_BEGIN!(), T_CS!("\\lx@si@column@prep")];
  if let Some(kv) = kv
    && !kv.is_empty()
  {
    before.push(T_OTHER!("["));
    before.extend(kv.unlist());
    before.push(T_OTHER!("]"));
  }
  before.push(T_CS!(parse_cs));
  let after = Tokens!(T_CS!("\\lx@si@column@end"), T_END!());
  with_current_build_template(|template_opt| {
    if let Some(t) = template_opt {
      t.add_column(Cell {
        before: Some(Tokens::new(before.clone())),
        after: Some(after.clone()),
        ..Cell::default()
      });
    }
  });
}

/// Perl siunitx.sty.ltxml L751-759: `sub six_wrap`. Reads color ONCE
/// at entry and reuses the captured value for both the open `{ \color
/// {...}` and the matching close `}`.
fn six_wrap(content: Tokens) -> Tokens {
  let color = six_get_tokens_sym(six_pin!("color"));
  let has_color = !color.is_empty();
  let mut tks = Vec::new();
  if has_color {
    tks.push(T_BEGIN!());
    tks.push(T_CS!("\\color"));
    tks.push(T_BEGIN!());
    tks.extend(color.unlist());
    tks.push(T_END!());
  }
  tks.push(T_CS!("\\lx@begin@inline@math"));
  tks.extend(content.unlist());
  tks.push(T_CS!("\\lx@end@inline@math"));
  if has_color {
    tks.push(T_END!());
  }
  Tokens::new(tks)
}

//======================================================================
// Unit system
//======================================================================

/// Unit definition record
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SixUnitDefn {
  name:         String,
  unit_type:    String,
  presentation: Tokens,
  power:        Option<Tokens>,
  base:         Option<i32>,
  arg:          Option<String>,
  color:        Option<Tokens>,
}

#[derive(Clone, Debug, Default)]
struct SixUnit {
  per:             bool,
  prefix:          Option<SixUnitDefn>,
  unit:            Option<SixUnitDefn>,
  prepower:        Option<SixUnitDefn>,
  postpower:       Option<SixUnitDefn>,
  qualifier:       Option<SixUnitDefn>,
  cancel:          bool,
  highlight_color: Option<Tokens>,
}

/// Parse unit definitions into structured units
fn six_parse_units(defns: Vec<SixUnitDefn>) -> Vec<SixUnit> {
  let mut units = Vec::new();
  let sticky_per = six_get_bool_sym(six_pin!("sticky-per"));
  let mut saved_per = false;
  // Consume defns as an iterator of Options — each slot holds the
  // SixUnitDefn until we move it into a SixUnit role slot. This skips
  // the per-role `.clone()` pass the prior `&[SixUnitDefn]` signature
  // forced (6 × N clones per unit block, each cloning 3 Strings + 3
  // optional allocations). Cost drops to zero allocations for the
  // role-binding phase.
  let mut slots: Vec<Option<SixUnitDefn>> = defns.into_iter().map(Some).collect();
  let mut idx = 0;

  while idx < slots.len() {
    let mut unit = SixUnit::default();

    for role in &[
      "per",
      "prepower",
      "prefix",
      "unit",
      "qualifier",
      "postpower",
    ] {
      while idx < slots.len() {
        let r_matches = slots[idx].as_ref().map(|d| d.unit_type.as_str()) == Some(*role);
        let r_is_style = slots[idx].as_ref().map(|d| d.unit_type.as_str()) == Some("style");
        if r_matches {
          let d = slots[idx].take().unwrap();
          match *role {
            "per" => unit.per = true,
            "prepower" => unit.prepower = Some(d),
            "prefix" => unit.prefix = Some(d),
            "unit" => unit.unit = Some(d),
            "qualifier" => unit.qualifier = Some(d),
            "postpower" => unit.postpower = Some(d),
            _ => {},
          }
          idx += 1;
          break;
        } else if r_is_style {
          let d = slots[idx].take().unwrap();
          if unit.prefix.is_none() && unit.unit.is_none() {
            if d.name == "cancel" {
              unit.cancel = true;
            }
            if d.name == "highlight" {
              unit.highlight_color = d.color;
            }
          }
          idx += 1;
        } else {
          break;
        }
      }
    }

    if unit.unit.is_none()
      && unit.prefix.is_none()
      && !unit.per
      && unit.prepower.is_none()
      && unit.postpower.is_none()
    {
      if idx < slots.len() {
        idx += 1;
      }
      continue;
    }

    if saved_per {
      unit.per = true;
    } else if sticky_per && unit.per {
      saved_per = true;
    }

    units.push(unit);
  }

  units
}

/// Format a single unit
fn six_format_1unit(unit: &SixUnit) -> Tokens {
  // Resolve presentations: replace unit/prefix CS references with their final text
  let pre_resolved = unit
    .prefix
    .as_ref()
    .map(|p| resolve_unit_presentation(&p.presentation));
  let u_resolved = unit
    .unit
    .as_ref()
    .map(|u| resolve_unit_presentation(&u.presentation));
  let mut p = unit
    .prepower
    .as_ref()
    .and_then(|pp| pp.power.clone())
    .or_else(|| unit.postpower.as_ref().and_then(|pp| pp.power.clone()));

  if unit.per {
    p = Some(if let Some(pp) = p {
      let mut tks = vec![T_OTHER!("-")];
      tks.extend(pp.unlist());
      Tokens::new(tks)
    } else {
      Tokens::new(vec![T_OTHER!("-"), T_OTHER!("1")])
    });
  }

  // Build unit name for meaning
  let unit_name = format!(
    "{}{}",
    unit.prefix.as_ref().map(|p| p.name.as_str()).unwrap_or(""),
    unit.unit.as_ref().map(|u| u.name.as_str()).unwrap_or(""),
  );

  // Build presentation: \mathrm{resolved prefix + resolved unit}.
  //
  // Use `mouth::tokenize` (re-parses through the std catcode table)
  // rather than `ExplodeText!` (turns every char into an OTHER token).
  // For a prefix like `\micro` whose `resolve_unit_presentation` returns
  // the string "\SIUnitSymbolMicro", `ExplodeText!` would produce 17
  // literal OTHER tokens (\, S, I, U, n, i, t, S, y, m, b, o, l, M,
  // i, c, r, o), losing CS-ness and rendering as the literal text
  // `\SIUnitSymbolMicro` in the math output. With `mouth::tokenize`
  // we get a single CS token that downstream digestion can re-expand
  // to the µ glyph. Witness: 1410.8171 used to render `\SI{0,1}{\micro
  // \kelvin}` as the literal text `\SIUnitSymbolMicroK` in math.
  // Prefix and unit are tokenized separately (same as pre_resolved /
  // u_resolved are computed separately), so the prefix→unit boundary
  // is preserved at the Token level even though both strings happen
  // to be ASCII.
  let mut pres_inner = Vec::new();
  if let Some(pr) = &pre_resolved {
    pres_inner.extend(mouth::tokenize(pr).unlist());
  }
  if let Some(ut) = &u_resolved {
    pres_inner.extend(mouth::tokenize(ut).unlist());
  }

  // \lx@unit{name}{\mathrm{presentation}}
  let mut result_tks = vec![T_CS!("\\lx@unit"), T_BEGIN!()];
  result_tks.extend(ExplodeText!(&unit_name));
  result_tks.push(T_END!());
  result_tks.push(T_BEGIN!());
  result_tks.push(T_CS!("\\mathrm"));
  result_tks.push(T_BEGIN!());
  result_tks.extend(pres_inner);
  result_tks.push(T_END!());
  result_tks.push(T_END!());

  let mut result = Tokens::new(result_tks);

  // Apply power
  if let Some(pp) = p {
    let mut tks = vec![T_CS!("\\lx@power"), T_BEGIN!()];
    tks.extend(result.unlist());
    tks.push(T_END!());
    tks.push(T_BEGIN!());
    tks.extend(pp.unlist());
    tks.push(T_END!());
    result = Tokens::new(tks);
  }

  if unit.cancel {
    let mut tks = vec![T_CS!("\\cancel"), T_BEGIN!()];
    tks.extend(result.unlist());
    tks.push(T_END!());
    result = Tokens::new(tks);
  }

  result
}

/// Format product of units
fn six_format_unitproduct(bracketed: bool, units: &[SixUnit]) -> Tokens {
  let formatted: Vec<Tokens> = units.iter().map(six_format_1unit).collect();
  let inter = six_get_op_sym(
    &[
      ("role", Tokenize!("MULOP")),
      ("meaning", Tokenize!("times")),
    ],
    six_pin!("inter-unit-product"),
  );
  six_format_infix(
    inter,
    if bracketed {
      Some(six_get_op_sym(
        &[("role", Tokenize!("OPEN"))],
        six_pin!("open-bracket"),
      ))
    } else {
      None
    },
    if bracketed {
      Some(six_get_op_sym(
        &[("role", Tokenize!("CLOSE"))],
        six_pin!("close-bracket"),
      ))
    } else {
      None
    },
    formatted,
  )
}

/// Format units handling per-mode
fn six_format_units(units: &[SixUnit]) -> Tokens {
  let permode = six_get_choice_sym(six_pin!("per-mode"));
  // Perl siunitx.sty.ltxml L1062-1063: `symbol-or-fraction` is resolved up
  // front to `fraction` in display math, `symbol` otherwise. Without this
  // remap it falls through to the "Unknown siunitx per-mode" catchall.
  // Witness 1811.06895 (`\sisetup{per-mode=symbol-or-fraction}`).
  let permode = if permode == "symbol-or-fraction" {
    let is_display = lookup_font()
      .and_then(|f| f.mathstyle.as_ref().map(|ms| ms.as_ref() == "display"))
      .unwrap_or(false);
    if is_display {
      "fraction".to_string()
    } else {
      "symbol".to_string()
    }
  } else {
    permode
  };
  if permode == "reciprocal" || units.iter().all(|u| !u.per) {
    // Perl siunitx.sty.ltxml L1065-1066: each unit processed in order with
    // its own per (if any).
    return six_format_unitproduct(false, units);
  }

  // Perl L1068-1072: collect numerator & denominator units (positive vs
  // negative powers). The denominator units keep their `per` markers for
  // now — `reciprocal-positive-first` needs them intact.
  let mut numer_units: Vec<SixUnit> = Vec::with_capacity(units.len());
  let mut denom_units: Vec<SixUnit> = Vec::with_capacity(units.len());
  for u in units {
    if u.per {
      denom_units.push(u.clone());
    } else {
      numer_units.push(u.clone());
    }
  }

  if permode == "reciprocal-positive-first" {
    // Perl L1073-1074: re-ordered (numerators first, then denominators),
    // each per left as-is (markers NOT stripped).
    let mut all = numer_units;
    all.extend(denom_units);
    return six_format_unitproduct(false, &all);
  }

  // Perl L1075-1076: otherwise, remove the per markers from the
  // denominator units before formatting.
  for u in &mut denom_units {
    u.per = false;
  }

  if permode == "fraction" {
    // Perl L1077-1080
    let mut tks = vec![T_CS!("\\frac"), T_BEGIN!()];
    tks.extend(six_format_unitproduct(false, &numer_units).unlist());
    tks.push(T_END!());
    tks.push(T_BEGIN!());
    tks.extend(six_format_unitproduct(false, &denom_units).unlist());
    tks.push(T_END!());
    Tokens::new(tks)
  } else if permode == "repeated-symbol" {
    // Perl L1081-1085: the per-symbol (divide MULOP) prefixes EACH
    // denominator unit in turn — `numer / d1 / d2 / …` rather than
    // `numer / (d1·d2)`. Witness 1812.05943 (elsarticle,
    // `\sisetup{per-mode=repeated-symbol}`).
    let per_sym = six_get_op_sym(
      &[
        ("role", Tokenize!("MULOP")),
        ("meaning", Tokenize!("divide")),
      ],
      six_pin!("per-symbol"),
    );
    let mut result = six_format_unitproduct(false, &numer_units);
    for d in &denom_units {
      result = six_format_infix(per_sym.clone(), None, None, vec![
        result,
        six_format_1unit(d),
      ]);
    }
    result
  } else if permode == "symbol" {
    // Perl L1086-1093
    let bracket = denom_units.len() > 1 && six_get_bool_sym(six_pin!("bracket-unit-denominator"));
    let per_sym = six_get_op_sym(
      &[
        ("role", Tokenize!("MULOP")),
        ("meaning", Tokenize!("divide")),
      ],
      six_pin!("per-symbol"),
    );
    six_format_infix(per_sym, None, None, vec![
      six_format_unitproduct(false, &numer_units),
      six_format_unitproduct(bracket, &denom_units),
    ])
  } else {
    // Perl siunitx.sty.ltxml:1094 — Error('unexpected', $permode, undef,
    //   "Unknown siunitx per-mode $permode") for the catchall arm.
    // In Rust we still emit the structured error; defaulting back to
    // a flat unit product mirrors the Perl caller's recovery path.
    six_log_error!(
      "unexpected",
      permode,
      "Unknown siunitx per-mode {}",
      permode
    );
    six_format_unitproduct(false, units)
  }
}

/// Perl: six_parse_literalunits — parse literal (non-macro) unit expressions
fn six_parse_literalunits(expr: &Tokens) -> Tokens {
  let mut result = Vec::new();
  let mut iter = expr.unlist_ref().iter().copied().peekable();

  while let Some(t) = iter.next() {
    let tc = t.get_catcode();
    if t.text == pin!(".") {
      result.extend(six_get_tokens_sym(six_pin!("inter-unit-product")).unlist());
    } else if tc == Catcode::SUPER {
      if let Some(next) = iter.peek() {
        if next.get_catcode() == Catcode::BEGIN {
          iter.next();
          let mut g = Vec::new();
          let mut level = 1;
          for t2 in iter.by_ref() {
            if t2.get_catcode() == Catcode::END {
              level -= 1;
              if level == 0 {
                break;
              }
            } else if t2.get_catcode() == Catcode::BEGIN {
              level += 1;
            }
            g.push(t2);
          }
          result.push(T_SUPER!());
          result.push(T_BEGIN!());
          result.extend(g);
          result.push(T_END!());
        } else {
          let next = iter.next().unwrap();
          result.push(T_SUPER!());
          result.push(T_BEGIN!());
          result.push(next);
          result.push(T_END!());
        }
      }
    } else if tc == Catcode::LETTER || tc == Catcode::OTHER {
      result.push(T_CS!("\\mathrm"));
      result.push(T_BEGIN!());
      result.push(t);
      result.push(T_END!());
    } else if tc == Catcode::BEGIN {
      // Perl L1119-1123: peel an explicit `{...}` group and emit it as-is,
      // WITHOUT re-walking its contents. Without this, the group's letters
      // would be re-wrapped in `\mathrm{}` even though they were already
      // wrapped by `six_resolve_unit_objects` (driver: `\mathrm{c}` from
      // `\centi` → `\mathrm{\mathrm{c}}` double-wrap).
      let mut g = Vec::new();
      let mut level = 1;
      for t2 in iter.by_ref() {
        if t2.get_catcode() == Catcode::END {
          level -= 1;
          if level == 0 {
            break;
          }
        } else if t2.get_catcode() == Catcode::BEGIN {
          level += 1;
        }
        g.push(t2);
      }
      result.push(T_BEGIN!());
      result.extend(g);
      result.push(T_END!());
    } else {
      result.push(t);
    }
  }

  Tokens::new(result)
}

/// Perl: six_process_units — top-level unit processing
/// Tries structured unit parsing first, falls back to literal parsing.
/// For mixed content (\pi . \mm . \mrad), resolves \lx@six@unitobject tokens
/// to their \mathrm{presentation} BEFORE falling to literalunits, while the
/// siunitx_macros mapping is still active.
fn six_process_units(expr: &Tokens) -> Tokens {
  let expanded = do_expand_partially(expr.clone()).unwrap_or_else(|_| expr.clone());
  if let Some(defns) = six_convert_units_from_tokens(&expanded)
    && !defns.is_empty()
  {
    let units = six_parse_units(defns);
    return six_format_units(&units);
  }
  // Fallback: resolve any \lx@six@unitobject tokens to their presentation
  // while the siunitx_macros mapping is still active, then parse as literal.
  let resolved = six_resolve_unit_objects(&expanded);
  six_parse_literalunits(&resolved)
}

/// Replace \lx@six@unitobject{name} tokens with \mathrm{presentation} tokens.
/// Must be called while siunitx_macros mapping is active (inside \SI{}{} processing).
fn six_resolve_unit_objects(tokens: &Tokens) -> Tokens {
  let mut result = Vec::new();
  let mut iter = tokens.unlist_ref().iter().copied().peekable();
  let mut had_substitution = false;
  // Pre-intern the two dispatch CS names so the per-token check is
  // u32 equality (not `t.to_string()` alloc + string compare).
  let unitobject_sym = pin!("\\lx@six@unitobject");
  let unitobject_arg_sym = pin!("\\lx@six@unitobject@arg");

  while let Some(t) = iter.next() {
    if t.text == unitobject_sym {
      if let Some(name) = read_brace_group_str(&mut iter) {
        if let Some(Stored::String(encoded)) = lookup_mapping_sym(pin!("siunitx_macros"), &name) {
          // Decode directly from the arena-borrowed &str — was
          // cloning via `.to_string()` into a temporary String.
          if let Some(defn) = decode_unit_defn_from_encoded_sym(&name, encoded) {
            let pres = defn.presentation;
            if !pres.is_empty() {
              // Mirror Perl L1216 (`siunitx.sty.ltxml`):
              //   `return Tokens(T_CS('\mathrm'), T_BEGIN, $pres, T_END);`
              // The `\mathrm{...}` wrapper is required so each unit becomes a
              // separate math atom; emitting raw presentation tokens leaves
              // prepower/postpower presentations like `^{3}` (from `\cubic`)
              // bare, which the math digester then sees as a second
              // superscript on the preceding atom (driver: 2304.12803).
              result.push(T_CS!("\\mathrm"));
              result.push(T_BEGIN!());
              result.extend(pres.unlist());
              result.push(T_END!());
              had_substitution = true;
              continue;
            }
          }
        }
        // Fallback: emit name as raw text
        result.extend(ExplodeText!(&name));
        had_substitution = true;
      }
    } else if t.text == unitobject_arg_sym {
      if let Some(name) = read_brace_group_str(&mut iter)
        && let Some(arg) = read_brace_group_str(&mut iter)
        && let Some(Stored::String(encoded)) = lookup_mapping_sym(pin!("siunitx_macros"), &name)
        && let Some(defn) = decode_unit_defn_from_encoded_sym(&name, encoded)
      {
        let pres = defn.presentation;
        if !pres.is_empty() {
          // Perl L1223: `Tokens($pres, T_BEGIN, $data, T_END)` — the
          // presentation here takes the data as its argument (e.g.
          // `\tothe{2}` → `^{2}`). The presentation already supplies
          // its own grouping so we don't add a `\mathrm{...}` wrapper.
          result.extend(pres.unlist());
          result.push(T_BEGIN!());
          result.extend(ExplodeText!(&arg));
          result.push(T_END!());
          had_substitution = true;
          continue;
        }
      }
    } else {
      result.push(t);
    }
  }

  if had_substitution {
    Tokens::new(result)
  } else {
    tokens.clone() // No changes — return original
  }
}

/// Parse expanded tokens looking for \lx@six@unitobject{name} patterns
fn six_convert_units_from_tokens(tokens: &Tokens) -> Option<Vec<SixUnitDefn>> {
  let mut iter = tokens.unlist_ref().iter().copied().peekable();
  let mut defns = Vec::new();

  // Pre-intern the CS token names we dispatch on — avoid a per-iteration
  // `t.to_string()` String alloc (was called on every peeked token).
  let unitobject_sym = pin!("\\lx@six@unitobject");
  let unitobject_arg_sym = pin!("\\lx@six@unitobject@arg");
  let dot_sym = pin!(".");

  while let Some(t) = iter.peek() {
    if t.text == unitobject_sym {
      iter.next();
      // Read {name} group
      if let Some(name) = read_brace_group_str(&mut iter) {
        // Look up in siunitx_macros mapping
        if let Some(Stored::String(encoded)) = lookup_mapping_sym(pin!("siunitx_macros"), &name)
          && let Some(defn) = decode_unit_defn_from_encoded_sym(&name, encoded)
        {
          defns.push(defn);
        }
      }
    } else if t.text == unitobject_arg_sym {
      iter.next();
      if let Some(name) = read_brace_group_str(&mut iter)
        && let Some(arg) = read_brace_group_str(&mut iter)
        && let Some(Stored::String(encoded)) = lookup_mapping_sym(pin!("siunitx_macros"), &name)
        && let Some(mut defn) = decode_unit_defn_from_encoded_sym(&name, encoded)
      {
        // Apply arg to the appropriate field
        if defn.unit_type == "postpower" || defn.unit_type == "prepower" {
          defn.power = Some(Tokenize!(&arg));
        } else if defn.unit_type == "qualifier" {
          defn.presentation = Tokenize!(&arg);
        }
        defns.push(defn);
      }
    } else if t.get_catcode() == Catcode::SPACE || t.text == dot_sym {
      iter.next(); // skip spaces and dots (unit product separators)
    } else if !defns.is_empty() {
      // Non-unit content after some units found — stop here, use what we have.
      // Perl handles mixed content (e.g., \pi\per\milli\meter) by parsing units
      // and passing non-unit content through. We stop at the first unrecognized token.
      break;
    } else {
      return None; // No units found yet — fall back to literal
    }
  }

  Some(defns)
}

/// Read a brace-delimited group and return its string content
fn read_brace_group_str<I: Iterator<Item = Token>>(
  iter: &mut std::iter::Peekable<I>,
) -> Option<String> {
  if let Some(t) = iter.peek()
    && t.get_catcode() == Catcode::BEGIN
  {
    iter.next();
    let mut result = String::new();
    let mut level = 1;
    for t in iter.by_ref() {
      if t.get_catcode() == Catcode::END {
        level -= 1;
        if level == 0 {
          break;
        }
      } else if t.get_catcode() == Catcode::BEGIN {
        level += 1;
      }
      // `.with_str` borrows the arena entry directly — no per-token
      // String alloc (the prior `&t.to_string()` was ~30 ns/token).
      t.with_str(|s| result.push_str(s));
    }
    return Some(result);
  }
  None
}

// Cache of decoded SixUnitDefn "template" (everything except the
// caller-provided `name`) keyed by the encoded string's interned
// SymStr. Every \SI / \num / \si invocation re-decodes the same
// unit definitions (e.g. `\metre`, `\kilo`) for every usage — the
// raw decode allocates 3 Strings + 2 Token vectors each time.
// Caching by the encoded SymStr is automatically invalidation-safe:
// if a \DeclareSIUnit redefines `\metre`, the new encoded string
// interns to a different SymStr, so the old cache entry is simply
// never looked up again.
thread_local! {
  static UNIT_DEFN_CACHE: std::cell::RefCell<
    rustc_hash::FxHashMap<::latexml_core::common::arena::SymStr, SixUnitDefn>,
  > = std::cell::RefCell::new(rustc_hash::FxHashMap::default());
}

/// Decode a SixUnitDefn from "type|presentation|power|base" format.
/// Takes the already-interned `encoded_sym` so we can cache-lookup
/// without re-pinning. `name` is the caller's lookup key (not part of
/// the encoded value).
fn decode_unit_defn_from_encoded_sym(
  name: &str,
  encoded_sym: ::latexml_core::common::arena::SymStr,
) -> Option<SixUnitDefn> {
  // Fast path: cache hit by the encoded string's SymStr.
  if let Some(cached) = UNIT_DEFN_CACHE.with(|c| c.borrow().get(&encoded_sym).cloned()) {
    let mut defn = cached;
    defn.name = name.to_string();
    return Some(defn);
  }

  let defn = ::latexml_core::common::arena::with(encoded_sym, |encoded| {
    let parts: Vec<&str> = encoded.splitn(4, '|').collect();
    if parts.is_empty() {
      return None;
    }
    Some(SixUnitDefn {
      name:         name.to_string(),
      unit_type:    parts[0].to_string(),
      presentation: Tokenize!(parts.get(1).unwrap_or(&"")),
      power:        parts.get(2).and_then(|p| {
        if p.is_empty() {
          None
        } else {
          Some(Tokenize!(p))
        }
      }),
      base:         parts.get(3).and_then(|b| b.parse().ok()),
      arg:          None,
      color:        None,
    })
  })?;
  UNIT_DEFN_CACHE.with(|c| c.borrow_mut().insert(encoded_sym, defn.clone()));
  Some(defn)
}

/// Perl: six_enableUnitMacros — let each unit CS point to its lx@six@ implementation
pub(crate) fn six_enable_unit_macros(overwrite: bool) {
  // with_value avoids the Stored::String envelope clone; we only need
  // the inner SymStr stringified for iteration.
  let names_str = with_value("siunitx_macro_names", |v| match v {
    Some(Stored::String(s)) => with(*s, |s| s.to_string()),
    _ => String::new(),
  });
  for name in names_str.split(',') {
    if name.is_empty() {
      continue;
    }
    let cs = T_CS!(&format!("\\{name}"));
    let impl_cs = T_CS!(&format!("\\lx@six@{name}"));
    if overwrite || !has_meaning(&cs) {
      let_i(&cs, &impl_cs, None);
    }
  }
}

/// Register a unit macro name for six_enableUnitMacros
fn register_unit_macro_name(name: &str) {
  let existing = with_value("siunitx_macro_names", |v| {
    v.map(|s| s.to_string()).unwrap_or_default()
  });
  let new = if existing.is_empty() {
    name.to_string()
  } else {
    format!("{existing},{name}")
  };
  assign_value("siunitx_macro_names", Stored::from(new), None);
}

/// Define a macro dynamically (CS, no params, expansion tokens)
fn define_macro_simple(cs: Token, expansion: Tokens) -> Result<()> {
  let def = Expandable::new(cs, None, Some(ExpansionBody::Tokens(expansion)), None)?;
  install_definition(def, None);
  Ok(())
}

/// Resolve unit presentation: if the presentation contains CS tokens that are
/// known unit macros, replace them with their stored presentation text.
/// E.g., \DeclareSIUnit \metre { \meter } → resolve \meter → "m"
fn resolve_unit_presentation(pres: &Tokens) -> String {
  let mut result = String::new();
  for tok in pres.unlist_ref() {
    if tok.get_catcode() == Catcode::CS {
      let cs_name = tok.to_string();
      let unit_name = cs_name.trim_start_matches('\\');
      // Look up in siunitx_macros mapping
      if let Some(Stored::String(encoded)) = lookup_mapping_sym(pin!("siunitx_macros"), unit_name) {
        let encoded_str = with(encoded, |s| s.to_string());
        let parts: Vec<&str> = encoded_str.splitn(4, '|').collect();
        if parts.len() >= 2 {
          let pres = parts[1];
          // If the presentation itself contains backslashes, try to resolve recursively
          if pres.contains('\\') {
            let sub = Tokenize!(pres);
            let sub_resolved = resolve_unit_presentation(&sub);
            result.push_str(&sub_resolved);
          } else {
            result.push_str(pres);
          }
          continue;
        }
      }
      // Not a unit macro — keep as-is
      result.push_str(&cs_name);
    } else if tok.get_catcode() != Catcode::SPACE {
      result.push_str(&tok.to_string());
    }
  }
  result
}

/// Build expansion tokens: \lx@six@unitobject{name}
/// Using T_CS! to avoid catcode issues with @ in the CS name.
fn make_unitobject_expansion(name: &str) -> Tokens {
  let mut tks = vec![T_CS!("\\lx@six@unitobject"), T_BEGIN!()];
  tks.extend(ExplodeText!(name));
  tks.push(T_END!());
  Tokens::new(tks)
}

/// Build expansion tokens: \lx@six@unitobject@collapsible{name}{presentation}
/// Perl L1227-1249: If presentation expands to more \lx@six@unitobject tokens,
/// pass them through (alias collapsing, e.g. \metre → \meter).
/// Otherwise fall back to \lx@six@unitobject{name}.
///
/// The presentation MUST be passed as Tokens (preserving CS catcodes), not
/// stringified-then-re-exploded — `ExplodeText!` would turn `\meter` into
/// the six character tokens `\`, `m`, `e`, `t`, `e`, `r` and silently break
/// the alias chain. Perl's `DefMacroI` re-tokenizes its body string with
/// current catcodes and produces a real CS token, so the Rust port must
/// pass real CS tokens through.
fn make_collapsible_expansion(name: &str, presentation: &Tokens) -> Tokens {
  let mut tks = vec![T_CS!("\\lx@six@unitobject@collapsible"), T_BEGIN!()];
  tks.extend(ExplodeText!(name));
  tks.push(T_END!());
  tks.push(T_BEGIN!());
  tks.extend(presentation.unlist_ref().iter().copied());
  tks.push(T_END!());
  Tokens::new(tks)
}

//======================================================================
// LoadDefinitions! block
//======================================================================

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("expl3");
  RequirePackage!("xcolor");
  RequirePackage!("amstext");
  RequirePackage!("array");
  // siunitx's unit parser accepts `\cancel{<unit>}` as a "style"
  // (registered as a builtin below), and the formatter at
  // `six_format_1unit` emits `\cancel{...}` in the output token stream.
  // Auto-load the cancel binding so papers that use the unit-cancel
  // style without explicitly `\usepackage{cancel}`'ing it still
  // convert. (TL siunitx registers its own internal `\cancel` for the
  // same reason: `\__siunitx_unit_set_symbolic:Npnn \cancel { ... }`.)
  // Witness: arXiv:2602.18218 `\SI{1}{\milli\electronvolt\per\cancel{c^2}}`.
  RequirePackage!("cancel");

  //======================================================================
  // Boolean SIX options. Perl siunitx.sty.ltxml L38-54:
  //   foreach my $key (qw(...)) { DefKeyVal('SIX', $key, '', 'true'); }
  // Without these registrations, the keyvals lookup during `\sisetup{...}`
  // / `\SI[opts]{...}{...}` falls through to the unknown-key path and
  // emits "Encountered unknown KeyVals key" (Warn-level after `21e730e71e`).
  for key in [
    "version-1-compatibility", "abbreviations", "binary-units",
    "free-standing-units", "overwrite-functions",
    "bracket-numbers", "detect-family", "detect-italic", "detect-mode",
    "detect-shape", "detect-weight", "multi-part-units", "parse-numbers",
    "parse-units", "product-units",
    "copy-complex-root", "copy-decimal-marker",
    "bracket-negative-numbers",
    "separate-uncertainty", "tight-spacing",
    "retain-explicit-plus", "add-decimal-zero", "add-integer-zero",
    "retain-unity-mantissa", "retain-zero-exponent",
    "omit-uncertainty",
    "add-arc-degree-zero", "add-arc-minute-zero", "add-arc-second-zero",
    "angle-symbol-over-decimal",
    "sticky-per", "prefixes-as-symbols",
  ] {
    DefKeyVal!("SIX", key, "", "true");
  }
  // Non-boolean SIX options that siunitx initializes via the
  // `\sisetup{...}` defaults block below (L2495-L2540 in this file).
  // Perl leaves these unregistered (Info-level pass-through under
  // KeyVals.pm:97). Rust would otherwise emit ~30 Warn entries per
  // siunitx-using paper after `21e730e71e`'s Info→Warn promotion —
  // pure noise from siunitx-internal initialization, not user typos.
  // Rust-only divergence: register the keys siunitx uses so the Warn
  // level is reserved for genuinely unknown keys (typos, version drift).
  for key in [
    "input-product", "input-quotient", "input-close-uncertainty",
    "input-complex-roots", "input-comparators", "input-decimal-markers",
    "input-digits", "input-exponent-markers", "input-open-uncertainty",
    "input-protect-tokens", "input-signs", "input-symbols",
    "input-uncertainty-signs",
    "close-bracket", "open-bracket", "complex-root-position",
    "exponent-base", "exponent-product",
    "group-digits", "group-minimum-digits", "group-separator",
    "output-close-uncertainty", "output-complex-root",
    "output-decimal-marker", "output-open-uncertainty",
    "output-product", "output-quotient",
    "fraction-function", "quotient-mode", "per-mode", "per-symbol",
    "qualifier-mode", "bracket-unit-denominator",
    "inter-unit-product", "number-unit-product", "number-angle-product",
    "list-final-separator", "list-pair-separator", "list-separator",
    "list-units", "range-phrase", "range-units",
    "arc-separator", "alsoload", "color",
    // siunitx 2.x rounding / formatting / table-figure / scientific
    // options exercised by `tests/complex/si.tex`.
    "round-mode", "round-precision", "round-half", "round-minimum",
    "round-integer-to-decimal",
    "scientific-notation", "fixed-exponent", "exponent-to-prefix",
    "explicit-sign", "literal-superscript-as-power",
    "minimum-integer-digits", "group-four-digits", "negative-color",
    "output-exponent-marker", "power-font",
    "qualifier-phrase", "uncertainty-separator",
    "table-figures-decimal", "table-figures-exponent",
    "table-figures-integer", "table-format",
    "table-number-alignment",
    "table-space-text-post", "table-space-text-pre",
  ] {
    DefKeyVal!("SIX", key, "", "");
  }

  //======================================================================
  // Key symbols
  DefMath!("\\SIUnitSymbolDegree", None, "\u{00B0}",
    meaning => "arcdegree", name => "");
  DefMath!("\\SIUnitSymbolArcminute", None, "\u{2032}",
    meaning => "arcminute", name => "");
  DefMath!("\\SIUnitSymbolArcsecond", None, "\u{2033}",
    meaning => "arcsecond", name => "");
  DefMath!("\\SIUnitSymbolCelsius", None, "\u{00B0}C");
  DefMath!("\\SIUnitSymbolOhm", None, "\u{2126}");
  DefMath!("\\SIUnitSymbolAngstrom", None, "\u{00C5}");
  DefMath!("\\SIUnitSymbolMicro", None, "\u{00B5}");

  //======================================================================
  // \lx@unit constructor — Perl siunitx.sty.ltxml L1009: `requireMath => 1`
  // forces the wrapper into math mode when invoked in text context, so the
  // inner XMWrap lands inside an <ltx:Math> subtree as Perl expects.
  DefConstructor!("\\lx@unit{}{}",
    "<ltx:XMWrap role='ID' meaning='#1' class='ltx_unit'>#2</ltx:XMWrap>",
    require_math => true, reversion => "#2");

  //======================================================================
  // Arc overlays
  DefConstructor!("\\lx@zerowidthperiod",
    "<ltx:XMTok width='0pt'>.</ltx:XMTok>");
  DefMacro!("\\lx@arcdegreeoverdot", "\\lx@zerowidthperiod\\SIUnitSymbolDegree");
  DefMacro!("\\lx@arcminuteoverdot", "\\lx@zerowidthperiod\\SIUnitSymbolArcminute");
  DefMacro!("\\lx@arcsecondoverdot", "\\lx@zerowidthperiod\\SIUnitSymbolArcsecond");

  //======================================================================
  // \sisetup
  DefPrimitive!("\\sisetup RequiredKeyVals:SIX", sub[(kv_arg)] {
    six_setup(&kv_arg);
  });

  def_macro_noop("\\ProvidesExplFile{}{}{}{}")?;

  //======================================================================
  // \lx@six@initialize
  DefPrimitive!("\\lx@six@initialize", {
    // Perl siunitx.sty.ltxml L112-115: rebuild and digest \sisetup{...}
    // from `opt@siunitx.sty` package-option list — so options passed via
    // `\usepackage[alsoload=synchem, ...]{siunitx}` reach the SIX_*
    // state values (otherwise only \sisetup{...} in the doc body would
    // populate them).
    //   my $pkgoptions = LookupValue('opt@siunitx.sty');
    //   my $setup = $pkgoptions && Tokenize('\sisetup{' . join(',', @$pkgoptions) . '}');
    //   Digest($setup) if $setup;
    let pkg_opts: Vec<String> = match lookup_value("opt@siunitx.sty") {
      Some(Stored::VecDequeStored(vdq)) => vdq.iter().filter_map(|item| match item {
        Stored::String(s) => Some(with(*s, |s| s.to_string())),
        _ => None,
      }).collect(),
      Some(Stored::Strings(rc)) => rc.iter().map(|s| with(*s, |s| s.to_string())).collect(),
      _ => Vec::new(),
    };
    if !pkg_opts.is_empty() {
      let setup = format!("\\sisetup{{{}}}", pkg_opts.join(","));
      Digest!(Tokenize!(&setup))?;
    }

    // Perl siunitx.sty.ltxml L115-121: if version-1-compatibility OR
    // alsoload is set, load six_load_compat1's DeclareSIPrePower /
    // DeclareSIUnit chain. The user's `\usepackage[alsoload=synchem]
    // {siunitx}` sets alsoload=synchem (truthy) so compat1 fires;
    // `\Molar`, `\torr`, `\angstrom`, `\parsec`, `\lightyear` etc.
    // become defined. Witness: 2209.04575 (Quantum Sci. Technol., uses
    // `\SI{50}{\micro\Molar}`).
    let v1_compat = six_get_bool_sym(six_pin!("version-1-compatibility"));
    let alsoload = six_get_sym(six_pin!("alsoload"))
      .map(|s| !s.to_string().trim().is_empty())
      .unwrap_or(false);
    if v1_compat || alsoload {
      RawTeX!(SIX_LOAD_COMPAT1);
    }
    if six_get_bool_sym(six_pin!("free-standing-units")) {
      six_enable_unit_macros(six_get_bool_sym(six_pin!("overwrite-functions")));
    }
  });

  at_begin_document(TokenizeInternal!(r"\lx@six@initialize"))?;

  //======================================================================
  // Unit object macros (fallback expansion).
  // Perl double-binds: first DefPrimitive (empty body, :1209) then DefMacro
  // with a siunitx_macros lookup sub (:1212, protected=>1) that emits
  // `\mathrm{presentation}` or "??". Rust uses only the empty DefPrimitive
  // — the literal-parser (`six_parse_literalunits`) walks token streams
  // containing these CSes and performs the siunitx_macros lookup itself
  // (see :1570+), so the DefMacro fallback would never fire in normal
  // control flow. Structural adaptation — audit flags last-binding-wins.
  DefPrimitive!("\\lx@six@unitobject{}", "");
  DefPrimitive!("\\lx@six@unitobject@arg{}{}", "");

  // Collapsible unit object: if presentation expands to unit objects, pass them through.
  // Otherwise fall back to \lx@six@unitobject{name}.
  // Perl L1227-1249: Enables alias collapsing (e.g. \metre → \meter).
  DefMacro!("\\lx@six@unitobject@collapsible{}{}", sub [args] {
    let name = args[0].to_string();
    let data_toks = args[1].clone().owned_tokens().unwrap_or_default();
    // Fully expand the presentation data (not just partially)
    // This resolves the chain: \meter → \lx@six@meter → collapsible{meter}{m} → unitobject{meter}
    let expanded = do_expand(data_toks)?;
    let toks = expanded.unlist();
    let mut result: Vec<Token> = Vec::new();
    let mut i = 0;
    let mut all_units = true;
    while i < toks.len() {
      let ts = toks[i].to_string();
      if ts == "\\lx@six@unitobject" || ts == "\\lx@six@unitobject@arg" {
        let ngroups = if ts == "\\lx@six@unitobject@arg" { 2 } else { 1 };
        result.push(toks[i]);
        i += 1;
        // Collect the brace groups
        for _ in 0..ngroups {
          if i < toks.len() {
            result.push(toks[i]); // T_BEGIN
            i += 1;
            let mut depth = 1;
            while i < toks.len() && depth > 0 {
              if toks[i].code == Catcode::BEGIN { depth += 1; }
              if toks[i].code == Catcode::END { depth -= 1; }
              result.push(toks[i]);
              i += 1;
            }
          }
        }
      } else if toks[i].code == Catcode::SPACE {
        i += 1; // Skip spaces (Perl L1245-1246)
      } else {
        // Non-unit content found — fall back to simple unitobject
        all_units = false;
        break;
      }
    }
    if all_units && !result.is_empty() {
      Tokens::new(result)
    } else {
      make_unitobject_expansion(&name)
    }
  });

  //======================================================================
  // Unit declaration primitives
  //======================================================================

  // \DeclareSIUnit [kv] \cs {presentation}
  // Perl `DefPrimitive('\DeclareSIUnit OptionalKeyVals:SIX SkipSpaces DefToken {}', ...)`
  // — `DefToken` reads either `\cs` directly OR `{\cs}` (braced) form.
  // hepunits.sty L79+ writes `\DeclareSIUnit{\invbarn}{\barn\tothe{-1}}`
  // (braced); siunitx's own internal config uses `\DeclareSIUnit \cs {…}`
  // (unbraced). Both must work. `DefToken` parameter type covers both;
  // earlier `gullet::read_token` would treat `{` as the token in the
  // braced case and miss the binding entirely, leaving \invbarn undefined.
  // Spec mirrors Perl `\DeclareSIUnit OptionalKeyVals:SIX SkipSpaces DefToken {}`
  // (siunitx.sty.ltxml:1343). The `SkipSpaces` between `[]` and the
  // CS is load-bearing: papers commonly write
  // `\DeclareSIUnit[opt=val] \dBm{dBm}` with a space before the CS,
  // which without `SkipSpaces` causes the DefToken arg to read the
  // space character and the actual CS to be parsed as the body.
  // Driver: 1501.03532 (stage 17 RUST-REGRESSION: \dBm undefined).
  DefPrimitive!("\\DeclareSIUnit[] SkipSpaces DefToken {}", sub[(_kv, cs, presentation)] {
    let name = cs.to_string().trim_start_matches('\\').to_string();
    let newcs_name = format!("\\lx@six@{name}");

    // Store in mapping as a simple string encoding (raw presentation)
    let pres_str = presentation.to_string();
    let encoded = format!("unit|{}", pres_str);
    assign_mapping("siunitx_macros", &name, Some(Stored::from(encoded)));
    register_unit_macro_name(&name);

    // Define \lx@six@<name> → expands to \lx@six@unitobject@collapsible{name}{presentation}
    // Perl L1350-1351: Uses collapsible form to enable alias resolution
    define_macro_simple(T_CS!(&newcs_name), make_collapsible_expansion(&name, &presentation))?;

    // Let \cs = \relax if not yet defined
    if !has_meaning(&cs) {
      let_i(&cs, &T_CS!("\\relax"), None);
    }
  });

  // \DeclareSIPrefix [kv] \cs {presentation} {power}
  DefPrimitive!("\\DeclareSIPrefix[]", {
    skip_spaces()?;
    let cs = read_si_declare_cs()?;
    skip_spaces()?;
    let presentation = read_arg(ExpansionLevel::Off)?;
    let power = read_arg(ExpansionLevel::Off)?;
    let name = cs.to_string().trim_start_matches('\\').to_string();
    let newcs_name = format!("\\lx@six@{name}");

    let pres_str = presentation.to_string();
    let encoded = format!("prefix|{}|{}|10", pres_str, power);
    assign_mapping("siunitx_macros", &name, Some(Stored::from(encoded)));
    register_unit_macro_name(&name);

    // Perl L1264: Uses collapsible form for prefix declarations
    define_macro_simple(T_CS!(&newcs_name), make_collapsible_expansion(&name, &presentation))?;
    if !has_meaning(&cs) {
      let_i(&cs, &T_CS!("\\relax"), None);
    }
  });

  // \DeclareSIPrePower [kv] \cs {power}
  DefPrimitive!("\\DeclareSIPrePower[]", {
    skip_spaces()?;
    let cs = read_si_declare_cs()?;
    skip_spaces()?;
    let power = read_arg(ExpansionLevel::Off)?;
    let name = cs.to_string().trim_start_matches('\\').to_string();
    let newcs_name = format!("\\lx@six@{name}");

    let encoded = format!("prepower|^{{{}}}|{}", power, power);
    assign_mapping("siunitx_macros", &name, Some(Stored::from(encoded)));
    register_unit_macro_name(&name);

    define_macro_simple(T_CS!(&newcs_name), make_unitobject_expansion(&name))?;
    if !has_meaning(&cs) {
      let_i(&cs, &T_CS!("\\relax"), None);
    }
  });

  // \DeclareSIPostPower [kv] \cs {power}
  DefPrimitive!("\\DeclareSIPostPower[]", {
    skip_spaces()?;
    let cs = read_si_declare_cs()?;
    skip_spaces()?;
    let power = read_arg(ExpansionLevel::Off)?;
    let name = cs.to_string().trim_start_matches('\\').to_string();
    let newcs_name = format!("\\lx@six@{name}");

    let encoded = format!("postpower|^{{{}}}|{}", power, power);
    assign_mapping("siunitx_macros", &name, Some(Stored::from(encoded)));
    register_unit_macro_name(&name);

    define_macro_simple(T_CS!(&newcs_name), make_unitobject_expansion(&name))?;
    // Let \cs = \relax if not yet defined (prevents "undefined" errors)
    if !has_meaning(&cs) {
      let_i(&cs, &T_CS!("\\relax"), None);
    }
  });

  // \DeclareSIQualifier [kv] \cs {qualifier}
  DefPrimitive!("\\DeclareSIQualifier[]", {
    skip_spaces()?;
    let cs = read_si_declare_cs()?;
    skip_spaces()?;
    let qualifier = read_arg(ExpansionLevel::Off)?;
    let name = cs.to_string().trim_start_matches('\\').to_string();
    let newcs_name = format!("\\lx@six@{name}");

    let encoded = format!("qualifier|{}", qualifier);
    assign_mapping("siunitx_macros", &name, Some(Stored::from(encoded)));
    register_unit_macro_name(&name);

    define_macro_simple(T_CS!(&newcs_name), make_unitobject_expansion(&name))?;
    if !has_meaning(&cs) {
      let_i(&cs, &T_CS!("\\relax"), None);
    }
  });

  // \DeclareBinaryPrefix [kv] \cs {presentation} {power}
  DefPrimitive!("\\DeclareBinaryPrefix[]", {
    skip_spaces()?;
    let cs = read_si_declare_cs()?;
    skip_spaces()?;
    let presentation = read_arg(ExpansionLevel::Off)?;
    let power = read_arg(ExpansionLevel::Off)?;
    let name = cs.to_string().trim_start_matches('\\').to_string();
    let newcs_name = format!("\\lx@six@{name}");

    let encoded = format!("prefix|{}|{}|2", presentation, power);
    assign_mapping("siunitx_macros", &name, Some(Stored::from(encoded)));
    register_unit_macro_name(&name);

    define_macro_simple(T_CS!(&newcs_name), make_unitobject_expansion(&name))?;
    if !has_meaning(&cs) {
      let_i(&cs, &T_CS!("\\relax"), None);
    }
  });

  //======================================================================
  // Special builtins: \per, \tothe, \raiseto, \of, \highlight
  //======================================================================

  DefMacro!("\\lx@six@per", "\\lx@six@unitobject{per}");
  DefMacro!("\\lx@six@tothe{}", "\\lx@six@unitobject@arg{tothe}{#1}");
  DefMacro!("\\lx@six@raiseto{}", "\\lx@six@unitobject@arg{raiseto}{#1}");
  DefMacro!("\\lx@six@of{}", "\\lx@six@unitobject@arg{of}{#1}");
  DefMacro!("\\lx@six@highlight{}", "\\lx@six@unitobject@arg{highlight}{#1}");
  DefMacro!("\\lx@six@cancel", "\\lx@six@unitobject{cancel}");

  // Register builtins in mapping
  DefPrimitive!("\\lx@six@builtins@setup", {
    let builtins = [
      ("per", "per|/"),
      ("cancel", "style|"),
      ("highlight", "style|"),
      ("tothe", "postpower|^"),
      ("raiseto", "prepower|^"),
      ("of", "qualifier|\\mathrm"),
    ];
    for (name, encoded) in &builtins {
      assign_mapping("siunitx_macros", name, Some(Stored::from(encoded.to_string())));
      register_unit_macro_name(name);
    }
  });
  RawTeX!(r"\lx@six@builtins@setup");

  //======================================================================
  // Top-level macros: \num, \si, \SI, etc.
  //======================================================================

  // \num[options]{number}
  DefMacro!("\\num OptionalKeyVals:SIX {}", sub[(kv, number_arg)] {
    let number_tokens = number_arg;
    six_begin_processing(kv.as_ref());
    let parsed = six_parse_number(&number_tokens);
    let formatted = six_format_number(&parsed, 0);
    let result = six_wrap(formatted);
    six_end_processing();
    Ok(result)
  });

  // \numlist[options]{number;...}
  DefMacro!("\\numlist OptionalKeyVals:SIX {}", sub[(kv, numbers_arg)] {
    let numbers = numbers_arg;
    six_begin_processing(kv.as_ref());
    let parsed = six_parse_numbers(&numbers);
    let formatted: Vec<Tokens> = parsed.iter().map(|p| six_format_number(p, 0)).collect();
    let list = six_format_list(false, formatted);
    let result = six_wrap(list);
    six_end_processing();
    Ok(result)
  });

  // \numrange[options]{first}{last}
  DefMacro!("\\numrange OptionalKeyVals:SIX {}{}", sub[(kv, first_arg, last_arg)] {
    let first = first_arg;
    let last = last_arg;
    six_begin_processing(kv.as_ref());
    let f = six_format_number(&six_parse_number(&first), 0);
    let l = six_format_number(&six_parse_number(&last), 0);
    let result = six_wrap(six_format_range(false, f, l));
    six_end_processing();
    Ok(result)
  });

  // \ang[options]{degrees;minutes;seconds}
  DefMacro!("\\ang OptionalKeyVals:SIX {}", sub[(kv, expr_arg)] {
    let expr = expr_arg;
    six_begin_processing(kv.as_ref());

    let mut items = six_parse_numbers(&expr);
    // Pad to 3 components so degree/minute/second indices are addressable.
    while items.len() < 3 {
      items.push(SixParseResult::Empty);
    }
    // Perl `\ang`: substitute "0" for an EMPTY component when the matching
    // `add-arc-<unit>-zero` option is set — minute/second gated on the earlier
    // components carrying no fractional part (siunitx.sty.ltxml L802-813).
    {
      let is_empty = |it: &SixParseResult| matches!(it, SixParseResult::Empty);
      let has_frac = |it: &SixParseResult| {
        matches!(it, SixParseResult::Parsed(SixNumber::Simple { fraction: Some(_), .. }))
      };
      let addd0 = is_empty(&items[0]) && six_get_bool_sym(six_pin!("add-arc-degree-zero"));
      let addm0 = is_empty(&items[1])
        && six_get_bool_sym(six_pin!("add-arc-minute-zero"))
        && (is_empty(&items[0]) || !has_frac(&items[0]));
      let adds0 = is_empty(&items[2])
        && six_get_bool_sym(six_pin!("add-arc-second-zero"))
        && (is_empty(&items[0]) || !has_frac(&items[0]))
        && (is_empty(&items[1]) || !has_frac(&items[1]));
      if addd0 {
        items[0] = SixParseResult::Parsed(SixNumber::simple(None, Some(Tokenize!("0")), None, None));
      }
      if addm0 {
        items[1] = SixParseResult::Parsed(SixNumber::simple(None, Some(Tokenize!("0")), None, None));
      }
      if adds0 {
        items[2] = SixParseResult::Parsed(SixNumber::simple(None, Some(Tokenize!("0")), None, None));
      }
    }
    // Perl `\ang`: pull the (overall) sign out of the first signed component —
    // it applies to the whole angle — and clear the per-component signs so it
    // renders once, in front (siunitx.sty.ltxml L815-821). Without this,
    // `\ang{;-2;}` with add-arc-degree-zero formats as `0°-2′` instead of
    // Perl's `-0°2′`.
    let overall_sign: Option<Tokens> = items.iter().find_map(|it| match it {
      SixParseResult::Parsed(n) => n.get_sign().cloned(),
      _ => None,
    });
    for it in items.iter_mut() {
      if let SixParseResult::Parsed(n) = it {
        n.set_sign(None);
      }
    }
    let sign_prefix: &str = match overall_sign.as_ref().map(|s| s.to_string()) {
      Some(st) if st.contains('-') => "-",
      Some(st) if st.contains('+') => "+",
      _ => "",
    };
    let mulop = Tokens::new(vec![T_CS!("\\lx@InvisibleTimes")]);
    let mut parts = Vec::new();

    // Perl `\ang` pushes a D/M/S component only if it is defined AND formats
    // to non-empty (`if ($fdegrees && $fdegrees->unlist)`). An `Empty`
    // component (e.g. the `;;` of `\ang{;;1.0}`) contributes no symbol.
    let push_part = |parts: &mut Vec<Tokens>, item: Option<&SixParseResult>, symbol: &str, mulop: &Tokens| {
      if let Some(c) = item
        && !matches!(c, SixParseResult::Empty) {
          let f = six_format_number(c, 0);
          if !f.is_empty() {
            parts.push(six_format_infix(
              mulop.clone(), None, None,
              vec![f, Tokens::new(vec![T_CS!(symbol)])],
            ));
          }
        }
    };
    push_part(&mut parts, items.first(), "\\SIUnitSymbolDegree", &mulop);
    push_part(&mut parts, items.get(1), "\\SIUnitSymbolArcminute", &mulop);
    push_part(&mut parts, items.get(2), "\\SIUnitSymbolArcsecond", &mulop);

    let combined = if parts.len() > 1 {
      let addop = Tokens::new(vec![T_CS!("\\lx@InvisiblePlus")]);
      six_format_infix(addop, None, None, parts)
    } else {
      parts.into_iter().next().unwrap_or_default()
    };
    // Perl: `if ($sign) { @punctuated = I_apply({}, $sign, @punctuated); }`
    let combined = if let Some(sign) = &overall_sign {
      i_apply(&[], sign.clone(), vec![combined])
    } else {
      combined
    };

    let mut meaning = String::from(sign_prefix);
    if let Some(SixParseResult::Parsed(d)) = items.first() {
      meaning.push_str(&six_number_string(d)); meaning.push('\u{00B0}');
    }
    if let Some(SixParseResult::Parsed(m)) = items.get(1) {
      meaning.push_str(&six_number_string(m)); meaning.push('\u{2032}');
    }
    if let Some(SixParseResult::Parsed(s)) = items.get(2) {
      meaning.push_str(&six_number_string(s)); meaning.push('\u{2033}');
    }

    let symbol = i_symbol(
      &[("role", Tokenize!("NUMBER")), ("meaning", Tokenize!(&meaning))],
      None,
    );
    let dual = i_dual(&[], symbol, combined, vec![]).unwrap_or_default();
    let result = six_wrap(dual);
    six_end_processing();
    Ok(result)
  });

  // \si[options]{units}
  DefMacro!("\\si OptionalKeyVals:SIX {}", sub[(kv, units_arg)] {
    let units = units_arg;
    six_begin_processing(kv.as_ref());
    six_enable_unit_macros(true);
    let funits = six_wrap(six_process_units(&units));
    six_end_processing();
    Ok(funits)
  });

  // siunitx v3 \unit[options]{units} — equivalent to v2 \si — but ONLY when
  // `\unit` isn't already defined. The older `units` package also defines
  // `\unit` (the `\unit[value]{unit}` syntax), and when both packages are
  // loaded Perl keeps units' `\unit` (siunitx.sty.ltxml never redefines it).
  // Unconditionally `\let`-ing `\unit` to `\si` clobbered units' command, so
  // `\unit[1.8$\times$10$^{17}$]{s$^{-1}$}` (units syntax) had its optional
  // `[…]` mis-scanned by `\si` → the inner `$…$` never entered math → `^`
  // "can only appear in math mode" (witness 1610.06392, units+siunitx). The
  // `\@ifundefined` guard preserves siunitx-only `\unit` (witnesses 2406.02765,
  // 2406.18417: `\unit` is undefined there, so it is defined here) while
  // deferring to units' `\unit` when that package is present — matching Perl,
  // and order-independent (if siunitx loads first, units' later DefMacro wins).
  RawTeX!(r"\@ifundefined{unit}{\let\unit\si}{}");

  // siunitx v3 \qty[options]{number}{units} — equivalent to v2 \SI.
  // Witness 2406.20067, 2407.03167. Defined below by Let to \SI after
  // \SI's own DefMacro registration.

  // \SI[options]{number}{units}
  DefMacro!("\\SI OptionalKeyVals:SIX {}{}", sub[(kv, number_arg, units_arg)] {
    let number = number_arg;
    let units = units_arg;
    six_begin_processing(kv.as_ref());
    let fnumber = six_format_number(&six_parse_number(&number), 0);
    six_enable_unit_macros(true);
    let times = six_get_op_sym(&[("role", Tokenize!("MULOP")), ("meaning", Tokenize!("times"))], six_pin!("number-unit-product"));
    let funits = i_wrap(None, six_process_units(&units));
    let result = six_wrap(six_format_infix(times, None, None, vec![fnumber, funits]));
    six_end_processing();
    Ok(result)
  });

  // \SIlist[options]{number;...}{units}
  DefMacro!("\\SIlist OptionalKeyVals:SIX {}{}", sub[(kv, numbers_arg, units_arg)] {
    let numbers = numbers_arg;
    let units = units_arg;
    six_begin_processing(kv.as_ref());
    let times = six_get_op_sym(&[("role", Tokenize!("MULOP")), ("meaning", Tokenize!("times"))], six_pin!("number-unit-product"));
    let mode = six_get_choice_sym(six_pin!("list-units"));
    let items: Vec<Tokens> = six_parse_numbers(&numbers).iter()
      .map(|p| six_format_number(p, 0)).collect();
    six_enable_unit_macros(true);
    let funits = six_process_units(&units);
    let result = if mode == "repeat" {
      let items_with_units: Vec<Tokens> = items.iter()
        .map(|item| six_format_infix(times.clone(), None, None, vec![item.clone(), funits.clone()]))
        .collect();
      six_format_list(mode == "brackets", items_with_units)
    } else {
      let list = six_format_list(mode == "brackets", items);
      six_format_infix(times, None, None, vec![list, funits])
    };
    let result = six_wrap(result);
    six_end_processing();
    Ok(result)
  });

  // \SIrange[options]{first}{last}{units}
  DefMacro!("\\SIrange OptionalKeyVals:SIX {}{}{}", sub[(kv, first_arg, last_arg, units_arg)] {
    let first = first_arg;
    let last = last_arg;
    let units = units_arg;
    six_begin_processing(kv.as_ref());
    let times = six_get_op_sym(&[("role", Tokenize!("MULOP")), ("meaning", Tokenize!("times"))], six_pin!("number-unit-product"));
    let mode = six_get_choice_sym(six_pin!("range-units"));
    let fnumber = six_format_number(&six_parse_number(&first), 0);
    let lnumber = six_format_number(&six_parse_number(&last), 0);
    six_enable_unit_macros(true);
    let funits = six_process_units(&units);
    let result = if mode == "repeat" {
      let fn_with = six_format_infix(times.clone(), None, None, vec![fnumber, funits.clone()]);
      let ln_with = six_format_infix(times, None, None, vec![lnumber, funits]);
      six_format_range(mode == "brackets", fn_with, ln_with)
    } else {
      let range = six_format_range(mode == "brackets", fnumber, lnumber);
      six_format_infix(times, None, None, vec![range, funits])
    };
    let result = six_wrap(result);
    six_end_processing();
    Ok(result)
  });

  Let!("\\tablenum", "\\num");

  // siunitx v3 \qty[options]{number}{units} — v3 spelling of v2 \SI.
  // Witness 2406.20067, 2407.03167.
  //
  // Only define `\qty` if not already defined. The physics package's
  // `\qty` ([opt]{expr} that wraps expr in delimiters) commonly
  // pre-occupies the name in papers that load both `physics` and
  // `siunitx`. siunitx-after-physics with unconditional Let blindly
  // shadows the physics shape, which causes papers writing
  // `\qty[ 1 + ... ] \rho` to mis-parse as
  // `\SI[opt-list]{value-only}` and fire siunitx number-parse errors.
  // Witness 2305.09755.
  RawTeX!("\\@ifundefined{qty}{\\let\\qty\\SI}{}");
  Let!("\\qtylist", "\\SIlist");
  Let!("\\qtyrange", "\\SIrange");
  Let!("\\qtyproduct", "\\SIlist");

  //======================================================================
  // Table column types S and s — Perl: DefColumnType('S Optional', ...) and
  // 's Optional'. The optional `[<options>]` (e.g. round-precision=1) must
  // be consumed by the column-type reader; otherwise each character of the
  // option string leaks back into the tabular template parser as a separate
  // column letter (driver: 1904.04279 with `S[round-precision=1]`).
  // Perl L1401-1407: cell-prep (begin SI processing for this column's [kv])
  // and the empty end-delimiter.
  DefMacro!("\\lx@si@column@prep OptionalKeyVals:SIX", sub[(kv)] {
    six_begin_processing(kv.as_ref());
    six_enable_unit_macros(true);
    Ok(Tokens!())
  });
  DefPrimitive!("\\lx@si@column@end", {});
  // Perl L1414-1451 `\lx@SI@column@parse` (the `S`, number column): read the
  // cell up to `\lx@si@column@end`, peel leading "surrounding material" —
  // spaces, leading non-symbol control sequences (formatting like `\bfseries`,
  // `\color`), and whole braced groups — into `pre`, parse the rest as a
  // number (NO error on leftover, UNLIKE `\num`), then emit
  // `pre + {\color{…}}?six_wrap(result) + post`. A leading `\color` cancels
  // the column's auto-color (Perl L1429).
  DefMacro!("\\lx@SI@column@parse XUntil:\\lx@si@column@end", sub[args] {
    use latexml_core::token::Catcode;
    let mut tokens: Vec<Token> = args[0].clone().into_tokens_result()?.unlist();
    let doparse = six_get_bool_sym(six_pin!("parse-numbers"));
    let mut color = six_get_tokens_sym(six_pin!("color"));
    let mut pre: Vec<Token> = Vec::new();
    let color_cs = T_CS!("\\color");
    loop {
      match tokens.first().map(|t| t.get_catcode()) {
        // SPACE, or (parsing AND a leading CS that is NOT an
        // input-comparator/protect-token/symbol) → peel into `pre`.
        Some(Catcode::SPACE) => { pre.push(tokens.remove(0)); },
        Some(Catcode::ESCAPE) if doparse
          && !six_token_matches_keys(&tokens[0], &[
            six_pin!("input-comparators"),
            six_pin!("input-protect-tokens"),
            six_pin!("input-symbols")]) => {
          if tokens[0] == color_cs { color = Tokens::new(vec![]); }
          pre.push(tokens.remove(0));
        },
        Some(Catcode::BEGIN) => {
          let mut depth = 0i32;
          let mut i = 0usize;
          while i < tokens.len() {
            match tokens[i].get_catcode() {
              Catcode::BEGIN => depth += 1,
              Catcode::END => { depth -= 1; if depth == 0 { i += 1; break; } },
              _ => {},
            }
            i += 1;
          }
          pre.extend(tokens.drain(0..i));
        },
        _ => break,
      }
    }
    let (result, mut post): (Tokens, Vec<Token>) = if doparse {
      let mut toks = six_apply_mathligatures(tokens);
      let parsed = six_postprocess(six_match_number(&mut toks));
      let res = match parsed {
        Some(num) => six_format_number(&SixParseResult::Parsed(num), 0),
        None => Tokens!(),
      };
      (res, toks)
    } else {
      (Tokens::new(tokens), Vec::new())
    };
    // Perl L1445-1447: color wraps the result (pre … {\color{c} result } … post).
    if !color.is_empty() {
      pre.push(T_BEGIN!());
      pre.push(color_cs);
      pre.push(T_BEGIN!());
      pre.extend(color.unlist());
      pre.push(T_END!());
      post.insert(0, T_END!());
    }
    six_end_processing();
    let mut out: Vec<Token> = pre;
    if !result.is_empty() {
      out.extend(six_wrap(result).unlist());
    }
    out.extend(post);
    Ok(Tokens::new(out))
  });
  // Perl L1454-1485 `\lx@si@column@parse` (the lowercase `s`, UNIT column):
  // peel leading spaces / braced groups into `pre`, then parse the remainder
  // as UNITS (`six_convertUnits`/`six_parse_units`/`six_format_units`, via the
  // shared `six_process_units` used by `\si`) rather than as a number. Color
  // wraps the whole cell (Perl L1479-1481: `{\color{c} pre result post }`).
  DefMacro!("\\lx@si@column@parse XUntil:\\lx@si@column@end", sub[args] {
    use latexml_core::token::Catcode;
    let mut tokens: Vec<Token> = args[0].clone().into_tokens_result()?.unlist();
    let color = six_get_tokens_sym(six_pin!("color"));
    let mut pre: Vec<Token> = Vec::new();
    loop {
      match tokens.first().map(|t| t.get_catcode()) {
        Some(Catcode::SPACE) => { pre.push(tokens.remove(0)); },
        Some(Catcode::BEGIN) => {
          let mut depth = 0i32;
          let mut i = 0usize;
          while i < tokens.len() {
            match tokens[i].get_catcode() {
              Catcode::BEGIN => depth += 1,
              Catcode::END => { depth -= 1; if depth == 0 { i += 1; break; } },
              _ => {},
            }
            i += 1;
          }
          pre.extend(tokens.drain(0..i));
        },
        _ => break,
      }
    }
    // Perl L1472: drop a trailing `\lx@column@trimright` sentinel if present.
    if tokens.last().map(|t| *t == T_CS!("\\lx@column@trimright")).unwrap_or(false) {
      tokens.pop();
    }
    let result = six_process_units(&Tokens::new(tokens));
    let mut post: Vec<Token> = Vec::new();
    let mut pre_out: Vec<Token> = Vec::new();
    // Perl L1479-1481: color wraps EVERYTHING (`{\color{c}` at the FRONT).
    if !color.is_empty() {
      pre_out.push(T_BEGIN!());
      pre_out.push(T_CS!("\\color"));
      pre_out.push(T_BEGIN!());
      pre_out.extend(color.unlist());
      pre_out.push(T_END!());
      post.push(T_END!());
    }
    pre_out.extend(pre);
    six_end_processing();
    let mut out: Vec<Token> = pre_out;
    if !result.is_empty() {
      out.extend(six_wrap(result).unlist());
    }
    out.extend(post);
    Ok(Tokens::new(out))
  });
  // Perl L1379-1399: `S` → number parse, lowercase `s` → unit parse.
  DefColumnType!("S Optional", sub[args] {
    add_si_column(args.first().cloned().and_then(ArgWrap::owned_tokens), "\\lx@SI@column@parse");
  });
  DefColumnType!("s Optional", sub[args] {
    add_si_column(args.first().cloned().and_then(ArgWrap::owned_tokens), "\\lx@si@column@parse");
  });

  //======================================================================
  // Default options
  //======================================================================
  RawTeX!(r#"\sisetup{
  abbreviations,
  binary-units,
  input-product  = x,
  input-quotient = /,
  input-close-uncertainty = ),
  input-complex-roots     = ij,
  input-comparators       = {<=>\approx\ge\geq\gg\le\leq\ll\sim},
  input-decimal-markers   = {.,},
  input-digits            = 0123456789,
  input-exponent-markers  = dDeE,
  input-open-uncertainty  = (,
  input-protect-tokens    = \approx\dots\ge\geq\gg\le\leq\ll\mp\pi\pm\sim,
  input-signs             = +-\mp\pm,
  input-symbols           = \dots\pi,
  input-uncertainty-signs = \pm,
  add-decimal-zero      = true,
  add-integer-zero      = true,
  retain-unity-mantissa = true,
  bracket-numbers           = true,
  close-bracket             = ),
  complex-root-position     = after-number,
  copy-decimal-marker       = false,
  exponent-base             = 10,
  exponent-product          = \times,
  group-digits              = true,
  group-minimum-digits      = 5,
  group-separator           = \,,
  open-bracket              = (,
  output-close-uncertainty  = ),
  output-complex-root       = \ensuremath{\mathrm{i}},
  output-decimal-marker     = .,
  output-open-uncertainty   = (,
  fraction-function = \frac,
  output-product    = \times,
  output-quotient   = /,
  parse-numbers     = true,
  quotient-mode     = symbol,
  prefixes-as-symbols = true,
  bracket-unit-denominator     = true,
  inter-unit-product           = \,,
  per-mode                     = reciprocal,
  per-symbol                   = /,
  qualifier-mode               = subscript,
  number-unit-product = \,,
  product-units       = repeat,
  list-final-separator = { and },
  list-pair-separator  = { and },
  list-separator       = {, },
  list-units           = repeat,
  range-phrase = { to },
  range-units  = repeat,
  bracket-numbers,
  parse-numbers,
  parse-units,
  product-units,
  number-angle-product=,
  arc-separator=
}"#);

  //======================================================================
  // Unit declarations
  //======================================================================
  RawTeX!(r#"
\DeclareSIUnit \kilogram { \kilo \gram }
\DeclareSIUnit \meter    { m }
\DeclareSIUnit \metre    { \meter }
\DeclareSIUnit \mole     { mol }
\DeclareSIUnit \second   { s }
\DeclareSIUnit \ampere   { A }
\DeclareSIUnit \kelvin   { K }
\DeclareSIUnit \candela  { cd }
\DeclareSIUnit \gram { g }
\DeclareSIPrefix \yocto { y } { -24 }
\DeclareSIPrefix \zepto { z } { -21 }
\DeclareSIPrefix \atto  { a } { -18 }
\DeclareSIPrefix \femto { f } { -15 }
\DeclareSIPrefix \pico  { p } { -12 }
\DeclareSIPrefix \nano  { n } { -9 }
\DeclareSIPrefix \micro { \SIUnitSymbolMicro } { -6 }
\DeclareSIPrefix \milli { m } { -3 }
\DeclareSIPrefix \centi { c } { -2 }
\DeclareSIPrefix \deci  { d } { -1 }
\DeclareSIPrefix \deca  { da } { 1 }
\DeclareSIPrefix \deka  { da } { 1 }
\DeclareSIPrefix \hecto { h }  { 2 }
\DeclareSIPrefix \kilo  { k }  { 3 }
\DeclareSIPrefix \mega  { M }  { 6 }
\DeclareSIPrefix \giga  { G }  { 9 }
\DeclareSIPrefix \tera  { T }  { 12 }
\DeclareSIPrefix \peta  { P }  { 15 }
\DeclareSIPrefix \exa   { E }  { 18 }
\DeclareSIPrefix \zetta { Z }  { 21 }
\DeclareSIPrefix \yotta { Y }  { 24 }
\DeclareSIUnit \becquerel     { Bq }
\DeclareSIUnit \celsius       { \SIUnitSymbolCelsius }
\DeclareSIUnit \degreeCelsius { \SIUnitSymbolCelsius }
\DeclareSIUnit \coulomb       { C }
\DeclareSIUnit \farad         { F }
\DeclareSIUnit \gray          { Gy }
\DeclareSIUnit \hertz         { Hz }
\DeclareSIUnit \henry         { H }
\DeclareSIUnit \joule         { J }
\DeclareSIUnit \katal         { kat }
\DeclareSIUnit \lumen         { lm }
\DeclareSIUnit \lux           { lx }
\DeclareSIUnit \newton    { N }
\DeclareSIUnit \ohm       { \SIUnitSymbolOhm }
\DeclareSIUnit \pascal    { Pa }
\DeclareSIUnit \radian    { rad }
\DeclareSIUnit \siemens   { S }
\DeclareSIUnit \sievert   { Sv }
\DeclareSIUnit \steradian { sr }
\DeclareSIUnit \tesla     { T }
\DeclareSIUnit \volt      { V }
\DeclareSIUnit \watt      { W }
\DeclareSIUnit \weber     { Wb }
\DeclareSIUnit \arcmin { \arcminute }
\DeclareSIUnit \arcminute { \SIUnitSymbolArcminute }
\DeclareSIUnit \arcsecond { \SIUnitSymbolArcsecond }
\DeclareSIUnit \day { d }
\DeclareSIUnit \degree { \SIUnitSymbolDegree }
\DeclareSIUnit \hectare { ha }
\DeclareSIUnit \hour    { h }
\DeclareSIUnit \litre   { l }
\DeclareSIUnit \liter   { L }
\DeclareSIUnit \minute  { min }
\DeclareSIUnit \percent { \% }
\DeclareSIUnit \tonne   { t }
\DeclareSIUnit \astronomicalunit { ua }
\DeclareSIUnit \atomicmassunit   { u }
\DeclareSIUnit \electronvolt     { eV }
\DeclareSIUnit \dalton           { Da }
\DeclareSIUnit \clight { c }
\DeclareSIUnit \electronmass { m }
\DeclareSIUnit \planckbar { \hbar }
\DeclareSIUnit \elementarycharge { e }
\DeclareSIUnit \bohr { a }
\DeclareSIUnit \hartree { E }
\DeclareSIUnit \angstrom     { \SIUnitSymbolAngstrom }
\DeclareSIUnit \bar          { bar }
\DeclareSIUnit \barn         { b }
\DeclareSIUnit \bel          { B }
\DeclareSIUnit \decibel      { \deci \bel }
\DeclareSIUnit \knot         { kn }
\DeclareSIUnit \mmHg         { mmHg }
\DeclareSIUnit \torr         { Torr }
\DeclareSIUnit \nauticalmile { M }
\DeclareSIUnit \neper        { Np }
\DeclareSIPrePower  \square  { 2 }
\DeclareSIPostPower \squared { 2 }
\DeclareSIPrePower  \cubic   { 3 }
\DeclareSIPostPower \cubed   { 3 }
\DeclareSIPrePower  \Square  { 2 }
\DeclareSIPrePower  \ssquare { 2 }
"#);

  //======================================================================
  // Abbreviation units (from siunitx-abbreviations.cfg)
  RawTeX!(r#"
\DeclareSIUnit \A  { \ampere }
\DeclareSIUnit \pA { \pico \ampere }
\DeclareSIUnit \nA { \nano \ampere }
\DeclareSIUnit \uA { \micro \ampere }
\DeclareSIUnit \mA { \milli \ampere }
\DeclareSIUnit \kA { \kilo \ampere }
\DeclareSIUnit \Hz  { \hertz }
\DeclareSIUnit \mHz { \milli \hertz }
\DeclareSIUnit \kHz { \kilo \hertz }
\DeclareSIUnit \MHz { \mega \hertz }
\DeclareSIUnit \GHz { \giga \hertz }
\DeclareSIUnit \THz { \tera \hertz }
\DeclareSIUnit \mol  { \mole }
\DeclareSIUnit \fmol { \femto \mole }
\DeclareSIUnit \pmol { \pico \mole }
\DeclareSIUnit \nmol { \nano \mole }
\DeclareSIUnit \umol { \micro \mole }
\DeclareSIUnit \mmol { \milli \mole }
\DeclareSIUnit \kmol { \kilo \mole }
\DeclareSIUnit \V  { \volt }
\DeclareSIUnit \pV { \pico \volt }
\DeclareSIUnit \nV { \nano \volt }
\DeclareSIUnit \uV { \micro \volt }
\DeclareSIUnit \mV { \milli \volt }
\DeclareSIUnit \kV { \kilo \volt }
\DeclareSIUnit \hl { \hecto \litre }
\DeclareSIUnit \l  { \litre }
\DeclareSIUnit \ml { \milli \litre }
\DeclareSIUnit \ul { \micro \litre }
\DeclareSIUnit \hL { \hecto \liter }
\DeclareSIUnit \L  { \liter }
\DeclareSIUnit \mL { \milli \liter }
\DeclareSIUnit \uL { \micro \liter }
\DeclareSIUnit \fg  { \femto \gram }
\DeclareSIUnit \pg  { \pico \gram }
\DeclareSIUnit \ng  { \nano \gram }
\DeclareSIUnit \ug  { \micro \gram }
\DeclareSIUnit \mg  { \milli \gram }
\DeclareSIUnit \g   { \gram }
\DeclareSIUnit \kg  { \kilo \gram }
\DeclareSIUnit \amu { \atomicmassunit }
\DeclareSIUnit \W   { \watt }
\DeclareSIUnit \uW  { \micro \watt }
\DeclareSIUnit \mW  { \milli \watt }
\DeclareSIUnit \kW  { \kilo \watt }
\DeclareSIUnit \MW  { \mega \watt }
\DeclareSIUnit \GW  { \giga \watt }
\DeclareSIUnit \J   { \joule }
\DeclareSIUnit \uJ  { \micro \joule }
\DeclareSIUnit \mJ  { \milli \joule }
\DeclareSIUnit \kJ  { \kilo \joule }
\DeclareSIUnit \eV  { \electronvolt }
\DeclareSIUnit \meV { \milli \electronvolt }
\DeclareSIUnit \keV { \kilo \electronvolt }
\DeclareSIUnit \MeV { \mega \electronvolt }
\DeclareSIUnit \GeV { \giga \electronvolt }
\DeclareSIUnit \TeV { \tera \electronvolt }
\DeclareSIUnit \kWh { \kilo \watt \hour }
\DeclareSIUnit \m  { \metre }
\DeclareSIUnit \pm { \pico \metre }
\DeclareSIUnit \nm { \nano \metre }
\DeclareSIUnit \um { \micro \metre }
\DeclareSIUnit \mm { \milli \metre }
\DeclareSIUnit \cm { \centi \metre }
\DeclareSIUnit \dm { \deci \metre }
\DeclareSIUnit \km { \kilo \metre }
\DeclareSIUnit \K { \kelvin }
\DeclareSIUnit \dB { \deci \bel }
\DeclareSIUnit \F  { \farad }
\DeclareSIUnit \fF { \femto \farad }
\DeclareSIUnit \pF { \pico \farad }
\DeclareSIUnit \N  { \newton }
\DeclareSIUnit \mN { \milli \newton }
\DeclareSIUnit \kN { \kilo \newton }
\DeclareSIUnit \MN { \mega \newton }
\DeclareSIUnit \Pa  { \pascal }
\DeclareSIUnit \kPa { \kilo \pascal }
\DeclareSIUnit \MPa { \mega \pascal }
\DeclareSIUnit \GPa { \giga \pascal }
\DeclareSIUnit \mohm { \milli \ohm }
\DeclareSIUnit \kohm { \kilo \ohm }
\DeclareSIUnit \Mohm { \mega \ohm }
\DeclareSIUnit \s  { \second }
\DeclareSIUnit \as { \atto \second }
\DeclareSIUnit \fs { \femto \second }
\DeclareSIUnit \ps { \pico \second }
\DeclareSIUnit \ns { \nano \second }
\DeclareSIUnit \us { \micro \second }
\DeclareSIUnit \ms { \milli \second }
\DeclareBinaryPrefix \kibi { Ki } { 10 }
\DeclareBinaryPrefix \mebi { Mi } { 20 }
\DeclareBinaryPrefix \gibi { Gi } { 30 }
\DeclareBinaryPrefix \tebi { Ti } { 40 }
\DeclareBinaryPrefix \pebi { Pi } { 50 }
\DeclareBinaryPrefix \exbi { Ei } { 60 }
\DeclareBinaryPrefix \zebi { Zi } { 70 }
\DeclareBinaryPrefix \yobi { Yi } { 80 }
\DeclareSIUnit \bit  { bit }
\DeclareSIUnit \byte { B }
"#);

  // Version-1 compatibility units (siunitx-version-1.cfg). Activated by
  // the `version-1-compatibility` package option. The v1.cfg file
  // declares ~65 v1-only unit aliases (BAR, Day, Gray, atomicmass,
  // arcmin, are, curie, gal, millibar, rad, rem, roentgen, micA, micg,
  // picm, micm, Sec, mics, cmc, cubiccentimetre, cubicdecimetre, etc.).
  // We always declare them — the names don't conflict with v3's
  // DeclareSIUnit set above, so the cost is just a few extra entries.
  // Driver: 2007.02084 \cubiccentimetre R=1 → R=0.
  RawTeX!(r#"
\DeclareSIUnit \BAR        { \bar }
\DeclareSIUnit \bbar       { \bar }
\DeclareSIUnit \Day        { \day }
\DeclareSIUnit \dday       { \day }
\DeclareSIUnit \Gray       { \gray }
\DeclareSIUnit \ggray      { \gray }
\DeclareSIUnit \atomicmass { \atomicmassunit }
\DeclareSIUnit \arcmin     { \arcminute }
\DeclareSIUnit \arcsec     { \arcsecond }
\DeclareSIUnit \are        { a }
\DeclareSIUnit \curie      { Ci }
\DeclareSIUnit \gal        { Gal }
\DeclareSIUnit \millibar   { \milli \bar }
\DeclareSIUnit \rad        { rad }
\DeclareSIUnit \rem        { rem }
\DeclareSIUnit \roentgen   { R }
\DeclareSIUnit \micA       { \micro \ampere }
\DeclareSIUnit \micmol     { \micro \mole   }
\DeclareSIUnit \micl       { \micro \litre  }
\DeclareSIUnit \micL       { \micro \liter  }
\DeclareSIUnit \nanog      { \nano  \gram   }
\DeclareSIUnit \micg       { \micro \gram   }
\DeclareSIUnit \picm       { \pico  \metre  }
\DeclareSIUnit \micm       { \micro \metre  }
\DeclareSIUnit \Sec        { \second }
\DeclareSIUnit \mics       { \micro \second }
\DeclareSIUnit \cmc        { \centi \metre \cubed }
\DeclareSIUnit \dmc        { \deci  \metre \cubed }
\DeclareSIUnit \cms        { \centi \metre \squared }
\DeclareSIUnit \centimetrecubed   { \centi \metre \cubed }
\DeclareSIUnit \centimetresquared { \centi \metre \squared }
\DeclareSIUnit \cubiccentimetre { \centi \metre \cubed }
\DeclareSIUnit \cubicdecimetre  { \deci \metre \cubed }
\DeclareSIUnit \squarecentimetre { \centi \metre \squared }
\DeclareSIUnit \squaremetre      { \metre \squared }
\DeclareSIUnit \squarekilometre  { \kilo \metre \squared }
\DeclareSIUnit \molar      { \mole \per \cubic \deci \metre }
\DeclareSIUnit \nb         { \nano \barn }
\DeclareSIUnit \pb         { \pico \barn }
\DeclareSIUnit \fb         { \femto \barn }
\DeclareSIUnit \ab         { \atto \barn }
\DeclareSIUnit \zb         { \zepto \barn }
\DeclareSIUnit \yb         { \yocto \barn }
% hep / particle-physics units — Perl siunitx.sty.ltxml L1795-1812.
% Activated by `\usepackage[alsoload=hep]{siunitx}` or
% `version-1-compatibility`. Driver: 1607.04783 (stage 19 RUST-
% REGRESSION: `\eVperc`/`\gauss`/`\nanobarn` undefined).
\DeclareSIUnit \gon        { gon }
\DeclareSIUnit \clight     { \text { \ensuremath { c } } }
\DeclareSIUnit \micron     { \micro \metre }
\DeclareSIUnit \mrad       { \milli \rad }
\DeclareSIUnit \gauss      { G }
\DeclareSIUnit \eVperc     { \eV \per \clight }
\DeclareSIUnit \nanobarn   { \nano \barn }
\DeclareSIUnit \picobarn   { \pico \barn }
\DeclareSIUnit \femtobarn  { \femto \barn }
\DeclareSIUnit \attobarn   { \atto \barn }
\DeclareSIUnit \zeptobarn  { \zepto \barn }
\DeclareSIUnit \yoctobarn  { \yocto \barn }
"#);

  DefMacro!("\\highlight{}", "#1");
  DefMacro!("\\of{}", "(#1)");
  DefMacro!("\\tothe{}", "\\textsuperscript{#1}");
  DefMacro!("\\raiseto{}", "\\textsuperscript{#1}");
  DefMacro!("\\per", "/");

  // Restore `~` to its normal ACTIVE (13) catcode after the expl3-backed
  // load. siunitx is an expl3 package; `\RequirePackage{expl3}` (above) plus
  // the `\ExplSyntaxOn` regime in our RawTeX blocks leave `~` at catcode 10
  // (SPACE), because Rust's `\ExplSyntaxOff` does not fully restore it (the
  // known partial-expl3-kernel gap). Real siunitx's `\ExplSyntaxOff` puts
  // `~` back to ACTIVE; xparse_sty.rs already does this same restore for its
  // own load. Without it, a LATER `\usepackage[english]{babel}` runs
  // `\initiate@active@char{~}` with `~` at catcode 10 → an
  // `expected:<relationaltoken>` cascade (Rust 86 / Perl 0 on 2204.05282;
  // minimal repro: `\usepackage{siunitx}\usepackage[english]{babel}`). Order-
  // sensitive: babel-before-siunitx was already fine. `~` is not part of the
  // expl3 LETTER set (unlike `_`/`:`), so this restore does not touch the
  // glossary-sensitive codepoint path.
  assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global));
});
