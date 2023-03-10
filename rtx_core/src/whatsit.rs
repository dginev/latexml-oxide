use std::borrow::Cow;
// use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;
use libxml::tree::Node;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::expandable::Expandable;
use crate::definition::{Definition, FontDirective, Reversion};
use crate::document::Document;
use crate::list::List;
use crate::state::State;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, DigestedData, TexMode};

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
  fn eq(&self, other: &Whatsit) -> bool {
    // identical definition, argument list and body
    *self.definition == *other.definition
      && self.args == other.args
      && if let Some(Stored::Digested(body1)) = self.properties.get("body") {
        if let Some(Stored::Digested(body2)) = other.properties.get("body") {
          *body1 == *body2
        } else {
          false
        }
      } else {
        other.properties.get("body").is_none()
      }
  }
}

impl Whatsit {
  pub fn is_math(&self) -> bool {
    match self.properties.get("isMath") {
      Some(&Stored::Bool(v)) => v,
      _ => false,
    }
  }
  pub fn is_empty(&self) -> bool { self.args.is_empty() }
  pub fn set_properties(&mut self, props: HashMap<String, Stored>) {
    for (key, value) in props {
      self.properties.insert(key, value);
    }
  }
  pub fn get_definition(&self) -> Arc<dyn Definition> { Arc::clone(&self.definition) }
  pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    match self.args.get(n - 1) {
      Some(Some(opt)) => Some(opt),
      _ => None,
    }
  }
  pub fn get_arg_mut(&mut self, n: usize) -> Option<&mut Digested> {
    match self.args.get_mut(n - 1) {
      Some(Some(opt)) => Some(opt),
      _ => None,
    }
  }
  pub fn get_args(&self) -> &Vec<Option<Digested>> { &self.args }
  pub fn set_args(&mut self, args: Vec<Option<Digested>>) { self.args = args; }
  pub fn get_trailer(&self) -> Option<Digested> {
    match self.properties.get("trailer") {
      Some(Stored::Digested(trailer)) => Some(trailer.clone()),
      _ => None,
    }
  }

  pub fn set_body(&mut self, mut body: Vec<Digested>, state: &mut State) {
    let trailer_opt = body.pop();
    let mode = if self.is_math() { TexMode::Math } else { TexMode::Text };

    let mut list = List::new(body, state);
    if self.is_math() {
      list.mode = Some(mode);
    }
    self.properties.insert(s!("body"), Digested::from(list).into());
    if let Some(digested) = trailer_opt {
      if let DigestedData::Whatsit(ref trailer) = digested.data() {
        // And copy any otherwise undefined properties from the trailer
        let trailer_val = trailer.read().unwrap();
        let props = trailer_val.get_properties();
        for (prop, value) in props {
          self.properties.entry(prop.to_string()).or_insert_with(|| value.clone());
        }
        self.properties.insert(s!("trailer"), digested.clone().into());
      }
    }
  }

  /// Like Tokens-substituteParameters, but substitutes in the Whatsit's arguments OR properties!
  /// #<digit> is the standard TeX positional argument
  /// # followed by a T_OTHER(propname) specifies the property propname!!
  fn substitute_parameters(&self, spec: Tokens, state: &State) -> Result<Vec<Token>> {
    // TODO: This is kind of unfortunate -- I am not sure what are the reasonable "entryways" into the Whatsit substituteParameters. For Expandable we
    // now have guarantees that "#,i" has been mapped into a single T_ARG(#i), but not here. so for now run on each call?
    let mut in_toks = VecDeque::from(spec.unlist());
    let args = self.get_args();
    let props = &self.properties;
    let mut result = Vec::new();
    while let Some(token) = in_toks.pop_front() {
      if token.get_catcode() != Catcode::ARG {
        // Non '#'; copy it
        result.push(token);
      } else {
        let s = token.get_string();
        let n = s.parse::<usize>().unwrap() - 1;
        let arg_opt = if n < 10 {
          args[n].clone()
        } else {
          match props.get(s) {
            Some(Stored::Digested(v)) => Some((*v).clone()),
            Some(other) => panic!("unexpected prop in substitute_parameters, needed Digested, got: {other:?}"),
            None => None,
          }
        };
        if let Some(arg) = arg_opt {
          result.extend(arg.revert(state)?.unlist());
        }
      }
    }
    Ok(result)
  }
}

impl fmt::Debug for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Whatsit {{ args: {:?}, properties: {:?} }}", self.args, self.properties) }
}

impl fmt::Display for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut state = State::default();
    write!(f, "{}", self.revert(&state).unwrap()) // What else??
  }
}

impl Object for Whatsit {
  fn get_locator(&self) -> Option<Cow<Locator>> { Some(Cow::Borrowed(&self.locator)) }

  fn revert(&self, state: &State) -> Result<Tokens> {
    // WARNING: Forbidden knowledge?
    // (1) provide a means to get the RAW, internal markup that can (hopefully) be RE-digested
    //     this is needed for getting the numerator of \over into textstyle!
    // (2) caching the reversion (which is a big performance boost)
    let saved_opt = if DUAL_BRANCH {
      // TODO, also alignment case
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
      let spec_opt = if let Some(rev) = self.properties.get("reversion") {
        match rev {
          Stored::Tokens(tks) => Some(Reversion::Tokens(tks.clone())),
          // TODO?
          // Stored::ReversionClosure(rfn) => Some(Reversion::Closure(rfn)),
          other => panic!("TODO: Unexpected reversion directive {other:?}"),
        }
      } else {
        defn.get_reversion_spec()
      };
      match spec_opt {
        Some(Reversion::Closure(spec)) => {
          let spec_tokens = spec(self, self.get_args(), state).unwrap();
          tokens = self.substitute_parameters(spec_tokens, state)?;
        },
        Some(Reversion::Tokens(spec)) => {
          if !spec.is_empty() {
            tokens = self.substitute_parameters(spec, state)?;
          }
        },
        None => {
          if let Some(alias) = defn.get_alias() {
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
              .map(|opt| match opt {
                Some(arg) => Some(arg.revert(state).ok()?),
                None => None,
              })
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
      // TODO: This causes a lot of mutability issues for arguable performance benefit. Maybe we are safe not using caching at all, and simply
      // recomputing the reversion? if !REVERT_RAW {
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
  fn get_properties(&self) -> &HashMap<String, Stored> { &self.properties }
  fn get_string(&self, state: &State) -> Result<Cow<str>> { Ok(Cow::Owned(self.revert(state)?.to_string())) }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<Vec<Node>> {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $STATE->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Definition::startProfiling($profiled, 'absorb') if $profiled;
    // info!(target:"whatsit:be_absorbed", "{:?}", self);

    self.definition.do_absorbtion(document, self, state)
    // LaTeXML::Definition::stopProfiling($profiled, 'absorb') if $profiled;

  }

  fn get_property(&self, key: &str) -> Option<Cow<Stored>> { self.properties.get(key).map(Cow::Borrowed) }
  fn has_property(&self, key: &str) -> bool { self.properties.contains_key(key) }

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) { self.properties.insert(key.to_string(), value.into()); }

  fn get_property_bool(&self, key: &str) -> bool { matches!(self.properties.get(key), Some(Stored::Bool(true))) }
  fn get_body(&self) -> Option<Digested> {
    match self.properties.get("body") {
      Some(Stored::Digested(body)) => Some(body.clone()),
      _ => None,
    }
  }

  fn get_font(&self, state: &mut State) -> Result<Option<Cow<Font>>> {
    match self.properties.get("font") {
      Some(Stored::Font(font)) => Ok(Some(Cow::Owned((**font).clone()))),
      Some(Stored::FontDirective(fd)) => match fd {
        FontDirective::Closure(ref code) => Ok(Some(Cow::Owned(code(Some(self), state)?))),
        FontDirective::Asset(ref asset) => Ok(Some(Cow::Borrowed(asset))),
      },
      _ => Ok(None),
    }
  }

  fn set_font(&mut self, font: Arc<Font>) { self.properties.insert("font".to_string(), Stored::Font(font)); }

  fn compute_size(&self, options: HashMap<String, Stored>, state: &mut State) -> Result<(Dimension, Dimension, Dimension)> {
    let defn = self.get_definition();
    if let Some(sizer) = defn.get_sizer() {
      sizer(self, state)
    } else {
      // Nothing specified? use #body if any, else sum all box args
      let mut boxes = Vec::new();
      if let Some(mut body_stored) = self.get_property("body") {
        if let Stored::Digested(ref body) = *body_stored {
          boxes.push((*body).clone());
        }
      }
      if boxes.is_empty() {
        // no body
        for arg in self.args.iter().flatten() {
          boxes.extend(arg.unlist().into_iter());
        }
      }
      let font = if let Stored::Font(ref sf) = *self.get_property("font").unwrap() {
        sf.clone()
      } else {
        state.lookup_font().unwrap()
      };
      font.compute_boxes_size(&boxes, options, state)
    }
  }
}
