//! Shared XML-replacement template AST + parser + runtime interpreter (#171).
//!
//! A constructor's `"<ltx:…>"` replacement is a tiny templating language (see the
//! grammar comment in `latexml_codegen::constructable`). Historically it had **two**
//! independent implementations: the compile-time proc-macro
//! (`latexml_codegen/src/constructable.rs`, a regex-strip state machine fused to
//! codegen) and the runtime byte-scanner (`latexml_contrib::script_bindings`).
//! Two implementations of one language is drift waiting to happen.
//!
//! This module is the single source of truth: one [`winnow`] parser produces one
//! [`ReplacementOp`] AST, consumed by **both** front-ends —
//! * the compile-time codegen walks `&[ReplacementOp]` and emits `quote!`,
//! * the runtime interpreter [`apply_ops`] walks the same AST against a live
//!   `Document`.
//!
//! The semantics mirror the Perl `LaTeXML::Core::Definition::Constructor::Compiler`
//! (`Compiler.pm`) faithfully — the existing `constructable.rs` is the ground
//! truth this reproduces, including its quirks (see [`unquote`]). The dialect:
//!
//! ```text
//!  #1..#9            n-th digested argument                 (Value::Arg)
//!  #name             named whatsit property                 (Value::Prop)
//!  &func(args,…)     function call (whitelisted at runtime)  (Value::Func)
//!  <q a='v' …>       open element + attributes              (OpenElement)
//!  <q … />           empty element (open + close)
//!  </q>              close element                          (CloseElement)
//!  <?q a='v' …?>     processing instruction                 (ProcessingInstruction)
//!  ?test(if)(else)   conditional                            (Conditional)
//!  ^ / ^^  prefix    float the next element/attribute       (FloatKind)
//!  key='v'           set attribute on current node          (SetAttribute)
//!  literal text      absorb as a string                     (Text)
//! ```

use std::borrow::Cow;

use rustc_hash::FxHashMap as HashMap;
use winnow::combinator::{alt, opt, peek, repeat};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{literal, one_of, take_while};

use libxml::tree::Node;

use crate::common::arena::SymHashMap;
use crate::common::error::{Error, Result};
use crate::common::font::Font;
use crate::common::store::Stored;
use crate::definition::FontDirective;
use crate::digested::Digested;
use crate::document::Document;

// ────────────────────────────── AST ──────────────────────────────

/// A `^` / `^^` float prefix attaching to the next open-element. `^` floats to
/// where the element is allowed; `^^` additionally closes intervening open
/// elements if possible (Perl Compiler.pm float_type 1 vs 2). Counts ≥2 collapse
/// to `Double` (counts ≥3 never occur in practice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatKind {
  Single,
  Double,
}

/// A substitutable value: `#n`, `#name`, `&func(…)`, or literal text. Literal only
/// arises inside function arguments / attribute strings — never at content
/// position (a bare literal there is [`ReplacementOp::Text`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
  /// `#n` (1..=9) — the n-th digested argument (1-based).
  Arg(usize),
  /// `#name` — a named whatsit property.
  Prop(String),
  /// `&func(args…)` — a function call (resolved through a whitelist at runtime,
  /// a Rust call at compile time).
  Func { name: String, args: Vec<FuncArg> },
  /// Literal text, already `unquote`d.
  Literal(String),
}

/// A `&func(…)` argument — either a bare value or a quoted interpolated string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncArg {
  Value(Value),
  Str(AttrValue),
}

/// An interpolated attribute value: `'role-#1'` → `[Literal("role-"), Value(Arg 1)]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrValue {
  pub parts: Vec<AttrPart>,
}

/// One piece of an [`AttrValue`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrPart {
  Literal(String),
  Value(Value),
  /// `?test(ifval)(elseval)` inside a quoted attribute string — branches are
  /// single values (Perl `translate_string`).
  Conditional { test: Value, then_val: Value, else_val: Value },
}

/// An attribute-list entry inside a `<tag …>` or `<?pi …?>` — a key/value pair or
/// a conditional set of pairs (Perl `translate_avpairs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrPair {
  KeyValue { key: String, value: AttrValue },
  Conditional { test: Value, then_attrs: Vec<AttrPair>, else_attrs: Vec<AttrPair> },
}

/// One operation in a compiled replacement template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplacementOp {
  /// `<q a='v' …>` (or `<q … />` when `self_closing`). `float` set when a `^`/`^^`
  /// prefix attaches here.
  OpenElement {
    qname: String,
    attrs: Vec<AttrPair>,
    float: Option<FloatKind>,
    self_closing: bool,
  },
  /// `</q>`.
  CloseElement { qname: String },
  /// `<?q a='v' …?>`.
  ProcessingInstruction { qname: String, attrs: Vec<AttrPair> },
  /// `#n` / `#name` / `&func(…)` at content position — absorb the value.
  AbsorbValue { value: Value },
  /// `key='v'` at content position — set the attribute on the current node.
  /// `float` set when a `^` prefix attaches here.
  SetAttribute { key: String, value: AttrValue, float: bool },
  /// Literal text to absorb.
  Text { text: String },
  /// `?test(if)(else)` — branches are op-lists.
  Conditional { test: Value, then_ops: Vec<ReplacementOp>, else_ops: Vec<ReplacementOp> },
}

// ───────────────────────────── parser ─────────────────────────────

/// Parse an XML-replacement template into a [`ReplacementOp`] op-list. The entry
/// point for both consumers (codegen at compile time, [`apply_ops`] at runtime).
pub fn parse_replacement(template: &str) -> Result<Vec<ReplacementOp>> {
  ops_with_float
    .parse(template)
    .map_err(|e| Error::from(format!("replacement template parse error: {e}")))
}

/// A leading `^`/`^^` float prefix, then a sequence of ops, with the float
/// attached to the first open-element / top-level attribute that follows. Used as
/// the top-level parser and for conditional op-branches (each re-parsed
/// independently, mirroring Perl's recursive `compile_replacement_tokens`).
fn ops_with_float(input: &mut &str) -> ModalResult<Vec<ReplacementOp>> {
  let float = opt(float_prefix).parse_next(input)?;
  let mut ops: Vec<ReplacementOp> = repeat(0.., op).parse_next(input)?;
  if let Some(fk) = float {
    attach_float(&mut ops, fk);
  }
  Ok(ops)
}

/// Attach a leading float to the first open-element / top-level attribute. Scans
/// only the top-level op list (a float does not reach into conditional branches).
fn attach_float(ops: &mut [ReplacementOp], fk: FloatKind) {
  for o in ops.iter_mut() {
    match o {
      ReplacementOp::OpenElement { float, .. } => {
        *float = Some(fk);
        return;
      },
      ReplacementOp::SetAttribute { float, .. } => {
        *float = true;
        return;
      },
      _ => {},
    }
  }
}

/// `^+ \s*` — one or more carets then whitespace (Perl `FLOAT_RE`).
fn float_prefix(input: &mut &str) -> ModalResult<FloatKind> {
  let carets = take_while(1.., '^').parse_next(input)?;
  ws(input)?;
  Ok(if carets.chars().count() >= 2 { FloatKind::Double } else { FloatKind::Single })
}

/// One top-level operation. Branch order mirrors `compile_replacement_tokens`'s
/// `while`-loop priority. `pi`/`open` consume leading whitespace (Perl regexes
/// allow `^\s*<`); `close`/`text` do not (asymmetry preserved faithfully).
fn op(input: &mut &str) -> ModalResult<ReplacementOp> {
  alt((
    conditional_op,
    pi_op,
    open_tag_op,
    close_tag_op,
    absorb_value_op,
    attribute_op,
    text_op,
  ))
  .parse_next(input)
}

fn conditional_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  peek((literal("?"), one_of(['#', '&']))).parse_next(input)?;
  let (test, if_s, else_s) = parse_conditional_raw(input)?;
  let then_ops = reparse_ops(&if_s)?;
  let else_ops = reparse_ops(&else_s)?;
  Ok(ReplacementOp::Conditional { test, then_ops, else_ops })
}

fn pi_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  ws(input)?;
  literal("<?").parse_next(input)?;
  let name = cut_err_(qname, input)?;
  let attrs = avpairs(input)?;
  ws(input)?;
  cut_err_lit("?>", input)?;
  Ok(ReplacementOp::ProcessingInstruction { qname: name, attrs })
}

fn open_tag_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  ws(input)?;
  literal("<").parse_next(input)?;
  // Backtrackable: `</…` or `<?…` must fall through to other branches.
  let name = qname.parse_next(input)?;
  let attrs = avpairs(input)?;
  let self_closing = opt(literal("/")).parse_next(input)?.is_some();
  cut_err_lit(">", input)?;
  Ok(ReplacementOp::OpenElement { qname: name, attrs, float: None, self_closing })
}

fn close_tag_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  // `</qname\s*>` — no leading whitespace (Perl `LEAD_CLOSE_TAG_RE`).
  literal("</").parse_next(input)?;
  let name = cut_err_(qname, input)?;
  ws(input)?;
  cut_err_lit(">", input)?;
  Ok(ReplacementOp::CloseElement { qname: name })
}

fn absorb_value_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  // Gate on `#`/`&` (Perl `LEAD_VALUE_RE`); content_value has no literal fallback.
  peek(one_of(['#', '&'])).parse_next(input)?;
  let value = content_value(input)?;
  Ok(ReplacementOp::AbsorbValue { value })
}

fn attribute_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  // `qname\s*=\s*'…'` (Perl `QNAME_KEY_RE`), no leading whitespace.
  let key = qname.parse_next(input)?;
  if opt((ws_p, literal("="), ws_p)).parse_next(input)?.is_none() {
    // qname without `=` is not an attribute — let alt try `text`.
    return Err(ErrMode::Backtrack(ContextError::new()));
  }
  let value = attr_string(input)?;
  Ok(ReplacementOp::SetAttribute { key, value, float: false })
}

fn text_op(input: &mut &str) -> ModalResult<ReplacementOp> {
  // Random text stops at `<` and the specials (Perl `LEAD_RANDOM_TEXT_RE`).
  let raw = take_literal_run(input, "", true)?;
  Ok(ReplacementOp::Text { text: unquote(&raw) })
}

// ── shared sub-parsers ──

/// `?test(a)(b)` → `(test, a_raw, b_raw)`. The caller (op / avpair / string
/// context) re-parses the raw branch strings appropriately. Mirrors Perl
/// `parse_conditional`: a missing first paren ⇒ both branches empty; a missing
/// second paren ⇒ else branch empty.
fn parse_conditional_raw(input: &mut &str) -> ModalResult<(Value, String, String)> {
  literal("?").parse_next(input)?;
  let test = match content_value(input) {
    Ok(v) => v,
    Err(ErrMode::Backtrack(e)) => return Err(ErrMode::Cut(e)),
    Err(e) => return Err(e),
  };
  match opt(bracketed).parse_next(input)? {
    None => Ok((test, String::new(), String::new())),
    Some(a) => {
      let b = opt(bracketed).parse_next(input)?.unwrap_or_default();
      Ok((test, a, b))
    },
  }
}

/// Extract a balanced `(…)` group, returning the inner text (Perl
/// `extract_bracketed`). Leading whitespace is skipped; a non-whitespace char
/// before any `(` means "no group here" (backtrack, input unchanged). Nested
/// parens balance; quotes are *not* special (faithful to the original).
fn bracketed(input: &mut &str) -> ModalResult<String> {
  let s: &str = input;
  let mut level: i32 = 0;
  let mut has_open = false;
  let mut extracted = String::new();
  let mut consumed = 0usize;
  let mut closed = false;
  for (i, c) in s.char_indices() {
    match c {
      ')' => {
        level -= 1;
        if level < 1 {
          consumed = i + c.len_utf8();
          closed = true;
          break;
        }
        extracted.push(c);
      },
      '(' => {
        has_open = true;
        level += 1;
        if level > 1 {
          extracted.push(c);
        }
      },
      other => {
        if level > 0 {
          extracted.push(other);
        } else if !other.is_whitespace() {
          break; // non-ws before any '(' — not a group
        }
      },
    }
  }
  if has_open && closed {
    *input = &s[consumed..];
    Ok(extracted)
  } else {
    Err(ErrMode::Backtrack(ContextError::new()))
  }
}

/// A value with no literal fallback: `&func(…)`, `#n`, `#name` (Perl
/// `translate_value` gated by `LEAD_VALUE_RE`).
fn content_value(input: &mut &str) -> ModalResult<Value> {
  alt((func_value, arg_value, prop_value)).parse_next(input)
}

/// A value *with* literal fallback (used for `&func` arguments): tries the value
/// forms, else a literal run excluding `exclude`.
fn full_value(input: &mut &str, exclude: &str) -> ModalResult<Value> {
  if let Some(v) = opt(content_value).parse_next(input)? {
    return Ok(v);
  }
  let raw = take_literal_run(input, exclude, false)?;
  Ok(Value::Literal(unquote(&raw)))
}

fn arg_value(input: &mut &str) -> ModalResult<Value> {
  literal("#").parse_next(input)?;
  let digits = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
  let n: usize = digits.parse().map_err(|_| ErrMode::Backtrack(ContextError::new()))?;
  if !(1..=9).contains(&n) {
    return Err(ErrMode::Cut(ContextError::new()));
  }
  Ok(Value::Arg(n))
}

fn prop_value(input: &mut &str) -> ModalResult<Value> {
  literal("#").parse_next(input)?;
  let name = take_while(1.., |c: char| c.is_alphanumeric() || c == '_' || c == '-')
    .map(str::to_string)
    .parse_next(input)?;
  Ok(Value::Prop(name))
}

fn func_value(input: &mut &str) -> ModalResult<Value> {
  // `&([\w:]*)\(` (Perl `FN_RE`).
  literal("&").parse_next(input)?;
  let name = take_while(0.., |c: char| c.is_alphanumeric() || c == '_' || c == ':')
    .map(str::to_string)
    .parse_next(input)?;
  literal("(").parse_next(input)?; // no '(' ⇒ backtrack (a bare `&amp;` is text)
  let mut args = Vec::new();
  loop {
    if probe(|i| (ws_p, literal(")")).void().parse_next(i), input) {
      break;
    }
    ws(input)?;
    let arg = if probe(|i| one_of(['\'', '"']).parse_next(i), input) {
      FuncArg::Str(attr_string(input)?)
    } else {
      FuncArg::Value(full_value(input, ",)")?)
    };
    args.push(arg);
    if opt((ws_p, literal(","), ws_p)).parse_next(input)?.is_none() {
      break;
    }
  }
  ws(input)?;
  cut_err_lit(")", input)?;
  Ok(Value::Func { name, args })
}

/// A quoted, interpolated attribute string (Perl `translate_string`). If no
/// opening quote is present, it consumes a single char and yields an empty value
/// — faithfully reproducing the original's quirk.
fn attr_string(input: &mut &str) -> ModalResult<AttrValue> {
  ws(input)?;
  let quote = match opt(one_of(['\'', '"'])).parse_next(input)? {
    Some(q) => q,
    None => {
      if let Some(c) = input.chars().next() {
        *input = &input[c.len_utf8()..];
      }
      return Ok(AttrValue { parts: Vec::new() });
    },
  };
  let mut parts = Vec::new();
  loop {
    if input.is_empty() {
      break;
    }
    if input.starts_with(quote) {
      *input = &input[quote.len_utf8()..];
      break;
    }
    if probe(|i| (literal("?"), one_of(['#', '&'])).void().parse_next(i), input) {
      let (test, if_s, else_s) = parse_conditional_raw(input)?;
      let then_val = parse_single_value(&if_s)?;
      let else_val = parse_single_value(&else_s)?;
      parts.push(AttrPart::Conditional { test, then_val, else_val });
      continue;
    }
    if probe(|i| one_of(['#', '&']).parse_next(i), input) {
      parts.push(AttrPart::Value(content_value(input)?));
      continue;
    }
    let raw = take_literal_run(input, "'\"", false)?;
    parts.push(AttrPart::Literal(unquote(&raw)));
  }
  Ok(AttrValue { parts })
}

/// A set of attribute pairs (Perl `translate_avpairs`): conditionals and
/// `key='v'` pairs, leading whitespace trimmed each iteration.
fn avpairs(input: &mut &str) -> ModalResult<Vec<AttrPair>> {
  let mut pairs = Vec::new();
  loop {
    ws(input)?;
    if let Some(c) = opt(avpair_conditional).parse_next(input)? {
      pairs.push(c);
      continue;
    }
    if let Some(kv) = opt(avpair_keyval).parse_next(input)? {
      pairs.push(kv);
      continue;
    }
    break;
  }
  Ok(pairs)
}

fn avpair_conditional(input: &mut &str) -> ModalResult<AttrPair> {
  peek((literal("?"), one_of(['#', '&']))).parse_next(input)?;
  let (test, if_s, else_s) = parse_conditional_raw(input)?;
  Ok(AttrPair::Conditional {
    test,
    then_attrs: reparse_avpairs(&if_s),
    else_attrs: reparse_avpairs(&else_s),
  })
}

fn avpair_keyval(input: &mut &str) -> ModalResult<AttrPair> {
  let key = qname.parse_next(input)?;
  if opt((ws_p, literal("="), ws_p)).parse_next(input)?.is_none() {
    return Err(ErrMode::Backtrack(ContextError::new()));
  }
  let value = attr_string(input)?;
  Ok(AttrPair::KeyValue { key, value })
}

/// A single value parsed from a conditional branch in string context (Perl
/// `translate_value` on the branch). Empty branch ⇒ empty literal; trailing text
/// is ignored (lenient, matching the original).
fn parse_single_value(s: &str) -> ModalResult<Value> {
  if s.is_empty() {
    return Ok(Value::Literal(String::new()));
  }
  let mut inp: &str = s;
  full_value(&mut inp, "")
}

/// Re-parse a conditional op-branch as a full op-list (strict: must consume all,
/// mirroring `compile_replacement_tokens` looping to empty).
fn reparse_ops(s: &str) -> ModalResult<Vec<ReplacementOp>> {
  ops_with_float.parse(s).map_err(|_| ErrMode::Cut(ContextError::new()))
}

/// Re-parse a conditional avpair-branch as an attribute list (lenient: trailing
/// text ignored, like `translate_avpairs`).
fn reparse_avpairs(s: &str) -> Vec<AttrPair> {
  let mut inp: &str = s;
  avpairs(&mut inp).unwrap_or_default()
}

/// `qname` (XML Name, Perl `QNAME_RE`). Approximated as alnum/`_`/`:`/`.`/`-`
/// with an alpha/`_`/`:` start — covers every `ltx:`-namespaced name in practice.
fn qname(input: &mut &str) -> ModalResult<String> {
  (one_of(is_qname_start), take_while(0.., is_qname_continue))
    .take()
    .map(str::to_string)
    .parse_next(input)
}

fn is_qname_start(c: char) -> bool { c.is_alphabetic() || c == '_' || c == ':' }
fn is_qname_continue(c: char) -> bool {
  c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-')
}

/// Consume optional whitespace, discarding it.
fn ws(input: &mut &str) -> ModalResult<()> {
  let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
  Ok(())
}

/// Whitespace as a tuple-usable parser (returns `()`).
fn ws_p(input: &mut &str) -> ModalResult<()> { ws(input) }

/// Non-consuming lookahead test: run `p` on a copy and report whether it
/// succeeds, leaving `input` untouched. (Pins the winnow error type, which a bare
/// `peek(...).is_ok()` cannot infer.)
fn probe<O>(mut p: impl FnMut(&mut &str) -> ModalResult<O>, input: &str) -> bool {
  let mut s: &str = input;
  p(&mut s).is_ok()
}

/// A literal that hard-fails (`cut_err`) instead of backtracking.
fn cut_err_lit(lit: &'static str, input: &mut &str) -> ModalResult<()> {
  match literal(lit).parse_next(input) {
    Ok(_) => Ok(()),
    Err(ErrMode::Backtrack(e)) => Err(ErrMode::Cut(e)),
    Err(e) => Err(e),
  }
}

/// Run a parser, converting a `Backtrack` into a `Cut` (committed position).
fn cut_err_<O>(
  mut p: impl FnMut(&mut &str) -> ModalResult<O>,
  input: &mut &str,
) -> ModalResult<O> {
  match p(input) {
    Ok(o) => Ok(o),
    Err(ErrMode::Backtrack(e)) => Err(ErrMode::Cut(e)),
    Err(e) => Err(e),
  }
}

/// Maximal run of literal text. A "unit" is `&amp;`, a TeX control sequence
/// `\letters`, an escape `\X`, or one ordinary char. Stops at `#`/`?`, a bare
/// `&`, a lone `\`, the `exclude` chars, and (when `exclude_lt`) `<`. Returns the
/// raw run (NOT yet `unquote`d). Faithful to Perl's `QUOTED_SPECIALS` classes.
fn take_literal_run(input: &mut &str, exclude: &str, exclude_lt: bool) -> ModalResult<String> {
  let s: &str = input;
  let mut pos = 0usize;
  while pos < s.len() {
    let rest = &s[pos..];
    let c = rest.chars().next().unwrap();
    if c == '&' {
      if rest.starts_with("&amp;") {
        pos += 5;
        continue;
      }
      break; // bare '&' is special
    }
    if c == '\\' {
      let after = &rest[1..];
      let letters: usize = after
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic() || *ch == '@')
        .map(char::len_utf8)
        .sum();
      if letters > 0 {
        pos += 1 + letters; // \textbf etc.
        continue;
      }
      if let Some(nc) = after.chars().next() {
        pos += 1 + nc.len_utf8(); // \X escape
        continue;
      }
      break; // lone trailing backslash
    }
    if c == '#' || c == '?' {
      break;
    }
    if exclude_lt && c == '<' {
      break;
    }
    if exclude.contains(c) {
      break;
    }
    pos += c.len_utf8();
  }
  if pos == 0 {
    return Err(ErrMode::Backtrack(ContextError::new()));
  }
  let raw = s[..pos].to_string();
  *input = &s[pos..];
  Ok(raw)
}

/// Reverse the template's escape conventions (Perl `unquote`). Reproduces the
/// original's exact behavior, including the quirk that `\X` (X ∈ `#?(&,<>\%`) is
/// **removed entirely** (the original `ESCAPED_OP` regex has no capture group, so
/// the replacement is the empty string), then `##`→`#` and `&amp;`→`&`.
pub fn unquote(text: &str) -> String {
  const ESCAPED: &[char] = &['#', '?', '(', '&', ',', '<', '>', '\\', '%'];
  let mut out = String::with_capacity(text.len());
  let mut i = 0usize;
  while i < text.len() {
    let rest = &text[i..];
    let c = rest.chars().next().unwrap();
    if c == '\\' {
      if let Some(nc) = rest[1..].chars().next() {
        if ESCAPED.contains(&nc) {
          i += 1 + nc.len_utf8(); // drop the whole `\X`
          continue;
        }
      }
    }
    out.push(c);
    i += c.len_utf8();
  }
  out.replace("##", "#").replace("&amp;", "&")
}

/// Double every backslash so the text can be embedded as a Rust string literal
/// (Perl `slashify`). Used by the compile-time codegen consumer.
pub fn slashify(text: &str) -> String { text.replace('\\', "\\\\") }

// ─────────────────────── runtime interpreter ───────────────────────

/// Execute a parsed replacement against a live `Document` — the runtime consumer
/// of the AST. Mirrors the Document operations the compile-time codegen emits.
pub fn apply_ops(
  ops: &[ReplacementOp],
  document: &mut Document,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<()> {
  let mut savenode: Option<Node> = None;
  exec_ops(ops, document, args, props, &mut savenode)?;
  if let Some(sn) = savenode {
    document.set_node(&sn);
  }
  Ok(())
}

fn exec_ops(
  ops: &[ReplacementOp],
  document: &mut Document,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
  savenode: &mut Option<Node>,
) -> Result<()> {
  for op in ops {
    match op {
      ReplacementOp::OpenElement { qname, attrs, float, self_closing } => {
        if let Some(fk) = float {
          *savenode = document.float_to_element(qname, matches!(fk, FloatKind::Double))?;
        }
        let av = eval_avpairs(attrs, args, props)?;
        if av.is_empty() {
          document.open_element(qname, None, None)?;
        } else {
          let mut map: HashMap<String, String> = HashMap::default();
          for (k, v) in av {
            map.insert(k, v);
          }
          let this_font_opt: Option<Cow<Font>> = match props.get("font") {
            Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
            Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
            Some(Stored::FontDirective(FontDirective::Closure(code))) => {
              Some(Cow::Owned(code(None)?))
            },
            _ => None,
          };
          if let Some(this_font) = this_font_opt {
            document.open_element(qname, Some(map), Some(&this_font))?;
          } else {
            document.open_element(qname, Some(map), None)?;
          }
        }
        if *self_closing {
          document.close_element(qname)?;
        }
      },
      ReplacementOp::CloseElement { qname } => {
        document.close_element(qname)?;
      },
      ReplacementOp::ProcessingInstruction { qname, attrs } => {
        let av = eval_avpairs(attrs, args, props)?;
        if av.is_empty() {
          document.insert_pi(qname, None)?;
        } else {
          let mut map: HashMap<String, String> = HashMap::default();
          for (k, v) in av {
            map.insert(k, v);
          }
          document.insert_pi(qname, Some(map))?;
        }
      },
      ReplacementOp::AbsorbValue { value } => {
        absorb_value(value, document, args, props)?;
      },
      ReplacementOp::SetAttribute { key, value, float } => {
        let val_str = eval_attr_value(value, args, props)?;
        if *float {
          *savenode = document.float_to_attribute(key);
          let mut node = document.get_node().clone();
          document.set_attribute(&mut node, key, &val_str)?;
          if let Some(ref sn) = savenode {
            document.set_node(sn);
          }
        } else {
          let mut node = document.get_node().clone();
          document.set_attribute(&mut node, key, &val_str)?;
        }
      },
      ReplacementOp::Text { text } => {
        document.absorb_string(text, props)?;
      },
      ReplacementOp::Conditional { test, then_ops, else_ops } => {
        if eval_bool(test, args, props)? {
          exec_ops(then_ops, document, args, props, savenode)?;
        } else {
          exec_ops(else_ops, document, args, props, savenode)?;
        }
      },
    }
  }
  Ok(())
}

/// Evaluate attribute pairs to `(key, value)` strings (font key dropped, as the
/// codegen drops it — the open font comes from `props["font"]` instead).
fn eval_avpairs(
  attrs: &[AttrPair],
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<Vec<(String, String)>> {
  let mut out = Vec::new();
  for a in attrs {
    match a {
      AttrPair::KeyValue { key, value } => {
        if key == "font" {
          continue;
        }
        out.push((key.clone(), eval_attr_value(value, args, props)?));
      },
      AttrPair::Conditional { test, then_attrs, else_attrs } => {
        let branch = if eval_bool(test, args, props)? { then_attrs } else { else_attrs };
        out.extend(eval_avpairs(branch, args, props)?);
      },
    }
  }
  Ok(out)
}

fn eval_attr_value(
  v: &AttrValue,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<String> {
  let mut s = String::new();
  for part in &v.parts {
    match part {
      AttrPart::Literal(lit) => s.push_str(lit),
      AttrPart::Value(val) => s.push_str(&value_to_attribute(val, args, props)?),
      AttrPart::Conditional { test, then_val, else_val } => {
        let chosen = if eval_bool(test, args, props)? { then_val } else { else_val };
        s.push_str(&value_to_attribute(chosen, args, props)?);
      },
    }
  }
  Ok(s)
}

/// Render a value as an attribute string: `to_attribute()` of the resolved
/// argument/property, or empty when absent (codegen's
/// `match … { Some(v) => v.to_attribute(), None => String::new() }`).
fn value_to_attribute(
  v: &Value,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<String> {
  Ok(match v {
    Value::Arg(n) => match args.get(n - 1) {
      Some(Some(d)) => d.to_attribute(),
      _ => String::new(),
    },
    Value::Prop(name) => match props.get(name) {
      Some(stored) => stored.to_attribute(),
      None => String::new(),
    },
    Value::Func { name, args: fargs } => call_func(name, fargs, args, props)?,
    Value::Literal(lit) => lit.clone(),
  })
}

/// Absorb a value at content position (codegen's `Into<Option<Digested>>` +
/// `document.absorb`).
fn absorb_value(
  v: &Value,
  document: &mut Document,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<()> {
  match v {
    Value::Arg(n) => {
      if let Some(Some(d)) = args.get(n - 1) {
        document.absorb(d, None)?;
      }
    },
    Value::Prop(name) => {
      if let Some(stored) = props.get(name) {
        let dig: Option<Digested> = stored.into();
        if let Some(ref d) = dig {
          document.absorb(d, None)?;
        }
      }
    },
    Value::Func { name, args: fargs } => {
      let s = call_func(name, fargs, args, props)?;
      if !s.is_empty() {
        document.absorb_string(&s, props)?;
      }
    },
    Value::Literal(_) => {}, // a literal never reaches content position
  }
  Ok(())
}

/// The truth test of a conditional (codegen's
/// `!v.to_string().is_empty() && v.to_string() != "false"`).
fn eval_bool(
  v: &Value,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<bool> {
  Ok(match v {
    Value::Arg(n) => match args.get(n - 1) {
      Some(Some(d)) => is_truthy(&d.to_string()),
      _ => false,
    },
    Value::Prop(name) => match props.get(name) {
      Some(stored) => is_truthy(&stored.to_string()),
      None => false,
    },
    Value::Func { name, args: fargs } => is_truthy(&call_func(name, fargs, args, props)?),
    Value::Literal(s) => is_truthy(s),
  })
}

fn is_truthy(s: &str) -> bool { !s.is_empty() && s != "false" }

/// Resolve a `&func(…)` at runtime through a curated whitelist (the compile-time
/// path resolves these to Rust calls; the runtime path keeps untrusted scripts
/// safe by only allowing vetted helpers). Arguments are rendered to strings.
/// No active binding uses `&func` (they appear only in commented Perl-only
/// templates), so the whitelist is intentionally minimal and grows on demand.
fn call_func(
  name: &str,
  fargs: &[FuncArg],
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<String> {
  let mut argv: Vec<String> = Vec::with_capacity(fargs.len());
  for fa in fargs {
    argv.push(match fa {
      FuncArg::Value(v) => value_to_attribute(v, args, props)?,
      FuncArg::Str(s) => eval_attr_value(s, args, props)?,
    });
  }
  match name {
    "ToString" => Ok(argv.join("")),
    _ => Err(Error::from(format!(
      "runtime template: function &{name}(…) is not in the whitelist"
    ))),
  }
}

// ────────────────────────────── tests ──────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  fn lit(s: &str) -> AttrValue { AttrValue { parts: vec![AttrPart::Literal(s.to_string())] } }
  fn argval(n: usize) -> AttrValue { AttrValue { parts: vec![AttrPart::Value(Value::Arg(n))] } }

  #[test]
  fn plain_element_with_arg() {
    let ops = parse_replacement("<ltx:emph>#1</ltx:emph>").unwrap();
    assert_eq!(
      ops,
      vec![
        ReplacementOp::OpenElement {
          qname: "ltx:emph".into(),
          attrs: vec![],
          float: None,
          self_closing: false,
        },
        ReplacementOp::AbsorbValue { value: Value::Arg(1) },
        ReplacementOp::CloseElement { qname: "ltx:emph".into() },
      ]
    );
  }

  #[test]
  fn element_with_literal_attribute_and_arg() {
    let ops = parse_replacement("<ltx:text class='ok'>#1</ltx:text>").unwrap();
    assert_eq!(
      ops,
      vec![
        ReplacementOp::OpenElement {
          qname: "ltx:text".into(),
          attrs: vec![AttrPair::KeyValue { key: "class".into(), value: lit("ok") }],
          float: None,
          self_closing: false,
        },
        ReplacementOp::AbsorbValue { value: Value::Arg(1) },
        ReplacementOp::CloseElement { qname: "ltx:text".into() },
      ]
    );
  }

  #[test]
  fn attribute_value_interpolates_arg() {
    let ops = parse_replacement("<ltx:ref class='#2'>#1</ltx:ref>").unwrap();
    let ReplacementOp::OpenElement { attrs, .. } = &ops[0] else { panic!() };
    assert_eq!(attrs, &vec![AttrPair::KeyValue { key: "class".into(), value: argval(2) }]);
  }

  #[test]
  fn self_closing_element() {
    let ops = parse_replacement("<ltx:break/>").unwrap();
    assert_eq!(
      ops,
      vec![ReplacementOp::OpenElement {
        qname: "ltx:break".into(),
        attrs: vec![],
        float: None,
        self_closing: true,
      }]
    );
  }

  #[test]
  fn whitespace_before_tag_is_dropped_but_text_kept() {
    // Leading ws before an open tag is eaten by the tag (Perl `^\s*<`); a close
    // tag has no leading-ws rule so the ws becomes text.
    let ops = parse_replacement("<a>\n  <b></b>\n  </a>").unwrap();
    assert_eq!(
      ops,
      vec![
        ReplacementOp::OpenElement { qname: "a".into(), attrs: vec![], float: None, self_closing: false },
        ReplacementOp::OpenElement { qname: "b".into(), attrs: vec![], float: None, self_closing: false },
        ReplacementOp::CloseElement { qname: "b".into() },
        ReplacementOp::Text { text: "\n  ".into() },
        ReplacementOp::CloseElement { qname: "a".into() },
      ]
    );
  }

  #[test]
  fn footnote_corpus_specimen() {
    // plain_constructs.rs:293 — the richest active template.
    let ops =
      parse_replacement("^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>")
        .unwrap();
    assert_eq!(
      ops,
      vec![
        ReplacementOp::OpenElement {
          qname: "ltx:note".into(),
          attrs: vec![
            AttrPair::KeyValue { key: "role".into(), value: lit("footnote") },
            AttrPair::Conditional {
              test: Value::Prop("mark".into()),
              then_attrs: vec![AttrPair::KeyValue {
                key: "mark".into(),
                value: AttrValue { parts: vec![AttrPart::Value(Value::Prop("mark".into()))] },
              }],
              else_attrs: vec![],
            },
          ],
          float: Some(FloatKind::Single),
          self_closing: false,
        },
        ReplacementOp::Conditional {
          test: Value::Prop("prenote".into()),
          then_ops: vec![
            ReplacementOp::AbsorbValue { value: Value::Prop("prenote".into()) },
            ReplacementOp::Text { text: " ".into() },
          ],
          else_ops: vec![],
        },
        ReplacementOp::AbsorbValue { value: Value::Arg(2) },
        ReplacementOp::CloseElement { qname: "ltx:note".into() },
      ]
    );
  }

  #[test]
  fn pi_corpus_specimen() {
    // latex_constructs.rs:2702 / :4088 — PI with inline conditional.
    let ops = parse_replacement("<?latexml class='#2' ?#1(options='#1')?>").unwrap();
    assert_eq!(
      ops,
      vec![ReplacementOp::ProcessingInstruction {
        qname: "latexml".into(),
        attrs: vec![
          AttrPair::KeyValue { key: "class".into(), value: argval(2) },
          AttrPair::Conditional {
            test: Value::Arg(1),
            then_attrs: vec![AttrPair::KeyValue { key: "options".into(), value: argval(1) }],
            else_attrs: vec![],
          },
        ],
      }]
    );
  }

  #[test]
  fn float_double_caret() {
    let ops = parse_replacement("^^<ltx:x/>").unwrap();
    let ReplacementOp::OpenElement { float, .. } = &ops[0] else { panic!() };
    assert_eq!(float, &Some(FloatKind::Double));
  }

  #[test]
  fn top_level_conditional_with_else() {
    let ops = parse_replacement("?#1(<a/>)(<b/>)").unwrap();
    assert_eq!(
      ops,
      vec![ReplacementOp::Conditional {
        test: Value::Arg(1),
        then_ops: vec![ReplacementOp::OpenElement {
          qname: "a".into(),
          attrs: vec![],
          float: None,
          self_closing: true,
        }],
        else_ops: vec![ReplacementOp::OpenElement {
          qname: "b".into(),
          attrs: vec![],
          float: None,
          self_closing: true,
        }],
      }]
    );
  }

  #[test]
  fn prop_hole_at_content_and_arg_distinguished() {
    let ops = parse_replacement("#mark#2").unwrap();
    assert_eq!(
      ops,
      vec![
        ReplacementOp::AbsorbValue { value: Value::Prop("mark".into()) },
        ReplacementOp::AbsorbValue { value: Value::Arg(2) },
      ]
    );
  }

  #[test]
  fn func_value_parses() {
    let ops = parse_replacement("<a x='&ToString(#1)'/>").unwrap();
    let ReplacementOp::OpenElement { attrs, .. } = &ops[0] else { panic!() };
    assert_eq!(
      attrs,
      &vec![AttrPair::KeyValue {
        key: "x".into(),
        value: AttrValue {
          parts: vec![AttrPart::Value(Value::Func {
            name: "ToString".into(),
            args: vec![FuncArg::Value(Value::Arg(1))],
          })],
        },
      }]
    );
  }

  #[test]
  fn unquote_reproduces_original_quirks() {
    assert_eq!(unquote("a&amp;b"), "a&b");
    assert_eq!(unquote(r"\#"), ""); // \X (X special) removed entirely
    assert_eq!(unquote(r"\textbf"), r"\textbf"); // CS survives
    assert_eq!(unquote("a##b"), "a#b");
  }

  #[test]
  fn empty_template_is_empty_oplist() {
    assert_eq!(parse_replacement("").unwrap(), vec![]);
  }

  // ── evaluation-semantics conformance (Document-free) ──
  //
  // These exercise the runtime interpreter's value/attribute/condition
  // evaluation directly, with controlled args/props, and assert it computes
  // exactly what the compile-time codegen emits. In particular they pin the
  // crucial point that attribute values render via `to_attribute()` (matching
  // codegen) — NOT the `untex()` the previous byte-scanner used.

  use crate::common::arena;

  fn dig(s: &str) -> Digested { s.to_string().into() }

  fn props_with(pairs: &[(&str, &str)]) -> SymHashMap<Stored> {
    let mut m = SymHashMap::default();
    for (k, v) in pairs {
      m.insert(k, Stored::String(arena::pin(v)));
    }
    m
  }

  /// Pull the `attrs` out of the footnote specimen's `OpenElement`.
  fn footnote_attrs() -> Vec<AttrPair> {
    let ops = parse_replacement(
      "^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>",
    )
    .unwrap();
    match &ops[0] {
      ReplacementOp::OpenElement { attrs, .. } => attrs.clone(),
      _ => panic!("expected OpenElement"),
    }
  }

  #[test]
  fn footnote_conditional_attr_fires_when_prop_present() {
    let attrs = footnote_attrs();
    let props = props_with(&[("mark", "MK")]);
    let av = eval_avpairs(&attrs, &[], &props).unwrap();
    // role literal + conditional mark attr (mark prop present ⇒ branch taken),
    // and the mark value is rendered via the Stored's `to_attribute()`.
    let mk = Stored::String(arena::pin("MK")).to_attribute();
    assert_eq!(av, vec![("role".to_string(), "footnote".to_string()), ("mark".to_string(), mk)]);
  }

  #[test]
  fn footnote_conditional_attr_absent_when_prop_missing() {
    let attrs = footnote_attrs();
    let av = eval_avpairs(&attrs, &[], &SymHashMap::default()).unwrap();
    assert_eq!(av, vec![("role".to_string(), "footnote".to_string())]);
  }

  #[test]
  fn footnote_prenote_condition_truth_test() {
    // `?#prenote(...)` truth test mirrors codegen: present non-"false" ⇒ true.
    assert!(eval_bool(&Value::Prop("prenote".into()), &[], &props_with(&[("prenote", "P")])).unwrap());
    assert!(!eval_bool(&Value::Prop("prenote".into()), &[], &SymHashMap::default()).unwrap());
    assert!(!eval_bool(&Value::Prop("x".into()), &[], &props_with(&[("x", "false")])).unwrap());
    assert!(!eval_bool(&Value::Prop("x".into()), &[], &props_with(&[("x", "")])).unwrap());
  }

  #[test]
  fn pi_attr_interpolation_uses_to_attribute_and_conditional() {
    let ops = parse_replacement("<?latexml class='#2' ?#1(options='#1')?>").unwrap();
    let ReplacementOp::ProcessingInstruction { attrs, .. } = &ops[0] else { panic!() };

    let a1 = dig("opts");
    let a2 = dig("article");
    let args = vec![Some(a1.clone()), Some(a2.clone())];
    let av = eval_avpairs(attrs, &args, &SymHashMap::default()).unwrap();
    // Both `#1` and `#2` render via Digested::to_attribute (the codegen rule),
    // and the `?#1(...)` conditional fires because arg 1 is present.
    assert_eq!(
      av,
      vec![
        ("class".to_string(), a2.to_attribute()),
        ("options".to_string(), a1.to_attribute()),
      ]
    );

    // arg 1 absent ⇒ the conditional avpair drops out; class (arg 2) stays.
    let av2 = eval_avpairs(attrs, &[None, Some(a2.clone())], &SymHashMap::default()).unwrap();
    assert_eq!(av2, vec![("class".to_string(), a2.to_attribute())]);
  }

  #[test]
  fn font_attribute_key_is_dropped() {
    // Codegen drops a literal `font=` attribute (the open font comes from
    // props["font"] instead). The interpreter must too.
    let ops = parse_replacement("<ltx:x font='ignored' class='keep'/>").unwrap();
    let ReplacementOp::OpenElement { attrs, .. } = &ops[0] else { panic!() };
    let av = eval_avpairs(attrs, &[], &SymHashMap::default()).unwrap();
    assert_eq!(av, vec![("class".to_string(), "keep".to_string())]);
  }

  #[test]
  fn plain_text_only() {
    assert_eq!(
      parse_replacement("hello world").unwrap(),
      vec![ReplacementOp::Text { text: "hello world".into() }]
    );
  }
}
