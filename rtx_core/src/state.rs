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
  Catcode,
  SFCode,
  UCCode,
  DelCode,
  Stash,
  StashActive,
}

#[derive(Clone)]
pub enum ObjectStore {
  // Primitives
  Bool(bool),
  String(String),
  // LaTeXML objects
  Catcode(Catcode),
  Token(Token),
  Expandable(Arc<Expandable>),
  Primitive(Arc<Primitive>),
  Constructor(Arc<Constructor>),
  Digested(Arc<::Digested>),

  // Collections
  VecChar(Vec<char>),
  VecString(Vec<String>),
  VecToken(Vec<Token>),
  VecDigested(Vec<::Digested>)
}

impl fmt::Debug for ObjectStore {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use state::ObjectStore::*;
    match self {
      &String(ref s) => write!(f, "{:?}", s),
      &VecChar(ref vs) => write!(f, "{:?}",vs),
      &VecString(ref vs) => write!(f, "{:?}", vs),
      &Bool(ref b) => write!(f, "{:?}", b),
      &Token(ref t) => write!(f, "{:?}", t),
      &Catcode(ref cc) => write!(f, "{:?}", cc),
      &Expandable(ref _expandable) => write!(f, "<closure for expandable definition>"),
      &Primitive(ref _primitive) => write!(f, "<closure for primitive definition>"),
      &Constructor(ref _constructor) => write!(f, "<closure for constructor definition>"),
      &Digested(ref digested) => write!(f, "{:?}", digested),
      &VecToken(ref token_vec) => write!(f, "{:?}", token_vec),
      &VecDigested(ref digested_vec) => write!(f, "{:?}", digested_vec),
    }
  }
}
impl fmt::Display for ObjectStore {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

pub struct State {
  pub verbosity: i32,
  pub map: Vec<String>,
  pub catcode: HashMap<char, Catcode>,
  pub mathcode: HashMap<char, Catcode>,
  pub meaning: HashMap<String, ObjectStore>,
  pub value: HashMap<String, ObjectStore>,
  pub parameters: HashMap<String, Parameter>,
  pub undo: Vec<HashMap<String, ObjectStore>>,
  pub status_code: usize,
  pub unlocked: bool,
  pub model: Model,
  pub current_token: Option<Token>,
  pub noexpand_the: bool
}

impl Default for State {
  fn default() -> Self {
    let mut locked_frame_hash = HashMap::new();
    locked_frame_hash.insert("_FRAME_LOCK_".to_string(), ObjectStore::Bool(true));
    State {
      // stomach : Stomach::default(),
      verbosity: 0,
      status_code: 0,
      unlocked: true,
      model: Model::default(),
      map: Vec::new(),
      catcode: HashMap::new(),
      mathcode: HashMap::new(),
      meaning: HashMap::new(),
      value: HashMap::new(),
      parameters: HashMap::new(),
      undo: vec![locked_frame_hash],
      current_token: None,
      noexpand_the: false
    }
  }
}

lazy_static! {
  static ref TEX_OR_BIB_EXT_RE : Regex = Regex::new(r"\.(tex|bib)$").unwrap();
  static ref CODE_TEX_EXT_RE : Regex = Regex::new(r"\.code\.tex$").unwrap();
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

  /// A bit of Perl "truth as existence" semantics mixed in with proper boolean lookup
  pub fn lookup_bool(&self, key: &str) -> bool {
    match self.lookup_value(key) {
      Some(& ObjectStore::Bool(ref v)) => *v,
      Some(_) => true,
      None => false
    }
  }

  pub fn lookup_value<'lv>(&'lv self, key: &'lv str) -> Option<&ObjectStore> {
    self.value.get(key)
  }

  pub fn remove_value<'lv>(&'lv mut self, key: &'lv str) -> Option<ObjectStore> {
    self.value.remove(key)
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

  /// $meaning should be a definition (for defining active control sequences)
  /// or another token, for \let
  pub fn assign_meaning<'t, 'm>(&'m mut self, token: &'t Token, meaning: ObjectStore, scope: Option<Scope>) {
    self.assign_internal(Table::Meaning, &token.get_cs_name(), meaning, scope);
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
  pub fn lookup_mathcode<'mc>(&'mc mut self, key: &char) -> Option<Catcode> {
    match self.mathcode.get(key) {
      None => None,
      Some(&c) => Some(c.clone()),
    }
  }

  pub fn lookup_mapping(&self, _map: &str, key: &str) -> Option<&Parameter> {
    // TODO:
    // let vtable = self.value;
    // if let Some(&ObjectStore::VecDigested(ref mapping)) = vtable.get(map) {
    //   if mapping.is_empty() {
    //     None
    //   } else {
    //     let first_mapping = mapping[0];
    //     first_mapping.get(key)
    //   }
    // } else {
    //   None
    // }
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
  pub fn assign_catcode<'ac>(&'ac mut self, key:char, value: Catcode, scope: Option<Scope>) {
    self.assign_internal(Table::Catcode, &key.to_string(), ObjectStore::Catcode(value), scope);
  }


  pub fn assign_definition<'def, T: Definition + Hash>(&'def mut self, _key: &'def Token, _definition: Box<T>) {}

  /// TODO: Handle scopes and undo table
  pub fn assign_internal<'ai>(&'ai mut self, table: Table, key: &'ai str, definition: ObjectStore, _scope: Option<Scope>) {
    let mut fallback_store = HashMap::new();
    match table {
      Table::Meaning => {self.meaning.insert(key.to_string(), definition);},
      Table::Value => {self.value.insert(key.to_string(), definition);},
      Table::Catcode => if let ObjectStore::Catcode(cc) = definition {
        self.catcode.insert(key.chars().next().unwrap(), cc);
      },
      _ => {fallback_store.insert(key.to_string(), definition);},
    };
  }
  pub fn assign_mapping<'mc>(&'mc mut self, _map: &'mc str, key: &'mc str, value: Parameter) {
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
          // report if the redefinition seems to come from document source
          if ((s == "Anonymous String") || TEX_OR_BIB_EXT_RE.is_match(&s)) && (!CODE_TEX_EXT_RE.is_match(&s)) {
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

  // NOTE: Common usage patterns seem to be to lookup
  //   expandable definitions
  //   register values
  //   conditionals
  //   digestibles
  // or just variants on testing defined-ness
  // May be will introduce more clarity (possibly efficiency)
  // to collect those more uniformly and implement here, or in Package

  //======================================================================

  pub fn push_frame(&mut self) {
    // Easy: just push a new undo hash.
    self.undo.push(HashMap::new());
  }

  pub fn pop_frame(&mut self) {
    if self.undo.last().as_ref().unwrap().get("_FRAME_LOCK_").is_some() {
      panic!("Fatal:unexpected:<endgroup> attempt to pop last locked stack frame");
      // Fatal('unexpected', '<endgroup>', $self->getStomach,
        // "Attempt to pop last locked stack frame"); }
    } else {
      // TODO:
      let _undo = self.undo.pop();
      // for (table, undotable) in undo.into_iter() {
      //   for (name, val) in undotable.into_iter() {
      //     // Typically only 1 value to shift off the table, unless scopes have been activated.
      //     let mut pop_count = val;
      //     while pop_count > 0 {
      //       pop_count -= 1;
      //       match table {
      //         "value" =>
      //         "meaning" =>
      //         "catcode" =>
      //       }
      //     }
      //     map { shift(@{ $$self{$table}{$name} }) } 1 .. $$undotable{$name};
        // }
      // }
    }
  }


  pub fn begin_semiverbatim(&mut self, extraspecials: Option<Vec<Token>>) {
    // Is this a good/safe enough shorthand, or should we really be doing beginMode?
    self.push_frame();
    self.assign_value("MODE", ObjectStore::String("text".to_string()), None);
    self.assign_value("IN_MATH", ObjectStore::Bool(false), None);
    let mut all_specials : Vec<char> = Vec::new();
    if let Some(extra) = extraspecials {
      for special in extra {
        let special_char = special.text.chars().next().unwrap();
        all_specials.push(special_char);
      }
    }
    if let Some(&ObjectStore::VecChar(ref specials_store)) = self.lookup_value("SPECIALS") {
      for special_char in specials_store {
        all_specials.push(special_char.clone());
      }
    }

    for special_char in all_specials {
      self.assign_catcode(special_char, Catcode::OTHER, Some(Scope::Local));
    }
    // TODO:
    // self.assign_mathcode('\'' => 0x8000, Some(Scope::Local));
    // // try to stay as ASCII as possible
    // self.assign_value("font" => $self->lookupValue('font')->merge(encoding => 'ASCII'), 'local');
  }

  pub fn end_semiverbatim(&mut self) {
    self.pop_frame();
  }

}
