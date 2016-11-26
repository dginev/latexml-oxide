use std::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;
use regex::Regex;

use common::model::Model;
// use stomach::{Stomach};

use token::{Catcode, Token};
use parameter::Parameter;
use definition::Definition;
use definition::expandable::Expandable;
use definition::constructor::Constructor;
use definition::primitive::Primitive;

#[derive(Clone)]
pub enum Scope {
  Global,
  Local,
}

#[derive(Clone)]
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
  String(String),
  VecChar(Vec<char>),
  VecString(Vec<String>),
  Bool(bool),
  Token(Token),
  Expandable(Arc<Expandable>),
  Primitive(Arc<Primitive>),
  Constructor(Arc<Constructor>),
}

impl fmt::Debug for ObjectStore {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use state::ObjectStore::*;
    match self {
      &String(ref s) => write!(f, "{}", s),
      &VecChar(ref vs) => write!(f, "vec of chars"),
      &VecString(ref vs) => write!(f, "vec of strings"),
      &Bool(ref b) => write!(f, "{}", b),
      &Token(ref t) => write!(f, "token"),
      &Expandable(ref expandable) => write!(f, "<closure for expandable definition>"),
      &Primitive(ref primitive) => write!(f, "<closure for primitive definition>"),
      &Constructor(ref constructor) => write!(f, "<closure for constructor definition>"),
    }
  }
}

pub struct State {
  pub verbosity: i32,
  pub map: Vec<String>,
  pub catcode: HashMap<char, Catcode>,
  pub meaning: HashMap<String, ObjectStore>,
  pub value: HashMap<String, ObjectStore>,
  pub parameters: HashMap<String, Parameter>,
  pub status_code: usize,
  pub unlocked: bool,
  pub model: Model,
}

impl Default for State {
  fn default() -> Self {
    State {
      // stomach : Stomach::default(),
      verbosity: 0,
      status_code: 0,
      unlocked: true,
      model: Model::default(),
      map: Vec::new(),
      catcode: HashMap::new(),
      meaning: HashMap::new(),
      value: HashMap::new(),
      parameters: HashMap::new(),
    }
  }
}

impl State {
  // TODO for all
  pub fn new() -> Self {
    use token::Catcode::*;
    // TODO: Only standard catcodes for now.

    // Setup default catcodes.
    let mut std_catcodes: HashMap<char, Catcode> = HashMap::new();
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

    State { catcode: std_catcodes, ..State::default() }
  }
  // $$self{value}{SPECIALS} = [['^', '_', '@', '~', '&', '$', '#', '%', "'"]];
  // if ($options{catcodes} eq 'style') {
  //   $$self{catcode}{'@'} = [LETTER]; }
  // }

  pub fn lookup_catcode<'lc>(&'lc mut self, c: &'lc char) -> Option<Catcode> {
    match self.catcode.get(c) {
      None => None,
      Some(&c) => Some(c.clone()),
    }
  }
  pub fn lookup_value<'lv>(&'lv self, key: &'lv str) -> Option<&ObjectStore> {
    self.value.get(key)
  }

  /// Get the `Meaning' of a token.  For active control sequence's
  /// this may give the definition object (if defined) or another token (if \let) or undef
  /// Any other token is returned as is.
  pub fn lookup_meaning<'t, 'm>(&'m mut self, token: &'t Token) -> Option<&ObjectStore>{
    if token.code.is_active_or_cs() && !token.text.is_empty() {

    } else {
      self.meaning.insert(token.text.clone(), ObjectStore::Token(token.clone()));
    }
    self.meaning.get(&token.text)
  }


  /// used for expansion & various queries
  /// Since we're not doing digestion here, we don't need to handle mathactive,
  /// nor cs let to executable tokens
  /// This returns a definition object, or undef
  pub fn lookup_definition<'def>(&'def mut self, key: &'def Token) -> Option<ObjectStore> {
    let cc = &key.code;
    let name = &key.text;
    let lookupname: String = if (cc == &Catcode::ACTIVE) || (cc == &Catcode::CS) {
      name.clone()
    } else {
      cc.name()
    };

    match lookupname.is_empty() {
      true => None,
      false => {
        match self.meaning.get(&lookupname) {
          None => None,
          Some(entry) => Some(entry.clone()),
        }
      }
    }
  }
  pub fn lookup_mathcode<'mc>(&'mc mut self, key: &'mc str) -> Option<Box<i32>> {
    None
  }
  pub fn lookup_mapping(&self, map: &str, key: &str) -> Option<&Parameter> {
    self.parameters.get(key)
  }

  pub fn lookup_digestable_definition<'def>(&'def mut self, token: &'def Token) -> Option<ObjectStore> {
    let cc = &token.code;
    let name = &token.text;
    if name.is_empty() {
      return None;
    }
    let lookupname = if (cc == &Catcode::ACTIVE) || (cc == &Catcode::CS) || ((cc == &Catcode::LETTER) || (cc == &Catcode::OTHER)) {
      // &&
      // self.lookup_value("IN_MATH").is_some() && ((self.lookup_mathcode(&name).is_some() || 0) == 0x8000)) {

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
      // if defn->isa('LaTeXML::Token')
      // && ($lookupname = $LaTeXML::Token::PRIMITIVE_NAME[$$defn[1]])
      // && ($entry      = $$self{meaning}{$lookupname})) {
      // $defn = $$entry[0]; }
      Some(defn.clone())
    } else {
      // println_stderr!("-- No definition for: {:?}", token);
      Some(ObjectStore::Token(token.clone()))
    }
  }
  pub fn assign_value<'av>(&'av mut self, key: &'av str, value: ObjectStore, scope: Option<Scope>) {
    self.assign_internal(Table::Value, key, value, scope);
    return;
  }
  pub fn assign_catcode<'ac>(&'ac mut self, _c: &'ac char, cc: Catcode) {}
  pub fn assign_definition<'def, T: Definition + Hash>(&'def mut self, _key: &'def Token, definition: Box<T>) {}
  pub fn assign_internal<'ai>(&'ai mut self, table: Table, key: &'ai str, definition: ObjectStore, _scope: Option<Scope>) {
    let mut fallback_store = HashMap::new();
    let mut store = match table {
      Table::Meaning => &mut self.meaning,
      Table::Value => &mut self.value,
      _ => &mut fallback_store,
    };

    store.insert(key.to_string(), definition);
  }
  pub fn assign_mapping<'mc>(&'mc mut self, map: &'mc str, key: &'mc str, value: Parameter) {
    self.parameters.insert(key.to_string(), value);
  }
  pub fn clear_prefixes<'cp>(&'cp mut self) {}

  /// And a shorthand for installing definitions
  pub fn install_definition<'id>(&'id mut self, definition: ObjectStore, scope: Option<Scope>) {
    // Locked definitions!!! (or should this test be in assignMeaning?)
    // Ignore attempts to (re)define $cs from tex sources
    //  my $cs = $definition->getCS->getCSName;
    let token = match &definition {
      &ObjectStore::Expandable(ref defn) => defn.get_cs(),
      &ObjectStore::Constructor(ref defn) => defn.get_cs(),
      &ObjectStore::Primitive(ref defn) => defn.get_cs(),
      &ObjectStore::Token(ref token) => token.clone(),
      _ => T_LETTER!("_wrong_argument_for_install_definition".to_string()),
    };
    let cs = token.get_cs_name();
    // println_stderr!("-- installing definition for: {:?}", token);

    let cs_locked = cs.clone() + ":locked";
    // TODO, .is_none() should be a real false check
    let is_cs_locked = match self.lookup_value(&cs_locked) {
      Some(&ObjectStore::Bool(ref x)) => *x,
      _ => false,
    };
    let is_state_unlocked: bool = match self.lookup_value("UNLOCKED") {
      Some(&ObjectStore::Bool(ref x)) => *x,
      _ => false,
    };
    if is_cs_locked && !is_state_unlocked {
      match self.lookup_value("SOURCEFILE") {
        Some(&ObjectStore::String(ref s)) => {
          lazy_static! {
            static ref tex_or_bib_ext_regex : Regex = Regex::new(r"\.(tex|bib)$").unwrap();
            static ref code_tex_ext_regex : Regex = Regex::new(r"\.code\.tex$").unwrap();
          }
          // report if the redefinition seems to come from document source
          if ((s == "Anonymous String") || tex_or_bib_ext_regex.is_match(&s)) && (!code_tex_ext_regex.is_match(&s)) {
            // TODO:
            //  info("ignore", cs, self.get_stomach(), "Ignoring redefinition of $cs");
          }
          return;
        }
        _ => {}
      };
    }
    self.assign_internal(Table::Meaning, &cs, definition, scope);
    return;
  }

  pub fn begin_semiverbatim(&self) {}
  pub fn end_semiverbatim(&self) {}
}
