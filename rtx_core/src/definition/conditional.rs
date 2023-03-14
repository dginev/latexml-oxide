use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, RwLock};
use libxml::tree::Node;

use crate::common::error::*;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
// use crate::common::numeric_ops::NumericOps;
use crate::definition::{BeforeDigestClosure, ConditionalClosure, Definition, DigestionClosure};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::state::{Scope, State};
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

// Conditional control sequences; Expandable
//   Expand enough to determine true/false, then maybe skip
//   record a flag somewhere so that \else or \fi is recognized
//   (otherwise, they should signal an error)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalType {
  If,
  Unless,
  Else,
  Or,
  Fi,
  Unknown,
}

impl ConditionalType {
  pub fn from(cs: &str) -> Self {
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

// This is ONLY used for \ifcase.
#[derive(Default)]
pub struct ConditionalOptions {
  pub scope: Option<Scope>,
  pub locked: Option<bool>,
  pub skipper: Option<bool>,
}

#[derive(Clone)]
pub struct Conditional {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub test: Option<ConditionalClosure>,
  pub conditional_type: ConditionalType,
  pub locked: Option<bool>,
  pub skipper: Option<bool>,
}
impl Default for Conditional {
  fn default() -> Self {
    Conditional {
      cs: T_CS!("Conditional"),
      paramlist: None,
      test: None,
      conditional_type: ConditionalType::Unknown,
      locked: None,
      skipper: None,
    }
  }
}
impl PartialEq for Conditional {
  fn eq(&self, other: &Conditional) -> bool { self.cs == other.cs }
}

impl fmt::Display for Conditional {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}",self.cs)
  }
}
impl Object for Conditional {
  fn is_expandable(&self) -> bool { true }
  fn stringify(&self) -> String { self.stringify_type("Conditional") }
  fn get_locator(&self) -> Option<Cow<Locator>> { None } // TODO
}
impl Definition for Conditional {
  // sub new {
  //   my ($class, $cs, $parameters, $test, %traits) = @_;
  //   my $source = $STATE->getStomach->getGullet->getMouth;
  //   return bless { cs => $cs, parameters => $parameters, test => $test,
  //     locator      => "from " . $source->getLocator(-1),
  //     isExpandable => 1,
  //     %traits }, $class; }

  // Note that although conditionals are Expandable,
  // they are NOT defined as macros, so they don't need to handle doInvocation,
  fn invoke(&self, gullet: &mut Gullet, _once_only: bool, state: &mut State) -> Result<Tokens> {
    // A real conditional must have condition_type set
    use self::ConditionalType::*;
    match self.conditional_type {
      If | Unless => self.invoke_conditional(gullet, state),
      Else => self.invoke_else(gullet, state),
      Or => self.invoke_else(gullet, state),
      Fi => self.invoke_fi(gullet, state),
      _ => {
        let message = s!(
          "Unknown conditional control sequence {}",
          state.get_current_token().unwrap().stringify()
        );
        Error!("unexpected", self.cs, gullet, state, message);
        Ok(Tokens!())
      },
    }
  }

  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }
  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Borrowed(self.cs.get_cs_name()) }
  fn get_alias(&self) -> Option<&String> { None }
  // Not implemented for expandable
  fn invoke_primitive(&self, _gullet: &mut Stomach, _caller: Arc<dyn Definition>, _state: &mut State) -> Result<Vec<Digested>> { unimplemented!() }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<Vec<Node>> {
    fatal!(Definition, Unexpected, "do_absorbtion on Conditional should never be called!");
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfFrame {
  pub token: Arc<Token>,
  pub start: Locator,
  pub parsing: bool,
  pub elses: bool,
  pub ifid: i64,
}

impl Conditional {
  pub fn invoke_conditional(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    // TODO!!! Implement in full
    // Keep a stack of the conditionals we are processing.
    let mut ifid = state.lookup_int("if_count");
    ifid += 1;
    state.assign_value("if_count", ifid, Some(Scope::Global));
    let if_frame = Arc::new(RwLock::new(IfFrame {
      token: Arc::clone(&state.get_current_token().unwrap()),
      start: gullet.get_locator().unwrap().into_owned(),
      parsing: true,
      elses: false,
      ifid,
    }));
    state.set_ifframe(Some(Arc::clone(&if_frame)));
    state.unshift_value("if_stack", vec![Arc::clone(&if_frame)]);

    let args = self.read_arguments(gullet, state)?;

    if_frame.write().unwrap().parsing = false;
    let tracing = state.lookup_bool("TRACINGCOMMANDS");
    //   print STDERR '{' . $self->tracingCSName . "} [#$ifid]\n" if $tracing;
    //   print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    if let Some(ref test) = self.test {
      if (test)(gullet, args, state)? {
        if tracing {
          Debug!("{{true}}\n");
        }
      } else {
        let to = self.skip_conditional_body(-1, gullet, state);
        if tracing {
          Debug!("{{false}} [skipped to {:?}]\n", to);
        }
      }
    } else {
      // If there's no test, it must be the Special Case, \ifcase
      let num = args[0].value_of();
      if num > 0 {
        let _to = self.skip_conditional_body(num, gullet, state);
        //       print STDERR "{$num} [skipped to " . ToString($to) . "]\n" if $tracing;
      }
    }
    state.expire_ifframe();
    Ok(Tokens!())
  }

  // #======================================================================
  // # Support for conditionals:

  // # Skipping for conditionals
  // #   0 : skip to \fi
  // #  -1 : skip to \else, if any, or \fi
  // #   n : skip to n-th \or, if any, or \else, if any, or \fi.

  // # NOTE that there are 2 kinds of "nested" ifs.
  // #  \if's inside the body of either the true or false branch
  // # are easily skipped by tracking a level of if nesting and skipping over the
  // # same number of \fi as you find \if.
  // #  \if's that get expanded while evaluating the test clause itself
  // # are considerably trickier. There's a frame on the if-stack for this \if
  // # that's above the one we're currently processing; typically the \else & \fi
  // # may still remain, but we need to either evaluate them a normal
  // # if we're continuing to follow the true branch, or skip oever them if
  // # we're trying to find the \else for the false branch.
  // # The danger is mistaking the \else that's associated with the test clause's \if
  // # and taking it for the \else that we're skipping to!
  // # Canonical example:
  // #   \if\ifx AA XY junk \else blah \fi True \else False \fi
  // # The inner \ifx should expand to "XY junk", since A==A
  // # Return the token we've skipped to, and the frame that this applies to.
  fn skip_conditional_body(&self, nskips: i64, gullet: &mut Gullet, state: &mut State) -> Tokens {
    let mut level = 1;
    let mut n_ors = 0;
    let _start = gullet.get_locator();
    // NOTE: Open-coded manipulation of if_stack!
    // [we're only reading tokens & looking up, so State shouldn't change behind our backs]

    let local_frame = state.get_ifframe();
    loop {
      let (t, cond_type) = match gullet.read_next_conditional(state) {
        Some((tok, typ)) => (Tokens!(tok), Some(typ)),
        None => (Tokens!(), None),
      };
      match cond_type {
        None => break,
        Some(ConditionalType::If) => level += 1, //  Found a \ifxx of some sort
        Some(ConditionalType::Fi) => {
          // Found a \fi
          if let Some(Stored::VecDequeStored(stack)) = state.lookup_value_mut("if_stack") {
            if let Some(Stored::IfFrame(stack_frame)) = stack.pop_front() {
              if *stack_frame.read().unwrap() != *local_frame.as_ref().unwrap().read().unwrap() {
                // But is it for a condition nested in the test clause?
                // then DO pop that conditional's frame; it's DONE!
              } else {
                level -= 1;
                if level == 0 {
                  // otherwise, if no more nesting, we're done.
                  // Done with this frame, keep it removed
                  return t; // AND Return the finishing token.
                } else {
                  stack.push_front(stack_frame.into());
                }
              }
            }
          }
        },
        Some(other_type) => {
          if level > 1 {
            // Ignore: \else,\or nested in the body.
          } else if other_type == ConditionalType::Or {
            n_ors += 1;
            if n_ors == nskips {
              return t;
            }
          } else if other_type == ConditionalType::Else && nskips != 0 {
            // Found \else and we're looking for one?
            // Make sure this \else is NOT for a nested \if that is part of the test clause!
            if let Some(Stored::VecDequeStored(stack)) = state.lookup_value("if_stack") {
              if let Some(Stored::IfFrame(ref stack_frame)) = stack.front() {
                if *stack_frame.read().unwrap() == *local_frame.as_ref().unwrap().read().unwrap() {
                  // No need to actually call elseHandler, but note that we've seen an \else!
                  stack_frame.write().unwrap().elses = true;
                  return t;
                }
              }
            }
          }
        },
      };
    }
    Error!("expected", "\\fi", self, state, "Missing \\fi or \\else, conditional fell off end");
    Tokens!()
  }

  pub fn invoke_else(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    let stack_frame_opt = if let Some(Stored::VecDequeStored(stack)) = state.lookup_value_mut("if_stack") {
      if let Some(Stored::IfFrame(stack_frame)) = stack.front() {
        Some(Arc::clone(stack_frame))
      } else {
        None
      }
    } else {
      None
    };
    let local_token = state.get_current_token().unwrap();
    if let Some(stack_frame) = stack_frame_opt {
      if stack_frame.read().unwrap().parsing {
        // Defer expanding the \else if we're still parsing the test
        Ok(Tokens!(T_RELAX, (*local_token).clone()))
      } else if stack_frame.read().unwrap().elses {
        // Already seen an \else's at this level?
        let message = s!(
          "Extra {} already saw \\else for {:?} [{:?}] at {:?}",
          local_token.stringify(),
          stack_frame.read().unwrap().token,
          stack_frame.read().unwrap().ifid,
          stack_frame.read().unwrap().start
        );
        let local_token_str = local_token.to_string();
        Error!("unexpected", local_token_str, gullet, state, message);
        Ok(Tokens!())
      } else {
        state.set_ifframe(Some(Arc::clone(&stack_frame)));
        let t = self.skip_conditional_body(0, gullet, state);
        //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
        //       . " [for " . ToString($$LaTeXML::IFFRAME{token}) . " #" .
        // $$LaTeXML::IFFRAME{ifid}       . " skipping to " . ToString($t) . "]\n"
        //       if $STATE->lookupValue('TRACINGCOMMANDS');
        state.expire_ifframe();
        Ok(Tokens!())
      }
    } else {
      // No if stack entry ?
      let message = s!("Didn't expect a {:?} since we seem not to be in a conditional", local_token.stringify());
      let local_token_str = local_token.to_string();
      Error!("unexpected", local_token_str, gullet, state, message);
      Ok(Tokens!())
    }
  }

  pub fn invoke_fi(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    let stack_frame_opt: Option<Arc<RwLock<IfFrame>>> = if let Some(Stored::VecDequeStored(ref stack)) = state.lookup_value("if_stack") {
      if let Some(Stored::IfFrame(frame)) = stack.front() {
        Some(Arc::clone(frame))
      } else {
        None
      }
    } else {
      None
    };
    if let Some(stack_frame) = stack_frame_opt {
      if stack_frame.read().unwrap().parsing {
        // Defer expanding the \else if we're still parsing the test
        let local_token = state.get_current_token().unwrap();
        Ok(Tokens!(T_RELAX, (*local_token).clone()))
      } else {
        // "expand" by removing the stack entry for this level
        state.set_ifframe(Some(stack_frame));
        state.shift_value("if_stack"); // Done with this frame

        //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
        // . " [for " . Stringify($$LaTeXML::IFFRAME{token}) . " #" . $$LaTeXML::IFFRAME{ifid} .
        // "]\n"       if $STATE->lookupValue('TRACINGCOMMANDS');
        state.expire_ifframe();
        Ok(Tokens!())
      }
    } else {
      let message = s!(
        "Didn't expect a {:?} since we seem not to be in a conditional",
        state.get_current_token().unwrap().stringify()
      );
      Error!("unexpected", "fi", gullet, state, message);
      Ok(Tokens!())
    }
  }
}
