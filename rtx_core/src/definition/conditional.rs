use std::cell::RefCell;
use std::rc::Rc;

use common::error::*;
use common::object::Object;
use common::store::Stored;
use definition::{BeforeDigestClosure, ConditionalClosure, Definition, DigestionClosure};
use document::Document;
use gullet::Gullet;
use parameter::Parameters;
use state::{Scope, State};
use stomach::Stomach;
use token::*;
use tokens::Tokens;
use whatsit::Whatsit;
use Digested;

// Conditional control sequences; Expandable
//   Expand enough to determine true/false, then maybe skip
//   record a flag somewhere so that \else or \fi is recognized
//   (otherwise, they should signal an error)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionalType {
  If,
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
      "\\else" => Else,
      "\\or" => Or,
      "\\fi" => Fi,
      _ => If,
    }
  }
}

// This is ONLY used for \ifcase.
pub struct ConditionalOptions {
  pub scope: Option<Scope>,
  pub locked: Option<bool>,
  pub skipper: Option<bool>,
}
impl Default for ConditionalOptions {
  fn default() -> Self {
    ConditionalOptions {
      scope: None,
      locked: None,
      skipper: None,
    }
  }
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
      cs: T_CS!(s!("Conditional")),
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

impl Object for Conditional {}
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
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    // A real conditional must have condition_type set
    use self::ConditionalType::*;
    match self.conditional_type {
      If => self.invoke_conditional(gullet, state),
      Else => self.invoke_else(gullet, state),
      Or => self.invoke_else(gullet, state),
      Fi => self.invoke_fi(gullet, state),
      _ => {
        error!(
          target: &s!("unexpected:{}", self.cs), //$gullet,
          "Unknown conditional control sequence {:?}",
          state.current_token
        );
        Ok(Tokens!())
      },
    }
  }
  // TODO:
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_cs(&self) -> Token { self.cs.clone() }

  fn get_cs_name(&self) -> String { self.cs.get_cs_name() }

  fn get_locator(&self) -> String { String::from("Locator is TODO") }

  // Not implemented for expandable
  fn invoke_primitive(
    &self,
    _gullet: &mut Stomach,
    _caller: Rc<Definition>,
    _state: &mut State,
  ) -> Result<Vec<Digested>>
  {
    unimplemented!()
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
    _state: &mut State,
  ) -> Result<()>
  {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Conditional should never be called!"
    );
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfFrame {
  pub token: Token,
  pub start: String,
  pub parsing: bool,
  pub elses: bool,
  pub ifid: i32,
}

impl Conditional {
  pub fn invoke_conditional(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    // TODO!!! Implement in full
    // Keep a stack of the conditionals we are processing.
    let mut ifid = state.lookup_int("if_count");
    ifid += 1;
    state.assign_value("if_count", ifid, Some(Scope::Global));
    let if_frame = Rc::new(RefCell::new(IfFrame {
      token: state.current_token.as_ref().unwrap().clone(),
      start: gullet.get_locator(),
      parsing: true,
      elses: false,
      ifid: ifid,
    }));
    state.if_frame = Some(if_frame.clone());
    state.unshift_value("if_stack", vec![if_frame.clone()]);

    let args = self.read_arguments(gullet, state)?;

    if_frame.borrow_mut().parsing = false;
    //   my $tracing = $STATE->lookupValue('TRACINGCOMMANDS');
    //   print STDERR '{' . $self->tracingCSName . "} [#$ifid]\n" if $tracing;
    //   print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    if let Some(ref test) = self.test {
      if (test)(gullet, args, state)? {

        // print STDERR "{true}\n" if $tracing;
      } else {
        let _to = self.skip_conditional_body(-1, gullet, state);
        // print STDERR "{false} [skipped to " . ToString($to) . "]\n" if $tracing;
      }
    } else {
      // If there's no test, it must be the Special Case, \ifcase
      let num = args[0].to_number().value_of();
      if num > 0 {
        let _to = self.skip_conditional_body(num, gullet, state);
        //       print STDERR "{$num} [skipped to " . ToString($to) . "]\n" if $tracing;
      }
    }
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
  fn skip_conditional_body(&self, nskips: i32, gullet: &mut Gullet, state: &mut State) -> Tokens {
    let mut level = 1;
    let mut n_ors = 0;
    let _start = gullet.get_locator();
    // NOTE: Open-coded manipulation of if_stack!
    // [we're only reading tokens & looking up, so State shouldn't change behind our backs]

    let local_frame = state.if_frame.clone();
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
              if stack_frame != *local_frame.as_ref().unwrap() {
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
        _ => {
          if level <= 1 {
            // Ignore \else,\or nested in the body.
            if cond_type == Some(ConditionalType::Or) {
              n_ors += 1;
              if n_ors == nskips {
                return t;
              }
            } else if cond_type == Some(ConditionalType::Else) && nskips != 0 {
              // Found \else and we're looking for one?
              // Make sure this \else is NOT for a nested \if that is part of the test clause!
              if let Some(Stored::VecDequeStored(stack)) = state.lookup_value_mut("if_stack") {
                if let Some(Stored::IfFrame(mut stack_frame)) = stack.pop_front() {
                  if stack_frame == *local_frame.as_ref().unwrap() {
                    // No need to actually call elseHandler, but note that we've seen an \else!
                    stack_frame.borrow_mut().elses = true;
                    stack.push_front(stack_frame.into());
                    return t;
                  } else {
                    stack.push_front(stack_frame.into());
                  }
                }
              }
            }
          }
        },
      };
    }
    error!(target: "expected:\\fi", "Missing \\fi or \\else, conditional fell off end");
    Tokens!()
  }

  pub fn invoke_else(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    let local_token = state.current_token.as_ref().unwrap().clone();
    let stack_frame_opt =
      if let Some(Stored::VecDequeStored(stack)) = state.lookup_value_mut("if_stack") {
        if let Some(Stored::IfFrame(stack_frame)) = stack.front() {
          Some(stack_frame.clone())
        } else {
          None
        }
      } else {
        None
      };

    if let Some(stack_frame) = stack_frame_opt {
      if stack_frame.borrow().parsing {
        // Defer expanding the \else if we're still parsing the test
        Ok(Tokens!(T_CS!("\\relax"), local_token))
      } else if stack_frame.borrow().elses {
        // Already seen an \else's at this level?
        error!(
          target: &s!("unexpected:{:?}", local_token),
          "Extra {:?} already saw \\else for {:?} [{:?}] at {:?}",
          local_token,
          stack_frame.borrow().token,
          stack_frame.borrow().ifid,
          stack_frame.borrow().start
        );
        Ok(Tokens!())
      } else {
        state.if_frame = Some(stack_frame.clone());
        let t = self.skip_conditional_body(0, gullet, state);
        //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
        //       . " [for " . ToString($$LaTeXML::IFFRAME{token}) . " #" .
        // $$LaTeXML::IFFRAME{ifid}       . " skipping to " . ToString($t) . "]\n"
        //       if $STATE->lookupValue('TRACINGCOMMANDS');
        Ok(Tokens!())
      }
    } else {
      // No if stack entry ?
      error!(
        target: &s!("unexpected:{:?}", local_token),
        "Didn't expect a {:?} since we seem not to be in a conditional",
        local_token
      );
      Ok(Tokens!())
    }
  }

  pub fn invoke_fi(&self, _gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    let local_token = state.current_token.as_ref().unwrap().clone();
    let stack_frame_opt: Option<Rc<RefCell<IfFrame>>> =
      if let Some(Stored::VecDequeStored(ref stack)) = state.lookup_value("if_stack") {
        if let Some(Stored::IfFrame(frame)) = stack.front() {
          Some(frame.clone())
        } else {
          None
        }
      } else {
        None
      };

    if let Some(stack_frame) = stack_frame_opt {
      if stack_frame.borrow().parsing {
        // Defer expanding the \else if we're still parsing the test
        Ok(Tokens!(T_CS!("\\relax"), local_token))
      } else {
        // "expand" by removing the stack entry for this level
        state.if_frame = Some(stack_frame);
        state.shift_value("if_stack"); // Done with this frame

        //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
        // . " [for " . Stringify($$LaTeXML::IFFRAME{token}) . " #" . $$LaTeXML::IFFRAME{ifid} .
        // "]\n"       if $STATE->lookupValue('TRACINGCOMMANDS');
        Ok(Tokens!())
      }
    } else {
      error!(target: "unexpected:fi",  "Didn't expect a {:?} since we seem not to be in a conditional", state.current_token);
      Ok(Tokens!())
    }
  }
}
