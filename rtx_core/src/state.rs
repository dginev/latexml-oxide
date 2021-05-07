use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::rc::Rc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::{Font, Fontmap};
use crate::common::glue::Glue;
use crate::common::model::{IndirectModel, Model};
use crate::common::number::Number;
pub use crate::common::store::Stored; // reexport for convenience
use crate::common::BindingDispatcher;
use crate::definition::conditional::{ConditionalType, IfFrame};
use crate::definition::expandable::Expandable;
use crate::definition::register::{RegisterCell, RegisterValue};
use crate::definition::Definition;
use crate::document::resource::Resource;
use crate::document::tag::TagOptions;
use crate::document::Document;

use crate::stomach::Stomach;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::util::pathname;

static CODE_TEX_EXT: &str = ".code.tex";

lazy_static! {
  static ref TEX_OR_BIB_EXT_RE: Regex = Regex::new(r"\.(tex|bib)$").unwrap();
  // Conversion to scaled points
  static ref UNITS: HashMap<String, f32> = map!(
    "pt" => 65536.0, "pc" => 12.0 * 65536.0, "in" => 72.27 * 65536.0, "bp" => 72.27 * 65536.0 / 72.0,
    "cm" => 72.27 * 65536.0 / 2.54, "mm" => 72.27 * 65536.0 / 2.54 / 10.0, "dd" => 1238.0 * 65536.0 / 1157.0,
    "cc" => 12.0 * 1238.0 * 65536.0 / 1157.0, "sp" => 1.0);

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scope {
  Global,
  Local,
}

#[derive(Debug, Copy, Clone)]
pub enum TableName {
  Meaning,
  Value,
  Catcode,
  Mathcode,
  Sfcode,
  Lccode,
  Uccode,
  Delcode,
  StashActive,
}
impl TableName {
  pub fn variants() -> Vec<TableName> {
    use self::TableName::*;
    vec![Meaning, Value, Catcode, Sfcode, Lccode, Uccode, Delcode, StashActive]
  }
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
pub type StashStore = HashMap<String, Vec<(TableName, String, Stored)>>;
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
      stash_active: HashMap::new(),
    }
  }
}
impl UndoFrame {
  pub fn table(&self, name: TableName) -> &AssignmentCount {
    use self::TableName::*;
    match name {
      Meaning => &self.meaning,
      Value => &self.value,
      Catcode => &self.catcode,
      Mathcode => &self.mathcode,
      Sfcode => &self.sfcode,
      Lccode => &self.lccode,
      Uccode => &self.uccode,
      Delcode => &self.delcode,
      StashActive => &self.stash_active,
    }
  }
  pub fn table_mut(&mut self, name: TableName) -> &mut AssignmentCount {
    use self::TableName::*;
    match name {
      Meaning => &mut self.meaning,
      Value => &mut self.value,
      Catcode => &mut self.catcode,
      Mathcode => &mut self.mathcode,
      Sfcode => &mut self.sfcode,
      Lccode => &mut self.lccode,
      Uccode => &mut self.uccode,
      Delcode => &mut self.delcode,
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
pub type Table = HashMap<String, VecDeque<Stored>>;
pub struct State {
  // Tables
  pub value: Table,
  pub meaning: Table,
  pub stash: StashStore,
  pub stash_active: Table,
  pub catcode: Table,
  pub mathcode: Table,
  pub sfcode: Table,
  pub lccode: Table,
  pub uccode: Table,
  pub delcode: Table,
  // Table bookkeeping
  pub undo: VecDeque<UndoFrame>,
  // Stateful runtime - data structures
  pub model: Model,
  pub document: Option<Document>,
  pub prefixes: HashMap<String, bool>,        // ?
  pub status: RefCell<HashMap<String, bool>>, // ?
  pub map: Vec<String>,                       // ?
  pub tag_properties: HashMap<String, TagOptions>,
  pub indirect_model: Option<IndirectModel>,
  pub pending_resources: Vec<Resource>,
  // Stateful runtime - simple fields
  pub verbosity: i32,
  pub status_code: usize,
  pub unlocked: bool,
  pub current_token: Option<Rc<Token>>,
  pub if_frame: Option<Rc<RefCell<IfFrame>>>,
  pub noexpand_the: bool,
  pub input_encoding: Option<String>,
  pub strict: bool,
  pub include_comments: bool,
  pub documentid: String,
  pub search_paths: VecDeque<String>,
  pub graphics_paths: VecDeque<String>,
  pub include_styles: bool,
  pub nomathparse: bool,
  // Auxiliary convenience -- extra dispatch
  // TODO: We can make this a Vec<BindingDispatcher> if we want to accumulate more definitions
  pub extra_bindings_dispatch: Option<BindingDispatcher>,
  // Circular dependency and global $STATE in Perl requires a bad
  // style use of interior mutability...
  pub stomach: Rc<RefCell<Stomach>>,
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
      status: RefCell::new(HashMap::new()),
      map: Vec::new(),
      tag_properties: HashMap::new(),
      indirect_model: None,
      pending_resources: Vec::new(),
      // Stateful runtime - simple fields
      verbosity: 0,
      status_code: 0,
      unlocked: true,
      current_token: None,
      if_frame: None,
      noexpand_the: false,
      input_encoding: None,
      strict: false,
      include_comments: true,
      documentid: String::new(),
      search_paths: VecDeque::new(),
      graphics_paths: VecDeque::new(),
      include_styles: false,
      nomathparse: false,
      extra_bindings_dispatch: None,
      // interiorly mutable
      stomach: Rc::new(RefCell::new(Stomach::default())),
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
    use crate::token::Catcode::*;

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
    specials_vdq.push_front(Stored::VecChar(vec!['^', '_', '@', '~', '&', '$', '#', '\'']));
    value_table.insert(s!("SPECIALS"), specials_vdq);

    let mut catcodes_typed: Table = HashMap::new();
    for (k, v) in catcodes {
      let mut vdq = VecDeque::new();
      vdq.push_front(Stored::Catcode(v));
      catcodes_typed.insert(k.to_string(), vdq);
    }

    // Basic defaults
    let model = match options.model {
      None => Model::default(),
      Some(m) => m,
    };
    let verbosity = options.verbosity.unwrap_or(0);
    let strict = options.strict.unwrap_or(false);
    let include_comments = options.include_comments.unwrap_or(true);
    let include_styles = options.include_styles.unwrap_or(false);
    let nomathparse = options.nomathparse.unwrap_or(false);

    let documentid = match options.documentid {
      None => String::new(),
      Some(id) => id,
    };
    let search_paths = match options.search_paths {
      None => VecDeque::new(),
      Some(paths) => paths.iter().map(|p| pathname::absolute(&pathname::canonical(p))).collect(),
    };
    let graphics_paths = match options.graphics_paths {
      None => VecDeque::new(),
      Some(paths) => paths.iter().map(|p| pathname::absolute(&pathname::canonical(p))).collect(),
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

  pub fn table(&self, name: TableName) -> &Table {
    use self::TableName::*;
    match name {
      Meaning => &self.meaning,
      Value => &self.value,
      Catcode => &self.catcode,
      Mathcode => &self.mathcode,
      Sfcode => &self.sfcode,
      Lccode => &self.lccode,
      Uccode => &self.uccode,
      Delcode => &self.delcode,
      StashActive => &self.stash_active,
    }
  }
  pub fn table_mut(&mut self, name: TableName) -> &mut Table {
    use self::TableName::*;
    match name {
      Meaning => &mut self.meaning,
      Value => &mut self.value,
      Catcode => &mut self.catcode,
      Mathcode => &mut self.mathcode,
      Sfcode => &mut self.sfcode,
      Lccode => &mut self.lccode,
      Uccode => &mut self.uccode,
      Delcode => &mut self.delcode,
      StashActive => &mut self.stash_active,
    }
  }

  pub fn assign_internal(&mut self, table_name: TableName, key: &str, value: Stored, scope_opt: Option<Scope>) {
    let scope = match scope_opt {
      Some(s) => s,
      None => {
        if self.get_prefix("global") {
          Scope::Global
        } else {
          Scope::Local
        }
      },
    };

    if scope == Scope::Global {
      // We are going to change the model, where we first count the total number of definitions to
      // pop, and then pop them, in order to never mutably morrow more than once at a time.
      let mut undo_count = 0;

      // Remove bindings made in all frames down-to & including the next lower locked frame
      let mut frame_table: &mut AssignmentCount = &mut HashMap::new();

      for frame in &mut self.undo {
        let is_locked = frame.locked;
        frame_table = frame.table_mut(table_name);
        if let Some(n) = frame_table.remove(key) {
          undo_count += n;
        }
        if is_locked {
          break;
        }
      }
      // whatever is left -- if anything -- should be bindings below the locked frame.
      frame_table.insert(key.to_string(), 1); // Note that there's only one value in the stack, now

      // Undo the bindings, if `key` was bound in this frame
      let state_table = self.table_mut(table_name);
      if let Some(defs) = state_table.get_mut(key) {
        for _ in 1..=undo_count {
          defs.pop_front();
        }
      }

      let table_entry = state_table.entry(key.to_string()).or_insert_with(VecDeque::new);
      table_entry.push_front(value);
    } else if scope == Scope::Local {
      // Again, split the logic as 1) bookkeeping in undo, then 2) operations in state tables
      let mut is_replace = false;
      // 1. Undo mutable logic
      if let Some(current_frame) = self.undo.front_mut() {
        let current_frame_table = current_frame.table_mut(table_name);
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
      // 2. State table mutable logic
      let state_table = self.table_mut(table_name);
      let defs = state_table.entry(key.to_string()).or_insert_with(VecDeque::new);
      if is_replace {
        // Replace the value
        defs.pop_front();
      }
      defs.push_front(value)
    }
    // TODO: stash cases
  }

  //======================================================================

  pub fn lookup_value(&self, key: &str) -> Option<&Stored> {
    match self.value.get(key) {
      None => None,
      Some(vvec) => vvec.front(),
    }
  }

  pub fn lookup_value_mut<'lv>(&'lv mut self, key: &'lv str) -> Option<&mut Stored> {
    match self.value.get_mut(key) {
      None => None,
      Some(vvec) => vvec.front_mut(),
    }
  }

  pub fn remove_value<'lv>(&'lv mut self, key: &'lv str) -> Option<Stored> {
    match self.value.get_mut(key) {
      None => None,
      Some(vvec) => vvec.pop_front(),
    }
  }

  pub fn assign_value<'av, T: Into<Stored>, S: Into<Option<Scope>>>(&'av mut self, key: &'av str, value: T, scope: S) {
    let value = value.into();
    let scope = scope.into();
    self.assign_internal(TableName::Value, key, value, scope);
  }

  // manage a (global) list of values
  pub fn push_value<T: Into<Stored>>(&mut self, key: &str, value: T) {
    let value = value.into();
    if self.value.get(key).is_none() {
      self.assign_internal(TableName::Value, key, Stored::VecDequeStored(VecDeque::new()), Some(Scope::Global));
    }
    if let Some(&mut Stored::VecDequeStored(ref mut front)) = self.value.get_mut(key).unwrap().front_mut() {
      front.push_back(value);
    } else {
      Error!("state", "Stored", None, self, "BUG: Tried to push_value into a non-vecdeque value key!");
    }
  }

  pub fn pop_value(&mut self, key: &str) -> Option<Stored> {
    if self.value.get(key).is_none() {
      self.assign_internal(TableName::Value, key, Stored::VecDequeStored(VecDeque::new()), Some(Scope::Global));
    }
    if let Some(&mut Stored::VecDequeStored(ref mut front)) = self.value.get_mut(key).unwrap().front_mut() {
      front.pop_back()
    } else {
      Error!("state", "Stored", None, self, "BUG: Tried to pop_value from a non-vecdeque value key!");
      None
    }
  }

  /// A bit of Perl "existence as truth" semantics mixed in with proper boolean lookup
  pub fn lookup_bool(&self, key: &str) -> bool {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      None => false,
    }
  }

  pub fn lookup_string(&self, key: &str) -> String {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      None => String::new(),
    }
  }

  pub fn lookup_int(&self, key: &str) -> i32 {
    match self.lookup_value(key) {
      Some(Stored::Int(i)) => *i,
      Some(Stored::Bool(true)) => 1, // this is Perl's boolean -> integer semantics
      _ => 0,
    }
  }

  pub fn lookup_vec_string<'lvec>(&'lvec self, key: &'lvec str) -> Option<&Vec<String>> {
    match self.lookup_value(key) {
      Some(Stored::VecString(v)) => Some(v),
      _ => None,
    }
  }

  pub fn lookup_vecdeque<'lvdq>(&'lvdq self, key: &'lvdq str) -> Option<&VecDeque<Stored>> {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      _ => None,
    }
  }

  pub fn remove_vecdeque<'lvdq>(&'lvdq mut self, key: &'lvdq str) -> Option<VecDeque<Stored>> {
    match self.remove_value(key) {
      Some(Stored::VecDequeStored(v)) => Some(v),
      _ => None,
    }
  }

  pub fn lookup_font(&self) -> Option<Rc<Font>> {
    match self.lookup_value("font") {
      Some(f) => f.into(),
      _ => None,
    }
  }

  pub fn lookup_mathfont(&self) -> Option<Rc<Font>> {
    match self.lookup_value("mathfont") {
      Some(v) => v.into(),
      _ => None,
    }
  }

  pub fn lookup_number(&self, key: &str) -> Option<Number> {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      _ => None,
    }
  }
  pub fn lookup_dimension(&self, key: &str) -> Option<Dimension> {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      _ => None,
    }
  }
  pub fn lookup_glue(&self, key: &str) -> Option<Glue> {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      _ => None,
    }
  }

  pub fn lookup_tokens(&self, key: &str) -> Option<Tokens> {
    match self.lookup_value(key) {
      Some(v) => v.into(),
      _ => None,
    }
  }

  pub fn lookup_register(&self, cs: &str, parameters: Vec<Token>) -> Option<RegisterValue> {
    let cs = T_CS!(cs);
    if let Some(defn) = self.lookup_definition(&cs) {
      if defn.is_register() {
        defn.value_of(parameters, self)
      } else {
        let message = s!("The control sequence {:?} is not a register", cs);
        Warn!("expected", "register", None, self, message);
        None
      }
    } else {
      let message = s!("The control sequence {:?} is not defined", cs);
      Warn!("expected", "register", None, self, message);
      None
    }
  }

  pub fn lookup_expandable(&self, token: &Token, toplevel: bool) -> Option<Rc<dyn Definition>> {
    if let Some(defn) = self.lookup_definition(&token) {
      // Can only be a token or definition; we want defns!
      if (*defn).is_expandable() && (toplevel || !(*defn).is_protected()) {
        // is this the right logic here? don't expand unless digesting?
        Some(defn)
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn lookup_conditional(&self, token: &Token) -> Option<ConditionalType> {
    let lookupname = token.get_executable_name();
    if lookupname.is_empty() {
      None
    } else if let Some(entry) = self.meaning.get(&lookupname) {
      if let Some(Stored::Conditional(defn)) = entry.front() {
        // Can only be a token or definition; we only want defns that have conditional_type
        Some(defn.conditional_type)
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn unshift_value<T: Into<Stored>>(&mut self, key: &str, values: Vec<T>) {
    let values: Vec<Stored> = values.into_iter().map(Into::into).collect();
    if self.value.get(key).is_none() {
      self.assign_internal(TableName::Value, key, Stored::VecDequeStored(VecDeque::new()), Some(Scope::Global))
    }
    if let Some(&mut Stored::VecDequeStored(ref mut front)) = self.value.get_mut(key).unwrap().front_mut() {
      for value in values.into_iter().rev() {
        // preserving order unshift, as Perl's
        front.push_front(value)
      }
    }
  }

  pub fn shift_value(&mut self, key: &str) -> Option<Stored> {
    if self.value.get(key).is_none() {
      self.assign_internal(TableName::Value, key, Stored::VecDequeStored(VecDeque::new()), Some(Scope::Global))
    }
    if let Some(&mut Stored::VecDequeStored(ref mut front)) = self.value.get_mut(key).unwrap().front_mut() {
      front.pop_front()
    } else {
      Error!("state", "Stored", None, self, "BUG: Tried to shift_value from a non-vecdeque value key!");
      None
    }
  }

  /// manage a (global) hash of values
  pub fn lookup_mapping(&self, map: &str, key: &str) -> Option<&Stored> {
    match self.value.get(map) {
      None => None,
      Some(map_vec) => match map_vec.front() {
        Some(&Stored::HashStored(ref h)) => h.get(key),
        _ => None,
      },
    }
  }

  pub fn assign_mapping<T: Into<Stored>>(&mut self, map: &str, key: &str, value: Option<T>) {
    if self.value.get(map).is_none() || self.value[map].is_empty() {
      self.assign_internal(TableName::Value, map, Stored::HashStored(HashMap::new()), Some(Scope::Global));
    }
    let map_store = self.value.get_mut(map).unwrap();
    let mut stub_hash = HashMap::new(); // TODO: What is the right abstraction here? this is hacky
    let mapping = match *map_store.front_mut().unwrap() {
      Stored::HashStored(ref mut mapping) => mapping,
      _ => &mut stub_hash,
    };

    match value {
      None => mapping.remove(key),
      Some(v) => mapping.insert(key.to_string(), v.into()),
    };
  }

  pub fn lookup_mapping_keys(&self, map: &str) -> Vec<&str> {
    match self.value.get(map) {
      None => Vec::new(),
      Some(map_vec) => match map_vec.front() {
        Some(&Stored::HashStored(ref h)) => h.keys().map(String::as_str).collect(),
        _ => Vec::new(),
      },
    }
  }

  pub fn lookup_stacked_values(&self, key: &str) -> Vec<&Stored> {
    if let Some(vdq) = self.value.get(key) {
      vdq.iter().collect::<Vec<&Stored>>()
    } else {
      Vec::new()
    }
  }

  //======================================================================
  /// Was `name` bound?  If  `frame` is given, check only whether it is bound in
  /// that frame (0 is the topmost).
  pub fn is_value_bound(&self, key: &str, frame_opt: Option<usize>) -> bool {
    match frame_opt {
      Some(frame) => self.undo.get(frame).as_ref().unwrap().table(TableName::Value).get(key).is_some(),
      None => self.value.get(key).unwrap_or(&VecDeque::new()).front().is_some(),
    }
  }

  pub fn value_in_frame(&self, key: &str, frame_opt: Option<usize>) -> Option<&Stored> {
    let frame = frame_opt.unwrap_or(0);
    let mut p = 0;
    for f in 0..=frame {
      let val_opt = self.undo.get(f).as_ref().unwrap().table(TableName::Value).get(key);
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
  pub fn lookup_catcode(&self, c: char) -> Option<Catcode> {
    match self.catcode.get(&c.to_string()) {
      None => None,
      Some(cvec) => match cvec.front() {
        Some(cc) => cc.into(),
        _ => None,
      },
    }
  }
  pub fn assign_catcode(&mut self, key: char, value: Catcode, scope: Option<Scope>) {
    self.assign_internal(TableName::Catcode, &key.to_string(), Stored::Catcode(value), scope);
  }

  pub fn lookup_mathcode(&self, key: &str) -> Option<u16> {
    match self.mathcode.get(&key.to_string()) {
      Some(c) => match c.front() {
        Some(&Stored::Charcode(ref codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  pub fn assign_mathcode<T: Into<u16>, C: Into<char>, S: Into<Option<Scope>>>(&mut self, key: C, value: T, scope: S) {
    let key: char = key.into();
    let scope: Option<Scope> = scope.into();
    self.assign_internal(TableName::Mathcode, &key.to_string(), Stored::Charcode(value.into()), scope);
  }

  pub fn lookup_sfcode(&self, key: char) -> Option<u16> {
    match self.sfcode.get(&key.to_string()) {
      Some(c) => match c.front() {
        Some(&Stored::Charcode(ref codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  pub fn assign_sfcode<T: Into<u16>, C: Into<char>, S: Into<Option<Scope>>>(&mut self, key: C, value: T, scope: S) {
    let key: char = key.into();
    let scope: Option<Scope> = scope.into();
    self.assign_internal(TableName::Sfcode, &key.to_string(), Stored::Charcode(value.into()), scope);
  }

  pub fn lookup_lccode(&self, key: char) -> Option<u16> {
    match self.lccode.get(&key.to_string()) {
      Some(c) => match c.front() {
        Some(&Stored::Charcode(ref codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  pub fn assign_lccode<T: Into<u16>, C: Into<char>, S: Into<Option<Scope>>>(&mut self, key: C, value: T, scope: S) {
    let key: char = key.into();
    let scope: Option<Scope> = scope.into();
    self.assign_internal(TableName::Lccode, &key.to_string(), Stored::Charcode(value.into()), scope);
  }

  pub fn lookup_uccode(&self, key: char) -> Option<u16> {
    match self.uccode.get(&key.to_string()) {
      Some(c) => match c.front() {
        Some(&Stored::Charcode(ref codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  pub fn assign_uccode<T: Into<u16>, C: Into<char>, S: Into<Option<Scope>>>(&mut self, key: C, value: T, scope: S) {
    let key: char = key.into();
    let scope: Option<Scope> = scope.into();
    self.assign_internal(TableName::Uccode, &key.to_string(), Stored::Charcode(value.into()), scope);
  }

  pub fn lookup_delcode(&self, key: char) -> Option<u16> {
    match self.delcode.get(&key.to_string()) {
      Some(c) => match c.front() {
        Some(&Stored::Charcode(ref codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  pub fn assign_delcode<T: Into<u16>, C: Into<char>, S: Into<Option<Scope>>>(&mut self, key: C, value: T, scope: S) {
    let key: char = key.into();
    let scope: Option<Scope> = scope.into();
    self.assign_internal(TableName::Delcode, &key.to_string(), Stored::Charcode(value.into()), scope);
  }

  /// Get the `Meaning' of a token.  For active control sequence's
  /// this may give the definition object (if defined) or another token (if \let) or undef
  /// Any other token is returned as is.
  pub fn lookup_meaning(&self, token: &Token) -> Option<Stored> {
    if token.get_catcode().is_active_or_cs() && !token.get_string().is_empty() {
      match self.meaning.get(&token.get_cs_name().to_owned()) {
        Some(entry) => entry.front().cloned(),
        None => None,
      }
    } else {
      Some(token.into())
    }
  }

  /// $meaning should be a definition (for defining active control sequences)
  /// or another token, for \let
  pub fn assign_meaning<T: Into<Stored>>(&mut self, token: &Token, meaning: T, scope: Option<Scope>) {
    let meaning = meaning.into();
    self.assign_internal(TableName::Meaning, &token.get_cs_name(), meaning, scope);
  }

  fn lookup_definition_internal<'def>(&'def self, key: &'def Token) -> Option<&VecDeque<Stored>> {
    let cc = key.get_catcode();
    let name = key.get_string();
    let lookupname: &str = if (cc == Catcode::ACTIVE) || (cc == Catcode::CS) {
      name
    } else {
      cc.name()
    };

    if !lookupname.is_empty() {
      self.meaning.get(lookupname)
    } else {
      None
    }
  }

  /// used for expansion & various queries
  /// Since we're not doing digestion here, we don't need to handle mathactive,
  /// nor cs let to executable tokens
  /// This returns a definition object, or undef
  pub fn lookup_definition<'def>(&'def self, key: &'def Token) -> Option<Rc<dyn Definition>> {
    if let Some(defs) = self.lookup_definition_internal(key) {
      match defs.front() {
        Some(Stored::Conditional(entry)) => Some(entry.clone()),
        Some(Stored::Constructor(entry)) => Some(entry.clone()),
        Some(Stored::Expandable(entry)) => Some(entry.clone()),
        Some(Stored::MathPrimitive(entry)) => Some(entry.clone()),
        Some(Stored::Primitive(entry)) => Some(entry.clone()),
        Some(Stored::Register(entry)) => Some(entry.clone()),
        // TODO: Is this take on reframing a Token definition as an Expandable acceptable?
        //      Does it have unintended side-effects? Are we missing useful code paths that specifically deal with a Token
        //      in Gullet, etc?
        Some(Stored::Token(entry)) => Some(Rc::new(Expandable {
          cs: T_CS!(key),
          paramlist: None,
          expansion: entry.clone().into(),
          ..Expandable::default()
        })),
        Some(v) => {
          let message = s!("in lookup_definition for {:?}. Value was: {:?}", key, v);
          Error!("unexpected", "value", None, self, message);
          None
        },
        None => None,
      }
    } else {
      None
    }
  }

  /// Returns a definition as `Stored` so that one can call `.read_arguments`,
  /// which can't be specialized during compile-time over a trait object
  /// Instead we'll dispatch via `Stored` at runtime, to allow generic calls
  pub fn lookup_definition_stored<'def>(&'def self, key: &'def Token) -> Option<Stored> {
    match self.lookup_definition_internal(key) {
      Some(defs) => match defs.front() {
        // Still, good time to handle the Token case and catch weird storage errors
        Some(Stored::Conditional(entry)) => Some(Stored::Conditional(Rc::clone(entry))),
        Some(Stored::Constructor(entry)) => Some(Stored::Constructor(Rc::clone(entry))),
        Some(Stored::Expandable(entry)) => Some(Stored::Expandable(Rc::clone(entry))),
        Some(Stored::MathPrimitive(entry)) => Some(Stored::MathPrimitive(Rc::clone(entry))),
        Some(Stored::Primitive(entry)) => Some(Stored::Primitive(Rc::clone(entry))),
        Some(Stored::Register(entry)) => Some(Stored::Register(Rc::clone(entry))),
        Some(Stored::Token(entry)) => Some(Stored::Expandable(Rc::new(Expandable {
          cs: T_CS!(key),
          paramlist: None,
          expansion: entry.clone().into(),
          ..Expandable::default()
        }))),
        Some(v) => {
          let message = s!("in lookup_definition for {:?}. Value was: {:?}", key, v);
          Error!("unexpected", "value", None, self, message);
          None
        },
        None => None,
      },
      _ => None,
    }
  }

  /// A specialized version of `lookup_definition` for registers, since we can't adequately perform
  /// multi-dispatch when we have a "Self: Sized" for the Definition trait object.
  pub fn lookup_register_definition(&self, key: &Token) -> Option<Rc<RegisterCell>> {
    match self.lookup_definition_internal(key) {
      Some(defs) => match defs.front() {
        Some(Stored::Register(entry)) => Some(Rc::clone(entry)),
        _ => None,
      },
      _ => None,
    }
  }

  pub fn lookup_digestable_definition<'def>(&'def mut self, token: &'def Token) -> Stored {
    let cc = token.get_catcode();
    let name = token.get_string();
    let lookupname = if (cc == Catcode::ACTIVE)
      || (cc == Catcode::CS)
      || ((cc == Catcode::LETTER) || (cc == Catcode::OTHER) && self.lookup_bool("IN_MATH") && ((self.lookup_mathcode(name).unwrap_or(0)) == 0x8000))
    {
      name
    } else {
      cc.name()
    };

    Debug!("Looking up digestable {:?}", lookupname);
    let entry_opt = self.meaning.get(lookupname);

    if !lookupname.is_empty() && entry_opt.is_some() {
      Debug!("Found definition for: {:?}", lookupname);
      if let Some(entry) = entry_opt {
        if let Some(front) = entry.front() {
          if let Stored::Token(ref t) = front {
            let cc = t.get_catcode();
            if let Some(lookupname) = t.get_primitive_name() {
              if let Some(retry_entry) = self.meaning.get(lookupname) {
                // special case,
                // If a cs has been let to an executable token, lookup ITS defn.
                return retry_entry.front().unwrap().clone();
              }
            }
          }
          // if a regular definition, just return.
          return front.clone();
        }
      }
    }
    // Default return:
    token.into()
  }

  pub fn assign_definition<'def, T: Definition + Hash>(&'def mut self, _key: &'def Token, _definition: T) { unimplemented!() }

  /// And a shorthand for installing definitions
  pub fn install_definition<T: Into<Stored>>(&mut self, definition: T, scope: Option<Scope>) {
    let definition = definition.into();

    // Locked definitions!!! (or should this test be in assignMeaning?)
    // Ignore attempts to (re)define $cs from tex sources
    let cs;
    let token = match definition {
      Stored::Expandable(ref defn) => defn.get_cs(),
      Stored::Conditional(ref defn) => defn.get_cs(),
      Stored::Constructor(ref defn) => defn.get_cs(),
      Stored::Primitive(ref defn) => defn.get_cs(),
      Stored::MathPrimitive(ref defn) => defn.get_cs(),
      Stored::Register(ref defn) => defn.get_cs(),
      Stored::Token(ref token) => Cow::Borrowed(token),
      _ => panic!("_wrong_argument_for_install_definition"),
    };
    cs = token.get_cs_name().to_owned();
    // info!("-- installing definition for: {:?}", token);

    let cs_locked = s!("{}:locked", cs);
    // TODO, .is_none() should be a real false check
    let is_cs_locked = self.lookup_bool(&cs_locked);
    let is_state_unlocked = self.lookup_bool("UNLOCKED");

    if is_cs_locked && !is_state_unlocked {
      if let Some(&Stored::String(ref s)) = self.lookup_value("SOURCEFILE") {
        // report if the redefinition seems to come from document source
        if ((s == "Anonymous String") || TEX_OR_BIB_EXT_RE.is_match(s)) && (!s.ends_with(CODE_TEX_EXT)) {
          //  info("ignore", cs, self.get_stomach(), "Ignoring redefinition of $cs");
        }
        return;
      }
    }
    self.assign_internal(TableName::Meaning, &cs, definition, scope);
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

  pub fn pop_frame(&mut self) -> Result<()> {
    if self.undo.front().as_ref().unwrap().locked {
      fatal!(TargetUnexpected, Endgroup, "attempt to pop last locked stack frame");
    // Fatal('unexpected', '<endgroup>', $self->getStomach,
    // "Attempt to pop last locked stack frame"); }
    } else {
      let popped_frame = self.undo.pop_front().unwrap();
      for table_name in TableName::variants() {
        let undo_table = popped_frame.table(table_name);
        for (key, undo_count) in undo_table.iter() {
          // Typically only 1 value to shift off the table, unless scopes have been activated.
          let name_table = self.table_mut(table_name).get_mut(key).unwrap();
          for _ in 1..=*undo_count {
            name_table.pop_front();
          }
        }
      }
    }
    Ok(())
  }

  pub fn begin_semiverbatim(&mut self, extraspecials: Option<Vec<char>>) {
    // Is this a good/safe enough shorthand, or should we really be doing beginMode?
    self.push_frame();
    self.assign_value("MODE", Stored::String(s!("text")), None);
    self.assign_value("IN_MATH", Stored::Bool(false), None);
    let mut all_specials: Vec<char> = Vec::new();
    if let Some(extra) = extraspecials {
      for special in extra {
        all_specials.push(special);
      }
    }
    if let Some(&Stored::VecChar(ref specials_store)) = self.lookup_value("SPECIALS") {
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
    let new_font = if let Some(&Stored::Font(ref current_font)) = self.lookup_value("font") {
      Some(current_font.merge(fontmap!(encoding => "ASCII")))
    } else {
      None
    };
    if let Some(local_font) = new_font {
      self.assign_value("font", Stored::Font(Rc::new(local_font)), Some(Scope::Local));
    }
  }

  pub fn end_semiverbatim(&mut self) -> Result<()> { self.pop_frame() }

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
  pub fn set_prefix(&mut self, prefix: &str) { self.prefixes.insert(prefix.to_string(), true); }

  pub fn get_prefix(&self, prefix: &str) -> bool {
    match self.prefixes.get(prefix) {
      Some(b) => *b,
      _ => false,
    }
  }

  pub fn clear_prefixes(&mut self) { self.prefixes = HashMap::new(); }

  // #======================================================================

  pub fn activate_scope(&mut self, scope: &str) {
    if let Some(scope_entry) = self.stash_active.get(scope) {
      if let Some(count) = scope_entry.front() {
        self.assign_internal(TableName::StashActive, scope, Stored::Bool(true), Some(Scope::Local));
        // For now we modify a bit the latexml implementation of stash.
        // The Perl code seems to consistently and exlcusively use the value
        //  $$self{stash}{$scope}[0]
        // which suggests that we are really looking at an array-ref stored as a single value of $$self{stash}{$scope}
        // so let's just do that for simplicity's sake and assume
        // defns = $$self{stash}{$scope}

        // Also, we need to take ownership of the stashed data, so that we can assign it. TODO: Potential to optimize?
        let defns: Vec<(TableName, String, Stored)> = if let Some(defns) = self.stash.get(scope) {
          defns
            .iter()
            .map(|(table_name, key, value)| (*table_name, key.to_string(), value.clone()))
            .collect()
        } else {
          Vec::new()
        };

        // Now make local assignments for all those in the stash.
        if let Some(frame) = self.undo.front_mut() {
          for (table_name, key, _value) in defns.iter() {
            // Here we ALWAYS push the stashed values into the table
            // since they may be popped off by deactivateScope
            let mut frame_table = frame.table_mut(*table_name);
            let mut frame_count = frame_table.entry(key.to_string()).or_default();
            *frame_count += 1; // Note that this many values must be undone
          }
        }
        if !self.undo.is_empty() {
          // avoid borrowing mutably twice, by iterating a separate time for inserting the defs
          for (table_name, key, value) in defns.iter() {
            let mut table_entry = self.table_mut(*table_name).entry(key.to_string()).or_insert_with(VecDeque::new);
            table_entry.push_front(value.clone()); // And push new binding.
          }
        }
      }
    }
  }

  // # Probably, in most cases, the assignments made by activateScope
  // # will be undone by egroup or popping frames.
  // # But they can also be undone explicitly

  pub fn deactivate_scope(&mut self, scope: &str) {
    if let Some(scope_entry) = self.stash_active.get(scope) {
      self.assign_internal(TableName::StashActive, scope, Stored::Bool(false), Some(Scope::Global));
      let defns: Vec<(TableName, String, Stored)> = if let Some(defns) = self.stash.get(scope) {
        defns
          .iter()
          .map(|(table_name, key, value)| (*table_name, key.to_string(), value.clone()))
          .collect()
      } else {
        Vec::new()
      };

      let mut countdown_keys = Vec::new();
      for (table_name, key, value) in defns.iter() {
        let table_entry = self.table_mut(*table_name).entry(key.to_string()).or_default();
        if (*table_entry).front() == Some(&value) {
          // Here we're popping off the values pushed by activateScope
          // to (possibly) reveal a local assignment in the same frame, preceding activateScope.
          (*table_entry).pop_front();
          countdown_keys.push((table_name, key));
        } else {
          let message = s!(
            "Unassigning wrong value for $key from table $table in deactivateScope\
             value is {:?} but stack is {:?}",
            value,
            table_entry.iter().map(ToString::to_string).collect::<Vec<String>>().join(", ")
          );
          let stomach = self.stomach.borrow();
          use crate::common::object::Object;
          Warn!("internal", key, stomach, self, message);
        }
      }

      if let Some(mut frame) = self.undo.front_mut() {
        for (table_name, key) in countdown_keys {
          let mut frame_table = frame.table_mut(*table_name);
          let mut frame_count = frame_table.entry(key.to_string()).or_default();
          *frame_count -= 1;
        }
      }
    }
  }

  // sub getKnownScopes {
  //   my ($self) = @_;
  //   my @scopes = sort keys %{ $$self{stash} };
  //   return @scopes; }

  // sub getActiveScopes {
  //   my ($self) = @_;
  //   my @scopes = sort keys %{ $$self{stash_active} };
  //   return @scopes; }

  //======================================================================
  // Units.
  // Put here since it could concievably evolve to depend on the current font.

  pub fn convert_unit<T: ToString>(&self, unit: T) -> f32 {
    let unit = unit.to_string().to_lowercase();
    // Eventually try to track font size?
    match unit.as_str() {
      "em" => 10.0 * 65536.0,
      "ex" => 4.3 * 65536.0,
      "mu" => 10.0 * 65536.0 / 18.0,
      u => match UNITS.get(u) {
        Some(sp) => *sp,
        None => {
          let message = s!("Illegal unit of measure {:?}, assuming pt.", u);
          Warn!("expected", "<unit>", None, self, message);
          *UNITS.get("pt").unwrap()
        },
      },
    }
  }

  // #======================================================================

  pub fn note_status(&self, category: &str) {
    // Ok, note status is *EXTREMELY* localized
    // it only touches the status field of state,
    // and has NO side-effects to any of the other stateful machinery.
    // So. Let's make it possible to call it from any context, including immutable state borrows,
    // by using interiror mutability.
    // "Proof by commenting intent"

    // if ($type eq 'undefined') {
    //   map { $$self{status}{undefined}{$_}++ } @data; }
    // elsif ($type eq 'missing') {
    //   map { $$self{status}{missing}{$_}++ } @data; }
    // else {
    //   $$self{status}{$type}++;
  }

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

      let mut desc_keys: Vec<String> = desc.keys().map(ToString::to_string).collect();
      desc_keys.sort();
      for kid in desc_keys {
        let mut best = 0; // Find best path to $kid.
        let mut desc_kid_keys: Vec<String> = desc
          .entry(kid.to_owned())
          .or_insert_with(HashMap::new)
          .keys()
          .map(ToString::to_string)
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
  ) {
    let start = match start_opt {
      Some(s) => s,
      None => String::new(),
    };

    // A bit tricky here, we need to release the state.model borrow immediately, which is why we
    // move ownership of the tag strings into the tag_contents vector.
    // That leads to a bunch of .clone()s later one, but stays close to the original algorithm
    let tag_contents: Vec<String> = self.model.get_tag_contents(tag).iter().map(ToString::to_string).collect();

    for kid in tag_contents {
      if desc.entry(kid.clone()).or_insert_with(HashMap::new).get(&start).is_some() {
        continue;
      } // Already solved

      if !start.is_empty() {
        desc.entry(kid.clone()).or_insert_with(HashMap::new).insert(start.clone(), desirability);
      }

      if kid != "#PCDATA" && openable.contains(&kid) {
        let inner = if !start.is_empty() { start.clone() } else { kid.to_string() };

        self.compute_indirect_model_aux(&kid, Some(inner), desirability, openable, desc);
      }
    }
  }

  /// Initialize various stomach parameters, preload, etc.
  pub fn initialize_stomach(&mut self) {
    self.assign_value("MODE", Stored::String(s!("text")), Some(Scope::Global));
    self.assign_value("IN_MATH", Stored::Bool(false), Some(Scope::Global));
    self.assign_value("PRESERVE_NEWLINES", Stored::Bool(true), Some(Scope::Global));
    self.assign_value("afterGroup", Stored::VecDigested(Vec::new()), Some(Scope::Global));
    self.assign_value("afterAssignment", Stored::Tokens(Tokens!()), Some(Scope::Global)); // undef ???
    self.assign_value("groupInitiator", Stored::String(s!("Initialization")), Some(Scope::Global));
    // Setup default fonts.
    self.assign_value("font", Stored::Font(Rc::new(Font::text_default())), Some(Scope::Global));
    self.assign_value("mathfont", Stored::Font(Rc::new(Font::math_default())), Some(Scope::Global));
  }

  // Package helpers used in core need to be localized here -- as State methods
  /// `Let` macro setter
  pub fn let_i(&mut self, token1: &Token, token2: Token, scope: Option<Scope>) {
    // If strings are given, assume CS tokens (most common case)
    let meaning = match self.lookup_meaning(&token2) {
      Some(m) => m,
      None => Stored::Token(token2),
    };
    self.assign_meaning(token1, meaning, scope);
    self.after_assignment();
  }
  /// `XEquals` check for two token arguments
  pub fn x_equals(&mut self, token1: &Token, token2: &Token) -> bool {
    let def1_opt = self.lookup_meaning(token1);
    let def2_opt = self.lookup_meaning(token2); // ditto
    match (def1_opt, def2_opt) {
      (None, None) => true,                     // true if both undefined
      (Some(def1), Some(def2)) => def1 == def2, // If both have defns, must be same defn!
      _ => false,                               // False, if only one has 'meaning'
    }
  }

  pub fn load_font_map(&self, encoding: &str) -> Option<&Fontmap> {
    let fontmap_key = s!("{}_fontmap", encoding);
    if let Some(map) = self.lookup_value(&fontmap_key) {
      return map.into();
    }

    // TODO: Once we try to load font maps via require package we will have some serious mutability
    // issues to resolve... punt for now.

    // no map, try to load one
    // let can_load_ok: bool;
    // let fail_suffix = "_fontmap_failed_to_load";
    // let fail_to_load_key = s!("{}{}", encoding, fail_suffix);
    // {
    //   can_load_ok = !self.lookup_bool(&fail_to_load_key);
    // }

    // if can_load_ok {
    //   self.assign_value(&fail_to_load_key, true, None); // Stop recursion?

    //   // TODO: difficult .... this is main rtx functionality
    //   // RequirePackage(lc($encoding), type => 'fontmap'); //
    //   self.assign_value(&fail_to_load_key, false, None);
    //   {
    //     if let Some(map) = self.lookup_value(&fontmap_key) {
    //       // Got map?
    //       return map.into();
    //     }
    //   }
    //   self.assign_value(&fail_to_load_key, false, None);

    //   self.assign_value(&fail_to_load_key, true, Some(Scope::Global));
    //   None
    // } else {
    None
    // }
  }

  // WALL OF SHAME
  // HACKY functions that violate Rust's mutability guards, to succeed in emulating the most
  // arbitrary uses of Global scope in latexml
  pub fn after_assignment(&mut self) {
    if let Some(Stored::Tokens(after)) = self.remove_value("afterAssignment") {
      if !after.is_empty() {
        self.stomach.borrow_mut().get_gullet_mut().unread(after); // primitive returns boxes, so these need to be digested!
      }
    }
  }
}
