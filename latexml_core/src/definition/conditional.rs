//! Conditionals Control sequence definitions.
//! These represent the control sequences for conditionals, as well as
//! `\else`, `\or` and `\fi`.

use std::{borrow::Cow, cell::RefCell, fmt, rc::Rc};

use libxml::tree::Node;

// use crate::common::numeric_ops::NumericOps;
use crate::Digested;
use crate::{
  common::{error::*, locator::Locator, object::Object},
  definition::{BeforeDigestClosure, ConditionalClosure, Definition, DigestionClosure},
  document::Document,
  gullet,
  parameter::Parameters,
  state::*,
  token::*,
  tokens::Tokens,
  whatsit::Whatsit,
};

// Conditional control sequences; Expandable
//   Expand enough to determine true/false, then maybe skip
//   record a flag somewhere so that \else or \fi is recognized
//   (otherwise, they should signal an error)

/// classify the standard pieces of a conditional
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalType {
  /// \if
  If,
  /// \unless
  Unless,
  /// \else
  Else,
  /// \or
  Or,
  /// \fi
  Fi,
  /// fallback?
  Unknown,
}

impl From<&str> for ConditionalType {
  fn from(cs: &str) -> Self {
    use self::ConditionalType::*;
    match cs {
      "\\if" => If,
      "\\unless" => Unless,
      "\\else" => Else,
      "\\or" => Or,
      "\\fi" => Fi,
      _ => If,
    }
  }
}

/// configurations for a conditional.
#[derive(Default)]
pub struct ConditionalOptions {
  /// scope to install in state
  pub scope:   Option<Scope>,
  /// is this definition locked?
  pub locked:  Option<bool>,
  /// skipper, currently only used for \ifcase.
  // TODO: implement this?
  pub skipper: Option<bool>,
}

/// A Conditional definition; Expandable.
#[derive(Clone)]
pub struct Conditional {
  /// the command sequence
  pub cs:               Token,
  /// list of parameters, if any
  pub paramlist:        Option<Parameters>,
  /// a test closure, if implemented in a binding
  pub test:             Option<ConditionalClosure>,
  /// the kind of piece in the syntax (if,else,fi...)
  pub conditional_type: ConditionalType,
  /// a skipper for \ifcase
  pub skipper:          Option<bool>,
}
impl Default for Conditional {
  fn default() -> Self {
    Conditional {
      cs:               T_CS!("Conditional"),
      paramlist:        None,
      test:             None,
      conditional_type: ConditionalType::Unknown,
      skipper:          None,
    }
  }
}
impl PartialEq for Conditional {
  fn eq(&self, other: &Conditional) -> bool { self.cs == other.cs }
}

impl fmt::Display for Conditional {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.cs) }
}
impl Object for Conditional {
  fn is_expandable(&self) -> bool { true }
  fn stringify(&self) -> String { self.stringify_type("Conditional") }
}
impl Definition for Conditional {
  // sub new {
  //   my ($class, $cs, $parameters, $test, %traits) = @_;
  //   my $source = $state->getStomach->getGullet->getMouth;
  //   return bless { cs => $cs, parameters => $parameters, test => $test,
  //     locator      => "from " . $source->getLocator(-1),
  //     isExpandable => 1,
  //     %traits }, $class; }

  // Note that although conditionals are Expandable,
  // they are NOT defined as macros, so they don't need to handle doInvocation,
  fn invoke(&self, _once_only: bool) -> Result<Tokens> {
    // A real conditional must have condition_type set
    use self::ConditionalType::*;
    match self.conditional_type {
      If | Unless => self.invoke_conditional(),
      Else | Or => self.invoke_else(),
      Fi => self.invoke_fi(),
      _ => {
        // Diagnostic-only path: format the current CS name, or "\\?" if the
        // current-token register is empty. Never panic here — we are
        // already in error-emission territory.
        let cur = get_current_token()
          .map(|t| t.stringify())
          .unwrap_or_else(|| String::from("\\?"));
        let message = s!("Unknown conditional control sequence {}", cur);
        Error!("unexpected", self.cs, message);
        Ok(Tokens!())
      },
    }
  }

  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }
  fn get_cs(&self) -> Cow<'_, Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<'_, str> { Cow::Owned(self.cs.with_cs_name(ToString::to_string)) }
  fn get_alias(&self) -> Option<&String> { None }
  fn get_test(&self) -> Option<&ConditionalClosure> { self.test.as_ref() }
  fn get_conditional_type(&self) -> Option<ConditionalType> { Some(self.conditional_type) }
  // Not implemented for expandable
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    // Conditionals are expandable, not primitive — this shouldn't be called
    Ok(Vec::new())
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorption(&self, _document: &mut Document, _whatsit: &Whatsit) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorption on Conditional should never be called!"
    );
  }
}

/// A Frame of data for the currently active conditional, stored in State
#[derive(Debug, Clone, PartialEq)]
pub struct IfFrame {
  /// the token which started the conditional
  pub token:   Token,
  /// source location of the conditional start
  pub start:   Locator,
  /// flag: currently parsing the test
  pub parsing: bool,
  /// flag: already seen an else at this level
  pub elses:   bool,
  /// in nested conditionals, give each an id
  pub ifid:    i64,
}

impl Conditional {
  fn invoke_conditional(&self) -> Result<Tokens> {
    let mut ifid = lookup_int("if_count");
    ifid += 1;
    assign_value("if_count", ifid, Some(Scope::Global));
    // Perl: if ($LaTeXML::IF_LIMIT and $ifid > $LaTeXML::IF_LIMIT) { Fatal(...) }
    let if_limit = lookup_int("if_limit");
    if if_limit > 0 && ifid > if_limit {
      Fatal!(
        Timeout,
        IfLimit,
        s!("Conditional limit of {} exceeded, infinite loop?", if_limit)
      );
    }
    let if_frame = Rc::new(RefCell::new(IfFrame {
      token: get_current_token().unwrap(),
      start: gullet::get_locator(),
      parsing: true,
      elses: false,
      ifid,
    }));
    set_ifframe(Some(Rc::clone(&if_frame)));
    unshift_value("if_stack", vec![Rc::clone(&if_frame)]);
    let args = self.read_arguments()?;

    get_ifframe().unwrap().borrow_mut().parsing = false;
    // `tracingcommands` is normally unset; defer the state probe to
    // the path that would actually read it. Conditional::invoke fires
    // on every \if/\ifx/\ifnum/…, so avoiding a mandatory state
    // lookup per conditional is a measurable win.
    if let Some(ref test) = self.test {
      if (test)(args)? {
        // true branch: do nothing, tokens follow naturally
      } else {
        let to = self.skip_conditional_body(-1);
        if lookup_bool("tracingcommands") {
          Debug!("{{false}} [skipped to {:?}]\n", to);
        }
      }
    } else {
      // If there's no test, it must be the Special Case, \ifcase
      // Note: num == 0 takes the 1st branch, no need to skip
      // num < 0 should skip all \or & end up on the \else
      let num = args.first().map(|a| a.value_of()).unwrap_or(0);
      if num != 0 {
        let _to = self.skip_conditional_body(num);
        //       print STDERR "{$num} [skipped to " . ToString($to) . "]\n" if $tracing;
      }
    }
    expire_ifframe();
    Ok(Tokens!())
  }

  // =====================================================================
  // Support for conditionals:
  //
  // Skipping for conditionals
  //   0 : skip to \fi
  //  -1 : skip to \else, if any, or \fi
  //   n : skip to n-th \or, if any, or \else, if any, or \fi.
  //
  // NOTE that there are 2 kinds of "nested" ifs.
  //  \if's inside the body of either the true or false branch
  // are easily skipped by tracking a level of if nesting and skipping over the
  // same number of \fi as you find \if.
  //  \if's that get expanded while evaluating the test clause itself
  // are considerably trickier. There's a frame on the if-stack for this \if
  // that's above the one we're currently processing; typically the \else & \fi
  // may still remain, but we need to either evaluate them a normal
  // if we're continuing to follow the true branch, or skip oever them if
  // we're trying to find the \else for the false branch.
  // The danger is mistaking the \else that's associated with the test clause's \if
  // and taking it for the \else that we're skipping to!
  // Canonical example:
  //   \if\ifx AA XY junk \else blah \fi True \else False \fi
  // The inner \ifx should expand to "XY junk", since A==A
  // Return the token we've skipped to, and the frame that this applies to.
  fn skip_conditional_body(&self, nskips: i64) -> Result<Tokens> {
    let mut level = 1;
    let mut n_ors = 0;
    let _start = gullet::get_locator();
    // NOTE: Open-coded manipulation of if_stack!
    // [we're only reading tokens & looking up, so state::shouldn't change behind our backs]
    loop {
      let (t, cond_type) = match gullet::read_next_conditional()? {
        Some((tok, typ)) => (Tokens!(tok), Some(typ)),
        None => (Tokens!(), None),
      };
      match cond_type {
        None => break,
        Some(ConditionalType::If) => level += 1, //  Found a \ifxx of some sort
        Some(ConditionalType::Fi) => {
          // Found a \fi
          let local_frame = get_ifframe();
          let maybe_last = with_value_mut("if_stack", |value_opt| {
            if let Some(Stored::VecDequeStored(stack)) = value_opt
              && let Some(Stored::IfFrame(stack_frame)) = stack.pop_front()
            {
              if *stack_frame.borrow() != *local_frame.as_ref().unwrap().borrow() {
                // But is it for a condition nested in the test clause?
                // then DO pop that conditional's frame; it's DONE!
              } else {
                level -= 1;
                if level == 0 {
                  // otherwise, if no more nesting, we're done.
                  // Done with this frame, keep it removed
                  return Some(t); // AND Return the finishing token.
                } else {
                  stack.push_front(stack_frame.into());
                }
              }
            }
            None
          });
          if let Some(t) = maybe_last {
            return Ok(t);
          }
        },
        Some(other_type) => {
          if level > 1 {
            // Ignore: \else,\or nested in the body.
          } else if other_type == ConditionalType::Or {
            n_ors += 1;
            if n_ors == nskips {
              return Ok(t);
            }
          } else if other_type == ConditionalType::Else && nskips != 0 {
            // Found \else and we're looking for one?
            let local_frame = get_ifframe();
            // Make sure this \else is NOT for a nested \if that is part of the test clause!
            let maybe_last = with_value("if_stack", |stack_opt| {
              if let Some(Stored::VecDequeStored(stack)) = stack_opt
                && let Some(Stored::IfFrame(stack_frame)) = stack.front()
                && *stack_frame.borrow() == *local_frame.as_ref().unwrap().borrow()
              {
                // No need to actually call elseHandler, but note that we've seen an \else!
                stack_frame.borrow_mut().elses = true;
                return Some(t);
              }
              None
            });
            if let Some(t) = maybe_last {
              return Ok(t);
            }
          }
        },
      };
    }
    Error!(
      "expected",
      "\\fi",
      self,
      s!(
        "Missing \\fi or \\else, conditional fell off end. Conditional started at {:?}",
        _start
      )
    );
    Ok(Tokens!())
  }

  fn invoke_else(&self) -> Result<Tokens> {
    let stack_frame_opt = with_value_mut("if_stack", |stack_opt| {
      if let Some(Stored::VecDequeStored(stack)) = stack_opt {
        if let Some(Stored::IfFrame(stack_frame)) = stack.front() {
          Some(Rc::clone(stack_frame))
        } else {
          None
        }
      } else {
        None
      }
    });
    let local_token = get_current_token().unwrap();
    if local_token.with_str(|s| s == "\\else") && stack_frame_opt.is_none() {
      let stack_len = with_value("if_stack", |v| match v {
        Some(Stored::VecDequeStored(s)) => s.len(),
        _ => 0,
      });
      log::warn!("\\else encountered with no active if-frame (stack_len={stack_len})");
    }
    if let Some(stack_frame) = stack_frame_opt {
      if stack_frame.borrow().parsing {
        // Defer expanding the \else if we're still parsing the test
        Ok(Tokens!(T_RELAX!(), local_token))
      } else if stack_frame.borrow().elses {
        // Already seen an \else's at this level?
        let message = s!(
          "Extra {} already saw \\else for {:?} [{:?}] at {:?}",
          local_token.stringify(),
          stack_frame.borrow().token,
          stack_frame.borrow().ifid,
          stack_frame.borrow().start
        );
        let local_token_str = local_token.to_string();
        Error!("unexpected", local_token_str, message);
        Ok(Tokens!())
      } else {
        set_ifframe(Some(Rc::clone(&stack_frame)));
        let _t = self.skip_conditional_body(0);
        //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
        //       . " [for " . ToString($$LaTeXML::IFFRAME{token}) . " #" .
        // $$LaTeXML::IFFRAME{ifid}       . " skipping to " . ToString($t) . "]\n"
        //       if $state->lookupValue('tracingcommands');
        expire_ifframe();
        Ok(Tokens!())
      }
    } else {
      // No if stack entry ?
      let message = s!(
        "Didn't expect a {:?} since we seem not to be in a conditional",
        local_token.stringify()
      );
      let local_token_str = local_token.to_string();
      Error!("unexpected", local_token_str, message);
      Ok(Tokens!())
    }
  }

  fn invoke_fi(&self) -> Result<Tokens> {
    let stack_frame_opt: Option<Rc<RefCell<IfFrame>>> = with_value("if_stack", |stack_opt| {
      if let Some(Stored::VecDequeStored(stack)) = stack_opt {
        if let Some(Stored::IfFrame(frame)) = stack.front() {
          Some(Rc::clone(frame))
        } else {
          None
        }
      } else {
        None
      }
    });
    if let Some(stack_frame) = stack_frame_opt {
      if stack_frame.borrow().parsing {
        // Defer expanding the \else if we're still parsing the test
        Ok(Tokens!(T_RELAX!(), get_current_token().unwrap()))
      } else {
        // "expand" by removing the stack entry for this level
        set_ifframe(Some(stack_frame));
        shift_value("if_stack")?; // Done with this frame

        //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
        // . " [for " . Stringify($$LaTeXML::IFFRAME{token}) . " #" . $$LaTeXML::IFFRAME{ifid} .
        // "]\n"       if $state->lookupValue('tracingcommands');
        expire_ifframe();
        Ok(Tokens!())
      }
    } else {
      let cur = get_current_token()
        .map(|t| t.stringify())
        .unwrap_or_else(|| String::from("\\?"));
      let message = s!(
        "Didn't expect a {:?} since we seem not to be in a conditional",
        cur
      );
      Error!("unexpected", "fi", message);
      Ok(Tokens!())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn conditional_type_from_str_known_variants() {
    assert_eq!(ConditionalType::from("\\if"), ConditionalType::If);
    assert_eq!(ConditionalType::from("\\unless"), ConditionalType::Unless);
    assert_eq!(ConditionalType::from("\\else"), ConditionalType::Else);
    assert_eq!(ConditionalType::from("\\or"), ConditionalType::Or);
    assert_eq!(ConditionalType::from("\\fi"), ConditionalType::Fi);
  }

  #[test]
  fn conditional_type_from_str_unknown_falls_back_to_if() {
    // The match's default arm is `_ => If`, not Unknown — that's a
    // surprising but documented behavior in the source. Lock it in.
    assert_eq!(ConditionalType::from("\\foo"), ConditionalType::If);
    assert_eq!(ConditionalType::from(""), ConditionalType::If);
  }

  #[test]
  fn conditional_type_equality() {
    assert_eq!(ConditionalType::If, ConditionalType::If);
    assert_ne!(ConditionalType::If, ConditionalType::Else);
  }

  #[test]
  fn conditional_default_fields() {
    let c = Conditional::default();
    assert!(c.paramlist.is_none());
    assert!(c.test.is_none());
    assert_eq!(c.conditional_type, ConditionalType::Unknown);
    assert!(c.skipper.is_none());
  }

  #[test]
  fn conditional_partial_eq_by_cs() {
    // PartialEq ignores conditional_type, skipper etc., compares by cs.
    let a = Conditional::default();
    let b = Conditional::default();
    assert!(a == b, "defaults have same cs");
  }

  #[test]
  fn conditional_is_expandable() {
    let c = Conditional::default();
    assert!(c.is_expandable());
    // Conditionals are NOT definitions (they're expandable machinery).
    // Actually the default Object::is_definition returns false; the
    // trait default applies here.
  }

  #[test]
  fn conditional_display_is_cs_text() {
    let c = Conditional::default();
    let s = format!("{c}");
    assert_eq!(s, "Conditional");
  }

  #[test]
  fn conditional_get_parameters_none_by_default() {
    let c = Conditional::default();
    assert!(c.get_parameters().is_none());
  }

  #[test]
  fn conditional_options_default_all_none() {
    let o = ConditionalOptions::default();
    assert!(o.scope.is_none());
    assert!(o.locked.is_none());
    assert!(o.skipper.is_none());
  }
}
