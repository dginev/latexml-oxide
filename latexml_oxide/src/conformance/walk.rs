//! The chain walker: a bounded symbolic reading of a definition, following its tail calls, that
//! yields the arguments the document supplies and the mode transitions made before material.
//!
//! A control sequence reads further arguments from the document only where it is the last call of
//! a body (or of a peek's branch); the arguments the body gives it itself are matched to its
//! parameters and substituted into its body (`\@rsbox{#1}` gives one; a passed continuation is
//! followed). A body whose first call is not its last is read up to its last top-level call, the
//! one that reads on (`\footnote`'s `\@footnotetext`). The kernel's peeks are read by name:
//! `\@ifnextchar`/`\kernel@ifnextchar` (latex.ltx:1756-1760), `\@ifstar`, `\@testopt` and
//! `\@protected@testopt` (latex.ltx:1249-1261). A robust command's `\x@protect\cs\protect\cs␣`
//! wrapper leads to its `\cs␣` body.

use latexml_core::{
  T_CS,
  common::store::Stored,
  definition::{Definition, ExpansionBody, PrimitiveBody},
  state::lookup_meaning,
  token::{Catcode, Token},
};

use super::view::{Arg, Given, View, args_of};

/// An argument a body gives a call: its tokens (a group's without the braces), and whether it was
/// a bracketed `[…]` one or a braced group.
#[derive(Clone)]
struct Item {
  tokens:  Vec<Token>,
  bracket: bool,
  group:   bool,
}

fn givens<'a>(texts: &'a [String], supplied: &[Item]) -> Vec<Given<'a>> {
  texts
    .iter()
    .zip(supplied)
    .map(|(text, item)| Given {
      text:    text.as_str(),
      bracket: item.bracket,
      group:   item.group,
    })
    .collect()
}

/// How many definitions one chain may pass through.
const MAX_DEPTH: usize = 16;

/// The kernel's peeks, with how many operands each takes.
const PEEKS: [(&str, usize); 5] = [
  ("\\@ifnextchar", 3),
  ("\\kernel@ifnextchar", 3),
  ("\\@ifstar", 2),
  ("\\@testopt", 2),
  ("\\@protected@testopt", 3),
];

/// The view of control sequence `cs` (`\makebox`) in the current State, or `None` when it is
/// undefined.
pub fn view_of(cs: &str) -> Option<View> {
  let token = T_CS!(cs);
  lookup_meaning(&token)?;
  Some(walk_cs(&token, &[], false, 0).0)
}

fn cs_name(token: &Token) -> String { token.with_cs_name(ToString::to_string) }

fn is_space(token: &Token) -> bool { token.get_catcode() == Catcode::SPACE }

/// One argument item of a token list: a braced group (its inner tokens) or one token. The flag
/// says the item was a bracketed `[…]` group, its inner tokens given.
fn next_item(tokens: &[Token], at: &mut usize, brackets: bool) -> Option<(Vec<Token>, bool)> {
  while tokens.get(*at).is_some_and(is_space) {
    *at += 1;
  }
  let first = *tokens.get(*at)?;
  *at += 1;
  let (open, close, bracket) = match first.get_catcode() {
    Catcode::BEGIN => (None, None, false),
    Catcode::OTHER if brackets && first.with_str(|s| s == "[") => (Some('['), Some(']'), true),
    _ => return Some((vec![first], false)),
  };
  let mut depth = 0usize;
  let mut inner = Vec::new();
  while let Some(&token) = tokens.get(*at) {
    *at += 1;
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 && open.is_none() => return Some((inner, false)),
      Catcode::END => depth = depth.saturating_sub(1),
      Catcode::OTHER if depth == 0 && close.is_some() && token.with_str(|s| s == "]") => {
        return Some((inner, bracket));
      },
      _ => {},
    }
    inner.push(token);
  }
  Some((inner, bracket))
}

fn text(tokens: &[Token]) -> String { tokens.iter().map(ToString::to_string).collect() }

/// The items of all of `tokens`.
fn items(tokens: &[Token]) -> Vec<Item> {
  let mut at = 0;
  let mut out = Vec::new();
  loop {
    while tokens.get(at).is_some_and(is_space) {
      at += 1;
    }
    let group = tokens
      .get(at)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN);
    let Some((tokens, bracket)) = next_item(tokens, &mut at, true) else {
      break;
    };
    out.push(Item { tokens, bracket, group });
  }
  out
}

/// How a call meets the items that follow it in a body: how many it consumes, and whether it goes
/// on reading the document after them. `None` for what is not a macro, primitive or constructor.
fn consumes(token: &Token, following: &[Item]) -> Option<(usize, bool)> {
  let params = match lookup_meaning(token)? {
    Stored::Expandable(d) => d.get_parameters().cloned(),
    Stored::Primitive(d) => d.get_parameters().cloned(),
    Stored::Constructor(d) => d.get_parameters().cloned(),
    _ => return None,
  };
  let Some(params) = params else {
    return Some((0, false));
  };
  let texts: Vec<String> = following.iter().map(|item| text(&item.tokens)).collect();
  let mut args = Vec::new();
  let read = args_of(&params, &givens(&texts, following), false, &mut args);
  Some((following.len() - read.unconsumed, !args.is_empty()))
}

/// Append a walked continuation to `view`.
fn extend(view: &mut View, tail: View) {
  view.args.extend(tail.args);
  let prologue = &mut view.prologue;
  prologue.enter_horizontal |= tail.prologue.enter_horizontal;
  prologue.leave_horizontal |= tail.prologue.leave_horizontal;
  prologue.bsphack |= tail.prologue.bsphack;
  prologue.mode_test |= tail.prologue.mode_test;
  prologue.declared_both |= tail.prologue.declared_both;
  if prologue.mode.is_none() {
    prologue.mode = tail.prologue.mode;
  }
  view.complete = tail.complete;
  view.prologue_final = tail.prologue_final;
  view.chain.extend(tail.chain);
  view.notes.extend(tail.notes);
}

/// Walk the definition of `token`, given the `supplied` items by the calling body. Returns the
/// view and whether a pending optional was read.
fn walk_cs(token: &Token, supplied: &[Item], pending: bool, depth: usize) -> (View, bool) {
  let mut view = View {
    chain: vec![cs_name(token)],
    ..View::default()
  };
  if depth > MAX_DEPTH {
    view
      .notes
      .push(format!("depth limit at {}", cs_name(token)));
    return (view, false);
  }
  let texts: Vec<String> = supplied.iter().map(|item| text(&item.tokens)).collect();
  let given = givens(&texts, supplied);
  let consumed = match lookup_meaning(token) {
    None => {
      view.notes.push(format!("{} is undefined", cs_name(token)));
      return (view, false);
    },
    Some(Stored::Expandable(expandable)) => {
      let read = expandable
        .get_parameters()
        .map(|params| args_of(params, &given, pending, &mut view.args));
      let consumed = read.as_ref().is_some_and(|read| read.consumed_pending);
      match expandable.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => {
          // The given arguments in place of their parameters: a continuation the caller passes
          // (`\adl@hdashline\adl@ihdashline`) is followed where the body calls it.
          let filled = read.map(|read| read.filled).unwrap_or_default();
          let body = substitute(body.unlist_ref(), &filled, supplied);
          let (tail, tail_consumed) = walk_list(&body, pending && !consumed, depth + 1);
          extend(&mut view, tail);
          consumed || tail_consumed
        },
        Some(ExpansionBody::Closure(_)) => {
          view
            .notes
            .push(format!("{} is coded in Rust", cs_name(token)));
          return (view, consumed);
        },
        None => {
          view.complete = true;
          view.prologue_final = true;
          consumed
        },
      }
    },
    Some(Stored::Primitive(primitive)) => {
      let coded = matches!(primitive.replacement, Some(PrimitiveBody::Closure(_)));
      declared(&mut view, &*primitive, &given, pending, coded)
    },
    Some(Stored::Constructor(constructor)) => {
      let coded = !constructor.before_digest.is_empty();
      declared(&mut view, &*constructor, &given, pending, coded)
    },
    // A character, register, conditional, math primitive: an operation of TeX's.
    Some(_) => {
      view.complete = true;
      false
    },
  };
  (view, consumed)
}

/// A primitive or constructor reached as a tail: its declared parameters and mode record. Its
/// prologue is final when the record declares a transition, or when no code of its own
/// (`coded`: a primitive's closure, a constructor's before-digestion hooks) could make one by hand
/// (KERNEL_CAPABILITIES K13 stage 0).
fn declared(
  view: &mut View,
  definition: &dyn Definition,
  supplied: &[Given],
  pending: bool,
  coded: bool,
) -> bool {
  let consumed = definition
    .get_parameters()
    .is_some_and(|params| args_of(params, supplied, pending, &mut view.args).consumed_pending);
  let record = definition.declared_mode();
  if let Some(declared) = record {
    view.prologue.declare(&declared);
  }
  let transition = record
    .is_some_and(|r| r.enter_horizontal || r.leave_horizontal || r.mode.is_some() || r.bounded);
  view.complete = true;
  view.prologue_final = transition || !coded;
  consumed
}

/// Walk a body or a branch. Returns its view and whether a pending optional was read.
fn walk_list(tokens: &[Token], pending: bool, depth: usize) -> (View, bool) {
  let mut view = View::default();
  let mut at = 0;
  while let Some(&token) = tokens.get(at) {
    at += 1;
    if is_space(&token) {
      continue;
    }
    if !token.get_catcode().is_active_or_cs() {
      // A character, `#1`, a group: the material begins, and what it does to the mode is TeX's
      // to decide (a letter in vertical mode starts a paragraph, tex.web §1090).
      view.complete = true;
      return (view, false);
    }
    let name = cs_name(&token);
    match name.as_str() {
      "\\relax" | "\\protect" => {},
      "\\x@protect" => at += 1,
      "\\leavevmode" | "\\hmode@bgroup" => view.prologue.enter_horizontal = true,
      "\\par" => view.prologue.leave_horizontal = true,
      "\\@bsphack" => view.prologue.bsphack = true,
      "\\ifmmode" => {
        view.prologue.mode_test = true;
        view.notes.push("tests \\ifmmode".to_string());
        return (view, false);
      },
      _ if PEEKS.iter().any(|(peek, _)| *peek == name) => {
        let (tail, consumed) = walk_peek(&name, &tokens[at..], pending, depth);
        extend(&mut view, tail);
        return (view, consumed);
      },
      _ => {
        // The head: a tail call when it consumes all the rest of the list as its arguments.
        let rest = &tokens[at..];
        let given = items(rest);
        if consumes(&token, &given).is_some_and(|(consumed, _)| consumed == given.len())
          || given.is_empty()
        {
          let (tail, consumed) = walk_cs(&token, &given, pending, depth);
          extend(&mut view, tail);
          return (view, consumed);
        }
        // Given more than it reads: the body goes on after it, and its last act is what reads the
        // document — a peek that ends it (`\noalign{\ifnum0=`}\fi … \@ifnextchar[{#1}{…}`,
        // arydshln.sty:432-437), or its last top-level call when that reads more than it is
        // given. The prologue is known only up to here.
        view.notes.push(format!("prologue unknown past {name}"));
        if let Some((peek, operands)) = trailing_peek(rest) {
          let (tail, consumed) = walk_peek(&peek, operands, pending, depth);
          extend(&mut view, tail);
          view.prologue_final = false;
          return (view, consumed);
        }
        match last_call(&tokens[at - 1..]) {
          LastCall::Reads(call, given) => {
            let (tail, consumed) = walk_cs(&call, &given, pending, depth);
            extend(&mut view, tail);
            view.prologue_final = false;
            return (view, consumed);
          },
          LastCall::Conditional => {
            view.notes.push("ends in a conditional".to_string());
            return (view, false);
          },
          LastCall::Complete => {
            view.complete = true;
            return (view, false);
          },
        }
      },
    }
  }
  view.complete = true;
  view.prologue_final = true;
  (view, false)
}

/// What the last top-level call of a body does, after its first call.
enum LastCall {
  /// It reads more arguments than the body gives it: they come from the document.
  Reads(Token, Vec<Item>),
  /// The call that reads on sits in a conditional's branch: whether it runs is TeX's to decide.
  Conditional,
  /// It reads no more than it is given: the body reads nothing further.
  Complete,
}

fn last_call(tokens: &[Token]) -> LastCall {
  let items = items(tokens);
  let single = |item: &Item| match item.tokens.as_slice() {
    [token] if !item.group && !item.bracket && token.get_catcode().is_active_or_cs() => {
      Some(*token)
    },
    _ => None,
  };
  // Conditionals open around the scan position: a call inside one reads on only in its branch.
  let mut conditionals = 0usize;
  let mut index = 0;
  while index < items.len() {
    let Some(call) = single(&items[index]) else {
      index += 1;
      continue;
    };
    let name = cs_name(&call);
    // The primitives that read their operands by hand, which their bindings' parameters do not
    // show (TeX's `\let` reads two tokens, `\def` a control sequence, its parameter text and a
    // group).
    let hand = match name.as_str() {
      "\\global" | "\\long" | "\\outer" | "\\protected" | "\\expandafter" | "\\relax" => Some(0),
      "\\let" => Some(2),
      "\\futurelet" => Some(3),
      "\\afterassignment" | "\\aftergroup" => Some(1),
      "\\def" | "\\edef" | "\\gdef" | "\\xdef" => {
        // The control sequence, then the parameter text up to and with the body's group.
        let body = items[index + 1..]
          .iter()
          .position(|item| item.group)
          .map(|at| at + 2);
        match body {
          Some(n) => Some(n),
          None => return LastCall::Conditional,
        }
      },
      _ => None,
    };
    if let Some(n) = hand {
      index += 1 + n;
      continue;
    }
    // The brace trick `\ifnum0=`{\fi}` (a group opened for TeX's alignment scanner only, the
    // conditional always false): bookkeeping, not a branch.
    if name == "\\ifnum" {
      let texts: Vec<String> = items[index + 1..]
        .iter()
        .take(4)
        .map(|item| text(&item.tokens))
        .collect();
      if texts.len() == 4
        && texts[..3] == ["0", "=", "`"]
        && items[index + 4].group
        && texts[3] == "\\fi"
      {
        index += 5;
        continue;
      }
    }
    if matches!(lookup_meaning(&call), Some(Stored::Conditional(_))) {
      match name.as_str() {
        "\\fi" => conditionals = conditionals.saturating_sub(1),
        "\\else" | "\\or" => {},
        _ => conditionals += 1,
      }
      index += 1;
      continue;
    }
    match consumes(&call, &items[index + 1..]) {
      // It reads on past the body's end: the tail. Inside a conditional it may not be taken.
      Some((consumed, true)) => {
        // Which branch reads on is TeX's to decide.
        return if conditionals > 0 {
          LastCall::Conditional
        } else {
          LastCall::Reads(call, items[index + 1..index + 1 + consumed].to_vec())
        };
      },
      Some((consumed, false)) => index += 1 + consumed,
      None => index += 1,
    }
  }
  LastCall::Complete
}

/// `body` with each `#k` replaced by the given item that fills parameter `k`; an unfilled one stays.
fn substitute(body: &[Token], filled: &[Option<usize>], supplied: &[Item]) -> Vec<Token> {
  let mut out = Vec::with_capacity(body.len());
  for token in body {
    let item = (token.get_catcode() == Catcode::ARG)
      .then(|| token.with_str(|s| s.parse::<usize>().ok()))
      .flatten()
      .and_then(|k| filled.get(k.checked_sub(1)?).copied().flatten())
      .and_then(|index| supplied.get(index));
    match item {
      Some(item) => out.extend(item.tokens.iter().copied()),
      None => out.push(*token),
    }
  }
  out
}

/// The last peek in `tokens` whose operands run exactly to its end (spaces aside): the body's last
/// act is to read the document with it. Returns its name and operands.
fn trailing_peek(tokens: &[Token]) -> Option<(String, &[Token])> {
  for (at, token) in tokens.iter().enumerate().rev() {
    if !token.get_catcode().is_active_or_cs() {
      continue;
    }
    let name = cs_name(token);
    let Some(&(_, wanted)) = PEEKS.iter().find(|(peek, _)| *peek == name) else {
      continue;
    };
    let operands = &tokens[at + 1..];
    let mut index = 0;
    if (0..wanted).all(|_| next_item(operands, &mut index, false).is_some())
      && operands[index..].iter().all(is_space)
    {
      return Some((name, &operands[..index]));
    }
  }
  None
}

/// The bracketed items a branch gives its head control sequence, in order.
fn given_brackets(branch: &[Token]) -> Vec<Vec<Token>> {
  let mut at = 0;
  let mut brackets = Vec::new();
  // Skip the head control sequence.
  if next_item(branch, &mut at, false).is_none() {
    return brackets;
  }
  while let Some((inner, bracket)) = next_item(branch, &mut at, true) {
    if bracket {
      // `[{x}]` gives `x`: one pair of braces protects the argument, as in TeX — when it encloses
      // the whole item (`[{a}{b}]` stays as written).
      let mut index = 0;
      let enclosing = inner
        .first()
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
        && next_item(&inner, &mut index, false).is_some()
        && inner[index..].iter().all(is_space);
      brackets.push(if enclosing {
        inner[1..inner.len() - 1].to_vec()
      } else {
        inner
      });
    }
  }
  brackets
}

/// The default of the optional argument a `[` peek guards: the bracketed item the absent branch
/// gives where the present branch leaves the document's (`\@ifnextchar[{\@imakebox[#1]}
/// {\@imakebox[#1][c]}` defaults to `c`). Unknown when the branches call different macros
/// (`\parbox`'s absent branch jumps to the last one with all defaults), or when the default is a
/// parameter of an intermediate macro.
fn given_default(present: &[Token], absent: &[Token]) -> Option<String> {
  let head = |branch: &[Token]| branch.iter().find(|t| !is_space(t)).copied();
  let (Some(p), Some(a)) = (head(present), head(absent)) else {
    return None;
  };
  if !p.get_catcode().is_active_or_cs() || cs_name(&p) != cs_name(&a) {
    return None;
  }
  given_brackets(absent)
    .into_iter()
    .nth(given_brackets(present).len())
    .filter(|default| default.iter().all(|t| t.get_catcode() != Catcode::ARG))
    .map(|default| text(&default))
}

/// A kernel peek and what follows it.
fn walk_peek(name: &str, rest: &[Token], pending: bool, depth: usize) -> (View, bool) {
  let mut view = View {
    chain: vec![name.to_string()],
    ..View::default()
  };
  let mut at = 0;
  let mut item = || next_item(rest, &mut at, false).map(|(tokens, _)| tokens);
  match name {
    "\\@testopt" | "\\@protected@testopt" => {
      let target = item();
      let target = if name == "\\@protected@testopt" {
        item()
      } else {
        target
      };
      let default = item();
      let (Some(target), Some(default)) = (target, default) else {
        view.notes.push(format!("{name} without its operands"));
        return (view, false);
      };
      view.args.push(Arg::Optional(Some(text(&default))));
      let (tail, _) = walk_list(&target, true, depth + 1);
      extend(&mut view, tail);
      (view, false)
    },
    "\\@ifstar" => {
      let (Some(present), Some(absent)) = (item(), item()) else {
        view
          .notes
          .push("\\@ifstar without its branches".to_string());
        return (view, false);
      };
      // The unstarred branch is the main form: `\section*` reads no `[short]`.
      view.args.push(Arg::Star);
      let (starred, _) = walk_list(&present, false, depth + 1);
      let (tail, consumed) = walk_list(&absent, pending, depth + 1);
      if starred.args != tail.args {
        view
          .notes
          .push(format!("starred form reads {:?}", starred.args));
      }
      extend(&mut view, tail);
      (view, consumed)
    },
    _ => {
      let (Some(peeked), Some(present), Some(absent)) = (item(), item(), item()) else {
        view.notes.push(format!("{name} without its operands"));
        return (view, false);
      };
      let peeked = text(&peeked);
      if peeked == "[" {
        let (with, _) = walk_list(&present, true, depth + 1);
        let (without, consumed) = walk_list(&absent, pending, depth + 1);
        view
          .args
          .push(Arg::Optional(given_default(&present, &absent)));
        if !with.args.ends_with(&without.args) {
          view
            .notes
            .push(format!("without `[`, reads {:?}", without.args));
        }
        extend(&mut view, with);
        (view, consumed)
      } else {
        // A peek for another token (`(` of the picture forms, `*`): the absent branch is the
        // main form.
        let (without, consumed) = walk_list(&absent, pending, depth + 1);
        view.args.push(if peeked == "*" {
          Arg::Star
        } else {
          Arg::Peek(peeked)
        });
        extend(&mut view, without);
        (view, consumed)
      }
    },
  }
}
