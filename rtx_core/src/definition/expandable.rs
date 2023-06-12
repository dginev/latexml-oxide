use std::borrow::Cow;
// use std::fmt;
use libxml::tree::Node;

use crate::common::error::*;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::state::{Scope, State};

use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure, ExpansionBody};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::{Tokens, NO_TOKENS};
use crate::whatsit::Whatsit;
use crate::Digested;

#[derive(Debug, Clone, Default)]
pub struct ExpandableOptions {
  pub locked: bool,
  pub protected: bool,
  pub outer: bool,
  pub long: bool,
  pub scope: Option<Scope>,
  pub alias: Option<String>,
  pub mathactive: bool,
  pub robust: bool,
  pub nopack_parameters: bool,
}

#[derive(Debug, Clone)]
pub struct Expandable {
  pub is_protected: bool,
  pub is_long: bool,
  pub is_outer: bool,
  pub alias: Option<String>,
  pub locator: Locator,
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub expansion: Option<ExpansionBody>,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected: false,
      is_long: false,
      is_outer: false,
      alias: None,
      locator: Locator::default(),
      cs: T_CS!("Expandable"),
      paramlist: None,
      expansion: None,
    }
  }
}
impl PartialEq for Expandable {
  fn eq(&self, other: &Expandable) -> bool {
    self.paramlist == other.paramlist && self.expansion == other.expansion
  }
}

// impl fmt::Display for Expandable {
//   fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//     unimplemented!();
//   }
// }
impl Object for Expandable {
  fn is_definition(&self) -> bool { true }
  fn is_expandable(&self) -> bool { true }
  fn get_locator(&self) -> Option<Cow<Locator>> { Some(Cow::Borrowed(&self.locator)) }
  fn stringify(&self) -> String { <Self as Definition>::stringify_type(self, "Expandable") }
}
impl Definition for Expandable {
  fn is_protected(&self) -> bool { self.is_protected }
  fn get_parameters(&self) -> Option<&Parameters> { self.paramlist.as_ref() }
  fn get_cs(&self) -> Cow<Token> { Cow::Borrowed(&self.cs) }
  fn get_cs_name(&self) -> Cow<str> {
    match self.alias {
      Some(ref alias) => Cow::Borrowed(alias),
      None => Cow::Owned(self.cs.with_cs_name(ToString::to_string)),
    }
  }
  // fn with_cs_name<R, FnR>(&self, caller: FnR) -> R
  // where FnR: FnOnce(&str) -> R {
  //   match self.alias {
  //     Some(ref alias) => caller(alias),
  //     None => self.cs.with_cs_name(caller),
  //   }
  // }
  fn get_expansion(&self) -> Option<&ExpansionBody> { self.expansion.as_ref() }
  fn get_alias(&self) -> Option<&String> { self.alias.as_ref() }

  /// Expand the expandable control sequence. This should be carried out by the Gullet.
  fn invoke(&self, gullet: &mut Gullet, once_only: bool, state: &mut State) -> Result<Tokens> {
    // shortcut for "trivial" macros; but only if not tracing & profiling!!!!
    let tracing = state.lookup_int("TRACINGMACROS") > 0;
    let profiled = state.lookup_bool("PROFILING");
    match &self.expansion {
      Some(ExpansionBody::Closure(closure)) => {
        // Harder to emulate \tracingmacros here.
        let args = if let Some(ref parms) = self.paramlist {
          parms.read_arguments(gullet, Some(self), state)?
        } else {
          Vec::new()
        };
        if profiled {
          // LaTeXML::Core::Definition::startProfiling($profiled, 'expand')
          unimplemented!();
        }
        let result = closure(gullet, args, state)?;
        if tracing {
          //$LaTeXML::DEBUG{tracing}) {    # More involved...
          unimplemented!();
          // Debug($self->tracingCSName . ' ==> ' . tracetoString($result));
          // Debug($self->tracingArgs(@args)) if @args; } }
        }
        Ok(result)
      },
      Some(ExpansionBody::Tokens(tokens)) => {
        let result = if self.paramlist.is_none() {
          if profiled {
            unimplemented!();
            // LaTeXML::Core::Definition::startProfiling($profiled, 'expand')
          }
          if tracing {
            unimplemented!();
            // Debug($self->tracingCSName . ' ->' . tracetoString($expansion))
            //   if $tracing || $LaTeXML::DEBUG{tracing};
          }
          // For trivial expansion, make sure we don't get \cs or \relax\cs direct recursion!
          let is_recursion = if !once_only {
            let token_vec = tokens.unlist_ref();
            let t0_opt = token_vec.get(0);
            let t1_opt = token_vec.get(1);
            if let Some(t0) = t0_opt {
              if t0 == &self.cs {
                true
              } else if let Some(t1) = t1_opt {
                t1 == &self.cs && t0 == &T_CS!("\\protect")
              } else {
                false
              }
            } else {
              false
            }
          } else {
            false
          };
          if is_recursion {
            Tokens!()
          } else {
            tokens.clone()
          }
        } else {
          let args = if let Some(ref parms) = self.paramlist {
            parms.read_arguments(gullet, Some(self), state)?
          } else {
            Vec::new()
          };
          let mut args_tks = Vec::new();
          for arg in args.iter() {
            args_tks.push(arg.as_tokens(state)?);
          }
          // for "real" macros, make sure all args are Tokens
          // let r;
          if tracing {
            // || $LaTeXML::DEBUG{tracing}) {    # More involved...
            unimplemented!();
            // Debug($self->tracingCSName . ' ->' . tracetoString($expansion));
            // Debug($self->tracingArgs(@targs)) if @args;
          }
          if profiled {
            unimplemented!();
            //LaTeXML::Core::Definition::startProfiling($profiled, 'expand');
          }
          tokens.substitute_parameters(args_tks.as_slice())
        };
        // Getting exclusive requires dubious Gullet support!
        if profiled {
          unimplemented!();
          // result = Tokens!(result, T_MARKER!(profiled));
        }
        Ok(result)
      },
      None => {
        // we always need to read the arguments, for e.g. things like \@gobble
        if let Some(ref parms) = self.paramlist {
          parms.read_arguments(gullet, Some(self), state)?;
        }
        Ok(NO_TOKENS)
      },
    }
  }

  // Not implemented for expandable
  fn invoke_primitive(&self, _gullet: &mut Stomach, _state: &mut State) -> Result<Vec<Digested>> {
    Ok(Vec::new())
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
    _state: &mut State,
  ) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Expandable should never be called!"
    );
  }
}

impl Expandable {
  pub fn new<T: Into<ExpansionBody>>(
    cs: Token,
    paramlist: Option<Parameters>,
    expansion: T,
    traits: Option<ExpandableOptions>,
    state: &State,
  ) -> Result<Self> {
    let mut expansion: ExpansionBody = expansion.into();
    let traits = traits.unwrap_or_default();
    if !traits.nopack_parameters {
      if let ExpansionBody::Tokens(expansion_tokens) = expansion {
        expansion = ExpansionBody::Tokens(Tokens::pack_parameters(expansion_tokens)?);
      }
    }
    // simplify: treat empty tokens as None
    let expansion = match expansion {
      ExpansionBody::Tokens(tks) if tks.is_empty() => None,
      other => Some(other),
    };
    Ok(Expandable {
      cs,
      paramlist,
      expansion,
      // locator           => $source->getLocator,
      is_protected: traits.protected || state.get_prefix("protected"),
      is_outer: traits.outer || state.get_prefix("outer"),
      is_long: traits.long || state.get_prefix("long"),
      ..Expandable::default()
    })
  }
}
