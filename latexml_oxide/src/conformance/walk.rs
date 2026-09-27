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
  T_BEGIN, T_CS, T_END, T_OTHER,
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

/// How long an expanded body may grow: a macro that hands its own items back to itself grows with
/// each level (tcolorbox's key handlers reached gigabytes before this bound).
const MAX_BODY: usize = 20_000;

/// The primitives that read their operands by hand, which their bindings' parameters do not show
/// (TeX's `\let` reads two tokens, `\def` a control sequence, its parameter text and a group):
/// how many given items each takes, `None` for the `\def` family (up to and with the body's group).
const HAND: [(&str, Option<usize>); 14] = [
  ("\\global", Some(0)),
  ("\\long", Some(0)),
  ("\\outer", Some(0)),
  ("\\protected", Some(0)),
  ("\\expandafter", Some(0)),
  ("\\relax", Some(0)),
  ("\\let", Some(2)),
  ("\\futurelet", Some(1)),
  ("\\afterassignment", Some(1)),
  ("\\aftergroup", Some(1)),
  ("\\def", None),
  ("\\edef", None),
  ("\\gdef", None),
  ("\\xdef", None),
];

/// The kernel's error reporters (latex.ltx `\GenericError` and its callers): a command that raises
/// one is invalid where the walk reads it (amsmath's `\intertext` outside an alignment), so it
/// says nothing about the arguments it takes where it is valid.
const ERRORS: [&str; 5] = [
  "\\GenericError",
  "\\PackageError",
  "\\ClassError",
  "\\@latex@error",
  "\\@latexerr",
];

/// The name of the primitive `token` means, or its own name: `\@xp` (`\let` to `\expandafter`) is
/// `\expandafter`.
fn primitive_name(token: &Token) -> String {
  match lookup_meaning(token) {
    Some(Stored::Primitive(primitive)) => cs_name(&primitive.get_cs()),
    Some(Stored::Conditional(conditional)) => cs_name(&conditional.get_cs()),
    _ => cs_name(token),
  }
}

/// Does `token` raise a LaTeX error: one of [`ERRORS`], or a macro whose body calls one at its top
/// level — outside any group or conditional (a check in a branch, `\ifx…\PackageError…\fi`, is a
/// validity test before the real work, as in fancyhdr's `\f@nch@fancyhf`).
fn raises_error(token: &Token) -> bool {
  if ERRORS.contains(&cs_name(token).as_str()) {
    return true;
  }
  let Some(Stored::Expandable(definition)) = lookup_meaning(token) else {
    return false;
  };
  let Some(ExpansionBody::Tokens(body)) = definition.get_expansion() else {
    return false;
  };
  let (mut groups, mut conditionals) = (0usize, 0usize);
  for t in body.unlist_ref() {
    match t.get_catcode() {
      Catcode::BEGIN => groups += 1,
      Catcode::END => groups = groups.saturating_sub(1),
      code if code.is_active_or_cs() => match lookup_meaning(t) {
        Some(Stored::Conditional(_)) => match primitive_name(t).as_str() {
          "\\fi" => conditionals = conditionals.saturating_sub(1),
          "\\else" | "\\or" => {},
          _ => conditionals += 1,
        },
        _ if groups == 0 && conditionals == 0 && ERRORS.contains(&cs_name(t).as_str()) => {
          return true;
        },
        _ => {},
      },
      _ => {},
    }
  }
  false
}

/// The tokens of a given item as the body wrote it: a group in its braces, a bracketed item in
/// its brackets.
fn item_tokens(item: &Item) -> Vec<Token> {
  let (open, close) = if item.group {
    (Some(T_BEGIN!()), Some(T_END!()))
  } else if item.bracket {
    (Some(T_OTHER!("[")), Some(T_OTHER!("]")))
  } else {
    (None, None)
  };
  open
    .into_iter()
    .chain(item.tokens.iter().copied())
    .chain(close)
    .collect()
}

/// The definition a parameterless wrapper hands over to: a robust command's
/// `\x@protect\cs\protect\cs␣` leads to `\cs␣`, a macro whose body is one control sequence to
/// it. The items a caller gives the wrapper are that definition's arguments.
fn resolve(token: &Token) -> Token {
  let mut token = *token;
  for _ in 0..MAX_DEPTH {
    let next = match lookup_meaning(&token) {
      Some(Stored::Expandable(d)) if d.get_parameters().is_none() => match d.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => {
          // `\x@protect\cs`, `\protect` and `\relax` before the call; nothing after it (a
          // `\y\relax` gives `\y` the `\relax`).
          let body = body.unlist_ref();
          let mut call = None;
          let mut at = 0;
          while let Some(t) = body.get(at) {
            at += 1;
            if is_space(t) {
              continue;
            }
            if call.is_some() || !t.get_catcode().is_active_or_cs() {
              call = None;
              break;
            }
            match cs_name(t).as_str() {
              "\\x@protect" => at += 1,
              "\\protect" | "\\relax" => {},
              _ => call = Some(*t),
            }
          }
          match call {
            Some(call) => call,
            None => break,
          }
        },
        _ => break,
      },
      _ => break,
    };
    token = next;
  }
  token
}

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
  let params = match lookup_meaning(&resolve(token))? {
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
          // (`\adl@hdashline\adl@ihdashline`) is followed where the body calls it. The given items
          // the parameters leave over follow the expansion, as in TeX: a robust wrapper's
          // `\cs␣` reads them (amsmath's `\dfrac` gives `\genfrac` four of its six).
          let given_tokens: usize = supplied.iter().map(|item| item.tokens.len()).sum();
          if body.unlist_ref().len() + given_tokens > MAX_BODY {
            view
              .notes
              .push(format!("{} expands past {MAX_BODY} tokens", cs_name(token)));
            return (view, consumed);
          }
          let unconsumed = read.as_ref().map_or(supplied.len(), |read| read.unconsumed);
          let filled = read.map(|read| read.filled).unwrap_or_default();
          let mut body = substitute(body.unlist_ref(), &filled, supplied);
          for item in &supplied[supplied.len() - unconsumed..] {
            body.extend(item_tokens(item));
          }
          if body.len() > MAX_BODY {
            view
              .notes
              .push(format!("{} expands past {MAX_BODY} tokens", cs_name(token)));
            return (view, consumed);
          }
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
      "\\__cmd_start:nNNnnn" | "\\__cmd_start_expandable:nNNNNn" => {
        let expandable = name.contains("expandable");
        let (tail, consumed) = walk_document_command(&tokens[at..], expandable, pending, depth);
        extend(&mut view, tail);
        return (view, consumed);
      },
      _ if PEEKS.iter().any(|(peek, _)| *peek == name) => {
        let (tail, consumed) = walk_peek(&name, &tokens[at..], pending, depth);
        extend(&mut view, tail);
        return (view, consumed);
      },
      _ if raises_error(&token) => {
        view.notes.push(format!("raises an error here ({name})"));
        return (view, false);
      },
      _ => {
        // The head: a tail call when it consumes all the rest of the list as its arguments. A
        // primitive that reads its operands by hand (`\def\x{…}`) is never one while the body
        // gives it them.
        let rest = &tokens[at..];
        let given = items(rest);
        let primitive = primitive_name(&token);
        let by_hand = HAND.iter().any(|(hand, _)| *hand == primitive);
        if given.is_empty()
          || (!by_hand
            && consumes(&token, &given).is_some_and(|(consumed, _)| consumed == given.len()))
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
        match last_call(&tokens[at - 1..], depth) {
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
          LastCall::Unknown(why) => {
            view.notes.push(why.to_string());
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
  /// What reads on cannot be read from the body (`\csname`), or the body raises an error here.
  Unknown(&'static str),
}

fn last_call(tokens: &[Token], depth: usize) -> LastCall {
  let items = items(tokens);
  let single = |item: &Item| match item.tokens.as_slice() {
    [token] if !item.group && !item.bracket && token.get_catcode().is_active_or_cs() => {
      Some(*token)
    },
    _ => None,
  };
  let named =
    |item: &Item, wanted: &str| single(item).is_some_and(|t| primitive_name(&t) == wanted);
  // Conditionals open around the scan position: a call inside one reads on only in its branch.
  let mut conditionals = 0usize;
  // The macros the body (re)defines before its last call mean what the body makes them, not what
  // the State holds now. A `\def`-family one: with parameters it reads (unknown what); without,
  // its text — a macro call in it may read on (amsmath's `\genfrac` ends by calling the `\@tempb`
  // it `\edef`s to a call of `\@genfrac`), plain text reads nothing (latex.ltx's `\@iiiparbox`
  // calls the `\@parboxto` it `\edef`s to `to<dimen>`). A `\let` one: its targets (arydshln's
  // `\@gtempa`, `\let` in each branch of a conditional; one outside any conditional replaces the
  // earlier ones). A `\futurelet` one: whatever TeX peeks.
  let mut defined: Vec<(String, Option<Vec<Token>>)> = Vec::new();
  let mut lets: Vec<(String, Token)> = Vec::new();
  // What runs later than the body says: a name built by `\csname`, a token saved by
  // `\aftergroup`/`\afterassignment`. If no later call reads on, it may be what does.
  let mut deferred: Option<&'static str> = None;
  let mut index = 0;
  while index < items.len() {
    let Some(call) = single(&items[index]) else {
      index += 1;
      continue;
    };
    let called = cs_name(&call);
    if let Some((_, text)) = defined.iter().rev().find(|(name, _)| *name == called) {
      let reads = text.as_ref().is_none_or(|text| {
        text.iter().any(|t| {
          t.get_catcode().is_active_or_cs()
            && (matches!(lookup_meaning(t), Some(Stored::Expandable(_)))
              || primitive_name(t) == "\\csname")
        })
      });
      if reads {
        return LastCall::Unknown("calls a macro its body defines");
      }
      index += 1;
      continue;
    }
    let targets: Vec<Token> = lets
      .iter()
      .filter(|(name, _)| *name == called)
      .map(|(_, target)| *target)
      .collect();
    let call = match targets.as_slice() {
      [] => call,
      [first, rest @ ..] => {
        // The targets must read the same arguments from the same given items — walked, so a
        // parameterless dispatch (`\let\next\relax` against `\let\next\peek`) is compared too.
        let reads = |target: &Token| {
          let (view, _) = walk_cs(target, &items[index + 1..], false, depth + 1);
          (view.args, view.complete)
        };
        let first_reads = reads(first);
        if rest.iter().any(|target| reads(target) != first_reads) {
          return LastCall::Unknown(
            "calls a macro its body `\\let`s to targets that read differently",
          );
        }
        *first
      },
    };
    let name = primitive_name(&call);
    if name == "\\csname" {
      deferred = Some("builds a control sequence by \\csname");
      index = after_endcsname(&items, index + 1);
      continue;
    }
    // An error raised outside any conditional: the command is invalid where the walk reads it.
    // (One in a branch is a validity check before the real work.)
    if conditionals == 0 && raises_error(&call) {
      return LastCall::Unknown("raises an error here");
    }
    if let Some(&(_, fixed)) = HAND.iter().find(|(hand, _)| *hand == name) {
      let mut at = index + 1;
      match name.as_str() {
        "\\let" | "\\futurelet" | "\\def" | "\\edef" | "\\gdef" | "\\xdef" => {
          // The control sequence assigned: one token, or a `\csname … \endcsname` run (whose
          // name the walk does not follow).
          let target = if items.get(at).is_some_and(|item| named(item, "\\csname")) {
            at = after_endcsname(&items, at + 1);
            None
          } else {
            at += 1;
            items.get(at - 1).and_then(single).map(|t| cs_name(&t))
          };
          match name.as_str() {
            "\\let" => {
              // `\let\x\y` or `\let\x=\y`.
              if items.get(at).is_some_and(|item| text(&item.tokens) == "=") {
                at += 1;
              }
              if let (Some(target), Some(meaning)) = (target, items.get(at).and_then(single)) {
                if conditionals == 0 {
                  lets.retain(|(name, _)| *name != target);
                }
                lets.push((target, meaning));
              }
              at += 1;
            },
            // `\futurelet\cs A B`: `\cs` is assigned, then A runs — A is the next call.
            "\\futurelet" => {
              if let Some(target) = target {
                defined.push((target, None));
              }
            },
            _ => {
              // The parameter text up to the body's group, then the group.
              let Some(group) = items
                .get(at..)
                .and_then(|rest| rest.iter().position(|item| item.group))
              else {
                return LastCall::Conditional;
              };
              if let Some(target) = target {
                let text = (group == 0).then(|| items[at].tokens.clone());
                defined.push((target, text));
              }
              at += group + 1;
            },
          }
        },
        "\\aftergroup" | "\\afterassignment" => {
          deferred = Some("saves a token for later");
          at += 1;
        },
        _ => at += fixed.unwrap_or(0),
      }
      index = at;
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
    let operands = consumes(&call, &items[index + 1..]);
    let is_delimiter = |item: &Item| {
      ["\\else", "\\fi", "\\or"]
        .iter()
        .any(|delimiter| named(item, delimiter))
    };
    // Given a conditional's `\else`/`\fi` as an operand, the call was reached by `\expandafter`
    // over it: its arguments come after the conditional, which only TeX resolves.
    if let Some((consumed, _)) = operands
      && items[index + 1..index + 1 + consumed]
        .iter()
        .any(is_delimiter)
    {
      return LastCall::Conditional;
    }
    match operands {
      // It reads on past the body's end: the tail. Inside a conditional it may not be taken.
      Some((consumed, true)) => {
        return if conditionals > 0 {
          LastCall::Conditional
        } else {
          LastCall::Reads(call, items[index + 1..index + 1 + consumed].to_vec())
        };
      },
      // The body's tail: its operands run to the end, so what its expansion reads beyond them
      // comes from the document, whether by parameters or by a peek (color.sty's `\pagecolor`
      // ends in `\color`, whose `\@ifnextchar[` reads on). Followed only by `\fi`s, it is a
      // branch's tail: which branch runs is TeX's to decide.
      Some((consumed, false)) if index + 1 + consumed == items.len() => {
        return if conditionals > 0 {
          LastCall::Conditional
        } else if let Some(why) = deferred {
          // After a deferred token (`\aftergroup\x\endgroup`) the tail may not be what reads.
          LastCall::Unknown(why)
        } else {
          LastCall::Reads(call, items[index + 1..].to_vec())
        };
      },
      Some((consumed, false))
        if items[index + 1 + consumed..]
          .iter()
          .all(|item| named(item, "\\fi")) =>
      {
        return LastCall::Conditional;
      },
      Some((consumed, false)) => index += 1 + consumed,
      None => index += 1,
    }
  }
  match deferred {
    Some(why) => LastCall::Unknown(why),
    None => LastCall::Complete,
  }
}

/// The index just past the `\endcsname` that closes a `\csname` run starting at `at`.
fn after_endcsname(items: &[Item], mut at: usize) -> usize {
  while let Some(item) = items.get(at) {
    at += 1;
    if matches!(item.tokens.as_slice(), [t] if !item.group && t.get_catcode().is_active_or_cs()
      && primitive_name(t) == "\\endcsname")
    {
      break;
    }
  }
  at
}

/// The arguments `token` reads after the given `following` items, by its parameters (a
/// parameterless wrapper by the definition it hands over to): what two `\let` targets are compared
/// by. `None` when it is not a definition with parameters.
fn reads_of(token: &Token, following: &[Item]) -> Option<Vec<Arg>> {
  let params = match lookup_meaning(&resolve(token))? {
    Stored::Expandable(d) => d.get_parameters().cloned(),
    Stored::Primitive(d) => d.get_parameters().cloned(),
    Stored::Constructor(d) => d.get_parameters().cloned(),
    _ => return None,
  }?;
  let texts: Vec<String> = following.iter().map(|item| text(&item.tokens)).collect();
  let mut args = Vec::new();
  args_of(&params, &givens(&texts, following), false, &mut args);
  Some(args)
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

/// A command made by `\NewDocumentCommand` (ltcmd, latex.ltx `\__cmd_start:nNNnnn`): its operands
/// are the argument spec, the command, its code macro, the grabbers ltcmd normalized the spec to
/// (`\__cmd_grab_t:w *`, `\__cmd_grab_D:w []`, `\__cmd_grab_m_1:w`, …) and the defaults, one per
/// argument (`\c_novalue_tl`, or `{\prg_do_nothing: x}` for `O{x}`). The arguments are the
/// grabbers'; the prologue is the code macro's, walked with every argument given. An expandable
/// command (`\__cmd_start_expandable:nNNNNn`, ltcmd's form when every argument can be grabbed by
/// expansion) has the spec, the command twice, its code macro, a marker and the grabbers
/// (`\__cmd_expandable_grab_m:w`, …).
fn walk_document_command(
  rest: &[Token],
  expandable: bool,
  pending: bool,
  depth: usize,
) -> (View, bool) {
  let start = if expandable {
    "\\__cmd_start_expandable:nNNNNn"
  } else {
    "\\__cmd_start:nNNnnn"
  };
  let mut view = View {
    chain: vec![start.to_string()],
    ..View::default()
  };
  let operands = items(rest);
  let (code, grabbers, defaults) = if expandable {
    (operands.get(3), operands.get(5), None)
  } else {
    (operands.get(2), operands.get(3), operands.get(4))
  };
  let (Some(code), Some(grabbers)) = (code, grabbers) else {
    view.notes.push(format!("{start} without its operands"));
    return (view, false);
  };
  let defaults = defaults.map(|item| items(&item.tokens)).unwrap_or_default();
  let default_of = |index: usize| {
    defaults.get(index).and_then(|item| {
      let tokens: Vec<Token> = item
        .tokens
        .iter()
        .copied()
        .filter(|t| !(t.get_catcode().is_active_or_cs() && cs_name(t) == "\\prg_do_nothing:"))
        .collect();
      (item.group).then(|| text(&tokens).trim().to_string())
    })
  };
  // The grabbers and their operands, one token or group each (`\__cmd_grab_D:w []` is two tokens).
  let mut operands_of = {
    let tokens = grabbers.tokens.clone();
    let mut at = 0;
    move || next_item(&tokens, &mut at, false).map(|(tokens, _)| tokens)
  };
  let mut count = 0usize;
  while let Some(grabber) = operands_of() {
    let [token] = grabber.as_slice() else {
      continue;
    };
    if !token.get_catcode().is_active_or_cs() {
      continue;
    }
    let name = cs_name(token);
    let Some(kind) = name
      .strip_prefix("\\__cmd_grab_")
      .or_else(|| name.strip_prefix("\\__cmd_expandable_grab_"))
      .and_then(|kind| kind.strip_suffix(":w"))
    else {
      continue;
    };
    // The expandable grabbers put a generated helper macro before their delimiters (latex.ltx
    // `\__cmd_add_expandable_type_D_aux:NNN`, `…_t:w`); an `_alt` one has a single delimiter
    // (opening and closing alike).
    let alt = kind.contains("_alt");
    if expandable && matches!(kind.split('_').next(), Some("D" | "R" | "t")) {
      operands_of();
    }
    let mut operand = || {
      operands_of()
        .map(|tokens| text(&tokens))
        .unwrap_or_default()
    };
    match kind.split('_').next().unwrap_or_default() {
      "m" => {
        // `m`, or ltcmd's run of mandatory arguments `\__cmd_grab_m_3:w`.
        let n = kind
          .rsplit('_')
          .next()
          .and_then(|n| n.parse().ok())
          .unwrap_or(1);
        view.args.extend(std::iter::repeat_n(Arg::Mandatory, n));
        count += n;
      },
      "t" => {
        let token = operand();
        view.args.push(if token == "*" {
          Arg::Star
        } else {
          Arg::Peek(token)
        });
        count += 1;
      },
      "D" => {
        let open = operand();
        let close = if alt { open.clone() } else { operand() };
        view.args.push(if open == "[" && close == "]" {
          Arg::Optional(default_of(count))
        } else {
          Arg::Peek(open)
        });
        count += 1;
      },
      "G" => {
        view.args.push(Arg::Peek("{".to_string()));
        count += 1;
      },
      "R" => {
        let open = operand();
        let close = if alt { open.clone() } else { operand() };
        view.args.push(Arg::Literal(format!("{open}…{close}")));
        count += 1;
      },
      "u" => {
        // The expandable form's only operand is a helper macro whose last delimited parameter is
        // the delimiter (xparse.sty `\__cmd_add_expandable_type_u:w`; `l` is `u` with `#{`).
        let delimiter = if expandable {
          operands_of()
            .and_then(|helper| helper.first().copied())
            .and_then(|helper| reads_of(&helper, &[]))
            .and_then(|args| {
              args.into_iter().rev().find_map(|arg| match arg {
                Arg::Delimited(delimiter) => Some(delimiter),
                _ => None,
              })
            })
        } else {
          Some(operand())
        };
        let Some(delimiter) = delimiter else {
          view.notes.push(format!("unread ltcmd grabber {name}"));
          return (view, false);
        };
        view.args.push(Arg::Delimited(delimiter));
        count += 1;
      },
      "l" => {
        view.args.push(Arg::Delimited("{".to_string()));
        count += 1;
      },
      "E" => {
        // Embellishments: one optional argument per token, each peeked for (the expandable form
        // pairs each token with its helper macro).
        let tokens = operands_of().unwrap_or_default();
        let peeked: Vec<Token> = if expandable {
          tokens.iter().copied().skip(1).step_by(2).collect()
        } else {
          tokens.iter().copied().filter(|t| !is_space(t)).collect()
        };
        count += peeked.len();
        view.args.push(Arg::Peek(text(&peeked)));
      },
      "v" => {
        view.args.push(Arg::Mandatory);
        count += 1;
      },
      _ => {
        view.notes.push(format!("unread ltcmd grabber {name}"));
        return (view, false);
      },
    }
  }
  // The prologue, and whatever the code reads on: the code macro, every argument given.
  let [code] = code.tokens.as_slice() else {
    view.notes.push(format!("{start} without its code macro"));
    return (view, false);
  };
  let given = vec![
    Item {
      tokens:  Vec::new(),
      bracket: false,
      group:   true,
    };
    count
  ];
  // What the code reads beyond the grabbed arguments comes from the document (hyperref's `{s}`
  // `\autoref` reads its label in its code's tail).
  let (body, consumed) = walk_cs(code, &given, pending, depth + 1);
  extend(&mut view, body);
  (view, consumed)
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
