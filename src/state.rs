use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;

use common::model::{Model};
// use core::stomach::{Stomach};

use core::token::{Catcode, Token};
use common::object::Object;
use core::parameter::Parameter;
use core::definition::{Definition};
use core::definition::expandable::Expandable;
use core::definition::constructor::Constructor;
use core::definition::primitive::Primitive;

pub enum Scope {
  Global,
  Local
}
pub enum Table {
  Meaning,
  Value,
  SFCode,
  UCCode,
  DelCode,
  Stash,
  StashActive,
}

#[derive(Clone)]
pub enum ObjectStore {
  TokenStore (Token),
  ExpandableStore (Arc<Box<Expandable>>),
  PrimitiveStore (Arc<Box<Primitive>>),
  ConstructorStore (Arc<Box<Constructor>>)
}

pub struct State {
  pub verbosity : i32,
  pub map : Vec<String>,
  pub catcode : HashMap<char, Catcode>,
  pub meaning : HashMap<String, ObjectStore>,
  pub status_code : usize,
  pub model : Model
}

impl Default for State {
  fn default() -> Self {
    State {
      // stomach : Stomach::default(),
      verbosity : 0,
      status_code: 0,
      model : Model::default(),
      map : Vec::new(),
      catcode : HashMap::new(),
      meaning : HashMap::new()
    }
  }
}

impl State {// TODO for all
  pub fn new() -> Self {
    use core::token::Catcode::*;
    // TODO: Only standard catcodes for now.
    
    // Setup default catcodes.
    let mut std_catcodes : HashMap<char,Catcode> = HashMap::new();
    std_catcodes.insert('\\', ESCAPE);
    std_catcodes.insert('{', BEGIN);
    std_catcodes.insert('}', END);
    std_catcodes.insert('$', MATH);
    std_catcodes.insert('&', ALIGN);
    std_catcodes.insert('\r', EOL);
    std_catcodes.insert('#', PARAM);
    std_catcodes.insert('^', SUPER);
    std_catcodes.insert('_', SUB);
    std_catcodes.insert(' ', SPACE);
    std_catcodes.insert('\t', SPACE);
    std_catcodes.insert('%', COMMENT);
    std_catcodes.insert('~', ACTIVE);
    std_catcodes.insert('\0', IGNORE);
    for c in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
      std_catcodes.insert(c, LETTER);
    }

    State {
      catcode : std_catcodes,
      ..State::default()   
    }
  }
  // $$self{value}{SPECIALS} = [['^', '_', '@', '~', '&', '$', '#', '%', "'"]];
  // if ($options{catcodes} eq 'style') {
  //   $$self{catcode}{'@'} = [LETTER]; }
  // }

  pub fn lookup_catcode<'lc>(&'lc mut self, c: &'lc char) -> Option<Catcode> {
    match self.catcode.get(c) {
      None => None,
      Some(&c) => Some(c.clone())
    }
  }
  pub fn lookup_value<'lv, T: Hash>(&'lv mut self, key: &'lv str) -> Option<Box<T>>{
    None
  }
  /// used for expansion & various queries
  /// Since we're not doing digestion here, we don't need to handle mathactive,
  /// nor cs let to executable tokens
  /// This returns a definition object, or undef
  pub fn lookup_definition<'def>(&'def mut self, key: &'def Token) -> Option<ObjectStore> {
    let cc = &key.code;
    let name = &key.text;
    let lookupname : String = if (cc == &Catcode::ACTIVE) || (cc == &Catcode::CS) {
      name.clone()
    } else {
      cc.name()
    };
    
    match lookupname.is_empty() {
      true => None,
      false => {
        match self.meaning.get(&lookupname) {
          None => None,
          Some(entry) => Some(entry.clone())
        }
      }
    }
  }
  pub fn lookup_mathcode<'mc>(&'mc mut self, key: &'mc str) -> Option<Box<i32>> {
    None
  }
  pub fn lookup_mapping<'mc>(&'mc mut self, map: &'mc str, key: &'mc str) -> Option<Parameter> {
    None
  }

  pub fn lookup_digestable_definition<'def>(&'def mut self, token: &'def Token) -> Option<ObjectStore> {
    let cc = &token.code;
    let name = &token.text;
    if name.is_empty() {
      return None;
    }
    let lookupname = if (cc == &Catcode::ACTIVE) || (cc == &Catcode::CS) || 
      ((cc == &Catcode::LETTER) || (cc == &Catcode::OTHER)) {//&& 
      //self.lookup_value("IN_MATH").is_some() && ((self.lookup_mathcode(&name).is_some() || 0) == 0x8000)) {

      name.clone()
    } else {
      cc.name()
    };

    // println!("Looking up digestable {:?}", lookupname);
    let entry = self.meaning.get(&lookupname);
    
    if !lookupname.is_empty() && entry.is_some() {
      // println_stderr!("-- Found definition for: {:?}", token);
      let defn = entry.unwrap();
      // If a cs has been let to an executable token, lookup ITS defn.
      // if defn->isa('LaTeXML::Core::Token')
      // && ($lookupname = $LaTeXML::Core::Token::PRIMITIVE_NAME[$$defn[1]])
      // && ($entry      = $$self{meaning}{$lookupname})) {
      // $defn = $$entry[0]; }
      Some(defn.clone())
    } else {
      // println_stderr!("-- No definition for: {:?}", token);
      Some(ObjectStore::TokenStore(token.clone()))
    }
  }
  pub fn assign_value<'av, T: Hash>(&'av mut self, key: &'av str, value: Box<T>, scope: &'av Scope) {}
  pub fn assign_catcode<'ac>(&'ac mut self, c: &'ac char, cc : Catcode) {}
  pub fn assign_definition<'def, T: Definition + Hash>(&'def mut self, key: &'def Token, definition : Box<T>) { }
  pub fn assign_internal<'ai>(&'ai mut self, table : Table, key : &'ai str, definition : ObjectStore, 
                              scope : &'ai Option<Scope>) {
    let mut fallback_store = HashMap::new();
    let mut store = match table {
      Table::Meaning => &mut self.meaning,
      _ => &mut fallback_store
    };

    store.insert(key.to_string(), definition);
  }
  pub fn assign_mapping<'mc>(&'mc mut self, map: &'mc str, key: &'mc str, value: Parameter) {
    
  }
  pub fn clear_prefixes<'cp>(&'cp mut self) {}

  /// And a shorthand for installing definitions
  pub fn install_definition<'id>(&'id mut self, definition: ObjectStore, scope: &'id Option<Scope>) {
    // Locked definitions!!! (or should this test be in assignMeaning?)
    // Ignore attempts to (re)define $cs from tex sources
    //  my $cs = $definition->getCS->getCSName;
    let token = match &definition {
      &ObjectStore::ExpandableStore(ref defn) => {defn.get_cs()},
      &ObjectStore::ConstructorStore(ref defn) => {defn.get_cs()},
      &ObjectStore::PrimitiveStore(ref defn) => {defn.get_cs()},
      &ObjectStore::TokenStore(ref token) => {token.clone()},
    };
    let cs = token.get_cs_name();

    let cs_locked = cs.clone() + ":locked";
    // TODO, .is_none() should be a real false check
    let is_cs_locked : Option<Box<bool>> = self.lookup_value(&cs_locked);
    let is_state_unlocked : Option<Box<bool>> = self.lookup_value("UNLOCKED");
    if is_cs_locked.is_some() && is_state_unlocked.is_none() {
      match self.lookup_value("SOURCEFILE") {
        Some(s) => {
          let tex_or_bib_ext_regex = regex!(r"\.(tex|bib)$");
          let code_tex_ext_regex = regex!(r"\.code\.tex$");
          // report if the redefinition seems to come from document source
          if ((*s == "Anonymous String") || tex_or_bib_ext_regex.is_match(*s)) && (! code_tex_ext_regex.is_match(*s)) {
            // TODO:
            //  info("ignore", cs, self.get_stomach(), "Ignoring redefinition of $cs");
          }
          return;
        },
        None => {}
      };
    }
    self.assign_internal(Table::Meaning, &cs, definition, scope);
    return;
  }

  pub fn begin_semiverbatim(&self) {}
  pub fn end_semiverbatim(&self) {}
}