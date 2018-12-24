use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::store::Stored;
use crate::definition::expandable::Expandable;
use crate::definition::Definition;
use crate::document::Document;
use crate::list::List;
use crate::state::State;
use crate::token::Token;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, TexMode};

#[derive(Clone)]
pub struct Whatsit {
  pub args: Vec<Option<Digested>>,
  pub properties: HashMap<String, Stored>,
  pub definition: Rc<Definition>,
}

impl Default for Whatsit {
  fn default() -> Self {
    Whatsit {
      args: Vec::new(),
      properties: HashMap::new(),
      definition: Rc::new(Expandable::default()),
    }
  }
}
impl PartialEq for Whatsit {
  fn eq(&self, _other: &Whatsit) -> bool {
    false // TODO ?
  }
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

  pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    match self.args.get(n - 1) {
      Some(&Some(ref opt)) => Some(opt),
      _ => None,
    }
  }

  pub fn get_args(&self) -> &Vec<Option<Digested>> { &self.args }

  pub fn set_args(&mut self, args: Vec<Option<Digested>>) { self.args = args; }

  pub fn set_body(&mut self, mut body: Vec<Digested>) {
    let trailer_opt = body.pop();
    let mode = if self.is_math() {
      TexMode::Math
    } else {
      TexMode::Text
    };

    let mut list = List::new(body);
    if self.is_math() {
      list.mode = Some(mode);
    }
    self
      .properties
      .insert(s!("body"), Digested::List(Box::new(list)).into());
    if let Some(Digested::Whatsit(ref trailer)) = trailer_opt {
      // And copy any otherwise undefined properties from the trailer
      for (prop, value) in trailer.borrow().get_properties() {
        self
          .properties
          .entry(prop.to_string())
          .or_insert_with(|| value.clone());
      }
      self
        .properties
        .insert(s!("trailer"), trailer_opt.as_ref().unwrap().clone().into());
    }
  }
}

impl fmt::Debug for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Whatsit {{ args: {:?}, properties: {:?} }}",
      self.args, self.properties
    )
  }
}

impl BoxOps for Whatsit {
  fn to_string(&self) -> String {
    self.revert().to_string() // What else??
  }

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

  fn get_property(&self, key: &str, _state: &mut State) -> Option<Cow<Stored>> {
    match self.properties.get(key) {
      None => None,
      Some(v) => Some(Cow::Borrowed(v)),
    }
  }

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    self.properties.insert(key.to_string(), value.into());
  }

  fn get_body(&self) -> Option<Digested> {
    match self.properties.get("body") {
      Some(&Stored::Digested(ref body)) => Some(*body.clone()),
      _ => None,
    }
  }

  fn revert(&self) -> Tokens {
    // TODO - mock for now
    Tokens!()
  }

  fn get_font(&self) -> Option<Cow<Font>> {
    match self.properties.get("font") {
      Some(&Stored::Font(ref font)) => Some(Cow::Owned((**font).clone())),
      _ => None,
    }
  }

  fn get_locator(&self) -> Option<Locator> {
    // TODO
    None
  }
}

impl From<Whatsit> for Digested {
  fn from(w: Whatsit) -> Digested { Digested::Whatsit(Rc::new(RefCell::new(w))) }
}
