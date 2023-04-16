use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::sync::{Arc, RwLock};
use string_interner::symbol::SymbolU32;

use crate::common::arena::{self, EMPTY_SYM, LTX_P_SYM, H_PCDATA_SYM, GLOBAL_DEFS_SYM, FONT_SYM};
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::glue::Glue;
use crate::common::model::{IndirectModel, Model};
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
pub use crate::common::store::Stored; // reexport for convenience
use crate::common::BindingDispatcher;
use crate::definition::argument::ArgWrap;
use crate::definition::conditional::{ConditionalType, IfFrame};
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::register::{Register, RegisterValue};
use crate::definition::Definition;
use crate::document::resource::Resource;
use crate::document::tag::TagOptions;
use crate::gullet::Gullet;
use crate::stomach::Stomach;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::util::pathname;

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

/// installation scope in the State tables
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
  /// globally visible, does not expire
  Global,
  /// globally visible, but expires at the end of the current group
  Local,
  /// a named scope - visible only when explicitly activated
  Named(String),
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
pub type AssignmentCount = HashMap<SymbolU32, usize>;
/// The `(table_name, key, value)` contents of a stored table of assignments
pub type StashTable = Vec<(TableName, SymbolU32, Stored)>;
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

/// There are tables for
///  catcode: keys are char;
///     Also, `math:char` =1 when `char` is active in math.
///  mathcode, sfcode, lccode, uccode, delcode : are similar to catcode but store
///    additional kinds codes per char (see TeX)
///  value: keys are anything (typically a string, though) and value is the value associated with it
///  meaning: The definition assocated with `key`, usually a control-sequence.
///  stash & stash_active: support named scopes
///      (see also activateScope & deactivateScope)
pub type Table = HashMap<SymbolU32, VecDeque<Stored>>;

/// The State efficiently maintain the bindings in a TeX-like fashion.
/// bindings associate data with keys (eg definitions with macro names)
/// and respect TeX grouping; that is, an assignment is only in effect
/// until the current group (opened by \bgroup) is closed (by \egroup).
// TODO: Maybe the right Rust metaphor here is to chunk State into a tuple of independent data:
// struct State(Gullet, Stomach, Model, StateTables, SessionState);
// maybe not...
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
  // Stateful runtime - data structures
  /// the schema-derived model used for the current document
  pub model: Model,
  prefixes: HashMap<SymbolU32, bool>, // ?
  pub tag_properties: HashMap<SymbolU32, TagOptions>,
  /// an optional indirect model for long-distance relationships
  pub indirect_model: Option<IndirectModel>,
  /// Document-related resources declared during core conversion, pending until XML is finalized
  pub pending_resources: Vec<Resource>,
  // Stateful runtime - simple fields
  // TODO: Maybe group these in a "SessionFlags" struct?
  //       we can then reset that if we reimplement a daemon app
  pub verbosity: i32,
  pub align_group_count: i32, // was $LaTeXML::ALIGN_STATE
  pub status_code: usize,
  pub unlocked: bool,
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
  /// TODO: marker for alignment
  pub reading_alignment: bool,
  // Local structures
  if_frames: Vec<Option<Arc<RwLock<IfFrame>>>>,
  smuggle_the: Vec<bool>,
  current_token: Vec<Token>,
  // TODO: We can make this a Vec<BindingDispatcher> if we want to accumulate more definitions
  /// A dispatcher routing to the compiled code of the main/official rtx bindings
  pub bindings_dispatch: Option<BindingDispatcher>,
  /// Auxiliary convenience -- extra dispatch
  pub extra_bindings_dispatch: Option<BindingDispatcher>,
  /// Circular dependency and global $STATE in Perl requires a bad
  /// style use of interior mutability...
  pub stomach: Arc<RwLock<Stomach>>,
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

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
      // Stateful runtime - data structures
      model: Model::default(),
      prefixes: HashMap::default(),
      tag_properties: HashMap::default(),
      indirect_model: None,
      pending_resources: Vec::new(),
      // Stateful runtime - simple fields
      verbosity: 0,
      status_code: 0,
      align_group_count: 0,
      unlocked: true,
      current_token: Vec::new(),
      if_frames: Vec::new(),
      input_encoding: None,
      // strict: false,
      // include_comments: true,
      search_paths: VecDeque::new(),
      graphics_paths: VecDeque::new(),
      // include_styles: false,
      nomathparse: false,
      smuggle_the: Vec::new(),
      reading_alignment: false,
      bindings_dispatch: None,
      extra_bindings_dispatch: None,
      // interiorly mutable
      stomach: Arc::new(RwLock::new(Stomach::default())),
    }
  }
}

pub static STY_STATE: Lazy<RwLock<State>> = Lazy::new(|| {
  RwLock::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Style),
    ..StateOptions::default()
  }))
});
pub static STD_STATE: Lazy<RwLock<State>> = Lazy::new(|| {
  RwLock::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  }))
});

/// State fields allowed for customization during construction
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
    specials_vdq.push_front(Stored::VecChar(vec!['^', '_', '~', '&', '$', '#', '\'']));
    value_table.insert(arena::pin("SPECIALS"), specials_vdq);

    let mut catcodes_typed: Table = HashMap::default();
    for (k, v) in catcodes {
      let mut vdq = VecDeque::new();
      vdq.push_front(Stored::Catcode(v));
      let mut tmp = [0u8; 3];
      let cat_key = arena::pin(k.encode_utf8(&mut tmp));
      catcodes_typed.insert(cat_key, vdq);
    }

    // Basic defaults
    let model = match options.model {
      None => Model::default(),
      Some(m) => m,
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
      model,
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
    // TODO: should these be *fields* in state, or really as in Perl - globally assigned values?
    state.assign_value(
      "DOCUMENT_ID",
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

  fn assign_internal(
    &mut self,
    table_name: TableName,
    key: SymbolU32,
    value: Stored,
    mut scope_opt: Option<Scope>,
  ) {
    // hotcode lookupDefinition for \globaldefs,
    // since this is called extremely often and should be highly standardized
    if let Some(globaldefs) = GLOBAL_DEFS_SYM.with(|sym| self.value.get(sym)) {
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

        let table_entry = state_table
          .entry(key)
          .or_insert_with(VecDeque::new);
        table_entry.push_front(value);
      },
      Scope::Local => {
        // Again, split the logic as 1) bookkeeping in undo, then 2) operations in state tables
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
        // 2. State table mutable logic
        let state_table = self.table_mut(table_name);
        let defs = state_table
          .entry(key)
          .or_insert_with(VecDeque::new);
        if is_replace {
          // 2.1. Replace the value, i.e. remove existing one
          defs.pop_front();
        }
        // 2.2 Add new value
        defs.push_front(value);
      },
      Scope::Named(scope_name) => {
        let scope_sym = arena::pin(scope_name);
        // initialize stash if empty
        let needs_init = match self.stash.get(&scope_sym) {
          None => true,
          Some(v) => v.is_empty(),
        };
        if needs_init {
          self.assign_internal(
            TableName::Stash,
            scope_sym,
            Stored::Stash(Vec::new()),
            Some(Scope::Global),
          );
        }
        if let Some(Stored::Stash(ref mut stash)) =
          self.stash.get_mut(&scope_sym).as_mut().unwrap().get_mut(0)
        {
          stash.push((table_name, key, value.clone()));
        }
        let has_active = match self.stash_active.get(&scope_sym) {
          None => false,
          Some(v) => !v.is_empty(),
        };
        if has_active {
          self.assign_internal(table_name, key, value, Some(Scope::Local));
        }
      },
    }
  }

  //======================================================================
  /// fetches a Stored value at the given key, from the Value table
  #[inline(always)]
  pub fn lookup_value(&self, key: &str) -> Option<&Stored> {
    match self.value.get(&arena::pin(key)) {
      None => None,
      Some(vvec) => match vvec.front() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other),
      },
    }
  }
  #[inline(always)]
  pub fn lookup_value_sym(&self, key: &SymbolU32) -> Option<&Stored> {
    match self.value.get(key) {
      None => None,
      Some(vvec) => match vvec.front() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other),
      },
    }
  }

  /// mutably borrows a Stored value at the given key, from the Value table
  pub fn lookup_value_mut<'lv>(&'lv mut self, key: &'lv str) -> Option<&mut Stored> {
    match self.value.get_mut(&arena::pin(key)) {
      None => None,
      Some(vvec) => match vvec.front_mut() {
        None | Some(Stored::None) => None,
        Some(other) => Some(other),
      },
    }
  }

  /// inline lookup_value after which globally assign an empty Tokens() to undo
  pub fn remove_value<'lv>(&'lv mut self, key: &'lv str) -> Option<Stored> {
    let key_sym = arena::pin(key);
    match self.value.get_mut(&key_sym) {
      None => None,
      Some(vvec) => match vvec.front_mut() {
        None | Some(&mut Stored::None) => Option::None,
        Some(found) => Some(std::mem::take(found))
      },
    }
  }

  /// Replaces the value in question with `Stored::None` (see `checkin_value` for returning it)
  pub fn checkout_value(&mut self, key: &str) -> Option<Stored> {
    match self.value.get_mut(&arena::pin(key)) {
      None => None,
      Some(vvec) => vvec
        .front_mut()
        .map(std::mem::take),
    }
  }
  /// Returns a value into its `Stored::None` placeholder (see `checkout_value` for taking it)
  pub fn checkin_value(&mut self, key: &str, value: Stored) {
    match self.value.get_mut(&arena::pin(key)) {
      None => unimplemented!(),
      Some(vvec) => match vvec.front_mut() {
        None => unimplemented!(),
        Some(found) => {
          match found {
            Stored::None => std::mem::replace(found, value),
            _ => panic!("checkin_value should only be called after checkout_value"),
          };
        },
      },
    }
  }
  /// assigns a `Stored` value at the given key and scope
  pub fn assign_value<'av, T: Into<Stored>, S: Into<Option<Scope>>>(
    &'av mut self,
    key: &'av str,
    value: T,
    scope: S,
  ) {
    let value = value.into();
    let scope = scope.into();
    let key_sym = arena::pin(key);
    self.assign_internal(TableName::Value, key_sym, value, scope);
  }
  /// assigns a `Stored` value at the given (arena ticket!) key and scope
  pub fn assign_value_sym<T: Into<Stored>, S: Into<Option<Scope>>>(
    &mut self,
    key: SymbolU32,
    value: T,
    scope: S,
  ) {
    let value = value.into();
    let scope = scope.into();
    self.assign_internal(TableName::Value, key, value, scope);
  }

  /// manage a (global) list of values
  pub fn push_value<T: Into<Stored>>(&mut self, key: &str, value: T) {
    let key_sym = arena::pin(key);
    let value = value.into();
    if !self.value.contains_key(&key_sym) {
      self.assign_internal(
        TableName::Value,
        key_sym,
        Stored::VecDequeStored(VecDeque::new()),
        Some(Scope::Global),
      );
    }
    match self.value.get_mut(&key_sym).unwrap().front_mut() {
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
        Error!("state", "Stored", None, self, message);
      },
    }
  }

  /// pops the last value in a named `Stored::VecDequeStored` queue, if any
  pub fn pop_value(&mut self, key: &str) -> Option<Stored> {
    let key_sym = arena::pin(key);
    if !self.value.contains_key(&key_sym) {
      self.assign_internal(
        TableName::Value,
        key_sym,
        Stored::VecDequeStored(VecDeque::new()),
        Some(Scope::Global),
      );
    }
    if let Some(&mut Stored::VecDequeStored(ref mut front)) =
      self.value.get_mut(&key_sym).unwrap().front_mut()
    {
      front.pop_back()
    } else {
      Error!(
        "state",
        "Stored",
        None,
        self,
        "BUG: Tried to pop_value from a non-vecdeque value key!"
      );
      None
    }
  }

  /// Check if the Value table contains a given key
  pub fn has_value(&self, key: &str) -> bool {
    let key_sym = arena::pin(key);
    match self.value.get(&key_sym) {
      None => false,
      Some(list) => match list.front() {
        None => false,
        Some(v) => !matches!(v, &Stored::None),
      },
    }
  }

  /// Pushes Tokens into a `Stored::Tokens` value when defined,
  /// or assigns when new.
  pub fn push_tokens(&mut self, key: &str, value: Tokens) {
    match self.lookup_value_mut(key) {
      Some(Stored::Tokens(ref mut tks)) => tks.unlist_mut().extend(value.unlist()),
      None | Some(Stored::None) => self.assign_value(key, Stored::Tokens(value), None),
      Some(other) => panic!("Can only push_tokens into a Stored::Tokens, but got {other:?}"),
    }
  }

  /// A bit of Perl "existence as truth" semantics mixed in with proper boolean lookup
  #[inline(always)]
  pub fn lookup_bool(&self, key: &str) -> bool {
    match self.lookup_value(key) {
      None => false,
      Some(v) => v.into(),
    }
  }

  /// like `lookup_value`, but casts the entry into a String (empty if None)
  #[inline]
  pub fn lookup_string(&self, key: &str) -> String {
    match self.lookup_value(key) {
      None => String::new(),
      Some(v) => v.into(),
    }
  }
  /// like `lookup_value` but only recognizes Int, Bool and Number variants of Stored (default: 0)
  #[inline]
  pub fn lookup_int(&self, key: &str) -> i64 {
    match self.lookup_value(key) {
      Some(Stored::Int(i)) => *i,
      Some(Stored::Bool(true)) => 1, // this is Perl's boolean -> integer semantics
      Some(Stored::Number(n)) => n.value_of(),
      _ => 0,
    }
  }
  #[inline]
  pub fn lookup_vec_string<'lvec>(&'lvec self, key: &'lvec str) -> Option<&Vec<String>> {
    match self.lookup_value(key) {
      Some(Stored::VecString(v)) => Some(v),
      _ => None,
    }
  }
  #[inline]
  pub fn lookup_vecdeque<'lvdq>(&'lvdq self, key: &'lvdq str) -> Option<&VecDeque<Stored>> {
    match self.lookup_value(key) {
      None | Some(Stored::None) => None,
      Some(v) => v.into(),
    }
  }
  #[inline]
  pub fn remove_vecdeque<'lvdq>(&'lvdq mut self, key: &'lvdq str) -> Option<VecDeque<Stored>> {
    match self.remove_value(key) {
      Some(Stored::VecDequeStored(v)) => Some(v),
      _ => None,
    }
  }
  /// convenience method to lookup the current value at the "font" key
  #[inline(always)]
  pub fn lookup_font(&self) -> Option<Arc<Font>> {
    match self.lookup_value_sym(&FONT_SYM.with(|sym| *sym)) {
      None | Some(Stored::None) => None,
      Some(f) => f.into(),
    }
  }
  /// convenience method to lookup the current value at the "mathfont" key
  #[inline]
  pub fn lookup_mathfont(&self) -> Option<Arc<Font>> {
    match self.lookup_value("mathfont") {
      None | Some(Stored::None) => None,
      Some(v) => v.into(),
    }
  }

  /// a convenience method to globally asign a `Font` to the "font" key
  #[inline(always)]
  pub fn assign_font(&mut self, font: Arc<Font>, scope: Option<Scope>) {
    self.assign_value("font", Stored::Font(font), scope);
  }

  /// a variant of `lookup_value` that casts the value into `Number`
  #[inline(always)]
  pub fn lookup_number(&self, key: &str) -> Option<Number> {
    match self.lookup_value(key) {
      None | Some(Stored::None) => None,
      Some(v) => v.into(),
    }
  }
  /// a variant of `lookup_value` that casts the value into `Dimension`
  #[inline(always)]
  pub fn lookup_dimension(&self, key: &str) -> Option<Dimension> {
    match self.lookup_value(key) {
      None | Some(Stored::None) => None,
      Some(v) => v.into(),
    }
  }
  /// a variant of `lookup_value` that only recognizes a `Stored::Glue`
  #[inline]
  pub fn lookup_glue(&self, key: &str) -> Option<Glue> {
    match self.lookup_value(key) {
      Some(Stored::Glue(v)) => Some(*v),
      None | Some(Stored::None) => None,
      Some(other) => panic!("state lookup expected Glue, found: {other:?}"),
    }
  }
  /// a variant of `lookup_value` that only recognizes a `Stored::Glue`
  #[inline]
  pub fn lookup_muglue(&self, key: &str) -> Option<MuGlue> {
    match self.lookup_value(key) {
      Some(Stored::MuGlue(v)) => Some(*v),
      None | Some(Stored::None) => None,
      Some(other) => panic!("state lookup expected MuGlue, found: {other:?}"),
    }
  }
  /// a variant of `lookup_value` that casts the response into `Tokens`
  #[inline]
  pub fn lookup_tokens(&self, key: &str) -> Option<Tokens> {
    match self.lookup_value(key) {
      None | Some(Stored::None) => None,
      Some(v) => v.into(),
    }
  }
  /// a variant of `lookup_value` that only recognizes a `Stored::Token`
  #[inline]
  pub fn lookup_token(&self, key: &str) -> Option<&Token> {
    match self.lookup_value(key) {
      Some(Stored::Token(t)) => Some(t),
      _ => None,
    }
  }
  pub fn lookup_register(&mut self, cs: &str, parameters: Vec<ArgWrap>) -> Option<RegisterValue> {
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
  #[inline]
  pub fn lookup_expandable(&self, token: &Token, toplevel: bool) -> Option<Arc<dyn Definition>> {
    // Can only be a token or definition; we want defns!
    // is this the right logic here? don't expand unless digesting?
    self
      .lookup_definition(token)
      .filter(|defn| (*defn).is_expandable() && (toplevel || !(*defn).is_protected()))
  }

  /// Whether token must be wrapped as dont_expand
  pub fn is_dont_expandable(&self, token: &Token) -> bool {
    // Basically: a CS or Active token that is either not defined, or is expandable
    // (but not \let to a token)
    if token.get_catcode().is_active_or_cs() {
      let lookupname = token.text;
      if lookupname != EMPTY_SYM.with(|sym| *sym) {
        match self.meaning.get(&lookupname) {
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

  pub fn lookup_conditional(&self, token: &Token) -> Option<ConditionalType> {
    let lookupname = token.get_executable_name();
    if lookupname.is_empty() {
      None
    } else if let Some(entry) = self.meaning.get(&arena::pin(lookupname)) {
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
    let values_iter = values.into_iter().map(Into::into);
    let key_sym = arena::pin(key);
    if !self.value.contains_key(&key_sym) {
      self.assign_internal(
        TableName::Value,
        key_sym,
        Stored::VecDequeStored(VecDeque::new()),
        Some(Scope::Global),
      )
    }
    let receiver = self.value.get_mut(&key_sym).unwrap().front_mut();
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

  pub fn shift_value(&mut self, key: &str) -> Option<Stored> {
    let key_sym = arena::pin(key);
    if !self.value.contains_key(&key_sym) {
      self.assign_internal(
        TableName::Value,
        key_sym,
        Stored::VecDequeStored(VecDeque::new()),
        Some(Scope::Global),
      )
    }
    if let Some(&mut Stored::VecDequeStored(ref mut front)) =
      self.value.get_mut(&key_sym).unwrap().front_mut()
    {
      front.pop_front()
    } else {
      Error!(
        "state",
        "Stored",
        None,
        self,
        "BUG: Tried to shift_value from a non-vecdeque value key!"
      );
      None
    }
  }

  /// manage a (global) hash of values
  #[inline]
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

  pub fn assign_mapping<T: Into<Stored>>(&mut self, map: &str, key: &str, value: Option<T>) {
    let map_sym = arena::pin(map);
    if !self.value.contains_key(&map_sym) || self.value[&map_sym].is_empty() {
      self.assign_internal(
        TableName::Value,
        map_sym,
        Stored::HashStored(HashMap::default()),
        Some(Scope::Global),
      );
    }
    let map_store = self.value.get_mut(&map_sym).unwrap();
    // TODO: What is the right abstraction here? this is hacky
    let mut stub_hash = HashMap::default();
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
    let map_sym = arena::pin(map);
    match self.value.get(&map_sym) {
      None => Vec::new(),
      Some(map_vec) => match map_vec.front() {
        Some(Stored::HashStored(h)) => h.keys().map(String::as_str).collect(),
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

  //======================================================================
  /// Was `name` bound?  If  `frame` is given, check only whether it is bound in
  /// that frame (0 is the topmost).
  pub fn is_value_bound(&self, key: &str, frame_opt: Option<usize>) -> bool {
    let key_sym = arena::pin(key);
    match frame_opt {
      Some(frame) => self
        .undo
        .get(frame)
        .as_ref()
        .unwrap()
        .table(TableName::Value)
        .contains_key(&key_sym),
      None => !self.value.get(&key_sym).unwrap_or(&VecDeque::new()).is_empty(),
    }
  }

  //======================================================================
  /// Lookup & assign a character's Catcode
  #[inline(always)]
  pub fn lookup_catcode(&self, c: char) -> Option<Catcode> {
    // speedup over variant with allocation
    // i.e. "let s = c.to_string();"
    let mut tmp = [0u8; 3];
    let s = arena::pin(c.encode_utf8(&mut tmp));
    match self.catcode.get(&s) {
      None => None,
      Some(cvec) => match cvec.front() {
        Some(Stored::Catcode(cc)) => Some(*cc),
        Some(_) => unimplemented!(), // best to fail hard if we set a nonsence value
        _ => None,
      },
    }
  }

  /// assigns a Catcode for a given character
  #[inline]
  pub fn assign_catcode(&mut self, key: char, value: Catcode, scope: Option<Scope>) {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    self.assign_internal(
      TableName::Catcode,
      s,
      Stored::Catcode(value),
      scope,
    );
  }
  /// like `lookup_catcode` but targets Mathcode and its table
  pub fn lookup_mathcode(&self, key: &str) -> Option<u16> {
    let key_sym = arena::pin(key);
    match self.mathcode.get(&key_sym) {
      Some(c) => match c.front() {
        Some(Stored::Charcode(codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  pub fn lookup_mathcode_sym(&self, key_sym: &SymbolU32) -> Option<u16> {
    match self.mathcode.get(key_sym) {
      Some(c) => match c.front() {
        Some(Stored::Charcode(codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  /// like `assign_catcode` but targets Mathcode and its table
  #[inline]
  pub fn assign_mathcode<T: Into<u16>>(
    &mut self,
    key: char,
    value: T,
    scope: Option<Scope>,
  ) {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    self.assign_internal(
      TableName::Mathcode,
      s,
      Stored::Charcode(value.into()),
      scope,
    );
  }
  /// like `lookup_catcode` but targets Sfcode and its table
  #[inline]
  pub fn lookup_sfcode(&self, key: char) -> Option<u16> {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    match self.sfcode.get(&s) {
      Some(c) => match c.front() {
        Some(Stored::Charcode(codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  /// like `assign_catcode` but targets Sfcode and its table
  #[inline]
  pub fn assign_sfcode<T: Into<u16>>(
    &mut self,
    key: char,
    value: T,
    scope: Option<Scope>,
  ) {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    self.assign_internal(
      TableName::Sfcode,
      s,
      Stored::Charcode(value.into()),
      scope,
    );
  }
  /// like `lookup_catcode` but targets Lccode and its table
  #[inline]
  pub fn lookup_lccode(&self, key: char) -> Option<u16> {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    match self.lccode.get(&s) {
      Some(c) => match c.front() {
        Some(Stored::Charcode(codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  /// like `assign_catcode` but targets Lccode and its table
  #[inline]
  pub fn assign_lccode<T: Into<u16>,C: Into<char>>(
    &mut self,
    key: C,
    value: T,
    scope: Option<Scope>,
  ) {
    let c : char = key.into();
    let mut tmp = [0u8; 3];
    let s = arena::pin(c.encode_utf8(&mut tmp));
    self.assign_internal(
      TableName::Lccode,
      s,
      Stored::Charcode(value.into()),
      scope,
    );
  }
  /// like `lookup_catcode` but targets Uccode and its table
  #[inline]
  pub fn lookup_uccode(&self, key: char) -> Option<u16> {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    match self.uccode.get(&s) {
      Some(c) => match c.front() {
        Some(Stored::Charcode(codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  /// like `assign_catcode` but targets Uccode and its table
  #[inline]
  pub fn assign_uccode<T: Into<u16>,C: Into<char>>(
    &mut self,
    key: C,
    value: T,
    scope: Option<Scope>
  ) {
    let c : char = key.into();
    let mut tmp = [0u8; 3];
    let s = arena::pin(c.encode_utf8(&mut tmp));
    self.assign_internal(
      TableName::Uccode,
      s,
      Stored::Charcode(value.into()),
      scope,
    );
  }
  /// like `lookup_catcode` but targets Delcode and its table
  #[inline]
  pub fn lookup_delcode(&self, key: char) -> Option<u16> {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    match self.delcode.get(&s) {
      Some(c) => match c.front() {
        Some(Stored::Charcode(codeval)) => Some(*codeval),
        _ => None,
      },
      None => None,
    }
  }
  /// like `assign_catcode` but targets Delcode and its table
  #[inline]
  pub fn assign_delcode<T: Into<u16>>(
    &mut self,
    key: char,
    value: T,
    scope: Option<Scope>
  ) {
    let mut tmp = [0u8; 3];
    let s = arena::pin(key.encode_utf8(&mut tmp));
    self.assign_internal(
      TableName::Delcode,
      s,
      Stored::Charcode(value.into()),
      scope,
    );
  }
  /// Get the `Meaning' of a token.  For active control sequence's
  /// this may give the definition object (if defined) or another token (if \let) or undef
  /// Any other token is returned as is.
  pub fn lookup_meaning(&self, token: &Token) -> Option<Cow<Stored>> {
    if token.get_catcode().is_active_or_cs()
      && !token.has_smuggled()
      && token.text != EMPTY_SYM.with(|sym| *sym)
    {
      match self.meaning.get(&token.text) {
        Some(entry) => match entry.front() {
          None | Some(Stored::None) => None,
          Some(other) => Some(Cow::Borrowed(other)),
        },
        None => None,
      }
    } else {
      Some(Cow::Owned(Stored::Token(token.clone())))
    }
  }

  /// $meaning should be a definition (for defining active control sequences)
  /// or another token, for \let
  pub fn assign_meaning<T: Into<Stored>>(
    &mut self,
    token: &Token,
    meaning: T,
    scope: Option<Scope>,
  ) {
    let meaning = meaning.into();
    // HACK!!!????
    // short-circuit guard to avoid e.g. T_MATH let to itself
    if let Stored::Token(ref mt) = meaning {
      if token == mt {
        return;
      }
    }
    let csname_sym = token.pin_cs_name();
    self.assign_internal(TableName::Meaning, csname_sym, meaning, scope);
  }

  fn lookup_definition_internal<'def>(&'def self, key: &'def Token) -> Option<&VecDeque<Stored>> {
    let cc = key.get_catcode();
    let name = key.get_sym();
    let lookupname: Option<SymbolU32> = if (cc == Catcode::ACTIVE) || (cc == Catcode::CS) {
      if name == EMPTY_SYM.with(|sym| *sym) {
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

  /// used for expansion & various queries
  /// Since we're not doing digestion here, we don't need to handle mathactive,
  /// nor cs let to executable tokens
  /// This returns a definition object, or undef
  pub fn lookup_definition<'def>(&'def self, key: &'def Token) -> Option<Arc<dyn Definition>> {
    if let Some(defs) = self.lookup_definition_internal(key) {
      match defs.front() {
        Some(Stored::Conditional(entry)) => Some(entry.clone()),
        Some(Stored::Constructor(entry)) => Some(entry.clone()),
        Some(Stored::Expandable(entry)) => Some(entry.clone()),
        Some(Stored::MathPrimitive(entry)) => Some(entry.clone()),
        Some(Stored::Primitive(entry)) => Some(entry.clone()),
        Some(Stored::Register(entry)) => Some(entry.clone()),
        // TODO: Is this take on reframing a Token definition as an Expandable acceptable?
        //      Does it have unintended side-effects? Are we missing useful code paths that
        // specifically deal with a Token      in Gullet, etc?
        Some(Stored::Token(entry)) => Some(Arc::new(Expandable {
          cs: key.as_cs(),
          paramlist: None,
          expansion: entry.clone().into(),
          ..Expandable::default()
        })),
        Some(Stored::None) | None => None,
        Some(v) => {
          let message = s!("in lookup_definition for {:?}. Value was: {:?}", key, v);
          Error!("unexpected", "value", None, self, message);
          None
        },
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
        Some(Stored::Conditional(entry)) => Some(Stored::Conditional(Arc::clone(entry))),
        Some(Stored::Constructor(entry)) => Some(Stored::Constructor(Arc::clone(entry))),
        Some(Stored::Expandable(entry)) => Some(Stored::Expandable(Arc::clone(entry))),
        Some(Stored::MathPrimitive(entry)) => Some(Stored::MathPrimitive(Arc::clone(entry))),
        Some(Stored::Primitive(entry)) => Some(Stored::Primitive(Arc::clone(entry))),
        Some(Stored::Register(entry)) => Some(Stored::Register(Arc::clone(entry))),
        Some(Stored::Token(entry)) => Some(Stored::Expandable(Arc::new(Expandable {
          cs: key.with_str(|k| T_CS!(k)),
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
  pub fn lookup_register_definition(&self, key: &Token) -> Option<Arc<Register>> {
    match self.lookup_definition_internal(key) {
      Some(defs) => match defs.front() {
        Some(Stored::Register(entry)) => Some(Arc::clone(entry)),
        _ => None,
      },
      _ => None,
    }
  }
  /// Recognizes mathactive tokens in math mode and also looks for
  /// cs that have been let to other `executable' tokens.
  /// Returns a definition object, or a "self inserting" token.
  /// Used for digestion.
  pub fn lookup_digestable_definition<'def>(&'def mut self, token: &'def Token) -> Option<Stored> {
    let cc = token.get_catcode();
    let t_sym = token.get_sym();
    let is_active_or_cs = cc.is_active_or_cs();
    let lookup_sym = if is_active_or_cs
      || ((cc == Catcode::LETTER || (cc == Catcode::OTHER))
        && self.lookup_bool("IN_MATH")
        && (self.lookup_mathcode_sym(&t_sym).unwrap_or(0) == 0x8000))
    {
      t_sym
    } else {
      arena::pin(cc.name())
    };
    // Debug!("Looking up digestable {:?}", lookupname);
    let entry_opt = self.meaning.get(&lookup_sym);
    if lookup_sym != EMPTY_SYM.with(|sym| *sym) && entry_opt.is_some() && !entry_opt.as_ref().unwrap().is_empty() {
      // Debug!("Found definition for: {:?}", lookupname);
      if let Some(entry) = entry_opt {
        if let Some(front) = entry.front() {
          if let Stored::Token(ref t) = front {
            let lookup_sym = if t.has_smuggled() {
              arena::pin_static("\\relax")
            } else {
              arena::pin(t.get_executable_primitive_name().unwrap())
            };
            if let Some(retry_entry) = self.meaning.get(&lookup_sym) {
              // special case,
              // If a cs has been let to an executable token, lookup ITS defn.
              return retry_entry.front().cloned();
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

  /// And a shorthand for installing definitions
  pub fn install_definition<T: Into<Stored>>(&mut self, definition: T, scope: Option<Scope>) {
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
    let cs = token.with_cs_name(|name| name.to_owned());
    // info!("-- installing definition for: {:?}", token);

    let cs_locked = s!("{}:locked", cs);
    // TODO, .is_none() should be a real false check
    let is_cs_locked = self.lookup_bool(&cs_locked);
    let is_state_unlocked = self.lookup_bool("UNLOCKED");

    if is_cs_locked && !is_state_unlocked {
      if let Some(Stored::String(s)) = self.lookup_value("SOURCEFILE") {
        // report if the redefinition seems to come from document source
        if ((s == "Anonymous String") || TEX_OR_BIB_EXT_RE.is_match(s))
          && (!s.ends_with(CODE_TEX_EXT))
        {
          //  info("ignore", cs, self.get_stomach(), "Ignoring redefinition of $cs");
        }
        return;
      }
    }
    self.assign_internal(TableName::Meaning, arena::pin(cs), definition, scope);
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
  #[inline]
  pub fn push_frame(&mut self) {
    // Easy: just push a new undo frame.
    self.undo.push_front(UndoFrame::default());
  }
  /// Ends the current level of grouping.
  /// Note that this is lower level than `\egroup`;
  pub fn pop_frame(&mut self) -> Result<()> {
    if self.undo.front().as_ref().unwrap().locked {
      fatal!(
        TargetUnexpected,
        Endgroup,
        "attempt to pop last locked stack frame"
      );
    // Fatal('unexpected', '<endgroup>', $self->getStomach,
    // "Attempt to pop last locked stack frame"); }
    } else {
      let popped_frame = self.undo.pop_front().unwrap();
      for table_name in TableName::variants() {
        let undo_table = popped_frame.table(*table_name);
        let state_table = self.table_mut(*table_name);
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

  /// Determine depth of group nesting created by {,},\bgroup,\egroup,\begingroup,\endgroup
  /// by counting all frames which are not Daemon frames (and thus don't possess _FRAME_LOCK_).
  /// This may give incorrect results for some special environments (e.g. minipage)
  #[inline]
  pub fn get_frame_depth(&self) -> usize {
    self.undo.iter().filter(|frame| !frame.locked).count() - 1
  }
  /// begins a semiverbatim frame, neutralizing the usual + requested characters
  pub fn begin_semiverbatim(&mut self, extraspecials: Option<&[char]>) {
    // Is this a good/safe enough shorthand, or should we really be doing beginMode?
    self.push_frame();
    self.assign_value("MODE", Stored::String(s!("text")), None);
    self.assign_value("IN_MATH", false, None);
    let mut all_specials: Vec<char> = Vec::new();
    if let Some(extra) = extraspecials {
      for special in extra {
        all_specials.push(*special);
      }
    }
    if let Some(Stored::VecChar(specials_store)) = self.lookup_value("SPECIALS") {
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
    if let Some(ref current_font) = self.lookup_font() {
      let local_font = current_font.merge(fontmap!(encoding => "ASCII"));
      self.assign_font(Arc::new(local_font), Some(Scope::Local));
    }
  }
  /// end by just calling `pop_frame`
  #[inline]
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
  /// Set one of the definition prefixes global, etc (only global matters!)
  #[inline(always)]
  pub fn set_prefix(&mut self, prefix: &str) { self.prefixes.insert(arena::pin(prefix), true); }
  /// gets the current value of a named prefix
  #[inline(always)]
  pub fn get_prefix(&self, prefix: &str) -> bool {
    match self.prefixes.get(&arena::pin(prefix)) {
      Some(b) => *b,
      _ => false,
    }
  }
  /// clears the global prefixes
  #[inline(always)]
  pub fn clear_prefixes(&mut self) { self.prefixes = HashMap::default(); }

  // #======================================================================
  ///
  pub fn activate_scope(&mut self, scope: &str) {
    // do not re-activate if already active.
    let scope_sym = arena::pin(scope);
    if let Some(stash_active_entry) = self.stash_active.get(&scope_sym) {
      if !stash_active_entry.is_empty() {
        return;
      }
    }

    self.assign_internal(
      TableName::StashActive,
      scope_sym,
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

    if let Some(Some(Stored::Stash(defns))) = self.stash.get(&scope_sym)
    .map(|x| x.iter().next()) {
      for (table_name, key, value) in defns {
        // copy the values out from the stashed defns, so that Rust
        // is calm we are borrowing safely.

        actions.push((*table_name, key.to_owned(), value.clone()));
      }
    }
    // Here we ALWAYS push the stashed values into the table
    // since they may be popped off by deactivateScope
    for (table_name, key, value) in actions {
      let frame = &mut self.undo[0];
      let frame_table = frame.table_mut(table_name);
      let entry = frame_table.entry(key).or_insert(0);
      *entry += 1; // Note that this many values must be undone
      let key_table = self
        .table_mut(table_name)
        .entry(key)
        .or_insert_with(VecDeque::new);
      key_table.push_front(value); // And push new binding.
    }
  }

  // Probably, in most cases, the assignments made by activateScope
  // will be undone by egroup or popping frames.
  // But they can also be undone explicitly

  /// Removes any definitions that were associated with the named `scope`.
  /// Normally not needed, since a scopes definitions are locally bound anyway.
  pub fn deactivate_scope(&mut self, scope: &str) {
    let scope_sym = arena::pin(scope);
    let scope_exists = match self.stash_active.get(&scope_sym) {
      None => false,
      Some(v) => !v.is_empty(),
    };
    if !scope_exists {
      return;
    }

    self.assign_internal(
      TableName::StashActive,
      scope_sym,
      Stored::Bool(false),
      Some(Scope::Global),
    );

    let mut collected = Vec::new();
    if let Some(Some(Stored::Stash(defns))) = self.stash.get(&scope_sym).map(|x| x.iter().next()) {
      for (table_name, key, value) in defns {
        collected.push((table_name.to_owned(), key.to_owned(), value.to_owned()));
      }
    }

    for (table_name, key, value) in collected {
      let front_is_value = if let Some(table_entry_peek) = self.table(table_name).get(&key) {
        if let Some(table_front) = table_entry_peek.front() {
          *table_front == value
        } else {
          false
        }
      } else {
        false
      };
      let table_entry = self.table_mut(table_name).entry(key).or_default();
      if front_is_value {
        // Here we're popping off the values pushed by activateScope
        // to (possibly) reveal a local assignment in the same frame, preceding activateScope.
        (*table_entry).pop_front();

        if let Some(frame) = self.undo.front_mut() {
          let frame_table = frame.table_mut(table_name);
          let frame_count = frame_table.entry(key).or_default();
          *frame_count -= 1;
        }
      } else {
        let message = arena::with(key, |key_str| s!(
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
        ));
        let stomach = self.stomach.read().unwrap();
        arena::with(key, |key_str|
          Warn!("internal", key_str, stomach, self, message));
      }
    }
  }
  /// return all known named scopes
  pub fn get_known_scopes(&self) -> Vec<SymbolU32> {
    self.stash.keys().copied().collect::<Vec<_>>()
  }
  /// return the currently activated named scopes
  pub fn get_active_scopes(&self) -> Vec<SymbolU32> {
    self.stash_active.keys()
      .copied().collect::<Vec<_>>()
  }

  //======================================================================
  // Units.
  // Put here since it could concievably evolve to depend on the current font.
  /// convert a unit name into a `f64` scaling factor over `sp`
  pub fn convert_unit(&self, unit_arg: &str) -> f64 {
    let unit = unit_arg.to_lowercase();
    // Eventually try to track font size?
    match unit.as_str() {
      "em" => self.lookup_font().unwrap().get_em_width() as f64,
      "ex" => self.lookup_font().unwrap().get_ex_height() as f64,
      "mu" => self.lookup_font().unwrap().get_mu_width() as f64,
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
  /// TODO
  pub fn note_status(&self, _category: &str, _what: &str) {
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
  /// and all descendents of a node that can be inserted after
  /// auto_open-ing intermediate elements.
  /// This model therefor includes information from the Schema, as well as
  /// auto_open information that may be introduced in binding files.
  // [Thus it should NOT be modifying the Model object, which may cover several documents]
  pub fn compute_indirect_model(&mut self) -> IndirectModel {
    let mut imodel: IndirectModel = HashMap::default();
    // Determine any indirect paths to each descendent via an `autoOpen-able' tag.
    let mut openable: HashSet<SymbolU32> = HashSet::default();
    for tag in self.model.get_sym_tags() {
      if let Some(x) = self.tag_properties.get(&tag) {
        if let Some(true) = x.auto_open {
          openable.insert(tag);
        }
      }
    }

    for tag in self.model.get_sym_tags() {
      let mut desc: HashMap<SymbolU32, HashMap<SymbolU32, usize>> = HashMap::default();
      {
        self.compute_indirect_model_aux(tag, None, 1, &mut openable, &mut desc);
      }

      let desc_keys: Vec<SymbolU32> = desc.keys().copied().collect();
      for kid in desc_keys {
        let mut best = 0; // Find best path to $kid.
        let desc_kid_keys: Vec<SymbolU32> = desc
          .entry(kid)
          .or_insert_with(HashMap::default)
          .keys()
          .copied()
          .collect();
        // desc_kid_keys.sort(); // TODO: why sort?
        for start in desc_kid_keys {
          let start_entry = {
            let kid_entry = desc.entry(kid).or_insert_with(HashMap::default);
            *kid_entry.entry(start.to_owned()).or_insert(0)
          };
          if start_entry > best {
            imodel
              .entry(tag)
              .or_insert_with(HashMap::default)
              .insert(kid, start.to_owned());
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
        .entry(arena::pin("#Document"))
        .or_insert_with(HashMap::default)
        .insert(H_PCDATA_SYM.with(|sym| *sym), LTX_P_SYM.with(|sym| *sym));
    }

    imodel
  }

  fn compute_indirect_model_aux(
    &mut self,
    tag: SymbolU32,
    start_opt: Option<SymbolU32>,
    desirability: usize,
    openable: &mut HashSet<SymbolU32>,
    desc: &mut HashMap<SymbolU32, HashMap<SymbolU32, usize>>,
  ) {
    let start = match start_opt {
      Some(s) => s,
      None => EMPTY_SYM.with(|sym| *sym),
    };

    // A bit tricky here, we need to release the state.model borrow immediately, which is why we
    // move ownership of the tag strings into the tag_contents vector.
    // That leads to a bunch of .clone()s later one, but stays close to the original algorithm
    let tag_contents: Vec<SymbolU32> = self.model.get_tag_contents(&tag);

    for kid in tag_contents {
      if desc
        .entry(kid)
        .or_insert_with(HashMap::default)
        .contains_key(&start)
      {
        continue;
      } // Already solved

      if start != EMPTY_SYM.with(|sym| *sym) {
        desc
          .entry(kid)
          .or_insert_with(HashMap::default)
          .insert(start, desirability);
      }

      if kid != H_PCDATA_SYM.with(|sym| *sym) && openable.contains(&kid) {
        let inner = if start != EMPTY_SYM.with(|sym| *sym) { start } else { kid };

        self.compute_indirect_model_aux(kid, Some(inner), desirability, openable, desc);
      }
    }
  }

  /// Initialize various stomach parameters, preload, etc.
  pub fn initialize_stomach(&mut self) {
    self.assign_value("MODE", String::from("text"), Some(Scope::Global));
    self.assign_value("IN_MATH", false, Some(Scope::Global));
    self.assign_value("PRESERVE_NEWLINES", Stored::Int(1), Some(Scope::Global));
    self.assign_value(
      "afterGroup",
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    );
    self.assign_value("afterAssignment", Stored::None, Some(Scope::Global)); // undef ???
    self.assign_value(
      "groupInitiator",
      String::from("Initialization"),
      Some(Scope::Global),
    );
    // Setup default fonts.
    self.assign_value("font", Font::text_default(), Some(Scope::Global));
    self.assign_value("mathfont", Font::math_default(), Some(Scope::Global));
  }

  // Package helpers used in core need to be localized here -- as State methods
  /// `Let` macro setter
  pub fn let_i(
    &mut self,
    token1: &Token,
    token2: Token,
    scope: Option<Scope>,
    gullet: &mut Gullet,
  ) {
    let meaning = if token2.get_dont_expand().is_some() {
      Cow::Owned(Stored::Token(token2))
    } else {
      self
        .lookup_meaning(&token2)
        .unwrap_or(Cow::Owned(Stored::None))
    };
    self.assign_meaning(token1, meaning.into_owned(), scope);
    self.after_assignment(gullet);
  }
  /// `XEquals` check for two token arguments
  pub fn x_equals(&self, token1: &Token, token2: &Token) -> bool {
    let def1_opt = self.lookup_meaning(token1); // # token, definition object or None
    let def2_opt = self.lookup_meaning(token2); // ditto
    match (def1_opt, def2_opt) {
      (Some(def1), Some(def2)) => *def1 == *def2, // If both have defns, must be same defn!
      (None, None) => true,                     // true if both undefined
      (_, _) => false,                          // False, if only one has 'meaning'
    }
  }

  /// Generate a stub definition for an undefined control-sequence,
  /// along with appropriate error messge.
  pub fn generate_error_stub(&mut self, caller: &mut Gullet, token: &Token) -> Result<Token> {
    let cs = token.with_cs_name(ToString::to_string);
    self.note_status("undefined", &cs); // TODO: Undefined:cs
                                       // To minimize chatter, go ahead and define it...
    if cs.starts_with("\\if") {
      // Apparently an \ifsomething ???
      let name = cs.replace("\\if", "");
      Error!(
        "undefined",
        token,
        caller,
        self,
        s!(
          "The token {} is not defined. Defining it now as with \\newif",
          token.stringify()
        )
      );
      self.install_definition(
        Expandable::new(
          T_CS!(s!("\\{}true", name)),
          None,
          s!("\\let{}\\iftrue", cs),
          None,
          self,
        ),
        Some(Scope::Global),
      );
      self.install_definition(
        Expandable::new(
          T_CS!(s!("\\{}false", name)),
          None,
          s!("\\let{}\\iffalse", cs),
          None,
          self,
        ),
        Some(Scope::Global),
      );
      self.let_i(token, T_CS!("\\iffalse"), Some(Scope::Global), caller);
    } else {
      Error!(
        "undefined",
        token,
        caller,
        self,
        s!(
          "The token {} is not defined. Defining it now as <ltx:ERROR/>",
          token.stringify()
        )
      );
      let owned_cs = cs.to_owned();
      self.install_definition(
        Constructor {
          cs: token.clone(),
          replacement: Some(Arc::new(move |document, _args, _props, i_state| {
            document.make_error("undefined", &owned_cs, i_state)
          })),
          ..Constructor::default()
        },
        //TODO: sizer => "X"),
        Some(Scope::Global),
      );
    }
    Ok(token.clone())
  }

  /// simple id generator for a ligature
  pub fn generate_ligature_id(&mut self) -> usize {
    let id = 1 + self.lookup_int("autogen_ligature_id");
    self.assign_value("autogen_ligature_id", Stored::Int(id), Scope::Global);
    id as usize
  }

  /// run the accumulated directives from `\afterassignment`
  pub fn after_assignment(&mut self, gullet: &mut Gullet) {
    match self.remove_value("afterAssignment") {
      Some(Stored::Tokens(after)) => gullet.unread(after),
      Some(Stored::Token(after)) => gullet.unread_one(after),
      None | Some(Stored::None) => {},
      Some(other) => panic!("unexpected in after_assignment: {other:?}"),
    }
  }

  // Ported from Perl's "local" declarations

  /// sets a (originally Perl-local) `IfFrame` that needs to be manually expired.
  pub fn set_ifframe(&mut self, if_frame: Option<Arc<RwLock<IfFrame>>>) {
    self.if_frames.push(if_frame);
  }

  /// retrieves the most recent (originally Perl-local) `IfFrame`
  pub fn get_ifframe(&self) -> Option<Arc<RwLock<IfFrame>>> {
    match self.if_frames.last() {
      Some(Some(frame)) => Some(Arc::clone(frame)),
      _ => None,
    }
  }
  /// expires the most recent (originally Perl-local) `IfFrame`
  pub fn expire_ifframe(&mut self) { self.if_frames.pop(); }
  /// set special (localized) flag for "\the smuggling mode"; useful for expanded definitions
  pub fn set_smuggle_the(&mut self, smuggle_the: bool) { self.smuggle_the.push(smuggle_the); }
  /// get special (localized) flag for "\the smuggling mode"; useful for expanded definitions
  pub fn get_smuggle_the(&self) -> bool {
    match self.smuggle_the.last() {
      Some(v) => *v,
      _ => false,
    }
  }
  /// expire special (localized) flag for "\the smuggling mode"; useful for expanded definitions
  pub fn expire_smuggle_the(&mut self) { self.smuggle_the.pop(); }
  /// sets the (localized) current token. see `Stomach::invoke_token`
  pub fn set_current_token(&mut self, token: Token) { self.current_token.push(token); }
  /// expires the most recent (localized) current token.
  pub fn expire_current_token(&mut self) { self.current_token.pop(); }
  /// gets the (localized) current token
  pub fn get_current_token(&self) -> Option<&Token> { self.current_token.last() }
}
