//! l3fp-parse's grammar over a token list (expl3-code.tex:16502-17855): the operand reader, the infix table with its
//! precedences, juxtaposition, the comparison chains, `?:`. It reads in the macro parser's order and expands what the
//! macro parser would at the moment it would, but only what is certainly pure.
//!
//! **What may expand.** The raw parser f-expands its input as it goes (`\__fp_parse_expand:w`: an argument read, which
//! skips spaces, then `` `^^@ ``'s `get_x_token`, which expands until an unexpandable token and swallows one space). The
//! native path must have no side effect until it has its answer, nothing reported above all, because on any doubt the raw
//! macro runs over the untouched input. So [`Parser::a`] substitutes, onto its own stack and never onto the input, only
//! what certainly changes no state: a macro with a token-list body; `\number`, `\the`, `\dim_to_fp:n`, `\fp_to_decimal:n` and `\fp_use:N` of
//! a plain register, one dimension or another fp expression; and `\csname` of a name that already means something. Anything
//! else TeX would expand (`\csname` of a new name, which defines `\relax`; a conditional; `\input`; a binding's closure;
//! an active character) is a [`Decline`]. The result is the same token sequence the raw parser reads, so the answer is the same.

use latexml_core::{
  common::store::Stored,
  definition::{
    expandable::{Expandable, SpecialCall},
    register::Register,
  },
  gullet::do_expand,
};

use super::{
  func::{self, SinKind, Unit},
  num::{self, Decline, Fp, Mode, ONE, POW10, R, Sign, Tag, ZERO},
};
use crate::prelude::*;

/// Macro expansions one call may substitute before it hands over to the raw macro.
const EXPANSION_BUDGET: u32 = 4000;
/// How many digits an exponent may have before `\int_value:w` would object.
const MAX_EXPONENT_DIGITS: u32 = 9;
/// Nested doors and `\fp_to_decimal:n` calls.
const MAX_NESTING: u8 = 16;
/// Recursion of [`Parser::operand`] and [`Parser::one`] together, and a function's commas, past which the expression is the
/// macro's: about 200 of `(`, 400 of `^`. Well below what the macro's own run takes without reaching the recursion limit,
/// whose Fatal then stays its own (`1^1^…` at 2,000 terms, `max(1,1,…)` at 2,000 arguments); an input of any length ends here
/// or in a loop, so no input can overflow the stack: at this limit 172 KB in a release build, 1.15 MB in a debug one, and
/// each level is [`stack_guard::maybe_grow`]n.
const MAX_DEPTH: u32 = 400;

/// `\c__fp_prec_..._int`
mod prec {
  pub const FUNC: i32 = 16;
  pub const HATII: i32 = 14;
  pub const HAT: i32 = 13;
  pub const NOT: i32 = 12;
  pub const JUXT: i32 = 11;
  pub const TIMES: i32 = 10;
  pub const PLUS: i32 = 9;
  pub const COMP: i32 = 7;
  pub const AND: i32 = 6;
  pub const OR: i32 = 5;
  pub const QUEST: i32 = 4;
  pub const COLON: i32 = 3;
  pub const COMMA: i32 = 2;
  pub const TUPLE: i32 = 1;
  pub const END: i32 = 0;
}

#[derive(Clone, Copy)]
enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  Pow,
  And,
  Or,
}

/// The infix macro `\__fp_parse_infix...:N` the next token selected.
#[derive(Clone, Copy)]
enum Infix {
  /// `\__fp_parse_infix_end:N`
  End,
  /// `)`
  RParen,
  /// `,`
  Comma,
  /// a binary operator: its macro's `#3` (compared with the precedence) and `#4` (its operand's)
  Bin(BinOp, i32, i32),
  /// `?`
  Quest,
  /// `:`
  Colon,
  /// `\__fp_parse_compare:NNNNNNN`: the flags so far (`<`, `=`, `>`, `?`), the value a symbol sets one to, the first
  /// symbol if one is read, and whether `\__fp_parse_excl_error:` still applies (`!`)
  Compare {
    flags: [bool; 4],
    set:   bool,
    first: Option<u8>,
    excl:  bool,
  },
}

/// An operand: one number, or a function's several arguments (only a `(` at function precedence yields them).
enum Val {
  One(Fp),
  Many(Vec<Fp>),
}

impl Val {
  fn one(self) -> R<Fp> {
    match self {
      Val::One(x) => Ok(x),
      Val::Many(_) => Err(Decline("a function's arguments used as a number")),
    }
  }

  fn args(self) -> Vec<Fp> {
    match self {
      Val::One(x) => vec![x],
      Val::Many(v) => v,
    }
  }
}

/// What a control sequence is to the parser.
enum Class {
  /// a macro the native path can substitute
  Macro(Rc<Expandable>),
  /// the format's `\__fp_parse:n` (`\fp_add:Nn`, `\fp_abs:n`, `\fp_sign:n` put it inside an expression)
  Door,
  /// `\s__fp`
  FpStart,
  /// a number or dimension register without arguments
  Register(Rc<Register>),
  /// `\number`, `\int_value:w`
  Number,
  /// `\the`, `\tex_the:D`, `\int_use:N`, `\dim_use:N`
  The,
  /// `\csname`, `\cs:w`, `\use:c`'s
  Csname,
  /// `\dim_to_fp:n`
  DimToFp,
  /// `\fp_to_decimal:n`, `\fp_eval:n`
  ToDecimal,
  /// `\fp_to_decimal:N`, `\fp_use:N`
  ToDecimalOfVariable,
}

/// A parser's state: the input as a stack, top last.
pub(super) struct Parser {
  stack:          Vec<Token>,
  budget:         u32,
  pending:        Option<Infix>,
  nesting:        u8,
  /// [`MAX_DEPTH`]'s count, across nested parsers
  depth:          u32,
  /// the token a decline stopped at, for the census
  pub(super) at:  Option<Token>,
  /// the macro last substituted, for the census
  pub(super) via: Option<Token>,
}

/// The ASCII character of a character token the parser reads by its text, as `\token_to_str:N` does: a letter, an
/// other, or the usual `^` and `&`, whatever the code. Anything else declines.
#[inline]
fn char_of(t: Token) -> R<u8> {
  match t.code {
    Catcode::LETTER | Catcode::OTHER | Catcode::SUPER | Catcode::ALIGN => t.with_str(|s| {
      let b = s.as_bytes();
      if b.len() == 1 {
        Ok(b[0])
      } else {
        Err(Decline("a non-ASCII character"))
      }
    }),
    Catcode::ACTIVE => Err(Decline("an active character")),
    Catcode::BEGIN | Catcode::END => Err(Decline("a brace group")),
    _ => Err(Decline("a character of an unusual catcode")),
  }
}

#[inline]
fn is_letter(c: u8) -> bool { c.is_ascii_alphabetic() }

impl Parser {
  /// A parser over the door's expression as the input's pushback holds it, its first token last.
  pub(super) fn over_input(input: &[Token]) -> Parser {
    let mut stack = Vec::with_capacity(input.len() + 32);
    stack.extend_from_slice(input);
    Parser {
      stack,
      budget: EXPANSION_BUDGET,
      pending: None,
      nesting: 0,
      depth: 0,
      at: None,
      via: None,
    }
  }

  /// A parser over an argument this one read, under this one's limits.
  fn nested(&self, arg: &[Token]) -> Parser {
    let mut stack = Vec::with_capacity(arg.len() + 32);
    stack.extend(arg.iter().rev().copied());
    Parser {
      stack,
      budget: self.budget,
      pending: None,
      nesting: self.nesting + 1,
      depth: self.depth,
      at: None,
      via: None,
    }
  }

  /// The whole expression.
  pub(super) fn parse(&mut self) -> R<Fp> {
    let v = self.operand(prec::END)?;
    match self.pending {
      Some(Infix::End) => v.one(),
      _ => Err(Decline("an expression that does not end")),
    }
  }

  fn decline_at<T>(&mut self, t: Token, why: &'static str) -> R<T> {
    self.at = Some(t);
    Err(Decline(why))
  }

  /// What control sequence `t` is, if the native path can read it at all. `\s__fp` by its name: the name's meaning is
  /// what it was at the install, or nothing is answered (`state::watched_meaning_changed`).
  fn classify(&mut self, t: Token) -> R<Class> {
    if t == T_CS!("\\s__fp") {
      return Ok(Class::FpStart);
    }
    match lookup_meaning(&t) {
      Some(Stored::Expandable(e)) => {
        // the engine's own and the format's: a document's macro of the name is a macro like any other
        if matches!(e.expansion, Some(ExpansionBody::Closure(_))) {
          if e.cs == T_CS!("\\number") {
            return Ok(Class::Number);
          }
          if e.cs == T_CS!("\\the") {
            return Ok(Class::The);
          }
          if e.cs == T_CS!("\\csname") {
            return Ok(Class::Csname);
          }
        } else if e.origin.is_latexml_owned() {
          if e.cs == T_CS!("\\__fp_parse:n") {
            return Ok(Class::Door);
          }
          if e.cs == T_CS!("\\dim_to_fp:n") {
            return Ok(Class::DimToFp);
          }
          if e.cs == T_CS!("\\fp_to_decimal:n") {
            return Ok(Class::ToDecimal);
          }
          if e.cs == T_CS!("\\fp_to_decimal:N") {
            return Ok(Class::ToDecimalOfVariable);
          }
        }
        if matches!(e.expansion, Some(ExpansionBody::Closure(_))) {
          return self.decline_at(t, "an expandable with a native body");
        }
        if e.is_protected
          || e.is_outer
          || !matches!(e.special_call, SpecialCall::None)
          || e.peeks_by_futurelet
        {
          return self.decline_at(t, "a macro with special expansion");
        }
        Ok(Class::Macro(e))
      },
      Some(Stored::Register(r)) => Ok(Class::Register(r)),
      Some(Stored::Conditional(_)) => self.decline_at(t, "a conditional"),
      Some(Stored::Primitive(_)) => self.decline_at(t, "a primitive"),
      Some(Stored::Token(_)) => self.decline_at(t, "a token let to a token"),
      None => self.decline_at(t, "an undefined control sequence"),
      _ => self.decline_at(t, "another definition"),
    }
  }

  /// Is the control sequence on top expandable? Expands it if so; false for anything unexpandable.
  fn expand_top(&mut self) -> R<bool> {
    let Some(&t) = self.stack.last() else {
      return Ok(false);
    };
    if t.code != Catcode::CS {
      if t.code == Catcode::ACTIVE {
        return self.decline_at(t, "an active character");
      }
      return Ok(false);
    }
    match self.classify(t)? {
      Class::Macro(e) => {
        self.stack.pop();
        self.substitute(t, &e)?;
        Ok(true)
      },
      Class::Door => {
        self.stack.pop();
        let x = self.read_door()?;
        let mut form = Vec::new();
        super::fp_tokens(x, &mut form);
        self.stack.extend(form.iter().rev().copied());
        Ok(true)
      },
      Class::Number => {
        self.stack.pop();
        let value = self.read_number()?;
        let mut digits = Vec::with_capacity(12);
        super::other_digits(&mut digits, value, 0);
        self.stack.extend(digits.iter().rev().copied());
        Ok(true)
      },
      Class::The => {
        self.stack.pop();
        let text = self.read_the(t)?;
        self.stack.extend(text.iter().rev().copied());
        Ok(true)
      },
      Class::Csname => {
        self.stack.pop();
        let cs = self.read_csname(t)?;
        self.stack.push(cs);
        Ok(true)
      },
      Class::DimToFp => {
        self.stack.pop();
        let form = self.read_dim_to_fp()?;
        self.stack.extend(form.iter().rev().copied());
        Ok(true)
      },
      Class::ToDecimal => {
        self.stack.pop();
        let text = self.read_to_decimal()?;
        self.stack.extend(text.iter().rev().copied());
        Ok(true)
      },
      Class::ToDecimalOfVariable => {
        self.stack.pop();
        let text = self.read_to_decimal_of_variable(t)?;
        self.stack.extend(text.iter().rev().copied());
        Ok(true)
      },
      Class::FpStart | Class::Register(_) => Ok(false),
    }
  }

  /// `\the` of a plain number or dimension register: what it prints, in characters of catcode 12.
  fn read_the(&mut self, the: Token) -> R<Vec<Token>> {
    self.expand_macros()?;
    self.skip_spaces();
    let Some(t) = self.stack.pop() else {
      return Err(Decline("a the past the expression's end"));
    };
    match lookup_meaning(&t) {
      Some(Stored::Register(r))
        if r.parameters.is_none() && r.getter.is_none() && r.mathglyph.is_none() =>
      {
        let value = match (r.register_type, r.value_of(Vec::new())) {
          (RegisterType::Number | RegisterType::CharDef | RegisterType::Dimension, Some(v)) => {
            v.to_string()
          },
          _ => return self.decline_at(t, "a the of another register"),
        };
        Ok(Explode!(value))
      },
      _ => self.decline_at(the, "a the of something else"),
    }
  }

  /// `\csname` of a name that already means something: no `\relax` is defined, so nothing changes. Any other name is the
  /// raw macro's, which defines it. The name's characters are read with macros substituted, as the scan would.
  fn read_csname(&mut self, csname: Token) -> R<Token> {
    let mut name = String::from("\\");
    loop {
      self.expand_macros()?;
      let Some(t) = self.stack.pop() else {
        return Err(Decline("a csname past the expression's end"));
      };
      if t.code == Catcode::CS {
        if t == T_CS!("\\cs_end:") || t == T_CS!("\\endcsname") {
          break;
        }
        return self.decline_at(t, "a control sequence in a csname");
      }
      if matches!(t.code, Catcode::BEGIN | Catcode::END | Catcode::ACTIVE) {
        return self.decline_at(csname, "a csname of an unusual name");
      }
      t.with_str(|s| name.push_str(s));
    }
    let cs = T_CS!(&name);
    if lookup_meaning(&cs).is_none() {
      return self.decline_at(csname, "a csname that defines its name");
    }
    Ok(cs)
  }

  /// `\number`'s operand where nothing can go wrong: a `\dimexpr` or `\numexpr` of one number (`\__dim_ratio:n`:
  /// `\int_value:w \__dim_eval:w (#1) \__dim_eval_end:`), a number register, or digits. The value of a dimension comes
  /// from the engine's own reader over a text checked to be `<decimal><unit>`, since that reader's arithmetic is what the
  /// raw macro gets; [`check_dimension_text`] keeps it below `\maxdimen`, the one thing it would report.
  fn read_number(&mut self) -> R<i64> {
    self.expand_macros()?;
    self.skip_spaces();
    let Some(t) = self.stack.pop() else {
      return Err(Decline("a number past the expression's end"));
    };
    if t.code != Catcode::CS {
      return self.decline_at(t, "a number from characters");
    }
    match lookup_meaning(&t) {
      Some(Stored::Register(r)) if r.cs == T_CS!("\\dimexpr") => {
        let text = self.expression_text()?;
        let text = match text.as_slice() {
          [open, inner @ .., close] if text_is(*open, "(") && text_is(*close, ")") => {
            inner.to_vec()
          },
          _ => text,
        };
        if let [register] = text.as_slice()
          && register.code == Catcode::CS
        {
          // checked by `expression_text`: a plain dimension register
          return self.plain_dimension_value(*register);
        }
        self.dimension_text_value(text, T_CS!("\\dimexpr"))
      },
      Some(Stored::Register(r)) if r.parameters.is_none() && r.getter.is_none() => {
        match (r.register_type, r.value_of(Vec::new())) {
          (RegisterType::Number | RegisterType::CharDef, Some(RegisterValue::Number(n))) => Ok(n.0),
          (RegisterType::Dimension, Some(RegisterValue::Dimension(d))) => Ok(d.0),
          _ => self.decline_at(t, "a register's value for number"),
        }
      },
      _ => self.decline_at(t, "another number for number"),
    }
  }

  /// A register [`is_plain_dimension_register`] holds: what `\dimexpr` makes of it, a glue's natural width included.
  fn plain_dimension_value(&mut self, register: Token) -> R<i64> {
    match lookup_meaning(&register) {
      Some(Stored::Register(r)) => match r.value_of(Vec::new()) {
        Some(RegisterValue::Dimension(d)) => Ok(d.0),
        Some(RegisterValue::Glue(g)) => Ok(g.value_of()),
        _ => self.decline_at(register, "a dimension register without a dimension"),
      },
      _ => self.decline_at(register, "a control sequence in a dimexpr"),
    }
  }

  /// `\number <primitive> <text> \relax` for a text checked to be one dimension, by the engine's own reader.
  fn dimension_text_value(&mut self, text: Vec<Token>, primitive: Token) -> R<i64> {
    check_dimension_text(&text)?;
    let mut call = vec![T_CS!("\\number"), primitive];
    call.extend(text);
    call.push(T_CS!("\\relax"));
    let value =
      do_expand(Tokens::new(call)).map_err(|_| Decline("a dimension the engine refused"))?;
    value
      .to_string()
      .parse::<i64>()
      .map_err(|_| Decline("a dimension's value"))
  }

  /// `\dim_to_fp:n`: `\__fp_from_dim_test:ww 0, \int_value:w \tex_glueexpr:D #1 ;`, for one dimension or one register. As
  /// its form, for the operand reader.
  fn read_dim_to_fp(&mut self) -> R<Vec<Token>> {
    let arg = self.read_argument()?;
    self.stack.push(T_CS!("\\relax"));
    self.stack.extend(arg.iter().rev().copied());
    let text = self.expression_text()?;
    let sp = if let [register] = text.as_slice()
      && register.code == Catcode::CS
    {
      self.plain_dimension_value(*register)?
    } else {
      self.dimension_text_value(text, T_CS!("\\glueexpr"))?
    };
    let mut form = Vec::new();
    super::fp_tokens(dimension_to_fp(sp, 0)?, &mut form);
    Ok(form)
  }

  /// A door's argument, read and answered as the macro's own run answers it where it stands.
  fn read_door(&mut self) -> R<Fp> {
    let arg = self.read_argument()?;
    if self.nesting >= MAX_NESTING {
      return Err(Decline("deep nesting"));
    }
    let mut inner = self.nested(&arg);
    let x = inner.parse().inspect_err(|_| {
      self.at = inner.at;
      self.via = inner.via;
    })?;
    self.budget = inner.budget;
    Ok(x)
  }

  /// `\fp_to_decimal:n`, `\fp_eval:n`: its argument's value in decimal digits, what the operand reader then reads.
  fn read_to_decimal(&mut self) -> R<Vec<Token>> {
    let x = self.read_door()?;
    Ok(Explode!(super::to_decimal(x)?))
  }

  /// `\fp_to_decimal:N`, `\fp_use:N`: a variable holding a floating point number.
  fn read_to_decimal_of_variable(&mut self, t: Token) -> R<Vec<Token>> {
    let arg = self.read_argument()?;
    let [var] = arg.as_slice() else {
      return self.decline_at(t, "a fp_use of something else");
    };
    let holds_fp = matches!(lookup_meaning(var), Some(Stored::Expandable(e))
      if matches!(&e.expansion, Some(ExpansionBody::Tokens(b)) if b.unlist_ref().first() == Some(&T_CS!("\\s__fp"))));
    if !holds_fp {
      return self.decline_at(*var, "a fp_use of something else");
    }
    let x = self.nested(&arg).parse()?;
    Ok(Explode!(super::to_decimal(x)?))
  }

  /// Every macro on top substituted.
  fn expand_macros(&mut self) -> R<()> {
    loop {
      let Some(&t) = self.stack.last() else {
        return Ok(());
      };
      if t.code != Catcode::CS {
        return Ok(());
      }
      match self.classify_quiet(t) {
        Some(Class::Macro(e)) => {
          self.stack.pop();
          self.substitute(t, &e)?;
        },
        _ => return Ok(()),
      }
    }
  }

  fn classify_quiet(&mut self, t: Token) -> Option<Class> {
    let at = self.at;
    let c = self.classify(t).ok();
    self.at = at;
    c
  }

  /// The expression after a `\dimexpr`, up to its `\relax` (`\__dim_eval_end:`), macros substituted as the scan would.
  fn expression_text(&mut self) -> R<Vec<Token>> {
    let mut out = Vec::with_capacity(16);
    loop {
      self.expand_macros()?;
      let Some(t) = self.stack.pop() else {
        return Err(Decline("an expression past the expression's end"));
      };
      if t.code == Catcode::CS {
        if t == T_CS!("\\__dim_eval_end:") || t == T_CS!("\\relax") {
          return Ok(out);
        }
        if !is_plain_dimension_register(&t) {
          return self.decline_at(t, "a control sequence in a dimexpr");
        }
      }
      out.push(t);
    }
  }

  /// A macro's expansion onto the stack: its arguments read and substituted as `\__fp_parse:n`'s expansion would.
  fn substitute(&mut self, t: Token, e: &Rc<Expandable>) -> R<()> {
    self.budget = self
      .budget
      .checked_sub(1)
      .ok_or(Decline("too many expansions"))?;
    self.via = Some(t);
    // What the watches of `Expandable::watched_call` see of a call. The document-call watch sees none here: it reads an
    // empty pushback, and the door's marks are on it.
    if definition::origin::setter_watch()
      && (setter_call_is_observed(&t)
        || (e.paramlist.is_some() && (e.origin.is_binding_made() || e.made_in_own_file)))
    {
      return self.decline_at(t, "a macro under the frontmatter's watch");
    }
    let body = match &e.expansion {
      Some(ExpansionBody::Tokens(body)) => body.unlist_ref().as_slice(),
      // an empty body
      _ => &[],
    };
    let Some(params) = &e.paramlist else {
      self.stack.extend(body.iter().rev().copied());
      return Ok(());
    };
    let parameters = params.get_parameters();
    if parameters.iter().any(|p| p.name != pin!("Plain")) {
      return self.decline_at(t, "a macro with delimited parameters");
    }
    let mut args: Vec<Vec<Token>> = Vec::with_capacity(parameters.len());
    for _ in 0..parameters.len() {
      let arg = self.read_argument().inspect_err(|_| self.at = Some(t))?;
      if e.checks_par && arg.iter().any(|x| *x == T_CS!("\\par")) {
        return self.decline_at(t, "a par in a macro's argument");
      }
      args.push(arg);
    }
    for b in body.iter().rev() {
      if b.code == Catcode::ARG {
        let n = b.with_str(|s| s.parse::<usize>().unwrap_or(0));
        let arg = n
          .checked_sub(1)
          .and_then(|i| args.get(i))
          .ok_or(Decline("a parameter out of range"))?;
        self.stack.extend(arg.iter().rev().copied());
      } else {
        self.stack.push(*b);
      }
    }
    Ok(())
  }

  /// An undelimited argument: spaces skipped, one token or a group without its braces. Past the input's end the raw
  /// macro would take its own marks for the argument.
  fn read_argument(&mut self) -> R<Vec<Token>> {
    self.skip_spaces();
    let Some(t) = self.stack.pop() else {
      return Err(Decline("a macro's argument past the expression's end"));
    };
    match t.code {
      Catcode::BEGIN => {
        let mut depth = 1;
        let mut out = Vec::new();
        loop {
          let Some(x) = self.stack.pop() else {
            return Err(Decline("an unbalanced group"));
          };
          match x.code {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                return Ok(out);
              }
            },
            _ => self.refuse_in_argument(x)?,
          }
          out.push(x);
        }
      },
      Catcode::END => Err(Decline("an unbalanced group")),
      _ => {
        self.refuse_in_argument(t)?;
        Ok(vec![t])
      },
    }
  }

  /// What the macro's read of an argument does more than take: an `\\outer` token's report (tex.web §336), and what the
  /// gullet's read does with a comment or a marker.
  #[inline]
  fn refuse_in_argument(&mut self, t: Token) -> R<()> {
    let refused = match t.code {
      Catcode::COMMENT | Catcode::MARKER => Some("a comment or a marker in a macro's argument"),
      Catcode::CS | Catcode::ACTIVE => match lookup_meaning(&t) {
        Some(Stored::Expandable(e)) if e.is_outer => Some("an outer token in a macro's argument"),
        _ => None,
      },
      _ => None,
    };
    match refused {
      Some(why) => self.decline_at(t, why),
      None => Ok(()),
    }
  }

  /// One level of [`Self::operand`] or [`Self::one`]: [`MAX_DEPTH`] levels at most, on a stack known to hold them.
  #[inline]
  fn deeper<T>(&mut self, level: impl FnOnce(&mut Self) -> R<T>) -> R<T> {
    if self.depth >= MAX_DEPTH {
      return Err(Decline("an expression too deep"));
    }
    self.depth += 1;
    let result = stack_guard::maybe_grow(|| level(self));
    self.depth -= 1;
    result
  }

  #[inline]
  fn skip_spaces(&mut self) {
    while self.stack.last().is_some_and(|t| t.code == Catcode::SPACE) {
      self.stack.pop();
    }
  }

  /// `\__fp_parse_expand:w`: the argument's skip of spaces, then `` `^^@ ``'s `get_x_token`: expansion until an
  /// unexpandable token, one space swallowed. A macro behind that space is read unexpanded by the next argument
  /// (`1\x` where `\x` expands to ` \y`), which only the raw macro gets right.
  fn a(&mut self) -> R<()> {
    self.skip_spaces();
    while self.expand_top()? {}
    if self.stack.last().is_some_and(|t| t.code == Catcode::SPACE) {
      self.stack.pop();
      self.skip_spaces();
      if let Some(&t) = self.stack.last()
        && t.code == Catcode::CS
        && !matches!(self.classify(t), Ok(Class::FpStart | Class::Register(_)))
      {
        return self.decline_at(t, "a macro behind an expansion's space");
      }
    }
    Ok(())
  }

  /// `\__fp_parse_operand:Nw`: an operand and the operators binding tighter than `prec`; [`Self::pending`] is left with
  /// the infix that stopped it.
  fn operand(&mut self, prec: i32) -> R<Val> { self.deeper(|p| p.operand_at(prec)) }

  fn operand_at(&mut self, prec: i32) -> R<Val> {
    self.a()?;
    let (mut val, after_paren) = self.one(prec)?;
    if self.pending.is_none() {
      self.pending = Some(self.read_infix(after_paren)?);
    }
    loop {
      let infix = self.pending.take().ok_or(Decline("no infix"))?;
      match infix {
        Infix::End => {
          self.pending = Some(infix);
          return Ok(val);
        },
        Infix::RParen => {
          if prec <= prec::END {
            return Err(Decline("an extra )"));
          }
          self.pending = Some(infix);
          return Ok(val);
        },
        Infix::Comma => {
          if prec < prec::COMMA {
            return Err(Decline("a tuple"));
          }
          self.pending = Some(infix);
          return Ok(val);
        },
        Infix::Bin(op, left, right) => {
          if prec < left {
            let a = val.one()?;
            let b = self.operand(right)?.one()?;
            val = Val::One(apply_binary(op, a, b)?);
          } else {
            self.pending = Some(infix);
            return Ok(val);
          }
        },
        Infix::Quest => {
          if prec < prec::QUEST {
            let cond = val.one()?;
            let yes = self.operand(prec::QUEST)?.one()?;
            if !matches!(self.pending, Some(Infix::Colon)) {
              return Err(Decline("a ? without its :"));
            }
            self.pending = None;
            let no = self.operand(prec::COLON)?.one()?;
            val = Val::One(if cond.is_zero() { no } else { yes });
          } else {
            self.pending = Some(infix);
            return Ok(val);
          }
        },
        Infix::Colon => {
          if prec < prec::QUEST {
            return Err(Decline("a : without its ?"));
          }
          self.pending = Some(infix);
          return Ok(val);
        },
        Infix::Compare { .. } => {
          if prec < prec::COMP {
            val = Val::One(self.compare_chain(val.one()?, infix)?);
          } else {
            self.pending = Some(infix);
            return Ok(val);
          }
        },
      }
    }
  }

  /// `\__fp_parse_compare:NNNNNNN` to `\__fp_parse_apply_compare_aux:NNwN`: a chain `a < b <= c`, each pair decided,
  /// every operand parsed.
  fn compare_chain(&mut self, first: Fp, infix: Infix) -> R<Fp> {
    let mut lhs = first;
    let mut holds = true;
    let mut infix = infix;
    loop {
      let flags = self.compare_flags(infix)?;
      let rhs = self.operand(prec::COMP)?.one()?;
      if holds {
        // \__fp_compare_back_any:ww #8 #2, its nan case absent
        holds = match num::compare(lhs, rhs) {
          -1 => flags[0],
          0 => flags[1],
          _ => flags[2],
        };
      }
      lhs = rhs;
      match self.pending {
        Some(next @ Infix::Compare { .. }) => {
          self.pending = None;
          infix = next;
        },
        _ => return Ok(if holds { ONE } else { ZERO }),
      }
    }
  }

  /// `\__fp_parse_compare_auxi:NNNNNNN`: the symbols `<=>?` after the first, each setting its flag.
  fn compare_flags(&mut self, infix: Infix) -> R<[bool; 4]> {
    let Infix::Compare {
      mut flags,
      set,
      first,
      mut excl,
    } = infix
    else {
      return Err(Decline("not a comparison"));
    };
    let mut symbol = first;
    loop {
      let c = match symbol.take() {
        Some(c) => c,
        None => {
          self.a()?;
          self.skip_spaces();
          let Some(&t) = self.stack.last() else {
            break;
          };
          if t.code == Catcode::CS {
            break;
          }
          let c = char_of(t)?;
          if !matches!(c, b'<' | b'=' | b'>' | b'?') {
            break;
          }
          self.stack.pop();
          c
        },
      };
      excl = false;
      flags[usize::from(c - b'<')] = set;
    }
    if excl {
      return Err(Decline("a ! without its ="));
    }
    Ok(flags)
  }

  /// `\__fp_parse_one:Nw`: the operand, and whether a `)` closed it (`\__fp_parse_infix_after_paren:NN` reads on).
  fn one(&mut self, prec: i32) -> R<(Val, bool)> { self.deeper(|p| p.one_at(prec)) }

  fn one_at(&mut self, prec: i32) -> R<(Val, bool)> {
    self.skip_spaces();
    let Some(t) = self.stack.pop() else {
      return Err(Decline("an empty operand"));
    };
    if t.code == Catcode::CS {
      return match self.classify(t)? {
        Class::FpStart => {
          let x = self.read_fp()?;
          self.a()?;
          Ok((Val::One(x), false))
        },
        Class::Register(r) => Ok((Val::One(self.read_register(t, &r)?), false)),
        // only behind an expansion's space, declined there
        _ => self.decline_at(t, "an unexpanded macro"),
      };
    }
    let c = char_of(t).inspect_err(|_| self.at = Some(t))?;
    match c {
      b'0'..=b'9' => {
        self.stack.push(t);
        Ok((Val::One(self.number()?), false))
      },
      b'.' => {
        self.stack.push(t);
        Ok((Val::One(self.number()?), false))
      },
      _ if is_letter(c) => {
        self.stack.push(t);
        self.word()
      },
      b'+' => {
        // \__fp_parse_prefix_+:Nw is \__fp_parse_one:Nw
        self.a()?;
        self.one(prec)
      },
      b'-' | b'!' => {
        let inner = self
          .operand(if prec::NOT < prec { prec } else { prec::NOT })?
          .one()?;
        let x = if c == b'-' {
          // \__fp_set_sign_o:w 2
          inner.with_sign(inner.sign().flip())
        } else if inner.is_zero() {
          // \__fp_not_o:w
          ONE
        } else {
          ZERO
        };
        Ok((Val::One(x), false))
      },
      b'(' => {
        // \__fp_parse_prefix_(:Nw
        if prec == prec::FUNC {
          let mut args = vec![self.operand(prec::COMMA)?.one()?];
          let depth = self.depth;
          while matches!(self.pending, Some(Infix::Comma)) {
            self.pending = None;
            // (each comma stays open in the macro's own run, `\__fp_parse_apply_comma:NwNwN`: a level of its recursion)
            if self.depth >= MAX_DEPTH {
              return Err(Decline("an expression too deep"));
            }
            self.depth += 1;
            args.push(self.operand(prec::COMMA)?.one()?);
          }
          self.depth = depth;
          if !matches!(self.pending, Some(Infix::RParen)) {
            return Err(Decline("a ( without its )"));
          }
          self.pending = None;
          self.a()?;
          let val = if args.len() == 1 {
            Val::One(args[0])
          } else {
            Val::Many(args)
          };
          Ok((val, true))
        } else {
          let x = self.operand(prec::TUPLE)?.one()?;
          if !matches!(self.pending, Some(Infix::RParen)) {
            return Err(Decline("a ( without its )"));
          }
          self.pending = None;
          self.a()?;
          Ok((Val::One(x), true))
        }
      },
      _ => {
        self.at = Some(t);
        Err(Decline("an unknown symbol"))
      },
    }
  }

  /// `\__fp_parse_infix:NN`, `\__fp_parse_infix_after_paren:NN`, `\__fp_parse_infix_check:NNN`: which infix macro the next
  /// token selects. A letter or a control sequence stays in the input: juxtaposition, tight after a number, a plain
  /// product after a `)`.
  fn read_infix(&mut self, after_paren: bool) -> R<Infix> {
    self.skip_spaces();
    let juxtaposed = if after_paren {
      Infix::Bin(BinOp::Mul, prec::TIMES, prec::TIMES)
    } else {
      Infix::Bin(BinOp::Mul, prec::JUXT, prec::JUXT)
    };
    let Some(&t) = self.stack.last() else {
      return Ok(Infix::End);
    };
    if t.code == Catcode::CS {
      return Ok(juxtaposed);
    }
    let c = char_of(t).inspect_err(|_| self.at = Some(t))?;
    if is_letter(c) {
      return Ok(juxtaposed);
    }
    self.stack.pop();
    self.a()?;
    Ok(match c {
      b'+' => Infix::Bin(BinOp::Add, prec::PLUS, prec::PLUS),
      b'-' => Infix::Bin(BinOp::Sub, prec::PLUS, prec::PLUS),
      b'/' => Infix::Bin(BinOp::Div, prec::TIMES, prec::TIMES),
      b'^' => Infix::Bin(BinOp::Pow, prec::HATII, prec::HAT),
      b'*' => {
        self.skip_spaces();
        if let Some(&n) = self.stack.last()
          && n.code != Catcode::CS
          && char_of(n)? == b'*'
        {
          self.stack.pop();
          Infix::Bin(BinOp::Pow, prec::HATII, prec::HAT)
        } else {
          Infix::Bin(BinOp::Mul, prec::TIMES, prec::TIMES)
        }
      },
      b'(' => {
        self.stack.push(t);
        Infix::Bin(BinOp::Mul, prec::TIMES, prec::TIMES)
      },
      b')' => Infix::RParen,
      b',' => Infix::Comma,
      b'?' => Infix::Quest,
      b':' => Infix::Colon,
      b'&' | b'|' => {
        // `\__fp_parse_infix_&:N`: however many of them
        loop {
          self.skip_spaces();
          match self.stack.last() {
            Some(&n) if n.code != Catcode::CS && char_of(n)? == c => {
              self.stack.pop();
              self.a()?;
            },
            _ => break,
          }
        }
        if c == b'&' {
          Infix::Bin(BinOp::And, prec::AND, prec::AND)
        } else {
          Infix::Bin(BinOp::Or, prec::OR, prec::OR)
        }
      },
      b'<' | b'=' | b'>' => Infix::Compare {
        flags: [false; 4],
        set:   true,
        first: Some(c),
        excl:  false,
      },
      b'!' => Infix::Compare {
        flags: [true; 4],
        set:   false,
        first: None,
        excl:  true,
      },
      _ => {
        self.at = Some(t);
        return Err(Decline("an unknown operator"));
      },
    })
  }

  /// The number's digits, from the stack's top: `\__fp_parse_trim_zeros:N` to `\__fp_parse_round_loop:N`. Correctly
  /// rounded to 16 digits, ties to even, the exponent counted.
  fn number(&mut self) -> R<Fp> {
    let mut mant: u64 = 0;
    let mut kept = 0u32;
    let mut next_digit: Option<u8> = None;
    let mut sticky = false;
    let mut exponent: i64 = 0;
    let mut seen_dot = false;
    loop {
      self.skip_spaces();
      let Some(&t) = self.stack.last() else {
        break;
      };
      if t.code == Catcode::CS {
        break;
      }
      let c = char_of(t).inspect_err(|_| self.at = Some(t))?;
      if c == b'.' && !seen_dot {
        seen_dot = true;
      } else if c.is_ascii_digit() {
        let d = c - b'0';
        if kept == 0 && d == 0 {
          if seen_dot {
            exponent -= 1;
          }
        } else {
          if !seen_dot {
            exponent += 1;
          }
          if kept < 16 {
            mant = mant * 10 + u64::from(d);
            kept += 1;
          } else if next_digit.is_none() {
            next_digit = Some(d);
          } else if d != 0 {
            sticky = true;
          }
        }
      } else {
        break;
      }
      self.stack.pop();
      self.a()?;
    }
    let ten = self.exponent()?;
    if kept == 0 {
      return Ok(ZERO);
    }
    mant *= POW10[(16 - kept) as usize];
    if let Some(d) = next_digit {
      // \__fp_round_s:NNNw
      let d = if sticky && (d == 0 || d == 5) {
        d + 1
      } else {
        d
      };
      if d > 5 || (d == 5 && mant % 2 == 1) {
        mant += 1;
        if mant == POW10[16] {
          mant = POW10[15];
          exponent += 1;
        }
      }
    }
    num::normal(Sign::Pos, exponent + ten, mant)
  }

  /// `\__fp_parse_exponent:N`: `e` or `E`, signs, digits, each read after an expansion. An `e` that a letter follows is
  /// no exponent: the unit `em` or `ex`, in juxtaposition.
  fn exponent(&mut self) -> R<i64> {
    self.skip_spaces();
    let Some(&t) = self.stack.last() else {
      return Ok(0);
    };
    if t.code == Catcode::CS {
      return Ok(0);
    }
    let c = char_of(t).inspect_err(|_| self.at = Some(t))?;
    if c != b'e' && c != b'E' {
      return Ok(0);
    }
    self.stack.pop();
    self.a()?;
    self.skip_spaces();
    let Some(&n) = self.stack.last() else {
      return Err(Decline("an e at the expression's end"));
    };
    if n.code == Catcode::CS {
      return self.decline_at(n, "an exponent from a control sequence");
    }
    let first = char_of(n).inspect_err(|_| self.at = Some(n))?;
    if first > b'9' {
      // \__fp_parse_exponent_aux:NN `0 ; #1`: back where it was
      self.stack.push(t);
      return Ok(0);
    }
    let mut negative = false;
    let mut digits = 0u32;
    let mut value: i64 = 0;
    loop {
      self.skip_spaces();
      let Some(&d) = self.stack.last() else {
        break;
      };
      if d.code == Catcode::CS {
        break;
      }
      let c = char_of(d).inspect_err(|_| self.at = Some(d))?;
      if digits == 0 && (c == b'+' || c == b'-') {
        if c == b'-' {
          negative = !negative;
        }
      } else if c.is_ascii_digit() {
        digits += 1;
        if digits > MAX_EXPONENT_DIGITS {
          return Err(Decline("an exponent past the integers"));
        }
        value = value * 10 + i64::from(c - b'0');
      } else {
        break;
      }
      self.stack.pop();
      self.a()?;
    }
    if digits == 0 {
      return Err(Decline("an exponent without digits"));
    }
    Ok(if negative { -value } else { value })
  }

  /// `\__fp_parse_one_register:NN` for a count or dimen register: `\tex_the:D` of it and an exponent. Its value is read
  /// by `\__fp_parse:n` (integer) or `\__fp_from_dim_test:ww` (dimension, `sp` times `1.52587890625e-5`).
  fn read_register(&mut self, t: Token, r: &Rc<Register>) -> R<Fp> {
    if r.parameters.is_some() || r.getter.is_some() {
      return self.decline_at(t, "a register with arguments or a computed value");
    }
    if r.mathglyph.is_some() {
      // `\__fp_parse_one_register_special:N`: \pi and \infty
      return self.decline_at(t, "a math character");
    }
    let value = match r.register_type {
      RegisterType::Number | RegisterType::CharDef => match r.value_of(Vec::new()) {
        Some(RegisterValue::Number(n)) => Reg::Int(n.0),
        _ => return self.decline_at(t, "a number register without a number"),
      },
      RegisterType::Dimension => match r.value_of(Vec::new()) {
        Some(RegisterValue::Dimension(d)) => Reg::Dim(d.0),
        _ => return self.decline_at(t, "a dimension register without a dimension"),
      },
      _ => return self.decline_at(t, "a register of another type"),
    };
    self.a()?;
    let ten = self.exponent()?;
    match value {
      Reg::Int(v) => num::from_int(v, ten),
      Reg::Dim(sp) => dimension_to_fp(sp, ten),
    }
  }

  /// A floating point number's stored form, `\s__fp` read: `\__fp_exp_after_f:nw` hands it on unchanged, and the parser's
  /// operations read its blocks as they stand, so only the canonical form is a value here: an exponent as `\int_value:w`
  /// writes it, four blocks of exactly four digits.
  fn read_fp(&mut self) -> R<Fp> {
    if self.stack.pop() != Some(T_CS!("\\__fp_chk:w")) {
      return Err(Decline("a misformed floating point number"));
    }
    let kind = self.read_digit()?;
    let sign = match self.read_digit()? {
      0 => Sign::Pos,
      2 => Sign::Neg,
      1 => return Err(Decline("nan")),
      _ => return Err(Decline("a misformed floating point number")),
    };
    match kind {
      1 => {
        let exp = self.read_exponent_group()?;
        let mut mant: u64 = 0;
        for _ in 0..4 {
          mant = mant * 10000 + self.read_block_group()?;
        }
        if !(POW10[15]..POW10[16]).contains(&mant) || exp.abs() > num::MAX_EXPONENT {
          return Err(Decline("a misformed floating point number"));
        }
        self.expect_sep()?;
        Ok(Fp::Normal { sign, exp: exp as i32, mant })
      },
      0 | 2 => {
        let mark = self.stack.pop();
        let tag = if mark == Some(T_CS!("\\s__fp_exact")) {
          Tag::Exact
        } else if mark == Some(T_CS!("\\s__fp_underflow")) && kind == 0 {
          Tag::Underflow
        } else {
          return Err(Decline("a flagged special value"));
        };
        self.expect_sep()?;
        Ok(if kind == 0 {
          Fp::Zero(sign, tag)
        } else {
          Fp::Inf(sign)
        })
      },
      _ => Err(Decline("nan")),
    }
  }

  fn expect_sep(&mut self) -> R<()> {
    if self.stack.pop() == Some(T_CS!("\\__fp_sep:")) {
      Ok(())
    } else {
      Err(Decline("a misformed floating point number"))
    }
  }

  fn read_digit(&mut self) -> R<u8> {
    match self.stack.pop() {
      Some(t) if t.code == Catcode::OTHER => t.with_str(|s| match s.as_bytes() {
        [d @ b'0'..=b'9'] => Ok(*d - b'0'),
        _ => Err(Decline("a misformed floating point number")),
      }),
      _ => Err(Decline("a misformed floating point number")),
    }
  }

  /// `{<exponent>}`: no leading zero, no `-0`.
  fn read_exponent_group(&mut self) -> R<i64> {
    let bad = Decline("a misformed floating point number");
    if self.stack.pop().map(|t| t.code) != Some(Catcode::BEGIN) {
      return Err(bad);
    }
    let mut value: i64 = 0;
    let mut negative = false;
    let mut digits = 0;
    loop {
      let t = self.stack.pop().ok_or(bad)?;
      if t.code == Catcode::END {
        break;
      }
      if t.code != Catcode::OTHER {
        return Err(bad);
      }
      t.with_str(|s| match s.as_bytes() {
        [b'-'] if digits == 0 && !negative => {
          negative = true;
          Ok(())
        },
        [d @ b'0'..=b'9'] if digits < 9 && (digits == 0 || value != 0) => {
          value = value * 10 + i64::from(*d - b'0');
          digits += 1;
          Ok(())
        },
        _ => Err(bad),
      })?;
    }
    if digits == 0 || (negative && value == 0) {
      return Err(bad);
    }
    Ok(if negative { -value } else { value })
  }

  /// `{<block>}`: exactly four digits.
  fn read_block_group(&mut self) -> R<u64> {
    let bad = Decline("a misformed floating point number");
    if self.stack.pop().map(|t| t.code) != Some(Catcode::BEGIN) {
      return Err(bad);
    }
    let mut value: u64 = 0;
    for _ in 0..4 {
      value = value * 10 + u64::from(self.read_digit()?);
    }
    if self.stack.pop().map(|t| t.code) != Some(Catcode::END) {
      return Err(bad);
    }
    Ok(value)
  }

  /// `\__fp_parse_letters:N`, `\__fp_parse_word:Nw`: the letters, an expansion before each next one.
  fn word(&mut self) -> R<(Val, bool)> {
    let mut buf = [0u8; 12];
    let mut n = 0;
    let first = *self.stack.last().ok_or(Decline("an empty word"))?;
    loop {
      self.skip_spaces();
      let Some(&t) = self.stack.last() else {
        break;
      };
      if t.code == Catcode::CS {
        break;
      }
      let c = char_of(t).inspect_err(|_| self.at = Some(t))?;
      if !is_letter(c) {
        break;
      }
      if n == buf.len() {
        return self.decline_at(first, "an unknown word");
      }
      buf[n] = c;
      n += 1;
      self.stack.pop();
      self.a()?;
    }
    let word = &buf[..n];
    if let Some(x) = word_constant(word) {
      return Ok((Val::One(x), false));
    }
    let Some(f) = word_function(word) else {
      return self.decline_at(first, "an unknown word");
    };
    let args = self.operand(prec::FUNC)?.args();
    Ok((Val::One(apply_function(f, &args)?), false))
  }
}

/// `\__fp_from_dim_test:ww <ten>, <sp> ;`: `sp` scaled points times `1.52587890625e-5`, rounded by `\__fp_mul_npos_o:Nww`.
fn dimension_to_fp(sp: i64, ten: i64) -> R<Fp> {
  if sp == 0 {
    return Ok(ZERO);
  }
  let unit = Fp::Normal {
    sign: Sign::Pos,
    exp:  (ten - 4).clamp(-20000, 20000) as i32,
    mant: 1525878906250000,
  };
  num::mul(num::from_int(sp, 0)?, unit)
}

/// A register `\dimexpr` reads a width from: no arguments, no computed value.
fn is_plain_dimension_register(t: &Token) -> bool {
  matches!(lookup_meaning(t), Some(Stored::Register(r))
    if matches!(r.register_type, RegisterType::Dimension | RegisterType::Glue) && r.parameters.is_none() && r.getter.is_none())
}

fn text_is(t: Token, s: &str) -> bool { t.code != Catcode::CS && t.with_str(|x| x == s) }

/// `[+-]* <decimal> <unit>` with spaces between, a magnitude below `\maxdimen` in its unit (a margin below it, whatever the
/// fraction): what `\dimexpr` reads without a word, and reports nothing on. Past that `Dimension too large` is the raw
/// macro's to report, once.
fn check_dimension_text(text: &[Token]) -> R<()> {
  let bad = Decline("a dimexpr beyond one dimension");
  let mut chars = Vec::with_capacity(text.len());
  for t in text {
    if t.code == Catcode::SPACE {
      chars.push(b' ');
      continue;
    }
    if t.code != Catcode::OTHER && t.code != Catcode::LETTER {
      return Err(bad);
    }
    chars.push(t.with_str(|s| if s.len() == 1 { s.as_bytes()[0] } else { 0 }));
  }
  let mut i = 0;
  let skip = |i: &mut usize| {
    while chars.get(*i) == Some(&b' ') {
      *i += 1;
    }
  };
  skip(&mut i);
  while matches!(chars.get(i), Some(b'+' | b'-')) {
    i += 1;
    skip(&mut i);
  }
  let (mut whole, mut count, mut dots) = (0u64, 0, 0);
  while let Some(c) = chars.get(i) {
    if c.is_ascii_digit() {
      count += 1;
      if dots == 0 {
        whole = whole.saturating_mul(10).saturating_add(u64::from(c - b'0'));
      }
    } else if *c == b'.' && dots == 0 {
      dots += 1;
    } else {
      break;
    }
    i += 1;
  }
  if count == 0 {
    return Err(bad);
  }
  skip(&mut i);
  let cap: u64 = match chars.get(i..i + 2).ok_or(bad)? {
    b"pt" | b"bp" => 16000,
    b"in" => 220,
    b"pc" => 1300,
    b"cm" => 570,
    b"mm" => 5700,
    b"dd" => 15000,
    b"cc" => 1250,
    b"sp" => 1 << 30,
    _ => return Err(bad),
  };
  if whole >= cap {
    return Err(bad);
  }
  i += 2;
  skip(&mut i);
  if i == chars.len() { Ok(()) } else { Err(bad) }
}

/// A register's value read for `\tex_the:D`.
enum Reg {
  Int(i64),
  Dim(i64),
}

fn normal_const(exp: i32, mant: u64) -> Fp { Fp::Normal { sign: Sign::Pos, exp, mant } }

/// `\__fp_parse_word_<word>:N` of a constant or a unit (expl3-code.tex:17246-17279); `nan` and the font's `em`, `ex`
/// are the raw macro's.
fn word_constant(word: &[u8]) -> Option<Fp> {
  Some(match word {
    b"inf" => num::INF,
    b"pi" => normal_const(1, 3141592653589793),
    b"deg" => normal_const(-1, 1745329251994330),
    b"true" => ONE,
    b"false" => ZERO,
    b"pt" => normal_const(1, 1000000000000000),
    b"in" => normal_const(2, 7227000000000000),
    b"pc" => normal_const(2, 1200000000000000),
    b"cm" => normal_const(2, 2845275590551181),
    b"mm" => normal_const(1, 2845275590551181),
    b"dd" => normal_const(1, 1070008564960630),
    b"cc" => normal_const(2, 1284010277952756),
    b"nd" => normal_const(1, 1066978346456693),
    b"nc" => normal_const(2, 1280374015748031),
    b"bp" => normal_const(1, 1003750000000000),
    b"sp" => normal_const(-4, 1525878906250000),
    _ => return None,
  })
}

#[derive(Clone, Copy)]
enum Function {
  Abs,
  Logb,
  Sign,
  Sqrt,
  Exp,
  Ln,
  Fact,
  Max,
  Min,
  Round(Mode),
  Sin(SinKind, Unit),
  Tan(bool, Unit),
  Asin(Unit),
  Acos(Unit),
  Acsc(Unit),
  Asec(Unit),
  Atan(Unit),
  Acot(Unit),
}

fn word_function(word: &[u8]) -> Option<Function> {
  use Function as F;
  use SinKind::{Cos, Csc, Sec, Sin};
  use Unit::{Deg, Rad};
  Some(match word {
    b"abs" => F::Abs,
    b"logb" => F::Logb,
    b"sign" => F::Sign,
    b"sqrt" => F::Sqrt,
    b"exp" => F::Exp,
    b"ln" => F::Ln,
    b"fact" => F::Fact,
    b"max" => F::Max,
    b"min" => F::Min,
    b"round" => F::Round(Mode::Nearest),
    b"trunc" => F::Round(Mode::Zero),
    b"floor" => F::Round(Mode::Ninf),
    b"ceil" => F::Round(Mode::Pinf),
    b"sin" => F::Sin(Sin, Rad),
    b"cos" => F::Sin(Cos, Rad),
    b"csc" => F::Sin(Csc, Rad),
    b"sec" => F::Sin(Sec, Rad),
    b"tan" => F::Tan(false, Rad),
    b"cot" => F::Tan(true, Rad),
    b"sind" => F::Sin(Sin, Deg),
    b"cosd" => F::Sin(Cos, Deg),
    b"cscd" => F::Sin(Csc, Deg),
    b"secd" => F::Sin(Sec, Deg),
    b"tand" => F::Tan(false, Deg),
    b"cotd" => F::Tan(true, Deg),
    b"asin" => F::Asin(Rad),
    b"acos" => F::Acos(Rad),
    b"acsc" => F::Acsc(Rad),
    b"asec" => F::Asec(Rad),
    b"atan" => F::Atan(Rad),
    b"acot" => F::Acot(Rad),
    b"asind" => F::Asin(Deg),
    b"acosd" => F::Acos(Deg),
    b"acscd" => F::Acsc(Deg),
    b"asecd" => F::Asec(Deg),
    b"atand" => F::Atan(Deg),
    b"acotd" => F::Acot(Deg),
    _ => return None,
  })
}

fn apply_binary(op: BinOp, a: Fp, b: Fp) -> R<Fp> {
  match op {
    BinOp::Add => num::add(a, b),
    BinOp::Sub => num::sub(a, b),
    BinOp::Mul => num::mul(a, b),
    BinOp::Div => num::div(a, b),
    BinOp::Pow => func::pow(a, b),
    // \__fp_&_o:ww, \__fp_|_o:ww
    BinOp::And => Ok(if a.is_zero() { a } else { b }),
    BinOp::Or => Ok(if a.is_zero() { b } else { a }),
  }
}

fn unary(args: &[Fp]) -> R<Fp> {
  match args {
    [x] => Ok(*x),
    _ => Err(Decline("a function's number of arguments")),
  }
}

fn apply_function(f: Function, args: &[Fp]) -> R<Fp> {
  match f {
    Function::Abs => {
      let x = unary(args)?;
      Ok(x.with_sign(Sign::Pos))
    },
    Function::Logb => match unary(args)? {
      Fp::Zero(..) => Err(Decline("division by zero: logb(0)")),
      Fp::Inf(_) => Ok(num::INF),
      Fp::Normal { exp, .. } => num::from_int(i64::from(exp) - 1, 0),
    },
    Function::Sign => {
      let x = unary(args)?;
      Ok(match x {
        Fp::Zero(..) => x,
        other => ONE.with_sign(other.sign()),
      })
    },
    Function::Sqrt => num::sqrt(unary(args)?),
    Function::Exp => func::exp(unary(args)?),
    Function::Ln => func::ln(unary(args)?),
    Function::Fact => func::fact(unary(args)?),
    Function::Max | Function::Min => {
      // \__fp_minmax_loop:Nww
      let is_min = matches!(f, Function::Min);
      let mut best = if is_min { num::INF } else { num::MINUS_INF };
      for x in args {
        let c = num::compare(best, *x);
        if (is_min && c == 1) || (!is_min && c == -1) {
          best = *x;
        }
      }
      Ok(best)
    },
    Function::Round(mode) => {
      let (x, places, mode) = match args {
        [x] => (*x, 0, mode),
        [x, n] => (
          *x,
          num::small_int(*n).ok_or(Decline("invalid: round's places"))?,
          mode,
        ),
        [x, n, third] if matches!(mode, Mode::Nearest) => (
          *x,
          num::small_int(*n).ok_or(Decline("invalid: round's places"))?,
          func::nearest_mode(*third),
        ),
        _ => return Err(Decline("a function's number of arguments")),
      };
      num::round(mode, x, places)
    },
    Function::Sin(kind, unit) => func::sin_like(unit, unary(args)?, kind),
    Function::Tan(cot, unit) => func::tan_like(unit, unary(args)?, cot),
    Function::Asin(unit) => func::asin(unit, unary(args)?),
    Function::Acos(unit) => func::acos(unit, unary(args)?),
    Function::Acsc(unit) => func::acsc(unit, unary(args)?),
    Function::Asec(unit) => func::asec(unit, unary(args)?),
    Function::Atan(unit) => match args {
      [y] => func::atan2(unit, *y, ONE),
      [y, x] => func::atan2(unit, *y, *x),
      _ => Err(Decline("a function's number of arguments")),
    },
    Function::Acot(unit) => match args {
      [x] => func::atan2(unit, ONE, *x),
      [x, y] => func::atan2(unit, *y, *x),
      _ => Err(Decline("a function's number of arguments")),
    },
  }
}
