use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::rc::Rc;

use crate::alignment::Alignment;
use crate::common::arena::{self, SymHashMap, SymStr, EMPTY_SYM, FONT_SYM, GLOBAL_DEFS_SYM};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::glue::Glue;
use crate::common::model::{self, compute_indirect_model_aux};
use crate::common::model::{IndirectModel, Model};
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
pub use crate::common::store::Stored; // reexport for convenience
use crate::common::BindingDispatcher;
use crate::definition::argument::ArgWrap;
use crate::definition::conditional::ConditionalType;
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::register::{Register, RegisterValue};
use crate::definition::Definition;
use crate::document::resource::Resource;
use crate::document::tag::TagOptions;
use crate::gullet;
use crate::mouth;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::util::pathname;
use crate::{Digested, DigestedData};

// expose Perl-style local assignments from state
pub use crate::common::local_assignments::*;

static CODE_TEX_EXT: &str = ".code.tex";

/// regex for *.tex and *.bib
static TEX_OR_BIB_EXT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(tex|bib)$").unwrap());
/// Used in conversion to scaled points
pub static UNITS: Lazy<HashMap<String, f64>> = Lazy::new(|| {
  map!(
    "pt" => 65536.0,
    "pc" => 12.0 * 65536.0,
    "in" => 72.27 * 65536.0,
    "bp" => 72.27 * 65536.0 / 72.0,
    "px" => 72.27 * 65536.0 / 72.0,   // Assume px=bp ?
    "cm" => 72.27 * 65536.0 / 2.54,
    "mm" => 72.27 * 65536.0 / 2.54 / 10.0,
    "dd" => 1238.0 * 65536.0 / 1157.0,
    "cc" => 12.0 * 1238.0 * 65536.0 / 1157.0,
    "sp" => 1.0
  )
});

/// installation scope in the state_tables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
  /// globally visible, does not expire
  Global,
  /// globally visible, but expires at the end of the current group
  Local,
  /// a named scope - visible only when explicitly activated
  Named(SymStr),
}

/// the kinds of tables bookkept in the State
#[derive(Debug, Copy, Clone)]
pub enum TableName {
  /// token meaning
  Meaning,
  /// all stateful values
  Value,
  /// catcode bindings
  Catcode,
  /// mathcode bindings
  Mathcode,
  /// sf code bindings
  Sfcode,
  /// lc code bindings
  Lccode,
  /// uc code bindings
  Uccode,
  /// del code bindings
  Delcode,
  /// stash of inactive named values
  Stash,
  /// active stash of named values
  StashActive,
}
impl Display for TableName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        TableName::Meaning => "Meaning",
        TableName::Value => "Value",
        TableName::Catcode => "Catcode",
        TableName::Mathcode => "Mathcode",
        TableName::Sfcode => "Sfcode",
        TableName::Lccode => "Lccode",
        TableName::Uccode => "Uccode",
        TableName::Delcode => "Delcode",
        TableName::Stash => "Stash",
        TableName::StashActive => "StashActive",
      }
    )
  }
}
impl TableName {
  /// provides all TableName variants. useful for iterating over all tables
  pub fn variants() -> &'static [TableName] {
    use self::TableName::*;
    &[
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
    ]
  }
}

/// High-level catcode profiles
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Catcodes {
  /// the usual mainmatter catcodes (e.g. @ is other)
  Standard,
  /// the usual style catcodes (e.g. @ is letter)
  Style,
  /// left unspecified
  None,
}

/// Ledger for stacked assignments
pub type AssignmentCount = HashMap<SymStr, usize>;
/// The `(table_name, key, value)` contents of a stored table of assignments
pub type StashTable = Vec<(TableName, SymStr, Stored)>;
#[derive(Debug, Clone, Default)]
/// For each of several tables (being "value", "meaning", "catcode" or other space of names),
/// each table maintains the bound values, and "undo" defines the stack frames
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

impl UndoFrame {
  /// borrow the undo assignment counts for a given table name
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
      Stash => &self.stash,
      StashActive => &self.stash_active,
    }
  }
  /// mutably borrow the undo assignment counts for a given table name
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
      Stash => &mut self.stash,
      StashActive => &mut self.stash_active,
    }
  }
}

/// The type of values that are storable by the different namespaced "tables" in State.
///
/// There are tables for:
///
///  catcode: keys are char;
///     Also, `math:char` =1 when `char` is active in math.
///  mathcode, sfcode, lccode, uccode, delcode : are similar to catcode but store
///    additional kinds codes per char (see TeX)
///  value: keys are anything (typically a string, though) and value is the value associated with it
///  meaning: The definition assocated with `key`, usually a control-sequence.
///  stash & stash_active: support named scopes
///      (see also activateScope & deactivateScope)
pub type Table = HashMap<SymStr, VecDeque<Stored>>;

/// The state efficiently bookkeeps the bindings in a TeX-like fashion.
///
/// Bindings associate data with keys (eg definitions with macro names)
/// and respect TeX grouping; that is, an assignment is only in effect
/// until the current group (opened by \bgroup) is closed (by \egroup).
pub struct State {
  // Tables
  /// bookkeeps arbitrary Stored values
  value: Table,
  /// The definition assocated with a key, usually a control-sequence.
  meaning: Table,
  stash: Table,
  stash_active: Table,
  catcode: Table,
  mathcode: Table,
  sfcode: Table,
  lccode: Table,
  uccode: Table,
  delcode: Table,
  // Table bookkeeping
  undo: VecDeque<UndoFrame>,
  // stateful runtime - data structures
  /// the schema-derived model used for the current document
  prefixes: HashMap<SymStr, bool>, // ?
  pub tag_properties: HashMap<SymStr, TagOptions>,
  /// an optional indirect model for long-distance relationships
  pub indirect_model: Option<IndirectModel>,
  /// Document-related resources declared during core conversion, pending until XML is finalized
  pub pending_resources: Vec<Resource>,
  // stateful runtime - simple fields
  // TODO: Maybe group these in a "SessionFlags" struct?
  //       we can then reset that if we reimplement a daemon app
  pub verbosity: i32,
  pub input_encoding: Option<String>,
  // strict: bool,
  // include_comments: bool,
  /// current paths to search for TeX inputs
  pub search_paths: VecDeque<String>,
  /// current paths to search for graphics
  pub graphics_paths: VecDeque<String>,
  // include_styles: bool,
  /// flag to disable math parsing
  pub nomathparse: bool,
  // TODO: We can make this a Vec<BindingDispatcher> if we want to accumulate more definitions
  /// A dispatcher routing to the compiled code of the in-distro latexml bindings
  pub bindings_dispatch: Option<BindingDispatcher>,
  /// Auxiliary convenience -- extra dispatch
  pub extra_bindings_dispatch: Option<BindingDispatcher>,
}
unsafe impl Send for State {}
// State is NOT Sync!
// each core conversion job must be localized in ONE thread.

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
      value: HashMap::default(),
      meaning: HashMap::default(),
      stash: HashMap::default(),
      stash_active: HashMap::default(),
      catcode: HashMap::default(),
      mathcode: HashMap::default(),
      sfcode: HashMap::default(),
      lccode: HashMap::default(),
      uccode: HashMap::default(),
      delcode: HashMap::default(),
      // Table bookkeeping
      undo: undo_vdq,
      // stateful runtime - data structures
      prefixes: HashMap::default(),
      tag_properties: HashMap::default(),
      indirect_model: None,
      pending_resources: Vec::new(),
      // stateful runtime - simple fields
      verbosity: 0,
      input_encoding: None,
      // strict: false,
      // include_comments: true,
      search_paths: VecDeque::new(),
      graphics_paths: VecDeque::new(),
      // include_styles: false,
      nomathparse: false,
      bindings_dispatch: None,
      extra_bindings_dispatch: None,
    }
  }
}

#[thread_local]
static STD_STATE: Lazy<RefCell<State>> = Lazy::new(|| {
  RefCell::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  }))
});
#[thread_local]
static STY_STATE: Lazy<RefCell<State>> = Lazy::new(|| {
  RefCell::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Style),
    ..StateOptions::default()
  }))
});
#[thread_local]
static STATE: Lazy<RefCell<State>> = Lazy::new(|| {
  RefCell::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  }))
});

macro_rules! state {
  () => {
    (*STATE).borrow()
  };
}
macro_rules! state_mut {
  () => {
    (*STATE).borrow_mut()
  };
}
macro_rules! sty_state_mut {
  () => {
    (*STY_STATE).borrow_mut()
  };
}
macro_rules! std_state_mut {
  () => {
    (*STD_STATE).borrow_mut()
  };
}

/// state fields allowed for customization during construction
#[derive(Default)]
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

// Public interface: package-access methods, for an implied thread-local singleton STATE

// Private interface: struct-access methods, for a concrete piece of State data

impl State {
  pub fn new(options: StateOptions) -> Self {
    use crate::token::Catcode::*;

    // Setup default catcodes.
    let catcode_profile = match options.catcodes {
      None => Catcodes::Standard,
      Some(cp) => cp,
    };

    let mut catcodes: HashMap<char, Catcode> = HashMap::default();
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
        catcodes.insert('\0', ESCAPE);
        catcodes.insert('\u{000c}', ACTIVE);
        for c in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
          catcodes.insert(c, LETTER);
        }
      },
      Catcodes::None => {},
    }
    if catcode_profile == Catcodes::Style {
      catcodes.insert('@', LETTER);
    }

    let mut value_table = HashMap::default();
    let mut specials_vdq = VecDeque::new();
    specials_vdq.push_front(Stored::Chars(Box::new([
      '^', '_', '~', '&', '$', '#', '\'',
    ])));
    value_table.insert(arena::pin_static("SPECIALS"), specials_vdq);

    let mut catcodes_typed: Table = HashMap::default();
    for (k, v) in catcodes {
      let mut vdq = VecDeque::new();
      vdq.push_front(Stored::Catcode(v));
      catcodes_typed.insert(arena::pin_char(k), vdq);
    }

    // Basic defaults
    if let Some(model) = options.model {
      model::set_model(model);
    };
    let verbosity = options.verbosity.unwrap_or(0);
    // let strict = options.strict.unwrap_or(false);
    // let include_comments = options.include_comments.unwrap_or(true);
    // let include_styles = options.include_styles.unwrap_or(false);
    let nomathparse = options.nomathparse.unwrap_or(false);

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

    let mut state = State {
      value: value_table,
      catcode: catcodes_typed,
      verbosity,
      // strict,
      // include_comments,
      search_paths,
      graphics_paths,
      // include_styles,
      input_encoding: options.input_encoding,
      nomathparse,
      ..State::default()
    };
    // TODO: should these be *fields* in state or really as in Perl - globally assigned values?
    state.assign_value(
      "DOCUMENTID",
      options.documentid.unwrap_or_default(),
      Some(Scope::Global),
    );

    state
  }

  /// borrow/get the named table
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
      Stash => &self.stash,
      StashActive => &self.stash_active,
    }
  }
  /// mutably borrow/get the named table
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
      Stash => &mut self.stash,
      StashActive => &mut self.stash_active,
    }
  }

  // needed for assign_internal, so keeping it as a object method
  /// gets the current value of a named prefix
  pub fn get_prefix(&self, prefix: &str) -> bool {
    match self.prefixes.get(&arena::pin(prefix)) {
      Some(b) => *b,
      _ => false,
    }
  }

  fn assign_internal(
    &mut self,
    table_name: TableName,
    key: SymStr,
    value: Stored,
    mut scope_opt: Option<Scope>,
  ) {
    // hotcode lookupDefinition for \globaldefs,
    // since this is called extremely often and should be highly standardized
    if let Some(globaldefs) = self.value.get(&GLOBAL_DEFS_SYM) {
      if let Some(global_value) = globaldefs.front() {
        // magic TeX register override: \globaldefs
        match *global_value {
          Stored::Int(1) => {
            scope_opt = Some(Scope::Global);
          },
          Stored::Int(-1) => {
            scope_opt = Some(Scope::Local);
          },
          _ => {},
        }
      }
    }
    // regular check, local scope is default, unless a global prefix is set
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
    match scope {
      Scope::Global => {
        // We are going to change the model, where we first count the total number of definitions to
        // pop, and then pop them, in order to never mutably morrow more than once at a time.
        let mut undo_count = 0;

        // Remove bindings made in all frames down-to & including the next lower locked frame
        let mut last_frame = None;
        for frame in &mut self.undo {
          let is_locked = frame.locked;
          let frame_table = frame.table_mut(table_name);
          if let Some(n) = frame_table.remove(&key) {
            undo_count += n;
          }
          last_frame = Some(frame);
          if is_locked {
            break;
          }
        }
        // whatever is left -- if anything -- should be bindings below the locked frame.
        if let Some(frame) = last_frame {
          frame.table_mut(table_name).insert(key, 1); // Note that there's only one
                                                      // value in the stack, now
        }

        // Undo the bindings, if `key` was bound in this frame
        let state_table = self.table_mut(table_name);
        if let Some(defs) = state_table.get_mut(&key) {
          for _ in 1..=undo_count {
            defs.pop_front();
          }
        }

        let table_entry = state_table.entry(key).or_default();
        table_entry.push_front(value);
      },
      Scope::Local => {
        // Again, split the logic as 1) bookkeeping in undo, then 2) operations in state_tables
        let mut is_replace = false;
        // 1. Undo mutable logic
        if let Some(current_frame) = self.undo.front_mut() {
          let current_frame_table = current_frame.table_mut(table_name);

          is_replace = current_frame_table.get(&key).unwrap_or(&0) > &0;
          if is_replace { // If the value was previously assigned in this frame
             // we do this in 2.1, then proceed to 2.2
          } else {
            // Otherwise, push new value & set 1 to be undone
            current_frame_table.insert(key, 1);
            //  And push new binding in 2.2
          }
        }
        // 2. state_table mutable logic
        let state_table = self.table_mut(table_name);
        let defs = state_table.entry(key).or_default();
        if is_replace {
          // 2.1. Replace the value, i.e. remove existing one
          defs.pop_front();
        }
        // 2.2 Add new value
        defs.push_front(value);
      },
      Scope::Named(scope_name) => {
        // initialize stash if empty
        let needs_init = match self.stash.get(&scope_name) {
          None => true,
          Some(v) => v.is_empty(),
        };
        if needs_init {
          self.assign_internal(
            TableName::Stash,
            scope_name,
            Stored::Stash(Vec::new()),
            Some(Scope::Global),
          );
        }
        if let Some(Stored::Stash(ref mut stash)) =
          self.stash.get_mut(&scope_name).as_mut().unwrap().get_mut(0)
        {
          stash.push((table_name, key, value.clone()));
        }
        let has_active = match self.stash_active.get(&scope_name) {
          None => false,
          Some(v) => !v.is_empty(),
        };
        if has_active {
          self.assign_internal(table_name, key, value, Some(Scope::Local));
        }
      },
    }
  }

  /// assigns a `Stored` value at the given key and scope
  pub fn assign_value<T: Into<Stored>, S: Into<Option<Scope>>>(
    &mut self,
    key: &str,
    value: T,
    scope: S,
  ) {
    let value = value.into();
    let scope = scope.into();
    let key_sym = arena::pin(key);
    self.assign_internal(TableName::Value, key_sym, value, scope);
  }
  //======================================================================
  /// fetches a Stored value at the given key, from the Value table
  pub fn lookup_value(&self, key: &str) -> Option<&Stored> {
    match self.value.get(&arena::pin(key)) {
      None => None,
      Some(vvec) => match vvec.front() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other),
      },
    }
  }
  pub fn lookup_value_sym(&self, key: &SymStr) -> Option<&Stored> {
    match self.value.get(key) {
      None => None,
      Some(vvec) => match vvec.front() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other),
      },
    }
  }

  /// mutably borrows a Stored value at the given key, from the Value table
  pub fn lookup_value_mut(&mut self, key: &str) -> Option<&mut Stored> {
    match self.value.get_mut(&arena::pin(key)) {
      None => None,
      Some(vvec) => match vvec.front_mut() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other),
      },
    }
  }
  /// like `lookup_value` but only recognizes `Stored::VecDequeStored`
  pub fn lookup_vecdeque(&self, key: &str) -> Option<&VecDeque<Stored>> {
    match self.lookup_value(key) {
      None | Some(Stored::None) => None,
      Some(v) => v.into(),
    }
  }
  pub fn lookup_font_info(&self, key: &Token) -> Result<Option<&Stored>> {
    let key_str = if let Some(defn) = lookup_definition(key)? {
      s!("fontinfo_{}", defn.get_cs_name())
    } else {
      s!("fontinfo_{key}")
    };
    Ok(self.lookup_value(&key_str))
  }
  /// manage a (global) hash of values
  pub fn lookup_mapping(&self, map: &str, key: &str) -> Option<&Stored> {
    let map_sym = arena::pin(map);
    match self.value.get(&map_sym) {
      None => None,
      Some(map_vec) => match map_vec.front() {
        Some(Stored::HashStored(h)) => h.get(key),
        _ => None,
      },
    }
  }

  pub fn lookup_mapping_keys(&self, map: &str) -> Vec<SymStr> {
    let map_sym = arena::pin(map);
    match self.value.get(&map_sym) {
      None => Vec::new(),
      Some(map_vec) => match map_vec.front() {
        Some(Stored::HashStored(h)) => h.keys().copied().collect(),
        _ => Vec::new(),
      },
    }
  }

  pub fn lookup_stacked_values(&self, key: &str) -> Vec<&Stored> {
    let key_sym = arena::pin(key);
    if let Some(vdq) = self.value.get(&key_sym) {
      vdq.iter().collect::<Vec<&Stored>>()
    } else {
      Vec::new()
    }
  }

  fn lookup_definition_internal(&self, key: &Token) -> Option<&VecDeque<Stored>> {
    let cc = key.get_catcode();
    let name = key.get_sym();
    let lookupname: Option<SymStr> = if (cc == Catcode::ACTIVE) || (cc == Catcode::CS) {
      if name == *EMPTY_SYM {
        None
      } else {
        Some(name)
      }
    } else {
      key.get_executable_primitive_name().map(arena::pin)
    };

    if let Some(lname) = lookupname {
      self.meaning.get(&lname)
    } else {
      None
    }
  }
  pub fn ensure_tag_property(&mut self, tag: SymStr) -> &mut TagOptions {
    self.tag_properties.entry(tag).or_default()
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum RotateState {
  Main,
  Std,
  Sty,
}
#[thread_local]
static mut STATE_IN_USE: RotateState = RotateState::Main;

pub fn use_sty_state() {
  unsafe {
    if STATE_IN_USE != RotateState::Sty {
      let mut sty_state = sty_state_mut!();
      let mut main_state = state_mut!();
      std::mem::swap(&mut *sty_state, &mut *main_state);
      STATE_IN_USE = RotateState::Sty;
    }
  }
}
pub fn use_std_state() {
  unsafe {
    if STATE_IN_USE != RotateState::Std {
      let mut std_state = std_state_mut!();
      let mut main_state = state_mut!();
      std::mem::swap(&mut *std_state, &mut *main_state);
      STATE_IN_USE = RotateState::Std;
    }
  }
}
pub fn use_main_state() {
  unsafe {
    match STATE_IN_USE {
      RotateState::Sty => {
        let mut sty_state = sty_state_mut!();
        let mut main_state = state_mut!();
        std::mem::swap(&mut *sty_state, &mut *main_state);
        STATE_IN_USE = RotateState::Main;
      },
      RotateState::Std => {
        let mut std_state = std_state_mut!();
        let mut main_state = state_mut!();
        std::mem::swap(&mut *std_state, &mut *main_state);
        STATE_IN_USE = RotateState::Main;
      },
      RotateState::Main => {},
    };
  }
}

/// A shorthand for installing definitions
pub fn install_definition<T: Into<Stored>>(definition: T, scope: Option<Scope>) {
  let definition = definition.into();

  // Locked definitions!!! (or should this test be in assignMeaning?)
  // Ignore attempts to (re)define $cs from tex sources
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
  let cs_sym = token.get_cs_name();
  let lock_key = token.with_cs_name(|cs| s!("{cs}:locked"));
  if lookup_bool(&lock_key) && !state_is_unlocked() {
    if let Some(Stored::String(s)) = state!().lookup_value("SOURCEFILE") {
      // report if the redefinition seems to come from document source
      if arena::with(*s, |txt| {
        txt == "Anonymous String" || TEX_OR_BIB_EXT_RE.is_match(txt) && !txt.ends_with(CODE_TEX_EXT)
      }) {
        Info!("ignore", lock_key, "Ignoring redefinition of {lock_key}");
      }
    }
  } else {
    state_mut!().assign_internal(TableName::Meaning, cs_sym, definition, scope);
  }
}

/// Generate a stub definition for an undefined control-sequence,
/// along with appropriate error messge.
pub fn generate_error_stub(token: &Token) -> Result<Token> {
  let cs = token.with_cs_name(ToString::to_string);
  note_status(LogStatus::Undefined, Some(&cs));
  // To minimize chatter, go ahead and define it...
  if cs.starts_with("\\if") {
    // Apparently an \ifsomething ???
    let name = cs.replace("\\if", "");
    Error!(
      "undefined",
      token,
      s!(
        "The token {} is not defined. Defining it now as with \\newif",
        token.stringify()
      )
    );
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}true", name)),
        None,
        Some(s!("\\let{}\\iftrue", cs).into()),
        None,
      )?,
      Some(Scope::Global),
    );
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}false", name)),
        None,
        Some(s!("\\let{}\\iffalse", cs).into()),
        None,
      )?,
      Some(Scope::Global),
    );
    let_i(token, &T_CS!("\\iffalse"), Some(Scope::Global));
  } else {
    Error!(
      "undefined",
      token,
      s!(
        "The token {} is not defined. Defining it now as <ltx:ERROR/>",
        token.stringify()
      )
    );
    install_definition(
      Constructor {
        cs: *token,
        replacement: Some(Rc::new(move |document, _args, _props| {
          document.make_error("undefined", &cs)
        })),
        ..Constructor::default()
      },
      //TODO: sizer => "X"),
      Some(Scope::Global),
    );
  }
  Ok(*token)
}

// SAFETY
// any method which does not return a borrowed piece of data should be package-level
// so that the global singleton State can get locked+unlocked during the same call
// thus entirely AVOIDING possible runtime panics due to RefCell lock races.
// TODO: Should this be a prelude?

/// assigns a `Stored` value at the given key and scope
pub fn assign_value<T: Into<Stored>, S: Into<Option<Scope>>>(key: &str, value: T, scope: S) {
  state_mut!().assign_value(key, value, scope)
}

/// assigns a `Stored` value at the given (arena ticket!) key and scope
pub fn assign_value_sym<T: Into<Stored>, S: Into<Option<Scope>>>(key: SymStr, value: T, scope: S) {
  let value = value.into();
  let scope = scope.into();
  state_mut!().assign_internal(TableName::Value, key, value, scope);
}

/// inline lookup_value after which globally assign an empty Tokens() to undo
pub fn remove_value(key: &str) -> Option<Stored> {
  let key_sym = arena::pin(key);
  match state_mut!().value.get_mut(&key_sym) {
    None => None,
    Some(vvec) => match vvec.front_mut() {
      None | Some(&mut Stored::None) => Option::None,
      Some(found) => Some(std::mem::take(found)),
    },
  }
}
/// Replaces the value in question with `Stored::None` (see `checkin_value` for returning it)
pub fn checkout_value(key: &str) -> Option<Stored> {
  match state_mut!().value.get_mut(&arena::pin(key)) {
    None => None,
    Some(vvec) => vvec.front_mut().map(std::mem::take),
  }
}
/// Returns a value into its `Stored::None` placeholder (see `checkout_value` for taking it)
pub fn checkin_value(key: &str, value: Stored) {
  match state_mut!().value.get_mut(&arena::pin(key)) {
    None => todo!(),
    Some(vvec) => match vvec.front_mut() {
      None => todo!(),
      Some(found) => {
        match found {
          Stored::None => std::mem::replace(found, value),
          _ => panic!("checkin_value should only be called after checkout_value"),
        };
      },
    },
  }
}
/// manage a (global) list of values
pub fn push_value<T: Into<Stored>>(key: &str, value: T) -> Result<()> {
  let key_sym = arena::pin(key);
  let value = value.into();
  let mut state = state_mut!();
  if !state.value.contains_key(&key_sym) {
    state.assign_internal(
      TableName::Value,
      key_sym,
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    );
  }
  match state.value.get_mut(&key_sym).unwrap().front_mut() {
    Some(&mut Stored::VecDequeStored(ref mut front)) => front.push_back(value),
    // auto-vivify, if None
    Some(ref mut field) if matches!(field, Stored::None) => {
      let mut new_vdq = VecDeque::new();
      new_vdq.push_back(value);
      **field = Stored::VecDequeStored(new_vdq);
    },
    other => {
      let message =
        s!("BUG: Tried to push_value into an unsupported Stored field! Field was: {other:?}");
      Error!("State", "Stored", message);
    },
  }
  Ok(())
}
/// pops the last value in a named `Stored::VecDequeStored` queue, if any
pub fn pop_value(key: &str) -> Result<Option<Stored>> {
  let key_sym = arena::pin(key);
  let mut state = state_mut!();
  if !state.value.contains_key(&key_sym) {
    state.assign_internal(
      TableName::Value,
      key_sym,
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    );
  }
  if let Some(&mut Stored::VecDequeStored(ref mut front)) =
    state.value.get_mut(&key_sym).unwrap().front_mut()
  {
    Ok(front.pop_back())
  } else {
    Error!(
      "State",
      "Stored",
      "BUG: Tried to pop_value from a non-vecdeque value key!"
    );
    Ok(None)
  }
}
/// Check if the Value table contains a given key
pub fn has_value(key: &str) -> bool {
  let key_sym = arena::pin(key);
  match state!().value.get(&key_sym) {
    None => false,
    Some(list) => match list.front() {
      None => false,
      Some(v) => !matches!(v, &Stored::None),
    },
  }
}
/// Pushes Tokens into a `Stored::Tokens` value when defined,
/// or assigns when new.
pub fn push_tokens(key: &str, value: Tokens) {
  let mut state = state_mut!();
  match state.lookup_value_mut(key) {
    Some(Stored::Tokens(ref mut tks)) => tks.unlist_mut().extend(value.unlist()),
    None | Some(Stored::None) => state.assign_value(key, Stored::Tokens(value), None),
    Some(other) => panic!("Can only push_tokens into a Stored::Tokens, but got {other:?}"),
  }
}

pub fn lookup_value(key: &str) -> Option<Stored> { state!().lookup_value(key).cloned() }
pub fn with_value<R, FnR>(key: &str, caller: FnR) -> R
where FnR: FnOnce(Option<&Stored>) -> R {
  caller(state!().lookup_value(key))
}
pub fn with_value_mut<R, FnR>(key: &str, caller: FnR) -> R
where FnR: FnOnce(Option<&mut Stored>) -> R {
  caller(state_mut!().lookup_value_mut(key))
}
/// A bit of Perl "existence as truth" semantics mixed in with proper boolean lookup
pub fn lookup_bool(key: &str) -> bool {
  let state = state!();
  match state.lookup_value(key) {
    None => false,
    Some(v) => v.into(),
  }
}
/// like `lookup_value`, but casts the entry into a SymStr from the string interner
///  (`EMPTY_SYM` if None)
pub fn lookup_string_sym(key: &str) -> SymStr {
  let state = state!();
  match state.lookup_value(key) {
    None => *EMPTY_SYM,
    Some(Stored::String(v)) => *v,
    Some(other) => arena::pin(other.to_string()),
  }
}
/// like `lookup_value`, but casts the entry into a String (empty if None)
pub fn lookup_string(key: &str) -> String {
  let state = state!();
  match state.lookup_value(key) {
    None => String::new(),
    Some(v) => v.into(),
  }
}
/// like `lookup_value` but only recognizes Int, Bool and Number variants of Stored (default: 0)
pub fn lookup_int(key: &str) -> i64 {
  let state = state!();
  match state.lookup_value(key) {
    Some(Stored::Int(i)) => *i,
    Some(Stored::Bool(true)) => 1, // this is Perl's boolean -> integer semantics
    Some(Stored::Number(n)) => n.value_of(),
    _ => 0,
  }
}

pub fn remove_vecdeque(key: &str) -> Option<VecDeque<Stored>> {
  match remove_value(key) {
    Some(Stored::VecDequeStored(v)) => Some(v),
    _ => None,
  }
}
/// convenience method to lookup the current value at the "font" key
pub fn lookup_font() -> Option<Rc<Font>> {
  match state!().lookup_value_sym(&FONT_SYM) {
    None | Some(Stored::None) => None,
    Some(f) => f.into(),
  }
}
/// convenience method to lookup the current value at the "mathfont" key
pub fn lookup_mathfont() -> Option<Rc<Font>> {
  match state!().lookup_value("mathfont") {
    None | Some(Stored::None) => None,
    Some(v) => v.into(),
  }
}

/// a convenience method to globally asign a `Font` to the "font" key
pub fn assign_font(font: Rc<Font>, scope: Option<Scope>) {
  assign_value("font", Stored::Font(font), scope);
}

/// a variant of `lookup_value` that casts the value into `Number`
pub fn lookup_number(key: &str) -> Option<Number> {
  match state!().lookup_value(key) {
    None | Some(Stored::None) => None,
    Some(v) => v.into(),
  }
}
/// a variant of `lookup_value` that casts the value into `Dimension`
pub fn lookup_dimension(key: &str) -> Option<Dimension> {
  match state!().lookup_value(key) {
    None | Some(Stored::None) => None,
    Some(v) => v.into(),
  }
}
/// a variant of `lookup_value` that only recognizes a `Stored::Glue`
pub fn lookup_glue(key: &str) -> Option<Glue> {
  match state!().lookup_value(key) {
    Some(Stored::Glue(v)) => Some(*v),
    None | Some(Stored::None) => None,
    Some(other) => panic!("State lookup expected Glue, found: {other:?}"),
  }
}
/// a variant of `lookup_value` that only recognizes a `Stored::Glue`
pub fn lookup_muglue(key: &str) -> Option<MuGlue> {
  match state!().lookup_value(key) {
    Some(Stored::MuGlue(v)) => Some(*v),
    None | Some(Stored::None) => None,
    Some(other) => panic!("State lookup expected MuGlue, found: {other:?}"),
  }
}
/// a variant of `lookup_value` that casts the response into `Tokens`
pub fn lookup_tokens(key: &str) -> Option<Tokens> {
  let state = state!();
  match state.lookup_value(key) {
    None | Some(Stored::None) => None,
    Some(Stored::Tokens(v)) => Some(v.clone()),
    Some(Stored::Token(v)) => Some(Tokens::new(vec![*v])),
    Some(Stored::String(sym)) => {
      let astr = arena::to_string(*sym);
      drop(state);
      Some(mouth::tokenize_internal(&astr))
    },
    Some(Stored::VecDequeStored(v)) => Stored::VecDequeStored(v.clone()).into(),
    _ => None,
  }
}
/// a variant of `lookup_value` that only recognizes a `Stored::Token`
pub fn lookup_token(key: &str) -> Option<Token> {
  match state!().lookup_value(key) {
    Some(Stored::Token(t)) => Some(*t),
    _ => None,
  }
}

pub fn lookup_alignment() -> Option<Digested> {
  // Can only be a token or definition; we want defns!
  // is this the right logic here? don't expand unless digesting?
  state!().lookup_value("Alignment").and_then(|v| {
    if let Stored::Digested(d) = v {
      if matches!(d.data(), DigestedData::Alignment(_)) {
        // for now clone the Digested object (approx. an Rc<_> clone)
        // instead of returning &Digested, to simplify lifetime checks
        Some(d.clone())
      } else {
        None
      }
    } else {
      None
    }
  })
}
pub fn assign_alignment(alignment: Alignment, scope: Option<Scope>) {
  assign_value("Alignment", alignment, scope);
}

pub fn assign_register(
  cs: &str,
  value: RegisterValue,
  scope: Option<Scope>,
  parameters: Vec<ArgWrap>,
) -> Result<()> {
  let cs = T_CS!(cs);
  let defn_opt = lookup_definition(&cs)?;
  if let Some(defn) = defn_opt {
    if defn.is_register() {
      defn.set_value(value, scope, parameters);
      return Ok(());
    }
  }
  Warn!(
    "expected",
    "register",
    format!("The control sequence '{cs}' is not a register")
  );
  Ok(())
}
pub fn lookup_register(cs: &str, parameters: Vec<ArgWrap>) -> Result<Option<RegisterValue>> {
  let cs = T_CS!(cs);
  Ok(if let Some(defn) = lookup_definition(&cs)? {
    if defn.is_register() {
      defn.value_of(parameters)
    } else {
      let message = s!("The control sequence '{}' is not a register", cs);
      Warn!("expected", "register", message);
      None
    }
  } else {
    // let message = s!("The control sequence '{}' is not defined", cs);
    // Warn!("expected", "register", message);
    None
  })
}

pub fn lookup_expandable(
  token: &Token,
  toplevel_opt: Option<bool>,
) -> Result<Option<Rc<dyn Definition>>> {
  let toplevel = toplevel_opt.unwrap_or(true); // Default, for full expansion, same as read_x_token
                                               // Can only be a token or definition; we want defns!
                                               // is this the right logic here? don't expand unless digesting?
  Ok(
    lookup_definition(token)?
      .filter(|defn| (*defn).is_expandable() && (toplevel || !(*defn).is_protected())),
  )
}

/// Whether token is affected by \noexpand
pub fn is_dont_expandable(token: &Token) -> bool {
  // Basically: a CS or Active token that is either not defined, or is expandable
  // (but not \let to a token)
  if token.get_catcode().is_active_or_cs() {
    let lookupname = token.text;
    if lookupname != *EMPTY_SYM {
      match state!().meaning.get(&lookupname) {
        Some(entry) => {
          if let Some(def) = entry.front() {
            // the expandable variants are allowed
            matches!(
              def,
              Stored::Expandable(_) | Stored::Conditional(_) | Stored::None
            )
          } else {
            // undefined is allowed too (this is *really* subtle -- took some debugging of
            // etoolbox) both an empty VDQ, a VDQ with an entry present but matching
            // Stored::Noney, OR a completely missing VDQ are allowed "undefined" cases, each of
            // which flagging as "true"
            true
          }
        },
        None => true,
      }
    } else {
      true
    }
  } else {
    false
  }
}

pub fn lookup_conditional(token: &Token) -> Option<ConditionalType> {
  let lookupname = token.get_executable_name();
  if lookupname.is_empty() {
    None
  } else if let Some(entry) = state!().meaning.get(&arena::pin(lookupname)) {
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

pub fn unshift_value<T: Into<Stored>>(key: &str, values: Vec<T>) {
  let values_iter = values.into_iter().map(Into::into);
  let key_sym = arena::pin(key);
  let mut state = state_mut!();
  if !state.value.contains_key(&key_sym) {
    state.assign_internal(
      TableName::Value,
      key_sym,
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    )
  }
  let receiver = state.value.get_mut(&key_sym).unwrap().front_mut();
  if let Some(&mut Stored::VecDequeStored(ref mut front)) = receiver {
    for value in values_iter.rev() {
      // preserving order unshift, as Perl's
      front.push_front(value)
    }
  } else {
    panic!(
      "unshift_value can only work on a Stored::VecDequeStored receiver. Instead, key {key:?} \
       got: {receiver:?}"
    );
  }
}

pub fn shift_value(key: &str) -> Result<Option<Stored>> {
  let key_sym = arena::pin(key);
  let mut state = state_mut!();
  if !state.value.contains_key(&key_sym) {
    state.assign_internal(
      TableName::Value,
      key_sym,
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    )
  }
  Ok(
    if let Some(&mut Stored::VecDequeStored(ref mut front)) =
      state.value.get_mut(&key_sym).unwrap().front_mut()
    {
      front.pop_front()
    } else {
      Error!(
        "State",
        "Stored",
        "BUG: Tried to shift_value from a non-vecdeque value key!"
      );
      None
    },
  )
}

pub fn assign_mapping<T: Into<Stored>>(map: &str, key: &str, value: Option<T>) {
  let map_sym = arena::pin(map);
  let mut state = state_mut!();
  if !state.value.contains_key(&map_sym) || state.value[&map_sym].is_empty() {
    state.assign_internal(
      TableName::Value,
      map_sym,
      Stored::HashStored(SymHashMap::default()),
      Some(Scope::Global),
    );
  }
  let map_store = state.value.get_mut(&map_sym).unwrap();
  // TODO: What is the right abstraction here? this is hacky
  let mut stub_hash = SymHashMap::default();
  let mapping = match *map_store.front_mut().unwrap() {
    Stored::HashStored(ref mut mapping) => mapping,
    _ => &mut stub_hash,
  };
  match value {
    None => mapping.remove(key),
    Some(v) => mapping.insert(key, v.into()),
  };
}

pub fn lookup_mapping(map: &str, key: &str) -> Option<Stored> {
  state!().lookup_mapping(map, key).cloned()
}

//======================================================================
/// Was `name` bound?  If  `frame` is given, check only whether it is bound in
/// that frame (0 is the topmost).
pub fn is_value_bound(key: &str, frame_opt: Option<usize>) -> bool {
  let key_sym = arena::pin(key);
  match frame_opt {
    Some(frame) => state!()
      .undo
      .get(frame)
      .as_ref()
      .unwrap()
      .table(TableName::Value)
      .contains_key(&key_sym),
    None => !state!()
      .value
      .get(&key_sym)
      .unwrap_or(&VecDeque::new())
      .is_empty(),
  }
}

//======================================================================
/// Lookup & assign a character's Catcode
pub fn lookup_catcode(c: char) -> Option<Catcode> {
  // speedup over variant with allocation
  // i.e. "let s = c.to_string();"
  let s = arena::pin_char(c);
  match state!().catcode.get(&s) {
    None => None,
    Some(cvec) => match cvec.front() {
      Some(Stored::Catcode(cc)) => Some(*cc),
      Some(_) => todo!(), // best to fail hard if we set a nonsence value
      _ => None,
    },
  }
}

/// assigns a Catcode for a given character
pub fn assign_catcode(key: char, value: Catcode, scope: Option<Scope>) {
  let s = arena::pin_char(key);
  state_mut!().assign_internal(TableName::Catcode, s, Stored::Catcode(value), scope);
}
/// like `lookup_catcode` but targets Mathcode and its table
pub fn lookup_mathcode(key: &str) -> Option<u16> {
  let key_sym = arena::pin(key);
  match state!().mathcode.get(&key_sym) {
    Some(c) => match c.front() {
      Some(Stored::Charcode(codeval)) => Some(*codeval),
      _ => None,
    },
    None => None,
  }
}
pub fn lookup_mathcode_sym(key_sym: &SymStr) -> Option<u16> {
  match state!().mathcode.get(key_sym) {
    Some(c) => match c.front() {
      Some(Stored::Charcode(codeval)) => Some(*codeval),
      _ => None,
    },
    None => None,
  }
}
/// like `assign_catcode` but targets Mathcode and its table
pub fn assign_mathcode<T: Into<u16>>(key: char, value: T, scope: Option<Scope>) {
  state_mut!().assign_internal(
    TableName::Mathcode,
    arena::pin_char(key),
    Stored::Charcode(value.into()),
    scope,
  );
}
/// like `lookup_catcode` but targets Sfcode and its table
pub fn lookup_sfcode(key: char) -> Option<u16> {
  match state!().sfcode.get(&arena::pin_char(key)) {
    Some(c) => match c.front() {
      Some(Stored::Charcode(codeval)) => Some(*codeval),
      _ => None,
    },
    None => None,
  }
}
/// like `assign_catcode` but targets Sfcode and its table
pub fn assign_sfcode<T: Into<u16>>(key: char, value: T, scope: Option<Scope>) {
  state_mut!().assign_internal(
    TableName::Sfcode,
    arena::pin_char(key),
    Stored::Charcode(value.into()),
    scope,
  );
}
/// like `lookup_catcode` but targets Lccode and its table
pub fn lookup_lccode(key: char) -> Option<u16> {
  match state!().lccode.get(&arena::pin_char(key)) {
    Some(c) => match c.front() {
      Some(Stored::Charcode(codeval)) => Some(*codeval),
      _ => None,
    },
    None => None,
  }
}
/// like `assign_catcode` but targets Lccode and its table
pub fn assign_lccode<T: Into<u16>, C: Into<char>>(key: C, value: T, scope: Option<Scope>) {
  let c: char = key.into();
  state_mut!().assign_internal(
    TableName::Lccode,
    arena::pin_char(c),
    Stored::Charcode(value.into()),
    scope,
  );
}
/// like `lookup_catcode` but targets Uccode and its table
pub fn lookup_uccode(key: char) -> Option<u16> {
  let mut tmp = [0u8; 4];
  let s = arena::pin(key.encode_utf8(&mut tmp));
  match state!().uccode.get(&s) {
    Some(c) => match c.front() {
      Some(Stored::Charcode(codeval)) => Some(*codeval),
      _ => None,
    },
    None => None,
  }
}
/// like `assign_catcode` but targets Uccode and its table
pub fn assign_uccode<T: Into<u16>, C: Into<char>>(key: C, value: T, scope: Option<Scope>) {
  let c: char = key.into();
  let mut tmp = [0u8; 4];
  let s = arena::pin(c.encode_utf8(&mut tmp));
  state_mut!().assign_internal(TableName::Uccode, s, Stored::Charcode(value.into()), scope);
}
/// like `lookup_catcode` but targets Delcode and its table
pub fn lookup_delcode(key: char) -> Option<u16> {
  let mut tmp = [0u8; 4];
  let s = arena::pin(key.encode_utf8(&mut tmp));
  match state!().delcode.get(&s) {
    Some(c) => match c.front() {
      Some(Stored::Charcode(codeval)) => Some(*codeval),
      _ => None,
    },
    None => None,
  }
}
/// like `assign_catcode` but targets Delcode and its table
pub fn assign_delcode<T: Into<u16>>(key: char, value: T, scope: Option<Scope>) {
  let mut tmp = [0u8; 4];
  let s = arena::pin(key.encode_utf8(&mut tmp));
  state_mut!().assign_internal(TableName::Delcode, s, Stored::Charcode(value.into()), scope);
}
/// Get the `Meaning' of a token.  For active control sequences
/// this may give the definition object (if defined) or another token (if \let) or undef
/// Any other token is returned as is.
pub fn lookup_meaning(token: &Token) -> Option<Stored> {
  if token.get_catcode().is_active_or_cs() && token.text != *EMPTY_SYM {
    match state!().meaning.get(&token.text) {
      Some(entry) => match entry.front() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other.clone()),
      },
      None => None,
    }
  } else {
    Some(Stored::Token(*token))
  }
}

/// like `lookup_value` but only recognizes `Stored::VecDequeStored`
pub fn lookup_vecdeque(key: &str) -> Option<VecDeque<Stored>> {
  match state!().lookup_value(key) {
    None | Some(Stored::None) => None,
    Some(v) => <Option<&VecDeque<Stored>>>::from(v).cloned(),
  }
}

pub fn with_vecdeque<R, FnR>(key: &str, caller: FnR) -> R
where FnR: FnOnce(Option<&VecDeque<Stored>>) -> R {
  caller(state!().lookup_vecdeque(key))
}

/// $meaning should be a definition (for defining active control sequences)
/// or another token, for \let
pub fn assign_meaning<T: Into<Stored>>(token: &Token, meaning: T, scope: Option<Scope>) {
  let meaning = meaning.into();
  // HACK!!!????
  // short-circuit guard to avoid e.g. T_MATH let to itself
  if let Stored::Token(ref mt) = meaning {
    if token == mt {
      return;
    }
  }
  let csname_sym = token.pin_cs_name();
  state_mut!().assign_internal(TableName::Meaning, csname_sym, meaning, scope);
}

// keep this in sync with `lookup_meaning`, it is copied over for optimization purposes
pub fn has_meaning(token: &Token) -> bool {
  if token.get_catcode().is_active_or_cs() && token.text != *EMPTY_SYM {
    match state!().meaning.get(&token.text) {
      Some(entry) => match entry.front() {
        None | Some(Stored::None) => false,
        Some(_) => true,
      },
      None => false,
    }
  } else {
    true
  }
}

/// used for expansion & various queries
/// Since we're not doing digestion here, we don't need to handle mathactive,
/// nor cs let to executable tokens
/// This returns a definition object, or undef
pub fn lookup_definition(key: &Token) -> Result<Option<Rc<dyn Definition>>> {
  Ok(
    if let Some(defs) = state!().lookup_definition_internal(key) {
      match defs.front() {
        Some(Stored::Conditional(entry)) => Some(entry.clone()),
        Some(Stored::Constructor(entry)) => Some(entry.clone()),
        Some(Stored::Expandable(entry)) => Some(entry.clone()),
        Some(Stored::MathPrimitive(entry)) => Some(entry.clone()),
        Some(Stored::Primitive(entry)) => Some(entry.clone()),
        Some(Stored::Register(entry)) => Some(entry.clone()),
        Some(Stored::None) | Some(Stored::Token(_)) | None => None,
        Some(v) => {
          let message = s!("in lookup_definition for {:?}. Value was: {:?}", key, v);
          Error!("unexpected", "value", message);
          None
        },
      }
    } else {
      None
    },
  )
}

/// Returns a definition as `Stored` so that one can call `.read_arguments`
///
/// This can't be specialized during compile-time over a trait object?
/// Instead we'll dispatch via `Stored` at runtime, to allow generic calls.
pub fn lookup_definition_stored(key: &Token) -> Result<Option<Stored>> {
  Ok(match state!().lookup_definition_internal(key) {
    Some(defs) => match defs.front() {
      // Still, good time to handle the Token case and catch weird storage errors
      Some(Stored::Conditional(entry)) => Some(Stored::Conditional(Rc::clone(entry))),
      Some(Stored::Constructor(entry)) => Some(Stored::Constructor(Rc::clone(entry))),
      Some(Stored::Expandable(entry)) => Some(Stored::Expandable(Rc::clone(entry))),
      Some(Stored::MathPrimitive(entry)) => Some(Stored::MathPrimitive(Rc::clone(entry))),
      Some(Stored::Primitive(entry)) => Some(Stored::Primitive(Rc::clone(entry))),
      Some(Stored::Register(entry)) => Some(Stored::Register(Rc::clone(entry))),
      Some(Stored::Token(entry)) => Some(Stored::Expandable(Rc::new(Expandable {
        cs: key.with_str(|k| T_CS!(k)),
        paramlist: None,
        expansion: (*entry).into(),
        ..Expandable::default()
      }))),
      Some(v) => {
        let message = s!("in lookup_definition for {:?}. Value was: {:?}", key, v);
        Error!("unexpected", "value", message);
        None
      },
      None => None,
    },
    _ => None,
  })
}

/// A specialized version of `lookup_definition` for registers, since we can't adequately perform
/// multi-dispatch when we have a "Self: Sized" for the Definition trait object.
pub fn lookup_register_definition(key: &Token) -> Option<Rc<Register>> {
  match state!().lookup_definition_internal(key) {
    Some(defs) => match defs.front() {
      Some(Stored::Register(entry)) => Some(Rc::clone(entry)),
      _ => None,
    },
    _ => None,
  }
}
/// Recognizes mathactive tokens in math mode and also looks for
/// cs that have been let to other `executable' tokens.
/// Returns a definition object, or a "self inserting" token.
/// Used for digestion.
pub fn lookup_digestable_definition(token: &Token) -> Option<Stored> {
  let cc = token.get_catcode();
  let t_sym = token.get_sym();
  let is_active_or_cs = cc.is_active_or_cs();
  let lookup_sym = if is_active_or_cs
    || ((cc == Catcode::LETTER || (cc == Catcode::OTHER))
      && lookup_bool("IN_MATH")
      && (lookup_mathcode_sym(&t_sym).unwrap_or(0) == 0x8000))
  {
    t_sym
  } else {
    arena::pin(cc.name())
  };
  // Debug!("Looking up digestable {:?}", lookupname);
  let state = state!();
  let entry_opt = state.meaning.get(&lookup_sym);
  if lookup_sym != *EMPTY_SYM && entry_opt.is_some() && !entry_opt.as_ref().unwrap().is_empty() {
    // Debug!("Found definition for: {:?}", lookupname);
    if let Some(entry) = entry_opt {
      if let Some(front) = entry.front() {
        if let Stored::Token(ref t) = front {
          if let Some(lookup_name) = t.get_executable_primitive_name() {
            let lookup_sym = arena::pin(lookup_name);
            if let Some(retry_entry) = state!().meaning.get(&lookup_sym) {
              // special case,
              // If a cs has been let to an executable token, lookup ITS defn.
              return retry_entry.front().cloned();
            }
          }
        }
        // if a regular definition, just return.
        return Some(front.clone());
      }
    }
  } else if is_active_or_cs {
    return None;
  }
  Some(token.into())
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
/// Starts a new level of grouping.
/// Note that this is lower level than C<\bgroup>;
pub fn push_frame() {
  // Easy: just push a new undo frame.
  state_mut!().undo.push_front(UndoFrame::default());
}
/// Ends the current level of grouping.
/// Note that this is lower level than `\egroup`;
pub fn pop_frame() -> Result<()> {
  let mut state = state_mut!();
  if state.undo.front().as_ref().unwrap().locked {
    fatal!(
      TargetUnexpected,
      Endgroup,
      "attempt to pop last locked stack frame"
    );
  // Fatal('unexpected', '<endgroup>', $self->getStomach,
  // "Attempt to pop last locked stack frame"); }
  } else {
    let popped_frame = state.undo.pop_front().unwrap();
    for table_name in TableName::variants() {
      let undo_table = popped_frame.table(*table_name);
      let state_table = state.table_mut(*table_name);
      for (key, undo_count) in undo_table.iter() {
        // Typically only 1 value to shift off the table, unless scopes have been activated.
        let named_table = state_table.get_mut(key).unwrap();
        for _ in 0..*undo_count {
          named_table.pop_front();
        }
      }
    }
  }
  Ok(())
}

/// Determine depth of group nesting.
///
/// nesting created by {,},\bgroup,\egroup,\begingroup,\endgroup
/// by counting all frames which are not Daemon frames (and thus don't possess _FRAME_LOCK_).
/// This may give incorrect results for some special environments (e.g. minipage)
pub fn get_frame_depth() -> usize {
  state!()
    .undo
    .iter()
    .filter(|frame| !frame.locked)
    .count()
    .saturating_sub(1)
}
/// begins a semiverbatim frame, neutralizing the usual + requested characters
pub fn begin_semiverbatim(extraspecials: Option<&[char]>) {
  // Is this a good/safe enough shorthand, or should we really be doing beginMode?
  push_frame();
  assign_value("MODE", "text", None);
  assign_value("IN_MATH", false, None);
  let mut all_specials: Vec<char> = Vec::new();
  if let Some(extra) = extraspecials {
    for special in extra {
      all_specials.push(*special);
    }
  }
  {
    if let Some(Stored::Chars(specials_store)) = state!().lookup_value("SPECIALS") {
      for special_char in &**specials_store {
        all_specials.push(*special_char);
      }
    }
  }

  for special_char in all_specials {
    assign_catcode(special_char, Catcode::OTHER, Some(Scope::Local));
  }
  // TODO:
  // self.assign_mathcode('\'' => 0x8000, Some(Scope::Local));
  // try to stay as ASCII as possible
  if let Some(ref current_font) = lookup_font() {
    let local_font = current_font.merge(fontmap!(encoding => "ASCII"));
    assign_font(Rc::new(local_font), Some(Scope::Local));
  }
}
/// end by just calling `pop_frame`
pub fn end_semiverbatim() -> Result<()> { pop_frame() }

//   #======================================================================

// sub pushDaemonFrame {
// ...  TODO
// }

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
/// Set one of the definition prefixes global, etc (only global matters!)
pub fn set_prefix(prefix: &str) { state_mut!().prefixes.insert(arena::pin(prefix), true); }
/// gets the current value of a named prefix
pub fn get_prefix(prefix: &str) -> bool { state!().get_prefix(prefix) }

/// clears the global prefixes
pub fn clear_prefixes() { state_mut!().prefixes = HashMap::default(); }

// #======================================================================
/// Activates all stashed definitions for the named scope. No-op if the scope is already active.
pub fn activate_scope(scope: SymStr) {
  let mut state = state_mut!();
  // do not re-activate if already active.
  if let Some(stash_active_entry) = state.stash_active.get(&scope) {
    if !stash_active_entry.is_empty() {
      return;
    }
  }

  state.assign_internal(
    TableName::StashActive,
    scope,
    Stored::Bool(true),
    Some(Scope::Local),
  );
  // Also, we need to take ownership of the stashed data, so that we can assign it.
  // TODO: Potential to optimize?
  // Also x2, we are using a shared "Stored" interface for all data that passes through
  // assign_internal, but that causes both uncertainty and overhead in the Stash table
  // specifically. TODO x2: Maybe a more ambitious refactor will separate out the Stash logic
  // and use "StashTable" directly instead of Stored::Stash(StashTable) ?

  let mut actions = Vec::new();

  if let Some(Some(Stored::Stash(defns))) = state.stash.get(&scope).map(|x| x.iter().next()) {
    for (table_name, key, value) in defns {
      // copy the values out from the stashed defns, so that Rust
      // is calm we are borrowing safely.

      actions.push((*table_name, key.to_owned(), value.clone()));
    }
  }
  // Here we ALWAYS push the stashed values into the table
  // since they may be popped off by deactivateScope
  for (table_name, key, value) in actions {
    let frame = &mut state.undo[0];
    let frame_table = frame.table_mut(table_name);
    let entry = frame_table.entry(key).or_insert(0);
    *entry += 1; // Note that this many values must be undone
    let key_table = state.table_mut(table_name).entry(key).or_default();
    key_table.push_front(value); // And push new binding.
  }
}

// Probably, in most cases, the assignments made by activateScope
// will be undone by egroup or popping frames.
// But they can also be undone explicitly

/// Removes any definitions that were associated with the named `scope`.
/// Normally not needed, since a scopes definitions are locally bound anyway.
pub fn deactivate_scope(scope: SymStr) {
  let mut state = state_mut!();
  let scope_exists = match state.stash_active.get(&scope) {
    None => false,
    Some(v) => !v.is_empty(),
  };
  if !scope_exists {
    return;
  }

  state.assign_internal(
    TableName::StashActive,
    scope,
    Stored::Bool(false),
    Some(Scope::Global),
  );

  let mut collected = Vec::new();
  if let Some(Some(Stored::Stash(defns))) = state.stash.get(&scope).map(|x| x.iter().next()) {
    for (table_name, key, value) in defns {
      collected.push((table_name.to_owned(), key.to_owned(), value.to_owned()));
    }
  }

  for (table_name, key, value) in collected {
    let front_is_value = if let Some(table_entry_peek) = state.table(table_name).get(&key) {
      if let Some(table_front) = table_entry_peek.front() {
        *table_front == value
      } else {
        false
      }
    } else {
      false
    };
    let table_entry = state.table_mut(table_name).entry(key).or_default();
    if front_is_value {
      // Here we're popping off the values pushed by activateScope
      // to (possibly) reveal a local assignment in the same frame, preceding activateScope.
      (*table_entry).pop_front();

      if let Some(frame) = state.undo.front_mut() {
        let frame_table = frame.table_mut(table_name);
        let frame_count = frame_table.entry(key).or_default();
        *frame_count -= 1;
      }
    } else {
      let message = arena::with(key, |key_str| {
        s!(
          "Unassigning wrong value for {} from table {} in deactivateScopevalue is {:?} but stack \
          is {:?}",
          key_str,
          table_name,
          value,
          table_entry
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
        )
      });
      arena::with(key, |key_str| Warn!("internal", key_str, message));
    }
  }
}
/// return all known named scopes
pub fn get_known_scopes() -> Vec<SymStr> { state!().stash.keys().copied().collect::<Vec<_>>() }
/// return the currently activated named scopes
pub fn get_active_scopes() -> Vec<SymStr> {
  state!().stash_active.keys().copied().collect::<Vec<_>>()
}

//======================================================================
// Units.
// Put here since it could concievably evolve to depend on the current font.
/// convert a unit name into a `f64` scaling factor over `sp`
pub fn convert_unit(unit_arg: &str) -> f64 {
  let unit = unit_arg.to_lowercase();
  // Eventually try to track font size?
  match unit.as_str() {
    "em" => lookup_font().unwrap().get_em_width() as f64,
    "ex" => lookup_font().unwrap().get_ex_height() as f64,
    "mu" => lookup_font().unwrap().get_mu_width() as f64,
    u => match UNITS.get(u) {
      Some(sp) => *sp,
      None => {
        let message = s!("Illegal unit of measure {:?}, assuming pt.", u);
        Warn!("expected", "<unit>", message);
        *UNITS.get("pt").unwrap()
      },
    },
  }
}

// ======================================================================

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

// TODO: Continue here -- need to diagnoze why the indirect model is not returning
// an intermediate "ltx:p" when asking for "#PCDATA" inside "ltx:_CaptureBlock_",
// instead getting an intermediate "ltx:para".

/// The indirect model includes all elements allowed as direct children,
/// and all descendents of a node that can be inserted after autoOpen'ing intermediate elements.
///
/// This model therefor includes information from the Schema, as well as
/// `auto_open` information that may be introduced in binding files.
// [Thus it should NOT be modifying the Model object, which may cover several documents in Daemon]
// `imodel[tag][child] => inter` means if in `tag`, to open `child`, we must first open `inter`
pub fn compute_indirect_model() -> IndirectModel {
  let mut imodel: IndirectModel = SymHashMap::default();
  // Determine any indirect paths to each descendent via an `autoOpen-able' tag.
  let mut openable: HashSet<SymStr> = HashSet::default();
  for tag in model::get_tags() {
    if let Some(x) = state!().tag_properties.get(&tag) {
      if let Some(true) = x.auto_open {
        openable.insert(tag);
      }
    }
  }

  for tag in model::get_tags() {
    let mut desc: SymHashMap<SymHashMap<usize>> = SymHashMap::default();
    compute_indirect_model_aux(tag, None, 1, &mut openable, &mut desc);
    let desc_keys: Vec<SymStr> = desc.keys().copied().collect();
    for kid in desc_keys {
      // Find best path to `kid`.
      let mut best = 0;
      let mut desc_kid_keys: Vec<SymStr> =
        desc.entry_sym(kid).or_default().keys().copied().collect();
      // TODO: why sort?
      // Update: it appears that "ltx:p" and "ltx:para" in ltx:_CaptureBlock_ is one reason!!!
      desc_kid_keys.sort_by(|a, b| arena::with2(*a, *b, |astr, bstr| astr.cmp(bstr)));
      for start in desc_kid_keys {
        if tag != kid && tag != start {
          let start_entry = {
            let kid_entry = desc.entry_sym(kid).or_default();
            *kid_entry.entry_sym(start.to_owned()).or_insert(0)
          };
          if start_entry > best {
            imodel
              .entry_sym(tag)
              .or_default()
              .insert_sym(kid, start.to_owned());
            {
              best = start_entry;
            }
          }
        }
      }
    }
  }
  // PATCHUP
  if model::is_permissive() {
    // !!! Alarm!!!
    imodel
      .entry("#Document")
      .or_default()
      .insert("#PCDATA", arena::pin_static("ltx:p"));
  }

  imodel
}

// Package helpers used in core need to be localized here -- as state methods
/// `Let` macro setter
pub fn let_i(token1: &Token, token2: &Token, scope: Option<Scope>) {
  let meaning =// if token2.get_dont_expand().is_some() {
  //   Stored::Token(token2.clone())
  // } else {
    lookup_meaning(token2)
      .unwrap_or(Stored::None);
  // };
  assign_meaning(token1, meaning, scope);
  after_assignment();
}
/// `XEquals` check for two token arguments
pub fn x_equals(token1: &Token, token2: &Token) -> bool {
  let def1_opt = lookup_meaning(token1); // # token, definition object or None
  let def2_opt = lookup_meaning(token2); // ditto
  match (def1_opt, def2_opt) {
    (Some(def1), Some(def2)) => def1 == def2, // If both have defns, must be same defn!
    (None, None) => true,                     // true if both undefined
    (..) => false,                            // False, if only one has 'meaning'
  }
}

/// simple id generator for a ligature
pub fn generate_ligature_id() -> usize {
  let id = 1 + lookup_int("autogen_ligature_id");
  assign_value("autogen_ligature_id", Stored::Int(id), Scope::Global);
  id as usize
}

/// run the accumulated directives from `\afterassignment`
pub fn after_assignment() {
  match remove_value("afterAssignment") {
    Some(Stored::Tokens(after)) => gullet::unread(after),
    Some(Stored::Token(after)) => gullet::unread_one(after),
    None | Some(Stored::None) => {},
    Some(other) => panic!("unexpected in after_assignment: {other:?}"),
  }
}

// Ported from Perl's "local" declarations

pub fn get_tag_property(tag: SymStr) -> TagOptions { state_mut!().ensure_tag_property(tag).clone() }
pub fn ensure_tag_property(tag: SymStr) { state_mut!().ensure_tag_property(tag); }

pub fn with_tag_property<R, FnR>(tag: SymStr, caller: FnR) -> R
where FnR: FnOnce(Option<&TagOptions>) -> R {
  caller(state!().tag_properties.get(&tag))
}
pub fn with_tag_property_mut<R, FnR>(tag: SymStr, caller: FnR) -> R
where FnR: FnOnce(&mut TagOptions) -> R {
  ensure_tag_property(tag);
  caller(state_mut!().tag_properties.get_mut(&tag).unwrap())
}

pub fn has_indirect_model() -> bool { state!().indirect_model.is_some() }
pub fn set_indirect_model(im: IndirectModel) {
  let mut state = state_mut!();
  state.indirect_model = Some(im);
}
pub fn get_nomathparse_flag() -> bool { state!().nomathparse }
pub fn set_nomathparse_flag(val: bool) {
  let mut state = state_mut!();
  state.nomathparse = val;
}

pub fn current_verbosity() -> i32 { state!().verbosity }

pub fn push_pending_resource(value: Resource) { state_mut!().pending_resources.push(value); }
pub fn take_pending_resources() -> Vec<Resource> {
  state_mut!().pending_resources.drain(..).collect()
}
pub fn reset_pending_resources() { state_mut!().pending_resources = Vec::new(); }
pub fn get_indirect_model_relationship(tag: SymStr, childtag: SymStr) -> Option<SymStr> {
  match state!().indirect_model.as_ref().unwrap().get_sym(&tag) {
    Some(sub_m) => sub_m.get_sym(&childtag).copied(),
    None => None,
  }
}

pub fn get_bindings_dispatch() -> Option<BindingDispatcher> { state!().bindings_dispatch.clone() }
pub fn get_extra_bindings_dispatch() -> Option<BindingDispatcher> {
  state!().extra_bindings_dispatch.clone()
}
pub fn set_bindings_dispatch(dispatcher: BindingDispatcher) {
  let mut state = state_mut!();
  state.bindings_dispatch = Some(dispatcher);
}
pub fn set_extra_bindings_dispatch(dispatcher: BindingDispatcher) {
  let mut state = state_mut!();
  state.extra_bindings_dispatch = Some(dispatcher);
}

pub fn get_search_paths() -> Vec<String> { state!().search_paths.iter().cloned().collect() }
pub fn with_search_paths<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&VecDeque<String>) -> R {
  caller(&state!().search_paths)
}
pub fn add_search_path(path: String) {
  let mut state = state_mut!();
  state.search_paths.push_back(path);
}
pub fn search_paths_push_front(path: String) {
  let mut state = state_mut!();
  state.search_paths.push_front(path);
}
pub fn has_search_paths() -> bool { !state!().search_paths.is_empty() }
pub fn get_graphics_paths() -> Vec<String> { state!().graphics_paths.iter().cloned().collect() }
pub fn graphics_paths_push_front(path: String) {
  let mut state = state_mut!();
  state.graphics_paths.push_front(path);
}

/// manage a (global) hash of values
pub fn with_mapping<R, FnR>(map: &str, key: &str, caller: FnR) -> R
where FnR: FnOnce(Option<&Stored>) -> R {
  let map_sym = arena::pin(map);
  caller(match state!().value.get(&map_sym) {
    None => None,
    Some(map_vec) => match map_vec.front() {
      Some(Stored::HashStored(h)) => h.get(key),
      _ => None,
    },
  })
}

pub fn with_mapping_sym<R, FnR>(map: SymStr, key: SymStr, caller: FnR) -> R
where FnR: FnOnce(Option<&Stored>) -> R {
  caller(match state!().value.get(&map) {
    None => None,
    Some(map_vec) => match map_vec.front() {
      Some(Stored::HashStored(h)) => h.get_sym(&key),
      _ => None,
    },
  })
}

pub fn with_mapping_keys<R, FnR>(map: &str, caller: FnR) -> R
where FnR: FnOnce(Vec<SymStr>) -> R {
  caller(state!().lookup_mapping_keys(map))
}

pub fn with_font_info<R, FnR>(key: &Token, caller: FnR) -> R
where FnR: FnOnce(Result<Option<&Stored>>) -> R {
  caller(state!().lookup_font_info(key))
}

pub fn get_input_encoding() -> Option<SymStr> { state!().input_encoding.as_ref().map(arena::pin) }
pub fn set_input_encoding(val: Option<String>) {
  let mut state = state_mut!();
  state.input_encoding = val;
}

pub fn with_stacked_values<R, FnR>(key: &str, caller: FnR) -> R
where FnR: FnOnce(Vec<&Stored>) -> R {
  caller(state!().lookup_stacked_values(key))
}

pub fn set_state(incoming_state: State) {
  let mut global_state = state_mut!();
  *global_state = incoming_state;
}
