use std::{
  borrow::Cow,
  cell::RefCell,
  collections::VecDeque,
  fmt::{self, Display},
  rc::Rc,
};

use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

// expose Perl-style local assignments from state
pub use crate::common::local_assignments::*;
pub use crate::common::store::Stored; // reexport for convenience
use crate::{
  Digested, DigestedData,
  alignment::Alignment,
  common::{
    BindingDispatcher, LabelMappingHook,
    arena::{self, SymHashMap, SymStr},
    dimension::Dimension,
    error::*,
    font::Font,
    glue::Glue,
    model::{self, IndirectModel, Model, compute_indirect_model_aux},
    muglue::MuGlue,
    number::Number,
    numeric_ops::{NumericOps, UNITY},
  },
  definition::{
    Definition, ExpansionBody,
    argument::ArgWrap,
    conditional::ConditionalType,
    constructor::Constructor,
    expandable::{self, Expandable},
    register::{Register, RegisterValue},
  },
  document::{resource::Resource, tag::TagOptions},
  gullet, mouth, pin,
  token::{Catcode, Token},
  tokens::Tokens,
  util::pathname,
};

static CODE_TEX_EXT: &str = ".code.tex";

/// regex for *.tex and *.bib
static TEX_OR_BIB_EXT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(tex|bib)$").unwrap());
/// Used in conversion to scaled points.
///
/// These are the float `sp`-per-unit ratios, kept for *display / scaling*
/// consumers (pgf/graphics/hyperref divide by them). For dimension
/// **construction** use [`convert_unit_ratio`] + `numeric_ops::fixpoint_unit`
/// instead — exact integer arithmetic, bit-faithful to TeX (issue #127). Each
/// value here equals `65536·num/den` of the matching `convert_unit_ratio` entry.
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    write!(f, "{}", match self {
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
    })
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
  locked:       bool,
  meaning:      AssignmentCount,
  value:        AssignmentCount,
  catcode:      AssignmentCount,
  mathcode:     AssignmentCount,
  sfcode:       AssignmentCount,
  lccode:       AssignmentCount,
  uccode:       AssignmentCount,
  delcode:      AssignmentCount,
  stash:        AssignmentCount,
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
  value:                       Table,
  /// The definition assocated with a key, usually a control-sequence.
  meaning:                     Table,
  stash:                       Table,
  stash_active:                Table,
  catcode:                     Table,
  mathcode:                    Table,
  sfcode:                      Table,
  lccode:                      Table,
  uccode:                      Table,
  delcode:                     Table,
  // Table bookkeeping
  undo:                        VecDeque<UndoFrame>,
  // stateful runtime - data structures
  /// the schema-derived model used for the current document
  prefixes:                    HashMap<SymStr, bool>, // ?
  pub tag_properties:          HashMap<SymStr, TagOptions>,
  /// an optional indirect model for long-distance relationships
  pub indirect_model:          Option<IndirectModel>,
  /// Document-related resources declared during core conversion, pending until XML is finalized
  pub pending_resources:       Vec<Resource>,
  // stateful runtime - simple fields
  // TODO: Maybe group these in a "SessionFlags" struct?
  //       we can then reset that if we reimplement a daemon app
  pub verbosity:               i32,
  pub input_encoding:          Option<String>,
  // strict: bool,
  // include_comments: bool,
  /// current paths to search for TeX inputs
  pub search_paths:            VecDeque<String>,
  /// current paths to search for graphics
  pub graphics_paths:          VecDeque<String>,
  // include_styles: bool,
  /// flag to disable math parsing
  pub nomathparse:             bool,
  /// flag enabling source-locator (`--source-map`) tracking + emission.
  /// Off by default; gates BOTH the per-token start capture and the
  /// per-element `data-sourcepos` stamping so a normal conversion pays
  /// nothing. See `docs/SOURCE_PROVENANCE.md`.
  pub source_map:              bool,
  /// Document-level `sources` table for the source-map feature: ordered
  /// list of source files seen, index = the integer `tag` emitted in
  /// `data-sourcepos` (Source-Map-v3 `sources` style — never an inlined
  /// path). Populated lazily via `source_tag()` only when `source_map` is
  /// on. See `docs/SOURCE_PROVENANCE.md` §0.1.
  pub source_table:            Vec<SymStr>,
  /// Read-log of every *named* source opened through `Mouth::create`
  /// (file paths and cached-content names; literal/anonymous mouths are
  /// not named, so not recorded). Distinct from `source_table`, which
  /// is populated lazily at *document-construction* time and filters to
  /// user sources — this log is complete and available right after a
  /// digest, which the LSP server's warm-cache dependency snapshot
  /// relies on (`lsp_server/overlay.rs::warmup_dep_snapshot`).
  pub opened_sources:          HashSet<SymStr>,
  // TODO: We can make this a Vec<BindingDispatcher> if we want to accumulate more definitions
  /// A dispatcher routing to the compiled code of the in-distro latexml bindings
  pub bindings_dispatch:       Option<BindingDispatcher>,
  /// Auxiliary convenience -- extra dispatch
  pub extra_bindings_dispatch: Option<BindingDispatcher>,
  /// All `(name, ext)` pairs for compile-time bindings the dispatchers can
  /// load, stacked one slice per registered dispatcher. Populated at
  /// startup by each binding crate via `add_binding_names`, so both
  /// `latexml_package` and `latexml_contrib` contribute their classes/
  /// styles/defs/pools to the fallback pool. Consumed by:
  /// - `find_file(notex=true)` to resolve compile-time bindings without touching the filesystem.
  /// - `load_class`'s Perl-parity prefix-match fallback (Package.pm L2702-2706) via the
  ///   `get_class_binding_names()` filtered view.
  pub binding_names:           Vec<&'static [(&'static str, &'static str)]>,
  /// Perl: LABEL_MAPPING_HOOK — closure mapping (label, counter, norefnum) -> (refnum, id)
  pub label_mapping_hook:      Option<LabelMappingHook>,
}
// SAFETY: `State` holds `Rc`/`RefCell`/`libxml::tree::Node` (!Send). Marked
// Send so callers can build it on one thread and then transition to another
// thread before any use. After first use, State MUST NOT cross thread
// boundaries (all `use_*_state()` helpers use a `#[thread_local]` switcher).
// Violating this contract would race libxml2's reference counts → UAF/UB.
// State is deliberately NOT Sync: no two threads may alias the same State.
unsafe impl Send for State {}

impl Default for State {
  fn default() -> Self {
    let top_frame = UndoFrame {
      locked: true,
      ..UndoFrame::default()
    };
    let mut undo_vdq = VecDeque::new();
    undo_vdq.push_front(top_frame);

    State {
      // Tables — pre-size the two largest to absorb dump load. The
      // `meaning` table receives 109,863 entries from latex.dump
      // alone, so without pre-sizing it doubles 5+ times during
      // dump load (each rehash is O(N)). `value` receives several
      // thousand register/state-key entries through the lifecycle.
      // Effective capacity (FxHashMap, 0.875 load factor): 131072 → ~115k.
      value:                   HashMap::with_capacity_and_hasher(8_192, Default::default()),
      meaning:                 HashMap::with_capacity_and_hasher(131_072, Default::default()),
      stash:                   HashMap::default(),
      stash_active:            HashMap::default(),
      // Char-keyed tables: ASCII alphabet + a smattering of high-codepoint
      // entries get installed (textcomp + ts1enc.dfu populate ~200-300
      // entries each). Pre-size to 512 to skip the 8→16→…→256→512
      // doubling chain on startup.
      catcode:                 HashMap::with_capacity_and_hasher(512, Default::default()),
      mathcode:                HashMap::with_capacity_and_hasher(512, Default::default()),
      sfcode:                  HashMap::with_capacity_and_hasher(512, Default::default()),
      lccode:                  HashMap::with_capacity_and_hasher(512, Default::default()),
      uccode:                  HashMap::with_capacity_and_hasher(512, Default::default()),
      delcode:                 HashMap::with_capacity_and_hasher(512, Default::default()),
      // Table bookkeeping
      undo:                    undo_vdq,
      // stateful runtime - data structures
      prefixes:                HashMap::default(),
      tag_properties:          HashMap::default(),
      indirect_model:          None,
      pending_resources:       Vec::new(),
      // stateful runtime - simple fields
      verbosity:               0,
      input_encoding:          None,
      // strict: false,
      // include_comments: true,
      search_paths:            VecDeque::new(),
      graphics_paths:          VecDeque::new(),
      // include_styles: false,
      nomathparse:             false,
      source_map:              false,
      source_table:            Vec::new(),
      opened_sources:          HashSet::default(),
      bindings_dispatch:       None,
      extra_bindings_dispatch: None,
      binding_names:           Vec::new(),
      label_mapping_hook:      None,
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

/// Eagerly initialize this thread's `STD_STATE`/`STY_STATE` catcode-regime
/// templates. They are accessed mid-conversion on catcode switches
/// (`\makeatletter`, verbatim, …); each one's `Lazy` initializer runs
/// `State::new`, which interns via the arena. Forcing them at conversion
/// entry — AFTER [`arena::force_init`](crate::common::arena::force_init) —
/// keeps them from initializing re-entrantly mid-expansion, the macOS
/// `#[thread_local]` hazard behind issue #217. (The active `STATE` is
/// already forced by `set_state` in `Core::new`.) No behavioral change on
/// Linux.
pub(crate) fn force_init() {
  Lazy::force(&STD_STATE);
  Lazy::force(&STY_STATE);
}

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
  pub model:            Option<Model>,
  pub verbosity:        Option<i32>,
  pub strict:           Option<bool>,
  pub include_comments: Option<bool>,
  pub include_styles:   Option<bool>,
  pub nomathparse:      Option<bool>,
  pub source_map:       Option<bool>,
  pub documentid:       Option<String>,
  pub search_paths:     Option<Vec<String>>,
  pub graphics_paths:   Option<Vec<String>>,
  pub catcodes:         Option<Catcodes>,
  pub input_encoding:   Option<String>,
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
        // NUL (`\^^@`, U+0000): Perl LaTeXML's default is catcode 12 (OTHER),
        // NOT the TeXbook's 9 (IGNORE). We follow Perl (ground truth) so that
        // `\^^@`/`` `^^@ `` reads code 0 (TeXbook 9 would *drop* the NUL token,
        // making `` `^^@ `` skip to the next token — `\relax` etc. — and return
        // a bogus code; xint's `\romannumeral`&&@` expansion idiom needs 0).
        // Real-world bbl files (e.g. astro-ph0004127's spie4012-01a.bbl) carry
        // stray NULs from BibTeX `\"u`-mangling; as OTHER they become harmless
        // literal chars (stripped at XML serialization), matching Perl —
        // crucially NOT ESCAPE, so no bogus `\uninger`-style CS forms. An
        // explicit `\catcode`^^Q=9` (user/package) is still honored; only the
        // *default* changes.
        catcodes.insert('\0', OTHER);
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
    // INCLUDE_COMMENTS: Perl defaults to true (Core.pm L143).
    // T_COMMENT tokens are now properly converted to XML comment nodes
    // via Document::insert_comment using raw libxml2 FFI.
    // Note: Only set when explicitly specified, because STY_STATE/STD_STATE
    // use default options and state rotation (swap) would overwrite the
    // main state's INCLUDE_COMMENTS value.
    let include_comments = options.include_comments;
    // let include_styles = options.include_styles.unwrap_or(false);
    let nomathparse = options.nomathparse.unwrap_or(false);
    let source_map = options.source_map.unwrap_or(false);

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
      source_map,
      ..State::default()
    };
    // INITEX-equivalent defaults — mirror Perl `State.pm:128-137`.
    // Sets letter/digit mathcodes (class 7, family 1 for letters / 0 for digits),
    // upper/lowercase mappings, and sfcode=999 for uppercase letters. Without
    // these, dump-load path leaves letter mathcodes unset (plain.dump.txt only
    // captures the 57 plain.tex OVERRIDES), so `\cal abc` math falls through
    // the text path and loses meaning/role attributes. NODUMP path used to set
    // these via plain_base.rs L17-41 only — not Perl-faithful since INITEX
    // owns these (TeXbook ch.17 p309). Setting them here makes both paths
    // consistent and matches Perl's State::new behaviour.
    for c in b'0'..=b'9' {
      state.assign_internal(
        TableName::Mathcode,
        arena::pin_char(c as char),
        Stored::Charcode(0x7000 + c as u16),
        None,
      );
    }
    for c in b'a'..=b'z' {
      let big = c - 32;
      state.assign_internal(
        TableName::Mathcode,
        arena::pin_char(c as char),
        Stored::Charcode(0x7100 + c as u16),
        None,
      );
      state.assign_internal(
        TableName::Mathcode,
        arena::pin_char(big as char),
        Stored::Charcode(0x7100 + big as u16),
        None,
      );
      state.assign_internal(
        TableName::Uccode,
        arena::pin_char(c as char),
        Stored::Charcode(big as u16),
        None,
      );
      state.assign_internal(
        TableName::Lccode,
        arena::pin_char(big as char),
        Stored::Charcode(c as u16),
        None,
      );
      state.assign_internal(
        TableName::Sfcode,
        arena::pin_char(big as char),
        Stored::Charcode(999),
        None,
      );
    }
    // TODO: should these be *fields* in state or really as in Perl - globally assigned values?
    state.assign_value(
      "DOCUMENTID",
      options.documentid.unwrap_or_default(),
      Some(Scope::Global),
    );
    // Perl Core.pm L143: assignValue(INCLUDE_COMMENTS => ..., 'global')
    // Only set when explicitly specified (not for STY_STATE/STD_STATE defaults)
    if let Some(ic) = include_comments {
      state.assign_value("INCLUDE_COMMENTS", ic, Some(Scope::Global));
    }
    // Perl Core.pm L47: INCLUDE_PATH_PIS (default true) — emit searchpath PIs
    state.assign_value("INCLUDE_PATH_PIS", true, Some(Scope::Global));
    // Perl Core.pm L43: STRICT (default false)
    if let Some(strict) = options.strict {
      state.assign_value("STRICT", strict, Some(Scope::Global));
    }
    // Perl Core.pm L62: NOMATHPARSE
    state.assign_value("NOMATHPARSE", nomathparse, Some(Scope::Global));
    // Perl Core.pm L61: PERL_INPUT_ENCODING (default utf-8)
    let enc = state.input_encoding.as_deref().unwrap_or("utf-8");
    state.assign_value(
      "PERL_INPUT_ENCODING",
      Stored::String(arena::pin(enc)),
      Some(Scope::Global),
    );

    // Perl Core.pm L53: $state->assignValue(GRAPHICSPATHS => [map {…} @{$opts{graphicspaths}}])
    // Mirror with a VecDequeStored of String entries; subsequent push/unshift
    // operations in `\graphicspath`, `\svgpath`, and Core.pm-equivalent source
    // directory prepends will append/prepend to this same list.
    if !state.graphics_paths.is_empty() {
      let vdq: VecDeque<Stored> = state
        .graphics_paths
        .iter()
        .map(|p| Stored::String(arena::pin(p)))
        .collect();
      state.assign_internal(
        TableName::Value,
        arena::pin("GRAPHICSPATHS"),
        Stored::VecDequeStored(vdq),
        Some(Scope::Global),
      );
    }

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

  /// Perl DumpFile equivalent: Take a snapshot of the current state.
  /// Returns a HashMap mapping (table_name, key) → Stored value.
  /// Only captures the front (current) value of each key.
  /// Used before processing latex.ltx to diff what changed.
  pub fn snapshot(&self) -> rustc_hash::FxHashMap<(TableName, SymStr), Stored> {
    let tables = [
      TableName::Value,
      TableName::Meaning,
      TableName::Catcode,
      TableName::Mathcode,
      TableName::Sfcode,
      TableName::Lccode,
      TableName::Uccode,
      TableName::Delcode,
    ];
    let mut snap = rustc_hash::FxHashMap::default();
    for &tname in &tables {
      let table = self.table(tname);
      for (key, values) in table {
        if let Some(front) = values.front() {
          snap.insert((tname, *key), front.clone());
        }
      }
    }
    snap
  }

  /// Perl DumpFile equivalent: Compute the diff between current state and a snapshot.
  /// Returns only entries that CHANGED since the snapshot was taken.
  /// Skips entries that contain closures (Primitive, Constructor, Conditional)
  /// since those can't be serialized — they come from Rust engine code.
  pub fn diff_from_snapshot(
    &self,
    snap: &rustc_hash::FxHashMap<(TableName, SymStr), Stored>,
  ) -> Vec<(TableName, SymStr, Stored)> {
    let tables = [
      TableName::Value,
      TableName::Meaning,
      TableName::Catcode,
      TableName::Mathcode,
      TableName::Sfcode,
      TableName::Lccode,
      TableName::Uccode,
      TableName::Delcode,
    ];
    let mut diff = Vec::new();
    for &tname in &tables {
      let table = self.table(tname);
      for (key, values) in table {
        if let Some(current) = values.front() {
          let key_pair = (tname, *key);
          let changed = match snap.get(&key_pair) {
            None => true, // new entry
            Some(prev) => {
              // Compare string representations (cheap approximation of Perl's dump-based diff)
              format!("{:?}", current) != format!("{:?}", prev)
            },
          };
          if changed && is_serializable(current) {
            diff.push((tname, *key, current.clone()));
          }
        }
      }
    }
    diff
  }

  // needed for assign_internal, so keeping it as a object method
  /// gets the current value of a named prefix
  pub fn get_prefix(&self, prefix: &str) -> bool {
    match self.prefixes.get(&arena::pin(prefix)) {
      Some(b) => *b,
      _ => false,
    }
  }

  pub(crate) fn assign_internal(
    &mut self,
    table_name: TableName,
    key: SymStr,
    value: Stored,
    mut scope_opt: Option<Scope>,
  ) {
    // hotcode lookupDefinition for \globaldefs,
    // since this is called extremely often and should be highly standardized.
    // TeX semantics: positive → all assignments global, negative → \global
    // ignored, zero → no override. `\globaldefs` is a Number register, so the
    // stored variant is `Stored::Number`, NOT `Stored::Int` — Perl's `==`
    // coerces both, Rust must unwrap explicitly. Perl `State.pm:144-151` uses
    // strict `==1`/`==-1`; we slightly broaden to TeX's sign-based rule
    // (matches behavior for the canonical `\globaldefs=1`/`\globaldefs=-1`
    // uses while also handling rare `\globaldefs=2` etc).
    // `Scope::Named(_)` is preserved per Perl's "ONLY override global/local/
    // undef" rule (State.pm:146).
    // Without this: pgfplots' `\pgfplots@pop@next@legend`
    // (`\def\foo{{\globaldefs=1 \let\x=\relax}}`) silently drops the `\let`
    // on group exit, leaving `\pgfplots@curlegend`/`@curplotlist` undefined
    // and looping `\pgfplots@createlegend` at the digest wall-clock cap.
    let preserve = matches!(scope_opt, Some(Scope::Named(_)));
    if !preserve
      && let Some(globaldefs) = self.value.get(&pin!("\\globaldefs"))
      && let Some(global_value) = globaldefs.front()
    {
      let int_value: i64 = match *global_value {
        Stored::Int(v) => v,
        Stored::Number(n) => n.0,
        _ => 0,
      };
      if int_value > 0 {
        scope_opt = Some(Scope::Global);
      } else if int_value < 0 {
        scope_opt = Some(Scope::Local);
      }
    }
    // TRACE: watch for cleanup:w
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
        if let Some(Stored::Stash(stash)) =
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
  pub fn lookup_value_sym(&self, key: SymStr) -> Option<&Stored> {
    match self.value.get(&key) {
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
    let key_str = match lookup_definition(key)? {
      Some(defn) => {
        s!("fontinfo_{}", defn.get_cs_name())
      },
      _ => {
        s!("fontinfo_{key}")
      },
    };
    Ok(self.lookup_value(&key_str))
  }
  /// manage a (global) hash of values
  pub fn lookup_mapping(&self, map: &str, key: &str) -> Option<&Stored> {
    self.lookup_mapping_sym(arena::pin(map), key)
  }
  pub fn lookup_mapping_sym(&self, map_sym: SymStr, key: &str) -> Option<&Stored> {
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
    self.lookup_stacked_values_sym(key_sym)
  }

  pub fn lookup_stacked_values_sym(&self, key: SymStr) -> Vec<&Stored> {
    if let Some(vdq) = self.value.get(&key) {
      vdq.iter().collect::<Vec<&Stored>>()
    } else {
      Vec::new()
    }
  }

  fn lookup_definition_internal(&self, key: &Token) -> Option<&VecDeque<Stored>> {
    let cc = key.get_catcode();
    let name = key.get_sym();
    let lookupname: Option<SymStr> = if (cc == Catcode::ACTIVE) || (cc == Catcode::CS) {
      if name == pin!("") { None } else { Some(name) }
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
// Perf/safety: `Cell<RotateState>` instead of `static mut` — RotateState is
// Copy, so Cell gives us Get/Set with no unsafe, preserving the thread_local
// single-threaded access guarantee without requiring unsafe at each call site.
#[thread_local]
static STATE_IN_USE: std::cell::Cell<RotateState> = std::cell::Cell::new(RotateState::Main);

pub fn use_sty_state() {
  if STATE_IN_USE.get() != RotateState::Sty {
    let mut sty_state = sty_state_mut!();
    let mut main_state = state_mut!();
    std::mem::swap(&mut *sty_state, &mut *main_state);
    STATE_IN_USE.set(RotateState::Sty);
  }
}
pub fn use_std_state() {
  if STATE_IN_USE.get() != RotateState::Std {
    let mut std_state = std_state_mut!();
    let mut main_state = state_mut!();
    std::mem::swap(&mut *std_state, &mut *main_state);
    STATE_IN_USE.set(RotateState::Std);
  }
}
pub fn use_main_state() {
  match STATE_IN_USE.get() {
    RotateState::Sty => {
      let mut sty_state = sty_state_mut!();
      let mut main_state = state_mut!();
      std::mem::swap(&mut *sty_state, &mut *main_state);
      STATE_IN_USE.set(RotateState::Main);
    },
    RotateState::Std => {
      let mut std_state = std_state_mut!();
      let mut main_state = state_mut!();
      std::mem::swap(&mut *std_state, &mut *main_state);
      STATE_IN_USE.set(RotateState::Main);
    },
    RotateState::Main => {},
  };
}

/// Free every definition/register/box this thread accumulated, returning
/// all three `State` singletons (`STATE`, `STD_STATE`, `STY_STATE`) to a
/// fresh, empty baseline and the rotation to `Main`.
///
/// **Danger:** this invalidates all live definitions/`SymStr`-keyed data
/// on the thread. Sound only between fully independent conversions in a
/// reused process — the test harness (each test serializes to owned
/// `String`s, then resets before its thread exits) or a future daemon
/// that re-initializes afterward. The single-conversion binary never
/// calls this; it exits instead. Pairs with [`crate::common::arena::reset`]
/// — see [`crate::reset_thread_engine`] for the combined entry point and
/// the `#[thread_local]`-no-drop rationale.
pub fn reset_thread_state() {
  // Make sure STATE holds the main state (not swapped out with std/sty)
  // before we replace it, so all three slots are freed for real.
  use_main_state();
  *STATE.borrow_mut() = State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  });
  *STD_STATE.borrow_mut() = State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  });
  *STY_STATE.borrow_mut() = State::new(StateOptions {
    catcodes: Some(Catcodes::Style),
    ..StateOptions::default()
  });
  STATE_IN_USE.set(RotateState::Main);
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
        Info!(
          "ignore",
          lock_key,
          s!("Ignoring redefinition of {lock_key}")
        );
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
  // Perl-faithful counter leniency. A `\c@<ctr>` control sequence is, by
  // LaTeX convention, the count register backing counter `<ctr>`. When code
  // reads an *undefined* one in a number/register context (e.g.
  // `\setcounter{x}{\value{y}}` or `\algrestore`/`\ContinuedFloat` reading
  // `\c@subalgorithm@save`), Perl does NOT raise a hard "undefined control
  // sequence" error — its counter machinery warns "Counter '<ctr>' was not
  // defined; assuming 0" (Package.pm L712) and treats it as 0. Without this,
  // `read_x_token` expands the bare undefined `\c@<ctr>` through the generic
  // <ltx:ERROR/> path below and the run gains a spurious error. Mirror Perl:
  // warn and define the register as 0 so the reader sees a register value,
  // not an undefined CS. Same category/message as `counter::dialect::
  // counter_value`. Witness 1910.02851 (`\algrestore{RLZFactorization}` +
  // `\ContinuedFloat` → `\c@subalgorithm@save`); Perl rc=0.
  if let Some(ctr) = cs.strip_prefix("\\c@") {
    if !lookup_bool("SUPPRESS_UNDEFINED_ERRORS") {
      Warn!(
        "undefined",
        ctr,
        s!("Counter {} was not defined; assuming 0", ctr)
      );
    }
    crate::binding::def::dialect::def_register(*token, None, Number::new(0), None)?;
    return Ok(*token);
  }
  // Gate the undefined-CS summary tally by SUPPRESS_UNDEFINED_ERRORS so it
  // matches the `Error!` gate at L1021 below — during expl3-code.tex raw
  // load with thousands of forward-references we install the ERROR stub
  // without polluting the user-facing summary count. See
  // project_kernel_dump_parity.md "iow_wrap residual" for full diagnosis.
  if !lookup_bool("SUPPRESS_UNDEFINED_ERRORS") {
    note_status(LogStatus::Undefined, Some(&cs));
  }
  // To minimize chatter, go ahead and define it...
  if cs.starts_with("\\if") {
    // Apparently an \ifsomething ???
    let name = cs.replace("\\if", "");
    // Perl `generateErrorStub` (State.pm L539-540) passes the recovery note
    // as a SEPARATE Error detail, so `generateMessage` renders it on its own
    // indented line — not merged into the primary message. Match that (and
    // the already-correct stomach.rs path) so the cortex `details`/log first
    // line is just "...is not defined." like Perl.
    Error!(
      "undefined",
      token,
      s!("The token {} is not defined.", token.stringify()),
      "Defining it now as with \\newif"
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
    // Allow suppression of undefined errors during bulk loading (e.g., expl3-code.tex)
    // where forward references are later resolved by post-load fixups.
    if !lookup_bool("SUPPRESS_UNDEFINED_ERRORS") {
      Error!(
        "undefined",
        token,
        s!("The token {} is not defined.", token.stringify()),
        "Defining it now as <ltx:ERROR/>"
      );
    }
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

/// Install a `Constructor` for `token` whose sole effect at digestion time is to
/// emit `<ltx:ERROR class='undefined'>content</ltx:ERROR>` (the Rust equivalent
/// of Perl `Document::makeError`). It logs NOTHING — the caller is responsible
/// for the `Error!`/`note_status`. Mirrors the make_error constructor that
/// `generate_error_stub` installs for undefined *commands*, so undefined
/// *environments* (`\begin{undefinedenv}`) leave the same visible
/// `<ltx:ERROR>` marker as Perl instead of silently vanishing from the output.
pub fn install_undefined_error_constructor(token: Token, content: &str) {
  let content = content.to_string();
  install_definition(
    Constructor {
      cs: token,
      replacement: Some(Rc::new(move |document, _args, _props| {
        document.make_error("undefined", &content)
      })),
      ..Constructor::default()
    },
    Some(Scope::Global),
  );
}

// SAFETY
// any method which does not return a borrowed piece of data should be package-level
// so that the global singleton State can get locked+unlocked during the same call
// thus entirely AVOIDING possible runtime panics due to RefCell lock races.
// TODO: Should this be a prelude?

/// assigns a `Stored` value at the given key and scope
/// Direct mirror of Perl's free-function form
/// `LaTeXML::Core::State::assign_internal($STATE, $table, $key, $value, $scope)`
/// (Core/State.pm L140). Bypasses every dialect / lock / let-chase / admission
/// layer Rust has accreted on top of the table mutation; used by the dump
/// loader (Core/Dumper.pm `V/Cc/Mc/Sc/Lc/Uc/Dc/Im/I/Lt`) so the dump replay
/// matches Perl exactly: one record == one `assign_internal` call.
pub fn assign_internal<T: Into<Stored>>(
  table_name: TableName,
  key: SymStr,
  value: T,
  scope: Option<Scope>,
) {
  state_mut!().assign_internal(table_name, key, value.into(), scope);
}

pub fn assign_value<T: Into<Stored>, S: Into<Option<Scope>>>(key: &str, value: T, scope: S) {
  state_mut!().assign_value(key, value, scope)
}

/// assigns a `Stored` value 'inplace': replaces the front value in whatever frame
/// it was originally assigned in, without recording an undo entry.
/// This matches Perl's `assignValue(key, value, 'inplace')`.
/// Used for MODE changes in enter_horizontal (switches mode without creating a new binding).
pub fn assign_value_inplace(key: &str, value: impl Into<Stored>) {
  assign_value_inplace_sym(arena::pin(key), value)
}
/// Sym-keyed variant of `assign_value_inplace` — skip the per-call
/// `arena::pin(key)` for hot callers with a pre-pinned SymStr.
pub fn assign_value_inplace_sym(key_sym: SymStr, value: impl Into<Stored>) {
  let value = value.into();
  let state = &mut *state_mut!();
  let table = &mut state.value;
  if let Some(vvec) = table.get_mut(&key_sym)
    && let Some(front) = vvec.front_mut()
  {
    *front = value;
    return;
  }
  // If the value was never assigned, push globally (matching Perl behavior)
  let vvec = table.entry(key_sym).or_default();
  vvec.push_front(value);
  // Find the locked frame and record the undo there
  for frame in &mut state.undo {
    if frame.locked {
      frame.table_mut(TableName::Value).insert(key_sym, 1);
      break;
    }
  }
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
      None | Some(&mut Stored::None) => None,
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
    None => {
      // Key was never assigned — silently ignore the checkin
      log::warn!("checkin_value called for unknown key '{key}'");
    },
    Some(vvec) => match vvec.front_mut() {
      None => {
        log::warn!("checkin_value called with empty value stack for key '{key}'");
      },
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
  // Capture any BUG-path message, but raise the Error! *after* the state_mut!()
  // borrow is dropped — Error! reads MAX_ERRORS, and a live mutable borrow there
  // panics "RefCell already mutably borrowed" (tikz-cd 2001.08973).
  let bug: Option<String> = {
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
      Some(&mut Stored::VecDequeStored(ref mut front)) => {
        front.push_back(value);
        None
      },
      // auto-vivify, if None
      Some(ref mut field) if matches!(field, Stored::None) => {
        let mut new_vdq = VecDeque::new();
        new_vdq.push_back(value);
        **field = Stored::VecDequeStored(new_vdq);
        None
      },
      // Convert Strings (immutable array) to VecDequeStored for push — matches Perl auto-vivification
      Some(ref mut field) if matches!(field, Stored::Strings(_)) => {
        let existing: VecDeque<Stored> = if let Stored::Strings(strings) = &**field {
          strings.iter().map(|s| Stored::String(*s)).collect()
        } else {
          VecDeque::new()
        };
        let mut new_vdq = existing;
        new_vdq.push_back(value);
        **field = Stored::VecDequeStored(new_vdq);
        None
      },
      other => Some(s!(
        "BUG: Tried to push_value into an unsupported Stored field! Field was: {other:?}"
      )),
    }
  };
  if let Some(message) = bug {
    // Lowercase category for consistency with engine convention.
    Error!("state", "Stored", message);
  }
  Ok(())
}
/// pops the last value in a named `Stored::VecDequeStored` queue, if any
pub fn pop_value(key: &str) -> Result<Option<Stored>> {
  let key_sym = arena::pin(key);
  // Compute the pop result under the borrow, then raise the BUG Error! *after*
  // dropping it — Error! reads MAX_ERRORS, which panics under a live mutable
  // borrow (mirrors push_value; tikz-cd 2001.08973).
  let popped: std::result::Result<Option<Stored>, ()> = {
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
      Err(())
    }
  };
  match popped {
    Ok(v) => Ok(v),
    Err(()) => {
      Error!(
        "State",
        "Stored",
        "BUG: Tried to pop_value from a non-vecdeque value key!"
      );
      Ok(None)
    },
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
    Some(Stored::Tokens(tks)) => tks.unlist_mut().extend(value.unlist()),
    None | Some(Stored::None) => state.assign_value(key, Stored::Tokens(value), None),
    Some(other) => panic!("Can only push_tokens into a Stored::Tokens, but got {other:?}"),
  }
}

pub fn lookup_value(key: &str) -> Option<Stored> { state!().lookup_value(key).cloned() }
pub fn with_value<R, FnR>(key: &str, caller: FnR) -> R
where FnR: FnOnce(Option<&Stored>) -> R {
  caller(state!().lookup_value(key))
}
/// Sym-keyed variant of `with_value` — avoids the per-call `arena::pin(key)`.
pub fn with_value_sym<R, FnR>(key: SymStr, caller: FnR) -> R
where FnR: FnOnce(Option<&Stored>) -> R {
  caller(state!().lookup_value_sym(key))
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

/// `lookup_bool` variant for hot call sites with a pre-pinned SymStr
/// (see `crate::pin!`). Skips the per-call `arena::pin(key)` hash
/// lookup — significant on every-expansion hot paths. `SymStr` is a
/// `u32` wrapper (Copy), so it passes by value — no borrow overhead.
pub fn lookup_bool_sym(key: SymStr) -> bool {
  let state = state!();
  match state.lookup_value_sym(key) {
    None => false,
    Some(v) => v.into(),
  }
}

/// `lookup_string` variant using a pre-pinned SymStr key.
pub fn lookup_string_from_sym(key: SymStr) -> String {
  let state = state!();
  match state.lookup_value_sym(key) {
    None => String::new(),
    Some(v) => v.into(),
  }
}
/// like `lookup_value`, but casts the entry into a SymStr from the string interner
///  (`pin!("")` if None)
pub fn lookup_string_sym(key: &str) -> SymStr {
  let state = state!();
  match state.lookup_value(key) {
    None => pin!(""),
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
/// `lookup_int` variant that never panics on a live mutable borrow.
///
/// Returns `None` when STATE is currently mutably borrowed (contention),
/// `Some(0)` when the key is absent/non-integer (matching `lookup_int`'s
/// default), else `Some(value)`.
///
/// This exists for the `Error!`/`Warn!` reporting path: an error can legitimately
/// be raised from inside a `state_mut()` scope (e.g. `push_value`'s BUG branch,
/// or any constructor `after_digest` holding the borrow). A plain `borrow()` there
/// panics "RefCell already mutably borrowed", aborting the whole conversion
/// (FATAL_panic; crashed tikz-cd 2001.08973 via `push_value("QED@stack", …)`).
/// The error reporter must be re-entrancy-safe regardless of what borrows are
/// held — degrade to "unknown" on contention rather than crash.
pub fn try_lookup_int(key: &str) -> Option<i64> {
  let state = (*STATE).try_borrow().ok()?;
  Some(match state.lookup_value(key) {
    Some(Stored::Int(i)) => *i,
    Some(Stored::Bool(true)) => 1,
    Some(Stored::Number(n)) => n.value_of(),
    _ => 0,
  })
}

pub fn remove_vecdeque(key: &str) -> Option<VecDeque<Stored>> {
  match remove_value(key) {
    Some(Stored::VecDequeStored(v)) => Some(v),
    _ => None,
  }
}
/// convenience method to lookup the current value at the "font" key
pub fn lookup_font() -> Option<Rc<Font>> {
  // try_borrow, not state!()'s borrow(): this accessor is reachable from a
  // Whatsit's Display/revert path (e.g. tex_glue::revert_skip → lookup_font)
  // which can run *while STATE is already mutably borrowed* — e.g. formatting a
  // whatsit into a log/error message inside a state_mut() scope. A plain
  // borrow() then panics "RefCell already mutably borrowed", aborting the worker
  // (FATAL_101; crashed hep-th9908053, a \documentstyle[12pt]{article} 2.09
  // paper). Degrade to None on contention instead of crashing.
  //
  // CAUTION for future callers (PR #249 review P3-18): None-on-contention is
  // only correct for Display/revert/log-formatting consumers (where a
  // defaulted font is cosmetic). Several digestion-path callers `.unwrap()`
  // the result (tbox.rs, whatsit.rs, stomach.rs) — they would panic loudly on
  // contention, which is the desired behavior there: a DIGESTION-path
  // re-entrant lookup is a real bug, and silently defaulting the font would
  // turn it into invisible wrong-font drift in the XML. If you add a caller,
  // pick deliberately: `.unwrap()` on digestion paths, graceful None only
  // where the font is presentational.
  let Ok(st) = (*STATE).try_borrow() else {
    return None;
  };
  match st.lookup_value_sym(pin!("font")) {
    None | Some(Stored::None) => None,
    Some(f) => f.into(),
  }
}
/// convenience method to lookup the current value at the "mathfont" key
pub fn lookup_mathfont() -> Option<Rc<Font>> {
  // Route through `lookup_value_sym` with a cached SymStr (via
  // `pin!`) to skip the per-call `arena::pin("mathfont")` probe on
  // this hot path (math-env entry/exit, per-formula checks).
  match state!().lookup_value_sym(pin!("mathfont")) {
    None | Some(Stored::None) => None,
    Some(v) => v.into(),
  }
}

/// a convenience method to globally asign a `Font` to the "font" key
pub fn assign_font(font: Rc<Font>, scope: Option<Scope>) {
  assign_value_sym(pin!("font"), Stored::Font(font), scope);
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
      // Release the state borrow first, then pass the interned &str directly
      // into tokenize_internal via the re-entrant arena — avoids materializing
      // an owned String just to tokenize.
      let sym = *sym;
      drop(state);
      arena::with(sym, |astr| Some(mouth::tokenize_internal(astr)))
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

/// a variant of `lookup_token` taking an already-pinned SymStr key —
/// avoids the per-call `arena::pin(key)` hash lookup.
pub fn lookup_token_sym(key: SymStr) -> Option<Token> {
  match state!().lookup_value_sym(key) {
    Some(Stored::Token(t)) => Some(*t),
    _ => None,
  }
}

pub fn lookup_alignment() -> Option<Digested> {
  // Can only be a token or definition; we want defns!
  // is this the right logic here? don't expand unless digesting?
  state!().lookup_value_sym(pin!("Alignment")).and_then(|v| {
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
  assign_register_token(&T_CS!(cs), value, scope, parameters)
}
/// `assign_register` variant taking a pre-built Token — lets hot
/// callers skip the `T_CS!(&str)` pin when they already have the CS
/// cached (e.g. via `T_CS!("\\c@…")` literal which routes through
/// `pin!`).
pub fn assign_register_token(
  cs: &Token,
  value: RegisterValue,
  scope: Option<Scope>,
  parameters: Vec<ArgWrap>,
) -> Result<()> {
  let defn_opt = lookup_definition(cs)?;
  if let Some(defn) = defn_opt
    && defn.is_register()
  {
    defn.set_value(value, scope, parameters);
    return Ok(());
  }
  Warn!(
    "expected",
    "register",
    format!("The control sequence '{cs}' is not a register")
  );
  Ok(())
}
pub fn lookup_register(cs: &str, parameters: Vec<ArgWrap>) -> Result<Option<RegisterValue>> {
  lookup_register_token(&T_CS!(cs), parameters)
}
/// Token-keyed variant of `lookup_register` — saves the per-call
/// `T_CS!(&str)` pin for hot callers with a cached CS token.
pub fn lookup_register_token(
  cs: &Token,
  parameters: Vec<ArgWrap>,
) -> Result<Option<RegisterValue>> {
  Ok(match lookup_definition(cs)? {
    Some(defn) => {
      if defn.is_register() {
        defn.value_of(parameters)
      } else {
        let message = s!("The control sequence '{}' is not a register", cs);
        Warn!("expected", "register", message);
        None
      }
    },
    _ => None,
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
    if lookupname != pin!("") {
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
  // `get_executable_name` previously built a fresh `String` + `arena::pin`
  // probe per call; `pin_cs_name` already returns a cached `SymStr`
  // (primitive → `Catcode::name_sym`, otherwise `self.text`). Saves a
  // RefCell mut-borrow on the interner + a hashmap probe per token in
  // the gullet's conditional dispatch.
  if !token.code.is_executable() {
    return None;
  }
  let lookup_sym = token.pin_cs_name();
  state!().meaning.get(&lookup_sym).and_then(|entry| {
    if let Some(Stored::Conditional(defn)) = entry.front() {
      Some(defn.conditional_type)
    } else {
      None
    }
  })
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
  } else if receiver.is_none() || matches!(receiver, Some(Stored::None)) {
    // Key doesn't exist yet — create a new VecDequeStored via the existing borrow
    let mut vd = VecDeque::new();
    for value in values_iter {
      vd.push_back(value);
    }
    state.assign_internal(
      TableName::Value,
      key_sym,
      Stored::VecDequeStored(vd),
      Some(Scope::Global),
    );
  } else {
    // Wrong type — warn but don't panic
    Warn!(
      "unexpected",
      "unshift_value",
      s!(
        "unshift_value expects VecDequeStored receiver for key {:?}, got: {:?}",
        key,
        receiver.map(|r| std::mem::discriminant(r))
      )
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
/// Sym-keyed variant — skip the per-call `arena::pin(map)` for hot
/// callers with a pre-pinned map key (e.g. via `pin!("siunitx_macros")`).
pub fn lookup_mapping_sym(map_sym: SymStr, key: &str) -> Option<Stored> {
  state!().lookup_mapping_sym(map_sym, key).cloned()
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
      Some(_) => None, // non-catcode value in catcode table — treat as undefined
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
pub fn lookup_mathcode_sym(key_sym: SymStr) -> Option<u16> {
  match state!().mathcode.get(&key_sym) {
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
  if token.get_catcode().is_active_or_cs() && token.text != pin!("") {
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

/// Closure-based variant of `lookup_meaning` — avoids the per-call
/// `Stored::clone()` when the caller only needs to *inspect* the
/// meaning (e.g. extract a CS Token from an Expandable/Primitive
/// definition). Stored::clone is ~1% of total instructions on
/// siunitx-heavy fixtures (5M+ calls per run, each cloning a full
/// Stored enum). This helper borrows the stored value instead.
///
/// For non-CS/ACTIVE tokens, passes `Some(Stored::Token(*token))` —
/// matching lookup_meaning's fallback semantics. Note this requires
/// a single stack allocation of Stored::Token (Copy), not a heap
/// clone.
pub fn with_meaning<R>(token: &Token, f: impl FnOnce(Option<&Stored>) -> R) -> R {
  let state = state!();
  if token.get_catcode().is_active_or_cs() && token.text != pin!("") {
    match state.meaning.get(&token.text) {
      Some(entry) => match entry.front() {
        None | Some(Stored::None) => f(None),
        Some(other) => f(Some(other)),
      },
      None => f(None),
    }
  } else {
    // Non-CS/ACTIVE: the "meaning" is just the token itself. The
    // caller gets a borrow of a temporary here, which is safe for
    // the duration of the closure.
    let s = Stored::Token(*token);
    f(Some(&s))
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
  let mut meaning = meaning.into();
  // short-circuit guard to avoid e.g. T_MATH let to itself
  if let Stored::Token(ref mt) = meaning
    && token == mt
  {
    return;
  }
  // For \let chains: if the target token has an expandable/primitive definition,
  // store that definition directly instead of the Token indirection.
  // This ensures `\let \foo \bar` where \bar is expandable makes \foo expandable too.
  // Follow at most 50 \let links to avoid cycles.
  if let Stored::Token(ref target) = meaning {
    let mut current = *target;
    for _ in 0..50 {
      match lookup_meaning(&current) {
        Some(Stored::Token(next)) => {
          current = next; // follow chain
        },
        Some(Stored::None) | None => break, // dead end — keep as Token
        Some(defn) => {
          // Found a real definition — use it directly
          meaning = defn;
          break;
        },
      }
    }
  }
  let csname_sym = token.pin_cs_name();
  state_mut!().assign_internal(TableName::Meaning, csname_sym, meaning, scope);
}

// keep this in sync with `lookup_meaning`, it is copied over for optimization purposes
pub fn has_meaning(token: &Token) -> bool {
  if token.get_catcode().is_active_or_cs() && token.text != pin!("") {
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
      && lookup_bool_sym(crate::pin!("IN_MATH"))
      && (lookup_mathcode_sym(t_sym).unwrap_or(0) == 0x8000))
  {
    t_sym
  } else {
    // Use cached SymStr from `Catcode::name_sym` instead of re-interning
    // `cc.name()` (a &'static str) on every non-active-or-cs token —
    // saves a hashmap probe per token on the digest hot path.
    cc.name_sym()
  };
  // Debug!("Looking up digestable {:?}", lookupname);
  let state = state!();
  let entry_opt = state.meaning.get(&lookup_sym);
  if lookup_sym != pin!("") && entry_opt.is_some() && !entry_opt.as_ref().unwrap().is_empty() {
    // Debug!("Found definition for: {:?}", lookupname);
    if let Some(entry) = entry_opt
      && let Some(front) = entry.front()
    {
      if let Stored::Token(t) = front {
        if let Some(lookup_name) = t.get_executable_primitive_name() {
          let lookup_sym = arena::pin(lookup_name);
          if let Some(retry_entry) = state!().meaning.get(&lookup_sym) {
            // special case,
            // If a cs has been let to an executable token, lookup ITS defn.
            return retry_entry.front().cloned();
          }
        }
        // Also follow \let chains for CS tokens: if \foo is \let to \bar,
        // resolve \bar's definition. This handles expl3 aliases like
        // \tex_long:D → \long, \tex_gdef:D → \gdef.
        if t.get_catcode() == Catcode::CS
          && let Some(target_entry) = state.meaning.get(&t.text)
          && let Some(target_front) = target_entry.front()
          && !matches!(target_front, Stored::Token(_) | Stored::None)
        {
          return Some(target_front.clone());
        }
      }
      // Perl State.pm:474 lookupDigestableDefinition: the guard
      // `($defn = $$entry[0])` is FALSE when the entry's value is undef, so
      // execution falls through to `return $token` (self-inserting) for a
      // LETTER/OTHER token and to `return undef` for an active/CS one. A
      // math-active LETTER/OTHER character whose active meaning was `\let`
      // to an undefined CS hits exactly this case — e.g. braket-style
      // `\Pr{A|B}`: the macro body does `\mathcode`\|=32768 \let|\SetVert`
      // with `\SetVert` itself undefined (neither our nor Perl's braket
      // binding defines it), leaving `|`'s meaning an explicit
      // `Stored::None`. Returning `Some(Stored::None)` here routed the `|`
      // to generateErrorStub ("The token T_OTHER[|] is not defined"); Perl
      // instead self-inserts the literal char. Mirror Perl: a None-valued
      // entry for a non-active/CS (math-active) char self-inserts; active/CS
      // tokens still fall to the `None` return below. Witness 1602.01342.
      if matches!(front, Stored::None) && !is_active_or_cs {
        return Some(token.into());
      }
      // if a regular definition, just return.
      return Some(front.clone());
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
/// Diagnostic helper: dump the keys in undo[0]'s value table.
/// For temporary instrumentation only — no production callers should rely on this.
pub fn dump_top_frame_keys() -> String {
  let state = state!();
  let f0 = state.undo.front().expect("undo is non-empty");
  let mut entries: Vec<String> = Vec::new();
  for (k, v) in f0.table(TableName::Value).iter() {
    let val = state
      .value
      .get(k)
      .and_then(|vec| vec.front())
      .map(|s| format!("{s:?}"))
      .unwrap_or_else(|| "<none>".into());
    let ks: String = arena::with(*k, |s| s.to_string());
    entries.push(format!("{ks}=[{v}, {val}]"));
  }
  entries.sort();
  entries.join(", ")
}

pub fn push_frame() {
  // Easy: just push a new undo frame.
  state_mut!().undo.push_front(UndoFrame::default());
}

/// Snapshot of the keys currently bound at the topmost (calling) undo frame
/// for the Meaning table. Used by Perl-style autoload triggers that need to
/// promote everything a package's load just installed at this scope to
/// GLOBAL — without that promotion, sibling autoload triggers fired AFTER
/// a group pop would re-fire on a now-undefined sibling CS (the canonical
/// case is `\begin{subequations}` triggering amsmath autoload at depth=N,
/// then a later `\begin{align}` at depth=0 finding `\align` undefined
/// because amsmath's depth=N install was popped on `\end{subequations}`).
pub fn snapshot_top_frame_meaning_keys() -> Vec<SymStr> {
  state!()
    .undo
    .front()
    .map(|f| f.meaning.keys().copied().collect())
    .unwrap_or_default()
}

/// Hoist every Meaning binding installed at the topmost frame since
/// `pre_snapshot` was taken to GLOBAL scope. Idempotent: keys already
/// in `pre_snapshot` are skipped. Operates on the Meaning table only —
/// callers that need to promote Value/Catcode/etc. should add parallel
/// helpers (none required so far).
pub fn hoist_top_frame_meaning_delta(pre_snapshot: &[SymStr]) {
  let pre: rustc_hash::FxHashSet<SymStr> = pre_snapshot.iter().copied().collect();
  let new_keys: Vec<SymStr> = {
    let state = state!();
    state
      .undo
      .front()
      .map(|f| {
        f.meaning
          .keys()
          .copied()
          .filter(|k| !pre.contains(k))
          .collect()
      })
      .unwrap_or_default()
  };
  for key in new_keys {
    let current = {
      let state = state!();
      state
        .meaning
        .get(&key)
        .and_then(|stack| stack.front().cloned())
    };
    if let Some(value) = current {
      // Direct re-bind via assign_internal so we don't need to round-trip a
      // full Token. The Meaning table is keyed by SymStr (the CS name);
      // any future read via `assign_meaning(token, ...)` would reach the
      // same cell. Scope::Global removes higher-frame undo entries and
      // installs at the lowest non-locked frame.
      state_mut!().assign_internal(TableName::Meaning, key, value, Some(Scope::Global));
    }
  }
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
pub fn get_frame_depth() -> usize { state!().undo.iter().filter(|frame| !frame.locked).count() }

/// `true` when the CURRENT (front) stack frame is the locked bottom frame —
/// i.e. there is no openable group/mode frame to pop. Popping it would FATAL.
pub fn current_frame_locked() -> bool { state!().undo.front().map(|f| f.locked).unwrap_or(true) }
/// begins a semiverbatim frame, neutralizing the usual + requested characters
pub fn begin_semiverbatim(extraspecials: Option<&[char]>) {
  // Is this a good/safe enough shorthand, or should we really be doing beginMode?
  push_frame();
  assign_value("MODE", "restricted_horizontal", None);
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
  assign_mathcode('\'', 0x8000u16, Some(Scope::Local));
  // try to stay as ASCII as possible
  if let Some(ref current_font) = lookup_font() {
    let local_font = current_font.merge(fontmap!(encoding => "ASCII"));
    assign_font(Rc::new(local_font), Some(Scope::Local));
  }
}
/// end by just calling `pop_frame`
pub fn end_semiverbatim() -> Result<()> { pop_frame() }

//   #======================================================================

// PARTIAL port of Perl `LaTeXML::Core::State::push/popDaemonFrame`
// (used by the Perl `latexmls` daemon to reset bindings between runs while
// keeping the loaded Pool). `pop_daemon_frame` is faithful (pop unlocked
// frames, unlock + pop the daemon frame, Fatal on the last frame).
// `push_daemon_frame` is NOT yet: Perl (State.pm L607-627) additionally
// `daemon_copy`s every mutable HASH/ARRAY value binding into the new frame —
// so IN-PLACE mutations under the daemon frame (Rust: `with_value_mut` on
// `VecDequeStored`/`HashTagData`/... values) can't corrupt the pre-frame
// state — and records `_PRELOADED_POOL_`. Without that copy, a daemon reset
// only undoes frame-tracked ASSIGNMENTS, not in-place mutations. The Rust
// persistent server (`latexml_oxide --server`) instead isolates each
// conversion in a `fork()`ed child, so these are not currently wired into a
// caller — kept (with the round-trip test in `tests/00_unit_state.rs`) as the
// seed of an in-process reset primitive for a future thread-reusing daemon
// mode, which MUST add the deep-copy semantics before relying on it. See
// `lsp_server` for the chosen fork-isolation design.
pub fn push_daemon_frame() {
  let daemon_frame = UndoFrame {
    locked: true,
    ..UndoFrame::default()
  };
  state_mut!().undo.push_front(daemon_frame);
}

pub fn pop_daemon_frame() -> Result<()> {
  let mut state = state_mut!();
  // `is_some_and(!locked)` rather than `unwrap()`: an (impossible-in-practice)
  // empty undo stack must fall through to the Fatal below, not panic.
  while state.undo.front().is_some_and(|f| !f.locked) {
    drop(state);
    pop_frame()?;
    state = state_mut!();
  }
  if state.undo.len() > 1 {
    state.undo.front_mut().unwrap().locked = false;
    drop(state);
    pop_frame()?;
  } else {
    fatal!(
      TargetUnexpected,
      Endgroup,
      "Daemon Attempt to pop last stack frame"
    );
  }
  Ok(())
}

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
  if let Some(stash_active_entry) = state.stash_active.get(&scope)
    && !stash_active_entry.is_empty()
  {
    return;
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
  // Font-relative units fall back to 10pt metrics when no current font is
  // set (e.g. pre-bootstrap unit conversion). Perl gets this via the
  // built-in default font; matching with a static fallback is cheaper
  // than forcing every caller to ensure a font frame exists.
  let font_metric =
    |getter: fn(&Font) -> i64| -> f64 { lookup_font().map(|f| getter(&f) as f64).unwrap_or(0.0) };
  match unit.as_str() {
    "em" => font_metric(|f| f.get_em_width()),
    "ex" => font_metric(|f| f.get_ex_height()),
    "mu" => font_metric(|f| f.get_mu_width()),
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

/// Convert a unit name into the exact TeX `(num, den)` fraction such that a
/// dimension of `value` units is `floor(round(value·65536)·num/den)` scaled
/// points (see `numeric_ops::fixpoint_unit`).
///
/// Physical units use TeX's `set_conversion(num)(denom)` fractions verbatim
/// (tex.web §458, lines 9020-9032): `in=7227/100, pc=12/1, cm=7227/254,
/// mm=7227/2540, bp=7227/7200, dd=1238/1157, cc=14856/1157`, plus `pt=1/1` and
/// `sp=1/65536`. `px` follows LaTeXML in aliasing `bp`. Font-relative units
/// (`em`/`ex`/`mu`) return `(metric_sp, 65536)`, matching tex.web §8983's
/// `nx_plus_y(_, v, xn_over_d(v, f, 65536))` for internal units. This is the
/// exact-integer counterpart of [`convert_unit`]; each physical entry satisfies
/// `convert_unit(u) == 65536·num/den`.
pub fn convert_unit_ratio(unit_arg: &str) -> (i64, i64) {
  let unit = unit_arg.to_lowercase();
  let font_metric =
    |getter: fn(&Font) -> i64| -> i64 { lookup_font().map(|f| getter(&f)).unwrap_or(0) };
  // UNITY == 65536 sp per pt; font-relative and `sp` units convert via the
  // `floor(fix·v/UNITY)` path (tex.web §8983 nx_plus_y/xn_over_d).
  match unit.as_str() {
    "em" => (font_metric(|f| f.get_em_width()), UNITY),
    "ex" => (font_metric(|f| f.get_ex_height()), UNITY),
    "mu" => (font_metric(|f| f.get_mu_width()), UNITY),
    "pt" => (1, 1),
    "pc" => (12, 1),
    "in" => (7227, 100),
    "bp" | "px" => (7227, 7200),
    "cm" => (7227, 254),
    "mm" => (7227, 2540),
    "dd" => (1238, 1157),
    "cc" => (14856, 1157),
    "sp" => (1, UNITY),
    u => {
      let message = s!("Illegal unit of measure {:?}, assuming pt.", u);
      Warn!("expected", "<unit>", message);
      (1, 1)
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

// TODO: Continue here -- need to diagnose why the indirect model is not returning
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
  // Perl Document.pm L196-199 maps the `autoOpen` property to a fractional
  // OPENABILITY. Most tags get 1.0; `ltx:picture` gets 0.5 (L4995) so it
  // loses path-priority against full auto-openers (para, p, text, item, …).
  // We scale to u32 (100 = full, 50 = half) to keep integer arithmetic; the
  // `desirability * openability / 100` recursion mirrors Perl's float math.
  let mut openability: SymHashMap<u32> = SymHashMap::default();
  // Collect all known tags: from the schema model AND from state tag_properties
  let mut all_tags: HashSet<SymStr> = model::get_tags().into_iter().collect();
  for tag in state!().tag_properties.keys() {
    all_tags.insert(*tag);
  }
  let picture_sym = pin!("ltx:picture");
  for tag in &all_tags {
    if let Some(x) = state!().tag_properties.get(tag)
      && let Some(true) = x.auto_open
    {
      // Perl: Tag('ltx:picture', autoOpen => 0.5). All other autoOpen
      // sites in the LaTeXML tree use `autoOpen => 1`, so a simple
      // `tag == ltx:picture` check reproduces the fraction faithfully.
      let priority = if *tag == picture_sym { 50u32 } else { 100u32 };
      openability.insert_sym(*tag, priority);
    }
  }

  for tag in &all_tags {
    let tag = *tag;
    let mut desc: SymHashMap<SymHashMap<usize>> = SymHashMap::default();
    compute_indirect_model_aux(tag, None, 100, &mut openability, &mut desc);
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
            *kid_entry.entry_sym(start).or_insert(0)
          };
          if start_entry > best {
            imodel.entry_sym(tag).or_default().insert_sym(kid, start);
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
  // Deep-copy the robust-wrapper pair.
  //
  // Our `DefConstructor`/`DefMacro` with `robust => true` stores the
  // public CS (e.g. `\ref`) as an Expandable wrapper that expands to
  // `\protect \<cs><space>`. The actual body lives under a SEPARATE
  // `\<cs><space>` slot. A plain `\let \origref \ref` would copy
  // only the wrapper — leaving the `\ref<space>` body shared between
  // `\origref` and `\ref`. A subsequent `\DeclareRobustCommand \ref
  // {...}` then overwrites `\ref<space>` and `\origref` silently
  // tracks the new body — often causing an infinite loop when the
  // new body references `\origref` itself (a common LaTeX idiom for
  // adding starred-form support: `\let\origref\ref
  // \DeclareRobustCommand\ref{\@ifstar\origref\origref}`).
  //
  // Match upstream LaTeX semantics by also `\let`ing the body half:
  // `\let \origref<space> \ref<space>` so the two CSes own
  // independent body slots and remain decoupled.
  //
  // Witnesses: canvas-3 stage-23 0810.0695 (PlanarMain.tex's
  // `\ifpdf...\else \let\origref\ref \DeclareRobustCommand\ref{
  // \@ifstar\origref\origref}\fi` triggers via the else-branch
  // because ifpdf.sty defaults `\ifpdf` to false in LaTeXML).
  // Recognize the robust-wrapper expansion `\protect \<name><space>`
  // by shape: a 2-token Expandable body matching exactly those tokens
  // where the second token's CS name equals `<token2-name><space>`.
  if let Stored::Expandable(ref defn) = meaning
    && let Some(ExpansionBody::Tokens(ref tks)) = defn.expansion
  {
    let body = tks.unlist_ref();
    if body.len() == 2 && body[0].with_str(|s| s == "\\protect") {
      let expected_body_name = token2.with_str(|s| s!("{s} "));
      if body[1].with_str(|s| s == expected_body_name) {
        // (1) Copy `\<token2><space>` body to `\<token1><space>`
        // so the two CSes have independent body slots.
        let token1_space = crate::T_CS!(token1.with_str(|s| s!("{s} ")));
        let token2_space = crate::T_CS!(expected_body_name);
        let body_meaning = lookup_meaning(&token2_space).unwrap_or(Stored::None);
        let body_csname_sym = token1_space.pin_cs_name();
        state_mut!().assign_internal(TableName::Meaning, body_csname_sym, body_meaning, scope);
        // (2) Install `\<token1>` as a NEW robust wrapper that
        // points to `\<token1><space>` (rather than reusing
        // `\<token2>`'s wrapper, which still hardcodes
        // `\<token2><space>` in its body and would silently
        // re-track any later `\DeclareRobustCommand\<token2>{...}`).
        let new_wrapper_body = Tokens::new(vec![crate::T_CS!("\\protect"), token1_space]);
        let new_wrapper = Expandable::new(
          *token1,
          None,
          Some(ExpansionBody::Tokens(new_wrapper_body)),
          Some(expandable::ExpandableOptions {
            robust: true,
            ..expandable::ExpandableOptions::default()
          }),
        );
        if let Ok(wrapper) = new_wrapper {
          install_definition(wrapper, scope);
          after_assignment();
          return;
        }
      }
    }
  }
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

/// Whether source-locator (`--source-map`) tracking + emission is on.
/// Read by the source-provenance machinery (mouth token-start capture,
/// `Document::absorb` `data-sourcepos` stamping) to stay zero-cost when off.
/// See `docs/SOURCE_PROVENANCE.md`.
pub fn source_map_enabled() -> bool { state!().source_map }
pub fn set_source_map_flag(val: bool) {
  let mut state = state_mut!();
  state.source_map = val;
}

/// Find-or-append a source file in the document-level `sources` table,
/// returning its integer `tag` (index). The per-element `data-sourcepos`
/// attribute carries this compact integer rather than a path — the
/// Source-Map-v3 `sources` convention (compact + anonymisable). Only
/// called on the source-map path. See `docs/SOURCE_PROVENANCE.md` §0.1.
pub fn source_tag(source: SymStr) -> u32 {
  let mut state = state_mut!();
  if let Some(idx) = state.source_table.iter().position(|s| *s == source) {
    idx as u32
  } else {
    state.source_table.push(source);
    (state.source_table.len() - 1) as u32
  }
}

/// Snapshot of the `sources` table (index = tag) for emitting the
/// document-level tag→file header.
pub fn source_table_snapshot() -> Vec<SymStr> { state!().source_table.clone() }

/// Record a *named* source in the opened-sources read-log. Called from
/// `Mouth::create` for file and cached-content mouths — a cold path (one
/// call per file open, not per token).
pub fn record_opened_source(source: SymStr) { state_mut!().opened_sources.insert(source); }

/// Snapshot of the opened-sources read-log (see `record_opened_source`).
pub fn opened_sources_snapshot() -> Vec<SymStr> {
  state!().opened_sources.iter().copied().collect()
}

pub fn current_verbosity() -> i32 { state!().verbosity }

pub fn push_pending_resource(value: Resource) { state_mut!().pending_resources.push(value); }
pub fn take_pending_resources() -> Vec<Resource> {
  state_mut!().pending_resources.drain(..).collect()
}
pub fn reset_pending_resources() { state_mut!().pending_resources = Vec::new(); }
pub fn get_indirect_model_relationship(tag: SymStr, childtag: SymStr) -> Option<SymStr> {
  match state!().indirect_model.as_ref().unwrap().get_sym(tag) {
    Some(sub_m) => sub_m.get_sym(childtag).copied(),
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

/// Snapshot of all registered (name, ext) binding pairs across all
/// dispatchers. Used by `find_file(notex=true)` to detect compiled-binding
/// existence regardless of extension (cls/sty/def/pool/code.tex/...).
pub fn get_binding_names() -> Vec<&'static [(&'static str, &'static str)]> {
  state!().binding_names.clone()
}
/// Append one crate's `(name, ext)` slice. Companion to
/// `set_bindings_dispatch` / `set_extra_bindings_dispatch` — call alongside
/// dispatcher registration so `find_file` can resolve compile-time
/// bindings. Duplicates are deduplicated by pointer so repeated calls from
/// the same crate don't inflate the fallback pool.
pub fn add_binding_names(names: &'static [(&'static str, &'static str)]) {
  let mut state = state_mut!();
  let ptr = names.as_ptr();
  if state.binding_names.iter().any(|s| s.as_ptr() == ptr) {
    return;
  }
  state.binding_names.push(names);
}

/// Filtered view of `get_binding_names()` returning ONLY class names
/// (without `.cls` suffix). Used by `load_class` for Perl's prefix-match
/// fallback (Package.pm L2702-2706). Returns a flat `Vec<&str>` rather
/// than per-crate slices — callers that need to preserve crate boundaries
/// should iterate `get_binding_names()` directly.
pub fn get_class_binding_names() -> Vec<&'static str> {
  state!()
    .binding_names
    .iter()
    .flat_map(|slice| slice.iter())
    .filter(|(_, ext)| *ext == "cls")
    .map(|(name, _)| *name)
    .collect()
}

/// `true` when at least one registered binding declares `ext` as its
/// extension. Used by `\input`'s heuristic to decide whether
/// `\input{name.<ext>}` should consult the binding registry — e.g.
/// `.sty`, `.cls`, `.def`, `.pool`, `code.tex` are all valid binding
/// extensions, while `.eps`, `.png`, `.bib` are not. Matches by extension
/// only (the `name` is checked separately by `dispatch()`'s exact lookup).
pub fn is_binding_extension(ext: &str) -> bool {
  state!()
    .binding_names
    .iter()
    .any(|slice| slice.iter().any(|(_, e)| *e == ext))
}

/// `true` when a binding is registered for the exact `(name, ext)` pair.
/// Convenience wrapper over the per-crate slices in `binding_names`.
/// Mirrors `dispatch()`'s lookup but without the side effect of loading.
pub fn binding_exists(name: &str, ext: &str) -> bool {
  state!()
    .binding_names
    .iter()
    .any(|slice| slice.iter().any(|(n, e)| *n == name && *e == ext))
}

pub fn get_label_mapping_hook() -> Option<LabelMappingHook> { state!().label_mapping_hook.clone() }
pub fn set_label_mapping_hook(hook: LabelMappingHook) {
  let mut state = state_mut!();
  state.label_mapping_hook = Some(hook);
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
/// Replace the entire search_paths list (Perl: `AssignValue(SEARCHPATHS => [...])`).
pub fn set_search_paths(paths: Vec<String>) {
  let mut state = state_mut!();
  state.search_paths.clear();
  for p in paths {
    state.search_paths.push_back(p);
  }
}
pub fn has_search_paths() -> bool { !state!().search_paths.is_empty() }
/// Mirror Perl's `LookupValue('GRAPHICSPATHS')` — a list value that all
/// `\graphicspath`, `\svgpath`, initial source-directory prepends, and
/// `image_candidates` consult. Always return as `Vec<String>` even if the
/// value was stored as `Strings` (initial assignValue) or `VecDequeStored`
/// (after any push/unshift).
pub fn get_graphics_paths() -> Vec<String> {
  lookup_value("GRAPHICSPATHS")
    .map(|v| match v {
      Stored::Strings(syms) => syms.iter().map(|s| arena::to_string(*s)).collect(),
      Stored::VecDequeStored(vdq) => vdq
        .iter()
        .filter_map(|item| match item {
          Stored::String(s) => Some(arena::to_string(*s)),
          _ => None,
        })
        .collect(),
      _ => Vec::new(),
    })
    .unwrap_or_default()
}

/// Zero-alloc membership test for GRAPHICSPATHS. Mirrors the Perl idiom
/// `grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') }` but
/// without allocating an owned `Vec<String>` for a single boolean — the
/// interned-symbol `with`/`with2` family resolves each path in place.
pub fn graphics_paths_contains(needle: &str) -> bool {
  lookup_value("GRAPHICSPATHS")
    .map(|v| match v {
      Stored::Strings(syms) => syms.iter().any(|s| arena::with(*s, |p| p == needle)),
      Stored::VecDequeStored(vdq) => vdq.iter().any(|item| match item {
        Stored::String(s) => arena::with(*s, |p| p == needle),
        _ => false,
      }),
      _ => false,
    })
    .unwrap_or(false)
}

/// Mirror Perl's `$state->unshiftValue(GRAPHICSPATHS => $dir)`. Used by
/// Core.pm-style source-directory prepends.
pub fn graphics_paths_push_front(path: String) {
  let key = arena::pin("GRAPHICSPATHS");
  let entry = Stored::String(arena::pin(&path));
  let mut state = state_mut!();
  if !state.value.contains_key(&key) {
    state.assign_internal(
      TableName::Value,
      key,
      Stored::VecDequeStored(VecDeque::new()),
      Some(Scope::Global),
    );
  }
  let receiver = state.value.get_mut(&key).unwrap().front_mut();
  match receiver {
    Some(Stored::VecDequeStored(vdq)) => vdq.push_front(entry),
    Some(Stored::Strings(syms)) => {
      let mut vdq: VecDeque<Stored> = syms.iter().map(|s| Stored::String(*s)).collect();
      vdq.push_front(entry);
      state.assign_internal(
        TableName::Value,
        key,
        Stored::VecDequeStored(vdq),
        Some(Scope::Global),
      );
    },
    _ => {
      let mut vdq = VecDeque::new();
      vdq.push_front(entry);
      state.assign_internal(
        TableName::Value,
        key,
        Stored::VecDequeStored(vdq),
        Some(Scope::Global),
      );
    },
  }
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
      Some(Stored::HashStored(h)) => h.get_sym(key),
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
/// Sym-keyed variant of `with_stacked_values`.
pub fn with_stacked_values_sym<R, FnR>(key: SymStr, caller: FnR) -> R
where FnR: FnOnce(Vec<&Stored>) -> R {
  caller(state!().lookup_stacked_values_sym(key))
}

pub fn set_state(incoming_state: State) {
  // Reset state rotation to Main to prevent stale Sty/Std state from previous runs
  STATE_IN_USE.set(RotateState::Main);
  let mut global_state = state_mut!();
  *global_state = incoming_state;
}

/// Check whether a Stored value can be serialized for the kernel dump.
/// Values containing closures (Primitive, Constructor, Conditional, etc.)
/// cannot be serialized — they come from Rust engine code, not the dump.
/// This matches Perl's DumpFile which only serializes Expandable macros.
pub fn is_serializable(stored: &Stored) -> bool {
  use Stored::*;
  match stored {
    // Data types: always serializable
    None | Bool(_) | String(_) | Charcode(_) | Int(_) | Catcode(_) => true,
    Token(_) | Tokens(_) | Number(_) | Float(_) => true,
    Glue(_) | MuGlue(_) | Dimension(_) | MuDimension(_) => true,
    Reversion(_) | KeyVal(_) => true,
    Chars(_) | Strings(_) => true,
    // Expandable: serializable when body is Tokens OR None (regular
    // macros). Closure-bodied Expandables (e.g. `\expandafter`,
    // `\unexpanded`, `\the` — defined via `DefMacro!` with a closure
    // body) ALSO pass — dump_writer's `serialize_stored` emits a PA
    // alias to the canonical CS so `\let \tex_expandafter:D
    // \expandafter`-style aliases survive the dump. (Bug C parity fix
    // — see project_kernel_dump_tdd.md.) The writer's add-only policy
    // at load time skips entries whose key is already defined in the
    // compiled engine, so primary CSes don't double-bind.
    Expandable(_) => true,
    // Register: serializable (stores value + type, no closures)
    Register(_) => true,
    // Font: serializable (data only)
    Font(_) => true,
    // Primitives/MathPrimitives/Conditionals: the CLOSURE can't be
    // serialized, but each carries its own canonical CS name. If the
    // entry's key differs from that canonical CS, this is a `\let`-alias
    // we CAN capture (as a "PA" pointer) so the dump reader replays the
    // `\let` at load time. dump_writer returns the PA tag; dump_reader
    // re-applies via state::let_i. This is how \tex_let:D, \tex_def:D,
    // \tex_ifx:D, \if_meaning:w, and the hundreds of other expl3-renamed
    // primitives + conditionals survive the dump without needing to re-run
    // 36k lines of expl3-code.tex.
    //
    // Returning true here only means "pass to dump_writer"; the writer's
    // serialize_stored emits the PA target. Self-aliases (primary CSes
    // not yet aliased anywhere) typically don't appear in the diff because
    // they're in the pre-snapshot — but if they do, the dump reader skips
    // them by comparing key to target.
    Primitive(_) | MathPrimitive(_) | Conditional(_) => true,
    // Constructor: same logic as Primitive/Conditional. Constructors carry a
    // closure body the dump can't serialize, BUT they each carry a canonical
    // CS field. When the entry key differs from that CS, it's a `\let`-alias
    // (e.g. `\let \tex_par:D \par` where `\par` is itself a `Let!` alias to
    // `\lx@normal@par` — a Constructor). dump_writer emits `PA\t<cs>`;
    // dump_reader replays via `state::let_i`. Mirrors Perl's writer:
    // `dump_constructor` is undefined in `Dumper.pm`, but TeX_Job.pool
    // `DumpFile`'s let-detection branch (L184-198) catches the (key !=
    // value->getCSName) case and emits `Lt(key, letkey)`. Without this,
    // `\tex_par:D`, `\tex_cr:D`, `\tex_noindent:D`, etc. drop from the dump
    // because diff_from_snapshot filters them before the writer's
    // Constructor arm sees them.
    Constructor(_) => true,
    // Collections: serializable if contents are
    VecDequeStored(_) | HashStored(_) | HashString(_) => true,
    // Everything else: skip for safety
    _ => false,
  }
}

/// Take a snapshot of the current State (for dump diff).
pub fn take_snapshot() -> rustc_hash::FxHashMap<(TableName, SymStr), Stored> { state!().snapshot() }

/// Compute diff from snapshot and return changed serializable entries.
pub fn diff_snapshot(
  snap: &rustc_hash::FxHashMap<(TableName, SymStr), Stored>,
) -> Vec<(TableName, SymStr, Stored)> {
  state!().diff_from_snapshot(snap)
}

// Thread-local holder for the snapshot taken at a named init phase.
// Currently only "bootstrap" is used: when `latex.rs` finishes loading
// `latex_bootstrap`, it stashes the state snapshot here. `ini_tex::dump_format`
// reads it so its diff matches Perl's `DumpFile` semantics — "what did raw
// latex.ltx + the rest of the engine init add on top of pure bootstrap".
// Without this hook the snapshot is taken after `_base.rs` + `_constructs.rs`
// have also run, making the diff far narrower than Perl's dump. See
// SYNC_STATUS D0 (d.1).
type StateSnapshot = rustc_hash::FxHashMap<(TableName, SymStr), Stored>;
type StagedSnapshotMap = rustc_hash::FxHashMap<&'static str, StateSnapshot>;

thread_local! {
  static STAGED_SNAPSHOTS: RefCell<StagedSnapshotMap> =
    RefCell::new(rustc_hash::FxHashMap::default());
}

/// Take a snapshot now and store it under a named key for later retrieval.
/// Intended for phased engine init (e.g. `stage_snapshot("bootstrap")` called
/// right after `latex_bootstrap` has loaded).
pub fn stage_snapshot(name: &'static str) {
  let snap = take_snapshot();
  STAGED_SNAPSHOTS.with(|m| {
    m.borrow_mut().insert(name, snap);
  });
}

/// Stage an already-taken snapshot under a named key. Used by callers
/// (like `ini_tex`) that want to snapshot at a specific point without
/// waiting for a pool hook.
pub fn stage_snapshot_value(
  name: &'static str,
  snap: rustc_hash::FxHashMap<(TableName, SymStr), Stored>,
) {
  STAGED_SNAPSHOTS.with(|m| {
    m.borrow_mut().insert(name, snap);
  });
}

/// Retrieve a previously staged snapshot, if present.
pub fn get_staged_snapshot(
  name: &str,
) -> Option<rustc_hash::FxHashMap<(TableName, SymStr), Stored>> {
  STAGED_SNAPSHOTS.with(|m| m.borrow().get(name).cloned())
}

#[cfg(test)]
mod reentrancy_tests {
  use super::*;

  /// `try_lookup_int` must degrade to `None` under a live mutable borrow
  /// (contention) instead of panicking, while behaving like `lookup_int`
  /// otherwise. This is the load-bearing primitive of the `Error!`-during-
  /// `state_mut()` fix (tikz-cd 2001.08973).
  #[test]
  fn try_lookup_int_degrades_on_contention() {
    // Absent key, no contention → Some(0), matching lookup_int's default.
    assert_eq!(try_lookup_int("p1a_absent_key_xyz"), Some(0));
    // A live mutable borrow → None (cannot read), no panic.
    let _guard = (*STATE).borrow_mut();
    assert_eq!(try_lookup_int("MAX_ERRORS"), None);
  }

  /// Reproduces tikz-cd 2001.08973: `push_value` into a non-VecDeque field
  /// hits the BUG-path `Error!`, which reads `MAX_ERRORS`. Before the fix,
  /// `push_value` held `state_mut!()` across that `Error!`, panicking
  /// "RefCell already mutably borrowed". It must now report the BUG and
  /// return Ok without panicking.
  #[test]
  fn push_value_bug_path_is_borrow_safe() {
    assign_value("p1a_bug_key", Stored::Int(7), Some(Scope::Global));
    let r = push_value("p1a_bug_key", Stored::Int(1));
    assert!(r.is_ok());
    // Same guarantee for the pop side.
    let r2 = pop_value("p1a_bug_key");
    assert!(r2.is_ok());
  }
}
