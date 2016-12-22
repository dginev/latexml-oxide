use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use state::{State, ObjectStore};
use list::List;
use definition::expandable::Expandable;
use {Digested, BoxOps, TexMode};
use definition::Definition;
use document::Document;

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
      definition: Rc::new(Expandable::default())
    }
  }
}

impl Whatsit {
  pub fn is_math(&self) -> bool {
    match self.properties.get("isMath") {
      Some(& ObjectStore::Bool(v)) => v,
      _ => false
    }
  }

  pub fn get_property(&self, key: &str) -> Option<&ObjectStore> {
    self.properties.get(key)
  }

  pub fn get_properties(&self) -> &HashMap<String, ObjectStore> {
    &self.properties
  }

  pub fn set_property(&mut self, key: &str, value: ObjectStore) {
    self.properties.insert(key.to_string(), value);
  }

  pub fn set_properties(&mut self, props: HashMap<String, ObjectStore>) {
    for (key, value) in props.into_iter() {
      self.properties.insert(key, value);
    }
  }

  pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    match self.args.get(n - 1) {
      None => None,
      Some(&None) => None,
      Some(&Some(ref opt)) => Some(&opt)
    }
  }

  pub fn get_args(&self) -> &Vec<Option<Digested>> {
    &self.args
  }

  pub fn set_args(&mut self, args: Vec<Option<Digested>>) {
    self.args = args;
  }

  pub fn get_body(&self) -> Option<&Digested> {
    match self.properties.get("body") {
      Some(& ObjectStore::Digested(ref body)) => Some(body),
      _ => None
    }
  }

  pub fn set_body(&mut self, mut body: Vec<Digested>) {
    let trailer_opt = body.pop();
    let mode = if self.is_math() {
      TexMode::Math
    } else {
      TexMode::Text
    };
    if !body.is_empty() {
      let list = List{ boxes: body, mode: mode };
      self.properties.insert("body".to_string(), ObjectStore::Digested(Rc::new(Digested::List(list))));
    }
    if let Some(trailer) = trailer_opt {
      self.properties.insert("trailer".to_string(), ObjectStore::Digested(Rc::new(trailer.clone())));
      // And copy any otherwise undefined properties from the trailer
      let trailer_whatsit = match trailer {
        Digested::Whatsit(w) => w,
        _ => Whatsit::default()
      };
      let trailer_props = trailer_whatsit.get_properties();
      for (prop, value) in trailer_props {
        self.properties.entry(prop.to_string()).or_insert(value.clone());
      }
    }
  }
}

impl fmt::Debug for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,
           "Whatsit {{ args: {:?}, properties: {:?} }}",
           self.args,
           self.properties)
  }
}

impl BoxOps for Whatsit {
  fn unlist(self) -> Vec<Digested> {
    Vec::new()
  }

  fn be_absorbed(mut self, document: &mut Document, state: &mut State) {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $STATE->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Definition::startProfiling($profiled, 'absorb') if $profiled;
    let self_mut = &mut self;
    self_mut.definition.do_absorbtion(document, self_mut, state);
    // LaTeXML::Definition::stopProfiling($profiled, 'absorb') if $profiled;
  }
}
