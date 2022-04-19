use std::borrow::Cow;
// use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::expandable::Expandable;
use crate::definition::{Definition, Reversion};
use crate::document::Document;
use crate::list::List;
use crate::state::State;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, TexMode};

const REVERT_RAW: bool = false; // TODO: what is this about?
const DUAL_BRANCH: bool = false; // TODO: what is this about?

#[derive(Clone)]
pub struct Whatsit {
  pub args: Vec<Option<Digested>>,
  pub properties: HashMap<String, Stored>,
  pub definition: Arc<dyn Definition>,
  pub reversion: Option<Tokens>,
  pub dual_reversion: Option<Tokens>,
  pub locator: Locator,
}

impl Default for Whatsit {
  fn default() -> Self {
    Whatsit {
      args: Vec::new(),
      properties: HashMap::new(),
      definition: Arc::new(Expandable::default()),
      reversion: None,
      dual_reversion: None,
      locator: Locator::default(),
    }
  }
}
impl PartialEq for Whatsit {
  fn eq(&self, other: &Whatsit) -> bool { *self.definition == *other.definition && self.args == other.args && self.properties == other.properties }
}

impl Whatsit {
  pub fn is_math(&self) -> bool {
    match self.properties.get("isMath") {
      Some(&Stored::Bool(v)) => v,
      _ => false,
    }
  }

  pub fn get_properties(&self) -> &HashMap<String, Stored> { &self.properties }
  pub fn properties(self) -> HashMap<String, Stored> { self.properties }
  pub fn set_properties(&mut self, props: HashMap<String, Stored>) {
    for (key, value) in props {
      self.properties.insert(key, value);
    }
  }
  pub fn get_definition(&self) -> Arc<dyn Definition> { Arc::clone(&self.definition) }
  pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    match self.args.get(n - 1) {
      Some(&Some(ref opt)) => Some(opt),
      _ => None,
    }
  }
  pub fn get_args(&self) -> &Vec<Option<Digested>> { &self.args }
  pub fn set_args(&mut self, args: Vec<Option<Digested>>) { self.args = args; }
  pub fn get_trailer(&self) -> Option<Digested> {
    match self.properties.get("trailer") {
      Some(&Stored::Digested(ref triler)) => Some(*triler.clone()),
      _ => None,
    }
  }

  pub fn set_body(&mut self, mut body: Vec<Digested>) {
    let trailer_opt = body.pop();
    let mode = if self.is_math() { TexMode::Math } else { TexMode::Text };

    let mut list = List::new(body);
    if self.is_math() {
      list.mode = Some(mode);
    }
    self.properties.insert(s!("body"), Digested::List(Arc::new(list)).into());
    if let Some(Digested::Whatsit(ref trailer)) = trailer_opt {
      // And copy any otherwise undefined properties from the trailer
      for (prop, value) in trailer.read().unwrap().get_properties() {
        self.properties.entry(prop.to_string()).or_insert_with(|| value.clone());
      }
      self.properties.insert(s!("trailer"), trailer_opt.as_ref().unwrap().clone().into());
    }
  }
}

impl fmt::Debug for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Whatsit {{ args: {:?}, properties: {:?} }}", self.args, self.properties) }
}

impl fmt::Display for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut state = State::default();
    write!(f, "{}", self.revert(&mut state).unwrap()) // What else??
  }
}

impl Object for Whatsit {
  fn get_locator(&self) -> Cow<Locator> { Cow::Borrowed(&self.locator) }

  fn revert(&self, state: &mut State) -> Result<Tokens> {
    // WARNING: Forbidden knowledge?
    // (1) provide a means to get the RAW, internal markup that can (hopefully) be RE-digested
    //     this is needed for getting the numerator of \over into textstyle!
    // (2) caching the reversion (which is a big performance boost)
    let saved_opt = if REVERT_RAW {
      None
    } else if DUAL_BRANCH {
      // TODO
      unimplemented!()
      // self.dual_reversion.get(DUAL_BRANCH)
    } else {
      self.reversion.clone()
    };
    if let Some(saved) = saved_opt {
      Ok(saved)
    } else {
      let mut tokens = Vec::new();
      let defn = &self.definition;
      let spec_opt = if REVERT_RAW { None } else { defn.get_reversion_spec() };
      match spec_opt {
        Some(Reversion::Closure(spec)) => {
          // If handled by CODE, call it
          tokens = spec(self, self.get_args(), state).unwrap().unlist();
        },
        Some(Reversion::Tokens(spec)) => {
          if !spec.is_empty() {
            tokens = spec
              .substitute_parameters(
                self
                  .get_args()
                  .iter()
                  .map(|arg_opt| match arg_opt {
                    Some(arg) => arg.revert(state),
                    None => Ok(Tokens!()),
                  })
                  .collect::<Result<Vec<Tokens>>>()?,
              )
              .unlist();
          }
        },
        None => {
          let alias_opt: Option<String> = None; //if REVERT_RAW { None } else { None }; //TODO: defn.get_alias() };
          if let Some(alias) = alias_opt {
            if !alias.is_empty() {
              tokens.push(T_CS!(alias));
            }
          } else {
            tokens.push(defn.get_cs().into_owned());
          }
          if let Some(parameters) = defn.get_parameters() {
            // TODO: This is a sticking point. Both in terms of type mismatch between revert_arguments and get_args,
            // but much worse with the expectation of passing in a gullet and state for the parameter reversion
            // for now approximate this with some slight of hand ...
            // tokens.extend(parameters.revert_arguments(self.get_args())?);
            //
            // Note 2: I've already had to dance around the T_BEGIN/T_END wrappers with my hacky workaround
            // so maybe worth taking some time and aligning the idea here with `.revert_arguments` to avoid the insanity?
            //
            // GOAL: push(@tokens, $parameters->revertArguments($self->getArgs)); } }
            let args = self
              .get_args()
              .iter()
              .flatten()
              .map(|arg| arg.revert(state).unwrap_or_default())
              .collect();
            tokens.extend(parameters.revert_arguments(args, state)?)
          }
        },
      };

      if let Some(mut body) = self.get_body() {
        tokens.extend(body.revert(state)?.unlist());
        if let Some(mut trailer) = self.get_trailer() {
          tokens.extend(trailer.revert(state)?.unlist());
        }
      }

      // Now cache it, in case it's needed again
      // TODO: This causes a lot of mutability issues for arguable performance benefit.
      //       Maybe we are safe not using caching at all, and simply recomputing the reversion?
      // if !REVERT_RAW {
      //   // don't cache when RAW
      //   if DUAL_BRANCH {
      //     // self.dual_reversion = Some(Tokens::new(tokens.clone()));
      //   } else {
      //     self.reversion = Some(Tokens::new(tokens.clone()));
      //   }
      // }
      Ok(Tokens::new(tokens))
    }
  }
}

impl BoxOps for Whatsit {
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { &mut self.properties }
  fn unlist(&self) -> Vec<Digested> { Vec::new() }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $STATE->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Definition::startProfiling($profiled, 'absorb') if $profiled;
    // info!(target:"whatsit:be_absorbed", "{:?}", self);

    self.definition.do_absorbtion(document, self, state)?;
    // LaTeXML::Definition::stopProfiling($profiled, 'absorb') if $profiled;
    Ok(())
  }

  fn get_property(&self, key: &str, _state: &mut State) -> Option<Cow<Stored>> { self.properties.get(key).map(Cow::Borrowed) }

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) { self.properties.insert(key.to_string(), value.into()); }

  fn get_property_bool(&self, key: &str) -> bool {
    match self.properties.get(key) {
      Some(v) => *v == Stored::Bool(true),
      _ => false,
    }
  }
  fn get_body(&self) -> Option<Digested> {
    match self.properties.get("body") {
      Some(&Stored::Digested(ref body)) => Some(*body.clone()),
      _ => None,
    }
  }

  fn get_font(&self) -> Option<Cow<Font>> {
    match self.properties.get("font") {
      Some(&Stored::Font(ref font)) => Some(Cow::Owned((**font).clone())),
      _ => None,
    }
  }
}
