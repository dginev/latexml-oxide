use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use once_cell::sync::Lazy;

use crate::common::arena;
use crate::common::error::*;
use crate::common::font;
use crate::common::font::Font;
use crate::common::store::Stored;
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::Definition;
use crate::list::List;
use crate::mouth::{Mouth, MouthOptions};
use crate::state::{self, Scope};
use crate::tbox::*;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{Digested, TexMode, gullet, gullet_mut, state_mut};

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

#[macro_export]
macro_rules! stomach {
  () => ((*$crate::stomach::STOMACH).borrow())
}
#[macro_export]
macro_rules! stomach_mut {
  () => ((*$crate::stomach::STOMACH).borrow_mut())
}

impl Stomach {
  pub fn initialize(&mut self) {
    self.boxing      = Vec::new();
    self.token_stack = Vec::new();
    self.box_list = Vec::new();
    self.localized_box_list = Vec::new();
    state_mut!().initialize_stomach();
  }
  /// get the current boxing level
  pub fn get_boxing_level(&self) -> usize { self.boxing.len() }
  /// ScriptLevel is similar to boxing level, but relative to current Math mode's level
  /// This is used for the scriptpos attribute to recognize overlapping sccripts.
  /// Making it relative to the math's level avoids unnecessary changes
  pub fn get_script_level(&self) -> usize {
    let boxlevel = self.boxing.len();
    if let Some(Stored::Int(prevlevel)) = state!().lookup_value("script_base_level") {
      boxlevel - (*prevlevel as usize) + 1
    } else {
      boxlevel
    }
  }
  /// steal the previously digested boxes from the current level.
  pub fn regurgitate(&mut self) -> Vec<Digested> { self.box_list.drain(..).collect() }

  //**********************************************************************
  // Maintaining state::
  //**********************************************************************
  // state::changes that the Stomach needs to moderate and know about (?)

  //======================================================================
  // Dealing with TeX's bindings & grouping.
  // Note that lookups happen more often than bgroup/egroup (which open/close frames).

  /// Adds a new stack frame for a TeX group.
  pub fn push_stack_frame(&mut self, nobox: bool) {
    let current_token = {state!().get_current_token().unwrap().clone()};
    state::push_frame();
    state::assign_value(
      "beforeAfterGroup",
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Local),
    ); // ALWAYS bind this!
    state::assign_value(
      "afterGroup",
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Local),
    ); // ALWAYS bind this!
    state::assign_value("afterAssignment", Stored::None, Some(Scope::Local)); // ALWAYS bind this!
    state::assign_value("groupNonBoxing", nobox, Some(Scope::Local)); // ALWAYS bind this!
    state::assign_value("groupInitiator", current_token.clone(), Some(Scope::Local));
    state::assign_value(
      "groupInitiatorLocator",
      gullet!().get_locator().unwrap().into_owned(),
      Some(Scope::Local),
    );
    if !nobox {
      // For begingroup/endgroup
      self.boxing.push(current_token)
    }
  }
  /// Removes the last/current stack frame, ending a TeX group
  pub fn pop_stack_frame(&mut self, nobox: bool) -> Result<()> {
    if let Some(Stored::VecDequeStored(beforeafter)) = state::remove_value("beforeAfterGroup") {
      if !beforeafter.is_empty() {
        let mut result = Vec::new();
        for beforeafter_frame in beforeafter.into_iter() {
          match beforeafter_frame {
            Stored::Tokens(frametoks) => result.push(frametoks.be_digested()?),
            Stored::Token(frametok) => result.push(frametok.be_digested()?),
            _ => {
              // TODO: Anything but Tokens in beforeAfterGroup?
              dbg!(beforeafter_frame);
              unimplemented!();
            },
          }
        }
        // TODO
        // if (my ($x) = grep { !$_->isaBox } @result) {
        // Error('misdefined', $x, $self, "Expected a Box|List|Whatsit, but got '" . Stringify($x) .
        // "'"); @result = (makeMisdefinedError(@result)); }
        self.box_list.extend(result);
      }
    }
    let after = state::remove_value("afterGroup");
    state_mut!().pop_frame()?;
    if !nobox {
      self.boxing.pop(); // For begingroup/endgroup
    }
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
  pub fn current_frame_message(&self) -> String {
    let target = if state!().is_value_bound("MODE", Some(0)) {
      // SET mode in CURRENT frame ?
      Cow::Owned(s!("mode-switch to {}", state::lookup_string("MODE")))
    } else if state!().lookup_bool("groupNonBoxing") {
      // Current frame is a non-boxing group?
      Cow::Borrowed("non-boxing group")
    } else {
      Cow::Borrowed("boxing group")
    };

    let initiator = if let Some(t) = state::lookup_token("groupInitiator") {
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
  pub fn bgroup(&mut self) {
    self.push_stack_frame(false);
    // NOTE: This is WRONG; should really only track "scanned" (not digested) braces
    // Alas, there're too many code structuring differences between TeX and LaTeXML
    // to find all the places to manage it.
    // So, let's try this for now...
    // was $LaTeXML::ALIGN_STATE
    state::increment_align_group_count();
  }
  /// End a level of binding by popping the last stack frame,
  /// undoing whatever bindings appeared there, and also
  /// decrementing the level of boxing.
  pub fn egroup(&mut self) -> Result<()> {
    if state::lookup_bool("groupNonBoxing") {
      // or group was opened with \begingroup
      Error!(
        "unexpected",
        {state!().get_current_token().unwrap()},
        "Attempt to close boxing group"
      );
    } else {
      // Don't pop if there's an error; maybe we'll recover?
      self.pop_stack_frame(false)?;
    }
    state::decrement_align_group_count();
    Ok(())
  }
  /// Begin a new level of binding by pushing a new stack frame.
  pub fn begingroup(&mut self) { self.push_stack_frame(true); }
  /// End a level of binding by popping the last stack frame,
  /// undoing whatever bindings appeared there.
  pub fn endgroup(&mut self) -> Result<()> {
    if !state::lookup_bool("groupNonBoxing") {
      // or group was opened with \bgroup
      Error!(
        "unexpected",
        {state!().get_current_token().unwrap().to_string()},
        s!(
          "Attempt to close non-boxing group; {}",
          self.current_frame_message()
        )
      );
    } else {
      self.pop_stack_frame(true)?;
    }
    Ok(())
  }

  //======================================================================
  // Mode (minimal so far; math vs text)
  // Could (should?) be taken up by Stomach by building horizontal, vertical or math lists ?

  /// This sets the mode without doing any grouping (NOR does it stack the modes!!)
  /// Useful for environments, where the group has already been established.
  /// (presumably, in the long run, modes & groups should be much less coupled)
  pub fn set_mode(&mut self, mode: &str) -> Result<()> {
    let prevmode = state::lookup_string("MODE");
    let ismath = mode.ends_with("math");
    state::assign_value("MODE", arena::pin(mode), Some(Scope::Local));
    state::assign_value("IN_MATH", ismath, Some(Scope::Local));
    if mode == prevmode {
    } else if ismath {
      let curfont = state!().lookup_font().unwrap();
      // When entering math mode, we set the font to the default math font,
      // and save the text font for any embedded text.
      state::assign_value("savedfont", curfont.clone(), Some(Scope::Local));
      // see get_script_level()
      state::assign_value("script_base_level", self.boxing.len(), None);
      let isdisplay = mode.starts_with("display");
      let new_font = state!().lookup_mathfont().unwrap().merge(Font {
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
      state::assign_font(Rc::new(new_font), Some(Scope::Local));
    } else {
      let curfont = state!().lookup_font().unwrap();
      // When entering text mode, we should set the font to the text font in use before the math
      // but inherit color and size
      let saved_opt = state!().lookup_value("savedfont").cloned();
      if let Some(Stored::Font(saved_font)) = saved_opt {
        state::assign_font(
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
  pub fn begin_mode(&mut self, mode: &str) -> Result<()> {
    self.push_stack_frame(false); // Effectively bgroup
    self.set_mode(mode)?;
    Ok(())
  }
  /// End processing in `mode`; an error is signalled if `stomach` is not
  /// currently in `mode`.  This also ends a level of grouping.
  pub fn end_mode(&mut self, mode: &str) -> Result<()> {
    // Last stack frame was NOT a mode switch!?!?!
    if !state!().is_value_bound("MODE", Some(0)) || (state::lookup_string_sym("MODE") != arena::pin(mode)) {
      // Or was a mode switch to a different mode
      let message = s!(
        "Attempt to end mode `{}` in `{}`",
        mode,
        state::lookup_string("MODE")
      );
      let category = match state!().get_current_token() {
        Some(ref token) => token.to_string(),
        None => String::from("mode"),
      };
      Error!("unexpected", category, &message); // self.currentFrameMessage);
    } else {
      // Don"t pop if there"s an error; maybe we'll recover?
      self.pop_stack_frame(false)?;
    } // Effectively egroup.
    Ok(())
  }

  pub fn new_local_box_list(&mut self) {
    let mut buffer = Vec::new();
    std::mem::swap(&mut self.box_list, &mut buffer);
    self.localized_box_list.push(buffer);
  }
  pub fn expire_local_box_list(&mut self) -> Vec<Digested> {
    let mut buffer = self.localized_box_list.pop().unwrap_or_default();
    std::mem::swap(&mut self.box_list, &mut buffer);
    buffer
  }

  //**********************************************************************
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
    state::clear_prefixes(); // prefixes shouldn't apply here.
    let mode = if state::lookup_bool("IN_MATH") {
      TexMode::Math
    } else {
      TexMode::Text
    };
    let initdepth = stomach!().boxing.len();
    let depth = initdepth;
    stomach_mut!().new_local_box_list();
    while let Some(token) = gullet::read_x_token(Some(true), true)?
    {
      // Done if we run out of tokens
      let invoked = invoke_token(&token)?;
      stomach_mut!().box_list.extend(invoked);

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

    let mut digested_list = List::new(stomach_mut!().expire_local_box_list());
    digested_list.mode = Some(mode);
    digested_list.into()
  })
}

/// Return the digested `List` after reading and digesting a body from the its Gullet.
/// The body extends until the current level of boxing or environment is closed.
pub fn digest_next_body(
  terminal_opt: Option<Token>,
) -> Result<Vec<Digested>> {

  let start_location = { gullet!().get_locator().unwrap().into_owned() };

  let init_depth = { stomach!().boxing.len() };
  let mut found_token = false;
  let mut found_terminal = false;
  stomach_mut!().new_local_box_list();
  let alignment_opt = state::lookup_alignment();
  // TODO: bookkeep for "expected" warning
  //let mut aug = Vec::new();

  // try reading a executable token
  while let Some(token) = gullet::read_x_token(Some(true), true)?
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
      return Ok(stomach_mut!().expire_local_box_list());
    }
    // normal case
    let invoked = invoke_token(&token)?;
    stomach_mut!().box_list.extend(invoked);

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
    stomach_mut!().box_list.push(Digested::from(Tbox::default()));
    // info!(target:"digest_next_body","no_token");
  }
  Ok(stomach_mut!().expire_local_box_list())
}


/// a convenience function for including chunks of raw TeX (or LaTeX) code
/// It is useful for copying portions of the normal
/// implementation that can be handled simply using macros and primitives.
pub fn raw_tex(text: &str) -> Result<()> {
  // It could be as simple as this, except if catcodes get changed, it's too late!!!
  //  Digest(TokenizeInternal($text));
  let savedcc = state::lookup_catcode('@').unwrap_or(Catcode::OTHER);
  state::assign_catcode('@', Catcode::LETTER, None);
  let raw_tex_mouth = Mouth::new(
    text,
    Some(MouthOptions {
      fordefinitions: true,
      ..MouthOptions::default()
    }),
  )?;
  gullet::reading_from_mouth(raw_tex_mouth, move || -> Result<()> {
    while let Some(token) = gullet::read_x_token(Some(false), false)? {
      if token != T_SPACE!() {
        invoke_token(&token)?;
      }
    }
    Ok(())
  })?;

  state::assign_catcode('@', savedcc, None);
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
      state::local_current_token(token.clone());
      { stomach_mut!().token_stack.push(token.clone()); }
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
      let digestable_def = {
        state_mut!().lookup_digestable_definition(&token)
      };
      match digestable_def {
        None => {
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
          stomach_mut!().token_stack.pop();
          state::expire_current_token();
          continue;
        },
        Some(Stored::Conditional(meaning)) => {
          // Conditionals are "expandable", use the regular invoke.
          let invoked_meaning = meaning.invoke(false)?;
          gullet::unread(invoked_meaning);
          maybe_token = gullet::read_x_token(None, false)?
              .map(Cow::Owned);
          stomach_mut!().token_stack.pop();
          { state_mut!().expire_current_token(); }
          continue;
        },
        Some(Stored::Constructor(meaning)) => {
          // Otherwise, a normal primitive or constructor
          result = meaning.invoke_primitive()?;
          if !meaning.is_prefix() {
            state_mut!().clear_prefixes(); // Clear prefixes unless we just set one.
          }
        },
        Some(Stored::Primitive(meaning)) => {
          // Otherwise, a normal primitive or constructor
          result = meaning.invoke_primitive()?;
          if !meaning.is_prefix() {
            state_mut!().clear_prefixes(); // Clear prefixes unless we just set one.
          }
        },
        Some(Stored::MathPrimitive(meaning)) => {
          // Copy of regular Primitive
          // Otherwise, a normal primitive or constructor
          result = meaning.invoke_primitive()?;
          if !meaning.is_prefix() {
            state_mut!().clear_prefixes(); // Clear prefixes unless we just set one.
          }
        },
        Some(Stored::Register(meaning)) => {
          // Registers are special primitives
          result = meaning.invoke_primitive()?;
          if !meaning.is_prefix() {
            state_mut!().clear_prefixes(); // Clear prefixes unless we just set one.
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
      state::expire_current_token();
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
      state::install_definition(
        Expandable::new(
          T_CS!(s!("\\{}true", name)),
          None,
          Tokens!(T_CS!("\\let"), T_CS!(&cs), T_CS!("\\iftrue")),
          None,
              )?,
        None,
      );
      state::install_definition(
        Expandable::new(
          T_CS!(s!("\\{}false", name)),
          None,
          Tokens!(T_CS!("\\let"), T_CS!(cs), T_CS!("\\iffalse")),
          None,
              )?,
        None,
      );

      let mut gullet = gullet_mut!();
      state::let_i(token, &T_CS!("\\iffalse"), None);
      gullet.unread_one(token.clone()); // Retry
      Ok(Vec::new())
    } else {
      let message = s!("The token {} is not defined.", token.stringify());
      Error!(
        "undefined",
        token,
        &message,
        "Defining it now as <ltx:ERROR/>"
      );
      state::install_definition(
        Constructor {
          cs: token.clone(),
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
    let font = state!().lookup_font();
    state_mut!().clear_prefixes(); // prefixes shouldn't apply here.
    match cc {
      Catcode::SPACE => {
        if state!().lookup_bool("IN_MATH") {
          Ok(None)
        } else {
          Ok(Some(Digested::from(Tbox::new(
            meaning.get_sym(),
            font,
            gullet!().get_locator().map(|l| l.into_owned()),
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
          None,               // locator
          Tokens!(meaning),   // tokens
          HashMap::default(), // properties
              ))))
      },
    }
  }

  pub fn bgroup() { stomach_mut!().bgroup() }
  pub fn egroup() -> Result<()> { stomach_mut!().egroup() }
  pub fn begingroup() { stomach_mut!().begingroup() }
  pub fn endgroup() -> Result<()> { stomach_mut!().endgroup() }
  pub fn set_mode(mode: &str) -> Result<()> { stomach_mut!().set_mode(mode) }
  pub fn begin_mode(mode: &str) -> Result<()> { stomach_mut!().begin_mode(mode) }
  pub fn end_mode(mode: &str) -> Result<()> { stomach_mut!().end_mode(mode) }