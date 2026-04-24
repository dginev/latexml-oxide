use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashSet as HashSet;
use std::cell::{RefCell, RefMut};
use std::collections::VecDeque;
// use std::mem;
// use std::rc::Rc;

use crate::alignment::Alignment;
use crate::common::arena::{self, SymStr};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::{FillCode, Glue};
use crate::common::locator::Locator;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::{NumericOps, UNITY, fixpoint};
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::state::*;
use crate::{DigestedData, state};

use crate::definition::Definition;
use crate::definition::conditional::ConditionalType;
use crate::definition::register::{Register, RegisterType, RegisterValue};
use crate::mouth::Mouth;
use crate::token::{Catcode, TOKEN_ENDCSNAME, TOKEN_RELAX, Token};
use crate::tokens::Tokens;

static DIGIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9]").unwrap());
static OCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-7]").unwrap());
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-F]").unwrap());

// Perl smuggles the unexpanded token inside \special_relax's slot [2].
// Rust Token is Copy+Clone with no extra slot, so we use a thread-local Cell.
use crate::pin;
use std::cell::Cell;
#[thread_local]
static SPECIAL_RELAX_SMUGGLED: Cell<Option<Token>> = Cell::new(None);

/// Store the unexpanded token smuggled inside \special_relax (Perl: $$special_relax[2])
fn set_special_relax_smuggled(token: Token) { SPECIAL_RELAX_SMUGGLED.set(Some(token)); }
/// Retrieve (and clear) the smuggled token from the last \special_relax
pub fn take_special_relax_smuggled() -> Option<Token> { SPECIAL_RELAX_SMUGGLED.take() }
/// Peek at the smuggled token without consuming it
fn peek_special_relax_smuggled() -> Option<Token> { SPECIAL_RELAX_SMUGGLED.get() }
/// Check if a token is \special_relax and its smuggled unexpanded token matches `target`
fn special_relax_matches(token: &Token, target: &Token) -> bool {
  token.code == Catcode::CS
    && token.text == pin!("\\special_relax")
    && peek_special_relax_smuggled().as_ref() == Some(target)
}
#[thread_local]
static DEFERRED_COMMANDS: Lazy<HashSet<SymStr>> = Lazy::new(|| {
  set!(
    arena::pin_static("\\the"),
    arena::pin_static("\\showthe"),
    arena::pin_static("\\unexpanded"),
    arena::pin_static("\\detokenize")
  )
});

// If it is a column ending token, Returns the token, a keyword and whether it is "hidden"
#[thread_local]
static COLUMN_ENDS: Lazy<[(Token, &'static str, bool); 6]> = Lazy::new(|| {
  [
    // besides T_ALIGN
    (T_CS!("\\cr"), "cr", false),
    (T_CS!("\\crcr"), "crcr", false),
    (T_CS!("\\lx@hidden@cr"), "cr", true),
    (T_CS!("\\lx@hidden@crcr"), "crcr", true),
    (T_CS!("\\lx@hidden@align"), "insert", true),
    (T_CS!("\\span"), "span", false),
  ]
});

#[derive(PartialEq, Debug)]
pub struct MouthRuntime {
  pub autoclose: bool,
  pub mouth:     Mouth,
  /// Pushback LIFO stack: the "next to read" token is at `pushback.last()`.
  /// Invariant: reading pops from the back; `unread_one` pushes to the back;
  /// `unread_vec` iterates its input in reverse and pushes each — so the
  /// first element of an unread Vec ends up on top (= next to read).
  /// See `flush_mouth` for the rare FIFO-prepend semantics (\endinput).
  ///
  /// Previously a `VecDeque<Token>` — switched to a plain Vec because the
  /// hot-path is pure LIFO and VecDeque's push_front/pop_front machinery
  /// (head-pointer + wrap arithmetic) showed up at ~3.3% of total Ir in
  /// callgrind on siunitx-heavy fixtures.
  pub pushback:  Vec<Token>,
}

#[derive(Debug, Default)]
pub struct Gullet {
  pub runtime:          Option<MouthRuntime>,
  pub mouthstack:       VecDeque<MouthRuntime>,
  pub pending_comments: VecDeque<Token>,
  pub token_limit:      Option<usize>,
  pub pushback_limit:   Option<usize>,
  pub progress:         usize,
}

#[thread_local]
pub static GULLET: Lazy<RefCell<Gullet>> = Lazy::new(|| {
  RefCell::new(Gullet {
    // Safety limit: prevents infinite loops from corrupted macro state.
    // A typical LaTeX document with expl3 processes ~5M tokens. TikZ documents
    // with complex figures can reach 30-80M tokens due to pgf's TeX-level math engine.
    // Papers with 30+ tikzpictures (e.g., graph theory) need ~80M.
    token_limit: Some(100_000_000),
    ..Gullet::default()
  })
});

macro_rules! gullet {
  () => {
    (*GULLET).borrow()
  };
}
macro_rules! gullet_mut {
  () => {
    (*GULLET).borrow_mut()
  };
}
/// Set the token limit and reset progress. Returns previous (limit, progress) for restoration.
pub fn set_token_limit(limit: Option<usize>) -> (Option<usize>, usize) {
  let mut g = gullet_mut!();
  let prev = (g.token_limit, g.progress);
  g.token_limit = limit;
  g.progress = 0;
  prev
}

/// Set the pushback limit (maximum pushback stack size before fatal error).
pub fn set_pushback_limit(limit: Option<usize>) { gullet_mut!().pushback_limit = limit; }

/// Restore the token limit and progress from a previous set_token_limit call.
pub fn restore_token_limit(saved: (Option<usize>, usize)) {
  let mut g = gullet_mut!();
  g.token_limit = saved.0;
  g.progress = saved.1;
}

macro_rules! runtime {
  () => {
    (*GULLET).borrow_mut().runtime
  };
}
macro_rules! runtime_mut {
  () => {
    (*GULLET).borrow_mut().runtime.as_mut()
  };
}

/// Initialize (or reset, if reentrant) a Gullet to its default empty state
pub fn initialize_gullet() {
  let mut gullet = gullet_mut!();
  gullet.runtime = None;
  gullet.mouthstack = VecDeque::new();
  gullet.pending_comments = VecDeque::new();
  // Reset smuggled token from previous conversion
  SPECIAL_RELAX_SMUGGLED.set(None);
}

/// Get the current location of input getting read
pub fn get_locator() -> Locator {
  let gullet = gullet!();
  let mut runtime_opt = gullet.runtime.as_ref();
  let mut mouthstack_iter = gullet.mouthstack.iter();
  while runtime_opt.is_some() && runtime_opt.as_ref().unwrap().mouth.get_source().is_empty() {
    runtime_opt = mouthstack_iter.next();
  }
  if let Some(runtime) = runtime_opt {
    // First exit condition: we found a mouth with a source, and asked it for a locator
    runtime.mouth.get_locator()
  } else if let Some(runtime) = gullet.mouthstack.front() {
    // Backup strategy: return the first locator in the mouthstack:
    runtime.mouth.get_locator()
  } else {
    // Final backup -- the default locator
    // TODO: Or should this be None?
    Locator::default()
  }
}

/// Comment-oriented location string, based on `get_locator`
pub fn get_location() -> String {
  let loc = get_locator();
  s!("at {}", loc)
}

pub fn mouth_is_open(mouth: &Mouth) -> bool {
  let gullet = gullet!();
  if let Some(ref runtime) = gullet.runtime {
    if mouth == &runtime.mouth {
      return true;
    }
  }
  gullet
    .mouthstack
    .iter()
    .any(|runtime| &runtime.mouth == mouth)
}

/// Push the `tokens` back into the input stream to be re-read.
pub fn unread(tokens: Tokens) { unread_vec(tokens.unlist()); }
/// Variant of `unread`, but drains the contents of `tokens` without taking ownership.
pub fn unread_mut(tokens: &mut Tokens) {
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    // Iterate in reverse and push to the stack top — the first element
    // of `tokens` ends up on top (= next to read). Same semantics as
    // the old VecDeque push_front pattern.
    for token in tokens.unlist_mut().drain(..).rev() {
      runtime.pushback.push(token);
    }
  };
}
/// Unreads a single `Token` to the start of the token stream.
/// Perl: unread() always adjusts $ALIGN_STATE when unreading { or } tokens.
pub fn unread_one(token: Token) {
  match token.get_catcode() {
    Catcode::BEGIN => decrement_align_group_count(), // Retract scanned brace
    Catcode::END => increment_align_group_count(),
    _ => {},
  }
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    runtime.pushback.push(token);
  };
}
/// Unreads a `Vec<Token>` to the start of the token stream
/// Perl: also adjusts ALIGN_STATE by retracting scanned braces (Gullet.pm lines 343-358)
pub fn unread_vec(tokens: Vec<Token>) {
  let mut level: i64 = 0;
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    // Reserve once, push each token in reverse-iteration order so the
    // first element of `tokens` ends up at the stack top. Same
    // semantics as the old VecDeque push_front loop, but without
    // per-element head-pointer arithmetic.
    runtime.pushback.reserve(tokens.len());
    for token in tokens.into_iter().rev() {
      match token.get_catcode() {
        Catcode::BEGIN => level -= 1, // Retract scanned braces
        Catcode::END => level += 1,
        _ => {},
      }
      runtime.pushback.push(token);
    }
  }
  if level != 0 {
    set_align_group_count(align_group_count() + level as i32);
  }
}

//**********************************************************************
// Start reading tokens from a new Mouth.
// This pushes the mouth as the current source that $gullet->readToken (etc) will read from.
// Once this Mouth has been exhausted, readToken, etc, will return undef,
// until you call $gullet->closeMouth to clear the source.
// Exception: if $toplevel=1, readXToken will step to next source
// Note that a Tokens can act as a Mouth.
pub fn open_mouth(mouth: Mouth, autoclose: bool) {
  let mut gullet = gullet_mut!();
  if let Some(runtime) = gullet.runtime.take() {
    gullet.mouthstack.push_front(runtime);
  };
  gullet.runtime = Some(MouthRuntime {
    mouth,
    autoclose,
    pushback: Vec::with_capacity(128),
  });
}

pub fn close_mouth(forced: bool) -> Result<()> {
  let mut shift_from_mouthstack = false;
  let mut error_has_more_input = false;
  if let Some(ref mut runtime) = runtime!() {
    if !forced && (!runtime.pushback.is_empty() || runtime.mouth.has_more_input()) {
      error_has_more_input = true
    }
  }
  if error_has_more_input {
    let next = match read_token()? {
      Some(t) => t.stringify(),
      None => String::from("Empty"),
    };
    let message = s!("Closing mouth with input remaining '{}'", next);
    Error!("unexpected", next, message);
  }
  let mut gullet = gullet_mut!();
  if let Some(ref mut runtime) = gullet.runtime {
    runtime.mouth.finish();
    shift_from_mouthstack = true;
  }
  if shift_from_mouthstack {
    gullet.runtime = gullet.mouthstack.pop_front();
  }
  Ok(())
}
/// This flushes a mouth so that it will be automatically closed, next time it's read
/// Corresponds to TeX's \endinput
pub fn flush_mouth() {
  if let Some(ref mut runtime) = runtime!() {
    // Collect remaining mouth tokens in mouth order (t1, t2, t3, …),
    // then splice them into the stack's BOTTOM in reverse order so
    // that after the stack's existing top is popped, the mouth tokens
    // come out in the original mouth order (t1 first, then t2, …).
    let mut trailer: Vec<Token> = Vec::new();
    while !runtime.mouth.is_eol() {
      if let Some(token) = runtime.mouth.read_token() {
        trailer.push(token);
      }
    }
    if !trailer.is_empty() {
      trailer.reverse();
      runtime.pushback.splice(0..0, trailer);
    }
    // Stop reading (clear buffers, close file) but do NOT restore catcodes.
    // Catcodes are restored by close_mouth → finish() when the mouth is
    // properly popped from the stack.
    runtime.mouth.stop_reading();
  }
}

//**********************************************************************
// Low-level readers: read token, read expanded token
//**********************************************************************
// # Get the next pending comment token (if any)
pub fn get_pending_comment() -> Option<Token> { gullet_mut!().pending_comments.pop_front() }

/// Note that every char (token) comes through here (maybe even twice, through args parsing),
/// So, be Fast & Clean!  This method only reads from the current input stream (Mouth).
fn handle_template(
  mut alignment: RefMut<Alignment>,
  token: Token,
  vtype: &str,
  hidden: bool,
) -> Result<()> {
  //  Append expansion to end!?!?!?!
  local_current_token(token);
  let post = alignment.get_column_after();
  set_align_group_count(1000000);
  // ### NOTE: Truly fishy smuggling w/ \lx@hidden@cr
  let arg_opt = if (vtype == "cr") && hidden {
    // \lx@hidden@cr gets an argument as payload!!!!!
    Some(read_arg(ExpansionLevel::Off)?)
  } else {
    None
  };
  // eprintln!("Halign: column after {post}");// . ToString($post) if $LaTeXML::DEBUG{halign};
  if (vtype == "cr" || vtype == "crcr")
    && alignment.is_in_row()
    && !alignment
      .current_row()
      .map(|v| v.is_pseudo())
      .unwrap_or(false)
  {
    unread_one(T_CS!("\\lx@alignment@row@after"));
  }
  if let Some(arg) = arg_opt {
    // slippery - to unread {arg} we first unread } then arg then {, as we push to the front.
    unread_one(T_END!());
    unread(arg);
    unread_one(T_BEGIN!());
  }
  unread_one(token);
  unread(post);
  expire_current_token();
  Ok(())
}

// internal low-level reader that extracts a token from a mouth,
// but always keeps comment tokens pending.
fn read_internal_token() -> Option<Token> {
  let mut next_token = None;
  let Gullet {
    ref mut runtime,
    ref mut pending_comments,
    ..
  } = *gullet_mut!();
  let pushback = &mut runtime.as_mut().unwrap().pushback;
  // Check in pushback first....
  while let Some(pushback_token) = pushback.pop() {
    match pushback_token.get_catcode() {
      Catcode::COMMENT => pending_comments.push_back(pushback_token),
      Catcode::MARKER => handle_marker(pushback_token),
      _ => {
        next_token = Some(pushback_token);
        break;
      },
    };
  }
  // Not in pushback, read from the current Mouth
  if next_token.is_none() {
    while let Some(token) = runtime.as_mut().unwrap().mouth.read_token() {
      match token.get_catcode() {
        Catcode::COMMENT => pending_comments.push_back(token),
        Catcode::MARKER => handle_marker(token),
        _ => {
          next_token = Some(token);
          break;
        },
      };
    }
  }
  next_token
}

pub fn read_token() -> Result<Option<Token>> {
  let mut next_token: Option<Token>;
  loop {
    // Defensive checks: combine into a single mutable borrow to avoid
    // the previous immutable→drop→mutable borrow dance. Also skip the
    // pushback_limit probe entirely when no limit is set (the default
    // for normal conversion runs).
    {
      let mut g = gullet_mut!();
      if g.runtime.is_none() {
        return Ok(None);
      }
      if let Some(limit) = g.token_limit {
        g.progress += 1;
        if g.progress > limit {
          let msg = s!("Token limit of {} exceeded, infinite loop?", limit);
          drop(g);
          Fatal!(Timeout, TokenLimit, msg);
        }
      }
      if let Some(limit) = g.pushback_limit {
        let pb_len = g.runtime.as_ref().map(|r| r.pushback.len()).unwrap_or(0);
        if pb_len > limit {
          let msg = s!("Pushback limit of {} exceeded, infinite loop?", limit);
          drop(g);
          Fatal!(Timeout, PushbackLimit, msg);
        }
      }
    }
    // internal low-level reader that extracts a token from a mouth,
    // but always keeps comment tokens pending.
    next_token = read_internal_token();
    // ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;

    // Wow!!!!! See TeX the Program \S 309
    // Perl: alignment check → dont_expand check → else break
    // ALIGN_STATE tracking happens AFTER the loop (Perl L320-324)
    if let Some(ref nextt) = next_token {
      // NOTE: Perl tracks { and } OUTSIDE the loop (after break), but in Rust
      // we track here BEFORE checks. This is an intentional divergence:
      // moving tracking after the loop causes expl3 kernel loading to proceed
      // further into problematic modules (fp). The pre-check tracking prevents
      // alignment triggers on { tokens (count becomes 1 before the check).
      // Both orderings produce the same result for non-alignment tokens.
      match nextt.get_catcode() {
        Catcode::BEGIN => increment_align_group_count(),
        Catcode::END => decrement_align_group_count(),
        _ => {},
      }
      if (align_group_count() == 0) && has_reading_alignment() {
        if let Some((atoken, atype, ahidden)) = is_column_end(nextt) {
          let reading_alignment = get_reading_alignment().unwrap();
          if let DigestedData::Alignment(data) = reading_alignment.data() {
            handle_template(data.borrow_mut(), atoken, atype, ahidden)?;
          } else {
            return Err("reading_alignment should always contain DigestedData::Alignment".into());
          }
          continue; // Perl: handleTemplate then continue while(1) loop
        }
      }
      if nextt.code == Catcode::CS && nextt.text == pin!("\\dont_expand") {
        let unexpanded = read_token()?;
        // Perl: smuggle the unexpanded token in the "meaning" slot of \special_relax
        if let Some(tok) = unexpanded {
          set_special_relax_smuggled(tok);
        }
        next_token = Some(T_CS!("\\special_relax"));
      }
      break;
    } else {
      break;
    }
  }
  Ok(next_token)
}

/// Read the next non-expandable token (expanding tokens until there's a non-expandable one).
///
/// Note that most tokens pass through here, so be Fast & Clean! readToken is folded in.
///    `Toplevel' processing, (if `toplevel` is true), used at the toplevel processing by Stomach,
///     will step to the next input stream (Mouth) if one is available,
///     `toplevel` when true:
/// * If a mouth is exhausted, move on to the containing mouth to continue reading `fully_expand`
///   when true, OR when None but `toplevel` is true
/// * expand even protected defns, essentially this means expand "for execution"
///
/// Note that, unlike readBalanced, this does NOT defer expansion of \the & friends.
///
/// Also, \noexpand'd tokens effectively act ilke \relax
///
/// For arguments to \if,\ifx, etc use `for_conditional` true,
///    which handles \noexpand and CS which have been \let to tokens specially.
pub fn read_x_token(
  toplevel_opt: Option<bool>,
  for_conditional: bool,
  fully_expand_opt: Option<bool>,
) -> Result<Option<Token>> {
  // toplevel should be true by default
  let toplevel = toplevel_opt.unwrap_or(true);
  let autoclose = toplevel;
  let fully_expand = fully_expand_opt.unwrap_or(toplevel);
  loop {
    // internal low-level reader that extracts a token from a mouth,
    // but always keeps comment tokens pending.
    let next_token = read_internal_token();
    //ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;
    if next_token.is_none() {
      {
        let gullet = gullet!();
        if !autoclose
          || !gullet
            .runtime
            .as_ref()
            .map(|r| r.autoclose)
            .unwrap_or(false)
          || gullet.mouthstack.is_empty()
        {
          return Ok(None);
        }
      }
      close_mouth(false)?; // Next input stream.
      continue;
    }
    // we got a token
    let token = next_token.unwrap();
    if token.get_catcode() == Catcode::CS && token.text == pin!("\\dont_expand") {
      let unexpanded = match read_token()? {
        Some(t) => t,
        None => return Ok(Some(T_CS!("\\special_relax"))), // \dont_expand at end-of-input
      };
      if for_conditional && unexpanded.code == Catcode::ACTIVE {
        return Ok(Some(unexpanded));
      } else {
        // Perl: smuggle the unexpanded token in \special_relax
        set_special_relax_smuggled(unexpanded);
        return Ok(Some(T_CS!("\\special_relax")));
      }
    }
    // Wow!!!!! See TeX the Program \S 309
    // SHOULD count nesting of { }!!! when SCANNED (not digested)
    let check_alignment_data = {
      if has_reading_alignment() && align_group_count() == 0 {
        if let Some((_atoken, atype, ahidden)) = is_column_end(&token) {
          let reading_alignment = get_reading_alignment().unwrap();
          Some((reading_alignment, atype, ahidden))
        } else {
          None
        }
      } else {
        None
      }
    };
    if let Some((reading_alignment, atype, ahidden)) = check_alignment_data {
      if let DigestedData::Alignment(data) = reading_alignment.data() {
        handle_template(data.borrow_mut(), token, atype, ahidden)?;
      } else {
        panic!("malformed alignmed was stored?");
      }
      // And *then* continue the main loop checks
    } else if token.get_catcode().is_active_or_cs() {
      // Read the meaning via closure so we can branch on the borrowed
      // Stored without cloning (Stored::clone was ~1% of total on
      // siunitx-heavy profiles; this site fires on every CS/ACTIVE
      // expansion — the hottest lookup_meaning caller).
      enum Outcome {
        LetTo(Token),
        Undefined,
        NonExpandable,
        Invoke(std::rc::Rc<dyn crate::definition::Definition>),
      }
      let outcome = state::with_meaning(&token, |defn_opt| match defn_opt {
        Some(Stored::Token(t)) => Outcome::LetTo(*t),
        Some(Stored::None) | None => Outcome::Undefined,
        Some(other) => match other.to_definition() {
          Some(defn) => {
            if !defn.is_expandable() || (defn.is_protected() && !fully_expand) {
              Outcome::NonExpandable
            } else {
              Outcome::Invoke(defn)
            }
          },
          None => Outcome::Undefined,
        },
      });
      match outcome {
        Outcome::LetTo(let_token) => {
          return Ok(Some(if for_conditional { let_token } else { token }));
        },
        Outcome::Undefined => {
          if token.get_catcode() == Catcode::CS {
            return Ok(Some(generate_error_stub(&token)?));
          } else {
            return Ok(Some(token));
          }
        },
        Outcome::NonExpandable => {
          return Ok(Some(token));
        },
        Outcome::Invoke(defn) => {
          local_current_token(token);
          let invoked = defn.invoke(false)?;
          unread(invoked);
          expire_current_token();
          continue;
        },
      }
    } else {
      // Perl Gullet.pm L421-422: track { and } at scan level for ALIGN_STATE
      match token.get_catcode() {
        Catcode::BEGIN => increment_align_group_count(),
        Catcode::END => decrement_align_group_count(),
        _ => {},
      }
      return Ok(Some(token));
    }
  }
}

/// Read the next raw line (string);
/// primarily to read from the Mouth, but keep any unread input!
pub fn read_raw_line() -> Option<String> {
  // If we've got unread tokens, they presumably should come before the Mouth's raw data
  // but we'll convert them back to string.
  let mut gullet = gullet_mut!();
  if let Some(ref mut runtime) = gullet.runtime {
    // Vec-as-stack stores bottom-to-top, but the caller expects
    // "next to read" first — reverse the drained order to match the
    // old VecDeque drain(..) which was front-to-back (= next-to-read).
    let tokens: Vec<Token> = runtime.pushback.drain(..).rev().collect();

    // TODO
    // let markers : Vec<&Token> = tokens.iter().filter(|t:Token| t.get_catcode() ==
    // Catcode::MARKER).collect(); if !markers.is_empty() {    // Whoops, profiling markers!

    // @tokens = grep { $_->getCatcode != Catcode::MARKER } @tokens;    // Remove
    // map { LaTeXML::Core::Definition::stopProfiling($_, 'expand') } @markers;
    // }

    // If we still have peeked tokens, we ONLY want to combine it with the remainder
    // of the current line from the Mouth (NOT reading a new line)
    if !tokens.is_empty() {
      Some(Tokens::new(tokens).to_string() + &runtime.mouth.read_raw_line(true).unwrap_or_default())
    } else {
      // Otherwise, read the next line from the Mouth.
      runtime.mouth.read_raw_line(false)
    }
  } else {
    None
  }
}

//**********************************************************************
// Mid-level readers: checking and matching tokens, strings etc.
//**********************************************************************
// The following higher-level parsing methods are built upon readToken & `.

/// Read a single non-space token
pub fn read_non_space() -> Result<Option<Token>> {
  loop {
    match read_token()? {
      None => return Ok(None),
      Some(t) => {
        if t.get_catcode() != Catcode::SPACE {
          return Ok(Some(t));
        }
      },
    }
  }
}

/// Read a single expanded, non-space, token
pub fn read_x_non_space() -> Result<Option<Token>> {
  loop {
    match read_x_token(Some(false), false, None)? {
      None => return Ok(None),
      Some(t) => {
        if t.get_catcode() != Catcode::SPACE {
          return Ok(Some(t));
        }
      },
    }
  }
}

/// A directive describing to what degree a gullet reader should perform TeX's expansion
#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum ExpansionLevel {
  // No expansion, reads currently present tokens
  #[default]
  Off,
  /// Expands while reading, but deferring `\the` and `\protected`
  Partial,
  /// Expands completely while reading
  Full,
}

/// Approximates TeX's scan_toks (but doesn't parse \def parameter lists)
/// and only optionally requires the openning "{".
///
/// It may return comments in the token lists.
/// The `is_macrodef` flag affects whether # parameters are "packed" for macro bodies.
/// If `require_open` is true, the opening T_BEGIN has not yet been read, and is required.
///
/// If `toplevel` is true, it will automatically close empty mouths as it reads,
/// and will also fully expand macros (unless overridden by `expansion_level` being explicitly Off).
pub fn read_balanced(
  expansion_level: ExpansionLevel,
  is_macrodef: bool,
  require_open: bool,
) -> Result<Tokens> {
  use ExpansionLevel::*;
  if !require_open {
    decrement_align_group_count();
  }
  local_align_group_count(1000000);
  // let startloc = if lookup_verbosity() > 0 { Some(get_locator()) } else { None };
  // Do we need to expand to get the { ???
  if require_open {
    let token_opt = if expansion_level != Off {
      read_x_token(Some(false), false, None)?
    } else {
      read_token()?
    };
    let is_open = match token_opt {
      None => false,
      Some(token) => {
        token.get_catcode() == Catcode::BEGIN
          || state::with_meaning(
            &token,
            |m| matches!(m, Some(Stored::Token(t)) if *t == T_BEGIN!()),
          )
      },
    };
    if !is_open {
      Error!(
        "expected",
        "{",
        s!("Expected opening '{{' got {token_opt:?}")
      );
      return Ok(Tokens!());
    }
  }
  // Pre-size the token accumulator: most balanced reads are short
  // macro arguments (~4–16 tokens). This skips the Vec's early
  // doublings that the callgrind profile attributes to
  // `raw_vec::finish_grow` (1% of total instructions in read_balanced
  // alone).
  let mut tokens: Vec<Token> = Vec::with_capacity(16);
  let mut level = 1;
  loop {
    // we'll keep comments in the result
    let mut next_token = None;
    if !gullet!().pending_comments.is_empty() {
      tokens.extend(gullet_mut!().pending_comments.drain(..));
    }
    // Examine pushback first
    while let Some(pushback_token) = runtime_mut!().unwrap().pushback.pop() {
      match pushback_token.get_catcode() {
        Catcode::COMMENT => tokens.push(pushback_token),
        Catcode::MARKER => handle_marker(pushback_token),
        _ => {
          next_token = Some(pushback_token);
          break;
        },
      };
    }
    // Not in pushback, read from the current Mouth
    if next_token.is_none() {
      while let Some(token) = runtime_mut!().unwrap().mouth.read_token() {
        match token.get_catcode() {
          Catcode::COMMENT => tokens.push(token),
          Catcode::MARKER => handle_marker(token),
          _ => {
            next_token = Some(token);
            break;
          },
        };
      }
    }
    // ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;
    match next_token {
      // What's the right error handling now?
      None => break,
      Some(token) => match token.get_catcode() {
        Catcode::CS if token.text == pin!("\\dont_expand") => {
          if let Some(next_t) = read_token()? {
            tokens.push(next_t); // Pass on NEXT token, unchanged.
          }
        },
        Catcode::END => {
          // Perl Gullet.pm L476: track ALIGN_STATE for } inside readBalanced
          decrement_align_group_count();
          level -= 1;
          if level <= 0 {
            break;
          }
          tokens.push(token);
        },
        Catcode::BEGIN => {
          // Perl Gullet.pm L482: track ALIGN_STATE for { inside readBalanced
          increment_align_group_count();
          level += 1;
          tokens.push(token);
        },
        cc => {
          // Wow!!!!! See TeX the Program \S 309
          // Not sure if this code still applies within scan_toks???
          // SHOULD count nesting of { }!!! when SCANNED (not digested)
          if has_reading_alignment() && align_group_count() == 0 {
            if let Some((_atoken, atype, ahidden)) = is_column_end(&token) {
              if let DigestedData::Alignment(data) = get_reading_alignment().unwrap().data() {
                handle_template(data.borrow_mut(), token, atype, ahidden)?;
              } else {
                panic!("malformed alignmed was stored?");
              }
              continue;
            }
          }
          // Note: use general-purpose lookup, since we may reexamine $defn below
          if expansion_level != Off && cc.is_active_or_cs() {
            // Borrow the stored meaning via with_meaning so the Stored
            // enum is not cloned per token. We extract (a) whether a
            // meaning exists at all (for the undefined-CS diagnostic
            // below) and (b) the Rc<dyn Definition> if it's a proper
            // definition — both are cheap (bool + Rc-clone).
            let (has_meaning, defn_opt) =
              state::with_meaning(&token, |m| (m.is_some(), m.and_then(|s| s.to_definition())));
            if let Some(defn) = defn_opt {
              if defn.is_expandable() && (!defn.is_protected() || expansion_level == Full) {
                local_current_token(token);
                let expansion = defn.invoke(false)?;
                if expansion.is_empty() {
                  expire_current_token();
                  continue;
                }
                // If a special \the type command, push the expansion directly into the result
                // Well, almost directly: handle any MARKER tokens now, and possibly un-pack T_PARAM
                if expansion_level != Full && DEFERRED_COMMANDS.contains(&defn.get_cs().text) {
                  for t in expansion.unlist() {
                    match t.get_catcode() {
                      Catcode::MARKER => handle_marker(t),
                      Catcode::PARAM if is_macrodef => {
                        // "unpack" to cover the packParameters at end!
                        tokens.push(t);
                        tokens.push(t);
                      },
                      _ => tokens.push(t),
                    }
                  }
                } else {
                  // otherwise, prepend to pushback to be expanded further.
                  unread(expansion);
                }
                expire_current_token();
                continue;
              }
            } else if cc == Catcode::CS && !has_meaning {
              // cs SHOULD have defn by now; report early!
              generate_error_stub(&token)?;
            }
          }
          // if no special handling triggered above, just return the token
          tokens.push(token);
        },
      },
    }
  }
  if level > 0 {
    // TODO: The current implementation has a limitation where if the balancing end is in a
    // different mouth,       it will not be recognized.
    // TODO: also, add the startloc details
    // my $loc_message = $startloc ? ("Started at " . ToString($startloc)) : ("Ended at " .
    // ToString($self->getLocator));
    Error!(
      "expected",
      "}",
      "Gullet->readBalanced ran out of input in an unbalanced state"
    );
  }
  expire_align_group_count();
  if tokens.is_empty() {
    Ok(Tokens!())
  } else {
    Ok(if is_macrodef {
      Tokens::new(tokens).pack_parameters()?
    } else {
      Tokens::new(tokens)
    })
  }
}

/// Match the input against a set of keywords; Similar to readMatch, but the keywords are strings,
/// and Case and catcodes are ignored; additionally, leading spaces are skipped.
/// AND, macros are expanded.
///
/// Perf: zero-allocation char-wise comparison against each keyword.
/// The previous version allocated two Strings per char-match (via `to_uppercase()`
/// and `char::to_string()`), which was expensive in hot parameter parsing loops.
pub fn read_keyword(keywords: &[&str]) -> Result<Option<String>> {
  skip_spaces()?;
  for keyword in keywords.iter() {
    // Pre-size to the keyword length — `matched` holds one token per
    // matched char, and we unread them on no-match. Keyword-match
    // runs on every parameter/keyword read; small win per call.
    let mut matched = Vec::with_capacity(keyword.len());
    let mut ok = true;
    for expected in keyword.chars() {
      let Some(tok) = read_x_token(Some(false), false, None)? else {
        ok = false;
        break;
      };
      // Compare char-by-char against the token's text, case-insensitively.
      let eq = tok.with_str(|s| {
        let mut it = s.chars();
        match it.next() {
          Some(c) if it.next().is_none() => {
            // single-char token: case-insensitive compare
            c.to_uppercase().eq(expected.to_uppercase())
          },
          _ => false,
        }
      });
      matched.push(tok);
      if !eq {
        ok = false;
        break;
      }
    }
    if ok {
      return Ok(Some(keyword.to_string()));
    } else {
      unread(matched.into());
    }
  }
  Ok(None)
}

/// Return a (balanced) sequence tokens until a match against one of the Tokens in @delims.
///
/// Note that Braces on input hides the contents from matching,
/// so this assumes there wont be braces in $delim!
/// But, see readUntilBrace for that case.
pub fn read_until(delim: &Tokens) -> Result<Tokens> {
  let mut tokens: Vec<Token> = Vec::new();
  let mut nbraces = 0;
  let want = delim.unlist_ref();
  let ntomatch = want.len();
  let mut has_matched;

  if ntomatch == 1 {
    let want = &want[0];
    loop {
      let token = match read_token()? {
        Some(t) => t,
        None => {
          // Ran out!
          unread(Tokens::new(tokens));
          return Ok(Tokens!()); // Not more correct, but maybe less confusing?
        },
      };
      // Perl: check direct match OR \special_relax smuggling (Gullet.pm line 662)
      if token == *want || special_relax_matches(&token, want) {
        break;
      }
      match token.get_catcode() {
        Catcode::MARKER => {
          // would have been handled by readToken, but we're bypassing
          handle_marker(token);
        },
        Catcode::BEGIN => {
          // And if it's a BEGIN, copy till balanced END
          nbraces += 1;
          tokens.push(token);
          let balanced_arg = read_balanced(ExpansionLevel::Off, false, false)?;
          if !balanced_arg.is_empty() {
            tokens.extend(balanced_arg.unlist());
          }
          tokens.push(T_END!());
        },
        _ => {
          tokens.push(token);
        },
      }
    }
  } else {
    let mut ring = VecDeque::new();
    loop {
      // prefill the required number of tokens
      while ring.len() < ntomatch {
        let token = match read_token()? {
          Some(t) => t,
          None => {
            // Ran out!
            unread(Tokens::new(tokens));
            return Ok(Tokens!()); // Not more correct, but maybe less confusing?
          },
        };
        // Perl: $$token[1] == CC_BEGIN — direct catcode check
        if token.get_catcode() == Catcode::BEGIN {
          // read balanced, and refill ring.
          nbraces += 1;
          for r_token in ring {
            tokens.push(r_token);
          }
          tokens.push(token);
          let balanced_arg = read_balanced(ExpansionLevel::Off, false, false)?;
          if !balanced_arg.is_empty() {
            tokens.append(&mut balanced_arg.unlist());
          }
          tokens.push(T_END!()); // Copy directly to result
          ring = VecDeque::new(); // and retry
        } else {
          ring.push_back(token);
        }
      }
      has_matched = &ring == want; // Test match
      if has_matched {
        break;
      } // Matched all!
      if let Some(ring_token) = ring.pop_front() {
        tokens.push(ring_token);
      }
    }
  }
  // Notice that IFF the arg looks like {balanced}, the outer braces are stripped
  // so that delimited arguments behave more similarly to simple, undelimited arguments.
  // Perl: ($nbraces == 1) && ($tokens[0][1] == CC_BEGIN) && ($tokens[-1][1] == CC_END)
  if nbraces == 1
    && tokens.first().unwrap().get_catcode() == Catcode::BEGIN
    && tokens.last().unwrap().get_catcode() == Catcode::END
  {
    tokens.remove(0);
    tokens.pop();
  }
  Ok(Tokens::new(tokens))
}

/// Convenience method wrapping around `read_until`
/// TODO: This seems to be the wrong Rust type interface, we need to rework...
pub fn read_until_token(t: Token) -> Result<Tokens> { read_until(&Tokens!(t)) }
/// reads until it encounters a Catcode::BEGIN token
/// Note: Perl uses $$token[1] == CC_BEGIN (catcode check, not defined_as)
pub fn read_until_brace() -> Result<Option<Tokens>> {
  let mut tokens = Vec::new();
  while let Some(token) = read_token()? {
    if token.get_catcode() == Catcode::BEGIN {
      unread_one(token); // Unread with proper agc adjustment
      break;
    } else {
      tokens.push(token);
    }
  }
  if tokens.is_empty() {
    Ok(None)
  } else {
    let tks = Tokens::new(tokens);
    Ok(Some(tks))
  }
}

pub fn read_cs_name() -> Result<Token> { read_cs_name_inner(false) }

/// Quiet version of read_cs_name — used by \ifcsname.
/// In TeX, \ifcsname silently skips non-expandable CS tokens and returns the constructed name
/// without emitting errors (unlike \csname which DOES emit errors).
pub fn read_cs_name_quiet() -> Result<Token> { read_cs_name_inner(true) }

fn read_cs_name_inner(quiet: bool) -> Result<Token> {
  // TeX does NOT store the csname with the leading `\`, BUT stores active chars with a flag
  // However, so long as the Mouth's CS and \string properly respect \escapechar, all's well!

  let mut cs = String::from("\\");
  // keep newlines from having \n inside!
  while let Some(token) = read_x_token(Some(true), false, None)? {
    if token.defined_as(&TOKEN_ENDCSNAME) {
      break;
    }
    match token.get_catcode() {
      Catcode::CS => {
        if !quiet {
          if lookup_definition(&token)?.is_some() {
            let message = s!(
              "The control sequence {:?} should not appear between \\csname and \\endcsname (partial cs so far: {:?})",
              token,
              cs
            );
            Error!("unexpected", token, message);
          } else {
            let message = s!("The token {:?} is not defined", token);
            Error!("undefined", token, message);
          }
        }
        // In quiet mode (ifcsname), just skip the CS token
      },
      Catcode::SPACE => cs.push(' '), // Keep newlines from having \n!
      _ => {
        token.with_str(|s| cs.push_str(s));
      },
    };
  }
  Ok(T_CS!(cs))
}

/// reads and discards tokens, until it encounters a conditional, if any.
/// Perl: skipConditionalBody inner loop (Conditional.pm L127-133) reads tokens directly
/// from pushback/mouth and manually tracks ALIGN_STATE for { and }.
pub fn read_next_conditional() -> Result<Option<(Token, ConditionalType)>> {
  loop {
    match read_token()? {
      Some(token) => {
        let cc = token.get_catcode();
        // Perl L128-130: ALIGN_STATE tracking for { and } now handled by read_token itself
        if cc.is_active_or_cs() {
          if let Some(cond_type) = lookup_conditional(&token) {
            return Ok(Some((token, cond_type)));
          }
        }
      },
      None => {
        // Current mouth exhausted. Try closing if autoclosable and there are
        // more mouths on the stack (TeX continues reading across input boundaries).
        let (autoclose, stack_len) = {
          let gullet = gullet!();
          let ac = gullet
            .runtime
            .as_ref()
            .map(|r| r.autoclose)
            .unwrap_or(false);
          let sl = gullet.mouthstack.len();
          (ac, sl)
        };
        if autoclose && stack_len > 0 {
          close_mouth(false)?;
          continue;
        }
        return Ok(None);
      },
    }
  }
}

//**********************************************************************
// Higher-level readers: Read various types of things from the input:
//  tokens, non-expandable tokens, args, Numbers, ...
//**********************************************************************

/// Read and return a "normal" TeX argument
///
/// The next Token or Tokens (if surrounded by braces).
/// `expansion_level` controls expansion as if the argument were read
///  and then expanded in isolation:
///
/// In the case of a single unbraced expandable token,
///  it will **not** read any macro arguments from the following input!
pub fn read_arg(expansion_level: ExpansionLevel) -> Result<Tokens> {
  match read_non_space()? {
    None => Ok(Tokens!()),
    Some(token) => {
      // Perl: $$token[1] == CC_BEGIN — checks actual catcode, NOT defined_as.
      // \bgroup (catcode CS) does NOT match here; only literal { does.
      if token.get_catcode() == Catcode::BEGIN {
        read_balanced(expansion_level, false, false)
      } else if matches!(expansion_level, ExpansionLevel::Off) {
        Ok(Tokens!(token))
      } else {
        unread_vec(vec![T_BEGIN!(), token, T_END!()]);
        read_balanced(expansion_level, false, true)
      }
    },
  }
}
/// Read and return a LaTeX optional argument
///
/// returns `default` if there is no '[', otherwise the contents of the array.
/// Note that this returns an empty array if `[]` is present,
/// i.e. `[contents]` in TeX will lead to `Tokens(contents)`, otherwise returns `None`
pub fn read_optional(default: Option<Tokens>) -> Result<Option<Tokens>> {
  match read_non_space()? {
    None => Ok(None),
    Some(t) => {
      if t.get_catcode() == Catcode::OTHER && t.get_sym() == pin!("[") {
        Ok(Some(read_until(&Tokens!(T_OTHER!("]")))?))
      } else {
        unread_one(t);
        Ok(default)
      }
    },
  }
}

/// <filler> = <optional spaces> | <filler>\relax<optional spaces>
/// TeX Book p.276 "<left brace> can be implicit", and experimentation, indicate Expansion!!!
pub fn skip_filler() -> Result<()> {
  while let Some(tok) = read_x_non_space()? {
    if !tok.defined_as(&TOKEN_RELAX) {
      unread_one(tok);
      break;
    }
  }
  Ok(())
}

pub fn if_next(token: Token) -> Result<bool> {
  let mut is_next = false;
  if let Some(tok) = read_token()? {
    is_next = tok == token;
    unread_one(tok);
  }
  Ok(is_next)
}

/// Perl: peekToken — peek at the next token without triggering alignment
/// Sets ALIGN_STATE to 1000000 to suppress alignment template handling (Perl line 331-337)
pub fn peek_token() -> Result<Option<Token>> {
  local_align_group_count(1000000);
  let result = read_token()?;
  if let Some(ref tok) = result {
    unread_one(*tok);
  }
  expire_align_group_count();
  Ok(result)
}

/// Perl: showUnexpected — returns a debug message about the next available token
pub fn show_unexpected() -> String {
  match peek_token() {
    Ok(Some(token)) => {
      let meaning = lookup_meaning(&token)
        .map(|m| format!("{:?}", m))
        .unwrap_or_else(|| "undef".to_string());
      s!("Next token is {} ( == {})", token.stringify(), meaning)
    },
    _ => "Input is empty".to_string(),
  }
}

//**********************************************************************
//  Numbers, Dimensions, Glue
// See TeXBook, Ch.24, pp.269-271.
//**********************************************************************

pub fn read_value(value_type: RegisterType) -> Result<RegisterValue> {
  match value_type {
    RegisterType::Number => Ok(read_number()?.into()),
    RegisterType::Dimension => Ok(read_dimension()?.into()),
    RegisterType::MuDimension => Ok(read_mu_dimension()?.into()),
    RegisterType::Glue => Ok(read_glue()?.into()),
    RegisterType::MuGlue => Ok(read_mu_glue()?.into()),
    RegisterType::Tokens => Ok(read_tokens_value()?.into()),
    RegisterType::Token => {
      // Perl: readValue('Token') checks for \csname (Gullet.pm line 770-775)
      #[thread_local]
      static TOKEN_CSNAME: Lazy<Token> = Lazy::new(|| T_CS!("\\csname"));
      let token = read_non_space()?.unwrap_or(*TOKEN_RELAX);
      if token.defined_as(&TOKEN_CSNAME) {
        Ok(read_cs_name()?.into())
      } else {
        Ok(token.into())
      }
    },
    RegisterType::CharDef => Ok(read_number()?.into()),
    RegisterType::Any => Ok(read_arg(ExpansionLevel::Off)?.into()),
  }
}

pub fn read_register_value(value_type: RegisterType) -> Result<Option<RegisterValue>> {
  read_register_value_coerce(value_type, false)
}

/// Read a register value, optionally coercing from a compatible larger type.
/// Perl: readRegisterValue($self, $type, $sign, $coerce)
/// Coercion rules (from Perl %RegisterCoercionTypes):
///   Number    <- Dimension, Glue     (extract raw i64)
///   Dimension <- Glue                (extract skip as Dimension)
///   MuDimension <- MuGlue            (extract skip as MuDimension)
pub fn read_register_value_coerce(
  value_type: RegisterType,
  coerce: bool,
) -> Result<Option<RegisterValue>> {
  match read_x_token(None, false, None)? {
    None => Ok(None),
    Some(token) => {
      let _is_fontdimen = token.with_str(|s| s == "\\fontdimen");
      if let Some(defn) = lookup_register_definition(&token) {
        if let Some(mut register_type) = defn.register_type() {
          if register_type == RegisterType::CharDef {
            // CharDefs treated as numbers here
            register_type = RegisterType::Number;
          }
          if register_type == value_type {
            let args = defn.read_arguments()?;
            Ok(defn.value_of(args))
          } else if coerce {
            // Try type coercion per Perl's %RegisterCoercionTypes
            if let Some(coerced) = coerce_register(value_type, register_type, &defn)? {
              Ok(Some(coerced))
            } else {
              unread_one(token);
              Ok(None)
            }
          } else {
            unread_one(token); // Unread
            Ok(None)
          }
        } else {
          unread_one(token); // Unread
          Ok(None)
        }
      } else {
        unread_one(token); // Unread
        Ok(None)
      }
    },
  }
}

/// Attempt to coerce a register value from `source_type` to `target_type`.
fn coerce_register(
  target_type: RegisterType,
  source_type: RegisterType,
  defn: &Register,
) -> Result<Option<RegisterValue>> {
  use crate::common::numeric_ops::NumericOps;
  // Perl fix 50f0061d: include self-coercions (Number→Number, etc.)
  // so \number \fam works when \fam is already a Number register
  let can_coerce = matches!(
    (target_type, source_type),
    (RegisterType::Number, RegisterType::Number)
      | (RegisterType::Number, RegisterType::Dimension)
      | (RegisterType::Number, RegisterType::Glue)
      | (RegisterType::Dimension, RegisterType::Dimension)
      | (RegisterType::Dimension, RegisterType::Glue)
      | (RegisterType::MuDimension, RegisterType::MuDimension)
      | (RegisterType::MuDimension, RegisterType::MuGlue)
      | (RegisterType::Glue, RegisterType::Glue)
      | (RegisterType::MuGlue, RegisterType::MuGlue)
  );
  if !can_coerce {
    return Ok(None);
  }
  let args = defn.read_arguments()?;
  if let Some(val) = defn.value_of(args) {
    let raw = match val {
      RegisterValue::Dimension(d) => d.value_of(),
      RegisterValue::Glue(g) => g.value_of(),
      RegisterValue::MuGlue(mg) => mg.value_of(),
      RegisterValue::Number(n) => n.value_of(),
      RegisterValue::MuDimension(md) => md.value_of(),
      _ => return Ok(None),
    };
    let coerced = match target_type {
      RegisterType::Number => RegisterValue::Number(Number::new(raw)),
      RegisterType::Dimension => RegisterValue::Dimension(Dimension::new(raw)),
      RegisterType::MuDimension => RegisterValue::MuDimension(MuDimension::new(raw)),
      _ => return Ok(None),
    };
    Ok(Some(coerced))
  } else {
    Ok(None)
  }
}

/// Match the input against one of the Token or Tokens in @choices; return the matching one or
/// undef.
pub fn read_match(choices: &[&Tokens]) -> Result<Option<Tokens>> {
  for choice in choices {
    let mut to_match: Vec<&Token> = choice.unlist_ref().iter().rev().collect();
    // `matched` accumulates tokens read so far, bounded by `choice.len()`.
    // Pre-size to avoid reallocations on multi-token match attempts.
    let mut matched = Vec::with_capacity(choice.unlist_ref().len());
    while !to_match.is_empty() {
      match read_token()? {
        None => break,
        Some(token) => {
          let cc = token.get_catcode();
          // Perl: also check smuggled \special_relax token (Gullet.pm line 612)
          let was_last_match = if let Some(&&want) = to_match.last() {
            token == want || special_relax_matches(&token, &want)
          } else {
            false
          };
          matched.push(token);
          if was_last_match {
            to_match.pop();
          } else {
            break;
          }

          if cc == Catcode::SPACE {
            // If this was space, SKIP any following!!!
            while let Some(space_token) = read_token()? {
              if space_token.get_catcode() != Catcode::SPACE {
                // Unread non-space and end — use unread_one for proper agc adjustment
                unread_one(space_token);
                break;
              } else {
                matched.push(space_token);
              }
            }
          }
        },
      }
    }
    if to_match.is_empty() {
      return Ok(Some((*choice).clone())); // All matched!!!
    } else {
      // Put 'em back and try next — use unread_vec for proper agc adjustment
      unread_vec(matched);
    }
  }
  Ok(None)
}

//======================================================================
// Integer, Number
//======================================================================
// ```
// <number> = <optional signs><unsigned number>
// <unsigned number> = <normal integer> | <coerced integer>
// <coerced integer> = <internal dimen> | <internal glue>
// ```
pub fn read_number() -> Result<Number> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1 } else { 1 };
  if let Some(n) = read_normal_integer()? {
    if is_negative { Ok(n.negate()) } else { Ok(n) }
  } else if let Some(n) = read_internal_dimension()? {
    Ok(Number::new(s * n.value_of()))
  } else if let Some(n) = read_internal_glue()? {
    Ok(Number::new(s * n.value_of()))
  } else {
    let next = read_token()?;
    // Fallback for the error message if the current-token register is not
    // populated — hitting "missing number" with no current token is rare
    // but plausible (deeply nested macro-expansion paths can leave the
    // register empty), and the diagnostic should not bring the run down.
    let current = get_current_token()
      .map(|t| format!("{t:?}"))
      .unwrap_or_else(|| String::from("<none>"));
    let message = s!(
      "Missing number, treated as zero while processing {}, next token is {:?}",
      current,
      next
    );
    Warn!("expected", "<number>", message);
    if let Some(next) = next {
      unread_one(next);
    }
    Ok(Number::new(0))
  }
}

/// ```bnf
/// <normal integer> = <internal integer> | <integer constant>
///   | '<octal constant><one optional space> | "<hexadecimal constant><one optional space>
///   | `<character token><one optional space>
/// ```
pub fn read_normal_integer() -> Result<Option<Number>> {
  match read_x_token(None, false, None)? {
    None => Ok(None),
    Some(token) => {
      let cc = token.get_catcode();
      let mut text = token.to_string();
      if cc == Catcode::OTHER && text.chars().all(|c| c.is_ascii_digit()) {
        // Read decimal literal. Overflow is rare but possible on weird
        // input (digit runs wider than i64::MAX); Perl's TeX silently
        // truncates such values, so we fall back to i64::MAX / MIN on
        // parse failure rather than panicking with .expect().
        text.push_str(&read_digits(&DIGIT_RE, true)?);
        let n = text.parse::<i64>().unwrap_or_else(|_| {
          if text.starts_with('-') {
            i64::MIN
          } else {
            i64::MAX
          }
        });
        Ok(Some(Number::new(n)))
      } else if token == T_OTHER!("'") {
        // Read Octal literal
        let decimal = i64::from_str_radix(&read_digits(&OCT_RE, true)?, 8)?;
        Ok(Some(Number::new(decimal)))
      } else if token == T_OTHER!("\"") {
        //  Read Hex literal
        let decimal = i64::from_str_radix(&read_digits(&HEX_RE, true)?, 16)?;
        Ok(Some(Number::new(decimal)))
      } else if token == T_OTHER!("`") {
        //  Read Charcode: `<character token><one optional space>
        let mut s = match read_token()? {
          None => String::new(),
          Some(next) => next.to_string(),
        };
        if s.starts_with('\\') {
          s.remove(0);
        }
        let s_char = s.chars().next().unwrap_or('\0');
        // Perl: skip1Space($self, 1); — expanded space-skip after charcode
        skip_one_space(true)?;
        Ok(Some(Number::new(s_char as i64))) //  Only a character token!!! NOT expanded!!!!
      } else {
        unread_one(token); // Unread
        read_internal_integer()
      }
    },
  }
}

///======================================================================
/// Float, a floating point number.
/// Similar to factor, but does NOT accept comma!
/// This is NOT part of TeX, but is convenient.
pub fn read_float() -> Result<Float> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1.0 } else { 1.0 };
  let mut string = read_digits(&DIGIT_RE, true)?;
  let mut token = read_x_token(None, false, None)?;
  if token.is_some() && token.as_ref().unwrap().get_sym() == pin!(".") {
    string = s!("{string}.{}", read_digits(&DIGIT_RE, true)?);
    token = read_x_token(None, false, None)?;
  }
  let n_opt: Option<f64> = if !string.is_empty() {
    if let Some(t) = token {
      if t.get_catcode() != Catcode::SPACE {
        unread_one(t);
      }
    }
    // Same rationale as read_normal_integer above: malformed float
    // literals (e.g. very long digit runs, "1e" without exponent)
    // should degrade to 0.0 rather than panic.
    Some(string.parse::<f64>().unwrap_or(0.0))
  } else {
    if let Some(t) = token {
      unread_one(t); // Unread
    }
    read_normal_integer()?.map(|v| v.value_of() as f64)
  };

  if let Some(n) = n_opt {
    Ok(Float::new_f64(s * n))
  } else {
    Ok(Float::new_f64(0.0))
  }
}

fn read_internal_integer() -> Result<Option<Number>> {
  match read_register_value(RegisterType::Number)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}
fn read_internal_dimension() -> Result<Option<Dimension>> {
  match read_register_value(RegisterType::Dimension)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}
fn read_internal_glue() -> Result<Option<Glue>> {
  match read_register_value(RegisterType::Glue)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}

//======================================================================
// Dimensions
//======================================================================
// ```
// <dimen> = <optional signs><unsigned dimen>
// <unsigned dimen> = <normal dimen> | <coerced dimen>
// <coerced dimen> = <internal glue>
// ```
pub fn read_dimension() -> Result<Dimension> {
  let is_negative = read_optional_signs()?;
  if let Some(d) = read_internal_dimension()? {
    Ok(if is_negative { d.negate() } else { d })
  } else if let Some(d) = read_internal_glue()? {
    Ok(Dimension::new(if is_negative {
      d.negate().value_of()
    } else {
      d.value_of()
    }))
  } else if let Some(d) = read_factor()? {
    let unit = match read_unit()? {
      Some(u) => u,
      None => {
        Warn!(
          "expected",
          "<unit>",
          "Illegal unit of measure (pt inserted)."
        );
        65536.0
      },
    };
    let d_signed = if is_negative { -d } else { d };
    Ok(Dimension::new(fixpoint(d_signed, Some(unit))))
  } else {
    let cur = get_current_token()
      .map(|t| format!("{t:?}"))
      .unwrap_or_else(|| String::from("<none>"));
    let message = s!("Missing number, treated as zero. while processing {}", cur);
    Warn!("expected", "<number>", message);
    Ok(Dimension::new(0))
  }
}

// ```
// <unit of measure> = <optional spaces><internal unit>
//     | <optional true><physical unit><one optional space>
// <internal unit> = em <one optional space> | ex <one optional space>
//     | <internal integer> | <internal dimen> | <internal glue>
// <physical unit> = pt | pc | in | bp | cm | mm | dd | cc | sp
// ```

/// Read a unit, returning the equivalent number of scaled points,
pub fn read_unit() -> Result<Option<f64>> {
  let unit_opt = if let Some(u) = read_keyword(&["ex", "em"])? {
    skip_one_space(true)?;
    Some(convert_unit(&u))
  } else if let Some(u) = read_internal_integer()? {
    Some(u.value_of() as f64) // These are coerced to number=>sp
  } else if let Some(u) = read_internal_dimension()? {
    Some(u.value_of() as f64)
  } else if let Some(u) = read_internal_glue()? {
    Some(u.value_of() as f64)
  } else {
    read_keyword(&["true"])?; // But ignore, we're not bothering with mag...
    if let Some(u) = read_keyword(&["pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp", "px"])? {
      skip_one_space(true)?;
      Some(convert_unit(&u))
    } else {
      None
    }
  };
  Ok(unit_opt)
}

//======================================================================
// Glue
//======================================================================
// <glue> = <optional signs><internal glue> | <dimen><stretch><shrink>
// <stretch> = plus <dimen> | plus <fil dimen> | <optional spaces>
// <shrink>  = minus <dimen> | minus <fil dimen> | <optional spaces>
pub fn read_glue() -> Result<Glue> {
  let is_negative = read_optional_signs()?;
  if let Some(n) = read_internal_glue()? {
    if is_negative { Ok(n.negate()) } else { Ok(n) }
  } else {
    let mut d = read_dimension()?;
    if is_negative {
      d = d.negate();
    }
    let (r1, f1) = match read_keyword(&["plus"])? {
      Some(_) => read_rubber(false)?,
      None => (None, None),
    };
    let (r2, f2) = match read_keyword(&["minus"])? {
      Some(_) => read_rubber(false)?,
      None => (None, None),
    };

    Ok(Glue::new_spec(
      &d.value_of().to_string(),
      r1.map(|v| v as f64),
      f1,
      r2.map(|v| v as f64),
      f2,
    ))
  }
}

pub fn read_rubber(mu: bool) -> Result<(Option<i64>, Option<FillCode>)> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1 } else { 1 };
  match read_factor()? {
    None => {
      let f = if mu {
        read_mu_dimension()?.value_of()
      } else {
        read_dimension()?.value_of()
      };
      Ok((Some(f * s), None))
    },
    Some(f) => match read_keyword(&["filll", "fill", "fil"])? {
      Some(fil) => Ok((Some(fixpoint(s as f64 * f, None)), FillCode::from(&fil))),
      None => {
        let u = if mu {
          match read_mu_unit()? {
            None => {
              Warn!(
                "expected",
                "<unit>",
                "Illegal unit of measure (mu inserted)."
              );
              None
            },
            Some(v) => Some(v as f64),
          }
        } else {
          match read_unit()? {
            None => {
              Warn!(
                "expected",
                "<unit>",
                "Illegal unit of measure (pt inserted)."
              );
              None
            },
            Some(v) => Some(v),
          }
        };
        Ok((Some(fixpoint(s as f64 * f, u)), None))
      },
    },
  }
}

//======================================================================
// Mu Glue
//======================================================================
// <muglue> = <optional signs><internal muglue> | <mudimen><mustretch><mushrink>
// <mustretch> = plus <mudimen> | plus <fil dimen> | <optional spaces>
// <mushrink> = minus <mudimen> | minus <fil dimen> | <optional spaces>
pub fn read_mu_glue() -> Result<MuGlue> {
  let is_negative = read_optional_signs()?;
  if let Some(n) = read_internal_mu_glue()? {
    Ok(if is_negative { n.negate() } else { n })
  } else {
    let mut d = read_mu_dimension()?;
    if is_negative {
      d = d.negate()
    }
    let (r1, f1) = if read_keyword(&["plus"])?.is_some() {
      read_rubber(true)?
    } else {
      (None, None)
    };
    let (r2, f2) = if read_keyword(&["minus"])?.is_some() {
      read_rubber(true)?
    } else {
      (None, None)
    };
    Ok(MuGlue::new_full(d.value_of(), r1, f1, r2, f2))
  }
}

//======================================================================
// Mu Dimensions
//======================================================================
// <mudimen> = <optional signs><unsigned mudimem>
// <unsigned mudimen> = <normal mudimen> | <coerced mudimen>
// <normal mudimen> = <factor><mu unit>
// <mu unit> = <optional spaces><internal muglue> | mu <one optional space>
// <coerced mudimen> = <internal muglue>
pub fn read_mu_dimension() -> Result<MuDimension> {
  let is_negative = read_optional_signs()?;
  if let Some(mut m) = read_factor()? {
    let munit = read_mu_unit()?;
    if munit.is_none() {
      Warn!(
        "expected",
        "<unit>",
        "Illegal unit of measure (mu inserted)."
      );
    }
    if is_negative {
      m *= -1.0;
    }
    Ok(MuDimension::new(fixpoint(m, munit.map(|v| v as f64))))
  } else if let Some(mglue) = read_internal_mu_glue()? {
    let m = if is_negative { mglue.negate() } else { mglue };
    Ok(MuDimension::new(m.value_of()))
  } else {
    Warn!("expected", "<mudimen>", "Expecting mudimen; assuming 0");
    Ok(MuDimension::new(0))
  }
}

pub fn read_mu_unit() -> Result<Option<i64>> {
  if read_keyword(&["mu"])?.is_some() {
    skip_one_space(true)?;
    Ok(Some(UNITY)) // effectively, scaled mu
  } else if let Some(m) = read_internal_mu_glue()? {
    Ok(Some(m.value_of()))
  } else {
    Ok(None)
  }
}

fn read_internal_mu_glue() -> Result<Option<MuGlue>> {
  match read_register_value(RegisterType::MuGlue)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}

/// Apparent behaviour of a token value (ie `\toks#=<arg>`)
pub fn read_tokens_value() -> Result<Tokens> {
  match read_non_space()? {
    None => Ok(Tokens!()),
    Some(token) => {
      // Perl: $$token[1] == CC_BEGIN — direct catcode check
      if token.get_catcode() == Catcode::BEGIN {
        Ok(read_balanced(ExpansionLevel::Off, false, false)?)
      } else if let Some(defn) = lookup_register_definition(&token) {
        match defn.register_type() {
          Some(RegisterType::Tokens) | Some(RegisterType::Token) => {
            // TODO: The mismatch between Vec<Tokens> for read_arguments and Vec<Token> for
            // value_of feels incorrect       but in which direction should it be
            // resolved?
            let args = defn.read_arguments()?;
            match defn.value_of(args) {
              None => Ok(Tokens!()),
              Some(v) => Ok(v.into()),
            }
          },
          _ => Ok(Tokens!(token)),
        }
      } else if let Some(defn) = lookup_definition(&token)? {
        // TODO: we are doing two lookups to avoid the type restriction of .read_arguments, any
        // way to circumvent? Is it slow in the first place?
        if defn.is_expandable() {
          let x = defn.invoke(false)?;
          if !x.is_empty() {
            unread(x);
          }
          read_tokens_value()
        } else {
          Ok(Tokens!(token))
        }
      } else {
        Ok(Tokens!(token))
      }
    },
  }
}

pub fn skip_spaces() -> Result<()> {
  if let Some(t) = read_non_space()? {
    unread_one(t);
  }
  Ok(())
}

/// Check if a token is a space token (catcode SPACE) or an "implicit space"
/// (a CS or ACTIVE token `\let` to a space token).
/// See TeXbook p269: `<one optional space>` absorbs both explicit and implicit spaces.
fn is_space_or_implicit_space(token: &Token) -> bool {
  if token.get_catcode() == Catcode::SPACE {
    return true;
  }
  // Check for implicit space: CS/ACTIVE let to a space token
  if token.get_catcode() == Catcode::CS || token.get_catcode() == Catcode::ACTIVE {
    return state::with_meaning(
      token,
      |m| matches!(m, Some(Stored::Token(t)) if t.get_catcode() == Catcode::SPACE),
    );
  }
  false
}

/// Skip one optional space.
/// If `expanded` is true, acts like `<one optional space>` and expands tokens (readXToken).
/// Perl: skip1Space($self, $expanded)
pub fn skip_one_space(expanded: bool) -> Result<()> {
  let token = if expanded {
    read_x_token(None, false, None)?
  } else {
    read_token()?
  };
  if let Some(t) = token {
    if !is_space_or_implicit_space(&t) {
      unread_one(t);
    }
  }
  Ok(())
}

//======================================================================
// some helpers...

// <optional signs> = <optional spaces> | <optional signs><plus or minus><optional spaces>
// returns false if None, or positive, true if negative
pub fn read_optional_signs() -> Result<bool> {
  let mut sign = false;
  while let Some(t) = read_x_token(None, false, None)? {
    let sym = t.get_sym();
    if sym == pin!("-") {
      sign = !sign;
    } else if (sym != pin!("+")) && !is_space_or_implicit_space(&t) {
      unread_one(t); // Unread and end
      break;
    }
  }
  Ok(sign)
}

fn read_digits(range_regex: &Regex, skip: bool) -> Result<String> {
  let mut result = String::new();
  while let Some(token) = read_x_token(None, false, None)? {
    let digit_opt = token.with_str(|s| {
      if s.len() == 1 && range_regex.is_match(s) {
        s.chars().next()
      } else {
        None
      }
    });
    if let Some(digit) = digit_opt {
      result.push(digit);
    } else {
      if !(skip && is_space_or_implicit_space(&token)) {
        unread_one(token);
      }
      break;
    }
  }
  Ok(result)
}

// ```
// <factor> = <normal integer> | <decimal constant>
// <decimal constant> = . | , | <digit><decimal constant> | <decimal constant><digit>
// ```
/// Return a number (Rust f64 number)
pub fn read_factor() -> Result<Option<f64>> {
  let mut factor = read_digits(&DIGIT_RE, false)?;
  let mut token_opt = read_x_token(None, false, None)?;
  if let Some(ref token) = token_opt {
    let sym = token.get_sym();
    if sym == pin!(".") || sym == pin!(",") {
      factor = s!("{}.{}", factor, read_digits(&DIGIT_RE, false)?);
      token_opt = read_x_token(None, false, None)?;
    }
  }

  // Note: zero is an edge case with the unwrap_or fallback, handle it
  if !factor.is_empty() {
    let factor_f64: f64 = factor.parse::<f64>().unwrap_or(0.0);
    if let Some(token) = token_opt {
      if token.get_catcode() != Catcode::SPACE {
        unread_one(token);
      }
    }
    Ok(Some(factor_f64))
  } else {
    if let Some(token) = token_opt {
      unread_one(token);
    }
    match read_normal_integer()? {
      None => Ok(None),
      Some(n) => Ok(Some(n.value_of() as f64)),
    }
  }
}

pub fn do_expand<T: Into<Tokens>>(tokens: T) -> Result<Tokens> {
  let tokens: Tokens = tokens.into();
  reading_from_mouth(Mouth::default(), move || -> Result<Tokens> {
    {
      unread_one(T_END!());
      unread(tokens);
      unread_one(T_BEGIN!());
    }
    read_balanced(ExpansionLevel::Full, false, true)
  })
}

pub fn do_expand_partially<T: Into<Tokens>>(tokens: T) -> Result<Tokens> {
  let tokens: Tokens = tokens.into();
  reading_from_mouth(Mouth::default(), move || -> Result<Tokens> {
    {
      unread_one(T_END!());
      unread(tokens);
      unread_one(T_BEGIN!());
    }
    read_balanced(ExpansionLevel::Partial, false, true)
  })
}

pub fn is_column_end(token: &Token) -> Option<(Token, &'static str, bool)> {
  match token.get_catcode() {
    Catcode::ALIGN => Some((*token, "align", false)),
    Catcode::CS | Catcode::ACTIVE => {
      // Embedded version of Equals, knowing both are tokens
      let defn = lookup_meaning(token).unwrap_or_else(|| Stored::Token(*token));
      // Perl Gullet.pm L273: if meaning is a Token with CC_ALIGN, treat as alignment tab
      if let Stored::Token(t) = &defn {
        if t.get_catcode() == Catcode::ALIGN {
          return Some((*token, "align", false));
        }
      }
      for end in *COLUMN_ENDS {
        let e = &end.0;
        // Would be nice to cache the defns, but don't know when they're present & constant!
        if defn == lookup_meaning(e).unwrap_or_else(|| Stored::Token(*e)) {
          return Some(end);
        }
      }
      None
    },
    _ => None,
  }
}
/// Handle a marker token, by updating the current alignment group count
fn handle_marker(marker_token: Token) {
  marker_token.with_str(|arg| match arg {
    "before-column" => {
      // Were in before-column template
      set_align_group_count(0);
    }, // switch to column proper!
    "after-column" => { // Were in before-column template
      // let alignment = lookup_alignment();
      // Debug("Halign $alignment: alignment  after column") if $LaTeXML::DEBUG{halign};
    },
    _ => {},
  });
}

/// Do something, while reading tokens from a specific Mouth.
///
/// This reads ONLY from that mouth (or any mouth openned by code in that source),
/// and the mouth should end up empty afterwards, and only be closed here.
pub fn reading_from_mouth<R, FnR>(mouth: Mouth, reader: FnR) -> Result<R>
where FnR: FnOnce() -> Result<R> {
  let context_mouth_source = arena::pin(mouth.get_source());
  open_mouth(mouth, false); // only allow mouth to be explicitly closed here.
  let reader_result = reader();
  // If the reader returned an error (e.g., Fatal from token limit),
  // we STILL need to clean up the mouth to preserve the caller's state.
  let results: R = match reader_result {
    Ok(v) => v,
    Err(e) => {
      // Force-close our mouth and any autoclosable mouths above it
      loop {
        let current = gullet!()
          .runtime
          .as_ref()
          .map(|r| arena::pin(r.mouth.get_source()));
        if current == Some(context_mouth_source) {
          close_mouth(true).ok();
          break;
        } else if gullet!().mouthstack.is_empty() {
          break; // Our mouth was already consumed
        } else {
          close_mouth(true).ok(); // Close stale mouth above ours
        }
      }
      // Reset progress counter so subsequent processing isn't immediately killed
      gullet_mut!().progress = 0;
      return Err(e);
    },
  };
  // `mouth` must still be open, with (at worst) empty autoclosable mouths in front of it.
  // Rate-limit the "mouth closed" error — when the gullet gets into a state
  // where the cleanup loop keeps finding stale mouths above the target, the
  // same error can fire on EVERY caller of reading_from_mouth. Arxiv 0906.1883
  // (birkmult + local .cls) can trigger 10K+ such firings, one per stack frame.
  // Fatal out after 50 repeat firings so the process surfaces a clear "we lost
  // the mouth stack" signal instead of filling the log with identical messages.
  thread_local! {
    static MOUTH_CLOSED_ERRORS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
  }
  fn record_mouth_closed_error() { MOUTH_CLOSED_ERRORS.with(|c| c.set(c.get().saturating_add(1))); }
  fn should_emit_mouth_closed() -> bool { MOUTH_CLOSED_ERRORS.with(|c| c.get() < 10) }
  fn mouth_closed_budget_exhausted() -> bool { MOUTH_CLOSED_ERRORS.with(|c| c.get() >= 50) }
  loop {
    let mouth_source = gullet!()
      .runtime
      .as_ref()
      .map(|r| arena::pin(r.mouth.get_source()));
    if mouth_source == Some(context_mouth_source) {
      close_mouth(true)?;
      break;
    } else if gullet!().mouthstack.is_empty() {
      if should_emit_mouth_closed() {
        // `arena::to_string` clones the resolved &str into an owned String
        // BEFORE we hand it to Error! — a following `arena::pin` triggered
        // deep inside generate_message!/get_location() can grow the
        // interner's buffer and invalidate a borrowed &str (observed as
        // garbled, buffer-adjacent symbol content in 0906.1883 errors).
        let src = arena::to_string(context_mouth_source);
        Error!(
          "unexpected",
          "<closed>",
          "Mouth is unexpectedly already closed",
          s!("Reading from {src}, but it has already been closed.")
        );
      }
      record_mouth_closed_error();
      if mouth_closed_budget_exhausted() {
        Fatal!(
          Stomach,
          Recursion,
          "Too many unexpectedly-closed mouth errors (>50); gullet mouth-stack state is inconsistent"
        );
      }
      break;
    } else {
      let is_autoclosable = gullet!()
        .runtime
        .as_ref()
        .map(|r| r.autoclose)
        .unwrap_or(false);
      if is_autoclosable {
        // Auto-closable mouth (e.g. from \scantokens, raw_tex) — safe to close
        close_mouth(true)?;
      } else {
        // Non-autoclosable mouth that isn't our target — this means our target
        // mouth was already consumed. Don't close this mouth (it belongs to an
        // outer reading_from_mouth call). Just error and stop.
        if should_emit_mouth_closed() {
          let src = arena::to_string(context_mouth_source);
          Error!(
            "unexpected",
            "<closed>",
            "Mouth is unexpectedly already closed",
            s!(
              "Reading from {src}, but it has already been closed (found different non-closable mouth on top)."
            )
          );
        }
        record_mouth_closed_error();
        if mouth_closed_budget_exhausted() {
          Fatal!(
            Stomach,
            Recursion,
            "Too many unexpectedly-closed mouth errors (>50); gullet mouth-stack state is inconsistent"
          );
        }
        break;
      }
    }
  }
  Ok(results)
}

/// Check if there is more input to be read from the current mouth
pub fn has_more_input() -> bool {
  match runtime!() {
    Some(ref mut runtime) => runtime.mouth.has_more_input(),
    None => false,
  }
}

/// Obscure, but the only way I can think of to End!! (see \bye or \end{document})
/// Flush all sources (close all pending mouth's)
pub fn flush() {
  let mut g = gullet_mut!();
  if let Some(ref mut runtime) = g.runtime {
    runtime.mouth.finish();
  }
  while !g.mouthstack.is_empty() {
    if let Some(mut entry) = g.mouthstack.pop_front() {
      entry.mouth.finish();
    }
  }
  g.runtime = Some(MouthRuntime {
    mouth:     Mouth::default(),
    pushback:  Vec::with_capacity(128),
    autoclose: true,
  });
  g.mouthstack = VecDeque::new();
}

/// Execute a function with a mutable reference to the current mouth
pub fn with_mouth_mut<FnR, R>(caller: FnR) -> R
where FnR: FnOnce(Option<&mut Mouth>) -> R {
  let mut gullet = gullet_mut!();
  let mouth_opt = match gullet.runtime {
    None => None,
    Some(ref mut runtime) => Some(&mut runtime.mouth),
  };
  caller(mouth_opt)
}
