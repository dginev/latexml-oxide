use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashSet as HashSet;
use std::cell::{RefCell,RefMut};
use std::collections::VecDeque;
// use std::mem;
// use std::rc::Rc;
use string_interner::symbol::SymbolU32;

use crate::alignment::Alignment;
use crate::common::arena::{self,DONT_EXPAND_SYM};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::float::Float;
use crate::common::glue::{FillCode, Glue};
use crate::common::locator::Locator;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::{fixpoint, NumericOps, UNITY};
use crate::common::object::Object;
use crate::{state, DigestedData};
use crate::state::*;

use crate::definition::conditional::ConditionalType;
use crate::definition::register::{RegisterType, RegisterValue};
use crate::definition::Definition;
use crate::mouth::Mouth;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;

static DIGIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9]").unwrap());
static OCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-7]").unwrap());
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-F]").unwrap());
#[thread_local]
static DEFERRED_COMMANDS: Lazy<HashSet<SymbolU32>> = Lazy::new(||
  set!(arena::pin_static("\\the"), arena::pin_static("\\showthe"),
    arena::pin_static("\\unexpanded"), arena::pin_static("\\detokenize")));

// If it is a column ending token, Returns the token, a keyword and whether it is "hidden"
#[thread_local]
static COLUMN_ENDS : Lazy<[(Token,&'static str, bool); 6]> = Lazy::new(|| [    // besides T_ALIGN
  (T_CS!("\\cr"),           "cr",     false),
  (T_CS!("\\crcr"),         "crcr",   false),
  (T_CS!("\\hidden@cr"),    "cr",     true),
  (T_CS!("\\hidden@crcr"),  "crcr",   true),
  (T_CS!("\\hidden@align"), "insert", true),
  (T_CS!("\\span"),         "span",   false)]);

#[derive(PartialEq, Debug)]
pub struct MouthRuntime {
  pub autoclose: bool,
  pub mouth: Mouth,
  pub pushback: VecDeque<Token>,
}

#[derive(Debug, Default)]
pub struct Gullet {
  pub runtime: Option<MouthRuntime>,
  pub mouthstack: VecDeque<MouthRuntime>,
  pub pending_comments: VecDeque<Token>,
  pub token_limit: Option<usize>,
  pub pushback_limit: Option<usize>,
  pub progress: usize
}

#[thread_local]
pub static GULLET : Lazy<RefCell<Gullet>> = Lazy::new(|| RefCell::new(Gullet::default()));

macro_rules! gullet {
  () => ((*GULLET).borrow())
}
macro_rules! gullet_mut {
  () => ((*GULLET).borrow_mut())
}
macro_rules! runtime {
  () => ((*GULLET).borrow_mut().runtime)
}
macro_rules! runtime_mut {
  () => ((*GULLET).borrow_mut().runtime.as_mut())
}

/// Initialize (or reset, if reentrant) a Gullet to its default empty state
pub fn initialize_gullet() {
  let mut gullet = gullet_mut!();
  gullet.runtime = None;
  gullet.mouthstack = VecDeque::new();
  gullet.pending_comments = VecDeque::new();
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
  gullet.mouthstack.iter().any(|runtime| &runtime.mouth == mouth)
}
  

/// Push the `tokens` back into the input stream to be re-read.
pub fn unread(tokens: Tokens) {
  unread_vec(tokens.unlist());
}
/// Variant of `unread`, but drains the contents of `tokens` without taking ownership.
pub fn unread_mut(tokens: &mut Tokens) {
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    for token in tokens.unlist_mut().drain(..).rev() {
      runtime.pushback.push_front(token);
    }
  };
}
/// Unreads a single `Token` to the start of the token stream
pub fn unread_one(token: Token) {
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    runtime.pushback.push_front(token);
  };
}
/// Unreads a `Vec<Token>` to the start of the token stream
pub fn unread_vec(tokens: Vec<Token>) {
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    for token in tokens.into_iter().rev() {
      runtime.pushback.push_front(token);
    }
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
    pushback: VecDeque::with_capacity(128),
  });
}

pub fn close_mouth(forced: bool) -> Result<()> {
  let mut shift_from_mouthstack = false;
  let mut error_has_more_input = false;
  if let Some(ref mut runtime) = runtime!() {
    if !forced && (!runtime.pushback.is_empty()) || runtime.mouth.has_more_input() {
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
    while !runtime.mouth.is_eol() {
      if let Some(token) = runtime.mouth.read_token() {
        runtime.pushback.push_back(token);
      }
    }
    runtime.mouth.finish(); // then finish the mouth (it'll get closed on next read)
  }
}

//**********************************************************************
// Low-level readers: read token, read expanded token
//**********************************************************************
// # Get the next pending comment token (if any)
pub fn get_pending_comment() -> Option<Token> {
  gullet_mut!().pending_comments.pop_front()
}

/// Note that every char (token) comes through here (maybe even twice, through args parsing),
/// So, be Fast & Clean!  This method only reads from the current input stream (Mouth).

fn handle_template(
  mut alignment: RefMut<Alignment>,
  token: Token,
  vtype: &str,
  hidden: bool,
) -> Result<()> {
  // eprintln!("Halign: ALIGNMENT Column ended at {} type {vtype} [{}]",token.stringify(),
  // lookup_meaning(&token).unwrap());     . "@ " . ToString($self->getLocator))
  // if $LaTeXML::DEBUG{halign};

  //  Append expansion to end!?!?!?!
  local_current_token(token);
  let post = alignment.get_column_after();
  set_align_group_count(1000000);
  // ### NOTE: Truly fishy smuggling w/ \hidden@cr
  let arg_opt = if (vtype == "cr") && hidden {
    // \hidden@cr gets an argument as payload!!!!!
    Some(read_arg()?)
  } else {
    None
  };
  // eprintln!("Halign: column after {post}");// . ToString($post)) if $LaTeXML::DEBUG{halign};
  if (vtype == "cr" || vtype == "crcr")
    && alignment.is_in_row()
    && !alignment
      .current_row()
      .map(|v| v.is_pseudo())
      .unwrap_or(false)
  {
    unread_one(T_CS!("\\@row@after"));
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
  let Gullet {ref mut runtime, ref mut pending_comments, ..} = *gullet_mut!();
  let pushback = &mut runtime.as_mut().unwrap().pushback;
  // Check in pushback first....
  while let Some(pushback_token) = pushback.pop_front() {
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
    { // each time we try to read, do the defensive checks
      let gullet = gullet!();
      // If we're without a runtime, bail
      if gullet.runtime.is_none() { return Ok(None); }
      // some infinite loops are hard to predict and may be
      // better guarded against via a global token limit.
      if let Some(token_limit) = gullet.token_limit {
        if gullet.progress > token_limit {
          Fatal!(Timeout, TokenLimit, s!("Token limit of {token_limit} exceeded, infinite loop?"));
        }
      }
      if let Some(pushback_limit) = gullet.pushback_limit {
        if gullet.runtime.as_ref().unwrap().pushback.len() > pushback_limit {
          Fatal!(Timeout, PushbackLimit, s!("Pushback limit of {pushback_limit} exceeded, infinite loop?"));
        }
      }
    }
    // internal low-level reader that extracts a token from a mouth,
    // but always keeps comment tokens pending.
    next_token = read_internal_token();
    // ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;

    // Wow!!!!! See TeX the Program \S 309
    if let Some(ref nextt) = next_token {
      let mut check_dont_expand = true;
      // SHOULD count nesting of { }!!! when SCANNED (not digested)
      if (align_group_count() == 0) && has_reading_alignment() {
        if let Some((atoken, atype, ahidden)) = is_column_end(nextt) {
          check_dont_expand = false;
          let reading_alignment = get_reading_alignment().unwrap();
          if let DigestedData::Alignment(data) = reading_alignment.data() {
            handle_template(data.borrow_mut(), atoken, atype, ahidden)?;
          } else {
            return Err("reading_alignment should always contain DigestedData::Alignment".into());
          }
        }
      }
      if check_dont_expand {
        if nextt.code == Catcode::CS && nextt.text == *DONT_EXPAND_SYM {
          let _unexpanded = read_token();
          // Replace next token with a special \relax
          next_token = Some(T_CS!("\\special_relax"));
        }
        break;
      }
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
///     `toplevel` is doing TWO distinct things. When true:
/// * If a mouth is exhausted, move on to the containing mouth to continue reading
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
) -> Result<Option<Token>> {
  // toplevel should be true by default
  let toplevel = toplevel_opt.unwrap_or(true);
  let autoclose = toplevel;
  let for_evaluation = toplevel;
  loop {
    // internal low-level reader that extracts a token from a mouth,
    // but always keeps comment tokens pending.
    let next_token = read_internal_token();
    //ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;
    if next_token.is_none() {
      {let gullet = gullet!();
      if !autoclose
        || !gullet.runtime.as_ref().map(|r| r.autoclose).unwrap_or(false)
        || gullet.mouthstack.is_empty() {
        return Ok(None);
      }}
      close_mouth(false)?; // Next input stream.
      continue;
    }
    // we got a token
    let token = next_token.unwrap();
    if token.get_catcode() == Catcode::CS && token.text == *DONT_EXPAND_SYM {
      let unexpanded = read_token()?.unwrap();
      return Ok(Some(
        if for_conditional && unexpanded.code == Catcode::ACTIVE {
          unexpanded
        } else {
          T_CS!("\\special_relax")
        }));
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
      }};
    if let Some((reading_alignment, atype, ahidden)) = check_alignment_data {
      if let DigestedData::Alignment(data) = reading_alignment.data() {
        handle_template(data.borrow_mut(), token, atype, ahidden)?;
      } else {
        panic!("malformed alignmed was stored?");
      }
      // And *then* continue the main loop checks
    } else if token.get_catcode().is_active_or_cs() {
      match lookup_meaning(&token) {
        Some(Stored::Token(let_token)) => {
          return Ok(Some(if for_conditional {
            let_token
          } else {
            token
          }))
        },
        Some(Stored::None) | None => {
          if token.get_catcode() == Catcode::CS {
            return Ok(Some(generate_error_stub(&token)?)); // cs SHOULD have defn by now; report early.
          } else {
            return Ok(Some(token));
          }
        },
        Some(typed_defn) => {
          let defn = typed_defn.to_definition()
            .expect("token expansion requires the Stored item to implement trait Definition");
          if !defn.is_expandable() || (defn.is_protected() && !for_evaluation) {
            return Ok(Some(token));
          } else {
            local_current_token(token);
            let invoked = defn.invoke(false)?;
            // add the newly expanded tokens back into the gullet stream, in the ordinary case.
            unread(invoked);
            expire_current_token();
            continue;
          }
        },
      }
    } else {
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
    let tokens: Vec<Token> = runtime.pushback.drain(..).collect();

    // TODO
    // let markers : Vec<&Token> = tokens.iter().filter(|t:Token| t.get_catcode() ==
    // Catcode::MARKER).collect(); if !markers.is_empty() {    // Whoops, profiling markers!

    // @tokens = grep { $_->getCatcode != Catcode::MARKER } @tokens;    // Remove
    // map { LaTeXML::Core::Definition::stopProfiling($_, 'expand') } @markers;
    // }

    // If we still have peeked tokens, we ONLY want to combine it with the remainder
    // of the current line from the Mouth (NOT reading a new line)
    if !tokens.is_empty() {
      Some(
        Tokens::new(tokens).to_string()
          + &runtime.mouth.read_raw_line(true).unwrap_or_default(),
      )
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
    match read_x_token(Some(false), false)? {
      None => return Ok(None),
      Some(t) => {
        if t.get_catcode() != Catcode::SPACE {
          return Ok(Some(t));
        }
      },
    }
  }
}

/// `read_balanced` approximates TeX's scan_toks (but doesn't parse \def parameter lists)
/// and only optionally requires the openning "{".
/// It may return comments in the token lists.
/// it optionally (`do_expand`) expands while reading, but deferring \the and related.
/// The `is_macrodef` flag affects whether # parameters are "packed" for macro bodies.
/// If `require_open` is true, the opening T_BEGIN has not yet been read, and is required.
pub fn read_balanced(do_expand: bool, is_macrodef: bool, require_open:bool) -> Result<Tokens> {
  if !require_open {
    decrement_align_group_count();
  }
  local_align_group_count(1000000);
  // let startloc = if lookup_verbosity() > 0 { Some(get_locator()) } else { None };
  // Do we need to expand to get the { ???
  if require_open {
    let token_opt = if do_expand { read_x_token(Some(false),false)? }
    else { read_token()? };
    let is_open = match token_opt {
      None => false,
      Some(token) => {
        token.get_catcode() == Catcode::BEGIN ||
        state::lookup_meaning(&token) == Some(Stored::Token(T_BEGIN!()))
      }
    };
    if !is_open {
      Error!("expected", "{", s!("Expected opening '{{' got {token_opt:?}"));
      return Ok(Tokens!());
    }
  }
  let mut tokens = Vec::new();
  let mut level  = 1;
  loop {
    // we'll keep comments in the result
    let mut next_token = None;
    if ! gullet!().pending_comments.is_empty() {
      tokens.extend(gullet_mut!().pending_comments.drain(..));
    }
    // Examine pushback first
  while let Some(pushback_token) = runtime_mut!().unwrap().pushback.pop_front() {
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
        Catcode::CS if token.text == *DONT_EXPAND_SYM => {
          if let Some(next_t) = read_token()? {
            tokens.push(next_t);  // Pass on NEXT token, unchanged.
          }
        },
        Catcode::END => {
          level -= 1;
          if level <= 0 { break; }
          tokens.push(token);
        },
        Catcode::BEGIN => {
          level +=1 ;
          tokens.push(token);
        }
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
          if do_expand && cc.is_active_or_cs() {
            let meaning_opt = lookup_meaning(&token);
            if let Some(defn) = meaning_opt.as_ref().and_then(|item| item.to_definition()) {
              if defn.is_expandable() && !defn.is_protected() {
                local_current_token(token);
                let expansion = defn.invoke(false)?;
                if expansion.is_empty() {
                  expire_current_token();
                  continue;
                }
                // If a special \the type command, push the expansion directly into the result
                // Well, almost directly: handle any MARKER tokens now, and possibly un-pack T_PARAM
                if DEFERRED_COMMANDS.contains(&defn.get_cs().text) {
                  for t in expansion.unlist() {
                    match t.get_catcode() {
                      Catcode::MARKER => handle_marker(t),
                      Catcode::PARAM if is_macrodef => {// "unpack" to cover the packParameters at end!
                        tokens.push(t);
                        tokens.push(t);
                      },
                      _ => tokens.push(t)
                    }
                  }
                } else {
                  // otherwise, prepend to pushback to be expanded further.
                  unread(expansion);
                }
                expire_current_token();
                continue;
              }
            } else if cc == Catcode::CS && meaning_opt.is_none() {
              // cs SHOULD have defn by now; report early!
              generate_error_stub(&token)?;
            }
          }
          // if no special handling triggered above, just return the token
          tokens.push(token);
        }
      }
    }
  }
  if level > 0 {
    // TODO: The current implementation has a limitation where if the balancing end is in a different mouth,
    //       it will not be recognized.
    // TODO: also, add the startloc details
    // my $loc_message = $startloc ? ("Started at " . ToString($startloc)) : ("Ended at " . ToString($self->getLocator));
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
pub fn read_keyword(keywords: &[&str]) -> Result<Option<String>> {
  skip_spaces()?;
  for keyword in keywords.iter() {
    let mut to_match: VecDeque<char> = keyword.to_uppercase().chars().collect();
    let mut matched = Vec::new();
    while !to_match.is_empty() {
      if let Some(tok) = read_x_token(Some(false), false)? {
        let cmp_tok = tok.with_str(|s| s.to_uppercase());
        matched.push(tok);
        if cmp_tok == to_match[0].to_string() {
          to_match.pop_front();
        } else {
          break;
        }
      } else {
        break;
      }
    }
    if to_match.is_empty() {
      // All matched!!!
      return Ok(Some(keyword.to_string()));
    } else {
      unread(matched.into()); // Put 'em back and try next!
    }
  }
  Ok(None)
}

/// Return a (balanced) sequence tokens until a match against one of the Tokens in @delims.
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
      if token == *want {
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
          let balanced_arg = read_balanced(false,false,false)?;
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
        if token.get_catcode() == Catcode::BEGIN {
          // read balanced, and refill ring.
          nbraces += 1;
          for r_token in ring {
            tokens.push(r_token);
          }
          tokens.push(token);
          let balanced_arg = read_balanced(false,false,false)?;
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
pub fn read_until_token(t: Token) -> Result<Tokens> {
  read_until(&Tokens!(t))
}
/// reads until it encounters a Catcode::BEGIN token
pub fn read_until_brace() -> Result<Option<Tokens>> {
  let mut tokens = Vec::new();
  while let Some(token) = read_token()? {
    if token.get_catcode() == Catcode::BEGIN {
      if let Some(runtime) = runtime_mut!() {
        runtime.pushback.push_front(token); // Unread
      } else {
        fatal!(Mouth, NotFound, "No Mouth in read_until_brace")
      }
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
/// reads and discards tokens, until it encounters a conditional, if any
pub fn read_next_conditional() -> Result<Option<(Token, ConditionalType)>> {
  while let Some(token) = read_token()? {
    if token.get_catcode().is_active_or_cs() {
      if let Some(cond_type) = lookup_conditional(&token) {
        return Ok(Some((token, cond_type)));
      }
    }
  }
  Ok(None)
}

///**********************************************************************
/// Higher-level readers: Read various types of things from the input:
///  tokens, non-expandable tokens, args, Numbers, ...
///**********************************************************************
pub fn read_arg() -> Result<Tokens> {
  match read_non_space()? {
    None => Ok(Tokens!()),
    Some(token) => if token.get_catcode() == Catcode::BEGIN {
        read_balanced(false,false,false)
      } else {
        Ok(Tokens!(token))
      }
  }
}
/// Read and return a LaTeX optional argument; returns C<$default> if there is no '[',
/// otherwise the contents of the [].
/// Note that this returns an empty array if [] is present,
/// i.e. "[contents]" in TeX will lead to Tokens(contents), otherwise returns None
pub fn read_optional(
  default: Option<Tokens>,
) -> Result<Option<Tokens>> {
  match read_non_space()? {
    None => Ok(None),
    Some(t) => {
      if t.get_catcode() == Catcode::OTHER && t.get_sym() == arena::pin_static("[") {
        Ok(Some(read_until(&Tokens!(T_OTHER!("]")))?))
      } else {
        unread_one(t);
        Ok(default)
      }
    },
  }
}

pub fn if_next(token: Token) -> Result<bool> {
  let mut is_next = false;
  if let Some(tok) = read_token()? {
    is_next = tok == token;
    unread_one(tok);
  }
  Ok(is_next)
}

//**********************************************************************
//  Numbers, Dimensions, Glue
// See TeXBook, Ch.24, pp.269-271.
//**********************************************************************

pub fn read_value(
  value_type: RegisterType,
) -> Result<RegisterValue> {
  match value_type {
    RegisterType::Number => Ok(read_number()?.into()),
    RegisterType::Dimension => Ok(read_dimension()?.into()),
    RegisterType::MuDimension => Ok(read_mu_dimension()?.into()),
    RegisterType::Glue => Ok(read_glue()?.into()),
    RegisterType::MuGlue => Ok(read_mu_glue()?.into()),
    RegisterType::Tokens => Ok(read_tokens_value()?.into()),
    // TODO: unwrap should be a proper error, value is expected
    RegisterType::Token => Ok(read_token()?.unwrap().into()),
    RegisterType::CharDef => Ok(read_number()?.into()),
    RegisterType::Any => Ok(read_arg()?.into()),
  }
}

pub fn read_register_value(
  value_type: RegisterType,
) -> Result<Option<RegisterValue>> {
  match read_x_token(None, false)? {
    None => Ok(None),
    Some(token) => {
      if let Some(defn) = lookup_register_definition(&token) {
        if let Some(mut register_type) = defn.register_type() {
          if register_type == RegisterType::CharDef {
            // CharDefs treated as numbers here
            register_type = RegisterType::Number;
          }
          if register_type == value_type {
            let args = defn.read_arguments()?;
            Ok(defn.value_of(args))
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

/// Match the input against one of the Token or Tokens in @choices; return the matching one or
/// undef.
pub fn read_match(choices: &[&Tokens]) -> Result<Option<Tokens>> {
  for choice in choices {
    let mut to_match: Vec<&Token> = choice.unlist_ref().iter().rev().collect();
    let mut matched = Vec::new();
    while !to_match.is_empty() {
      match read_token()? {
        None => break,
        Some(token) => {
          let cc = token.get_catcode();
          let was_last_match = Some(&&token) == to_match.last();
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
                // Unread non-space and end
                match runtime_mut!() {
                  Some(mouth) => mouth.pushback.push_front(space_token),
                  None => fatal!(Mouth, NotFound, "No Mouth in read_match"),
                }
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
      for matched_token in matched.into_iter().rev() {
        match runtime_mut!() {
          Some(mouth) => mouth.pushback.push_front(matched_token), // Put 'em back and try next!
          None => fatal!(Mouth, NotFound, "No Mouth in read_match"),
        }
      }
    }
  }
  Ok(None)
}

///======================================================================
/// Integer, Number
///======================================================================
/// <number> = <optional signs><unsigned number>
/// <unsigned number> = <normal integer> | <coerced integer>
/// <coerced integer> = <internal dimen> | <internal glue>
pub fn read_number() -> Result<Number> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1 } else { 1 };
  if let Some(n) = read_normal_integer()? {
    if is_negative {
      Ok(n.negate())
    } else {
      Ok(n)
    }
  } else if let Some(n) = read_internal_dimension()? {
    Ok(Number::new(s * n.value_of()))
  } else if let Some(n) = read_internal_glue()? {
    Ok(Number::new(s * n.value_of()))
  } else {
    let next = read_token()?;
    let message = s!(
      "Missing number, treated as zero while processing {:?}, next token is {:?}",
      get_current_token().unwrap(),
      next
    );
    Warn!("expected", "<number>", message);
    if let Some(next) = next {
      unread_one(next);
    }
    Ok(Number::new(0))
  }
}

/// <normal integer> = <internal integer> | <integer constant>
///   | '<octal constant><one optional space> | "<hexadecimal constant><one optional space>
///   | `<character token><one optional space>
pub fn read_normal_integer() -> Result<Option<Number>> {
  match read_x_token(None, false)? {
    None => Ok(None),
    Some(token) => {
      let cc = token.get_catcode();
      let mut text = token.to_string();
      if cc == Catcode::OTHER && text.chars().all(|c| c.is_ascii_digit()) {
        // Read decimal literal
        text.push_str(&read_digits(&DIGIT_RE, true)?);
        Ok(Some(Number::new(text.parse::<i64>().expect(&text))))
      } else if token == T_OTHER!("'") {
        // Read Octal literal
        let decimal = i64::from_str_radix(&read_digits(&OCT_RE, true)?, 8)?;
        Ok(Some(Number::new(decimal)))
      } else if token == T_OTHER!("\"") {
        //  Read Hex literal
        let decimal = i64::from_str_radix(&read_digits(&HEX_RE, true)?, 16)?;
        Ok(Some(Number::new(decimal)))
      } else if token == T_OTHER!("`") {
        //  Read Charcode
        let mut s = match read_token()? {
          None => String::new(),
          Some(next) => next.to_string(),
        };
        if s.starts_with('\\') {
          s.remove(0);
        }
        let s_char = s.chars().next().unwrap();
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
  let mut token = read_x_token(None, false)?;
  if token.is_some() && token.as_ref().unwrap().get_sym() == arena::pin_static(".") {
    string = s!("{string}.{}", read_digits(&DIGIT_RE, true)?);
    token = read_x_token(None, false)?;
  }
  let n_opt: Option<f64> = if !string.is_empty() {
    if let Some(t) = token {
      if t.get_catcode() != Catcode::SPACE {
        unread_one(t);
      }
    }
    Some(string.parse::<f64>().expect(&string))
  } else {
    if let Some(t) = token {
      unread_one(t); // Unread
    }
    read_normal_integer()?
      .map(|v| v.value_of() as f64)
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
// <dimen> = <optional signs><unsigned dimen>
// <unsigned dimen> = <normal dimen> | <coerced dimen>
// <coerced dimen> = <internal glue>
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
    let message = s!(
      "Missing number, treated as zero. while processing {:?}",
      get_current_token().unwrap()
    );
    Warn!("expected", "<number>", message);
    Ok(Dimension::new(0))
  }
}

// <unit of measure> = <optional spaces><internal unit>
//     | <optional true><physical unit><one optional space>
// <internal unit> = em <one optional space> | ex <one optional space>
//     | <internal integer> | <internal dimen> | <internal glue>
// <physical unit> = pt | pc | in | bp | cm | mm | dd | cc | sp

/// Read a unit, returning the equivalent number of scaled points,
fn read_unit() -> Result<Option<f64>> {
  let unit_opt = if let Some(u) = read_keyword(&["ex", "em"])? {
    skip_one_space()?;
    Some(convert_unit(&u))
  } else if let Some(u) = read_internal_integer()? {
    Some(u.value_of() as f64) // These are coerced to number=>sp
  } else if let Some(u) = read_internal_dimension()? {
    Some(u.value_of() as f64)
  } else if let Some(u) = read_internal_glue()? {
    Some(u.value_of() as f64)
  } else {
    read_keyword(&["true"])?; // But ignore, we're not bothering with mag...
    if let Some(u) = read_keyword(
      &["pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp", "px"],
        )? {
      skip_one_space()?;
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
    if is_negative {
      Ok(n.negate())
    } else {
      Ok(n)
    }
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

pub fn read_rubber(
  mu: bool,
) -> Result<(Option<i64>, Option<FillCode>)> {
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
    Warn!(
      "expected",
      "<mudimen>",
      "Expecting mudimen; assuming 0"
    );
    Ok(MuDimension::new(0))
  }
}

pub fn read_mu_unit() -> Result<Option<i64>> {
  if read_keyword(&["mu"])?.is_some() {
    skip_one_space()?;
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

/// Apparent behaviour of a token value (ie \toks#=<arg>)
pub fn read_tokens_value() -> Result<Tokens> {
  match read_non_space()? {
    None => Ok(Tokens!()),
    Some(token) => {
      if token.get_catcode() == Catcode::BEGIN {
        Ok(read_balanced(false,false,false)?)
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

pub fn skip_one_space() -> Result<()> {
  if let Some(token) = read_token()? {
    if token.get_catcode() != Catcode::SPACE {
      unread_one(token);
    }
  }
  Ok(())
}

//======================================================================
// some helpers...

// <optional signs> = <optional spaces> | <optional signs><plus or minus><optional spaces>
// returns false if None, or positive, true if negative
fn read_optional_signs() -> Result<bool> {
  let mut sign = false;
  while let Some(t) = read_x_token(None, false)? {
    let sym = t.get_sym();
    if sym == arena::pin_static("-") {
      sign = !sign;
    } else if (sym != arena::pin_static("+")) && t.get_catcode() != Catcode::SPACE {
      unread_one(t); // Unread and end
      break;
    }
  }
  Ok(sign)
}

fn read_digits(range_regex: &Regex, skip: bool) -> Result<String> {
  let mut result = String::new();
  while let Some(token) = read_x_token(None, false)? {
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
      if !(skip && token.get_catcode() == Catcode::SPACE) {
        unread_one(token);
      }
      break;
    }
  }
  Ok(result)
}

// <factor> = <normal integer> | <decimal constant>
// <decimal constant> = . | , | <digit><decimal constant> | <decimal constant><digit>
// Return a number (Rust f64 number)
fn read_factor() -> Result<Option<f64>> {
  let mut factor = read_digits(&DIGIT_RE, false)?;
  let mut token_opt = read_x_token(None, false)?;
  if let Some(ref token) = token_opt {
    let sym = token.get_sym();
    if sym == arena::pin_static(".") || sym == arena::pin_static(",") {
      factor = s!("{}.{}", factor, read_digits(&DIGIT_RE, false)?);
      token_opt = read_x_token(None, false)?;
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
  reading_from_mouth(
    Mouth::default(),
    move || -> Result<Tokens> {
      { unread(tokens); }
      let mut expanded = Vec::new();
      while let Some(t) = read_x_token(Some(false), false)? {
        expanded.push(t);
      }
      Ok(Tokens::new(expanded))
    },
  )
}

pub fn is_column_end(token: &Token) -> Option<(Token, &'static str, bool)> {
  match token.get_catcode() {
    Catcode::ALIGN => Some((*token, "align", false)),
    Catcode::CS | Catcode::ACTIVE => {
      // Embedded version of Equals, knowing both are tokens
      let defn = lookup_meaning(token)
        .unwrap_or_else(|| Stored::Token(*token));
      for end in *COLUMN_ENDS {
        let e = &end.0;
        // Would be nice to cache the defns, but don't know when they're present & constant!
        if defn
          == lookup_meaning(e)
            .unwrap_or_else(|| Stored::Token(*e))
        {
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
      // let alignment = lookup_alignment();
      // Debug("Halign $alignment: alignment => 0") if $LaTeXML::DEBUG{halign};
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
/// This reads ONLY from that mouth (or any mouth openned by code in that source),
/// and the mouth should end up empty afterwards, and only be closed here.
pub fn reading_from_mouth<R, FnR>(
  mouth: Mouth,
  reader: FnR,
) -> Result<R>
where
  FnR: FnOnce() -> Result<R>,
{
  let context_mouth_source = arena::pin(mouth.get_source());
  open_mouth(mouth, false); // only allow mouth to be explicitly closed here.
  let results: R = { reader()? };
  // `mouth` must still be open, with (at worst) empty autoclosable mouths in front of it
  loop {
    let mouth_source = gullet!().runtime.as_ref()
      .map(|r| arena::pin(r.mouth.get_source()));
    if mouth_source == Some(context_mouth_source) {
      close_mouth(true)?;
      break;
    } else if gullet!().mouthstack.is_empty() {
      Error!(
        "unexpected",
        "<closed>",
        "Mouth is unexpectedly already closed",
        arena::with(context_mouth_source,|source| s!("Reading from {source}, but it has already been closed."))
      );
      break;
    } else {
      let has_input_remaining = {
        if let Some(ref mut runtime) = runtime!() {
          !runtime.autoclose
            || !runtime.pushback.is_empty()
            || runtime.mouth.has_more_input()
        } else { false }};
      if has_input_remaining {
        let next = read_token()?.unwrap();
        Error!(
          "unexpected",
          next,
          s!("Unexpected input remaining: '{next}'"),
          arena::with(context_mouth_source,|source|
            s!("Finished reading from {source}, but it still has input."))
        );
        {
          if let Some(ref mut runtime) = runtime!() {
            runtime.mouth.finish();
          }
        }
        close_mouth(true)?;
      }
      // ?? if we continue?
      else {
        close_mouth(false)?;
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
    mouth: Mouth::default(),
    pushback: VecDeque::with_capacity(128),
    autoclose: true,
  });
  g.mouthstack = VecDeque::new();
}

/// Execute a function with a mutable reference to the current mouth
pub fn with_mouth_mut<FnR,R>(caller: FnR) -> R
where FnR: FnOnce(Option<&mut Mouth>) -> R {
  let mut gullet = gullet_mut!();
  let mouth_opt = match gullet.runtime {
    None => None,
    Some(ref mut runtime) => Some(&mut runtime.mouth),
  };
  caller(mouth_opt)
}
