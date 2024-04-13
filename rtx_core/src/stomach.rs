use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use once_cell::sync::Lazy;

use crate::common::arena;
use crate::common::arena::SymHashMap as HashMap;
use crate::common::error::*;
use crate::common::font;
use crate::common::font::Font;
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::Definition;
use crate::list::List;
use crate::mouth::{Mouth, MouthOptions};
use crate::state::*;
use crate::tbox::*;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{Digested, TexMode, gullet};

static MAXSTACK: usize = 200;

/// The Stomach is responsible for digesting tokens into boxes, lists, etc.
#[derive(Default)]
pub struct Stomach {
  /// currently invoked tokens
  pub token_stack: Vec<Token>,
  /// tracks the tokens of boxing groups(?)
  pub boxing: Vec<Token>,
  /// localized box lists for stacked digestion calls
  localized_box_list: Vec<Vec<Digested>>,
  /// collects the intermediate boxes resulting from a `digest` call.
  pub box_list: Vec<Digested>,
}

#[thread_local]
pub static STOMACH : Lazy<RefCell<Stomach>> = Lazy::new(|| RefCell::new(Stomach::default()));

macro_rules! stomach {
  () => ((*STOMACH).borrow())
}
macro_rules! stomach_mut {
  () => ((*STOMACH).borrow_mut())
}

impl Stomach {
  /// Initialize various stomach parameters, preload, etc.
  pub fn initialize(&mut self) {
    self.boxing      = Vec::new();
    self.token_stack = Vec::new();
    self.box_list = Vec::new();
    self.localized_box_list = Vec::new();

    assign_value("MODE", "text", Some(Scope::Global));
    assign_value("IN_MATH", false, Some(Scope::Global));
    assign_value("PRESERVE_NEWLINES", Stored::Int(1), Some(Scope::Global));
    assign_value(
      "afterGroup",
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    );
    assign_value("afterAssignment", Stored::None, Some(Scope::Global)); // undef ???
    assign_value(
      "groupInitiator",
      "Initialization",
      Some(Scope::Global),
    );
    // Setup default fonts.
    assign_value("font", Font::text_default(), Some(Scope::Global));
    assign_value("mathfont", Font::math_default(), Some(Scope::Global));
  }
  //**********************************************************************
}

/// steal the previously digested boxes from the current level.
pub fn regurgitate() -> Vec<Digested> { stomach_mut!().box_list.drain(..).collect() }

//**********************************************************************
// Maintaining state
//**********************************************************************
// state changes that the Stomach needs to moderate and know about (?)

//======================================================================
// Dealing with TeX's bindings & grouping.
// Note that lookups happen more often than bgroup/egroup (which open/close frames).

/// Adds a new stack frame for a TeX group.
pub fn push_stack_frame(nobox: bool) {
  let current_token = get_current_token().unwrap();
  push_frame();
  assign_value(
    "beforeAfterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Local),
  ); // ALWAYS bind this!
  assign_value(
    "afterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Local),
  ); // ALWAYS bind this!
  assign_value("afterAssignment", Stored::None, Some(Scope::Local)); // ALWAYS bind this!
  assign_value("groupNonBoxing", nobox, Some(Scope::Local)); // ALWAYS bind this!
  assign_value("groupInitiator", current_token, Some(Scope::Local));
  assign_value(
    "groupInitiatorLocator",
    gullet::get_locator(),
    Some(Scope::Local),
  );
  if !nobox {
    // For begingroup/endgroup
    stomach_mut!().boxing.push(current_token)
  }
}
/// Removes the last/current stack frame, ending a TeX group
pub fn pop_stack_frame(nobox: bool) -> Result<()> {
  if let Some(Stored::VecDequeStored(beforeafter)) = remove_value("beforeAfterGroup") {
    if !beforeafter.is_empty() {
      let mut result = Vec::new();
      for beforeafter_frame in beforeafter.into_iter() {
        match beforeafter_frame {
          Stored::Tokens(frametoks) => result.push(frametoks.be_digested()?),
          Stored::Token(frametok) => result.push(frametok.be_digested()?),
          _ => {
            // TODO: Anything but Tokens in beforeAfterGroup?
            dbg!(beforeafter_frame);
            todo!();
          },
        }
      }
      // TODO
      // if (my ($x) = grep { !$_->isaBox } @result) {
      // Error('misdefined', $x, $self, "Expected a Box|List|Whatsit, but got '" . Stringify($x) .
      // "'"); @result = (makeMisdefinedError(@result)); }
      {
        stomach_mut!().box_list.extend(result);
      }
    }
  }
  let after = remove_value("afterGroup");
  pop_frame()?;
  if !nobox {{
    stomach_mut!().boxing.pop(); // For begingroup/endgroup
  }}
  if let Some(Stored::VecDequeStored(after_entries)) = after {
    for entry in after_entries.into_iter().rev() {
      match entry {
        Stored::Tokens(t) => gullet::unread(t),
        Stored::Token(t) => gullet::unread_one(t),
        other => panic!(r"\aftergroup should be used with tokens, got instead: {other:?}"),
      };
    }
  }
  Ok(())
}

/// explain the current frame
pub fn current_frame_message() -> String {
  let target = if is_value_bound("MODE", Some(0)) {
    // SET mode in CURRENT frame ?
    Cow::Owned(s!("mode-switch to {}", lookup_string("MODE")))
  } else if lookup_bool("groupNonBoxing") {
    // Current frame is a non-boxing group?
    Cow::Borrowed("non-boxing group")
  } else {
    Cow::Borrowed("boxing group")
  };

  let initiator = if let Some(t) = lookup_token("groupInitiator") {
    t.stringify()
  } else {
    String::new()
  };
  //   TODO:
  //   . " " . ToString(state!().lookup_value('groupInitiatorLocator'));
  s!("current frame is {} due to {}", target, initiator)
}

//======================================================================
// Grouping pushes a new stack frame for binding definitions, etc.
//======================================================================

/// Begin a new level of binding by pushing a new stack frame,
/// and a new level of boxing the digested output.
pub fn bgroup() {
  push_stack_frame(false);
  // NOTE: This is WRONG; should really only track "scanned" (not digested) braces
  // Alas, there're too many code structuring differences between TeX and LaTeXML
  // to find all the places to manage it.
  // So, let's try this for now...
  // was $LaTeXML::ALIGN_STATE
  increment_align_group_count();
}
/// End a level of binding by popping the last stack frame,
/// undoing whatever bindings appeared there, and also
/// decrementing the level of boxing.
pub fn egroup() -> Result<()> {
  if lookup_bool("groupNonBoxing") {
    // or group was opened with \begingroup
    Error!(
      "unexpected",
      get_current_token().unwrap(),
      "Attempt to close boxing group"
    );
  } else {
    // Don't pop if there's an error; maybe we'll recover?
    pop_stack_frame(false)?;
  }
  decrement_align_group_count();
  Ok(())
}
/// Begin a new level of binding by pushing a new stack frame.
pub fn begingroup() { push_stack_frame(true); }
/// End a level of binding by popping the last stack frame,
/// undoing whatever bindings appeared there.
pub fn endgroup() -> Result<()> {
  if !lookup_bool("groupNonBoxing") {
    // or group was opened with \bgroup
    Error!(
      "unexpected",
      get_current_token().unwrap().to_string(),
      s!(
        "Attempt to close non-boxing group; {}",
        current_frame_message()
      )
    );
  } else {
    pop_stack_frame(true)?;
  }
  Ok(())
}

//======================================================================
// Mode (minimal so far; math vs text)
// Could (should?) be taken up by Stomach by building horizontal, vertical or math lists ?

/// This sets the mode without doing any grouping (NOR does it stack the modes!!)
/// Useful for environments, where the group has already been established.
/// (presumably, in the long run, modes & groups should be much less coupled)
pub fn set_mode(mode: &str) -> Result<()> {
  let prevmode = lookup_string("MODE");
  let ismath = mode.ends_with("math");
  assign_value("MODE", arena::pin(mode), Some(Scope::Local));
  assign_value("IN_MATH", ismath, Some(Scope::Local));
  if mode == prevmode {
  } else if ismath {
    let curfont = lookup_font().unwrap();
    // When entering math mode, we set the font to the default math font,
    // and save the text font for any embedded text.
    assign_value("savedfont", curfont.clone(), Some(Scope::Local));
    // see get_script_level()
    assign_value("script_base_level", stomach!().boxing.len(), None);
    let isdisplay = mode.starts_with("display");
    let new_font = lookup_mathfont().unwrap().merge(Font {
      color: curfont.color.clone(),
      bg: curfont.bg.clone(),
      size: curfont.size,
      mathstyle: if isdisplay {
        Some("display".into())
      } else {
        Some("text".into())
      },
      ..Font::default()
    });
    assign_font(Rc::new(new_font), Some(Scope::Local));
  } else {
    let curfont = lookup_font().unwrap();
    // When entering text mode, we should set the font to the text font in use before the math
    // but inherit color and size
    let saved_opt = lookup_value("savedfont");
    if let Some(Stored::Font(saved_font)) = saved_opt {
      assign_font(
        Rc::new(saved_font.merge(Font {
          color: curfont.color.clone(),
          bg: curfont.bg.clone(),
          size: curfont.size,
          ..Font::default()
        })),
        Some(Scope::Local),
      );
    }
  }
  Ok(())
}

/// Begin processing in `mode`; one of "text", "display-math" or "inline-math".
/// This also begins a new level of grouping and switches to a font
/// appropriate for the mode.
pub fn begin_mode(mode: &str) -> Result<()> {
  push_stack_frame(false); // Effectively bgroup
  set_mode(mode)?;
  Ok(())
}
/// End processing in `mode`; an error is signalled if `stomach` is not
/// currently in `mode`.  This also ends a level of grouping.
pub fn end_mode(mode: &str) -> Result<()> {
  // Last stack frame was NOT a mode switch!?!?!
  if !is_value_bound("MODE", Some(0)) || (lookup_string_sym("MODE") != arena::pin(mode)) {
    // Or was a mode switch to a different mode
    let message = s!(
      "Attempt to end mode `{}` in `{}`",
      mode,
      lookup_string("MODE")
    );
    let category = match get_current_token() {
      Some(ref token) => token.to_string(),
      None => String::from("mode"),
    };
    Error!("unexpected", category, &message); // self.currentFrameMessage);
  } else {
    // Don"t pop if there"s an error; maybe we'll recover?
    pop_stack_frame(false)?;
  } // Effectively egroup.
  Ok(())
}

pub fn new_local_box_list() {
  let mut buffer = Vec::new();
  let mut stomach = stomach_mut!();
  std::mem::swap(&mut stomach.box_list, &mut buffer);
  stomach.localized_box_list.push(buffer);
}
pub fn expire_local_box_list() -> Vec<Digested> {
  let mut stomach = stomach_mut!();
  let mut buffer = stomach.localized_box_list.pop().unwrap_or_default();
  std::mem::swap(&mut stomach.box_list, &mut buffer);
  buffer
}

pub fn extend_box_list<I>(arg: I) where I: IntoIterator<Item = Digested> {
  stomach_mut!().box_list.extend(arg)
}
pub fn push_box_list(arg: Digested) {
  stomach_mut!().box_list.push(arg)
}
pub fn pop_box_list() -> Option<Digested> {
  stomach_mut!().box_list.pop()
}

// **********************************************************************
// Digestion
// **********************************************************************

/// Digest a list of tokens independent from any current Gullet.
/// Typically used to digest arguments to primitives or constructors.
/// Returns a List containing the digested material.
pub fn digest<T: Into<Tokens>>(
  tokens: T
) -> Result<Digested> {
  let tokens: Tokens = tokens.into();
  if tokens.is_empty() {
    return Ok(Digested::default());
  }
  gullet::reading_from_mouth(Mouth::default(), || {
    gullet::unread(tokens);
    clear_prefixes(); // prefixes shouldn't apply here.
    let mode = if lookup_bool("IN_MATH") {
      TexMode::Math
    } else {
      TexMode::Text
    };
    let initdepth = stomach!().boxing.len();
    let depth = initdepth;
    new_local_box_list();
    while let Some(token) = match gullet::get_pending_comment() {
      Some(comment) => Some(comment),
      None => gullet::read_x_token(Some(true), false)?
    }
    {
      // Done if we run out of tokens
      let invoked = invoke_token(&token)?;
      extend_box_list(invoked);

      if initdepth > stomach!().boxing.len() {
        // if we've closed the initial mode.
        break;
      }
      if initdepth < depth {
        // TODO
        fatal!(Internal, EoF, "We've fallen off the end, somehow !?!?!?");
        //     Fatal('internal', '<EOF>', self,
        //       "We've fallen off the end, somehow!?!?!",
        //       "Last token " . ToString($LaTeXML::CURRENT_TOKEN)
        //         . " (Boxing depth was $initdepth, now $depth: Boxing generated by "
        //         . join(', ', map { ToString($_) } @{ $self{boxing} }))
        //       if $initdepth < $depth;
      }
    }

    let mut digested_list = List::new(expire_local_box_list());
    digested_list.mode = Some(mode);
    digested_list.into()
  })
}

/// Return the digested `List` after reading and digesting a body from the its Gullet.
/// The body extends until the current level of boxing or environment is closed.
pub fn digest_next_body(
  terminal_opt: Option<Token>,
) -> Result<Vec<Digested>> {

  let start_location = { gullet::get_locator() };

  let init_depth = { stomach!().boxing.len() };
  let mut found_token = false;
  let mut found_terminal = false;
  new_local_box_list();
  let alignment_opt = lookup_alignment();
  // TODO: bookkeep for "expected" warning
  //let mut aug = Vec::new();

  // try reading a executable token
  while let Some(token) = match gullet::get_pending_comment() {
    Some(comment) => Some(comment),
    None => gullet::read_x_token(Some(true), false)?
  }
  {
    // done if we run out of tokens
    found_token = true;
    // first, check for alignment case
    if alignment_opt.is_some()
      && !stomach!().box_list.is_empty()
      && (token == T_ALIGN!()
        || token == T_CS!("\\cr")
        || token == T_CS!("\\hidden@cr")
        || token == T_CS!("\\hidden@crcr"))
    {
      // at least \over calls in here without the intent to passing through the alignment.
      // So if we already have some digested boxes available, return them here.
      gullet::unread_one(token);
      return Ok(expire_local_box_list());
    }
    // normal case
    let invoked = invoke_token(&token)?;
    extend_box_list( invoked);

    if let Some(ref terminal) = terminal_opt {
      if &token == terminal {
        found_terminal = true;
        break;
      }
    }
    if init_depth > stomach!().boxing.len() {
      break;
    }
  }

  if let Some(ref terminal) = terminal_opt {
    if !found_terminal {
      let message = s!(
        "body should have ended with {:?}. current body started at {:?}",
        terminal,
        start_location
      );
      Warn!("expected", terminal, message);
    }
  }
  // and add a Dummy `trailer' if none explicit.
  if !found_token {
    push_box_list(Digested::from(Tbox::default()));
    // info!(target:"digest_next_body","no_token");
  }
  Ok(expire_local_box_list())
}


/// a convenience function for including chunks of raw TeX (or LaTeX) code
/// It is useful for copying portions of the normal
/// implementation that can be handled simply using macros and primitives.
pub fn raw_tex(text: &str) -> Result<()> {
  // It could be as simple as this, except if catcodes get changed, it's too late!!!
  //  Digest(TokenizeInternal($text));
  let savedcc = lookup_catcode('@').unwrap_or(Catcode::OTHER);
  assign_catcode('@', Catcode::LETTER, None);
  let raw_tex_mouth = Mouth::new(
    text,
    Some(MouthOptions {
      fordefinitions: true,
      // at_letter: true,
      ..MouthOptions::default()
    }),
  )?;
  gullet::reading_from_mouth(raw_tex_mouth, || -> Result<()> {
    while let Some(token) = gullet::read_x_token(Some(false), false)? {
      if token.get_catcode() != Catcode::SPACE {
        invoke_token(&token)?;
      }
    }
    Ok(())
  })?;

  assign_catcode('@', savedcc, None);
  Ok(())
}

/// Invoke a token;
/// If it is a primitive or constructor, the definition will be invoked,
/// possibly arguments will be parsed from the Gullet.
/// Otherwise, the token is simply digested: turned into an appropriate box.
/// Returns a list of boxes/whatsits.
pub fn invoke_token<'a>(
  input_token: &'a Token,
) -> Result<Vec<Digested>> {
  let mut maybe_token: Option<Cow<'a, Token>> = Some(Cow::Borrowed(input_token));
  // Overly complex, but want to avoid recursion/stack
  let mut result: Vec<Digested> = Vec::new();
  // INVOKE:
  while maybe_token.is_some() {
    let token = maybe_token.take().unwrap().into_owned();
    // info!(target:"invoke_token", "{:?}", token);
    local_current_token(token);
    { stomach_mut!().token_stack.push(token); }
    if { stomach!().token_stack.len() } > MAXSTACK {
      fatal!(
        Stomach,
        Recursion,
        s!(
          "Excessive recursion(?): Tokens on stack: {:?}",
          stomach!().token_stack
        )
      );
    }
    result = Vec::new();

    // Rust notes: It would be ideal if we could unify the cases for (Primtive, Constructor,
    // MathPrimitive), as well as (Expandable, Conditional) since the
    // API is identical. However, as the types are different, Rust
    // constrains us here, we need separate match arms for each
    // distinctly typed enum case.
    let digestable_def = lookup_digestable_definition(&token);
    match digestable_def {
      None | Some(Stored::None) => {
        result = invoke_token_undefined(&token)?;
      },
      Some(Stored::Token(meaning)) => {
        // Common case
        let cc = meaning.get_catcode();
        if cc == Catcode::CS {
          result = invoke_token_undefined(&token)?;
        } else if cc.is_absorbable() {
          if let Some(digested) = invoke_token_simple(meaning)? {
            result.push(digested);
          }
        } else {
          let message = s!(
            "The token {:?} (catcode {:?}) should never reach Stomach!",
            token,
            cc
          );
          Error!("misdefined", token, &message);
          if let Some(digested) = invoke_token_simple(meaning)? {
            result.push(digested);
          }
        }
      },
      Some(Stored::Expandable(meaning)) => {
        // A math-active character will (typically) be a macro,
        // but it isn't expanded in the gullet, but later when digesting, in math mode
        // (? I think)
        let invoked_meaning = meaning.invoke( false)?;
        if !invoked_meaning.is_empty() {
          { gullet::unread(invoked_meaning); }
        }
        // replace the token by it's expansion!!!
        maybe_token = gullet::read_x_token(None, false)?
          .map(Cow::Owned);
        { stomach_mut!().token_stack.pop(); }
        expire_current_token();
        continue;
      },
      Some(Stored::Conditional(meaning)) => {
        // Conditionals are "expandable", use the regular invoke.
        let invoked_meaning = meaning.invoke(false)?;
        gullet::unread(invoked_meaning);
        maybe_token = gullet::read_x_token(None, false)?
            .map(Cow::Owned);
        { stomach_mut!().token_stack.pop(); }
        expire_current_token();
        continue;
      },
      Some(Stored::Constructor(meaning)) => {
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::Primitive(meaning)) => {
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::MathPrimitive(meaning)) => {
        // Copy of regular Primitive
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::Register(meaning)) => {
        // Registers are special primitives
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      meaning => {
        fatal!(
          Stomach,
          Misdefined,
          s!("The object {:?} should never reach Stomach!", meaning)
        );
      },
    }
    expire_current_token();
    break;
  }
  stomach_mut!().token_stack.pop();
  Ok(result)
}

fn invoke_token_undefined(
  token: &Token,
) -> Result<Vec<Digested>> {
  let cs = token.with_cs_name(|cs| String::from(cs));
  note_status(LogStatus::Undefined, Some( &cs));

  // To minimize chatter, go ahead and define it...
  if cs.starts_with("\\if") {
    // Apparently an \ifsomething ???
    let name = cs.replace("\\if", "");
    let message = s!("The token {} is not defined.", token.stringify());
    Error!(
      "undefined",
      token,
      &message,
      "Defining it now as with \\newif"
    );
    // install stub definitions for new conditional
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}true", name)),
        None,
        Tokens!(T_CS!("\\let"), T_CS!(&cs), T_CS!("\\iftrue")).into(),
        None,
            )?,
      None,
    );
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}false", name)),
        None,
        Tokens!(T_CS!("\\let"), T_CS!(cs), T_CS!("\\iffalse")).into(),
        None,
            )?,
      None,
    );

    let_i(token, &T_CS!("\\iffalse"), None);
    gullet::unread_one(*token); // Retry
    Ok(Vec::new())
  } else {
    let message = s!("The token {} is not defined.", token.stringify());
    Error!(
      "undefined",
      token,
      &message,
      "Defining it now as <ltx:ERROR/>"
    );
    install_definition(
      Constructor {
        cs: *token,
        paramlist: None,
        replacement: Some(Rc::new(move |document, _args, _props| {
          document.make_error("undefined", &cs)
        })),
        ..Constructor::default()
      },
      Some(Scope::Global),
    );
    // and then invoke it.
    invoke_token(token)
  }
}

fn invoke_token_simple(meaning: Token) -> Result<Option<Digested>> {
  let cc = meaning.get_catcode();
  let font = lookup_font();
  clear_prefixes(); // prefixes shouldn't apply here.
  match cc {
    Catcode::SPACE => {
      if lookup_bool("IN_MATH") {
        Ok(None)
      } else {
        Ok(Some(Digested::from(Tbox::new(
          meaning.get_sym(),
          font,
          None,
          Tokens!(meaning),
          HashMap::default(),
                ))))
      }
    },
    Catcode::COMMENT => {
      let comment = meaning.to_string();
      // TODO:
      // let comment = font_decode_string(meaning.to_string(), None, true);
      // However, spaces normally would have be digested away as positioning...
      // let badspace = pack('U', 0xA0) . "\x{0335}"; // This is at space's pos in OT1
      // $comment =~ s/\Q$badspace\E/ /g;
      Ok(Some(Digested::from(comment)))
    },
    _ => {
      let text = font::decode_string(meaning.get_sym(), None, true);
      Ok(Some(Digested::from(Tbox::new(
        text,
        None,
        None,
        Tokens!(meaning),   // tokens
        HashMap::default(), // properties
            ))))
    },
  }
}

pub fn initialize_stomach() {
  stomach_mut!().initialize()
}
pub fn set_stomach(new_stomach: Stomach) {
  let mut singleton = stomach_mut!();
  *singleton = new_stomach;
}
pub fn clone_box_list() -> Vec<Digested> {
  stomach!().box_list.clone()
}

/// get the current boxing level
pub fn get_boxing_level() -> usize { stomach!().boxing.len() }
/// ScriptLevel is similar to boxing level, but relative to current Math mode's level
/// This is used for the scriptpos attribute to recognize overlapping sccripts.
/// Making it relative to the math's level avoids unnecessary changes
pub fn get_script_level() -> usize {
  let boxlevel = get_boxing_level();
  with_value("script_base_level",|val_opt|
    if let Some(Stored::Int(prevlevel)) = val_opt {
      boxlevel - (*prevlevel as usize) + 1
    } else {
      boxlevel
    })
}
