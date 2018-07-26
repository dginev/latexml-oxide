use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use common::error::*;
use common::font::Font;
use definition::expandable::Expandable;
use definition::Definition;
use document::Document;
use list::List;
use state::{ObjectStore, State};
use tokens::Tokens;
use {BoxOps, Digested, TexMode};

#[derive(Clone)]
pub struct Whatsit {
  pub args: Vec<Option<Digested>>,
  pub properties: HashMap<String, ObjectStore>,
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
      Some(&ObjectStore::Bool(v)) => v,
      _ => false,
    }
  }

  pub fn get_properties(&self) -> &HashMap<String, ObjectStore> { &self.properties }
  pub fn properties(self) -> HashMap<String, ObjectStore> { self.properties }

  pub fn set_properties(&mut self, props: HashMap<String, ObjectStore>) {
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
    self.properties.insert(
      s!("body"),
      ObjectStore::Digested(Rc::new(Digested::List(list))),
    );
    if let Some(trailer) = trailer_opt {
      self.properties.insert(
        s!("trailer"),
        ObjectStore::Digested(Rc::new(trailer.clone())),
      );
      // And copy any otherwise undefined properties from the trailer
      let trailer_whatsit = match trailer {
        Digested::Whatsit(w) => w,
        _ => Whatsit::default(),
      };
      let trailer_props = trailer_whatsit.get_properties();
      for (prop, value) in trailer_props {
        self
          .properties
          .entry(prop.to_string())
          .or_insert_with(|| value.clone());
      }
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

  fn unlist(self) -> Vec<Digested> { Vec::new() }

  fn be_absorbed(mut self, document: &mut Document, state: &mut State) -> Result<()> {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $STATE->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Definition::startProfiling($profiled, 'absorb') if $profiled;
    let self_mut = &mut self;
    self_mut
      .definition
      .do_absorbtion(document, self_mut, state)?;
    // LaTeXML::Definition::stopProfiling($profiled, 'absorb') if $profiled;
    Ok(())
  }

  fn get_property(&self, key: &str) -> Option<&ObjectStore> { self.properties.get(key) }

  fn set_property(&mut self, key: &str, value: ObjectStore) {
    self.properties.insert(key.to_string(), value);
  }

  fn get_body(&self) -> Option<&Digested> {
    match self.properties.get("body") {
      Some(&ObjectStore::Digested(ref body)) => Some(body),
      _ => None,
    }
  }

  fn revert(&self) -> Tokens {
    // TODO - mock for now
    Tokens!()
  }

  fn get_font(&self) -> Option<&Font> {
    match self.properties.get("font") {
      Some(&ObjectStore::Font(ref font)) => Some(font),
      _ => None,
    }
  }
}
