use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use common::font::Font;
use common::model::{IndirectModel, Model};
use common::number::Number;
use definition::conditional::Conditional;
use definition::constructor::Constructor;
use definition::expandable::Expandable;
use definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use definition::primitive::Primitive;
use definition::Definition;
use document::resource::Resource;
use document::tag::{TagData, TagOptions};
use document::Document;
use parameter::Parameter;
use token::{Catcode, Token};
use tokens::Tokens;
use util::pathname;

static CODE_TEX_EXT: &'static str = ".code.tex";

lazy_static! {
  static ref TEX_OR_BIB_EXT_RE: Regex = Regex::new(r"\.(tex|bib)$").unwrap();
}

#[derive(Clone, PartialEq)]
pub enum Scope {
  Global,
  Local,
}

#[derive(Clone)]
pub enum TableName {
  Meaning,
  Value,
  Catcode,
  Mathcode,
  Sfcode,
  Lccode,
  Uccode,
  Delcode,
  Stash,
  StashActive,
}
impl TableName {
  pub fn variants() -> Vec<TableName> {
    use self::TableName::*;
    vec![
      Meaning,
      Value,
      Catcode,
      Sfcode,
      Lccode,
      Uccode,
      Delcode,
      Stash,
      StashActive,
    ]
  }
}

#[derive(Clone, PartialEq)]
pub enum ObjectStore {
  // Primitives
  Bool(bool),
  String(String),
  Mathcode(usize),
  Int(i32),
  // LaTeXML objects
  Catcode(Catcode),
  Token(Token),
  Tokens(Tokens),
  Expandable(Rc<Expandable>),
  Conditional(Rc<Conditional>),
  Primitive(Rc<Primitive>),
  MathPrimitive(Rc<MathPrimitive>),
  // MathPrimitiveOptions(MathPrimitiveOptions), // Maybe later
  Constructor(Rc<Constructor>),
  Digested(Rc<::Digested>),
  Parameter(Parameter),
  Font(Font),
  Number(Number),
  // Collections
  VecChar(Vec<char>),
  VecString(Vec<String>),
  VecToken(Vec<Token>),
  VecDigested(Vec<::Digested>),
  HashStr(HashMap<String, String>),
  VecDequeOS(VecDeque<ObjectStore>),
  HashOS(HashMap<String, ObjectStore>),
  HashTagData(HashMap<String, Vec<TagData>>),
}

impl fmt::Debug for ObjectStore {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use state::ObjectStore::*;
    match *self {
      String(ref s) => write!(f, "{}", s),
      Int(ref num) => write!(f, "{}", num),
      VecChar(ref vs) => write!(f, "{:?}", vs),
      VecString(ref vs) => write!(f, "{:?}", vs),
      Bool(ref b) => write!(f, "{:?}", b),
      Token(ref t) => write!(f, "{:?}", t),
      Tokens(ref t) => write!(f, "{:?}", t),
      Catcode(ref cc) => write!(f, "{:?}", cc),
      Mathcode(ref cc) => write!(f, "{:?}", cc),
      Expandable(ref _expandable) => write!(f, "<closure for expandable definition>"),
      Conditional(ref _conditional) => write!(f, "<closure for conditional definition>"),
      Primitive(ref _primitive) => write!(f, "<closure for primitive definition>"),
      MathPrimitive(ref _primitive) => write!(f, "<closure for math primitive definition>"),
      // MathPrimitiveOptions(ref _primitive) => write!(f, "<math primitive options>"),
      Constructor(ref _constructor) => write!(f, "<closure for constructor definition>"),
      Digested(ref digested) => write!(f, "{:?}", digested),
      Parameter(ref parameter) => write!(f, "{:?}", parameter),
      Font(ref font) => write!(f, "{:?}", font),
      Number(ref number) => write!(f, "{:?}", number),
      VecToken(ref token_vec) => write!(f, "{:?}", token_vec),
      VecDigested(ref digested_vec) => write!(f, "{:?}", digested_vec),
      VecDequeOS(ref vec) => write!(f, "VecDequeOS({:?})", vec),
      HashOS(ref hos) => write!(f, "HashOS({:?})", hos),
      HashTagData(ref htd) => write!(f, "HashTagData({:?})", htd),
      HashStr(ref hstr) => write!(f, "HashStr({:?})", hstr),
    }
  }
}
impl fmt::Display for ObjectStore {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

/// High-level catcode profiles
#[derive(Clone, Debug, PartialEq)]
pub enum Catcodes {
  Standard,
  Style,
  None,
}

/// Ledger for stacked assignments
pub type AssignmentCount = HashMap<String, usize>;
#[derive(Debug, Clone)]
pub struct UndoFrame {
  locked: bool,
  meaning: AssignmentCount,
  value: AssignmentCount,
  catcode: AssignmentCount,
  mathcode: AssignmentCount,
  sfcode: AssignmentCount,
  lccode: AssignmentCount,
  uccode: AssignmentCount,
  delcode: AssignmentCount,
  stash: AssignmentCount,
  stash_active: AssignmentCount,
}

impl Default for UndoFrame {
  fn default() -> Self {
    UndoFrame {
      locked: false,
      meaning: HashMap::new(),
      value: HashMap::new(),
      catcode: HashMap::new(),
      mathcode: HashMap::new(),
      sfcode: HashMap::new(),
      lccode: HashMap::new(),
      uccode: HashMap::new(),
      delcode: HashMap::new(),
      stash: HashMap::new(),
      stash_active: HashMap::new(),
    }
  }
}
impl UndoFrame {
  pub fn table(&self, name: &TableName) -> &AssignmentCount {
    use self::TableName::*;
    match *name {
      Meaning => &self.meaning,
      Value => &self.value,
      Catcode => &self.catcode,
      Mathcode => &self.mathcode,
      Sfcode => &self.sfcode,
      Lccode => &self.lccode,
      Uccode => &self.uccode,
      Delcode => &self.delcode,
      Stash => &self.stash,
      StashActive => &self.stash_active,
    }
  }
  pub fn table_mut(&mut self, name: &TableName) -> &mut AssignmentCount {
    use self::TableName::*;
    match *name {
      Meaning => &mut self.meaning,
      Value => &mut self.value,
      Catcode => &mut self.catcode,
      Mathcode => &mut self.mathcode,
      Sfcode => &mut self.sfcode,
      Lccode => &mut self.lccode,
      Uccode => &mut self.uccode,
      Delcode => &mut self.delcode,
      Stash => &mut self.stash,
      StashActive => &mut self.stash_active,
    }
  }
}

/// The State efficiently maintain the bindings in a TeX-like fashion.
/// bindings associate data with keys (eg definitions with macro names)
/// and respect TeX grouping; that is, an assignment is only in effect
/// until the current group (opened by \bgroup) is closed (by \egroup).
///----------------------------------------------------------------------
/// The objective is to make the following, most-common, operations FAST:
///   begin & end a group (ie. push/pop a stack frame)
///   lookup & assignment of values
/// With the more obvious approach, a "stack of frames", either lookup would involve
/// checking a sequence of frames until the current value is found;
/// or, starting a new frame would involve copying bindings for all values
/// I never quite studied how Knuth does it;
/// The following structures allow these to be constant operations (usually),
/// except for endgroup (which is linear in # of changed values in that frame).
///
/// There are 2 main structures used here.
/// For each of several `Table`s (being "value", "meaning", "catcode" or other space of names),
/// each table maintains the bound values, and "undo" defines the stack frames:
///    self.table[key] = [`current_value`, `previous_value`, ...]
///    self.undo[frame].table[key] = (None | n)
/// such that the "current value" associated with `key` is the 0th element of the table array;
/// the `previous_value`s (if any) are values that had been assigned within previous groups.
/// The undo list indicates how many values have been assigned for `key` in
/// the `frame`'th frame (usually the last is the one of interest).
/// [Would be simpler to store boolean in undo, but see deactivateScope]
/// [An UndoFrame contains fields for each State table, and a lock attribute]
///
/// So, in handwaving form, the algorithms are as follows:
/// push-frame == bgroup == begingroup:
///    push an empty hash {} onto the undo stack;
/// pop-frame == egroup == endgroup:
///   for the `n` associated with every key in the topmost hash in the undo stack
///     pop `n` values from the table
///   then remove the hash from the undo stack.
/// Lookup value:
///   we simply fetch the last element from the table
/// Assign a value:
///   local scope (the normal way):
///     we push a new value into the table described above,
///     and also increment the associated value in the undo stack
///   global scope:
///     remove any locally scoped values, and undo entries for the key
///     then set the only remaining value to the given one.
///   named-scope `scope`:
///      push an entry `[table,key,value]` globally to the `stash` table's value.
///      And assign locally, if the `scope` is active (has non-zero value in `stash_active` table),
///
/// There are tables for
///  catcode: keys are char;
///     Also, `math:char` =1 when `char` is active in math.
///  mathcode, sfcode, lccode, uccode, delcode : are similar to catcode but store
///    additional kinds codes per char (see TeX)
///  value: keys are anything (typically a string, though) and value is the value associated with it
///  meaning: The definition assocated with `key`, usually a control-sequence.
///  stash & stash_active: support named scopes
///      (see also activateScope & deactivateScope)
pub type Table = HashMap<String, VecDeque<ObjectStore>>;
pub struct State {
  /// Tables
  pub value: Table,
  pub meaning: Table,
  pub stash: Table,
  pub stash_active: Table,
  pub catcode: Table,
  pub mathcode: Table,
  pub sfcode: Table,
  pub lccode: Table,
  pub uccode: Table,
  pub delcode: Table,
  /// Table bookkeeping
  pub undo: VecDeque<UndoFrame>,
  /// Stateful runtime - data structures
  pub model: Model,
  pub document: Option<Document>,
  pub prefixes: HashMap<String, bool>, // ?
  pub status: HashMap<String, bool>,   // ?
  pub map: Vec<String>,                // ?
  pub tag_properties: HashMap<String, TagOptions>,
  pub indirect_model: Option<IndirectModel>,
  pub pending_resources: Vec<Resource>,
  /// Stateful runtime - simple fields
  pub verbosity: i32,
  pub status_code: usize,
  pub unlocked: bool,
  pub current_token: Option<Token>,
  pub noexpand_the: bool,
  pub input_encoding: Option<String>,
  pub strict: bool,
  pub include_comments: bool,
  pub documentid: String,
  pub search_paths: VecDeque<String>,
  pub graphics_paths: VecDeque<String>,
  pub include_styles: bool,
  pub nomathparse: bool,
}

impl Default for State {
  fn default() -> Self {
    let top_frame = UndoFrame {
      locked: true,
      ..UndoFrame::default()
    };
    let mut undo_vdq = VecDeque::new();
    undo_vdq.push_front(top_frame);

    State {
      // Tables
      value: HashMap::new(),
      meaning: HashMap::new(),
      stash: HashMap::new(),
      stash_active: HashMap::new(),
      catcode: HashMap::new(),
      mathcode: HashMap::new(),
      sfcode: HashMap::new(),
      lccode: HashMap::new(),
      uccode: HashMap::new(),
      delcode: HashMap::new(),
      // Table bookkeeping
      undo: undo_vdq,
      // Stateful runtime - data structures
      model: Model::default(),
      document: None,
      prefixes: HashMap::new(),
      status: HashMap::new(),
      map: Vec::new(),
      tag_properties: HashMap::new(),
      indirect_model: None,
      pending_resources: Vec::new(),
      // Stateful runtime - simple fields
      verbosity: 0,
      status_code: 0,
      unlocked: true,
      current_token: None,
      noexpand_the: false,
      input_encoding: None,
      strict: false,
      include_comments: true,
      documentid: String::new(),
      search_paths: VecDeque::new(),
      graphics_paths: VecDeque::new(),
      include_styles: false,
      nomathparse: false,
    }
  }
}
/// State fields allowed for customization during construction
pub struct StateOptions {
  pub model: Option<Model>,
  pub verbosity: Option<i32>,
  pub strict: Option<bool>,
  pub include_comments: Option<bool>,
  pub include_styles: Option<bool>,
  pub nomathparse: Option<bool>,
  pub documentid: Option<String>,
  pub search_paths: Option<Vec<String>>,
  pub graphics_paths: Option<Vec<String>>,
  pub catcodes: Option<Catcodes>,
  pub input_encoding: Option<String>,
}
impl Default for StateOptions {
  fn default() -> Self {
    StateOptions {
      model: None,
      verbosity: None,
      strict: None,
      include_comments: None,
      include_styles: None,
      nomathparse: None,
      documentid: None,
      search_paths: None,
      graphics_paths: None,
      catcodes: None,
      input_encoding: None,
    }
  }
}

impl State {
  pub fn new(options: StateOptions) -> Self {
    use token::Catcode::*;

    // Setup default catcodes.
    let catcode_profile = match options.catcodes {
      None => Catcodes::Standard,
      Some(cp) => cp,
    };

    let mut catcodes: HashMap<char, Catcode> = HashMap::new();
    match catcode_profile {
      Catcodes::Standard | Catcodes::Style => {
        catcodes.insert('\\', ESCAPE);
        catcodes.insert('{', BEGIN);
        catcodes.insert('}', END);
        catcodes.insert('$', MATH);
        catcodes.insert('&', ALIGN);
        catcodes.insert('\r', EOL);
        catcodes.insert('#', PARAM);
        catcodes.insert('^', SUPER);
        catcodes.insert('_', SUB);
        catcodes.insert(' ', SPACE);
        catcodes.insert('\t', SPACE);
        catcodes.insert('%', COMMENT);
        catcodes.insert('~', ACTIVE);
        catcodes.insert('\0', IGNORE);
        for c in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
          catcodes.insert(c, LETTER);
        }
      },
      Catcodes::None => {},
    }
    if catcode_profile == Catcodes::Style {
      catcodes.insert('@', LETTER);
    }

    let mut value_table = HashMap::new();
    let mut specials_vdq = VecDeque::new();
    specials_vdq.push_front(ObjectStore::VecChar(vec![
      '^', '_', '@', '~', '&', '$', '#', '%', '\'',
    ]));
    value_table.insert(s!("SPECIALS"), specials_vdq);

    let mut catcodes_typed: Table = HashMap::new();
    for (k, v) in catcodes {
      let mut vdq = VecDeque::new();
      vdq.push_front(ObjectStore::Catcode(v));
      catcodes_typed.insert(k.to_string(), vdq);
    }

    // Basic defaults
    let model = match options.model {
      None => Model::default(),
      Some(m) => m,
    };
    let verbosity = match options.verbosity {
      None => 0,
      Some(v) => v,
    };
    let strict = match options.strict {
      None => false,
      Some(s) => s,
    };
    let include_comments = match options.include_comments {
      None => true,
      Some(ic) => ic,
    };
    let include_styles = match options.include_styles {
      None => false,
      Some(is) => is,
    };
    let nomathparse = match options.nomathparse {
      None => false,
      Some(is) => is,
    };

    let documentid = match options.documentid {
      None => String::new(),
      Some(id) => id,
    };
    let search_paths = match options.search_paths {
      None => VecDeque::new(),
      Some(paths) => paths
        .iter()
        .map(|p| pathname::absolute(&pathname::canonical(p)))
        .collect(),
    };
    let graphics_paths = match options.graphics_paths {
      None => VecDeque::new(),
      Some(paths) => paths
        .iter()
        .map(|p| pathname::absolute(&pathname::canonical(p)))
        .collect(),
    };

    State {
      value: value_table,
      catcode: catcodes_typed,
      model,
      verbosity,
      strict,
      include_comments,
      documentid,
      search_paths,
      graphics_paths,
      include_styles,
      input_encoding: options.input_encoding,
      nomathparse,
      ..State::default()
    }
  }

  pub fn table(&self, name: &TableName) -> &Table {
    use self::TableName::*;
    match *name {
      Meaning => &self.meaning,
      Value => &self.value,
      Catcode => &self.catcode,
      Mathcode => &self.mathcode,
      Sfcode => &self.sfcode,
      Lccode => &self.lccode,
      Uccode => &self.uccode,
      Delcode => &self.delcode,
      Stash => &self.stash,
      StashActive => &self.stash_active,
    }
  }
  pub fn table_mut(&mut self, name: &TableName) -> &mut Table {
    use self::TableName::*;
    match *name {
      Meaning => &mut self.meaning,
      Value => &mut self.value,
      Catcode => &mut self.catcode,
      Mathcode => &mut self.mathcode,
      Sfcode => &mut self.sfcode,
      Lccode => &mut self.lccode,
      Uccode => &mut self.uccode,
      Delcode => &mut self.delcode,
      Stash => &mut self.stash,
      StashActive => &mut self.stash_active,
    }
  }

  pub fn assign_internal(
    &mut self,
    table_name: TableName,
    key: &str,
    value: ObjectStore,
    scope_opt: Option<Scope>,
  )
  {
    let scope = match scope_opt {
      Some(s) => s,
      None => if let Some(&true) = self.prefixes.get("global") {
        Scope::Global
      } else {
        Scope::Local
      },
    };

    if scope == Scope::Global {
      // We are going to change the model, where we first count the total number of definitions to
      // pop, and then pop them, in order to never mutably morrow more than once at a time.
      let mut undo_count = 0;
      {
        // Remove bindings made in all frames down-to & including the next lower locked frame
        let mut frame_table: &mut AssignmentCount = &mut HashMap::new();

        for frame in &mut self.undo {
          let is_locked = frame.locked;
          frame_table = frame.table_mut(&table_name);
          if let Some(n) = frame_table.remove(key) {
            undo_count += n;
          }

          if is_locked {
            break;
          }
        }
        // whatever is left -- if anything -- should be bindings below the locked frame.
        frame_table.insert(key.to_string(), 1); // Note that there's only one value in the stack, now
      }
      {
        // Undo the bindings, if `key` was bound in this frame
        let state_table = self.table_mut(&table_name);
        if let Some(defs) = state_table.get_mut(key) {
          for _ in 1..undo_count + 1 {
            defs.pop_front();
          }
        }

        let table_entry = state_table
          .entry(key.to_string())
          .or_insert_with(VecDeque::new);
        table_entry.push_front(value);
      }
    } else if scope == Scope::Local {
      // Again, split the logic as 1) bookkeeping in undo, then 2) operations in state tables
      let mut is_replace = false;
      {
        // 1. Undo mutable logic
        if let Some(current_frame) = self.undo.front_mut() {
          let current_frame_table = current_frame.table_mut(&table_name);

          // If the value was previously assigned in this frame
          if current_frame_table.get(key).is_some() {
            is_replace = true;
          } else {
            // Otherwise, push new value & set 1 to be undone
            current_frame_table.insert(key.to_string(), 1);
            //  And push new binding.
            is_replace = false;
          }
        }
      }
      {
        // 2. State table mutable logic
        let state_table = self.table_mut(&table_name);
        let defs = state_table
          .entry(key.to_string())
          .or_insert_with(VecDeque::new);
        if is_replace {
          // Replace the value
          defs.pop_front();
        }
        defs.push_front(value)
      }
    }
    // TODO: stash cases
  }

  //======================================================================

  pub fn lookup_value<'lv>(&'lv self, key: &'lv str) -> Option<&ObjectStore> {
    match self.value.get(key) {
      None => None,
      Some(vvec) => vvec.front(),
    }
  }

  pub fn lookup_value_mut<'lv>(&'lv mut self, key: &'lv str) -> Option<&mut ObjectStore> {
    match self.value.get_mut(key) {
      None => None,
      Some(vvec) => vvec.front_mut(),
    }
  }

  pub fn remove_value<'lv>(&'lv mut self, key: &'lv str) -> Option<ObjectStore> {
    match self.value.get_mut(key) {
      None => None,
      Some(vvec) => vvec.pop_front(),
    }
  }

  pub fn assign_value<'av>(&'av mut self, key: &'av str, value: ObjectStore, scope: Option<Scope>) {
    self.assign_internal(TableName::Value, key, value, scope);
  }

  // manage a (global) list of values
  pub fn push_value(&mut self, key: &str, value: ObjectStore) {
    if self.value.get(key).is_none() {
      self.assign_internal(
        TableName::Value,
        key,
        ObjectStore::VecDequeOS(VecDeque::new()),
        Some(Scope::Global),
      );
    }
    if let Some(&mut ObjectStore::VecDequeOS(ref mut front)) =
      self.value.get_mut(key).unwrap().front_mut()
    {
      front.push_back(value);
    } else {
      error!(target: "state:objectstore", "BUG: Tried to push_value into a non-vecdeque value key!");
    }
  }

  pub fn pop_value(&mut self, key: &str) -> Option<ObjectStore> {
    if self.value.get(key).is_none() {
      self.assign_internal(
        TableName::Value,
        key,
        ObjectStore::VecDequeOS(VecDeque::new()),
        Some(Scope::Global),
      );
    }
    if let Some(&mut ObjectStore::VecDequeOS(ref mut front)) =
      self.value.get_mut(key).unwrap().front_mut()
    {
      front.pop_back()
    } else {
      error!(target: "state:objectstore", "BUG: Tried to pop_value from a non-vecdeque value key!");
      None
    }
  }

  /// A bit of Perl "existence as truth" semantics mixed in with proper boolean lookup
  pub fn lookup_bool(&self, key: &str) -> bool {
    match self.lookup_value(key) {
      Some(&ObjectStore::Bool(ref v)) => *v,
      Some(_) => true,
      None => false,
    }
  }

  pub fn lookup_string(&self, key: &str) -> String {
    match self.lookup_value(key) {
      Some(&ObjectStore::String(ref v)) => v.to_owned(),
      _ => String::new(),
    }
  }

  pub fn lookup_vecdeque<'lvdq>(&'lvdq self, key: &'lvdq str) -> Option<&VecDeque<ObjectStore>> {
    match self.lookup_value(key) {
      Some(&ObjectStore::VecDequeOS(ref v)) => Some(v),
      _ => None,
    }
  }

  pub fn lookup_font<'font>(&'font self) -> Option<Font> {
    match self.lookup_value("font") {
      Some(&ObjectStore::Font(ref f)) => Some(f.clone()), /* TODO: is this clone heavy/slow?
                                                             * We can refactor into refs */
      _ => None,
    }
  }
  pub fn lookup_mathfont<'font>(&'font self) -> Option<Font> {
    match self.lookup_value("mathfont") {
      Some(&ObjectStore::Font(ref f)) => Some(f.clone()), /* TODO: is this clone heavy/slow?
                                                             * We can refactor into refs */
      _ => None,
    }
  }

  pub fn lookup_number(&self, key: &str) -> Option<Number> {
    match self.lookup_value(key) {
      Some(&ObjectStore::Number(ref n)) => Some(n.clone()), /* TODO: is this clone heavy/slow?
                                                             * We can refactor into refs */
      _ => None,
    }
  }

  pub fn lookup_tokens(&self, key: &str) -> Option<Tokens> {
    match self.lookup_value(key) {
      Some(&ObjectStore::Tokens(ref ts)) => Some(ts.clone()), /* TODO: is this clone heavy/slow?
                                                             * We can refactor into refs */
      _ => None,
    }
  }

  pub fn unshift_value(&mut self, key: &str, values: Vec<ObjectStore>) {
    if self.value.get(key).is_none() {
      self.assign_internal(
        TableName::Value,
        key,
        ObjectStore::VecDequeOS(VecDeque::new()),
        Some(Scope::Global),
      )
    }
    if let Some(&mut ObjectStore::VecDequeOS(ref mut front)) =
      self.value.get_mut(key).unwrap().front_mut()
    {
      for value in values.into_iter().rev() {
        // preserving order unshift, as Perl's
        front.push_front(value)
      }
    }
  }

  pub fn shift_value(&mut self, key: &str) -> Option<ObjectStore> {
    if self.value.get(key).is_none() {
      self.assign_internal(
        TableName::Value,
        key,
        ObjectStore::VecDequeOS(VecDeque::new()),
        Some(Scope::Global),
      )
    }
    if let Some(&mut ObjectStore::VecDequeOS(ref mut front)) =
      self.value.get_mut(key).unwrap().front_mut()
    {
      front.pop_front()
    } else {
      error!(target: "state:objectstore", "BUG: Tried to shift_value from a non-vecdeque value key!");
      None
    }
  }

  /// manage a (global) hash of values
  pub fn lookup_mapping(&self, map: &str, key: &str) -> Option<&ObjectStore> {
    match self.value.get(map) {
      None => None,
      Some(map_vec) => match map_vec.front() {
        Some(&ObjectStore::HashOS(ref h)) => h.get(key),
        _ => None,
      },
    }
  }

  pub fn assign_mapping(&mut self, map: &str, key: &str, value: Option<ObjectStore>) {
    if self.value.get(map).is_none() || self.value[map].is_empty() {
      self.assign_internal(
        TableName::Value,
        map,
        ObjectStore::HashOS(HashMap::new()),
        Some(Scope::Global),
      );
    }
    let map_store = self.value.get_mut(map).unwrap();
    let mut stub_hash = HashMap::new(); // TODO: What is the right abstraction here? this is hacky
    let mapping = match *map_store.front_mut().unwrap() {
      ObjectStore::HashOS(ref mut mapping) => mapping,
      _ => &mut stub_hash,
    };

    match value {
      None => mapping.remove(key),
      Some(v) => mapping.insert(key.to_string(), v),
    };
  }

  // sub lookupMappingKeys {
  //   my ($self, $map) = @_;
  //   my $vtable  = $$self{value};
  //   my $mapping = $$vtable{$map}[0];
  //   return ($mapping ? sort keys %$mapping : ()); }

  // sub lookupStackedValues {
  //   my ($self, $key) = @_;
  //   my $stack = $$self{value}{$key};
  //   return ($stack ? @$stack : ()); }

  //======================================================================
  /// Was `name` bound?  If  `frame` is given, check only whether it is bound in
  /// that frame (0 is the topmost).
  pub fn is_value_bound(&self, key: &str, frame_opt: Option<usize>) -> bool {
    match frame_opt {
      Some(frame) => self
        .undo
        .get(frame)
        .as_ref()
        .unwrap()
        .table(&TableName::Value)
        .get(key)
        .is_some(),
      None => self
        .value
        .get(key)
        .unwrap_or(&VecDeque::new())
        .front()
        .is_some(),
    }
  }

  pub fn value_in_frame(&self, key: &str, frame_opt: Option<usize>) -> Option<&ObjectStore> {
    let frame = match frame_opt {
      None => 0,
      Some(n) => n,
    };
    let mut p = 0;
    for f in 0..frame + 1 {
      let val_opt = self
        .undo
        .get(f)
        .as_ref()
        .unwrap()
        .table(&TableName::Value)
        .get(key);
      let value = match val_opt {
        Some(v) => *v,
        _ => 0,
      };
      p += value;
    }
    self.value[key].get(p)
  }

  //======================================================================
  /// Lookup & assign a character's Catcode
  pub fn lookup_catcode(&self, c: &char) -> Option<Catcode> {
    match self.catcode.get(&c.to_string()) {
      None => None,
      Some(cvec) => match cvec.front() {
        Some(&ObjectStore::Catcode(ref cc)) => Some(*cc),
        _ => None,
      },
    }
  }

  pub fn assign_catcode(&mut self, key: char, value: Catcode, scope: Option<Scope>) {
    self.assign_internal(
      TableName::Catcode,
      &key.to_string(),
      ObjectStore::Catcode(value),
      scope,
    );
  }

  pub fn lookup_mathcode(&mut self, key: &str) -> Option<usize> {
    match self.mathcode.get(&key.to_string()) {
      Some(c) => match c.front() {
        Some(&ObjectStore::Mathcode(ref codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }

  pub fn assign_mathcode(&mut self, key: char, value: usize, scope: Option<Scope>) {
    self.assign_internal(
      TableName::Mathcode,
      &key.to_string(),
      ObjectStore::Mathcode(value),
      scope,
    );
  }

  /// Get the `Meaning' of a token.  For active control sequence's
  /// this may give the definition object (if defined) or another token (if \let) or undef
  /// Any other token is returned as is.
  pub fn lookup_meaning<'t, 'm>(&'m mut self, token: &'t Token) -> Option<&ObjectStore> {
    if token.code.is_active_or_cs() && !token.text.is_empty() {

    } else {
      let mut token_defs = VecDeque::new();
      token_defs.push_front(ObjectStore::Token(token.clone()));
      self.meaning.insert(token.text.clone(), token_defs);
    }
    match self.meaning.get(&token.text) {
      Some(m) => m.front(),
      None => None,
    }
  }

  /// $meaning should be a definition (for defining active control sequences)
  /// or another token, for \let
  pub fn assign_meaning(&mut self, token: &Token, meaning: ObjectStore, scope: Option<Scope>) {
    self.assign_internal(TableName::Meaning, &token.get_cs_name(), meaning, scope);
  }

  /// used for expansion & various queries
  /// Since we're not doing digestion here, we don't need to handle mathactive,
  /// nor cs let to executable tokens
  /// This returns a definition object, or undef
  pub fn lookup_definition<'def>(&'def self, key: &'def Token) -> Option<ObjectStore> {
    let cc = &key.code;
    let name = &key.text;
    let lookupname: String = if (cc == &Catcode::ACTIVE) || (cc == &Catcode::CS) {
      name.clone()
    } else {
      cc.name()
    };

    if lookupname.is_empty() {
      None
    } else {
      match self.meaning.get(&lookupname) {
        Some(defs) => match defs.front() {
          Some(entry) => Some(entry.clone()),
          None => None,
        },
        _ => None,
      }
    }
  }

  pub fn lookup_digestable_definition<'def>(
    &'def mut self,
    token: &'def Token,
  ) -> Option<ObjectStore>
  {
    let cc = &token.code;
    let name = &token.text;
    if name.is_empty() {
      return None;
    }
    let lookupname = if (cc == &Catcode::ACTIVE) || (cc == &Catcode::CS)
      || ((cc == &Catcode::LETTER)
        || (cc == &Catcode::OTHER)
          && self.lookup_bool("IN_MATH")
          && ((self.lookup_mathcode(name).unwrap_or(0)) == 0x8000))
    {
      name.clone()
    } else {
      cc.name()
    };

    debug!("Looking up digestable {:?}", lookupname);
    let entry = self.meaning.get(&lookupname);

    if !lookupname.is_empty() && entry.is_some() {
      debug!("Found definition for: {:?}", lookupname);
      let defn = entry.unwrap();
      // If a cs has been let to an executable token, lookup ITS defn.
      // if defn->isa('LaTeXML::Token')
      // && ($lookupname = $LaTeXML::Token::PRIMITIVE_NAME[$$defn[1]])
      // && ($entry      = $$self{meaning}{$lookupname})) {
      // $defn = $$entry[0]; }
      Some(defn.front().unwrap().clone())
    } else {
      Some(ObjectStore::Token(token.clone()))
    }
  }

  pub fn assign_definition<'def, T: Definition + Hash>(
    &'def mut self,
    _key: &'def Token,
    _definition: T,
  )
  {
  }

  /// And a shorthand for installing definitions
  pub fn install_definition(&mut self, definition: ObjectStore, scope: Option<Scope>) {
    // Locked definitions!!! (or should this test be in assignMeaning?)
    // Ignore attempts to (re)define $cs from tex sources
    //  my $cs = $definition->getCS->getCSName;
    let token = match definition {
      ObjectStore::Expandable(ref defn) => defn.get_cs(),
      ObjectStore::Conditional(ref defn) => defn.get_cs(),
      ObjectStore::Constructor(ref defn) => defn.get_cs(),
      ObjectStore::Primitive(ref defn) => defn.get_cs(),
      ObjectStore::MathPrimitive(ref defn) => defn.get_cs(),
      ObjectStore::Token(ref token) => token.clone(),
      _ => T_LETTER!(s!("_wrong_argument_for_install_definition")),
    };
    let cs = token.get_cs_name();
    // info!("-- installing definition for: {:?}", token);

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
      if let Some(&ObjectStore::String(ref s)) = self.lookup_value("SOURCEFILE") {
        // report if the redefinition seems to come from document source
        if ((s == "Anonymous String") || TEX_OR_BIB_EXT_RE.is_match(s))
          && (!s.ends_with(CODE_TEX_EXT))
        {
          //  info("ignore", cs, self.get_stomach(), "Ignoring redefinition of $cs");
        }
        return;
      }
    }
    self.assign_internal(TableName::Meaning, &cs, definition, scope);
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
    // Easy: just push a new undo frame.
    self.undo.push_front(UndoFrame::default());
  }

  pub fn pop_frame(&mut self) {
    if self.undo.front().as_ref().unwrap().locked {
      panic!("Fatal:unexpected:<endgroup> attempt to pop last locked stack frame");
    // Fatal('unexpected', '<endgroup>', $self->getStomach,
    // "Attempt to pop last locked stack frame"); }
    } else {
      let popped_frame = self.undo.pop_front().unwrap();
      for table_name in &TableName::variants() {
        let undo_table = popped_frame.table(table_name);
        for (key, undo_count) in undo_table.iter() {
          // Typically only 1 value to shift off the table, unless scopes have been activated.
          let name_table = self.table_mut(table_name).get_mut(key).unwrap();
          for _ in 1..*undo_count + 1 {
            name_table.pop_front();
          }
        }
      }
    }
  }

  pub fn begin_semiverbatim(&mut self, extraspecials: Option<Vec<Token>>) {
    // Is this a good/safe enough shorthand, or should we really be doing beginMode?
    self.push_frame();
    self.assign_value("MODE", ObjectStore::String(s!("text")), None);
    self.assign_value("IN_MATH", ObjectStore::Bool(false), None);
    let mut all_specials: Vec<char> = Vec::new();
    if let Some(extra) = extraspecials {
      for special in extra {
        let special_char = special.text.chars().next().unwrap();
        all_specials.push(special_char);
      }
    }
    if let Some(&ObjectStore::VecChar(ref specials_store)) = self.lookup_value("SPECIALS") {
      for special_char in specials_store {
        all_specials.push(*special_char);
      }
    }

    for special_char in all_specials {
      self.assign_catcode(special_char, Catcode::OTHER, Some(Scope::Local));
    }
    // TODO:
    // self.assign_mathcode('\'' => 0x8000, Some(Scope::Local));
    // try to stay as ASCII as possible
    let new_font = if let Some(&ObjectStore::Font(ref current_font)) = self.lookup_value("font") {
      Some(current_font.merge(Font {
        encoding: Some(s!("ASCII")),
        ..Font::default()
      }))
    } else {
      None
    };
    if let Some(local_font) = new_font {
      self.assign_value("font", ObjectStore::Font(local_font), Some(Scope::Local));
    }
  }

  pub fn end_semiverbatim(&mut self) { self.pop_frame(); }

  //   #======================================================================

  // sub pushDaemonFrame {
  //   my ($self) = @_;
  //   my $frame = {};
  //   unshift(@{ $$self{undo} }, $frame);
  //   # Push copys of data for any data that is mutable;
  //   # Only the value & stash tables need to be to be checked.
  //   # NOTE ??? No...
  //   foreach my $table (qw(value stash)) {
  //     if (my $hash = $$self{$table}) {
  //       foreach my $key (keys %$hash) {
  //         my $value = $$hash{$key}[0];
  //         my $type  = ref $value;
  // if (($type eq 'HASH') || ($type eq 'ARRAY')) {    # Only concerned with mutable perl
  // data?                                                           # Local assignment
  //           $$frame{$table}{$key} = 1;                      # Note new value in this frame.
  //           unshift(@{ $$hash{$key} }, daemon_copy($value)); } } } }    # And push new binding.
  //       # Record the contents of LaTeXML::Package::Pool as preloaded
  //   my $pool_preloaded_hash = { map { $_ => 1 } keys %LaTeXML::Package::Pool:: };
  //   $self->assignValue('_PRELOADED_POOL_', $pool_preloaded_hash, 'global');
  //   # Now mark the top frame as LOCKED!!!
  //   $$frame{_FRAME_LOCK_} = 1;
  //   return; }

  // sub daemon_copy {
  //   my ($ob) = @_;
  //   if (ref $ob eq 'HASH') {
  //     my %hash = map { ($_ => daemon_copy($$ob{$_})) } keys %$ob;
  //     return \%hash; }
  //   elsif (ref $ob eq 'ARRAY') {
  //     return [map { daemon_copy($_) } @$ob]; }
  //   else {
  //     return $ob; } }

  // sub popDaemonFrame {
  //   my ($self) = @_;
  //   while (!$$self{undo}[0]{_FRAME_LOCK_}) {
  //     $self->popFrame; }
  //   if (scalar(@{ $$self{undo} } > 1)) {
  //     delete $$self{undo}[0]{_FRAME_LOCK_};
  //     # Any non-preloaded Pool routines should be wiped away, as we
  //     # might want to reuse the Pool namespaces for the next run.
  //     my $pool_preloaded_hash = $self->lookupValue('_PRELOADED_POOL_');
  //     $self->assignValue('_PRELOADED_POOL_', undef, 'global');
  //     foreach my $subname (keys %LaTeXML::Package::Pool::) {
  //       unless (exists $$pool_preloaded_hash{$subname}) {
  //         undef $LaTeXML::Package::Pool::{$subname};
  //         delete $LaTeXML::Package::Pool::{$subname};
  //       } }
  //     # Finally, pop the frame
  //     $self->popFrame; }
  //   else {
  //     Fatal('unexpected', '<endgroup>', $self->getStomach,
  //       "Daemon Attempt to pop last stack frame"); }
  //   return; }

  // ======================================================================
  // Set one of the definition prefixes global, etc (only global matters!)
  pub fn set_prefix(&mut self, prefix: String) { self.prefixes.insert(prefix, true); }

  pub fn get_prefix(&self, prefix: &str) -> bool {
    match self.prefixes.get(prefix) {
      Some(b) => *b,
      _ => false,
    }
  }

  pub fn clear_prefixes(&mut self) { self.prefixes = HashMap::new(); }

  // #======================================================================

  pub fn activate_scope(&mut self, scope: &str) {}
  //   if (!$$self{stash_active}{$scope}[0]) {
  //     assign_internal($self, 'stash_active', $scope, 1, 'local');
  //     if (defined(my $defns = $$self{stash}{$scope}[0])) {
  //       # Now make local assignments for all those in the stash.
  //       my $frame = $$self{undo}[0];
  //       foreach my $entry (@$defns) {
  //         # Here we ALWAYS push the stashed values into the table
  //         # since they may be popped off by deactivateScope
  //         my ($table, $key, $value) = @$entry;
  //         $$frame{$table}{$key}++;    # Note that this many values must be undone
  //         unshift(@{ $$self{$table}{$key} }, $value); } } }    # And push new binding.
  //   return; }

  // # Probably, in most cases, the assignments made by activateScope
  // # will be undone by egroup or popping frames.
  // # But they can also be undone explicitly

  pub fn deactivate_scope(&mut self, scope: &str) {}
  //   my ($self, $scope) = @_;
  //   if ($$self{stash_active}{$scope}[0]) {
  //     assign_internal($self, 'stash_active', $scope, 0, 'global');
  //     if (defined(my $defns = $$self{stash}{$scope}[0])) {
  //       my $frame = $$self{undo}[0];
  //       foreach my $entry (@$defns) {
  //         my ($table, $key, $value) = @$entry;
  //         if ($$self{$table}{$key}[0] eq $value) {
  //           # Here we're popping off the values pushed by activateScope
  //           # to (possibly) reveal a local assignment in the same frame, preceding activateScope.
  //           shift(@{ $$self{$table}{$key} });
  //           $$frame{$table}{$key}--; }
  //         else {
  //           Warn('internal', $key, $self->getStomach,
  //             "Unassigning wrong value for $key from table $table in deactivateScope",
  //             "value is $value but stack is " . join(', ', @{ $$self{$table}{$key} })); } } } }
  //   return; }
  pub fn deactivate_counter_scope(&mut self, scope: &str) {} // ???

  // sub getKnownScopes {
  //   my ($self) = @_;
  //   my @scopes = sort keys %{ $$self{stash} };
  //   return @scopes; }

  // sub getActiveScopes {
  //   my ($self) = @_;
  //   my @scopes = sort keys %{ $$self{stash_active} };
  //   return @scopes; }

  // #======================================================================
  // # Units.
  // #   Put here since it could concievably evolve to depend on the current font.

  // # Conversion to scaled points
  // my %UNITS = (    # [CONSTANT]
  //   pt => 65536, pc => 12 * 65536, in => 72.27 * 65536, bp => 72.27 * 65536 / 72,
  //   cm => 72.27 * 65536 / 2.54, mm => 72.27 * 65536 / 2.54 / 10, dd => 1238 * 65536 / 1157,
  //   cc => 12 * 1238 * 65536 / 1157, sp => 1);

  // sub convertUnit {
  //   my ($self, $unit) = @_;
  //   $unit = lc($unit);
  //   # Eventually try to track font size?
  //   if    ($unit eq 'em') { return 10.0 * 65536; }
  //   elsif ($unit eq 'ex') { return 4.3 * 65536; }
  //   elsif ($unit eq 'mu') { return 10.0 * 65536 / 18; }
  //   else {
  //     my $sp = $UNITS{$unit};
  //     if (!$sp) {
  //       Warn('expected', '<unit>', undef, "Illegal unit of measure '$unit', assuming pt.");
  //       $sp = $UNITS{'pt'}; }
  //     return $sp; } }

  // #======================================================================

  // sub noteStatus {
  //   my ($self, $type, @data) = @_;
  //   if ($type eq 'undefined') {
  //     map { $$self{status}{undefined}{$_}++ } @data; }
  //   elsif ($type eq 'missing') {
  //     map { $$self{status}{missing}{$_}++ } @data; }
  //   else {
  //     $$self{status}{$type}++; }
  //   return; }

  // sub getStatus {
  //   my ($self, $type) = @_;
  //   return $$self{status}{$type}; }

  // sub getStatusMessage {
  //   my ($self) = @_;
  //   my $status = $$self{status};
  //   my @report = ();
  // push(@report, colorizeString("$$status{warning} warning" . ($$status{warning} > 1 ? 's' :
  // ''), 'warning'))     if $$status{warning};
  // push(@report, colorizeString("$$status{error} error" . ($$status{error} > 1 ? 's' : ''),
  // 'error'))     if $$status{error};
  //   push(@report, "$$status{fatal} fatal error" . ($$status{fatal} > 1 ? 's' : ''))

  //     if $$status{fatal};
  //   my @undef = ($$status{undefined} ? keys %{ $$status{undefined} } : ());
  //   push(@report, colorizeString(scalar(@undef) . " undefined macro" . (@undef > 1 ? 's' : '')
  //         . "[" . join(', ', @undef) . "]", 'details'))
  //     if @undef;
  //   my @miss = ($$status{missing} ? keys %{ $$status{missing} } : ());
  //   push(@report, colorizeString(scalar(@miss) . " missing file" . (@miss > 1 ? 's' : '')
  //         . "[" . join(', ', @miss) . "]", 'details'))
  //     if @miss;
  //   return join('; ', @report) || colorizeString('No obvious problems', 'success'); }

  // sub getStatusCode {
  //   my ($self) = @_;
  //   my $status = $$self{status};
  //   my $code;
  //   if ($$status{fatal} && $$status{fatal} > 0) {
  //     $code = 3; }
  //   elsif ($$status{error} && $$status{error} > 0) {
  //     $code = 2; }
  //   elsif ($$status{warning} && $$status{warning} > 0) {
  //     $code = 1; }
  //   else {
  //     $code = 0; }
  //   return $code; }
  // #======================================================================

  /// The indirect model includes all elements allowed as direct children,
  /// and all descendents of a node that can be inserted after autoOpen'ing intermediate elements.
  /// This model therefor includes information from the Schema, as well as
  /// autoOpen information that may be introduced in binding files.
  /// [Thus it should NOT be modifying the Model object, which may cover several documents in `]
  /// $imodel{$tag}{$child} => $open means if in $tag, to open $child, we must first open $open
  pub fn compute_indirect_model(&mut self) -> IndirectModel {
    let mut imodel: IndirectModel = HashMap::new();
    // Determine any indirect paths to each descendent via an `autoOpen-able' tag.
    let mut openable: HashSet<String> = HashSet::new();
    for tag in self.model.get_tags() {
      if let Some(x) = self.tag_properties.get(&tag) {
        if let Some(true) = x.auto_open {
          openable.insert(tag.to_owned());
        }
      }
    }

    for tag in self.model.get_tags() {
      let mut desc: HashMap<String, HashMap<String, usize>> = HashMap::new();
      {
        self.compute_indirect_model_aux(&tag, None, 1, &mut openable, &mut desc);
      }

      let mut desc_keys: Vec<String> = desc.keys().map(|k| k.to_string()).collect();
      desc_keys.sort();
      for kid in desc_keys {
        let mut best = 0; // Find best path to $kid.
        let mut desc_kid_keys: Vec<String> = desc
          .entry(kid.to_owned())
          .or_insert_with(HashMap::new)
          .keys()
          .map(|k| k.to_string())
          .collect();
        desc_kid_keys.sort();
        for start in desc_kid_keys {
          let start_entry = {
            let kid_entry = desc.entry(kid.to_owned()).or_insert_with(HashMap::new);
            *kid_entry.entry(start.to_owned()).or_insert(0)
          };
          if start_entry > best {
            imodel
              .entry(tag.to_owned())
              .or_insert_with(HashMap::new)
              .insert(kid.to_owned(), start.to_owned());
            {
              best = desc[&kid][&start];
            }
          }
        }
      }
    }
    // PATCHUP
    if self.model.permissive {
      // !!! Alarm!!!
      imodel
        .entry(s!("#Document"))
        .or_insert_with(HashMap::new)
        .insert(s!("#PCDATA"), s!("ltx:p"));
    }

    imodel
  }

  fn compute_indirect_model_aux(
    &mut self,
    tag: &str,
    start_opt: Option<String>,
    desirability: usize,
    openable: &mut HashSet<String>,
    desc: &mut HashMap<String, HashMap<String, usize>>,
  )
  {
    let start = match start_opt {
      Some(s) => s,
      None => String::new(),
    };

    // A bit tricky here, we need to release the state.model borrow immediately, which is why we
    // move ownership of the tag strings into the tag_contents vector.
    // That leads to a bunch of .clone()s later one, but stays close to the original algorithm
    let tag_contents: Vec<String> = self
      .model
      .get_tag_contents(tag)
      .iter()
      .map(|t| t.to_string())
      .collect();

    for kid in tag_contents {
      if desc
        .entry(kid.clone())
        .or_insert_with(HashMap::new)
        .get(&start)
        .is_some()
      {
        continue;
      } // Already solved

      if !start.is_empty() {
        desc
          .entry(kid.clone())
          .or_insert_with(HashMap::new)
          .insert(start.clone(), desirability);
      }

      if kid != "#PCDATA" && openable.contains(&kid) {
        let inner = if !start.is_empty() {
          start.clone()
        } else {
          kid.to_string()
        };

        self.compute_indirect_model_aux(&kid, Some(inner), desirability, openable, desc);
      }
    }
  }

  /// Initialize various stomach parameters, preload, etc.
  pub fn initialize_stomach(&mut self) {
    self.assign_value("MODE", ObjectStore::String(s!("text")), Some(Scope::Global));
    self.assign_value("IN_MATH", ObjectStore::Bool(false), Some(Scope::Global));
    self.assign_value(
      "PRESERVE_NEWLINES",
      ObjectStore::Bool(true),
      Some(Scope::Global),
    );
    self.assign_value(
      "afterGroup",
      ObjectStore::VecDigested(Vec::new()),
      Some(Scope::Global),
    );
    self.assign_value(
      "afterAssignment",
      ObjectStore::VecDigested(Vec::new()),
      Some(Scope::Global),
    ); // undef ???
    self.assign_value(
      "groupInitiator",
      ObjectStore::String(s!("Initialization")),
      Some(Scope::Global),
    );
    // Setup default fonts.
    self.assign_value(
      "font",
      ObjectStore::Font(Font::text_default()),
      Some(Scope::Global),
    );
    self.assign_value(
      "mathfont",
      ObjectStore::Font(Font::math_default()),
      Some(Scope::Global),
    );
  }

  // Package helpers used in core need to be localized here -- as State methods
  /// `Let` macro setter
  pub fn let_i(&mut self, token1: &Token, token2: Token, scope: Option<Scope>) {
    // If strings are given, assume CS tokens (most common case)
    let meaning = match self.lookup_meaning(&token2) {
      Some(m) => m.clone(),
      None => ObjectStore::Token(token2),
    };
    self.assign_meaning(token1, meaning, scope);
    // TODO: AfterAssignment!();
  }
  /// `XEquals` check for two token arguments
  pub fn x_equals(&mut self, token1: &Token, token2: &Token) -> bool {
    let def1_opt: Option<ObjectStore>;
    let def2_opt: Option<ObjectStore>;
    {
      // mutability guard
      def1_opt = match self.lookup_meaning(token1) {
        // token, definition object or undef
        None => None,
        Some(ref obj) => Some((*obj).clone()), /* TODO: Can this code pattern be reworked
                                                * without a clone? What is the idiomatic Rust
                                                * for this? */
      };
    }
    let def2_opt = self.lookup_meaning(token2); // ditto
    if def1_opt.is_none() && def2_opt.is_none() {
      // true if both undefined
      true
    } else if let Some(def1) = def1_opt {
      if let Some(def2) = def2_opt {
        def1 == *def2 // If both have defns, must be same defn!
      } else {
        // False, if only one has 'meaning'
        false
      }
    } else {
      false // False, if only one has 'meaning'
    }
  }
}
