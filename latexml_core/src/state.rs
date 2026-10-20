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
    BindingDispatcher, LabelMappingHook, ResolvingBindingDispatcher,
    arena::{self, SymHashMap, SymStr},
    dimension::Dimension,
    error::{emit_warn, *},
    float::Float,
    font::{self, Font},
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
  document::{
    resource::Resource,
    tag::{AutoOpen, TagOptions},
  },
  gullet, mouth, pin,
  token::{Catcode, Token},
  tokens::{TeXString, Tokens},
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
  /// in-place: replace the value in the frame it was last bound in, WITHOUT
  /// recording a new undo entry (or globally, at the locked base frame, if it
  /// was never bound). Perl `State.pm:175` `$scope eq 'inplace'` ("Special case
  /// for `\box` & friends"). This is Knuth's "same level" reassignment / what
  /// @xworld21 tentatively called `scope => 'definition'`: the binding keeps its
  /// save-stack level, so a mutation rides exactly as long as the current
  /// binding — persisting past the current group if the binding was made above
  /// it, reverting with the group if it was made locally. Distinct from Global
  /// (which promotes + wipes lower-frame undo) and Local (which pushes an outer
  /// binding down a level). The Value-table fast path is `assign_value_inplace`.
  InPlace,
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
#[derive(Debug, Clone)]
/// For each of several tables (being "value", "meaning", "catcode" or other space of names),
/// each table maintains the bound values, and "undo" defines the stack frames
pub struct UndoFrame {
  /// The frame's identity, unique to it (an assignment journal tells the frames it began with from later ones at the
  /// same depth: `AssignmentJournalGuard::rollback`).
  id:           u64,
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

/// The next `UndoFrame::id`.
static NEXT_FRAME_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Default for UndoFrame {
  fn default() -> Self {
    UndoFrame {
      id:           NEXT_FRAME_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
      locked:       false,
      meaning:      AssignmentCount::default(),
      value:        AssignmentCount::default(),
      catcode:      AssignmentCount::default(),
      mathcode:     AssignmentCount::default(),
      sfcode:       AssignmentCount::default(),
      lccode:       AssignmentCount::default(),
      uccode:       AssignmentCount::default(),
      delcode:      AssignmentCount::default(),
      stash:        AssignmentCount::default(),
      stash_active: AssignmentCount::default(),
    }
  }
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

/// The bindings of every key assigned since an assignment journal began ([`begin_assignment_journal`]), as they were
/// then: taken at the key's first assignment after, the key's values in its table and its assignment counts in each
/// frame then open (by the frame's id), so that [`AssignmentJournalGuard::rollback`] can put them back. A key's first
/// assignment is the first time anything touches it, so the frames opened since hold none of it.
#[derive(Debug, Default)]
pub struct AssignmentJournal {
  /// the ids of the frames open when it began, from the bottom of the stack
  frames: Vec<u64>,
  saved:  rustc_hash::FxHashMap<(TableName, SymStr), JournalEntry>,
}
/// A key's values in its table (`None`: unbound) and its assignment counts by frame id.
type JournalEntry = (Option<VecDeque<Stored>>, Vec<(u64, usize)>);

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
  /// Seed only for the group-scoped `SEARCHPATHS` value-table list (read once at
  /// construction to seed it); live lookups go through [`get_search_paths`].
  pub search_paths:            VecDeque<String>,
  /// Seed only for the group-scoped `GRAPHICSPATHS` value-table list (read once at
  /// construction to seed it); live lookups go through [`get_graphics_paths`].
  pub graphics_paths:          VecDeque<String>,
  // include_styles: bool,
  /// flag to disable math parsing
  pub nomathparse:             bool,
  /// Perl `Conditional.pm:61-62`: the `if_count` Value entry, assigned
  /// `'global'` on every `\if…` so the per-conditional id survives group
  /// exit. A value only ever assigned globally collapses to one base-frame
  /// binding, so a plain monotonic field is the same counter without the
  /// per-conditional `assign_internal(Global)` undo-frame walk (measured
  /// 4.4 % of instructions on `\ifnum`-dense pgf input). Runtime-only, so
  /// it never enters a dump. Micro-divergence: `\globaldefs<0` demoted
  /// Perl's assignment to local (ids could repeat after a group); the id is
  /// the `IfFrame`'s diagnostic tag, never a matching key, so it is unaffected.
  pub if_count:                i64,
  /// Perl `Conditional.pm:63` `unshiftValue('if_stack', $ifframe)`: the active conditionals,
  /// innermost last. Perl edits the Value in place (`unshiftValue`/`shiftValue`, never grouped
  /// or undone), so a typed field is the same stack without the per-`\if`/`\else`/`\fi`
  /// value-table lookups. Runtime-only, so it never enters a dump.
  pub if_stack:                Vec<Rc<RefCell<crate::definition::conditional::IfFrame>>>,
  /// Perl `$LaTeXML::IF_LIMIT` (`latexml.sty.ltxml:101`): a plain global,
  /// never a State value. `0` = unlimited.
  pub if_limit:                i64,
  /// Perl `Object.pm:165` / `Box.pm:145`: the `absorb_count` Value, assigned
  /// `'global'` per absorbed box — the same mechanism as [`Self::if_count`]
  /// (a Perl global-only Value becomes a typed State scalar).
  pub absorb_count:            i64,
  /// Perl `$LaTeXML::ABSORB_LIMIT` (`latexml.sty.ltxml:104`), `0` = unlimited.
  pub absorb_limit:            i64,
  /// flag enabling source-locator (`--source-map`) tracking + emission.
  /// Off by default; gates BOTH the per-token start capture and the
  /// per-element `data-sourcepos` stamping so a normal conversion pays
  /// nothing. See `docs/performance/SOURCE_PROVENANCE.md`.
  pub source_map:              bool,
  /// Document-level `sources` table for the source-map feature: ordered
  /// list of source files seen, index = the integer `tag` emitted in
  /// `data-sourcepos` (Source-Map-v3 `sources` style — never an inlined
  /// path). Populated lazily via `source_tag()` only when `source_map` is
  /// on. See `docs/performance/SOURCE_PROVENANCE.md` §0.1.
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
  /// The installed binding-resolution chain — reports the source path it loaded
  /// from (for a `.rhai` file binding), or `None` for a compiled-in binding.
  pub bindings_dispatch:       Option<ResolvingBindingDispatcher>,
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
  /// The names whose current meaning a fallback class installed (`DefinitionOrigin::Fallback`, OmniBus), a kernel stub
  /// or a borrow, with that origin, until something else assigns them: a document's `\newcommand` may define them
  /// (`is_fallback_meaning`, `fallback_meaning_origin`).
  pub fallback_meanings: rustc_hash::FxHashMap<SymStr, crate::definition::origin::DefinitionOrigin>,
  /// The names assigned while a provide-only load lives ([`ProvideOnlyGuard`](crate::definition::origin::ProvideOnlyGuard)):
  /// the load's own definitions, which it may define again (`provide_only_declines`).
  pub provide_only_assigned:   rustc_hash::FxHashSet<SymStr>,
  /// The files first marked loaded while a provide-only load lives: loaded partly (their definitions of names defined
  /// before were declined), so the guard unmarks them when it ends and a later request loads them whole
  /// (`end_provide_only`).
  pub provide_only_loaded:     Vec<String>,
  /// The files first marked loaded while a binding borrows a package (`DefinitionOrigin::Borrowed`,
  /// `binding::content::require_package_borrowed`): the borrowed package and what it loads (amsthm's amsgen), which the
  /// borrow unmarks again, as LaTeX never loaded them.
  pub borrowed_loaded:         Vec<String>,
  /// The paper's copies of author files read after their stand-in bindings, by number less one
  /// ([`crate::definition::origin::copy_provides`]; [`note_copy_provides_assignment`]).
  pub copy_provides_copies:    Vec<CopyProvidesRecord>,
  /// The frames of the stand-in copies' provides-mode in force, innermost last
  /// ([`enter_copy_provides_frame`]).
  pub copy_provides_frames:    Vec<CopyProvidesFrame>,
  /// The names assigned while a stand-in binding loaded ([`crate::definition::origin::StandInLoadGuard`]): its own, which
  /// the paper's copy read after it only provides ([`note_copy_provides_assignment`]).
  pub stand_in_meanings:       rustc_hash::FxHashSet<SymStr>,
  /// The names whose meaning a class the stand-in binding loads assigned last
  /// ([`crate::definition::origin::stand_in_base_class`]): its base class's, whose text defaults are not the binding's
  /// (`unowned_text_default`).
  pub stand_in_base_defaults:  rustc_hash::FxHashSet<SymStr>,
  /// The names each stand-in binding loading now has assigned in its own code, innermost last ([`begin_stand_in_load`]).
  pub stand_in_loads:          Vec<Vec<SymStr>>,
  /// The names the stand-in binding loaded last assigned in its own code ([`end_stand_in_load`]): the public ones are
  /// the command set the paper's copy must define to be that binding's file (`binding::content` `read_stand_in_copy`).
  pub stand_in_last_load:      Vec<SymStr>,
  /// The files each stand-in binding loading now loads from its own code (its `LoadClass!`/`RequirePackage!`),
  /// innermost last ([`note_stand_in_file`]).
  pub stand_in_load_files:     Vec<Vec<String>>,
  /// The files the stand-in binding loaded last loaded from its own code ([`end_stand_in_load`]): the files it is
  /// built on, which the paper's copy may declare itself (`binding::content` `copy_is_the_stood_in_file`).
  pub stand_in_last_files:     Vec<String>,
  /// The names LaTeXML's own code (`Unknown`, `Pool`, `Binding` origins) `\let`, recorded as it makes them
  /// ([`let_i`]): a `\let` copies another name's definition, so the name a binding gave it is no longer on the object
  /// (`owned_by_the_semantic_layer`; aas_support_sty.rs's `\let\subjectheadings\keywords`).
  pub binding_lets:            rustc_hash::FxHashSet<SymStr>,
  /// The frontmatter setters whose calls are observed ([`crate::definition::origin::watch_setter_calls`]).
  pub watched_setters:         rustc_hash::FxHashSet<SymStr>,
  /// Each class loaded, with the names its load frame assigned and the classes loaded inside it: what that class
  /// defines, as its real file does ([`ClassLoadFrame`], `copy_base_class_defines`, `class_layer_definition`).
  pub class_loads:             Vec<ClassLoadRecord>,
  /// The class loads open now, as indices into [`State::class_loads`], innermost last ([`ClassLoadFrame`]).
  pub class_load_frames:       Vec<usize>,
  /// The stand-in copies whose own `\LoadClass` is open now, innermost last, each with the classes the class load
  /// open when it began had loaded: a class loaded inside it is a base class of that copy too ([`ClassLoadFrame`]; the
  /// reviewer's probe nc_chain, `\LoadClass{mybase}` over article), and a copy's first entry holds the classes its
  /// binding's load loaded ([`open_class_load_classes`]).
  pub copy_class_loads:        Vec<(u32, Vec<String>)>,
  /// The open assignment journal, if any ([`begin_assignment_journal`]).
  journal:                     Option<AssignmentJournal>,
}

/// One class loaded ([`ClassLoadFrame`]): its file name (`article.cls`), the names its load frame assigned a macro (the
/// macro itself, to tell it is still the meaning: compared by [`Rc::ptr_eq`]) or had a definition refused by a lock
/// (`None`), and the classes loaded while its frame was open, innermost included (a binding's `LoadClass!` of its base
/// class: ceurart_cls.rs → OmniBus → article).
#[derive(Debug, Default)]
pub struct ClassLoadRecord {
  pub file:    String,
  pub names:   rustc_hash::FxHashMap<SymStr, Option<Rc<Expandable>>>,
  pub classes: Vec<String>,
}

/// What [`note_copy_provides_assignment`] keeps for one stand-in's copy: the copy's file, its own definitions of names
/// the binding owns (in force only inside its code), and the names and deferring seams logged.
#[derive(Debug, Default)]
pub struct CopyProvidesRecord {
  pub file:          String,
  /// The stand-in binding the copy is read after (`binding::content::read_stand_in_copy`): a package that binding's
  /// load loaded first is the copy's own load (`<file>:loaded_by_stand_in`).
  pub binding:       String,
  pub overlay:       rustc_hash::FxHashMap<SymStr, Stored>,
  pub declined:      rustc_hash::FxHashSet<SymStr>,
  pub deferred:      rustc_hash::FxHashSet<SymStr>,
  /// The copy's own `\maketitle`/`\@maketitle` bodies, refused or overlaid as they may be
  /// ([`copy_provides_title_pages`]).
  pub title_page:    Vec<Tokens>,
  /// The classes the copy's own code loaded (`\LoadClass`, and the classes loaded inside it: [`ClassLoadFrame`]): the
  /// real class has the names they define ([`copy_provides_text_default_clashes`]).
  pub loads_classes: Vec<String>,
}

/// One frame of a stand-in copy's provides-mode ([`enter_copy_provides_frame`]): the copy (`0`: a suspension), whether
/// it repeats the frame in force (a hook chunk wrapped twice), and the names whose meanings its code replaced, with
/// the meanings kept aside.
#[derive(Debug, Default)]
pub struct CopyProvidesFrame {
  pub id:        u32,
  pub repeat:    bool,
  pub kept:      Vec<(SymStr, Stored)>,
  pub kept_keys: rustc_hash::FxHashSet<SymStr>,
  /// The other names the copy's code assigned in this frame, which may be `\let` to its own version of a kept name
  /// ([`leave_copy_provides_frame`]).
  pub assigned:  rustc_hash::FxHashSet<SymStr>,
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
      if_count:                0,
      if_stack:                Vec::new(),
      if_limit:                0,
      absorb_count:            0,
      absorb_limit:            0,
      source_map:              false,
      source_table:            Vec::new(),
      opened_sources:          HashSet::default(),
      bindings_dispatch:       None,
      extra_bindings_dispatch: None,
      binding_names:           Vec::new(),
      label_mapping_hook:      None,
      fallback_meanings:       rustc_hash::FxHashMap::default(),
      provide_only_assigned:   rustc_hash::FxHashSet::default(),
      provide_only_loaded:     Vec::new(),
      borrowed_loaded:         Vec::new(),
      copy_provides_copies:    Vec::new(),
      copy_provides_frames:    Vec::new(),
      stand_in_meanings:       rustc_hash::FxHashSet::default(),
      stand_in_base_defaults:  rustc_hash::FxHashSet::default(),
      stand_in_loads:          Vec::new(),
      stand_in_last_load:      Vec::new(),
      stand_in_load_files:     Vec::new(),
      stand_in_last_files:     Vec::new(),
      binding_lets:            rustc_hash::FxHashSet::default(),
      watched_setters:         rustc_hash::FxHashSet::default(),
      class_loads:             Vec::new(),
      class_load_frames:       Vec::new(),
      copy_class_loads:        Vec::new(),
      journal:                 None,
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

    // Perl Core.pm L50-52: `SEARCHPATHS => [map { pathname_absolute(...) } '.',
    // @searchpaths]` — the current working directory is ALWAYS searched, and
    // FIRST (it takes precedence over the `--path` dirs). A `--path` entry
    // ending in `//` is the kpsewhich recursive-search marker; preserve it
    // across canonicalization (strip it, canonicalize the base, re-append) so
    // `candidate_pathnames` can expand the marker to the whole subtree.
    let search_paths: VecDeque<String> = std::iter::once(String::from("."))
      .chain(options.search_paths.into_iter().flatten())
      .map(|p| match p.strip_suffix("//") {
        Some(base) => format!("{}//", pathname::absolute(&pathname::canonical(base))),
        None => pathname::absolute(&pathname::canonical(&p)),
      })
      .collect();
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
    // Perl Core.pm L55-57: `--includestyles` fans out to BOTH INCLUDE_STYLES
    // and INCLUDE_CLASSES (both default 0), so it raw-loads .sty packages AND
    // .cls classes from the search path. Faithful to Perl — its own
    // `# … accept both classes and styles?` hedge notwithstanding, the code
    // sets both, and INCLUDE_CLASSES is what unlocks raw .cls in LoadClass.
    if let Some(include_styles) = options.include_styles {
      state.assign_value("INCLUDE_STYLES", include_styles, Some(Scope::Global));
      state.assign_value("INCLUDE_CLASSES", include_styles, Some(Scope::Global));
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

    // SEARCHPATHS mirrors GRAPHICSPATHS: a group-scoped value list (Perl
    // `AssignValue(SEARCHPATHS…)`, local-by-default) rather than a plain field,
    // so an `\import`/`\subimport` group reverts its change at `}` while a
    // package's global add persists (#561). Seeded Global from the `search_paths`
    // field (`.` + the `--path` dirs); `get_search_paths` reads the value table.
    {
      let vdq: VecDeque<Stored> = state
        .search_paths
        .iter()
        .map(|p| Stored::String(arena::pin(p)))
        .collect();
      state.assign_internal(
        TableName::Value,
        arena::pin("SEARCHPATHS"),
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
          let value = match front {
            // A font's parameter array is written in place: the snapshot
            // keeps a copy of its own, or no later change would show.
            Stored::FontDimens(params) => {
              Stored::FontDimens(Rc::new(RefCell::new(params.borrow().clone())))
            },
            value => value.clone(),
          };
          snap.insert((tname, *key), value);
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
              // Perl diffs the SERIALIZED dump form against the bootstrap
              // snapshot (TeX_Job.pool.ltxml:126-200: a key whose `Dump`
              // string equals the `<fmt>_bootstrap` baseline is omitted), so
              // a raw redefinition to an identical body — latex.ltx:18386
              // `\gdef\@currnamestack{}` over the bootstrap's empty macro —
              // is NOT dumped. A `Debug` comparison also sees definition
              // metadata (the `:redefined@nargs` lock records of batch 56z)
              // and reported such entries as changed; the dumped empty
              // `\@currnamestack` then clobbered the live file stack of the
              // preload being read (a4wide 3 → 35 errors, cortex 2026-09-17).
              match (
                crate::dump_writer::serialize_stored(current),
                crate::dump_writer::serialize_stored(prev),
              ) {
                (Some(a), Some(b)) => a != b,
                _ => format!("{:?}", current) != format!("{:?}", prev),
              }
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
  pub fn get_prefix(&self, prefix: &str) -> bool { self.get_prefix_sym(arena::pin(prefix)) }

  /// `get_prefix` variant with a pre-pinned SymStr (see `crate::pin!`) —
  /// `assign_internal` probes the `global` prefix on every unscoped
  /// assignment, so the per-call `arena::pin` there was pure overhead.
  pub fn get_prefix_sym(&self, prefix: SymStr) -> bool {
    match self.prefixes.get(&prefix) {
      Some(b) => *b,
      _ => false,
    }
  }

  /// Record `key`'s bindings in the open assignment journal before its first assignment since the journal began.
  #[inline]
  fn journal_touch(&mut self, table_name: TableName, key: SymStr) {
    // (a journal whose guard is gone is stale: nothing records into it until the next begin discards it)
    if !JOURNAL_GUARD_LIVE.get() {
      return;
    }
    let Some(journal) = self.journal.as_ref() else {
      return;
    };
    if journal.saved.contains_key(&(table_name, key)) {
      return;
    }
    let values = self.table(table_name).get(&key).cloned();
    let counts: Vec<(u64, usize)> = self
      .undo
      .iter()
      .filter(|frame| journal.frames.contains(&frame.id))
      .filter_map(|frame| frame.table(table_name).get(&key).map(|n| (frame.id, *n)))
      .collect();
    if let Some(journal) = self.journal.as_mut() {
      journal.saved.insert((table_name, key), (values, counts));
    }
  }

  /// The `Scope::Local` binding itself: record one undo entry in the top
  /// frame (unless the key is already bound there — then replace) and push the
  /// value. Reached by every local assignment, and DIRECTLY by the save-frame
  /// bookkeeping ([`assign_frame_value_sym`]): tex.web §1214 lets
  /// `\globaldefs` (and the `\global` prefix, §1211) globalize an
  /// ASSIGNMENT made by `prefixed_command`, never the save stack (§274
  /// `new_save_level` / §282 `unsave`), so a group's own records must stay
  /// local whatever `\globaldefs` says. Perl (State.pm:144-151) routes its
  /// frame bookkeeping through the same override, and under `\globaldefs=1`
  /// (msc.sty:2616 `\msc@global@set`, tikzexternalshared, latex.ltx's
  /// `\math@fonts` block) a closed `{` group kept reporting itself as the
  /// current frame — the D12 "close non-boxing group" cascades.
  pub(crate) fn assign_local_unconditional(
    &mut self,
    table_name: TableName,
    key: SymStr,
    value: Stored,
  ) {
    self.journal_touch(table_name, key);
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
  }

  pub(crate) fn assign_internal(
    &mut self,
    table_name: TableName,
    key: SymStr,
    value: Stored,
    mut scope_opt: Option<Scope>,
  ) {
    // (a meaning a fallback class, a kernel stub or a borrowed package installs, until anything else assigns the name:
    // `fallback_meanings`)
    if table_name == TableName::Meaning {
      let origin = crate::definition::origin::current_origin();
      if matches!(
        origin,
        crate::definition::origin::DefinitionOrigin::Fallback
          | crate::definition::origin::DefinitionOrigin::Stub
          | crate::definition::origin::DefinitionOrigin::Borrowed
      ) {
        self.fallback_meanings.insert(key, origin);
      } else if !self.fallback_meanings.is_empty() {
        self.fallback_meanings.remove(&key);
      }
      if crate::definition::origin::provide_only() {
        self.provide_only_assigned.insert(key);
      }
      // (a macro a class's load frame defines — its binding's, its raw file's, a package's it loads — not LaTeXML's
      // kernel layers: `ClassLoadFrame`)
      if let Some(&class) = self.class_load_frames.last()
        && matches!(value, Stored::Expandable(_))
        && !matches!(
          origin,
          crate::definition::origin::DefinitionOrigin::Unknown
            | crate::definition::origin::DefinitionOrigin::Plain
            | crate::definition::origin::DefinitionOrigin::LatexDump
            | crate::definition::origin::DefinitionOrigin::Pool
            | crate::definition::origin::DefinitionOrigin::Format
            | crate::definition::origin::DefinitionOrigin::Document
        )
        && let Stored::Expandable(defn) = &value
        && let Some(record) = self.class_loads.get_mut(class)
      {
        record.names.insert(key, Some(Rc::clone(defn)));
      }
      if crate::definition::origin::stand_in_loading() {
        self.stand_in_meanings.insert(key);
        if crate::definition::origin::stand_in_base_class() {
          self.stand_in_base_defaults.insert(key);
        } else if !self.stand_in_base_defaults.is_empty() {
          self.stand_in_base_defaults.remove(&key);
        }
        if crate::definition::origin::stand_in_own_code()
          && let Some(load) = self.stand_in_loads.last_mut()
        {
          load.push(key);
        }
      }
    } else if table_name == TableName::Value && matches!(value, Stored::Bool(true)) {
      // (`<file>_loaded`, `load_ltxml`: a file loaded under the provide-only guard, or by a borrow)
      let provide_only = crate::definition::origin::provide_only();
      let borrowed = crate::definition::origin::current_origin()
        == crate::definition::origin::DefinitionOrigin::Borrowed;
      if provide_only || borrowed {
        arena::with(key, |name| {
          if let Some(file) = name.strip_suffix("_loaded")
            && !file.ends_with(".ltxml")
            && !file.ends_with("_raw")
          {
            for (on, list) in [
              (provide_only, &mut self.provide_only_loaded),
              (borrowed, &mut self.borrowed_loaded),
            ] {
              if on && !list.iter().any(|f| f == file) {
                list.push(file.to_string());
              }
            }
          }
        });
      }
    }
    // hotcode lookupDefinition for \globaldefs,
    // since this is called extremely often and should be highly standardized.
    // TeX semantics: positive → all assignments global, negative → \global
    // ignored, zero → no override. `\globaldefs` is a Number register, so the
    // stored variant is `Stored::Number`, NOT `Stored::Int` — Perl's `==`
    // coerces both, Rust must unwrap explicitly. Perl `State.pm:144-151` uses
    // strict `==1`/`==-1`; we slightly broaden to TeX's sign-based rule
    // (matches behavior for the canonical `\globaldefs=1`/`\globaldefs=-1`
    // uses while also handling rare `\globaldefs=2` etc).
    // `Scope::Named(_)` and `Scope::InPlace` are preserved per Perl's "ONLY
    // override global/local/undef" rule (State.pm:146 — `$scope ne 'global' &&
    // $scope ne 'local'` short-circuits the override): an in-place mutation of
    // the existing binding is NOT re-scoped by `\globaldefs`.
    // Without this: pgfplots' `\pgfplots@pop@next@legend`
    // (`\def\foo{{\globaldefs=1 \let\x=\relax}}`) silently drops the `\let`
    // on group exit, leaving `\pgfplots@curlegend`/`@curplotlist` undefined
    // and looping `\pgfplots@createlegend` at the digest wall-clock cap.
    let preserve = matches!(scope_opt, Some(Scope::Named(_) | Scope::InPlace));
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
    let mut scope = match scope_opt {
      Some(s) => s,
      None => {
        if self.get_prefix_sym(pin!("global")) {
          Scope::Global
        } else {
          Scope::Local
        }
      },
    };
    // A build-time local binding at the build's BASE level is global in
    // effect: eager builds after digestion, when no group can pop it. The
    // streaming driver interleaves the build with digestion, so the same
    // binding would land in a digestion frame and vanish at that group's end
    // (`with_build_floor`).
    if matches!(scope, Scope::Local) && BUILD_FLOOR.get() == Some(self.undo.len()) {
      scope = Scope::Global;
    }
    self.journal_touch(table_name, key);
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
      Scope::Local => self.assign_local_unconditional(table_name, key, value),
      Scope::InPlace => {
        // Perl `State.pm:175`: replace the front value in the frame it was last
        // bound in, adding NO undo entry, so the mutation keeps the binding's
        // save-stack level (the `\box` / same-level semantics). If the key was
        // never bound, seed it at the locked base frame (Perl's "push globally"
        // fallback). Mirrors the Value-table `assign_value_inplace` fast path.
        let state_table = self.table_mut(table_name);
        if let Some(defs) = state_table.get_mut(&key)
          && let Some(front) = defs.front_mut()
        {
          *front = value;
        } else {
          state_table.entry(key).or_default().push_front(value);
          for frame in &mut self.undo {
            if frame.locked {
              frame.table_mut(table_name).insert(key, 1);
              break;
            }
          }
        }
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
        self.journal_touch(TableName::Stash, scope_name);
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
    self.lookup_value_sym(arena::pin(key))
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
    let key = arena::pin(key);
    self.journal_touch(TableName::Value, key);
    match self.value.get_mut(&key) {
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
      // `\special_relax`-family tokens all resolve under the bare `\special_relax`
      // name (shared `\relax` meaning; identity lives in `noexpand_shadowed`).
      if name == pin!("") {
        None
      } else {
        Some(meaning_key(key))
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

/// A refused redefinition of a locked `\<name>` that EXTENDS the meaning it read — the locked binding's replacement
/// text with tokens appended or prepended, as etoolbox's `\appto`/`\preto` (`\edef#1{\expandonce#1…}`) and
/// latex.ltx's `\g@addto@macro` (`\toks@\expandafter{#1#2}\xdef#1{\the\toks@}`) make it — extends what the
/// class itself defined: in TeX the class's own definition is the meaning it extends, so the record of the class's
/// definition (`\lx@dropped@<name>`, `dropped_key`) becomes that definition with the same tokens appended or
/// prepended. wiley-article.cls:420-466 `\renewcommand{\maketitle}{…\@papertype…\@paperfield…}` then :576
/// `\appto{\maketitle}{…}`: the record held the kernel's body and the append, and the class's title page, which
/// the `\maketitle` deposit replays, was gone (1911.06213). `None` (the refused definition is recorded as it is)
/// unless the class's record and the binding are parameterless token bodies and the refused one is the binding's body
/// with tokens added at one end.
fn extended_refused_definition(
  token: &Token,
  definition: &Stored,
  dropped_key: SymStr,
) -> Option<Stored> {
  let parameterless = |d: &Expandable| {
    d.get_parameters()
      .is_none_or(|p| p.get_parameters().is_empty())
  };
  let Stored::Expandable(new) = definition else {
    return None;
  };
  let Some(Stored::Expandable(binding)) = lookup_meaning(token) else {
    return None;
  };
  let Some(Stored::Expandable(prior)) =
    lookup_meaning(&T_CS!(arena::with(dropped_key, ToString::to_string)))
  else {
    return None;
  };
  if !parameterless(new) || !parameterless(&binding) || !parameterless(&prior) {
    return None;
  }
  let (
    Some(ExpansionBody::Tokens(new_body)),
    Some(ExpansionBody::Tokens(kernel)),
    Some(ExpansionBody::Tokens(own)),
  ) = (
    new.get_expansion(),
    binding.get_expansion(),
    prior.get_expansion(),
  )
  else {
    return None;
  };
  let (new_body, kernel, own) = (new_body.unlist_ref(), kernel.unlist_ref(), own.unlist_ref());
  if kernel.is_empty() || new_body.len() <= kernel.len() {
    return None;
  }
  let extended: Vec<Token> = if new_body.starts_with(kernel) {
    own
      .iter()
      .chain(&new_body[kernel.len()..])
      .copied()
      .collect()
  } else if new_body.ends_with(kernel) {
    new_body[..new_body.len() - kernel.len()]
      .iter()
      .chain(own)
      .copied()
      .collect()
  } else {
    return None;
  };
  let mut kept = (*prior).clone();
  kept.expansion = Some(ExpansionBody::Tokens(Tokens::new(extended)));
  Some(Stored::Expandable(Rc::new(kept)))
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
  // A real definition retires the error-stub provenance (`is_error_stub`):
  // a name stubbed as `<ltx:ERROR/>` and later `\def`ed is a defined cs
  // again, so `\csname…\endcsname` takes tex.web's §373 path for it, not the
  // §370 discard. Probe-only, like the `:locked` check below.
  let stub_key = token.with_cs_name(|cs| s!("{cs}:error@stub"));
  if let Some(stub_sym) = arena::get(&stub_key)
    && lookup_bool_sym(stub_sym)
  {
    assign_value_sym(stub_sym, false, Some(Scope::Global));
  }
  // (read against this redefinition's own interface: a lock that holds only against copies, `files_lock_holds`; an
  // unlocked state reads no lock and allocates nothing here)
  let locked =
    !state_is_unlocked() && token.with_cs_name(|cs| redefinition_locked(cs, &definition));
  // (a stand-in's copy's own write to LaTeX's document hooks is document-time code, which it does not register, as it
  // does not its `\AtBeginDocument` code: `copy_provides_defers`. ltxgrid's `\prepdef\@begindocumenthook{\open@column@one
  // \@ne\set@colht…}` in qm2008_abs.cls and snp.cls, REVTeX 4 copies routed to the revtex4 binding, ran its page
  // grid over the binding's names at `\begin{document}`: 7 "Missing number" Warnings, 0805.2253, 0806.0481, 0805.4759)
  if !state_is_unlocked() && copy_provides_defers_hook_write(&token) {
    return;
  }
  // (a stand-in's copy's own title page, whatever becomes of it: what it prints decides its setters' defaults,
  // `copy_title_page_prints`)
  if let Stored::Expandable(ref defn) = definition
    && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
  {
    note_copy_provides_title_page(&token, body);
  }
  if locked && copy_provides_refuses_locked(&token) {
    restore_cleared_binding(&token);
  } else if locked {
    restore_cleared_binding(&token);
    // A LOCKED BINDING OWNS ITS CALLS. The lock keeps the semantic markup of a call (Perl acb3b27e: commands "that
    // lose semantics when redefined"), and a use without an argument cannot produce that markup: the binding would
    // read the next token as its argument (a pgffor `\foreach \i/\text` loop variable: 15 errors; `\@for\text:=`:
    // Fatal; `\def\title{On Widgets}` then `\section*{\title}`). So a VALUE the document defines — a definition
    // without parameters — of a binding that takes a braced argument ([`owns_calls`]; a robust binding by its inner
    // `\cs␣`, the kernel `\cite`: eplain's btxmac scratch, SYNC_STATUS 67n) becomes the meaning: what
    // `\ifx`, `\meaning` and every use without an argument see, as in pdflatex; the binding stays the handler of
    // its calls, as `\lx@locked@<cs>` ([`crate::definition::expandable::Expandable::yields_calls`]). A definition
    // with parameters defines a command, not a value, so every use of it stays the binding's (`\renewcommand\mathbf
    // [1]{\boldsymbol{#1}}` with `\mathbf a`; 2006.08408's `\renewcommand{\sqrt}[2][\mkern8mu]{…}` with
    // `\sqrt k`). Raw class and package code stays refused, as in Perl (State.pm:502-517), and so does a binding
    // without a braced interface (sectioning, `\footnote`, `\url`, `\maketitle`). The records below are written
    // either way, so calls behave as before. Accepted residual: a parameterless forwarder used without braces
    // (`\def\mathcal{\mathscr}` with `\mathcal C`; the `\mathpalette` `\sqrt` hack with `\sqrt n`, 1806.09639,
    // 1904.10871) runs the document's definition, as pdflatex does: author code is not analysed.
    // OXIDIZED_DESIGN #500; repros macro-state/m5_*; witnesses 2405.18512, 1406.2544, 1511.05179.
    let call_owner = match definition {
      Stored::Expandable(ref defn)
        if crate::definition::origin::current_origin()
          == crate::definition::origin::DefinitionOrigin::Document
          && defines_value(&token, defn) =>
      {
        lookup_meaning(&token)
          .filter(|binding| owns_calls(&token, binding))
          .map(|binding| (Rc::clone(defn), binding))
      },
      _ => None,
    };
    // THE DROPPED SETTER'S INTERNALS EXIST, EMPTY. A raw class that
    // redefines a locked frontmatter command as a plain setter —
    // afthesis.cls:520 `\def\author#1{\def\auth@r{#1}}` — expects the
    // internal it would define to exist for its own readers (`\flyleaf`,
    // `\titlepage` :637/:688 read `\auth@r`); afthesis itself initializes its
    // other internals empty at load (:494-495 `\def\ti@tle{}`). Perl
    // (`State.pm:502-517`) drops the redefinition and nothing else, so
    // `\auth@r` stays undefined (usethesis). The locked binding remains the
    // sole semantic source (running the setter's body with the real
    // arguments would duplicate the author through the class layout), and
    // every `\def`/`\gdef`/`\edef`/`\xdef <cs>` in the dropped body whose
    // `<cs>` is still undefined gets an EMPTY definition — the class's own
    // default convention, with no name enumerated. Guard:
    // `perfect_kernel_batch54::refused_setter_internals_take_the_replayed_value`.
    if let Stored::Expandable(ref defn) = definition
      && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
    {
      let body = body.unlist_ref();
      let def_names = ["\\def", "\\gdef", "\\edef", "\\xdef"];
      for (i, t) in body.iter().enumerate() {
        if t.get_catcode() == Catcode::CS
          && t.with_cs_name(|n| def_names.contains(&n))
          && let Some(target) = body.get(i + 1)
          && target.get_catcode() == Catcode::CS
          && lookup_meaning(target).is_none()
          && let Ok(empty) = Expandable::new(
            *target,
            None,
            Some(ExpansionBody::Tokens(Tokens::new(Vec::new()))),
            None,
          )
        {
          state_mut!().assign_internal(
            TableName::Meaning,
            target.get_cs_name(),
            Stored::Expandable(Rc::new(empty)),
            Some(Scope::Global),
          );
        }
      }
    }
    // Keep the dropped definition itself, as `\lx@dropped@<name>`: a locked
    // binding may run what the class wrote where its own semantics leave a gap —
    // `\lx@deposit@maketitle` (sect05.rs) runs a class's title-page body for the
    // fields the frontmatter does not carry (ryethesis.cls:282 degree/program/
    // university). Nothing runs it unless a binding asks. OXIDIZED_DESIGN #265.
    // Macros only: a later `\let<cs>\relax` (article-style classes end their
    // `\maketitle` with one) must not overwrite the body it follows.
    if matches!(definition, Stored::Expandable(_)) {
      let dropped_key =
        arena::pin(token.with_cs_name(|cs| s!("\\lx@dropped@{}", cs.trim_start_matches('\\'))));
      let kept = extended_refused_definition(&token, &definition, dropped_key)
        .unwrap_or_else(|| definition.clone());
      state_mut!().assign_internal(TableName::Meaning, dropped_key, kept, Some(Scope::Global));
    }
    // Record that a redefinition was dropped: a locked binding may consult
    // `<cs>:redefined` to yield the parts of its behaviour the class took
    // over (see `\lx@maketitle@cleanup`).
    // (a class whose load defines the name the lock kept defines it all the same, as its real file does:
    // `ClassLoadFrame`, `copy_base_class_defines`)
    note_class_load_name(meaning_key(&token));
    let redefined_key = arena::pin(token.with_cs_name(|cs| s!("{cs}:redefined")));
    state_mut!().assign_internal(
      TableName::Value,
      redefined_key,
      Stored::Bool(true),
      Some(Scope::Global),
    );
    // …and the dropped setter's SIGNATURE: an argument-taking `\def\abstract#1`
    // (apa7.cls:785, uwthesis.cls:805) means the class reads `{…}` as one
    // pre-tokenized argument stored for later, which the locked binding can
    // honour (`\abstract`, sect05.rs). `\newcommand` records it at its own
    // locked bail (sect08.rs).
    if let Stored::Expandable(ref defn) = definition {
      let nargs = defn
        .get_parameters()
        .map_or(0, |p| p.get_parameters().len());
      let nargs_key = arena::pin(token.with_cs_name(|cs| s!("{cs}:redefined@nargs")));
      state_mut!().assign_internal(
        TableName::Value,
        nargs_key,
        Stored::Number(Number::new(nargs as i64)),
        Some(Scope::Global),
      );
    }
    // …and, against a lock that holds only against copies, the default of the optional argument the copy takes first
    // (`[1][\thu@proof@name]`; a robust copy's by its inner `\cs␣`), which the binding titles its own use with
    // (amsthm's proof, amsthm_sty.rs).
    if let Some(default) = copy_default(&definition) {
      record_redefined_default(&token, default);
    }
    if let Some((defn, binding)) = call_owner {
      // The binding, kept on the first override (a later one, a loop's next iteration, overrides an override).
      if !matches!(binding, Stored::Expandable(ref b) if b.yields_calls) {
        let owner = locked_binding_token(&token);
        state_mut!().assign_internal(
          TableName::Meaning,
          owner.get_cs_name(),
          binding,
          Some(Scope::Global),
        );
      }
      let mut overriding = (*defn).clone();
      overriding.yields_calls = true;
      state_mut!().assign_internal(
        TableName::Meaning,
        cs_sym,
        Stored::Expandable(Rc::new(overriding)),
        scope,
      );
      // One report per control sequence and document (a loop overrides it on every iteration).
      let reported_key = arena::pin(token.with_cs_name(|cs| s!("{cs}:lock@reported")));
      if !lookup_bool_sym(reported_key) {
        assign_value_sym(reported_key, true, Some(Scope::Global));
        let cs_name = token.with_cs_name(ToString::to_string);
        Info!(
          "lock",
          cs_name,
          s!(
            "{cs_name} is locked: its calls keep the binding; its other uses take the document's definition"
          )
        );
      }
      return;
    }
    if let Some(Stored::String(s)) = state!().lookup_value("SOURCEFILE") {
      // report if the redefinition seems to come from document source
      if arena::with(*s, |txt| {
        txt == "Anonymous String" || TEX_OR_BIB_EXT_RE.is_match(txt) && !txt.ends_with(CODE_TEX_EXT)
      }) {
        // Perl `State.pm` L514 reports the CS itself — `Ignoring redefinition
        // of \cite` — not the lookup key. Reporting `lock_key` here named a
        // control sequence that does not exist (`\cite:locked`), so the one
        // diagnostic for a refused redefinition did not grep for the command
        // it was about.
        let cs_name = token.with_cs_name(ToString::to_string);
        Info!("ignore", cs_name, s!("Ignoring redefinition of {cs_name}"));
      }
    }
  } else {
    note_copy_provides_assignment(&token);
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
        s!("Counter '{}' was not defined; assuming 0", ctr)
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
    let setters = [
      Expandable::new(
        T_CS!(s!("\\{}true", name)),
        None,
        Some(s!("\\let{}\\iftrue", cs).into()),
        None,
      )?,
      Expandable::new(
        T_CS!(s!("\\{}false", name)),
        None,
        Some(s!("\\let{}\\iffalse", cs).into()),
        None,
      )?,
    ];
    let scope = stub_scope(&cs);
    past_the_lock(|| {
      for setter in setters {
        install_definition(setter, Some(Scope::Global));
      }
      let_i(token, &T_CS!("\\iffalse"), Some(scope));
    });
    crate::util::logger::note_held_stub(*token);
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
    let scope = stub_scope(&cs);
    let stub = Constructor {
      cs: *token,
      replacement: Some(Rc::new(move |document, _args, _props| {
        document.make_error("undefined", &cs)
      })),
      ..Constructor::default()
    };
    //TODO: sizer => "X"),
    past_the_lock(|| install_definition(stub, Some(scope)));
    // The stub's provenance travels with the NAME (tex.web §370 discards an
    // undefined cs inside `\csname…\endcsname`; a defined non-expandable one
    // takes the §373 `back_error` path): `<cs>:error@stub`, read by
    // `gullet::read_cs_name_inner` — not a global "last stubbed token", which
    // went stale across interleaved stubs.
    // (at the stub's own scope: a locked name's stub that ends with its group leaves the binding, which is no stub)
    assign_value_sym(
      arena::pin(token.with_cs_name(|c| s!("{c}:error@stub"))),
      true,
      Some(scope),
    );
    crate::util::logger::note_held_stub(*token);
  }
  Ok(*token)
}

/// The scope [`generate_error_stub`] installs a stub at: global, as Perl's `generateErrorStub` (State.pm:553
/// `installDefinition(…, 'global')`), except for a name that carries a lock ([`name_carries_lock`]). Such a name is
/// undefined only because a `\let` cleared its binding, and that `\let` may be local: the stub is the recovery for
/// this clearing, so it is local too and ends where the clearing ends — `{\let\cite\undefined X\cite{a}} Y\cite{b}`
/// typesets the second citation, as pdflatex does, and `\begingroup\let\section\nonexist\section{A}\endgroup
/// \section{B}` keeps the section B. A clearing at the top level leaves the stub for the rest of the document, one
/// Error per name as before. Guards `perfect_kernel_batch68u::a_locked_name_cleared_in_a_group_is_back_after_it`.
fn stub_scope(cs: &str) -> Scope {
  if name_carries_lock(cs) {
    Scope::Local
  } else {
    Scope::Global
  }
}

/// Run [`generate_error_stub`]'s installs with the state unlocked. The stub is the engine's recovery from an undefined
/// control sequence, not a source redefinition, so a lock does not refuse it: a document's `\let` of a locked binding to
/// an undefined name (`\let\cite\parencite` with `\parencite` undefined, 2006.07968's apa6 paper; pittetd.cls:313
/// `\let\maketitle\HyOrg@maketitle`, 1509.06375) clears the binding, and a refused stub left the name undefined for the
/// Stomach's retry (stomach.rs, Perl Stomach.pm:236-237 `invokeToken_undefined`), which raised the same Error until
/// TooManyErrors. Installed, the name gives one Error and its `<ltx:ERROR/>`, as pdflatex gives one "Undefined control
/// sequence" per use. Perl refuses its own locked names' stubs the same way (State.pm:509; `\let\section\nonexist`:
/// 101 Errors and Fatal, KNOWN_PERL_ERRORS #841). OXIDIZED_DESIGN_DIVERGENCES #641; repro
/// macro-state/locked_name_let_to_undefined_stub_loop; guard
/// `perfect_kernel_batch68u::a_locked_name_let_to_an_undefined_one_errs_once`.
fn past_the_lock(install: impl FnOnce()) {
  let _unlocked = local_state_unlocked_guard(true);
  install();
}

/// Undo [`generate_error_stub`]: `token` (and a stubbed `\ifX`'s `\Xtrue`/`\Xfalse`) is undefined
/// again. A discarded [`crate::util::logger::DiagnosticsHold`] takes back its scope's errors and the
/// stubs standing in for them, so the document's own later use of the command reports it.
pub fn remove_error_stub(token: &Token) {
  let cs = token.with_cs_name(ToString::to_string);
  // A locked name's stub was local ([`stub_scope`]): undone only while it still stands — not after the group it was
  // made in has ended, which gave the binding back — and undone there, so the binding of the enclosing groups stands.
  let scope = stub_scope(&cs);
  let local = matches!(scope, Scope::Local);
  if let Some(name) = cs.strip_prefix("\\if") {
    for setter in [T_CS!(s!("\\{name}true")), T_CS!(s!("\\{name}false"))] {
      assign_meaning(&setter, Stored::None, Some(Scope::Global));
    }
    if local && !x_equals(token, &T_CS!("\\iffalse")) {
      return;
    }
  } else {
    if local && !is_error_stub(token) {
      return;
    }
    assign_value_sym(arena::pin(s!("{cs}:error@stub")), false, Some(scope));
  }
  assign_meaning(token, Stored::None, Some(scope));
}

/// Was `token` stubbed as `<ltx:ERROR/>` by [`generate_error_stub`]? Probe
/// only: the flag key is interned when the stub is made, so an unknown name
/// costs no arena entry (the `:locked` probe's discipline).
pub fn is_error_stub(token: &Token) -> bool {
  let key = token.with_cs_name(|c| s!("{c}:error@stub"));
  arena::get(&key).is_some_and(lookup_bool_sym)
}

/// Install a `Constructor` for `token` whose sole effect at digestion time is to
/// emit `<ltx:ERROR class='undefined'>content</ltx:ERROR>` (the Rust equivalent
/// of Perl `Document::makeError`). It logs NOTHING — the caller is responsible
/// for the `Error!`/`note_status`. Mirrors the make_error constructor that
/// `generate_error_stub` installs for undefined *commands*, so undefined
/// *environments* (`\begin{undefinedenv}`) leave the same visible
/// `<ltx:ERROR>` marker as Perl instead of silently vanishing from the output.
/// Like that stub, it installs past a lock (`past_the_lock`, below).
pub fn install_undefined_error_constructor(token: Token, content: &str) {
  let content = content.to_string();
  let stub = Constructor {
    cs: token,
    replacement: Some(Rc::new(move |document, _args, _props| {
      document.make_error("undefined", &content)
    })),
    ..Constructor::default()
  };
  let scope = token.with_cs_name(stub_scope);
  past_the_lock(|| install_definition(stub, Some(scope)));
}

/// Does a provide-only load ([`ProvideOnlyGuard`](crate::definition::origin::ProvideOnlyGuard)) decline this definition
/// of `token`? While the guard lives, a binding's definition (`DefinitionOrigin::Binding`, `Fallback` or `Borrowed`:
/// OmniBus and every binding it requires, inst_support's `\author[]{}` and epsf's `\epsfbox` among them) is declined
/// when the name was defined before the guard: defined now in the `\providecommand` sense (latex.ltx `\@ifundefined`: a
/// meaning other than `\relax`, `binding::def::dialect::is_defined_token`), not a fallback guess
/// (`is_fallback_meaning`), and not assigned since the guard began (`provide_only_assigned`: the load's own names,
/// which it may define again — OmniBus's `\keywords{}` before its `\keywords` that also opens `{keywords}`). An
/// environment `\begin{X}`/`\end{X}` counts as defined when `\X` is (latex.ltx `\begin` runs `\X`), and `\end{X}` also
/// when `\endX` is. The binding definers (`def_macro`, `def_primitive`, `def_constructor`, `def_register`,
/// `def_conditional`, `def_environment`, `def_math`) and `Let!` consult it before they assign anything; TeX's own
/// `\def`/`\let` do not, so TeX code run during the load keeps its usual effect (a hook appended to, a scratch macro),
/// nor does the loader's bookkeeping, made at its caller's origin. A macro marked `extends` (it wraps and delegates to
/// the name's current meaning) is not declined.
pub fn provide_only_declines(token: &Token) -> bool {
  use crate::{
    binding::def::dialect::is_defined_token,
    definition::origin::{DefinitionOrigin, current_origin, provide_only},
  };
  if !provide_only()
    || !matches!(
      current_origin(),
      DefinitionOrigin::Binding | DefinitionOrigin::Fallback | DefinitionOrigin::Borrowed
    )
  {
    return false;
  }
  let defined = |token: &Token| {
    is_defined_token(token)
      && !is_fallback_meaning(token)
      && !state!().provide_only_assigned.contains(&meaning_key(token))
  };
  if defined(token) {
    return true;
  }
  token.with_cs_name(|name| {
    let env = |prefix: &str| {
      name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix('}'))
    };
    if let Some(env) = env("\\begin{") {
      defined(&T_CS!(s!("\\{env}")))
    } else if let Some(env) = env("\\end{") {
      defined(&T_CS!(s!("\\{env}"))) || defined(&T_CS!(s!("\\end{env}")))
    } else {
      false
    }
  })
}

/// "A stand-in's copy provides" (OXIDIZED_DESIGN_DIVERGENCES #731): note this assignment of `token`'s meaning, made
/// while the paper's copy of an author file is read after the binding that stands in for it
/// (`binding::content::read_stand_in_copy`), or while the `\AtEndOfClass`/`\AtEndOfPackage` code it registered runs
/// at the end of that load ([`copy_provides_wrap`]). Every assignment of a meaning goes through here — TeX's `\def`
/// family, `\let`, `\futurelet`, `\chardef` and the `\countdef` family, and the LaTeX declarators (`\newcommand`,
/// `\renewcommand`, `\DeclareRobustCommand`, `\newenvironment`, `\newcounter`, `\newif`, `\newtheorem`, `\newlength`…),
/// at [`install_definition`] and [`assign_meaning`]. One of a name LaTeXML's semantic layer owns
/// (`owned_by_the_semantic_layer`; an environment `{X}` is owned when `\begin{X}`, `\X` or `\endX` is) holds only
/// inside the copy's own code: the binding's meaning is kept aside and put back when the copy's read ends
/// ([`leave_copy_provides_frame`]), so for the document the definition acted as `\providecommand` does, while the
/// copy's load-time logic sees what it defined, as in TeX: aipproc.cls:559 `\ifnum\pageref>0` after its option's
/// `\chardef\pageref` (astro-ph0105128); epl2.cls:1317-1318's `\let\epl@org@bibitem\bibitem` before its `\def\bibitem`,
/// which the epl binding makes too (a kept `\let` would take the binding's wrapper for the original, and `\bibitem`
/// would call itself: 0906.2627, 1102.2776, 1203.4765). Every other assignment stands as TeX makes it: an undefined
/// name (the copy's extras), a raw kernel name, an earlier raw file's or the document's, a stub or borrow. Not while
/// the state is unlocked (a binding's `before`/`after` code, an `AddToMacro!` append), nor for a locked name, which
/// `install_definition` refuses first ([`copy_provides_refuses_locked`]). Each name kept for the binding is logged
/// once per copy, at Info.
pub fn note_copy_provides_assignment(token: &Token) {
  let id = crate::definition::origin::copy_provides();
  if id == 0 || state_is_unlocked() || !matches!(token.get_catcode(), Catcode::CS | Catcode::ACTIVE)
  {
    return;
  }
  let key = meaning_key(token);
  let Some(at) = state!()
    .copy_provides_frames
    .iter()
    .rposition(|frame| frame.id == id && !frame.repeat)
  else {
    return;
  };
  if state!().copy_provides_frames[at].kept_keys.contains(&key) {
    return;
  }
  // (a name whose earlier assignment in the copy's code stood is the copy's: `\let\refname\relax` then
  // `\newcommand\refname{…}`, as LaTeX's `\newcommand` finds a `\relax` name definable)
  if state!().copy_provides_frames[at].assigned.contains(&key) {
    return;
  }
  // (a counter's representation and reset list are owned only when the stand-in binding's own load assigned them:
  // `counter_part`)
  // (an environment's names are provided or declined together: once the copy's code redefined one the binding owns —
  // its `\begin{X}`, kept aside — the others it defines next are kept too, though the name it redefined now carries
  // the copy's meaning; cimart.cls:295-315's `\newtheorem{theorem}[definition]` under OmniBus's `{theorem}` left the
  // binding's `\begin{theorem}` with the copy's `\end{theorem}`, a group open at the document's end: 2412.00032,
  // 2408.07575's `{example}`)
  let pair_kept = token.with_cs_name(|name| {
    environment_names(name).iter().any(|other| {
      state!().copy_provides_frames[at]
        .kept_keys
        .contains(&meaning_key(&T_CS!(other.as_str())))
    })
  });
  let owned = if token.with_cs_name(counter_part).is_some() {
    state!().stand_in_meanings.contains(&key)
  } else if unowned_text_default(token) || base_class_name_made_relax(token) {
    false
  } else {
    // (not by its macro partner alone: plain's `\endgroup` does not make a copy's `\group` LaTeXML's)
    let environment = |name: &str| name.starts_with("\\begin{") || name.starts_with("\\end{");
    pair_kept
      || owned_by_the_semantic_layer(token)
      || token.with_cs_name(|name| {
        environment_names(name)
          .iter()
          .filter(|other| environment(name) || environment(other))
          .any(|other| owned_by_the_semantic_layer(&T_CS!(other.as_str())))
      })
  };
  if !owned {
    state_mut!().copy_provides_frames[at].assigned.insert(key);
    return;
  }
  let binding = lookup_meaning(token).unwrap_or(Stored::None);
  let mut state = state_mut!();
  let frame = &mut state.copy_provides_frames[at];
  frame.kept.push((key, binding));
  frame.kept_keys.insert(key);
}

/// Is `token` a name the stand-in binding's base class assigned (`State::stand_in_base_defaults`) that a file since
/// made `\relax` — a raw base class the copy loads: `\let\figurename\relax` before the copy's
/// `\newcommand\figurename{Fig}`? LaTeX finds a `\relax` name undefined (`\newcommand`, `\@ifundefined`), so the copy's
/// definition stands, as in TeX.
fn base_class_name_made_relax(token: &Token) -> bool {
  state!()
    .stand_in_base_defaults
    .contains(&meaning_key(token))
    && lookup_meaning(token)
      .is_some_and(|meaning| Some(meaning) == lookup_meaning(&crate::token::TOKEN_RELAX))
}

/// Does a stand-in's copy, read or running its hook now ([`note_copy_provides_assignment`]), redefine the locked
/// `token`? Then the lock refuses it as for any file, but keeps no record of it — no `\lx@dropped@<name>` body for
/// the binding to run in a gap, no `<cs>:redefined` for it to yield to, no default or signature (the records of a raw
/// class taking over a locked command, OXIDIZED_DESIGN #265, #500): the binding stands in for this very file, so the
/// copy takes nothing over. Logged once per copy, at Info, as a redefinition of an owned name is.
pub fn copy_provides_refuses_locked(token: &Token) -> bool {
  let id = crate::definition::origin::copy_provides();
  if id == 0 {
    return false;
  }
  let key = meaning_key(token);
  let mut state = state_mut!();
  if let Some(copy) = state.copy_provides_copies.get_mut(id as usize - 1)
    && copy.declined.insert(key)
  {
    let file = copy.file.clone();
    drop(state);
    let name = token.with_cs_name(ToString::to_string);
    Info!(
      "provide",
      name,
      s!("{name} keeps LaTeXML's locked definition: the paper's {file} redefines it")
    );
  }
  true
}

/// Does a stand-in's copy, read now ([`note_copy_provides_assignment`]), save `source` again into `token` — `\let
/// \token\source` — where `token` already holds an earlier meaning of `source` (its definition is a `\let` copy made
/// under `source`'s name, `definition_cs`) that the stand-in binding's load assigned, and `source` has been redefined
/// since? Then the `\let` is declined, in the copy's code too: `token` keeps the meaning TeX gives it, `source` before
/// the redefinition. The binding (or a file it read) made the very save-and-wrap the copy makes — `\let\org\X`, then
/// `\X` redefined to call `\org` —, and saving the wrapper into `\org` makes `\X` call itself: myjmlr.cls and
/// jmlr_mod.cls (renamed jmlr.cls copies routed to the jmlr binding, whose jmlrutils.sty made the same wrap) run
/// `\let\jmlr@org@newtheorem\newtheorem` (jmlr.cls:753-754) before their own `\newtheorem{example}`, which looped to
/// the timeout (1506.02582, 2304.14217; OXIDIZED_DESIGN_DIVERGENCES #731 rule 2, the epl2.cls `\let\epl@org@bibitem`
/// case met in the copy's own code). Logged once per copy, at Info.
pub fn copy_provides_keeps_saved_meaning(token: &Token, source: &Token, meaning: &Stored) -> bool {
  let id = crate::definition::origin::copy_provides();
  if id == 0
    || state_is_unlocked()
    || !matches!(token.get_catcode(), Catcode::CS | Catcode::ACTIVE)
    || token == source
  {
    return false;
  }
  let key = meaning_key(token);
  if !state!().stand_in_meanings.contains(&key) {
    return false;
  }
  let Some(saved) = lookup_meaning(token) else {
    return false;
  };
  if definition_cs(&saved) != Some(*source) || same_meaning(&saved, meaning) {
    return false;
  }
  let mut state = state_mut!();
  if let Some(copy) = state.copy_provides_copies.get_mut(id as usize - 1)
    && copy.declined.insert(key)
  {
    let file = copy.file.clone();
    drop(state);
    let name = token.with_cs_name(ToString::to_string);
    let source_name = source.with_cs_name(ToString::to_string);
    Info!(
      "provide",
      name,
      s!(
        "{name} keeps its saved {source_name}: the paper's {file} saves {source_name} again after its binding \
         redefined it"
      )
    );
  }
  true
}

/// Does a stand-in's copy, read now ([`note_copy_provides_assignment`]), (re)declare or renumber the counter `ctr` the
/// stand-in binding's own load declared or numbered (its `\c@<ctr>` assigned while the binding loaded,
/// `State::stand_in_meanings`, or the `\the<ctr>` of a counter that exists: `\newcounter`, `\numberwithin`,
/// `\@addtoreset`, `\counterwithin`)? Then the counter keeps the binding's numbering: its reset list is the binding's, as
/// its `\the<ctr>` is. Any other counter is the copy's, declared for real (`new_counter` suspends the copy's
/// provides-mode for it): the kernel's `equation` with the pool's article `\theequation` — as article.cls's defaults
/// yield to the real class's own: jheppub.sty:313-314
/// `\renewcommand\theequation{\thesection.\arabic{equation}}` and `\numberwithin{equation}{section}` number (2.1) as its
/// PDF does (2112.14648, 2607.20603) — and a counter the binding only has a `\the<ctr>` for, OmniBus's `\thechapter`
/// autoload stub: WileyNJDv5.cls:5728 and MRM.cls:2160 `\newcounter{chapter}[part]`, ws-rv9x6.cls:399
/// `\newcounter{chapter}` (2301.03442: declined, `\c@chapter` was "not a register"). Logged once per copy, at Info.
pub fn copy_provides_keeps_counter(ctr: &str) -> bool {
  let id = crate::definition::origin::copy_provides();
  if id == 0 || state_is_unlocked() {
    return false;
  }
  let counter = T_CS!(s!("\\c@{ctr}"));
  let numbered = |token: &Token| state!().stand_in_meanings.contains(&meaning_key(token));
  // (a `\the<ctr>` numbers a counter that exists — the kernel's `equation` — not one it only stands for: OmniBus's
  // `\thechapter` autoload stub with no `\c@chapter`)
  let exists = lookup_meaning(&counter).is_some_and(|meaning| !matches!(meaning, Stored::None));
  if !numbered(&counter) && !(exists && numbered(&T_CS!(s!("\\the{ctr}")))) {
    return false;
  }
  let key = meaning_key(&counter);
  let mut state = state_mut!();
  if let Some(copy) = state.copy_provides_copies.get_mut(id as usize - 1)
    && copy.declined.insert(key)
  {
    let file = copy.file.clone();
    drop(state);
    Info!(
      "provide",
      ctr,
      s!("Counter {ctr} keeps LaTeXML's numbering: the paper's {file} redeclares or renumbers it")
    );
  }
  true
}

/// The counter whose representation or reset list `name` is — `\the<ctr>`, `\p@<ctr>`, `\cl@<ctr>` of a counter
/// `\c@<ctr>` — for [`note_copy_provides_assignment`] and [`copy_provides_keeps_counter`].
fn counter_part(name: &str) -> Option<String> {
  let ctr = name
    .strip_prefix("\\the")
    .or_else(|| name.strip_prefix("\\p@"))
    .or_else(|| name.strip_prefix("\\cl@"))?;
  if ctr.is_empty() {
    return None;
  }
  matches!(
    lookup_meaning(&T_CS!(s!("\\c@{ctr}"))),
    Some(Stored::Register(_))
  )
  .then(|| ctr.to_string())
}

/// Keep a stand-in copy's own title page (`\maketitle`, `\@maketitle`), defined while it is read
/// ([`install_definition`], `\newcommand`'s refusal), whether the binding's lock refuses it or the overlay holds it:
/// what it prints decides which of the copy's setter defaults the document shows (`frontmatter_stores::copy_title_page_prints`,
/// latexml_engine).
pub fn note_copy_provides_title_page(token: &Token, body: &Tokens) {
  let id = crate::definition::origin::copy_provides();
  if id == 0 || !matches!(token.get_catcode(), Catcode::CS) {
    return;
  }
  if !token.with_cs_name(|name| matches!(name, "\\maketitle" | "\\@maketitle")) {
    return;
  }
  if let Some(copy) = state_mut!().copy_provides_copies.get_mut(id as usize - 1) {
    copy.title_page.push(body.clone());
  }
}

/// Observe the calls of the setters `names` ([`State::watched_setters`]).
pub fn watch_setters(names: &[&str]) {
  let keys: Vec<SymStr> = names.iter().map(|name| meaning_key(&T_CS!(name))).collect();
  state_mut!().watched_setters.extend(keys);
}

/// A macro `cs` is called (`Expandable::invoke`, while [`crate::definition::origin::setter_watch`]): a watched setter
/// called outside a stand-in copy's read is the document's (or its binding's) call, `<cs>:called`, whatever its
/// arguments — `\author{}` as much as `\author{Jane Doe}`, which a binding's accumulating `\author` (inst_support's)
/// does not queue nor clear.
pub fn note_setter_call(cs: &Token) {
  if crate::definition::origin::copy_provides() != 0 {
    return;
  }
  let key = meaning_key(cs);
  if state!().watched_setters.contains(&key) {
    assign_value(
      &cs.with_cs_name(|name| s!("{name}:called")),
      true,
      Some(Scope::Global),
    );
  }
}

/// Was the watched setter `cs` called outside a stand-in copy's read ([`note_setter_call`])?
pub fn setter_called(cs: &Token) -> bool {
  lookup_bool(&cs.with_cs_name(|name| s!("{name}:called")))
}

/// The title pages stand-in copies defined ([`note_copy_provides_title_page`]), each with its copy's number, for
/// `frontmatter_stores::copy_title_page_prints` (latexml_engine).
pub fn copy_provides_title_pages() -> Vec<(u32, Tokens)> {
  state!()
    .copy_provides_copies
    .iter()
    .enumerate()
    .flat_map(|(index, copy)| {
      copy
        .title_page
        .iter()
        .map(move |body| (index as u32 + 1, body.clone()))
    })
    .collect()
}

/// Copy `id`'s own version of the name `token` the binding owns, kept in its overlay while the binding's meaning is in
/// force ([`note_copy_provides_assignment`]).
pub fn copy_provides_overlaid_meaning(id: u32, token: &Token) -> Option<Stored> {
  state!()
    .copy_provides_copies
    .get((id as usize).checked_sub(1)?)
    .and_then(|copy| copy.overlay.get(&meaning_key(token)).cloned())
}

/// Begin a frame of copy `id`'s provides-mode ([`crate::definition::origin::enter_copy_provides`]; `0` suspends the mode for
/// a file the copy loads, a repeat of the mode in force adds nothing): the copy's own definitions of names the binding
/// owns, kept from its earlier read or hooks, are in force again for its code, the meanings they replace kept aside.
pub fn enter_copy_provides_frame(id: u32, previous: u32) {
  let mut kept = Vec::new();
  let mut kept_keys = rustc_hash::FxHashSet::default();
  if id != 0 && id != previous {
    let overlay: Vec<(SymStr, Stored)> = state!()
      .copy_provides_copies
      .get(id as usize - 1)
      .map(|copy| copy.overlay.iter().map(|(k, v)| (*k, v.clone())).collect())
      .unwrap_or_default();
    for (key, meaning) in overlay {
      let current = meaning_by_key(key);
      kept.push((key, current));
      kept_keys.insert(key);
      state_mut!().assign_internal(TableName::Meaning, key, meaning, Some(Scope::Global));
    }
  }
  state_mut!().copy_provides_frames.push(CopyProvidesFrame {
    id,
    repeat: id == previous,
    kept,
    kept_keys,
    assigned: rustc_hash::FxHashSet::default(),
  });
}

/// End the frame [`enter_copy_provides_frame`] began: each name the copy's code redefined, and the binding owns, takes
/// back the meaning kept aside, and the copy's own is kept for its next frame (a hook it registered).
pub fn leave_copy_provides_frame() {
  let Some(frame) = state_mut!().copy_provides_frames.pop() else {
    return;
  };
  if frame.id == 0 || frame.repeat {
    return;
  }
  // A name the copy `\let` to its own version of a name the binding owns takes the binding's back with it:
  // emulateapj5.sty:386 defines `\keywords`, :394 `\let\subjectheadings=\keywords`, and the document's
  // `\subjectheadings` made the class's "Subject headings:" paragraph where the binding makes `<keywords>`
  // (astro-ph0011001). Kept in the copy's overlay with the rest, for its hooks. Only a definition the copy made under
  // the kept name itself: a kept flag the copy `\let` to `\iftrue` shares that object with every other true flag
  // (elife.cls:336-337's `\ifstre@msuppinfoopen`, newfile.sty:91: 2303.08822).
  let replaced: Vec<(Stored, Stored)> = frame
    .kept
    .iter()
    .map(|(key, kept)| (*key, meaning_by_key(*key), kept.clone()))
    .filter(|(key, copys, kept)| {
      !same_meaning(copys, kept) && definition_cs(copys).is_some_and(|cs| meaning_key(&cs) == *key)
    })
    .map(|(_, copys, kept)| (copys, kept))
    .collect();
  if !replaced.is_empty() {
    for key in &frame.assigned {
      let current = meaning_by_key(*key);
      if let Some((_, kept)) = replaced
        .iter()
        .find(|(copys, _)| same_meaning(copys, &current))
      {
        let mut state = state_mut!();
        if let Some(copy) = state.copy_provides_copies.get_mut(frame.id as usize - 1) {
          copy.overlay.insert(*key, current);
        }
        state.assign_internal(TableName::Meaning, *key, kept.clone(), Some(Scope::Global));
      }
    }
  }
  for (key, kept) in frame.kept.into_iter().rev() {
    let copys = meaning_by_key(key);
    // (a name the copy's code left as it found it — the kernel's `\protected@edef` lets `\protect` aside and back —
    // was not redefined: nothing to log)
    let redefined = !same_meaning(&copys, &kept);
    let mut state = state_mut!();
    let mut logged = None;
    if let Some(copy) = state.copy_provides_copies.get_mut(frame.id as usize - 1) {
      if redefined && copy.declined.insert(key) {
        logged = Some(copy.file.clone());
      }
      copy.overlay.insert(key, copys);
    }
    state.assign_internal(TableName::Meaning, key, kept, Some(Scope::Global));
    drop(state);
    if let Some(file) = logged {
      let name = arena::to_string(key);
      Info!(
        "provide",
        name,
        s!(
          "{name} keeps LaTeXML's definition: the paper's {file} redefines it for its own code only"
        )
      );
    }
  }
}

/// Are `a` and `b` the same meaning: the same definition object (a `\let` copies the object), or equal otherwise?
fn same_meaning(a: &Stored, b: &Stored) -> bool {
  match (a, b) {
    (Stored::Expandable(x), Stored::Expandable(y)) => Rc::ptr_eq(x, y),
    (Stored::Constructor(x), Stored::Constructor(y)) => Rc::ptr_eq(x, y),
    (Stored::Primitive(x), Stored::Primitive(y)) => Rc::ptr_eq(x, y),
    (Stored::MathPrimitive(x), Stored::MathPrimitive(y)) => Rc::ptr_eq(x, y),
    (Stored::Conditional(x), Stored::Conditional(y)) => Rc::ptr_eq(x, y),
    (Stored::Register(x), Stored::Register(y)) => Rc::ptr_eq(x, y),
    (Stored::Token(x), Stored::Token(y)) => x == y,
    (Stored::None, Stored::None) => true,
    _ => false,
  }
}

/// The meaning bound to `key` (a [`meaning_key`]), `Stored::None` when there is none.
fn meaning_by_key(key: SymStr) -> Stored {
  state!()
    .meaning
    .get(&key)
    .and_then(|entry| entry.front().cloned())
    .unwrap_or(Stored::None)
}

/// Number a new stand-in copy of `file` for [`note_copy_provides_assignment`] (1-based, [`crate::definition::origin::copy_provides`]).
pub fn new_copy_provides_record(file: &str, binding: &str) -> u32 {
  let mut state = state_mut!();
  state.copy_provides_copies.push(CopyProvidesRecord {
    file: file.to_string(),
    binding: binding.to_string(),
    ..CopyProvidesRecord::default()
  });
  state.copy_provides_copies.len() as u32
}

/// The other names of the environment a name `\X`, `\endX`, `\begin{X}` or `\end{X}` belongs to (latex.ltx `\begin{X}`
/// runs `\X`, `\end{X}` runs `\endX`): for `\X` the binding's `\begin{X}`, `\end{X}` and the macro `\endX`, for `\endX`
/// `\begin{X}`, `\end{X}` and `\X`; for `\begin{X}` the macros `\X`, `\endX` and `\end{X}`, for `\end{X}` `\X`, `\endX`
/// and `\begin{X}`.
fn environment_names(name: &str) -> Vec<String> {
  if let Some(env) = name
    .strip_prefix("\\begin{")
    .and_then(|rest| rest.strip_suffix('}'))
  {
    return vec![s!("\\{env}"), s!("\\end{env}"), s!("\\end{{{env}}}")];
  }
  if let Some(env) = name
    .strip_prefix("\\end{")
    .and_then(|rest| rest.strip_suffix('}'))
  {
    return vec![s!("\\{env}"), s!("\\end{env}"), s!("\\begin{{{env}}}")];
  }
  match name.strip_prefix('\\') {
    Some(macro_name) if !macro_name.is_empty() => {
      match macro_name.strip_prefix("end").filter(|env| !env.is_empty()) {
        Some(env) => vec![s!("\\begin{{{env}}}"), s!("\\end{{{env}}}"), s!("\\{env}")],
        None => vec![
          s!("\\begin{{{macro_name}}}"),
          s!("\\end{{{macro_name}}}"),
          s!("\\end{macro_name}"),
        ],
      }
    },
    _ => Vec::new(),
  }
}

/// Is `token`'s meaning a text default: a public name (letters only) whose meaning is a macro without parameters
/// expanding to text (characters, at least one letter: `\refname`'s "References")?
fn text_default(token: &Token) -> bool {
  let public = token.with_cs_name(|name| {
    name.strip_prefix('\\').is_some_and(|letters| {
      !letters.is_empty() && letters.chars().all(|c| c.is_ascii_alphabetic())
    })
  });
  if !public {
    return false;
  }
  let Some(Stored::Expandable(defn)) = lookup_meaning(token) else {
    return false;
  };
  if defn
    .paramlist
    .as_ref()
    .is_some_and(|params| !params.get_parameters().is_empty())
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = &defn.expansion else {
    return false;
  };
  body.unlist_ref().iter().all(|t| {
    matches!(
      t.get_catcode(),
      Catcode::LETTER | Catcode::OTHER | Catcode::SPACE
    )
  }) && body
    .unlist_ref()
    .iter()
    .any(|t| t.get_catcode() == Catcode::LETTER)
}

/// Is `token`'s meaning a text default ([`text_default`]) the stand-in binding does not own, for
/// [`note_copy_provides_assignment`]: assigned last by the base class the binding loaded
/// (`State::stand_in_base_defaults`: article, report or book, whose bindings assign their classes' names in their own
/// load, the OmniBus guess) or by LaTeXML's kernel before it? Then the copy's definition of it stands, as the real
/// class's does over article.cls's: MRM.cls:3191 `\def\refname{REFERENCES}` (2506.14995's PDF), ws-rv9x6.cls:91
/// `\newcommand\bibname{References}` (2301.03442). One the binding's own code assigned, or a binding it required
/// (natbib's), stays the binding's.
fn unowned_text_default(token: &Token) -> bool {
  use crate::definition::origin::DefinitionOrigin;
  if !text_default(token) {
    return false;
  }
  let key = meaning_key(token);
  if state!().stand_in_meanings.contains(&key) {
    return state!().stand_in_base_defaults.contains(&key);
  }
  let Some(Stored::Expandable(defn)) = lookup_meaning(token) else {
    return false;
  };
  matches!(
    defn.origin,
    DefinitionOrigin::Unknown | DefinitionOrigin::Pool
  ) || class_layer_definition(token)
}

/// Does a stand-in's copy, read now, `\newcommand` a text default its binding does not own (`unowned_text_default`),
/// in a real class that lacks the name? Then the declaration defines it, as it does in TeX: LaTeXML's kernel or the
/// binding's base class predefined it, and no base class the copy loads defines it — a standalone class (ws-rv9x6.cls:91
/// `\newcommand\bibname{References}`, 2301.03442), or a name its base class lacks (article.cls has no `\bibname`). A
/// name the base class the copy `\LoadClass`es defines is the real class's already, so its `\newcommand` is TeX's
/// "already defined" ([`copy_provides_text_default_clashes`]).
pub fn copy_provides_text_default_yields(token: &Token) -> bool {
  crate::definition::origin::copy_provides() != 0
    && !state_is_unlocked()
    && unowned_text_default(token)
    && !copy_base_class_defines(token)
}

/// Does a stand-in's copy, read now, `\newcommand` a text default its binding does not own that a base class it
/// `\LoadClass`ed defines (`copy_base_class_defines`: article.cls:616 `\newcommand\refname`, :618 `\figurename`, which
/// the article binding assigns in its own load; a raw base class's own definition)? The real class has the name, so
/// LaTeX stops with "Command … already defined" and keeps the first definition: an Error here as there (sect08.rs
/// `\newcommand`; the reviewer's probes newcmd_base and nc_chain, pdflatex 2 errors, "References"/"Figure 1";
/// nc_namedef 1). A name the base class made `\relax` is definable, as LaTeX's `\newcommand` finds it.
/// `\def`/`\renewcommand` redefine as before.
pub fn copy_provides_text_default_clashes(token: &Token) -> bool {
  crate::definition::origin::copy_provides() != 0
    && !state_is_unlocked()
    && text_default(token)
    && !binding_owns(token)
    && copy_base_class_defines(token)
}

/// The stand-in binding the copy read now (`copy_provides`) was read after, or the empty string when no copy is read.
pub fn copy_provides_binding() -> String {
  let id = crate::definition::origin::copy_provides();
  if id == 0 {
    return String::new();
  }
  state!()
    .copy_provides_copies
    .get(id as usize - 1)
    .map(|copy| copy.binding.clone())
    .unwrap_or_default()
}

/// Is `token` a text name the standard class's binding assigned in its own load, which a derivative of that class read
/// now defines itself (`binding::content::standard_class_binding_assigned`, OXIDIZED_DESIGN_DIVERGENCES #731 rule 13)?
/// In TeX the derivative is read in the class's place, so its `\newcommand` of the name is the first: aip-cp.cls:875
/// `\newcommand\refname{REFERENCES}` (1506.04050, 1604.08112). Only a text name yields (`binding_text_name`:
/// `\refname`, `\figurename`, `\@ptsize`, `\@pnumwidth`); a structural command stays the binding's, LaTeXML's markup of
/// the class — `\appendix` (`<ltx:appendix>`, article_cls.rs, book_cls.rs), book's `\frontmatter`/`\mainmatter`/
/// `\backmatter` (unnumbered chapters), whose `\newcommand`s in a full class copy (hep-ex0405054's book_modified.cls:531,
/// math0301023's khbook.cls:285; 0705.2210, hep-ph0008011, cond-mat0402505, quant-ph0409078) lost the appendices and
/// numbered the preface. Names a stand-in the derivative loads assigned are not the class's (the 77w r2 re-review's
/// finding 4).
pub fn standard_class_binding_name(token: &Token) -> bool {
  crate::binding::content::standard_class_binding_assigned(meaning_key(token))
    && lookup_meaning(token).is_some_and(|meaning| binding_text_name(&meaning))
}

/// Is `meaning` a binding's text name: a macro its code defined (origin `Binding`), without parameters, whose expansion
/// is text — tokens, none a control sequence or active character (`\refname`'s "References", `\@ptsize`'s "0",
/// `\@pnumwidth`'s "1.55em")? A primitive, a constructor, a register, an empty macro (a hook) or a macro that calls
/// anything (`\appendix`'s `\@appendix`, book's `\@chapapp` → `\chaptername`) is not.
fn binding_text_name(meaning: &Stored) -> bool {
  let Stored::Expandable(defn) = meaning else {
    return false;
  };
  if defn.origin != crate::definition::origin::DefinitionOrigin::Binding
    || defn
      .paramlist
      .as_ref()
      .is_some_and(|params| !params.get_parameters().is_empty())
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = &defn.expansion else {
    return false;
  };
  let body = body.unlist_ref();
  !body.is_empty()
    && body
      .iter()
      .all(|t| !matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE))
}

/// Did the stand-in binding's own load assign `token` last (`State::stand_in_meanings`, not its base class's
/// `State::stand_in_base_defaults`)? Then the copy's definition only provides it ([`note_copy_provides_assignment`]).
fn binding_owns(token: &Token) -> bool {
  let key = meaning_key(token);
  let state = state!();
  state.stand_in_meanings.contains(&key) && !state.stand_in_base_defaults.contains(&key)
}

/// The class file name a load request names (`article`, `dir/mybase.cls` → `article.cls`, `mybase.cls`).
pub fn class_file_name(file: &str) -> String {
  let name = file.trim();
  let name = name.rsplit(['/', '\\']).next().unwrap_or(name);
  if name.ends_with(".cls") {
    name.to_string()
  } else {
    s!("{name}.cls")
  }
}

/// RAII frame of one class load (`binding::content::input_definitions`, a `.cls` with its options handled: the
/// `\documentclass`, a `\LoadClass`, a binding's `LoadClass!`). While it is open, the macros assigned outside LaTeXML's
/// kernel layers — the class binding's own (article.cls's names, article_cls.rs), the raw file's, a package's it
/// loads — and the names a lock kept from a definition it made are recorded as the class's (`State::class_loads`),
/// as its real file defines them; a repeat load records nothing new. A load a stand-in's copy requests (`copy`: its own
/// `\LoadClass`, or a class loaded while that one is open) also names the class among the copy's base classes
/// (`CopyProvidesRecord::loads_classes`), whose names its `\newcommand` finds defined
/// ([`copy_provides_text_default_clashes`]): chained `\LoadClass{mybase}` over article (the reviewer's probe nc_chain).
pub struct ClassLoadFrame {
  copy: bool,
}

impl ClassLoadFrame {
  /// Open the frame of loading the class `file`, requested by the stand-in copy `copy` if any.
  pub fn enter(file: &str, copy: Option<u32>) -> Self {
    let name = class_file_name(file);
    let mut state = state_mut!();
    if let Some(id) = copy
      && let Some(record) = state.copy_provides_copies.get_mut(id as usize - 1)
      && !record.loads_classes.contains(&name)
    {
      record.loads_classes.push(name.clone());
    }
    // (a class loaded inside the frames open now is loaded inside each of them: the binding's `LoadClass!` of its
    // base class, ceurart_cls.rs → OmniBus → article, is a class the binding's load loaded)
    let state = &mut *state;
    // (the classes the load open now had loaded before this one: for a copy's own load, its binding's base classes)
    let outer_classes = copy
      .and_then(|_| state.class_load_frames.last())
      .and_then(|&open| state.class_loads.get(open))
      .map(|record| record.classes.clone())
      .unwrap_or_default();
    for &open in &state.class_load_frames {
      if let Some(outer) = state.class_loads.get_mut(open)
        && outer.file != name
        && !outer.classes.contains(&name)
      {
        outer.classes.push(name.clone());
      }
    }
    let index = match state
      .class_loads
      .iter()
      .position(|record| record.file == name)
    {
      Some(index) => index,
      None => {
        state.class_loads.push(ClassLoadRecord {
          file: name,
          ..ClassLoadRecord::default()
        });
        state.class_loads.len() - 1
      },
    };
    state.class_load_frames.push(index);
    if let Some(id) = copy {
      state.copy_class_loads.push((id, outer_classes));
    }
    ClassLoadFrame { copy: copy.is_some() }
  }
}

impl Drop for ClassLoadFrame {
  fn drop(&mut self) {
    let mut state = state_mut!();
    state.class_load_frames.pop();
    if self.copy {
      state.copy_class_loads.pop();
    }
  }
}

/// The stand-in copy whose own `\LoadClass` is open now, if any ([`ClassLoadFrame`]).
pub fn open_copy_class_load() -> Option<u32> { state!().copy_class_loads.last().map(|(id, _)| *id) }

/// The classes the stand-in binding's load loaded — its base class — for the copy read now
/// ([`ClassLoadRecord::classes`]). While the copy's own code runs, the classes of the class load open now; inside a
/// class the copy's own `\LoadClass` loaded (a shipped mybase.cls between the copy and a TeX Live class), the classes
/// loaded when the copy's first class load began ([`State::copy_class_loads`]), so a class mybase.cls `\LoadClass`es
/// meets the binding's base class too, not the empty class load of mybase.cls (72a review r7 NIT 8; guard
/// `perfect_kernel_batchint23::a_shipped_intermediate_class_meets_the_class_slot`).
pub fn open_class_load_classes() -> Vec<String> {
  let state = state!();
  if crate::definition::origin::copy_provides() != 0 {
    return state
      .class_load_frames
      .last()
      .and_then(|&class| state.class_loads.get(class))
      .map(|record| record.classes.clone())
      .unwrap_or_default();
  }
  let Some((id, _)) = state.copy_class_loads.last() else {
    return Vec::new();
  };
  state
    .copy_class_loads
    .iter()
    .find(|(copy, _)| copy == id)
    .map(|(_, classes)| classes.clone())
    .unwrap_or_default()
}

/// Name `classes` among the base classes of the stand-in copy `id` (`CopyProvidesRecord::loads_classes`): the classes
/// its binding loaded, which stand for the class the copy's own `\LoadClass` named (`binding::content`
/// `binding_base_class_stands_for`).
pub fn note_copy_base_classes(id: u32, classes: &[String]) {
  let mut state = state_mut!();
  if let Some(record) = state.copy_provides_copies.get_mut(id as usize - 1) {
    for class in classes {
      if !record.loads_classes.contains(class) {
        record.loads_classes.push(class.clone());
      }
    }
  }
}

/// Record `key` as a name the class load open now defines ([`ClassLoadFrame`]).
fn note_class_load_name(key: SymStr) {
  let mut state = state_mut!();
  if let Some(&class) = state.class_load_frames.last()
    && let Some(record) = state.class_loads.get_mut(class)
  {
    record.names.entry(key).or_insert(None);
  }
}

/// Does a base class of the stand-in copy read now — a class its own `\LoadClass` requested, or one loaded while that
/// was open ([`ClassLoadFrame`]) — define `token`: did that class's load frame assign it (article.cls:613-622's names,
/// which article_cls.rs assigns in its own load; a raw base class's `\expandafter\def\csname bibname\endcsname`, the
/// reviewer's probe nc_namedef) or a lock keep its definition? Its document-time code
/// (`\AtBeginDocument{\def\bibname…}`, nc_cond) has not run, so defines nothing yet, as in TeX.
fn copy_base_class_defines(token: &Token) -> bool {
  let id = crate::definition::origin::copy_provides();
  if id == 0 {
    return false;
  }
  let key = meaning_key(token);
  let state = state!();
  let Some(copy) = state.copy_provides_copies.get(id as usize - 1) else {
    return false;
  };
  copy.loads_classes.iter().any(|class| {
    state
      .class_loads
      .iter()
      .any(|record| record.file == *class && record.names.contains_key(&key))
  })
}

/// Is `token`'s meaning the macro a class's load frame assigned it ([`ClassLoadFrame`]): the class layer's —
/// article.cls's names, which its binding assigns, a raw class's own —, not LaTeXML's semantic layer? A text default of
/// the class layer is the class's, as LaTeXML's kernel predefinitions of the same names are ([`unowned_text_default`]).
fn class_layer_definition(token: &Token) -> bool {
  let Some(Stored::Expandable(defn)) = lookup_meaning(token) else {
    return false;
  };
  let key = meaning_key(token);
  state!().class_loads.iter().any(
    |record| matches!(record.names.get(&key), Some(Some(assigned)) if Rc::ptr_eq(assigned, &defn)),
  )
}

/// The one "has chapters" test (OXIDIZED_DESIGN #179): does the class have chapters, as the real class's
/// `\@ifundefined{chapter}` sees it? natbib's forks head the bibliography by it (NJDnatbib.sty:1050, ws-rv-har.sty:676),
/// appendix.sty:38 and titlesec.sty:1526 number by it, and the kernel's `\appendix` numbers within chapters only then
/// (`begin_appendices`). The answer the "has chapters" step stored after the document's class loaded
/// (`settle_chapter_after_class_load`, latexml_engine sect01.rs, [`CLASS_HAS_CHAPTERS`]); while a class loads, the
/// test made now ([`class_has_chapters_now`]), as a class's own `\@ifundefined{chapter}` is answered by what it has
/// defined so far. Not a `chapter` counter alone: WileyNJDv5.cls:5728 and MRM.cls:2160 `\newcounter{chapter}[part]` with
/// no `\chapter` (2609.08425, 2506.14995: "REFERENCES"; 2211.16474: "Appendix A"). A LaTeX 2.09 `\documentstyle`
/// document never settles (only a `\documentclass` retracts and settles, `\chapter:settles`); its class's end stores
/// the answer (`\chapter:stored`, `store_chapter_after_latex_ix_class_load`, latexml_engine sect01.rs), so
/// `\documentstyle[appendix]{report}` numbers its appendix chapters "Appendix A" and `\documentstyle{article}` with the
/// document's own `\newcounter{chapter}` + `\def\chapter` letters its appendix "A", as pdflatex. Before any class load
/// (plain TeX), the test made now.
pub fn class_defines_chapter() -> bool {
  if crate::binding::content::class_loading()
    || !(lookup_bool("\\chapter:settles") || lookup_bool("\\chapter:stored"))
  {
    class_has_chapters_now()
  } else {
    lookup_bool(CLASS_HAS_CHAPTERS)
  }
}

/// The value [`class_defines_chapter`] reads once the document's class has loaded.
pub const CLASS_HAS_CHAPTERS: &str = "\\chapter:class@has@chapters";

/// The "has chapters" test made now (OXIDIZED_DESIGN #179), which `settle_chapter_after_class_load` stores after each
/// class load: the class has chapters when
/// - (a) its own code assigned `\chapter`: a meaning other than the kernel's (a raw class's `\providecommand\chapter`, a
///   binding's own `\chapter`: ws-rv9x6, nic-series, jmlr, kluwer's `chapter` option; a `\let\chapter\artchapter`), a
///   redefinition the kernel's lock refused (`\chapter:redefined`, state.rs `install_definition`, sect08.rs), or a
///   stand-in copy's definition its binding's owned name kept aside or declined (pasj00.cls's `\def\chapter`;
///   ws-rv9x6.cls:499 `\newcommand\chapter`, 2301.03442). A meaning that is undefined, `\relax`
///   (`\let\chapter\@undefined`; `\csname chapter\endcsname`) or the undefined-error stub of a call is no definition;
/// - (b) a compiled binding made the chapter counter (`\c@chapter` of LaTeXML's own origin: report, book, memoir…;
///   Perl's own probe, latex_constructs.pool.ltxml:690). A raw or copy `\newcounter{chapter}` alone is not;
/// - (c) the class is unknown and OmniBus guesses it (`OmniBus:fallback`: the class loader's fallback, and subfiles,
///   whose subfiles.cls:38-40 loads the main file's class), as an unknown class may well have chapters: OmniBus
///   autoloads book.cls on `\thechapter` (arXiv:2602.10407).
pub fn class_has_chapters_now() -> bool {
  let (assigned, kernel_kept) = class_chapter_sources();
  assigned || kernel_kept
}

/// The two halves of [`class_has_chapters_now`]: whether the class's own code assigned `\chapter` (a), and whether the
/// class keeps LaTeXML's kernel `\chapter` as a binding's chapters (b) or OmniBus's guess (c).
pub fn class_chapter_sources() -> (bool, bool) {
  use crate::definition::origin::DefinitionOrigin;
  let chapter = T_CS!("\\chapter");
  let relax = lookup_meaning(&T_CS!("\\relax"));
  let meaningful =
    |meaning: &Stored| !matches!(meaning, Stored::None) && relax.as_ref() != Some(meaning);
  let key = meaning_key(&chapter);
  let by_copy = state!()
    .copy_provides_copies
    .iter()
    .any(|copy| copy.declined.contains(&key) || copy.overlay.get(&key).is_some_and(meaningful));
  let meaning = lookup_meaning(&chapter);
  let defined = meaning.as_ref().is_some_and(meaningful) && !lookup_bool("\\chapter:error@stub");
  // the kernel's own meaning (the pool's, or `\lx@kernel@chapter` / `\lx@kernel@chapter@counted` put back): a class's
  // same-bodied `\newcommand\chapter{\@startsection{chapter}…}` is the class's
  let kernels = defined
    && (x_equals(&chapter, &T_CS!("\\lx@kernel@chapter"))
      || x_equals(&chapter, &T_CS!("\\lx@kernel@chapter@counted")))
    && meaning
      .as_ref()
      .and_then(definition_origin)
      .is_some_and(|origin| matches!(origin, DefinitionOrigin::Pool | DefinitionOrigin::Unknown));
  let assigned = by_copy || lookup_bool("\\chapter:redefined") || (defined && !kernels);
  let binding_counter = matches!(
    lookup_meaning(&T_CS!("\\c@chapter")),
    Some(Stored::Register(ref register)) if register.origin.is_latexml_owned()
  );
  (assigned, binding_counter || lookup_bool("OmniBus:fallback"))
}

/// The meaning the latest stand-in copy of `file` (a file name, `NJDnatbib.sty`) gave `token` in its own code: its
/// overlaid version of a name its binding owns, if the copy redefined it ([`note_copy_provides_assignment`]). For a
/// binding's code that runs after its copy (`\AtEndOfPackage`), to tell the copy's version by what it defined.
pub fn stand_in_copy_meaning(file: &str, token: &Token) -> Option<Stored> {
  let key = meaning_key(token);
  state!()
    .copy_provides_copies
    .iter()
    .rev()
    .find(|copy| copy.file.rsplit(['/', '\\']).next() == Some(file))
    .and_then(|copy| copy.overlay.get(&key).cloned())
}

/// Is `token`'s current meaning owned by LaTeXML's semantic layer, for [`note_copy_provides_assignment`]? Either the
/// name was assigned while a stand-in binding loaded ([`crate::definition::origin::StandInLoadGuard`]: its own definitions,
/// `\let`s and `RawTeX`, and the bindings it required), at an origin of LaTeXML's own, or its meaning is a definition
/// made under this very name (not a `\let` copy of another's, whose object carries the other name) at such an origin —
/// `Unknown`, `Pool` or `Binding` (definition/origin.rs): a pool's (`\section`, `\maketitle`) or another binding's
/// (natbib's `\citep`, xkeyval's `\usevalue`); a name the stand-in binding's load assigned is owned at `Fallback` too,
/// the OmniBus base a class binding loads (`\received`, `\corres`). The format's own definitions (`Plain`,
/// `LatexDump`, `Format`), the document's world (`File`, `Document`) and the other guesses that yield (`Fallback`,
/// `Stub`, `Borrowed`) are not owned.
/// A name a package the binding loaded defined stays owned: the narrower rule (only the binding's own code, so that a
/// copy's redefinition of a package's macro stands as in TeX) gave the copy the class bindings' frontmatter and layout
/// macros (`\title`, `\author`, `\thanks`, `\@makecaption`, `\the<counter>`, the NFSS size commands; aastex7's three
/// Fatals again, 2504.00922, 2605.15371, 2607.03848); the case it was for, 2411.02780's equations.sty, is another file
/// of that name (`binding::content` `copy_is_the_stood_in_file`).
fn owned_by_the_semantic_layer(token: &Token) -> bool {
  use crate::definition::origin::DefinitionOrigin;
  let ours = |origin: DefinitionOrigin| {
    matches!(
      origin,
      DefinitionOrigin::Unknown | DefinitionOrigin::Pool | DefinitionOrigin::Binding
    )
  };
  lookup_meaning(token).is_some_and(|meaning| {
    definition_origin(&meaning).is_some_and(|origin| {
      let key = meaning_key(token);
      if state!().binding_lets.contains(&key) && ours(origin) {
        // (a `\let` LaTeXML's own code made: the object carries the source's name, `State::binding_lets`)
        true
      } else if state!().stand_in_meanings.contains(&key) {
        // (the stand-in binding's guesses too: the OmniBus base a class binding loads, `Fallback` so that the
        // document's own definition wins, OXIDIZED_DESIGN_DIVERGENCES #408 — the copy is not the document: WileyNJDv5's
        // `\received` keeps OmniBus's dated frontmatter, 2609.19994)
        ours(origin) || origin == DefinitionOrigin::Fallback
      } else {
        ours(origin) && definition_cs(&meaning) == Some(*token)
      }
    })
  })
}

/// The name a definition was made under (a `\let` copy keeps its source's).
fn definition_cs(meaning: &Stored) -> Option<Token> {
  match meaning {
    Stored::Expandable(defn) => Some(defn.cs),
    Stored::Constructor(defn) => Some(defn.cs),
    Stored::Primitive(defn) => Some(defn.cs),
    Stored::MathPrimitive(defn) => Some(defn.cs),
    Stored::Conditional(defn) => Some(defn.cs),
    Stored::Register(defn) => Some(defn.cs),
    _ => None,
  }
}

/// Who made the definition `meaning` (`None`: not a definition — a `\let` to a character, an undefined name).
fn definition_origin(meaning: &Stored) -> Option<crate::definition::origin::DefinitionOrigin> {
  match meaning {
    Stored::Expandable(defn) => Some(defn.origin),
    Stored::Constructor(defn) => Some(defn.origin),
    Stored::Primitive(defn) => Some(defn.origin),
    Stored::MathPrimitive(defn) => Some(defn.origin),
    Stored::Conditional(defn) => Some(defn.origin),
    Stored::Register(defn) => Some(defn.origin),
    _ => None,
  }
}

/// The code a stand-in's copy registers with `\AtEndOfClass`/`\AtEndOfPackage`, which runs at the end of its own
/// load: wrapped in the copy's scope markers `\lx@copy@provides@begin{N}` … `\lx@copy@provides@end`, so it runs under
/// the copy's provides-mode, with the copy's own definitions in force ([`enter_copy_provides_frame`]; aa.cls:71's
/// `\AtEndOfClass` redefinition of `{abstract}` keeps the binding's). `tokens` unchanged when no copy is read or it is
/// wrapped already.
pub fn copy_provides_wrap(tokens: Tokens) -> Tokens {
  let id = crate::definition::origin::copy_provides();
  let begin = T_CS!("\\lx@copy@provides@begin");
  if id == 0 || tokens.unlist_ref().first() == Some(&begin) {
    return tokens;
  }
  let mut wrapped = vec![begin, T_BEGIN!()];
  wrapped.extend(mouth::tokenize_internal(TeXString::assembled(id.to_string())).unlist());
  wrapped.push(T_END!());
  wrapped.extend(tokens.unlist());
  wrapped.push(T_CS!("\\lx@copy@provides@end"));
  Tokens::new(wrapped)
}

/// Does a stand-in's copy, read now, defer code to the document through `seam` (`\AtBeginDocument`,
/// `\AtEndDocument`, etoolbox's `\AtEndPreamble`/`\AfterEndPreamble`/`\AfterEndDocument`, lthooks' `\AddToHook`)? Then
/// the code is not registered: it is the class's or style's document-time machinery, which the binding has taken over
/// — layout checks against the class's page dimensions (icml2024.sty's "The page layout violates the ICML style",
/// 2407.02750), validations of the class's own setters the document called as the binding's (PoS.cls "\abstract{...}
/// should appear before \begin{document}", 1001.3214; ieeeaccess.cls's `\EOD`, 2007.03025; aipproc.cls's
/// `\layoutstyle`, astro-ph0105128), patches of the kernel's `\enddocument` (aastex7.cls:459, whose
/// `\rvtx@enddocument@patch` reads up to `\@checkend` and ran off the file: 2504.00922, 2605.15371, 2607.03848). On the
/// 2026-10-17 census sample no Error was fixed by such code. Logged once per copy and seam, at Info.
pub fn copy_provides_defers(seam: &str) -> bool {
  let id = crate::definition::origin::copy_provides();
  if id == 0 {
    return false;
  }
  let key = arena::pin(seam);
  let mut state = state_mut!();
  if let Some(copy) = state.copy_provides_copies.get_mut(id as usize - 1)
    && copy.deferred.insert(key)
  {
    let file = copy.file.clone();
    drop(state);
    Info!(
      "provide",
      seam,
      s!(
        "The paper's {file} defers code to the document ({seam}); not registered: its binding stands in for that code"
      )
    );
  }
  true
}

/// Is `token` one of LaTeX's document hooks (`\@begindocumenthook`, `\@enddocumenthook`), which a stand-in's copy read
/// now writes — `\g@addto@macro`, ltxgrid's `\prepdef`, a `\def` —, so that the write is document-time code, which the
/// copy does not register, as it does not its `\AtBeginDocument` code ([`copy_provides_defers`], Info)? ltxgrid's
/// `\prepdef\@begindocumenthook{\open@column@one\@ne\set@colht…}` (qm2008_abs.cls:3233, snp.cls; REVTeX 4 copies
/// routed to the revtex4 binding) ran its page grid over the binding's names at `\begin{document}`: 7 "Missing number"
/// Warnings in 0805.2253, 0806.0481, 0805.4759.
pub fn copy_provides_defers_hook_write(token: &Token) -> bool {
  crate::definition::origin::copy_provides() != 0
    && token.with_cs_name(|cs| matches!(cs, "\\@begindocumenthook" | "\\@enddocumenthook"))
    && token.with_cs_name(copy_provides_defers)
}

/// Put lthooks' code registration (`\AddToHook code`, the inner macro latex.ltx's `\AddToHook` hands the hook name,
/// label and code: `\hook_gput_code:nnn{#1}{#2}{#3}`) under `\lx@copy@provides@AddToHook`, which registers nothing
/// ([`copy_provides_defers`]), while `on` — a stand-in's copy is read — and back after. The kernel's
/// own meaning is kept as `\lx@copy@provides@saved@AddToHook`. No-op where the format has no hooks or no seam is
/// defined.
pub fn swap_copy_provides_hook_seam(on: bool) {
  let seam = T_CS!("\\AddToHook code");
  let saved = T_CS!("\\lx@copy@provides@saved@AddToHook");
  let wrapper = T_CS!("\\lx@copy@provides@AddToHook");
  let set = |token: &Token, meaning: Stored| {
    let key = token.pin_cs_name();
    state_mut!().assign_internal(TableName::Meaning, key, meaning, Some(Scope::Global));
  };
  if on {
    if let (Some(kernel), Some(wrap)) = (lookup_meaning(&seam), lookup_meaning(&wrapper)) {
      set(&saved, kernel);
      set(&seam, wrap);
    }
  } else if let Some(kernel) = lookup_meaning(&saved) {
    set(&seam, kernel);
    set(&saved, Stored::None);
  }
}

/// Begin a stand-in binding's load ([`crate::definition::origin::StandInLoadGuard`]): no name of its own assigned yet
/// ([`State::stand_in_loads`]); a stand-in it loads records its own apart.
pub fn begin_stand_in_load() {
  let mut state = state_mut!();
  state.stand_in_loads.push(Vec::new());
  state.stand_in_load_files.push(Vec::new());
}

/// A stand-in binding's own code loads `file` ([`State::stand_in_load_files`]).
pub fn note_stand_in_file(file: &str) {
  if let Some(files) = state_mut!().stand_in_load_files.last_mut() {
    files.push(file.to_string());
  }
}

/// The files the stand-in binding loaded last loaded from its own code ([`State::stand_in_last_files`]).
pub fn stand_in_built_on() -> Vec<String> { state!().stand_in_last_files.clone() }

/// End the stand-in binding's load [`begin_stand_in_load`] began: its names are the last load's
/// ([`State::stand_in_last_load`]), and, when the stand-in binding loading it requested it in its own code
/// (`into_outer`, `StandInLoadGuard::loaded_by`), that binding's too.
pub fn end_stand_in_load(into_outer: bool) {
  let mut state = state_mut!();
  let files = state.stand_in_load_files.pop().unwrap_or_default();
  if let Some(load) = state.stand_in_loads.pop() {
    // (a stand-in its own code loads is part of its command set: the diagrams binding's `diagrams.tex`, whose names
    // are Paul Taylor's package's, hep-ph9609379; not one a file it loads requests — OmniBus's aas_macros under
    // asme2ej, 2309.12017, or nature-pre, 2102.01782, 2111.09589, swelled their sets from 22 and 3 names to 87 and 68)
    if into_outer && let Some(outer) = state.stand_in_loads.last_mut() {
      outer.extend(load.iter().copied());
    }
    state.stand_in_last_load = load;
    state.stand_in_last_files = files;
  }
}

/// Every name the latest stand-in binding's own code assigned ([`State::stand_in_last_load`]).
pub fn stand_in_last_load_names() -> rustc_hash::FxHashSet<SymStr> {
  state!().stand_in_last_load.iter().copied().collect()
}

/// The public names (letters only: no `@`) the latest stand-in binding's own code assigned ([`State::stand_in_last_load`]).
pub fn stand_in_public_names() -> rustc_hash::FxHashSet<String> {
  state!()
    .stand_in_last_load
    .iter()
    .map(|key| arena::to_string(*key))
    .filter(|name| {
      name.strip_prefix('\\').is_some_and(|letters| {
        !letters.is_empty() && letters.chars().all(|c| c.is_ascii_alphabetic())
      })
    })
    .collect()
}

/// Begin a provide-only load: no name assigned in it yet.
pub fn reset_provide_only_assigned() {
  let mut state = state_mut!();
  state.provide_only_assigned.clear();
  state.provide_only_loaded.clear();
}

/// End a provide-only load: the files first loaded during it (but the fallback class itself) loaded only what they
/// could provide, so they are marked unloaded again — `<file>_loaded`, its `.ltxml_loaded` twin, `_load_attempted` and
/// the kernel's `\ver@<file>` — and a later `\usepackage`/`\RequirePackage` of one loads it whole
/// (`\documentstyle[graphics,myjunkx]{article}\usepackage{graphicx}`: graphicx's keyval `\includegraphics`).
pub fn end_provide_only() {
  let files = std::mem::take(&mut state_mut!().provide_only_loaded);
  for file in files {
    if file == "OmniBus.cls" {
      continue;
    }
    mark_unloaded(&file);
  }
}

/// Mark `file` (`amsthm.sty`) as not loaded, globally: its `<file>_loaded`, the `.ltxml_loaded` twin, the raw load's
/// `_raw_loaded` and `_load_attempted` are cleared and the kernel's `\ver@<file>` and `\opt@<file>` are undefined, so
/// `\@ifpackageloaded` is false, no option clash is checked, and a later `\usepackage`/`\RequirePackage` loads it
/// again — the binding and the raw file it reads. The definitions it made stay in force. Used when a load provided
/// only part of a file ([`end_provide_only`]) and for a binding's borrowed package and the files it loaded
/// (`binding::content::require_package_borrowed`).
pub fn mark_unloaded(file: &str) {
  for flag in ["_loaded", ".ltxml_loaded", "_raw_loaded", "_load_attempted"] {
    if lookup_bool(&s!("{file}{flag}")) {
      assign_value(&s!("{file}{flag}"), false, Some(Scope::Global));
    }
  }
  for kernel in ["ver", "opt"] {
    assign_meaning(
      &T_CS!(s!("\\{kernel}@{file}")),
      Stored::None,
      Some(Scope::Global),
    );
  }
}

/// Begin a borrow (`binding::content::require_package_borrowed`): no file loaded by it yet. Returns the list of an
/// enclosing borrow, which [`end_borrow`] puts back.
pub fn begin_borrow() -> Vec<String> { std::mem::take(&mut state_mut!().borrowed_loaded) }

/// End a borrow: the files first marked loaded during it (`borrowed_loaded`) are marked unloaded ([`mark_unloaded`]),
/// and the enclosing borrow's list is put back.
pub fn end_borrow(outer: Vec<String>) {
  let files = std::mem::replace(&mut state_mut!().borrowed_loaded, outer);
  for file in files {
    mark_unloaded(&file);
  }
}

/// Has the document unloaded `file` (`amsthm.sty`), which a class binding marked loaded on its class's behalf, by
/// letting its `\ver@<file>` to `\relax` or `\undefined`? LaTeX's loaded test is that macro (latex.ltx `\@ifl@aded`:
/// `\expandafter\ifx\csname ver@#2.#1\endcsname\relax`, true for both), which a binding load defines (content.rs, Perl
/// Package.pm:2326). The rule is narrow, as floatrow's marks are (user ruling 2026-10-14: no general loader `\ver@`
/// rule; floatrow_sty.rs): only a file a binding recorded as `<file>:class_marked` — ams_support_sty.rs for amsart's
/// `\@namedef{ver@amsthm.sty}` (amsart.cls:41-42; amsproc, amsbook alike) — is so taken back; every other file keeps
/// the loader's own record. The amsart idiom is `\expandafter\let\csname ver@amsthm.sty\endcsname\relax` before
/// `\usepackage{amsthm}` (1510.08833, 1704.05671, 1707.00258, 1611.00843, 2106.07418, 2304.10644). Perl's loader reads
/// only `<file>_loaded` (Package.pm:2327-2329), KNOWN_PERL_ERRORS #720.
pub fn unmarked_by_document(file: &str) -> bool {
  if !lookup_bool(&s!("{file}:class_marked")) || !lookup_bool(&s!("{file}_loaded")) {
    return false;
  }
  match lookup_meaning(&T_CS!(s!("\\ver@{file}"))) {
    None => true,
    Some(ver) => lookup_meaning(&T_CS!("\\relax")).is_some_and(|relax| ver == relax),
  }
}

/// Is `token`'s current meaning a fallback class's guess (`DefinitionOrigin::Fallback`), a kernel stub
/// (`DefinitionOrigin::Stub`) or a borrowed package's (`DefinitionOrigin::Borrowed`), which a document's own
/// `\newcommand` may replace?
pub fn is_fallback_meaning(token: &Token) -> bool {
  let state = state!();
  !state.fallback_meanings.is_empty() && state.fallback_meanings.contains_key(&meaning_key(token))
}

/// The origin of `token`'s current meaning when it is a fallback guess, a kernel stub or a borrowed package's
/// ([`is_fallback_meaning`]): `Fallback`, `Stub` or `Borrowed`, whatever kind of definition it is (only macros record
/// their own origin, `Definition::get_origin`; amsthm's proof is a constructor).
pub fn fallback_meaning_origin(
  token: &Token,
) -> Option<crate::definition::origin::DefinitionOrigin> {
  let state = state!();
  if state.fallback_meanings.is_empty() {
    return None;
  }
  state.fallback_meanings.get(&meaning_key(token)).copied()
}

/// A borrowing binding's claim on the borrowed names its real package defines too: their meanings no longer carry the
/// borrowed mark (`DefinitionOrigin::Borrowed`, [`fallback_meaning_origin`]), so they are the binding's own, as if it
/// had defined them again at its own origin, and a later file's `\newcommand`/`\newenvironment` of them is refused
/// as LaTeX refuses it ("Command … already defined"), not honoured. ntheorem.sty's `[amsthm]` option defines
/// `\proofname` and `{proof}` (ntheorem.sty:668-711), which its binding takes from the borrowed amsthm; theorem.sty's
/// `\newskip\theorempreskipamount` (theorem.sty:122-123) likewise. A name whose meaning is not borrowed is left
/// alone.
pub fn claim_borrowed_meanings(names: &[&str]) {
  use crate::definition::origin::DefinitionOrigin;
  if state!().fallback_meanings.is_empty() {
    return;
  }
  let keys: Vec<SymStr> = names
    .iter()
    .map(|name| meaning_key(&crate::T_CS!(*name)))
    .collect();
  let mut state = state_mut!();
  for key in keys {
    if state.fallback_meanings.get(&key) == Some(&DefinitionOrigin::Borrowed) {
      state.fallback_meanings.remove(&key);
    }
  }
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

/// Bind `key` to `value` in the value table — Perl's `AssignValue`.
///
/// [`Scope`] decides how long the binding lasts: [`Scope::Local`] expires with
/// the enclosing TeX group, [`Scope::Global`] does not, [`Scope::Named`] applies
/// only while that scope is activated, and [`Scope::InPlace`] rebinds at the
/// frame the value was last bound in. Passing `None` takes the state's current
/// default. Read back with [`lookup_value`], or one of the typed
/// [`lookup_string`] / [`lookup_number`] / [`lookup_bool`] accessors.
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
  state.journal_touch(TableName::Value, key_sym);
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

/// A save-frame bookkeeping binding (`groupInitiator`, `BOUND_MODE`, the
/// frame id, an environment's group code…): always local to the current
/// frame, exempt from `\globaldefs` and the `\global` prefix — tex.web §1214
/// vs §274. See `State::assign_local_unconditional`.
pub fn assign_frame_value_sym<T: Into<Stored>>(key: SymStr, value: T) {
  state_mut!().assign_local_unconditional(TableName::Value, key, value.into());
}

/// [`assign_frame_value_sym`] with a string key.
pub fn assign_frame_value<T: Into<Stored>>(key: &str, value: T) {
  assign_frame_value_sym(arena::pin(key), value);
}

/// assigns a `Stored` value at the given (arena ticket!) key and scope
pub fn assign_value_sym<T: Into<Stored>, S: Into<Option<Scope>>>(key: SymStr, value: T, scope: S) {
  let value = value.into();
  let scope = scope.into();
  state_mut!().assign_internal(TableName::Value, key, value, scope);
}

/// inline lookup_value after which globally assign an empty Tokens() to undo
pub fn remove_value(key: &str) -> Option<Stored> { remove_value_sym(arena::pin(key)) }

/// `remove_value` variant for hot call sites with a pre-pinned SymStr (see
/// `crate::pin!`) — added for `after_assignment`, which fires on every
/// `\def`/`\let`/register assignment.
pub fn remove_value_sym(key_sym: SymStr) -> Option<Stored> {
  let mut state = state_mut!();
  state.journal_touch(TableName::Value, key_sym);
  match state.value.get_mut(&key_sym) {
    None => None,
    Some(vvec) => match vvec.front_mut() {
      None | Some(&mut Stored::None) => None,
      Some(found) => Some(std::mem::take(found)),
    },
  }
}
/// Replaces the value in question with `Stored::None` (see `checkin_value` for returning it)
pub fn checkout_value(key: &str) -> Option<Stored> {
  let key = arena::pin(key);
  let mut state = state_mut!();
  state.journal_touch(TableName::Value, key);
  match state.value.get_mut(&key) {
    None => None,
    Some(vvec) => vvec.front_mut().map(std::mem::take),
  }
}
/// Returns a value into its `Stored::None` placeholder (see `checkout_value` for taking it)
pub fn checkin_value(key: &str, value: Stored) {
  let key_sym = arena::pin(key);
  let mut state = state_mut!();
  state.journal_touch(TableName::Value, key_sym);
  match state.value.get_mut(&key_sym) {
    None => {
      // Key was never assigned — silently ignore the checkin
      emit_warn(
        "internal",
        "state",
        &format!("checkin_value called for unknown key '{key}'"),
      );
    },
    Some(vvec) => match vvec.front_mut() {
      None => {
        emit_warn(
          "internal",
          "state",
          &format!("checkin_value called with empty value stack for key '{key}'"),
        );
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
    state.journal_touch(TableName::Value, key_sym);
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
    state.journal_touch(TableName::Value, key_sym);
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
pub fn has_value(key: &str) -> bool { has_value_sym(arena::pin(key)) }
/// Sym-keyed variant of `has_value` — avoids the per-call `arena::pin(key)`.
pub fn has_value_sym(key_sym: SymStr) -> bool {
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
    // Perl `pushValue` (State.pm:218-223) creates a missing list globally: a `\vadjust` inside a group
    // (`Alpha {\vadjust{Bravo}} Charlie.`) kept its material past the group's end, where a local list was lost
    // (guard `box_primitives::vadjust`).
    None | Some(Stored::None) => {
      state.assign_value(key, Stored::Tokens(value), Some(Scope::Global))
    },
    Some(other) => panic!("Can only push_tokens into a Stored::Tokens, but got {other:?}"),
  }
}

/// Appends a digested box to the `Stored::VecDigested` list under `key`, creating the list globally when missing (as
/// [`push_tokens`], Perl `pushValue`, State.pm:218-223).
pub fn push_digested(key: &str, value: Digested) {
  let mut state = state_mut!();
  match state.lookup_value_mut(key) {
    Some(Stored::VecDigested(list)) => list.push(value),
    None | Some(Stored::None) => {
      state.assign_value(key, Stored::VecDigested(vec![value]), Some(Scope::Global))
    },
    Some(other) => panic!("Can only push_digested into a Stored::VecDigested, but got {other:?}"),
  }
}

/// Take the list [`push_tokens`] or [`push_digested`] has built under `key`, leaving it empty in place; `None` when
/// there is none.
pub fn take_pushed(key: &str) -> Option<Stored> {
  match state_mut!().lookup_value_mut(key) {
    Some(Stored::Tokens(tks)) => Some(Stored::Tokens(std::mem::take(tks))),
    Some(Stored::VecDigested(list)) => Some(Stored::VecDigested(std::mem::take(list))),
    _ => None,
  }
}

/// Make `value` the list under `key` again, in place: what [`take_pushed`] took, the list built since dropped.
pub fn restore_pushed(key: &str, value: Option<Stored>) {
  let mut state = state_mut!();
  match (state.lookup_value_mut(key), value) {
    (Some(slot @ (Stored::Tokens(_) | Stored::VecDigested(_))), Some(value)) => *slot = value,
    (Some(slot @ (Stored::Tokens(_) | Stored::VecDigested(_))), None) => *slot = Stored::None,
    (None | Some(Stored::None), Some(value)) => state.assign_value(key, value, Some(Scope::Global)),
    _ => {},
  }
}

/// The value bound to `key`, or `None` when nothing is bound — Perl's
/// `LookupValue`.
///
/// Returns whatever [`Stored`] variant was assigned, so a caller that knows the
/// type usually wants [`lookup_string`] / [`lookup_number`] / [`lookup_bool`]
/// instead. Clones the value; [`with_value`] lends it when inspecting is enough.
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
/// `\fontdimen<p>` of the font whose parameter array is stored under `key`,
/// in sp (Perl's `$$fontinfo{data}`, TeX_Fonts.pool.ltxml:131-137); `None`
/// when the font has no array yet or `p` lies past its end.
pub fn font_parameter(key: &str, p: usize) -> Option<i64> {
  with_value(key, |value| match value {
    Some(Stored::FontDimens(params)) => p
      .checked_sub(1)
      .and_then(|index| params.borrow().get(index).copied()),
    _ => None,
  })
}

/// Give the font whose array is stored under `key` its parameters, in sp:
/// Perl's `\font` filling a new font's `$$fontinfo{data}` from its metric
/// (TeX_Fonts.pool.ltxml:108-117). Unbound by any group, as a font is.
pub fn assign_font_parameters(key: &str, params: Vec<i64>) {
  assign_value_inplace(key, Stored::FontDimens(Rc::new(RefCell::new(params))));
}

/// Set `\fontdimen<p>` of the font whose array is stored under `key`; a
/// parameter past its end grows it with zeros (TeX_Fonts.pool.ltxml:138-146,
/// tex.web §580), and a font without one gets an array of its own — as TeX's
/// `\nullfont` has parameters, where Perl's setter does nothing for a font
/// without data (:141). Written in
/// place and so global, as every font parameter assignment is (tex.web §1253):
/// the array is created unbound by any group, whatever `\globaldefs`. Returns
/// false, assigning nothing, for parameter 0 or one past TeX's font memory
/// ([`font::FONT_MEM_SIZE`]).
pub fn set_font_parameter(key: &str, p: usize, value: i64) -> bool {
  let Some(index) = p
    .checked_sub(1)
    .filter(|&index| index < font::FONT_MEM_SIZE)
  else {
    return false;
  };
  let set = |params: &mut Vec<i64>| {
    if params.len() <= index {
      params.resize(index + 1, 0);
    }
    params[index] = value;
  };
  let existing = with_value(key, |stored| match stored {
    Some(Stored::FontDimens(params)) => Some(Rc::clone(params)),
    _ => None,
  });
  match existing {
    Some(params) => set(&mut params.borrow_mut()),
    None => {
      let mut params = Vec::new();
      set(&mut params);
      assign_font_parameters(key, params);
    },
  }
  true
}

/// Undo-stack depth (open TeX groups) — pass-1 streaming telemetry.
pub fn undo_depth() -> usize { state!().undo.len() }

/// The save-stack depth at which an interleaved build step began (`None` =
/// not building, or building after digestion the eager way).
#[thread_local]
static BUILD_FLOOR: std::cell::Cell<Option<usize>> = std::cell::Cell::new(None);

/// Run a build step (a fragment's `Document::absorb`) that the streaming
/// driver interleaves with digestion. Eager Rust and Perl build AFTER
/// digestion, outside every group, so a constructor's local `assign_value`
/// (the aligning context's `ALIGNING_NODE`, a frontmatter store, a
/// counter) never pops; interleaved, the same binding lands in whichever
/// digestion group is open at the seam and is popped at that group's end —
/// tikz-network under tufte-book streamed with a stale root-level aligning
/// context re-exposed by `\end{fullwidth}`, stamping `ltx_align_left` on
/// chapters 4-5 and the appendices. Bindings made at the depth the step
/// began at are therefore global for the step; the frames the build itself
/// pushes above that depth (collected environment frames) keep their local
/// bindings and pop as in eager.
pub fn with_build_floor<R>(f: impl FnOnce() -> R) -> R {
  let previous = BUILD_FLOOR.replace(Some(undo_depth()));
  let result = f();
  BUILD_FLOOR.set(previous);
  result
}

/// A bit of Perl "existence as truth" semantics mixed in with proper boolean lookup
pub fn lookup_bool(key: &str) -> bool { lookup_bool_sym(arena::pin(key)) }

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
    // A list value has no scalar string form; return "" rather than leaking the
    // internal `VecDequeStored[…]`/`Strings` Debug repr (#315). Structural
    // access to list values is via the Rhai `LookupValue` binding, which
    // returns an array (mirroring Perl's `LookupValue` → arrayref).
    Some(v) if v.is_list() => String::new(),
    Some(v) => v.into(),
  }
}
/// like `lookup_value` but only recognizes Int, Bool and Number variants of Stored (default: 0)
pub fn lookup_int(key: &str) -> i64 { lookup_int_sym(arena::pin(key)) }

/// Perl `Conditional.pm:61-62`: bump the global conditional counter and
/// return the new id for the `IfFrame` (see [`State::if_count`]).
pub fn next_if_id() -> i64 {
  let mut state = state_mut!();
  state.if_count += 1;
  state.if_count
}
/// Perl `unshiftValue('if_stack', …)` (Conditional.pm:63): open a conditional (see
/// [`State::if_stack`]).
pub fn push_if_frame(frame: Rc<RefCell<crate::definition::conditional::IfFrame>>) {
  state_mut!().if_stack.push(frame);
}
/// The innermost active conditional, Perl `LookupValue('if_stack')->[0]`.
pub fn top_if_frame() -> Option<Rc<RefCell<crate::definition::conditional::IfFrame>>> {
  state!().if_stack.last().cloned()
}
/// Perl `shiftValue('if_stack')`: close the innermost active conditional.
pub fn pop_if_frame() -> Option<Rc<RefCell<crate::definition::conditional::IfFrame>>> {
  state_mut!().if_stack.pop()
}
/// The number of active conditionals.
pub fn if_stack_depth() -> usize { state!().if_stack.len() }
/// Perl `$LaTeXML::IF_LIMIT`: the runaway ceiling on conditional ids
/// (`0` = unlimited), set by `latexml.sty`'s `iflimit` option.
pub fn if_limit() -> i64 { state!().if_limit }
/// Set [`if_limit`] (Perl `latexml.sty.ltxml:101`).
pub fn set_if_limit(limit: i64) { state_mut!().if_limit = limit; }
/// Perl `Object.pm:163-168`: bump the global absorb counter and return it
/// (see [`State::absorb_count`]).
pub fn next_absorb_id() -> i64 {
  let mut state = state_mut!();
  state.absorb_count += 1;
  state.absorb_count
}
/// Perl `$LaTeXML::ABSORB_LIMIT`: the runaway ceiling on absorbed boxes
/// (`0` = unlimited), set by `latexml.sty`'s `absorblimit` option.
pub fn absorb_limit() -> i64 { state!().absorb_limit }
/// Set [`absorb_limit`] (Perl `latexml.sty.ltxml:104`).
pub fn set_absorb_limit(limit: i64) { state_mut!().absorb_limit = limit; }
/// `lookup_int` variant for hot call sites with a pre-pinned SymStr (see
/// `crate::pin!`). Skips the per-call `arena::pin(key)` hash lookup — the
/// sibling of [`lookup_bool_sym`].
pub fn lookup_int_sym(key: SymStr) -> i64 {
  let state = state!();
  match state.lookup_value_sym(key) {
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
/// a variant of `lookup_value` that casts the value into `Float`
///
/// The float counterpart of [`lookup_number`]. `Float` isn't a TeX register
/// type (see `common::float`), but binding authors need a fractional read/write
/// pair — e.g. `NOMINAL_FONT_SIZE` at the `11pt` class option is `10.95`, which
/// [`lookup_number`]/[`lookup_int`] would truncate to `10` (issue #542).
pub fn lookup_float(key: &str) -> Option<Float> {
  match state!().lookup_value(key) {
    None | Some(Stored::None) => None,
    Some(v) => v.into(),
  }
}
/// a variant of `lookup_value` that casts the value into `Dimension`
pub fn lookup_dimension(key: &str) -> Option<Dimension> {
  let key = register_value_key(key);
  match state!().lookup_value(&key) {
    None | Some(Stored::None) => None,
    Some(v) => v.into(),
  }
}
/// Whether `address` is a slot of a primitive register bank — `\count<n>`,
/// `\dimen<n>`, `\skip<n>`, `\muskip<n>`, `\toks<n>`, `\box<n>` — i.e. what
/// `\newdimen` & co. allocate (tex.web §1224), as opposed to a register named
/// after its own (or another) control sequence.
pub fn is_allocated_register_address(address: &str) -> bool {
  [
    "\\count", "\\dimen", "\\skip", "\\muskip", "\\toks", "\\box",
  ]
  .iter()
  .any(|bank| {
    address
      .strip_prefix(bank)
      .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
  })
}

/// Where a register's value lives: its address, which is not its CS name once the
/// format allocated it (latex.ltx's `\newdimen\columnsep` is `\dimen124`; the
/// dump records it and `def_register` keeps it). Name-keyed readers of a kernel
/// length (`lookup_dimension("\\textwidth")`) go through the address.
fn register_value_key(key: &str) -> Cow<'_, str> {
  if key.starts_with('\\')
    && let Some(register) = lookup_register_definition(&T_CS!(key))
    && is_allocated_register_address(&register.address)
  {
    return Cow::Owned(register.address.clone());
  }
  Cow::Borrowed(key)
}
/// a variant of `lookup_value` that only recognizes a `Stored::Glue`
pub fn lookup_glue(key: &str) -> Option<Glue> {
  let key = register_value_key(key);
  match state!().lookup_value(&key) {
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
      // Release the state borrow first, then read the interned string through
      // the re-entrant arena. The copy is unavoidable: an arena `&str` is not
      // `'static` (it dangles across a realloc — see WISDOM), and `TeXString`
      // borrows only `'static`, so the value has to be owned to cross into the
      // tokenizer. `Mouth::new` copies its input anyway.
      let sym = *sym;
      drop(state);
      arena::with(sym, |astr| {
        Some(mouth::tokenize_internal(TeXString::assembled(
          astr.to_string(),
        )))
      })
    },
    Some(Stored::VecDequeStored(v)) => {
      // Reverting the queue to Tokens routes each String item through
      // `mouth::tokenize_internal`, which takes a *mutable* STATE borrow — so
      // clone the queue and drop the immutable `state` borrow first (mirrors
      // the `Stored::String` branch above). Without this, LookupTokens on a
      // VecDequeStored key (e.g. "class_options") panics "RefCell already
      // borrowed" (#314).
      let vdq = v.clone();
      drop(state);
      Stored::VecDequeStored(vdq).into()
    },
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

/// Quiet sibling of [`lookup_register`] for call sites that mirror Perl's
/// explicit `lookupDefinition(cs) && $defn->isRegister ? $defn->valueOf : <default>`
/// guard — e.g. TeX_Tables `\lx@text@intercol`/`\lx@math@intercol`
/// (`TeX_Tables.pool.ltxml` L639/L646), where a document may legitimately
/// `\renewcommand` a length register (`\tabcolsep`/`\arraycolsep`) into a plain
/// macro. In that case the register-ness is genuinely gone in Perl too, and Perl
/// silently falls back to its default (`Dimension(0)`) with **no warning**.
/// Returns `None` (no warning) when the CS is undefined or is not a register,
/// so the caller can apply its own faithful default.
pub fn lookup_register_quiet(cs: &str) -> Option<RegisterValue> {
  let defn = lookup_definition(&T_CS!(cs)).ok().flatten()?;
  if defn.is_register() {
    defn.value_of(Vec::new())
  } else {
    None
  }
}

/// Faithful port of Perl `LookupDimension` (`Package.pm` L1371-1393, as
/// widened by upstream PR #2829): try to turn the argument into a Dimension,
/// recognizing strings, registers, ….
///
/// * a string that looks like an obvious dimension (`/^[0-9+-.]\w\w+$/`,
///   e.g. `"3pt"` — but NOT `"0.4pt"`, whose `.` fails `\w`) parses directly;
/// * otherwise the string is tokenized: a single token that resolves to a
///   register returns its value ("easy and proper case");
/// * a multi-token sequence is read as a dimension from a fresh mouth;
/// * anything else warns (`expected:register`) unless `noerror`, and yields
///   `None` (Perl returns undef).
///
/// NOTE the #2829 semantics change carried over faithfully: a single token
/// whose definition is a MACRO (e.g. a document that `\def`s `\jot`) no
/// longer reads its body as a dimension — it now falls through to the warn
/// branch. (Perl's digested-Box coercion branch has no Rust equivalent here:
/// all our callers pass strings.)
pub fn lookup_dimension_cs(cs: &str, noerror: bool) -> Option<Dimension> {
  use std::str::FromStr;
  // Obvious dimension string? (Perl: /^[0-9\+\-\.]\w\w+$/)
  let mut chars = cs.chars();
  let leading_sign_or_digit =
    matches!(chars.next(), Some(c) if c.is_ascii_digit() || matches!(c, '+' | '-' | '.'));
  let obvious = leading_sign_or_digit
    && cs.chars().count() >= 3
    && chars.all(|c| c.is_alphanumeric() || c == '_');
  if obvious && let Ok(d) = Dimension::from_str(cs) {
    return Some(d);
  }
  let tokens = mouth::tokenize_internal(TeXString::assembled(cs.to_string()));
  let toks = tokens.unlist();
  if toks.len() == 1 {
    match lookup_definition(&toks[0]) {
      Ok(Some(defn)) if defn.is_register() => {
        // Easy (and proper) case.
        return defn.value_of(Vec::new()).map(|rv| Dimension::from(&rv));
      },
      // Defined but not a register (a `\def`-ized length): fall through and
      // read its body as a dimension. NB this is a deliberate DIVERGENCE
      // from post-#2829 Perl, which unintentionally LOST this path in the
      // rewrite (a single macro token falls to the warn branch upstream) —
      // see KNOWN_PERL_ERRORS #41. Real arXiv papers `\def\arraycolsep{...}`
      // (cluster regressions cover this); pre-#2829 Perl read the body.
      Ok(Some(_)) => {},
      // Undefined single token: warn like Perl and yield nothing.
      _ => {
        if !noerror {
          let message = s!("The control sequence '{}' is not a register", cs);
          Warn!("expected", "register", message);
        }
        return None;
      },
    }
  }
  // Read the token sequence (a defined single CS expands here, exactly like
  // Perl's readingFromMouth) as a dimension from a fresh mouth; an
  // unreadable sequence warns Missing-number inside read_dimension and
  // yields Dimension(0), matching Perl.
  gullet::reading_from_mouth(mouth::Mouth::default(), move || {
    gullet::unread(Tokens::new(toks));
    gullet::read_dimension()
  })
  .ok()
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
    let lookupname = meaning_key(token);
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
  state.journal_touch(TableName::Value, key_sym);
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
  state.journal_touch(TableName::Value, key_sym);
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

/// Bind `key` to `value` inside the named mapping — Perl's `AssignMapping`.
///
/// A mapping is a named hash living in the value table (`TAG_PROPERTIES`,
/// `counter_for_type`, …). The mapping itself is created **globally** on first
/// use, so that entries assigned inside a group are still found from outside
/// it; passing `None` for `value` removes the key. Read entries back with
/// [`with_mapping`].
pub fn assign_mapping<T: Into<Stored>>(map: &str, key: &str, value: Option<T>) {
  let map_sym = arena::pin(map);
  let mut state = state_mut!();
  state.journal_touch(TableName::Value, map_sym);
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

/// The value `key` has in stack frame `frame` (0 = innermost): Perl State.pm:280-286 `valueInFrame`, which skips
/// the bindings the frames above it made.
pub fn value_in_frame(key: &str, frame: usize) -> Option<Stored> {
  let key_sym = arena::pin(key);
  let state = state!();
  let p: usize = state
    .undo
    .iter()
    .take(frame)
    .map(|f| {
      f.table(TableName::Value)
        .get(&key_sym)
        .copied()
        .unwrap_or(0)
    })
    .sum();
  state.value.get(&key_sym).and_then(|v| v.get(p)).cloned()
}

//======================================================================
/// Lookup & assign a character's Catcode
pub fn lookup_catcode(c: char) -> Option<Catcode> {
  // speedup over variant with allocation
  // i.e. "let s = c.to_string();"
  let s = arena::pin_char(c);
  match state!().catcode.get(&s) {
    None => unicode_letter_catcode_default(c),
    Some(cvec) => match cvec.front() {
      Some(Stored::Catcode(cc)) => Some(*cc),
      Some(_) => None, // non-catcode value in catcode table — treat as undefined
      _ => unicode_letter_catcode_default(c),
    },
  }
}

/// Whether the document's engine persona is a Unicode engine: the `luatex` or
/// `xetex` profile of latexml.sty. Both formats load load-unicode-data
/// (latex.ltx:22071-22076), take `\char` code points above 255 natively and
/// have no utf8.def byte model; the engine-identity probes stay per-profile.
pub fn unicode_engine_profile() -> bool {
  lookup_bool("LUATEX_PROFILE") || lookup_bool("XETEX_PROFILE")
}

/// The format-time Unicode letter catcodes of the LuaTeX/XeTeX kernels, as a
/// lazy default for code points the catcode table does not pin.
///
/// latex.ltx:22071-22076 inputs `load-unicode-data` only under LuaTeX/XeTeX, and
/// load-unicode-data.tex:98-99,134-135 assigns `\catcode 11` to every code point
/// of General_Category L or M. pdfTeX never does (utf8.def makes the bytes
/// active), and Perl `State.pm:113-115` letters only ASCII, so outside the
/// LuaTeX profile a non-ASCII char without a table entry stays OTHER
/// (`Mouth::get_next_char`). The profile check runs only for non-ASCII chars,
/// so the ASCII tokenizer path is untouched. U+0080–U+00FF are pinned OTHER by
/// the format dump; the `luatex` option re-letters them eagerly
/// (`latexml_sty/mod.rs`). Witnesses: circledtext (102→0), jnuexam, tikz-bagua;
/// guards `perfect_kernel_batch56::{non_ascii_letters_stay_other_under_pdftex,
/// non_ascii_letters_are_letters_under_luatex}`.
fn unicode_letter_catcode_default(c: char) -> Option<Catcode> {
  if c > '\x7f'
    && ((unicode_engine_profile() && is_unicode_letter_or_mark(c))
      || (lookup_bool("PTEX_PROFILE") && is_ptex_kanji_letter(c)))
  {
    Some(Catcode::LETTER)
  } else {
    None
  }
}

/// K12 — pTeX's control-word letters. pTeX/upTeX give every non-ASCII code
/// point a `kcatcode` (upTeX manual §kcatcode: 16 kanji, 17 kana, 18 other
/// kchar, 19 hangul, 20 modifier; ASCII is always Latin), and a control word
/// after `\` extends over kanji, kana (and, upTeX, hangul/modifier) characters
/// — `\if西暦` is one control sequence (jsarticle.cls:1927 `\newif\if西暦`,
/// ptex-manual `\黄マーカー`); kcatcode 18 (symbols, fullwidth punctuation)
/// ends it, and Latin-1/Greek/Cyrillic stay inputenc bytes (kcatcode 15), so
/// this is the block set of the kanji/kana/hangul kcatcodes, not the Unicode
/// letter class. Under `PTEX_PROFILE` (set by `\NeedsTeXFormat{pLaTeX2e}` /
/// `{upLaTeX2e}`) these default to LETTER, as the LuaTeX profile letters the
/// L/M class; under pdfTeX they stay OTHER, where both LaTeXML engines let
/// `\newif\if西暦`'s bare `\if` to `\iffalse` and every `\if` after it fails.
pub fn is_ptex_kanji_letter(c: char) -> bool {
  matches!(c as u32,
    0x3040..=0x30FF   // Hiragana, Katakana (kana, 17)
    | 0x31F0..=0x31FF // Katakana phonetic extensions (17)
    | 0x1AFF0..=0x1B16F // Kana Supplement / Extended-A/B, Small Kana (17)
    | 0xFF66..=0xFF6F // halfwidth katakana (17; U+FF70 ー is 18)
    | 0xFF71..=0xFF9D
    | 0xFF10..=0xFF19 // fullwidth digits and Latin letters (17)
    | 0xFF21..=0xFF3A
    | 0xFF41..=0xFF5A
    | 0x2E80..=0x2FDF // CJK radicals, Kangxi radicals (16)
    | 0x3105..=0x312F // Bopomofo (16)
    | 0x3190..=0x31EF // Kanbun, Bopomofo extended, CJK strokes (16)
    | 0x3400..=0x4DBF // CJK Unified Ideographs Extension A (16)
    | 0x4E00..=0x9FFF // CJK Unified Ideographs (16)
    | 0xF900..=0xFAFF // CJK Compatibility Ideographs (16)
    | 0x20000..=0x3FFFF // CJK Extensions B-J (16)
    | 0x1100..=0x11FF // Hangul Jamo (19)
    | 0x3130..=0x318F // Hangul compatibility Jamo (19)
    | 0xA960..=0xA97F // Hangul Jamo Extended-A (19)
    | 0xAC00..=0xD7FF // Hangul syllables and Jamo Extended-B (19)
  )
}

/// load-unicode-data.tex:134-135 for U+0080–U+00FF: the LuaTeX/XeTeX format
/// letters every L/M code point, but the latex dump pins that range OTHER
/// (pdfTeX's utf8.def model), so the lazy default in `lookup_catcode` never
/// reaches it. Called at the format seam (`latexml_engine::latex`) and by the
/// `luatex`/`xetex` options of latexml.sty; 65 global assignments.
pub fn assign_luatex_latin1_letters() {
  for n in 0x80u32..=0xFF {
    if let Some(ch) = char::from_u32(n)
      && is_unicode_letter_or_mark(ch)
    {
      assign_catcode(ch, Catcode::LETTER, Some(Scope::Global));
    }
  }
}

/// General_Category L ∪ M, the set load-unicode-data.tex letters. `is_alphabetic`
/// also admits Nl (Roman numerals), which the kernel leaves OTHER; combining marks
/// (M) are added explicitly.
pub fn is_unicode_letter_or_mark(c: char) -> bool {
  (c.is_alphabetic() && !c.is_numeric()) || unicode_normalization::char::is_combining_mark(c)
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
/// The key under which a token's meaning is stored. **All** `\special_relax`-family
/// tokens (`\noexpand`'d forms — the bare `\special_relax` and every
/// `\special_relax\x01<shadowed>`) resolve under the bare `\special_relax` name:
/// they share its `\relax` meaning, faithful to TeX where a `\noexpand`'d token
/// has relax meaning regardless of which token it shadows. The shadowed identity
/// is recovered separately via [`Token::noexpand_shadowed`] (delimited matching
/// only). Use this anywhere a token's *name* keys a meaning lookup or a
/// "same control sequence?" comparison. Cheap on the common path: non-CS tokens
/// short-circuit before any string access.
#[inline]
pub fn meaning_key(token: &Token) -> SymStr {
  if token.is_noexpand_family() {
    pin!("\\special_relax")
  } else {
    token.text
  }
}

/// Get the "Meaning" of a token.
///
/// For active control sequences this may give the definition object (if
/// defined) or another token (if `\let`) or `None`. Any other token is returned
/// as is — which is what makes a `\let`-style comparison between a control
/// sequence and a character token work.
///
/// Clones the stored meaning; use [`with_meaning`] when inspecting it is enough.
pub fn lookup_meaning(token: &Token) -> Option<Stored> {
  if token.get_catcode().is_active_or_cs() && token.text != pin!("") {
    match state!().meaning.get(&meaning_key(token)) {
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

/// Whether the current meaning of `token` was made inside a group open now, so it is gone once that group closes —
/// whether or not a meaning stands outside it (a global assignment clears the counts of the open groups' frames). A
/// binding that queues what it reads to digest later (the frontmatter API) loses such a meaning: aistats's
/// `\aistatsaddress` prints in place inside `\twocolumn[…]`, where 1302.5348 defines `\newcommand{\UMDAddress}{…}`
/// just before it, and where a paper may renew a macro it defined globally (77o review probe `g_globlocal`: pdflatex
/// prints the local meaning).
pub fn meaning_is_group_local(token: &Token) -> bool {
  if !token.get_catcode().is_active_or_cs() {
    return false;
  }
  let key = meaning_key(token);
  let state = state!();
  let Some(values) = state.meaning.get(&key) else {
    return false;
  };
  if values
    .front()
    .is_none_or(|value| matches!(value, Stored::None))
  {
    return false;
  }
  // (the groups above the locked base frame, which holds the global binding: a global assignment is counted there,
  // and clears the counts above it, state.rs `assign_internal`)
  state
    .undo
    .iter()
    .take_while(|frame| !frame.locked)
    .any(|frame| frame.meaning.get(&key).is_some_and(|count| *count > 0))
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
    match state.meaning.get(&meaning_key(token)) {
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

/// Whether `token`'s definition is locked while state is not unlocked, so `install_definition` drops a redefinition.
pub fn is_definition_locked(token: &Token) -> bool { token.with_cs_name(is_name_locked) }

/// Whether `token` means nothing a lock protects: undefined, or `\relax`, what TeX's `\csname` makes of an undefined
/// name (tex.web §372) on the way to every `\@namedef`, `\expandafter\def\csname…` and expl3 `\cs_set:cpn`/`…:cc`
/// definition ([`restore_cleared_binding`]).
fn is_cleared_meaning(token: &Token) -> bool {
  matches!(lookup_meaning(token), None | Some(Stored::None))
    || x_equals(token, &crate::T_CS!("\\relax"))
}

/// Puts back the binding of a locked name that a file's `\let` cleared ([`let_i`], `"{cs}:locked@cleared"`) when
/// the file's new meaning for it is refused, so that the binding's other callers (a raw `\end` running `\csname
/// end<env>\endcsname`, alatex.sty:105-109) still meet it rather than an undefined name.
fn restore_cleared_binding(token: &Token) {
  let cleared_key = token.with_cs_name(|cs| s!("{cs}:locked@cleared"));
  if arena::get(&cleared_key).is_none() || !is_cleared_meaning(token) {
    return;
  }
  if let Some(binding) = lookup_value(&cleared_key).filter(|b| !matches!(b, Stored::None)) {
    state_mut!().assign_internal(
      TableName::Meaning,
      token.get_cs_name(),
      binding,
      Some(Scope::Global),
    );
    // Once: a later `\relax` the document gives the name is its own, and a later clearing `\let` keeps the binding
    // aside again.
    assign_value(&cleared_key, Stored::None, Some(Scope::Global));
  }
}

/// Whether the control sequence named `cs` is locked while state is not unlocked: against every definition
/// (`"{cs}:locked"`), or against those the class and package files make (`"{cs}:locked@files"`), which leaves the
/// document free to define it, as LaTeX and Perl do (the kernel minipage: sect12.rs, OXIDIZED_DESIGN_DIVERGENCES #453).
pub fn is_name_locked(cs: &str) -> bool { !state_is_unlocked() && name_carries_lock(cs) }

/// Whether the control sequence named `cs` carries a lock ([`is_name_locked`]'s two), read as a value whatever the
/// unlocked state: Perl's `\newenvironment`/`\renewenvironment` test `LookupValue('\name:locked')` directly
/// (latex_constructs.pool.ltxml:2787-2803), so a class's `\renewenvironment{thebibliography}` that a binding's hook runs
/// unlocked (as Perl runs before/after-digest code, Primitive.pm:44/:52) still leaves the locked environment alone
/// (wiley2sp's w2sp-pss.clo:444 in `\AtEndOfClass`: 100 of run 336's PushbackLimit papers, 0710.1676, 0809.3379;
/// ptptex: 0706.0316).
pub fn name_carries_lock(cs: &str) -> bool { lock_holds_against(cs, || true) }

/// [`name_carries_lock`] for a known redefinition: `copies_interface` tells whether it takes an optional argument first
/// with a default ([`takes_optional_first`]), which a lock against the files that holds only against copies of the
/// binding's interface ([`files_lock_holds`]) reads.
pub fn lock_holds_against(cs: &str, copies_interface: impl FnOnce() -> bool) -> bool {
  // Probe-only: if "{cs}:locked" was never interned it cannot be bound, so
  // skip both the intern (which permanently grew the arena by one ":locked"
  // twin per defined cs) and the table lookup (2026-08-23 audit R6).
  let locked = |key: String| arena::get(&key).is_some_and(lookup_bool_sym);
  locked(s!("{cs}:locked"))
    || (crate::definition::origin::current_origin()
      != crate::definition::origin::DefinitionOrigin::Document
      && locked(s!("{cs}:locked@files"))
      && files_lock_holds(cs, copies_interface))
}

/// Whether a lock against the class and package files on `cs` holds against this redefinition. Most such locks hold
/// against every one. One that holds only against a COPY of the binding's interface (`"{cs}:locked@copies"`: amsthm's
/// proof, amsthm_sty.rs) holds against a redefinition taking an optional argument with a default first
/// ([`takes_optional_first`]) — amsthm.sty:431's own
/// `{proof}[1][\proofname]`, which smfart.cls:1194, ijmart.cls:336 or thuthesis.cls:4052 re-implement over a
/// `\trivlist` — and lets one with a head of its own through (fascicules.sty:142 `{\textbf{D\'emonstration:} \par}`,
/// NotesTeX.sty:124, a paper's own `\renewenvironment{proof}{…Beweisskizze:…}{…$\blacksquare$}`), whose head text
/// and end mark the binding would replace. The environment's end half (`\end<name>` of a qualified `\<name>`)
/// follows its begin half: locked while that is still the binding's (not cleared, and LaTeXML's own,
/// `DefinitionOrigin::is_latexml_owned`). OXIDIZED_DESIGN_DIVERGENCES #596.
pub fn files_lock_holds(cs: &str, copies_interface: impl FnOnce() -> bool) -> bool {
  // It keeps the binding's own meaning only while that is in force (or kept aside by a file's clearing `\let`,
  // [`let_i`]): once the document, or a definition the lock let through, has replaced or undefined the name, the
  // lock has lapsed and later definitions go through as in LaTeX (amsart, the document's `\let\proof\undefined`, then
  // a package's copy `\newenvironment{proof}[1][Nachweis]`: `perfect_kernel_batch67ps::
  // proof_lock_lapses_once_the_document_undefines_it`; 0705.0545's nccthm.sty:173 `\newcommand*{\proof}[1][]{…}` after
  // the same `\let` goes through on the copy rule alone, its default being empty). The protected
  // meaning is the one the lock names (`\proof:locked@copies` = `\@proof`), not any binding's: llncs's proof that
  // apxproof.sty:46 puts back is not amsthm's (2311.01302).
  if let Some(name) = cs.strip_prefix("\\end")
    && !name.is_empty()
  {
    let begin = s!("\\{name}");
    if let Some(protected) = copies_lock(&begin) {
      return copies_lock_in_force(&begin, &protected) || copies_lock_kept_aside(&begin);
    }
  }
  if let Some(protected) = copies_lock(cs) {
    if !copies_lock_in_force(cs, &protected) && !copies_lock_kept_aside(cs) {
      return false;
    }
    return copies_interface();
  }
  true
}

/// The meaning a lock that holds only against copies protects (`"{cs}:locked@copies"`, [`files_lock_holds`]).
pub fn copies_lock(cs: &str) -> Option<Token> {
  arena::get(s!("{cs}:locked@copies"))?;
  match lookup_value(&s!("{cs}:locked@copies")) {
    Some(Stored::Token(token)) => Some(token),
    _ => None,
  }
}

/// Whether `cs` still has the meaning its copies lock protects.
fn copies_lock_in_force(cs: &str, protected: &Token) -> bool { x_equals(&T_CS!(cs), protected) }

/// Whether a file's clearing `\let` has put the protected meaning of `cs` aside (`"{cs}:locked@cleared"`, [`let_i`]).
fn copies_lock_kept_aside(cs: &str) -> bool {
  lookup_value(&s!("{cs}:locked@cleared")).is_some_and(|b| !matches!(b, Stored::None))
}

/// [`install_definition`]'s verdict, in the locked state, on a redefinition of `cs`: [`lock_holds_against`] read
/// against its own interface, and a redefinition the lock lets through drops what a copies lock kept aside
/// ([`drop_copies_lock_aside`]) — probed only where a lock against the files is set, as every copies lock is one (so
/// a name without one costs the two probes the lock test always made).
fn redefinition_locked(cs: &str, definition: &Stored) -> bool {
  let probe = |key: String| arena::get(&key).is_some_and(lookup_bool_sym);
  if probe(s!("{cs}:locked")) {
    return true;
  }
  if !probe(s!("{cs}:locked@files")) {
    return false;
  }
  let holds = crate::definition::origin::current_origin()
    != crate::definition::origin::DefinitionOrigin::Document
    && files_lock_holds(cs, || takes_optional_first(definition));
  if !holds {
    drop_copies_lock_aside(cs);
  }
  holds
}

/// A definition the copies lock on `cs` let through replaces the protected meaning for good: nothing kept aside is
/// put back, and the end half follows (`\endproof` after a file's own `\proof`).
fn drop_copies_lock_aside(cs: &str) {
  if copies_lock(cs).is_some() && copies_lock_kept_aside(cs) {
    assign_value(
      &s!("{cs}:locked@cleared"),
      Stored::None,
      Some(Scope::Global),
    );
  }
}

/// Record the default of the optional argument a refused redefinition of `token` takes first, as
/// `"{cs}:redefined@default"` (`[1][\proofname]`), for the locked binding to title its own use with — only under a
/// lock that holds only against copies ([`copies_lock`]), whose binding reads it.
pub fn record_redefined_default(token: &Token, default: Tokens) {
  if token.with_cs_name(copies_lock).is_none() {
    return;
  }
  let key = arena::pin(token.with_cs_name(|cs| s!("{cs}:redefined@default")));
  state_mut!().assign_internal(
    TableName::Value,
    key,
    Stored::Tokens(default),
    Some(Scope::Global),
  );
}

/// Whether a definition takes an optional argument first whose default is not empty — `[1][\proofname]`
/// (amsthm.sty:431), `[1][\thu@proof@name]` (thuthesis.cls:4052): the shape of a copy of amsthm's proof, whose
/// optional argument is the head's text and whose default the name, which a lock that holds only against copies reads
/// ([`files_lock_holds`]). An empty default (`[1][]`) adds to a head of the file's own (thmbox.sty:214
/// `\proofname\ #1`) or makes a command, not a proof environment (nccthm.sty:173 `\newcommand*{\proof}[1][]`).
///
/// A robust definition is judged by what it wraps, as [`owns_calls`] judges a robust binding: its own meaning is the
/// parameterless `\protect \cs␣` (`robust_wrapper_body`), defined after the inner `\cs␣` that takes the arguments
/// (`\DeclareRobustCommand\proof[1][\proofname]{…}`).
pub fn takes_optional_first(definition: &Stored) -> bool {
  copy_default(definition).is_some_and(|default| !default.is_empty())
}

/// The default of the optional argument `definition` takes first, if it takes one first ([`takes_optional_first`];
/// a robust definition's by its inner `\cs␣`).
fn copy_default(definition: &Stored) -> Option<Tokens> {
  if let Stored::Expandable(defn) = definition
    && let Some(inner) = robust_wrapper_body(defn, &defn.get_cs())
  {
    return lookup_meaning(&inner).and_then(|inner| copy_default(&inner));
  }
  let params = match definition {
    Stored::Expandable(defn) => defn.get_parameters(),
    Stored::Constructor(defn) => defn.get_parameters(),
    Stored::Primitive(defn) => defn.get_parameters(),
    Stored::MathPrimitive(defn) => defn.get_parameters(),
    _ => None,
  };
  params
    .and_then(|params| params.get_parameters().into_iter().next())
    .filter(|param| param.optional)
    .and_then(|param| param.extra.first().cloned())
}

/// Whether the locked `binding` of `cs` owns its calls, so that the document's own definition of `cs` takes its other
/// uses ([`install_definition`]): its signature reads a braced argument — its first mandatory parameter is one of
/// [`BRACED_ARGUMENT_TYPES`], after optional ones only (`\text{}`, `\title[]{}`, `\cite OptionalMatch:* [][]
/// Semiverbatim`) — or it is itself such an override (a loop redefining it on every iteration). Derived from the
/// binding's own parameter list; a binding that reads its argument downstream (`\section`'s `\@startsection`,
/// `\footnote`, `\url`) or none (`\maketitle`) stays absolutely locked. So does the machinery (user ruling
/// 2026-10-11: the machinery stays absolutely locked): LaTeX's internal namespace, a name with `@`, which LaTeX's
/// convention keeps out of a document's reach until `\makeatletter` (clsguide.tex:187, latex.ltx:1797).
/// Its locks keep LaTeXML's engine model against code that re-implements it (`\alloc@{}{}{}{}{}`, called
/// `\alloc@0\count\countdef\insc@unt`; `\@startsection`, `\@caption`, `\@lbibitem`).
pub fn owns_calls(cs: &Token, binding: &Stored) -> bool {
  if cs.with_cs_name(|name| name.contains('@')) {
    return false;
  }
  // A robust binding is judged by what it wraps (user ruling 2026-10-15, D1 extends to robust bindings): its own
  // meaning is the parameterless `\protect \cs␣` ([`robust_wrapper_body`]), its signature the inner `\cs␣`'s — the
  // kernel `\cite` (sect11.rs) reads `[] Semiverbatim` there. Witnesses math0308041, math0612670, 0705.2148,
  // 0802.3843, 1207.1752, 1401.4197 (eplain.tex:346 `\edef\cite{\the\catcode`@}`; SYNC_STATUS 67n).
  if let Stored::Expandable(defn) = binding
    && !defn.yields_calls
    && let Some(inner) = robust_wrapper_body(defn, cs)
  {
    return lookup_meaning(&inner).is_some_and(|inner_binding| owns_calls(&inner, &inner_binding));
  }
  let params = match binding {
    Stored::Expandable(defn) if defn.yields_calls => return true,
    Stored::Expandable(defn) => defn.get_parameters(),
    Stored::Constructor(defn) => defn.get_parameters(),
    Stored::Primitive(defn) => defn.get_parameters(),
    Stored::MathPrimitive(defn) => defn.get_parameters(),
    _ => None,
  };
  params
    .and_then(|params| {
      params
        .get_parameters()
        .into_iter()
        .find(|param| !param.optional)
    })
    .is_some_and(|param| arena::with(param.name, |name| BRACED_ARGUMENT_TYPES.contains(&name)))
}

/// Whether the document's definition `defn` of `cs` is a VALUE — no parameters — rather than a command
/// ([`install_definition`]): neither it nor what it wraps reads an argument. A robust wrapper (`\protect \cs␣`,
/// [`robust_wrapper_body`]: `\DeclareRobustCommand\mathit[1]{…}`) is judged by its inner `\cs␣`, an ltcmd wrapper
/// (`\__cmd_start_optimized: \cs code`, `\__cmd_start:nNNnnn {o m}…`, `\__cmd_start_expandable:nNNNNn {…}…`:
/// `\RenewDocumentCommand\mathbf{m}{…}`) by its signature: written out (braced) in the general and expandable wrappers,
/// and for the optimized one the argument count ltcmd generates `\cs code` from right after the wrapper
/// (`\l__cmd_current_arg_int`, latex.ltx:1963-1983 `\__cmd_declare_cmd_optimized:Nnn`) (2306.08511, 2306.10719,
/// 2306.05287).
fn defines_value(cs: &Token, defn: &Expandable) -> bool {
  let reads_arguments = |params: Option<&crate::parameter::Parameters>| {
    params.is_some_and(|params| !params.get_parameters().is_empty())
  };
  let wrapped_reads_arguments = |inner: &Token| {
    lookup_definition(inner)
      .ok()
      .flatten()
      .is_some_and(|inner| reads_arguments(inner.get_parameters()))
  };
  if reads_arguments(defn.get_parameters()) {
    return false;
  }
  if let Some(inner) = robust_wrapper_body(defn, cs) {
    return !wrapped_reads_arguments(&inner);
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return true;
  };
  let body = body.unlist_ref();
  if !body
    .first()
    .is_some_and(|first| first.with_str(|name| name.starts_with("\\__cmd_start")))
  {
    return true;
  }
  // `\__cmd_start:nNNnnn {<signature>}…`: a non-empty signature reads arguments.
  let signature_reads = body
    .get(1)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    && body.get(2).is_some_and(|t| t.get_catcode() != Catcode::END);
  let optimized_count = lookup_definition(&T_CS!("\\l__cmd_current_arg_int"))
    .ok()
    .flatten()
    .and_then(|count| count.value_of(Vec::new()))
    .is_some_and(|count| matches!(count, RegisterValue::Number(n) if n.value_of() > 0));
  let optimized = body
    .first()
    .is_some_and(|first| first.with_str(|name| name == "\\__cmd_start_optimized:"));
  // Not `\cs code`: ltcmd installs the wrapper before it rebuilds `\cs code`, which is still the previous
  // declaration's (`\RenewDocumentCommand\text{m}{…}` then `\RenewDocumentCommand\text{}{V}` is a value).
  !signature_reads && !(optimized && optimized_count)
}

/// The parameter types that read one TeX argument, a braced group (`read_arg`/`read_balanced`, base_parameter_types.rs):
/// a binding whose first mandatory parameter is one of them takes a braced argument ([`owns_calls`]).
pub const BRACED_ARGUMENT_TYPES: &[&str] = &[
  "Plain",
  "Position",
  "Semiverbatim",
  "ExpandedSemiverbatim",
  "Undigested",
  "Digested",
  "Expanded",
  "ExpandedPartially",
  "ProtectedExpanded",
  "Verbatim",
  "HyperVerbatim",
  "RequiredKeyVals",
  "RequiredKeyValsStar",
  "RequiredKeyValsPlus",
  "RequiredKeyValsStarPlus",
  "DisplayStyle",
  "TextStyle",
  "ScriptStyle",
  "ScriptscriptStyle",
  "InFractionStyle",
  "InScriptStyle",
];

/// `\lx@locked@<cs>`: where a document's override of the locked `cs` keeps the binding that handles its calls
/// ([`install_definition`]). The binding object itself, so its reversion still names `cs`.
pub fn locked_binding_token(cs: &Token) -> Token {
  T_CS!(cs.with_cs_name(|name| s!("\\lx@locked@{}", name.trim_start_matches('\\'))))
}

/// The inner `\cs␣` that the robust command `cs` wraps (its meaning the parameterless `\protect \cs␣`:
/// `\DeclareRobustCommand`, a binding's `robust => true`, [`let_i`]'s copy of either), whose definition takes the
/// arguments; `None` for any other meaning.
pub fn robust_inner(cs: &Token) -> Option<Token> {
  match lookup_meaning(cs) {
    Some(Stored::Expandable(ref wrapper)) => robust_wrapper_body(wrapper, &wrapper.get_cs()),
    _ => None,
  }
}

/// The other tokens that open a call of `binding` (a [`locked_binding_token`]) besides an explicit `{`: the `[` of
/// each optional argument its signature opens with, and the token of an `OptionalMatch:*`. A robust binding's
/// signature is its inner `\cs␣`'s, as in [`owns_calls`]: `\cite[p.~3]{x}` opens the kernel `\cite`.
pub fn call_openers(binding: &Token) -> Vec<Token> {
  let inner = robust_inner(binding);
  let Ok(Some(defn)) = lookup_definition(inner.as_ref().unwrap_or(binding)) else {
    return Vec::new();
  };
  let mut openers = Vec::new();
  if let Some(params) = defn.get_parameters() {
    for param in params
      .get_parameters()
      .into_iter()
      .take_while(|param| param.optional)
    {
      if arena::with(param.name, |name| name == "OptionalMatch") {
        openers.extend(
          param
            .extra
            .first()
            .and_then(|extra| extra.unlist_ref().first().copied()),
        );
      } else if !param.novalue {
        openers.push(T_OTHER!("["));
      }
    }
  }
  openers
}

/// Every control sequence carrying a `"{cs}:locked"` lock, sorted: for the census of which own their calls
/// ([`owns_calls`]).
pub fn locked_control_sequences() -> Vec<String> {
  let state = state!();
  let mut names: Vec<String> = state
    .value
    .iter()
    .filter(|(_, values)| matches!(values.front(), Some(Stored::Bool(true))))
    .filter_map(|(key, _)| {
      arena::with(*key, |key| {
        key.strip_suffix(":locked").map(ToString::to_string)
      })
    })
    .collect();
  names.sort();
  names
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
  note_copy_provides_assignment(token);
  let csname_sym = token.pin_cs_name();
  state_mut!().assign_internal(TableName::Meaning, csname_sym, meaning, scope);
}

/// Re-point every `\let` copy of `cs`'s current definition — any other meaning bound to that very
/// definition (the same `Rc`: taken while it was in force, and not redefined since) — to `new`.
/// Each binding is replaced in place, so a copy keeps its scope: a local copy stays local, and one
/// a group shadows is `new` again when the group ends. Returns how many bindings were re-pointed;
/// `cs` itself is left to the caller, which installs `new` for it. For a definition TeX treats as
/// fixed before any code runs but installed late here: the job's `\jobname`
/// (`core_interface::install_jobname`).
///
/// The walk over the meaning table is bounded by the definition's holders. Each holder besides
/// `cs`'s own binding and this lookup may be a copy, so a definition nobody else holds needs no
/// walk (no preload ran before the job), and the walk stops once as many copies as holders are
/// found. A holder outside the table (a snapshot) leaves the bound loose, never short.
pub fn rebind_let_copies(cs: &Token, new: &Stored) -> usize {
  let Some(Stored::Expandable(old_def)) = lookup_meaning(cs) else {
    return 0;
  };
  let mut unseen = Rc::strong_count(&old_def).saturating_sub(2);
  if unseen == 0 {
    return 0;
  }
  let own = meaning_key(cs);
  let mut rebound = 0;
  let mut state = state_mut!();
  // (the keys holding a copy, journaled before they are rewritten in place)
  if state.journal.is_some() {
    let copies: Vec<SymStr> = state
      .meaning
      .iter()
      .filter(|(key, values)| {
        **key != own
          && values
            .iter()
            .any(|value| matches!(value, Stored::Expandable(def) if Rc::ptr_eq(def, &old_def)))
      })
      .map(|(key, _)| *key)
      .collect();
    for key in copies {
      state.journal_touch(TableName::Meaning, key);
    }
  }
  'walk: for (key, values) in state.table_mut(TableName::Meaning).iter_mut() {
    if *key == own {
      continue;
    }
    for value in values.iter_mut() {
      if matches!(value, Stored::Expandable(def) if Rc::ptr_eq(def, &old_def)) {
        *value = new.clone();
        rebound += 1;
        unseen -= 1;
        if unseen == 0 {
          break 'walk;
        }
      }
    }
  }
  rebound
}

/// Remove a token's meaning entirely — the token becomes undefined, as if it
/// had never been defined, so a later use takes the normal undefined-CS error
/// path naming the token itself. Bypasses the group-undo journal: intended
/// ONLY for format-bootstrap time (no user groups open), where a format layer
/// retracts a definition inherited from a lower layer that the emulated
/// format must not expose (e.g. plain.tex's `\+` in a LaTeX session — real
/// LaTeX is INITEX-based and never defines it).
pub fn remove_meaning_global(token: &Token) {
  let key = meaning_key(token);
  let mut state = state_mut!();
  state.journal_touch(TableName::Meaning, key);
  state.meaning.remove(&key);
}

/// Begin recording what assignments change, to undo them all ([`AssignmentJournalGuard::rollback`]) or keep them
/// ([`AssignmentJournalGuard::keep`]; also on drop, so a panic leaves no journal open): a transaction over the binding
/// tables, of every assignment however scoped and of the in-place edits of the tables (`push_value`,
/// `assign_mapping`, `lookup_value_mut`, `checkout_value`/`checkin_value`, named-scope activation,
/// `rebind_let_copies`, the graphics path list). Not of what lives outside the tables: a font's parameter array
/// (`Stored::FontDimens`, edited in place) and the fallback/provide-only name sets. Made for the replay of a refused
/// frontmatter setter (sect05.rs `replay_class_setter`), which a setter that errs half-way must not leave half-done
/// (jsty3-author.sty:542's expl3 author list, which its printer then looped over: 2607.24171). `None`, recording
/// nothing, while one is already open (journals do not nest); a journal its guard could not close (the state was
/// borrowed when the guard dropped) is discarded here.
pub fn begin_assignment_journal() -> Option<AssignmentJournalGuard> {
  let mut state = state_mut!();
  if state.journal.is_some() && JOURNAL_GUARD_LIVE.get() {
    return None;
  }
  let frames = state.undo.iter().rev().map(|frame| frame.id).collect();
  state.journal = Some(AssignmentJournal {
    frames,
    saved: rustc_hash::FxHashMap::default(),
  });
  JOURNAL_GUARD_LIVE.set(true);
  Some(AssignmentJournalGuard { open: true })
}

#[thread_local]
/// Whether an [`AssignmentJournalGuard`] is live: one that dropped while the state was borrowed leaves its journal
/// for the next [`begin_assignment_journal`] to discard.
static JOURNAL_GUARD_LIVE: std::cell::Cell<bool> = std::cell::Cell::new(false);

/// The open assignment journal ([`begin_assignment_journal`]).
#[must_use = "an assignment journal is kept or rolled back"]
pub struct AssignmentJournalGuard {
  open: bool,
}

impl AssignmentJournalGuard {
  /// The keys of `table` the journal saw assigned since it began, each with its value then (the front of its stack;
  /// `None`: unbound).
  pub fn assigned(&self, table: TableName) -> Vec<(SymStr, Option<Stored>)> {
    let state = state!();
    let Some(journal) = state.journal.as_ref() else {
      return Vec::new();
    };
    journal
      .saved
      .iter()
      .filter(|((t, _), _)| *t == table)
      .map(|((_, key), (values, _))| (*key, values.as_ref().and_then(|v| v.front().cloned())))
      .collect()
  }

  /// Close the journal, keeping every assignment made since it began.
  pub fn keep(mut self) {
    self.open = false;
    JOURNAL_GUARD_LIVE.set(false);
    state_mut!().journal = None;
  }

  /// Close the journal, undoing every assignment made since it began: each key's values and its counts in the frames
  /// open then are put back as they were. Only when exactly those frames are open again — the same frames, by id,
  /// each one opened since closed and none of them closed: false, undoing nothing (the assignments stay), otherwise.
  pub fn rollback(mut self) -> bool {
    self.open = false;
    JOURNAL_GUARD_LIVE.set(false);
    let mut state = state_mut!();
    let Some(journal) = state.journal.take() else {
      return false;
    };
    if !state
      .undo
      .iter()
      .rev()
      .map(|frame| frame.id)
      .eq(journal.frames.iter().copied())
    {
      return false;
    }
    for ((table_name, key), (values, counts)) in journal.saved {
      match values {
        Some(values) => {
          state.table_mut(table_name).insert(key, values);
        },
        None => {
          state.table_mut(table_name).remove(&key);
        },
      }
      for frame in state.undo.iter_mut() {
        let id = frame.id;
        let frame_table = frame.table_mut(table_name);
        frame_table.remove(&key);
        if let Some((_, n)) = counts.iter().find(|(at, _)| *at == id) {
          frame_table.insert(key, *n);
        }
      }
    }
    true
  }
}

impl Drop for AssignmentJournalGuard {
  fn drop(&mut self) {
    if self.open {
      JOURNAL_GUARD_LIVE.set(false);
      if let Ok(mut state) = STATE.try_borrow_mut() {
        state.journal = None;
      }
    }
  }
}

/// Every control sequence whose current meaning satisfies `keep`, with that meaning: a whole-table scan, for a
/// one-off survey of what a loaded file defined (K11's reader scan, `frontmatter_stores.rs`), never a hot path.
pub fn current_meanings_where(keep: impl Fn(&Stored) -> bool) -> Vec<(SymStr, Stored)> {
  state!()
    .meaning
    .iter()
    .filter_map(|(key, entry)| match entry.front() {
      Some(stored) if !matches!(stored, Stored::None) && keep(stored) => {
        Some((*key, stored.clone()))
      },
      _ => None,
    })
    .collect()
}

// keep this in sync with `lookup_meaning`, it is copied over for optimization purposes
pub fn has_meaning(token: &Token) -> bool {
  if token.get_catcode().is_active_or_cs() && token.text != pin!("") {
    match state!().meaning.get(&meaning_key(token)) {
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
      // A control sequence `\let` to an undefined one (`let_i`) has no definition, as in
      // `lookup_definition`.
      Some(Stored::None) => None,
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
      && (lookup_mathcode_sym(t_sym).unwrap_or(0) == 0x8000)
      && !crate::stomach::digesting_delimiter())
  {
    // `\special_relax`-family tokens digest under the bare `\special_relax` no-op.
    meaning_key(token)
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
/// Diagnostic helper: dump the keys in undo`0`'s value table.
/// For temporary instrumentation only — no production callers should rely on this.
/// The newest `n` values bound to `key` in the Value table, newest first —
/// one entry per frame that bound it (`LXML_TRACE_FRAMES` group-stack dump).
pub fn value_history(key: &str, n: usize) -> Vec<String> {
  let key_sym = arena::pin(key);
  state!()
    .value
    .get(&key_sym)
    .map(|d| d.iter().take(n).map(|v| v.to_string()).collect())
    .unwrap_or_default()
}

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
  if std::env::var_os("LXML_TRACE_FRAMES").is_some() {
    eprintln!(
      "[state] push_frame depth {} -> {}",
      get_frame_depth(),
      get_frame_depth() + 1
    );
  }
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

/// The meanings first made in the innermost group: the control sequences bound at the topmost frame and nowhere
/// outside it (every binding they have is the frame's), with their current binding (a key bound there to nothing is
/// left out).
pub fn top_frame_new_meanings() -> Vec<(SymStr, Stored)> {
  let state = state!();
  let Some(frame) = state.undo.front() else {
    return Vec::new();
  };
  frame
    .meaning
    .iter()
    .filter_map(|(key, count)| {
      let values = state.meaning.get(key)?;
      if values.len() > *count {
        return None;
      }
      values
        .front()
        .filter(|value| !matches!(value, Stored::None))
        .map(|value| (*key, value.clone()))
    })
    .collect()
}

/// The meanings the topmost `n` frames bind — new ones and changes to what an outer frame defined — that are plain
/// groups (`{…}`, `\begingroup`): a frame an environment or a mode opened (it binds `current_environment` or `MODE`)
/// is skipped, its bindings being that construct's own machinery. Each key with the binding of the innermost such frame
/// that binds it (a key bound there to nothing is left out). For a replayed environment begin code (sect05.rs
/// `replay_class_setter`).
pub fn top_group_frames_meanings(n: usize) -> Vec<(SymStr, Stored)> {
  let state = state!();
  let environment = crate::pin!("current_environment");
  let mode = crate::pin!("MODE");
  // how many bindings of a key the frames above the current one hold: the current frame's is at that position
  let mut above: HashMap<SymStr, usize> = HashMap::default();
  let mut chosen: Vec<(SymStr, Stored)> = Vec::new();
  let mut seen: HashSet<SymStr> = HashSet::default();
  for frame in state.undo.iter().take(n) {
    let plain_group = !frame.value.contains_key(&environment) && !frame.value.contains_key(&mode);
    for (key, count) in frame.meaning.iter() {
      let offset = above.entry(*key).or_insert(0);
      if plain_group
        && *count > 0
        && seen.insert(*key)
        && let Some(value) = state
          .meaning
          .get(key)
          .and_then(|values| values.get(*offset))
        && !matches!(value, Stored::None)
      {
        chosen.push((*key, value.clone()));
      }
      *offset += *count;
    }
  }
  chosen
}

/// The keys bound at the topmost frame for the Meaning and Value tables, taken
/// before an autoloaded package loads; see [`hoist_top_frame_package_load`].
pub struct TopFrameKeys {
  meaning: Vec<SymStr>,
  value:   Vec<SymStr>,
}
pub fn snapshot_top_frame_keys() -> TopFrameKeys {
  let state = state!();
  let frame = state.undo.front();
  TopFrameKeys {
    meaning: frame
      .map(|f| f.meaning.keys().copied().collect())
      .unwrap_or_default(),
    value:   frame
      .map(|f| f.value.keys().copied().collect())
      .unwrap_or_default(),
  }
}

/// Hoist what an autoloaded package installed at the topmost frame since
/// `pre` was taken — its meanings and its values — to GLOBAL scope. The
/// trigger (`def_autoload`, OmniBus's `\lx@late@usepackage`, the LaTeX-pool
/// autoload) is cleared and the package's loaded-flag set globally, so what the
/// package defines must live as long. Fired inside a group, natbib's locked
/// `\citep` was otherwise popped while its lock survived, every later `\citep`
/// undefined (arXiv 2605.08349, 2605.10423, 2605.14513, 2605.04028:
/// `Fatal:TooManyErrors`), and its citation style reverted after the group.
pub fn hoist_top_frame_package_load(pre: &TopFrameKeys) {
  hoist_top_frame_delta(TableName::Meaning, &pre.meaning, false);
  hoist_top_frame_delta(TableName::Value, &pre.value, false);
}

/// Hoist the conditionals installed at the topmost frame since `pre_snapshot`
/// was taken to GLOBAL scope: for a package loaded inside a LaTeXML subfile
/// bracket (`require_package`, OXIDIZED_DESIGN #65). Idempotent: keys already
/// in `pre_snapshot` are skipped.
pub fn hoist_top_frame_conditional_delta(pre_snapshot: &[SymStr]) {
  hoist_top_frame_delta(TableName::Meaning, pre_snapshot, true);
}

fn hoist_top_frame_delta(table: TableName, pre_snapshot: &[SymStr], conditionals_only: bool) {
  let pre: rustc_hash::FxHashSet<SymStr> = pre_snapshot.iter().copied().collect();
  let new_keys: Vec<SymStr> = {
    let state = state!();
    state
      .undo
      .front()
      .map(|f| {
        f.table(table)
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
        .table(table)
        .get(&key)
        .and_then(|stack| stack.front().cloned())
    };
    if let Some(value) = current {
      // CONDITIONALS ONLY, for a subfile bracket. The failure this exists for is a definition destroyed
      // while a GLOBAL document hook still reads it, and every witness is a
      // `\newif` conditional (`\ifpgf@external@grabshipout`, OXIDIZED_DESIGN
      // #65). Hoisting a package's ordinary macros too is what makes a second
      // sibling subfile render the FIRST one's content: promoting pkgA's
      // `\newcommand` to global makes pkgB's same-named `\newcommand` a silent
      // no-op, so sibling B shows A's body — silent wrong content, and worse
      // than Perl, which scopes both. `\newif` installs `\ifX` as a Conditional
      // (`\Xtrue`/`\Xfalse` are plain macros the hooks do not read), so this
      // filter keeps every witness working while leaving macros scoped.
      if conditionals_only && !matches!(value, Stored::Conditional(_)) {
        continue;
      }
      // Direct re-bind via assign_internal so we don't need to round-trip a
      // full Token. The Meaning table is keyed by SymStr (the CS name);
      // any future read via `assign_meaning(token, ...)` would reach the
      // same cell. Scope::Global removes higher-frame undo entries and
      // installs at the lowest non-locked frame. A scope move, not a new
      // definition: whatever origin is current, a meaning keeps its standing
      // as a fallback class's guess or not (`fallback_meanings`; the lazy
      // LaTeX.pool hoist had made a pool convenience ours and refused the
      // document's `\newcommand`). Guard:
      // `reentrancy_tests::package_load_hoist_keeps_fallback_standing`.
      let fallback = if table == TableName::Meaning {
        state!().fallback_meanings.get(&key).copied()
      } else {
        None
      };
      let mut state = state_mut!();
      state.assign_internal(table, key, value, Some(Scope::Global));
      if table == TableName::Meaning {
        if let Some(origin) = fallback {
          state.fallback_meanings.insert(key, origin);
        } else {
          state.fallback_meanings.remove(&key);
        }
      }
    }
  }
}
/// Ends the current level of grouping.
/// Note that this is lower level than `\egroup`;
pub fn pop_frame() -> Result<()> {
  if std::env::var_os("LXML_TRACE_FRAMES").is_some() {
    eprintln!(
      "[state] pop_frame depth {} -> {}",
      get_frame_depth(),
      get_frame_depth().saturating_sub(1)
    );
  }
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
  // `\dospecials` carries every ACTIVE shorthand a language registered
  // (babel.def `\bbl@add@special`), so `\url`'s `\@makeother` loop inertizes
  // babel-czech's `-`, babel-german's `"` … A live active `-` in a hyperref
  // url reached the `href` attribute's expansion, where czech.ldf:243-322's
  // `\futurelet` word scanner cannot assign and spun to PushbackLimit
  // (csbulletin `\nolinkurl{a-b}`; pdflatex/Perl clean). Guard:
  // `perfect_kernel_batch54::semiverbatim_inertizes_babel_shorthands`.
  for code in 0x21u8..=0x7Eu8 {
    let ch = code as char;
    if lookup_catcode(ch) == Some(Catcode::ACTIVE) {
      assign_catcode(ch, Catcode::OTHER, Some(Scope::Local));
    }
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
// `VecDequeStored`/`HashTagData`/... values, and a font's parameter array,
// `Stored::FontDimens`, written through its own `Rc<RefCell>` without any
// state borrow) can't corrupt the pre-frame
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
/// `get_prefix` with a pre-pinned SymStr key (see `crate::pin!`) — for the
/// per-`\def` prefix probes in `Expandable::new` and friends.
pub fn get_prefix_sym(prefix: SymStr) -> bool { state!().get_prefix_sym(prefix) }

/// clears the global prefixes
pub fn clear_prefixes() { state_mut!().prefixes = HashMap::default(); }
/// Whether one of TeX's own prefixes (`\global`, `\long`, `\outer`,
/// `\protected`) is waiting for its command. LaTeXML's internal prefixes (the
/// algorithm2e binding's `didpar`) are not TeX's and keep Perl's clearing.
pub fn has_tex_prefixes() -> bool {
  // Probed for every digested primitive: pre-pinned keys, no per-call interning.
  let state = state!();
  !state.prefixes.is_empty()
    && [
      pin!("global"),
      pin!("long"),
      pin!("outer"),
      pin!("protected"),
    ]
    .into_iter()
    .any(|p| state.get_prefix_sym(p))
}

// #======================================================================
/// Named scope bracketing a subfile LaTeXML included itself — a `standalone`
/// child's preamble, an `\import`ed file. Real LaTeX has no group at either spot
/// (standalone *gobbles* the child preamble; import restores its paths by plain
/// `\def` after the `\input`), so a package loaded inside one is an artifact of
/// LaTeXML executing what the real packages skip. Bindings that open such a
/// bracket activate this scope; `require_package` reads it to decide whether a
/// load must outlive the bracket. See OXIDIZED_DESIGN #65.
///
/// The name carries the frame depth of the bracket that opened it — Perl's own
/// `section:4` / `label:foo` convention (State.pm L965-975) — because activity
/// alone is not enough: `StashActive` is `Scope::Local` at the bracket's frame,
/// so a plain "is the region active?" test is ALSO true at every deeper frame,
/// and an author's `{\usepackage{…}}` written *inside* a subfile preamble would
/// be hoisted as well. That is a downgrade: pdflatex and Perl both leave such a
/// package lost. Matching the depth confines the region to the bracket's own
/// level.
pub fn subfile_scope_at_depth(depth: usize) -> SymStr { arena::pin(format!("subfile:{depth}")) }

/// The subfile scope for the CURRENT frame depth — what a bracket activates on
/// opening, and what `require_package` tests before hoisting.
pub fn subfile_scope_here() -> SymStr { subfile_scope_at_depth(get_frame_depth()) }

/// Is the named scope currently active? See `scope_active_in` for why this is a
/// front-value test and not a presence test, and `subfile_scope_at_depth` for the region
/// marker it supports.
pub fn is_scope_active(scope: SymStr) -> bool { scope_active_in(&state!(), scope) }

/// Perl's scope-activity test, shared by the three call sites that need it
/// (`is_scope_active`, `activate_scope`, `deactivate_scope`; `get_active_scopes`
/// deliberately still enumerates KEYS, faithful to Perl State.pm L722-725, which
/// has the same latent quirk — it has no callers):
/// the truthiness of the FRONT `stash_active` value — `$$self{stash_active}
/// {$scope}[0]` in `activateScope` (State.pm L682) and `deactivateScope`
/// (L700).
///
/// Presence is NOT the test, and cannot be: deactivation OVERWRITES the front
/// value with a falsy one rather than removing the key — an ordinary global
/// assignment (`assign_internal(… 'stash_active', $scope, 0, 'global')`,
/// State.pm L701; ours passes `Stored::Bool(false)`). A global assign replaces
/// rather than layers: it drops the per-frame counts down to the locked frame,
/// pops exactly that many values, and leaves ONE. So the front value is the
/// whole state. (A delete is not available anyway — `stash_active` rides the
/// generic table + undo machinery, whose per-frame pop counts a removed key
/// would desynchronise.) Pinned by
/// `reentrancy_tests::scope_activity_tracks_value_not_presence`.
///
/// Reach: the production consumers — `counter/dialect.rs` (a reference number's
/// `<ctr>:<refnum>` scope, deactivated then re-activated as the counter moves,
/// mirroring Perl `Package.pm` L774-779) and `latex_constructs.rs`'s `label:`
/// scopes — activate names that nothing currently STASHES into, so an activation
/// installs no bindings and the re-activation fix is correct but latent. It
/// becomes observable the moment a binding defines with `scope => "<ctr>:<n>"`.
/// That is why the guards are unit-level: there is no output difference to
/// assert end-to-end yet. (The one `Scope::Named` stash writer, `declare.rs`'s
/// declaration scope — `id:<unit id>` or `label:<label>` — is consumed by `rewrite.rs`
/// by prefix, not by activation.)
///
/// The local/global asymmetry is deliberate (Perl's own note above
/// `deactivateScope`): activation is `local`, so it expires with its group
/// without a teardown call — which is what makes a named scope usable as a
/// region marker (see `subfile_scope_at_depth`) — while deactivation is `global` so it
/// survives group exit. A local deactivation would be undone by the very group
/// that contained it.
fn scope_active_in(state: &State, scope: SymStr) -> bool {
  state
    .stash_active
    .get(&scope)
    .and_then(|entry| entry.front())
    .is_some_and(|v| !matches!(v, Stored::Bool(false)))
}

/// Activates all stashed definitions for the named scope. No-op if the scope is already active.
pub fn activate_scope(scope: SymStr) {
  let mut state = state_mut!();
  // Perl L682 `if (!$$self{stash_active}{$scope}[0])` — do not re-activate if
  // already active, but a scope that was DEACTIVATED must be activatable again.
  if scope_active_in(&state, scope) {
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
    state.journal_touch(table_name, key);
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
  // Perl L700 `if ($$self{stash_active}{$scope}[0])` — only an ACTIVE scope is
  // deactivated; a second deactivation must not re-run the pop below.
  if !scope_active_in(&state, scope) {
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
    state.journal_touch(table_name, key);
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
    "em" | "zw" | "zh" => font_metric(|f| f.get_em_width()),
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
    "em" | "zw" | "zh" => (font_metric(|f| f.get_em_width()), UNITY),
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
  // OPENABILITY. Most tags get 1.0; `ltx:picture` gets 0.5 (latex_constructs.pool.ltxml:4943) so it
  // loses path-priority against full auto-openers (para, p, text, item, …).
  // We scale to u32 (100 = full, 50 = half, `AutoOpen`) to keep integer
  // arithmetic; the `desirability * openability / 100` recursion mirrors
  // Perl's float math. An openability of 0 is no auto-opener (Perl's falsy
  // `$::OPENABILITY{$kid}`, L216).
  let mut openability: SymHashMap<u32> = SymHashMap::default();
  // Collect all known tags: from the schema model AND from state tag_properties
  let mut all_tags: HashSet<SymStr> = model::get_tags().into_iter().collect();
  for tag in state!().tag_properties.keys() {
    all_tags.insert(*tag);
  }
  for tag in &all_tags {
    if let Some(x) = state!().tag_properties.get(tag)
      && let Some(AutoOpen(priority)) = x.auto_open
      && priority > 0
    {
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
/// Run `body` with `cs` temporarily `\let` to `target` (local scope), then
/// restore `cs`'s prior meaning — the `{\let\cs\target …}` idiom of latex.ltx
/// (`\protected@edef`'s `\let\protect\@unexpandable@protect`, :1442) without
/// a save-stack group, for a reader that must not open one. The prior
/// meaning is restored whether or not `body` fails. A `cs` with no meaning
/// is restored to none.
pub fn with_let<R>(cs: &Token, target: &Token, body: impl FnOnce() -> Result<R>) -> Result<R> {
  let saved = lookup_meaning(cs);
  let_i(cs, target, None);
  let result = body();
  match saved {
    Some(meaning) => assign_meaning(cs, meaning, None),
    None => assign_meaning(cs, Stored::None, None),
  }
  result
}

/// A binding's `\let` (`Let!`): [`let_i`], unless a provide-only load declines it ([`provide_only_declines`]).
/// TeX's own `\let` and `\futurelet` are `let_i`.
pub fn let_definition(token1: &Token, token2: &Token, scope: Option<Scope>) {
  if !provide_only_declines(token1) {
    let_i(token1, token2, scope);
  }
}

pub fn let_i(token1: &Token, token2: &Token, scope: Option<Scope>) {
  {
    use crate::definition::origin::{DefinitionOrigin, current_origin, stand_in_loading};
    // (only while a stand-in binding loads, whose `\let`s a copy may meet: not every `\newif` flag of the pool)
    if stand_in_loading()
      && matches!(
        current_origin(),
        DefinitionOrigin::Unknown | DefinitionOrigin::Pool | DefinitionOrigin::Binding
      )
      && matches!(token1.get_catcode(), Catcode::CS | Catcode::ACTIVE)
    {
      let key = meaning_key(token1);
      state_mut!().binding_lets.insert(key);
    }
  }
  let meaning =// if token2.get_dont_expand().is_some() {
  //   Stored::Token(token2.clone())
  // } else {
    lookup_meaning(token2)
      .unwrap_or(Stored::None);
  // };
  if copy_provides_keeps_saved_meaning(token1, token2, &meaning) {
    after_assignment();
    return;
  }
  // A lock against the class and package files (`"{cs}:locked@files"`, [`name_carries_lock`]) holds for their `\let`
  // as for their `\def` ([`install_definition`]): arabtex's asect.sty:297 `\let\endabstract\endquotation`, which its
  // own raw `\end` (alatex.sty:105-109) then calls, closed the kernel abstract's body as a quotation (2401.14559). A
  // plain `:locked` is left to a `\let` to a non-robust meaning, as Perl's `Let` checks none (Package.pm:383-390); a
  // `\let` to a robust command takes the body copy below, whose wrapper the lock refuses (OXIDIZED_DESIGN #568). So
  // are the engine's own meaning writes ([`assign_meaning`]: `\csname`'s `\relax`, restores). A `\let` to an undefined token only clears
  // the name, and is kept: the "undefine, then define anew" idiom (projlib-author.sty:243-246 `\cs_undefine:N
  // \endabstract` before `\newenvsc{abstract}`, whose scontents `\NewDocumentEnvironment` "already defined" check
  // would fail; the homework and jwjournal manuals). The binding is kept aside (`"{cs}:locked@cleared"`) and back in
  // place when the file's new meaning is refused ([`restore_cleared_binding`]), whether the name is still undefined or
  // `\csname` has made it `\relax` on the way. Read at the origins `install_definition` reads the lock at (every one
  // but the document's, [`name_carries_lock`]). OXIDIZED_DESIGN #569; guards `perfect_kernel_batch66b::file_let_*`,
  // `…::refused_*_restores_the_binding`.
  // A lock that holds only against copies of the binding's interface ([`files_lock_holds`]) judges the `\let` by the
  // meaning it copies: one taking an optional argument with a default first is refused as above; any other goes
  // through, the binding kept aside as for a clearing `\let`, so that a copy defined next and refused puts it back
  // (a package's `\let\proof\relax` … `\newenvironment{proof}[1][\proofname]`; guard
  // `perfect_kernel_batch67s::package_redefined_proof_keeps_the_binding`). Not GT.cls (2503.05183): its `\let\proof
  // \relax` (:1469-1470) is the class's, but its copy is defined inside `\AtBeginDocument` (:1488-1494), at the
  // document's origin, and honoured (OXIDIZED_DESIGN_DIVERGENCES #596).
  if crate::definition::origin::current_origin()
    != crate::definition::origin::DefinitionOrigin::Document
    && !state_is_unlocked()
    && matches!(token1.get_catcode(), Catcode::CS | Catcode::ACTIVE)
    && token1.with_cs_name(|cs| arena::get(s!("{cs}:locked@files")).is_some_and(lookup_bool_sym))
  {
    let cs = token1.with_cs_name(|cs| cs.to_string());
    let holds = files_lock_holds(&cs, || takes_optional_first(&meaning));
    if holds && !matches!(meaning, Stored::None) {
      restore_cleared_binding(token1);
      after_assignment();
      return;
    }
    let protected = copies_lock(&cs);
    let clearing = matches!(meaning, Stored::None) || x_equals(token2, &crate::T_CS!("\\relax"));
    if clearing {
      if !is_cleared_meaning(token1)
        && protected
          .as_ref()
          .is_none_or(|protected| copies_lock_in_force(&cs, protected))
        && let Some(binding) = lookup_meaning(token1)
      {
        let cleared_key = token1.with_cs_name(|cs| s!("{cs}:locked@cleared"));
        assign_value(&cleared_key, binding, Some(Scope::Global));
      }
    } else {
      drop_copies_lock_aside(&cs);
    }
  } else if !state_is_unlocked()
    && matches!(token1.get_catcode(), Catcode::CS | Catcode::ACTIVE)
    && (matches!(meaning, Stored::None) || x_equals(token2, &crate::T_CS!("\\relax")))
    && !is_cleared_meaning(token1)
    && token1.with_cs_name(|cs| arena::get(s!("{cs}:locked")).is_some_and(lookup_bool_sym))
    && let Some(binding) = lookup_meaning(token1)
  {
    // A plain `:locked` binding a file's `\let` clears is kept aside too, the `\let` itself going through as in Perl,
    // and back in place when the file's own new meaning is refused ([`restore_cleared_binding`]): eulerpx.sty:130-132
    // `\let\mathscr=\undefined` then `\DeclareMathAlphabet\mathscr{U}{zeus}{m}{n}`, which the lock on the kernel's
    // `\mathscr` (sect07.rs) refuses, left `\mathscr` undefined, an Error at every use (2311.14793, 2501.10505,
    // 2510.26876; KNOWN_PERL_ERRORS #863). OXIDIZED_DESIGN #569. The document's own `\let` too, at every origin: the
    // plain lock refuses the document's redefinition as it refuses a file's (`\let\mathcal\undefined
    // \DeclareMathAlphabet{\mathcal}{OMS}{cmsy}{m}{n}` left `\mathcal` undefined; `\let\mathscr\relax` before
    // `\DeclareMathAlphabet{\mathscr}{U}{rsfs}{m}{n}` lost its script font), and a stale binding a package's
    // group-local clear set aside is replaced by the current one. A document value-definition after the restore still
    // reaches the binding's call owner (OXIDIZED_DESIGN #500). The `:locked@files` branch above keeps its origin test.
    let cleared_key = token1.with_cs_name(|cs| s!("{cs}:locked@cleared"));
    assign_value(&cleared_key, binding, Some(Scope::Global));
  }
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
  // So the body half is `\let` too: `\let \origref<space> \ref<space>`, and
  // the two CSes own independent body slots. This is NOT TeX's `\let`, which
  // copies the wrapper `\protect \ref<space>` itself (and loops on that idiom
  // for a robust `\ref`; latex.ltx's `\ref` is not robust, the binding's is):
  // `\meaning\origref` reads `\protect \origref  ` where TeX reads `\protect
  // \ref  `, and `\ifx` compares the bodies ([`same_robust_body`]).
  // OXIDIZED_DESIGN_DIVERGENCES #315.
  //
  // Witnesses: canvas-3 stage-23 0810.0695 (PlanarMain.tex's
  // `\ifpdf...\else \let\origref\ref \DeclareRobustCommand\ref{
  // \@ifstar\origref\origref}\fi` triggers via the else-branch
  // when `\ifpdf` is false; its `\pdfoutput=1` makes it true since K6).
  //
  // Not when the body calls `\<token1><space>` itself: copied there it would call itself, so `\<token1><space>` is left
  // alone, as TeX's `\let` leaves it, and `\<token1>` takes `\<token2>`'s wrapper. mathfixs' autobold
  // (mathfixs.sty:24-32) builds `\mafx@bfseries` by one-step expansion of the robust `\bfseries` — its body
  // `\protect\bfseries<space>\ifmmode…` — and then `\let\bfseries=\mafx@bfseries` (33 of run 336's papers ran to
  // PushbackLimit, 2205.10090, 2211.07279). A copy over an existing body slot is still a copy (`\DeclareCommandCopy`
  // onto a robust command, `\let\origref\ref` run twice).
  //
  // Not over a locked inner: the body copy writes `\<token1><space>` directly, so for a locked robust binding (the
  // kernel `\cite`, whose inner is locked against the files, OXIDIZED_DESIGN #568) a class's `\let\cite\mycite` would
  // put its body under the binding's own wrapper, whose reinstallation the lock then refuses. Such a `\let` takes
  // TeX's (and Perl's `Let`) plain path below instead. A locked binding whose inner is free (a constructor: the
  // kernel `\LaTeX`) keeps the copy, whose wrapper the lock refuses: bera.sty:12-23's `\let\LaTeX=\fveLaTeX` (a
  // `\DeclareRobustCommand`) would otherwise replace the logo with its typeset boxes (enumitem, colorspace and mathador
  // manuals); its `\let\TeX=\fveTeX` (:10-11, a `\def`) takes the plain path and replaces `\TeX`, as in Perl. Guard
  // `perfect_kernel_batch66b::robust_let_over_a_locked_constructor_keeps_it`.
  if let Stored::Expandable(ref defn) = meaning
    && let Some(token2_space) = robust_wrapper_body(defn, token2)
    && !token1.with_str(|s| body_mentions(&token2_space, &s!("{s} ")))
    && !token1.with_str(|s| is_name_locked(&s!("{s} ")))
  {
    let token1_space = crate::T_CS!(token1.with_str(|s| s!("{s} ")));
    // (1) Copy `\<token2><space>` body to `\<token1><space>`
    // so the two CSes have independent body slots.
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
  assign_meaning(token1, meaning, scope);
  after_assignment();
}
/// `XEquals` check for two token arguments
pub fn x_equals(token1: &Token, token2: &Token) -> bool {
  let def1_opt = lookup_meaning(token1); // # token, definition object or None
  let def2_opt = lookup_meaning(token2); // ditto
  match (def1_opt, def2_opt) {
    (Some(def1), Some(def2)) => def1 == def2 || same_robust_body(token1, &def1, token2, &def2),
    (None, None) => true, // true if both undefined
    (..) => false,        // False, if only one has 'meaning'
  }
}

/// Whether the macro `cs`'s token body mentions the control sequence named `name` (a closure body is opaque: no). By
/// name, so no token is interned for a name that may never exist.
fn body_mentions(cs: &Token, name: &str) -> bool {
  match lookup_meaning(cs) {
    Some(Stored::Expandable(defn)) => match defn.expansion {
      Some(ExpansionBody::Tokens(ref tks)) => {
        tks.unlist_ref().iter().any(|t| t.with_str(|s| s == name))
      },
      _ => false,
    },
    _ => false,
  }
}

/// The inner control sequence of a robust wrapper: `defn` is `cs`'s meaning,
/// a macro whose expansion is exactly `\protect \<cs><space>`
/// (`\DeclareRobustCommand`, a binding's `robust => true`, or [`let_i`]'s copy
/// of either).
fn robust_wrapper_body(defn: &Expandable, cs: &Token) -> Option<Token> {
  let Some(ExpansionBody::Tokens(ref tks)) = defn.expansion else {
    return None;
  };
  let body = tks.unlist_ref();
  if body.len() != 2 || !body[0].with_str(|s| s == "\\protect") {
    return None;
  }
  let inner = cs.with_str(|s| s!("{s} "));
  body[1]
    .with_str(|s| s == inner)
    .then(|| crate::T_CS!(inner))
}

/// `\ifx` of two robust wrappers whose inner macros have the same meaning.
/// In TeX `\let\x\bfseries` copies the wrapper `\protect \bfseries␣` itself,
/// so `\ifx\x\bfseries` is true (tex.web §507 compares the token lists);
/// [`let_i`] instead gives `\x` its own wrapper `\protect \x␣` and a copy of
/// the body under `\x␣` (arXiv 0810.0695, OXIDIZED_DESIGN_DIVERGENCES #315), so
/// the wrappers differ by name only and the bodies decide. `\ifx\reset@font
/// \normalfont` relies on it, and in the preamble amsthm.sty:209-212's
/// `\nonslanted` (`\let\@tempa\csname\f@shape shape\endcsname \ifx\@tempa
/// \itshape`; in the body the shape switches are `\protected` macros, a plain
/// meaning comparison). The bodies must be the same definition, not only
/// equal: two robust commands declared apart with the same body compare
/// false, as in TeX. A `\LetLtxMacro` copy (letltxmacro.sty) and a
/// `\NewCommandCopy`/`\DeclareCommandCopy`/`\RenewCommandCopy` copy (each its
/// own wrapper over a `\let` of the body) compare true, where TeX says false:
/// their state is [`let_i`]'s.
fn same_robust_body(token1: &Token, def1: &Stored, token2: &Token, def2: &Stored) -> bool {
  let (Stored::Expandable(defn1), Stored::Expandable(defn2)) = (def1, def2) else {
    return false;
  };
  if defn1.paramlist != defn2.paramlist {
    return false;
  }
  match (
    robust_wrapper_body(defn1, token1),
    robust_wrapper_body(defn2, token2),
  ) {
    (Some(inner1), Some(inner2)) => match (lookup_meaning(&inner1), lookup_meaning(&inner2)) {
      // The body [`let_i`] copies is the source's own `Rc<Expandable>`, so its
      // `cs` still names the inner macro it was defined as (`\bfseries␣`, as
      // the dump keeps it): equal bodies from two separate declarations
      // (`\DeclareRobustCommand\ra{x}`, `\DeclareRobustCommand\rb{x}`) stay
      // unequal, as their wrappers are in TeX.
      (Some(Stored::Expandable(body1)), Some(Stored::Expandable(body2))) => {
        body1 == body2 && body1.cs == body2.cs
      },
      (Some(body1), Some(body2)) => body1 == body2,
      _ => false,
    },
    _ => false,
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
  match remove_value_sym(pin!("afterAssignment")) {
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
/// See `docs/performance/SOURCE_PROVENANCE.md`.
pub fn source_map_enabled() -> bool { state!().source_map }
pub fn set_source_map_flag(val: bool) {
  let mut state = state_mut!();
  state.source_map = val;
}

/// Find-or-append a source file in the document-level `sources` table,
/// returning its integer `tag` (index). The per-element `data-sourcepos`
/// attribute carries this compact integer rather than a path — the
/// Source-Map-v3 `sources` convention (compact + anonymisable). Only
/// called on the source-map path. See `docs/performance/SOURCE_PROVENANCE.md` §0.1.
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
  std::mem::take(&mut state_mut!().pending_resources)
}
pub fn reset_pending_resources() { state_mut!().pending_resources = Vec::new(); }
pub fn get_indirect_model_relationship(tag: SymStr, childtag: SymStr) -> Option<SymStr> {
  match state!().indirect_model.as_ref().unwrap().get_sym(tag) {
    Some(sub_m) => sub_m.get_sym(childtag).copied(),
    None => None,
  }
}

pub fn get_bindings_dispatch() -> Option<ResolvingBindingDispatcher> {
  state!().bindings_dispatch.clone()
}
pub fn get_extra_bindings_dispatch() -> Option<BindingDispatcher> {
  state!().extra_bindings_dispatch.clone()
}
pub fn set_bindings_dispatch(dispatcher: ResolvingBindingDispatcher) {
  let mut state = state_mut!();
  state.bindings_dispatch = Some(dispatcher);
}
pub fn set_extra_bindings_dispatch(dispatcher: BindingDispatcher) {
  let mut state = state_mut!();
  state.extra_bindings_dispatch = Some(dispatcher);
}

std::thread_local! {
  static PLAIN_TEX_DOCUMENT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// Records whether the document about to be converted is a plain TeX one, by arXiv AutoTeX's test (its main file, or a
/// file a cue-less main file inputs, names neither `\documentclass`, `\documentstyle` nor `\begin{document}` before an
/// `\endinput`); set per conversion before the session's preloads load (`latexml::converter::Converter::note_main_source`).
pub fn set_plain_tex_document(plain: bool) { PLAIN_TEX_DOCUMENT.with(|c| c.set(plain)); }
/// Whether the document being converted is a plain TeX one ([`set_plain_tex_document`]): latexml.sty's preload must not
/// load the LaTeX format for it.
pub fn plain_tex_document() -> bool { PLAIN_TEX_DOCUMENT.with(std::cell::Cell::get) }
std::thread_local! {
  static DVI_DRIVER_OPTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// Records whether the main file's `\documentclass` names a DVI driver option (`dvips`, `dvipdfmx`, …) and the file sets
/// no `\pdfoutput=1`: the document is a latex+dvips one (the K6 ruling's DVI cue, stream F).
pub fn set_dvi_driver_option(dvi: bool) { DVI_DRIVER_OPTION.with(|c| c.set(dvi)); }
/// Whether the document is a latex+dvips one by its class options ([`set_dvi_driver_option`]).
pub fn dvi_driver_option() -> bool { DVI_DRIVER_OPTION.with(std::cell::Cell::get) }
std::thread_local! {
  static PDF_DRIVER_OPTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// Records whether the main file's `\documentclass` names the `pdftex` driver option: the document is a pdflatex one,
/// whatever PostScript files it also ships (the K6 ruling's PDF cue, mirroring [`set_dvi_driver_option`]).
pub fn set_pdf_driver_option(pdf: bool) { PDF_DRIVER_OPTION.with(|c| c.set(pdf)); }
/// Whether the document is a pdflatex one by its class options ([`set_pdf_driver_option`]).
pub fn pdf_driver_option() -> bool { PDF_DRIVER_OPTION.with(std::cell::Cell::get) }
/// What the figures a source includes say of its output mode (the K6 ruling's figure cue,
/// OXIDIZED_DESIGN_DIVERGENCES #537): read from the `\includegraphics`-family targets the main file and the files it
/// inputs name, resolved against the source tree, and from a pstricks load.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FigureOutputCue {
  /// No included figure resolves and no pstricks is loaded: the source tree's files decide.
  #[default]
  Undecided,
  /// Every included figure that resolves has a PostScript form, one has no other, or pstricks is loaded: latex+dvips.
  Dvi,
  /// An included figure exists only as PDF/PNG/JPEG, which latex+dvips cannot include: pdflatex.
  Pdf,
}
std::thread_local! {
  static FIGURE_OUTPUT_CUE: std::cell::Cell<FigureOutputCue> =
    const { std::cell::Cell::new(FigureOutputCue::Undecided) };
}
/// Records the output-mode cue of the figures the document's source includes ([`FigureOutputCue`]); set per conversion
/// before the session's preloads load.
pub fn set_figure_output_cue(cue: FigureOutputCue) { FIGURE_OUTPUT_CUE.with(|c| c.set(cue)); }
/// The output-mode cue of the figures the document's source includes ([`set_figure_output_cue`]).
pub fn figure_output_cue() -> FigureOutputCue { FIGURE_OUTPUT_CUE.with(std::cell::Cell::get) }
std::thread_local! {
  static SOURCE_DATE_EPOCH: std::cell::Cell<Option<i64>> = const { std::cell::Cell::new(None) };
}
/// Records the date of the document's source (Unix seconds, UTC): the newest modification time among the TeX source
/// files of the bundle it came in (`Config::source_date_epoch`), `None` when the source carries none; set per
/// conversion before the session's preloads load. ar5iv's archival preload reads it as the job's clock.
pub fn set_source_date_epoch(epoch: Option<i64>) { SOURCE_DATE_EPOCH.with(|c| c.set(epoch)); }
/// The date of the document's source ([`set_source_date_epoch`]).
pub fn source_date_epoch() -> Option<i64> { SOURCE_DATE_EPOCH.with(std::cell::Cell::get) }

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

/// Read SEARCHPATHS from the group-scoped value table (Perl
/// `LookupValue('SEARCHPATHS')`). Mirrors [`get_graphics_paths`]: the list is a
/// group-scoped value, not a plain field, so an `\import`/`\subimport` group
/// reverts its change at `}` and a package's global add persists.
pub fn get_search_paths() -> Vec<String> {
  lookup_value("SEARCHPATHS")
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
pub fn with_search_paths<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&[String]) -> R {
  caller(&get_search_paths())
}
/// Global append (Perl `PushValue(SEARCHPATHS)`) — a persistent search dir.
pub fn add_search_path(path: String) {
  let mut paths = get_search_paths();
  paths.push(path);
  set_search_paths(paths);
}
/// Global prepend (Perl `UnshiftValue(SEARCHPATHS)`) — a persistent search dir.
pub fn search_paths_push_front(path: String) {
  let mut paths = get_search_paths();
  paths.insert(0, path);
  set_search_paths(paths);
}
/// Replace SEARCHPATHS GLOBALLY (Perl `AssignValue(SEARCHPATHS => [...], 'global')`).
/// For the local-by-default `\import` scoping, use [`set_search_paths_local`].
pub fn set_search_paths(paths: Vec<String>) { assign_search_paths(paths, Scope::Global); }
/// Replace SEARCHPATHS in the CURRENT group only (Perl `AssignValue(SEARCHPATHS
/// => [...])` default-local): reverted when the enclosing `\import`/`\subimport`
/// group closes. This is what makes `import.sty` faithful without an explicit
/// save/restore stack.
pub fn set_search_paths_local(paths: Vec<String>) { assign_search_paths(paths, Scope::Local); }
fn assign_search_paths(paths: Vec<String>, scope: Scope) {
  let vdq: VecDeque<Stored> = paths
    .into_iter()
    .map(|p| Stored::String(arena::pin(&p)))
    .collect();
  assign_value("SEARCHPATHS", Stored::VecDequeStored(vdq), Some(scope));
}
pub fn has_search_paths() -> bool { !get_search_paths().is_empty() }
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
  state.journal_touch(TableName::Value, key);
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
    // A font's parameter array: serializable (data only), as an `IA` record.
    FontDimens(_) => true,
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
mod conditional_counter_tests {
  use super::*;

  /// Perl `Conditional.pm:61-62` assigns `if_count` *globally* so the
  /// per-conditional id keeps climbing across group exits; the typed field
  /// must not revert with the frame it was bumped in, and the limit is a
  /// plain global (`$LaTeXML::IF_LIMIT`), unlimited when `0`.
  #[test]
  fn if_ids_climb_across_group_exits() {
    reset_thread_state();
    let first = next_if_id();
    push_frame();
    let inner = next_if_id();
    pop_frame().expect("pop the probe frame");
    let after = next_if_id();
    assert_eq!((first, inner, after), (1, 2, 3));
    assert_eq!(
      if_limit(),
      0,
      "unlimited until latexml.sty's iflimit sets it"
    );
    set_if_limit(3_999_999);
    assert_eq!(if_limit(), 3_999_999);
    // The absorb counter is the same mechanism (Perl Object.pm:165).
    push_frame();
    assert_eq!(next_absorb_id(), 1);
    pop_frame().expect("pop the probe frame");
    assert_eq!(next_absorb_id(), 2);
    set_absorb_limit(7);
    assert_eq!(absorb_limit(), 7);
    reset_thread_state();
    assert_eq!(next_if_id(), 1, "a fresh conversion restarts the counter");
    assert_eq!(if_limit(), 0, "and forgets the previous limit");
    assert_eq!((next_absorb_id(), absorb_limit()), (1, 0));
  }
}

#[cfg(test)]
mod reentrancy_tests {
  use super::*;

  /// A package-load hoist moves a meaning to global scope without changing whether it is a fallback class's
  /// guess (`fallback_meanings`), whatever origin is current while it runs.
  #[test]
  fn package_load_hoist_keeps_fallback_standing() {
    use crate::definition::origin::{DefinitionOrigin, with_origin};
    let guess = T_CS!("\\p1a_hoisted_fallback_probe");
    let own = T_CS!("\\p1a_hoisted_own_probe");
    push_frame();
    let pre = snapshot_top_frame_keys();
    with_origin(DefinitionOrigin::Fallback, || {
      let_i(&guess, &T_LETTER!("x"), None)
    });
    let_i(&own, &T_LETTER!("y"), None);
    with_origin(DefinitionOrigin::Fallback, || {
      hoist_top_frame_package_load(&pre)
    });
    assert!(pop_frame().is_ok());
    assert!(
      lookup_meaning(&guess).is_some() && lookup_meaning(&own).is_some(),
      "both hoisted to global scope"
    );
    assert!(is_fallback_meaning(&guess), "a fallback guess stays one");
    assert!(
      !is_fallback_meaning(&own),
      "a hoist under a fallback origin makes nothing a guess"
    );
  }

  /// `with_let` binds for the body only and restores the prior meaning —
  /// including "no meaning" — even when the body fails.
  #[test]
  fn with_let_restores_the_prior_meaning() {
    let cs = T_CS!("\\p1a_with_let_probe");
    // A character token is its own meaning even in a bare State (no engine).
    let target = T_OTHER!("x");
    assert!(lookup_meaning(&cs).is_none());
    let seen: Result<bool> = with_let(&cs, &target, || Ok(lookup_meaning(&cs).is_some()));
    assert_eq!(seen.ok(), Some(true));
    assert!(lookup_meaning(&cs).is_none(), "restored to no meaning");
    let_i(&cs, &T_LETTER!("y"), None);
    let before = lookup_meaning(&cs);
    assert!(before.is_some());
    let failed: Result<()> = with_let(&cs, &target, || Err("boom".into()));
    assert!(failed.is_err());
    assert_eq!(lookup_meaning(&cs), before, "restored after a failing body");
  }

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

  /// `Scope::InPlace` (Perl `State.pm:175` 'inplace') is the same-level
  /// reassignment behind the Rhai `LookupDefinition(cs).push*` hook-splice
  /// (`install_definition(d, Some(Scope::InPlace))`). It must be neither Global
  /// nor Local across a group boundary — this is exactly the divergence
  /// @xworld21 flagged in PR #333 (r3623947537). Exercised on the Value table,
  /// which funnels through the identical `assign_internal` arm.
  #[test]
  fn inplace_scope_keeps_the_bindings_level() {
    // Scenario 1: a value bound ABOVE the group, mutated in-place from INSIDE
    // the group, PERSISTS past group exit (Local would have reverted it). This
    // is BookML's real case: patch an already-global def, mutation stays.
    assign_value("ip_above", Stored::Int(1), Some(Scope::Global));
    push_frame();
    assign_value("ip_above", Stored::Int(2), Some(Scope::InPlace));
    assert_eq!(
      lookup_int("ip_above"),
      2,
      "in-place mutation is active at once"
    );
    pop_frame().expect("pop group");
    assert_eq!(
      lookup_int("ip_above"),
      2,
      "in-place patch of an outer-bound value rode the outer binding past group \
       exit (Local would revert to 1)"
    );

    // Scenario 2: a value LOCALLY redefined in the group, then mutated in-place,
    // REVERTS to the outer value at group exit (Global would have kept the
    // patch). The in-place edit rode the LOCAL binding, which is discarded.
    assign_value("ip_local", Stored::Int(1), Some(Scope::Global));
    push_frame();
    assign_value("ip_local", Stored::Int(2), Some(Scope::Local));
    assign_value("ip_local", Stored::Int(3), Some(Scope::InPlace));
    assert_eq!(
      lookup_int("ip_local"),
      3,
      "in-place mutated the local front"
    );
    pop_frame().expect("pop group");
    assert_eq!(
      lookup_int("ip_local"),
      1,
      "in-place patch of a locally-bound value was discarded with the group \
       (Global would keep 3)"
    );
  }

  /// `is_scope_active` must track the FRONT value's truthiness, not key presence:
  /// `deactivate_scope` OVERWRITES the front value with `Stored::Bool(false)`
  /// instead of removing the entry, so a presence test reports a deactivated
  /// scope as still active.
  /// Perl writes the same test inline as `$$self{stash_active}{$scope}[0]`
  /// (State.pm L682). The region property `subfile_scope_at_depth` relies on
  /// (OXIDIZED_DESIGN #65) is the second assertion: an activation made inside a
  /// group is undone by that group, with no matching teardown call.
  #[test]
  fn scope_activity_tracks_value_not_presence() {
    let scope = arena::pin("t@scope@activity");
    assert!(!is_scope_active(scope), "unknown scope must be inactive");

    activate_scope(scope);
    assert!(is_scope_active(scope), "activated scope must read active");

    deactivate_scope(scope);
    assert!(
      !is_scope_active(scope),
      "a deactivated scope must read INACTIVE — `stash_active` still holds the \
       key, carrying Stored::Bool(false)"
    );
    // Deactivation is a GLOBAL assign, which replaces rather than layers: the
    // stack collapses to exactly that one falsy value. This is why the front
    // value is the whole state, and why presence cannot be the activity test.
    let depth = state!()
      .stash_active
      .get(&scope)
      .map(|entry| entry.len())
      .unwrap_or(0);
    assert_eq!(
      depth, 1,
      "global assign must overwrite, leaving one value — not stack a second"
    );

    // A second deactivation must be a silent no-op. Perl gates on `[0]` being
    // TRUE (State.pm L700) precisely so the binding-pop below it does not run
    // twice; re-running it pops values `activate_scope` never pushed, which is
    // what Perl's "Unassigning wrong value for KEY from table T in
    // deactivateScope" warning reports.
    deactivate_scope(scope);
    assert!(
      !is_scope_active(scope),
      "still inactive after a second deactivate"
    );
    let depth2 = state!()
      .stash_active
      .get(&scope)
      .map(|entry| entry.len())
      .unwrap_or(0);
    assert_eq!(
      depth2, 1,
      "a second deactivation must not stack another value"
    );
  }

  /// A DEACTIVATED scope must be activatable again — Perl gates `activateScope`
  /// on `!$$self{stash_active}{$scope}[0]` (State.pm L682), the front value's
  /// truthiness. Gating on key presence instead made the first deactivation
  /// permanent, since `deactivate_scope` leaves `Stored::Bool(false)` behind.
  /// Reached in Perl by the counter/label scopes, which deactivate the old
  /// reference number before activating the next (`Package.pm` L774-779).
  #[test]
  fn a_deactivated_scope_can_be_reactivated() {
    let scope = arena::pin("t@scope@reactivate");
    activate_scope(scope);
    deactivate_scope(scope);
    assert!(!is_scope_active(scope), "precondition: deactivated");

    activate_scope(scope);
    assert!(
      is_scope_active(scope),
      "re-activation after deactivation must take effect (Perl State.pm L682)"
    );
  }

  /// The self-terminating half: `activate_scope` marks `StashActive` with
  /// `Scope::Local`, so the enclosing group ends the region by construction.
  #[test]
  fn scope_activation_is_bounded_by_its_group() {
    let scope = arena::pin("t@scope@bounded");
    push_frame();
    activate_scope(scope);
    assert!(
      is_scope_active(scope),
      "active inside the group that opened it"
    );
    pop_frame().expect("pop group");
    assert!(
      !is_scope_active(scope),
      "the region must end with its group — no explicit deactivate_scope"
    );
  }
}

#[cfg(test)]
mod let_copy_tests {
  use super::*;

  fn macro_of(cs: &str, body: &str) -> Stored {
    Stored::Expandable(Rc::new(Expandable {
      cs: T_CS!(cs),
      paramlist: None,
      expansion: Tokens::new(Explode!(body)).into(),
      ..Expandable::default()
    }))
  }

  fn holds(cs: &str, def: &Stored) -> bool {
    matches!((lookup_meaning(&T_CS!(cs)), def),
      (Some(Stored::Expandable(a)), Stored::Expandable(b)) if Rc::ptr_eq(&a, b))
  }

  fn body_of(cs: &str) -> String {
    match lookup_meaning(&T_CS!(cs)) {
      Some(Stored::Expandable(def)) => match def.get_expansion() {
        Some(ExpansionBody::Tokens(body)) => body.to_string(),
        None => String::new(),
        Some(ExpansionBody::Closure(_)) => "<closure>".to_string(),
      },
      other => format!("{other:?}"),
    }
  }

  /// A `\let` copy still sharing the definition follows it to the late one — a copy a group
  /// shadows as well, once the group ends; a copy of an older definition keeps its own; the source
  /// itself is left to the caller; no copies, nothing moves. No clone of the placeholder is held,
  /// so its holders are exactly its bindings and the walk ends on the last copy found.
  #[test]
  fn let_copies_follow_a_late_definition() {
    reset_thread_state();
    let job = T_CS!("\\lcjob");
    let older = macro_of("\\lcjob", "older");
    install_definition(older.clone(), None);
    let_i(&T_CS!("\\lcstale"), &job, None);
    install_definition(macro_of("\\lcjob", ""), None);
    let_i(&T_CS!("\\lccopy"), &job, None);
    let_i(&T_CS!("\\lcshadowed"), &job, None);
    push_frame();
    let_i(&T_CS!("\\lcshadowed"), &T_CS!("\\lcstale"), None);

    let name = macro_of("\\lcjob", "name");
    assert_eq!(rebind_let_copies(&job, &name), 2);
    assert!(holds("\\lccopy", &name));
    assert!(
      holds("\\lcshadowed", &older),
      "the group's own binding is untouched"
    );
    pop_frame().expect("pop the probe frame");
    assert!(
      holds("\\lcshadowed", &name),
      "the shadowed copy was re-pointed in place"
    );
    assert!(
      holds("\\lcstale", &older),
      "a copy of an older definition keeps it"
    );
    assert_eq!(
      body_of("\\lcjob"),
      "",
      "the source is the caller's to install"
    );

    install_definition(macro_of("\\lcsolo", "x"), None);
    assert_eq!(
      rebind_let_copies(&T_CS!("\\lcsolo"), &macro_of("\\lcsolo", "y")),
      0
    );
  }
}

#[cfg(test)]
mod assignment_journal_tests {
  use super::*;

  /// A rollback puts back a global and a local assignment made since the journal began; a kept journal keeps them.
  #[test]
  fn rollback_undoes_and_keep_keeps() {
    assign_value("j65e_probe", "before", Some(Scope::Global));
    let journal = begin_assignment_journal().expect("no journal open");
    push_frame();
    assign_value("j65e_probe", "inner", None);
    assign_value("j65e_probe", "global", Some(Scope::Global));
    assert!(pop_frame().is_ok());
    assert_eq!(lookup_string("j65e_probe"), "global");
    assert!(journal.rollback(), "the frames it began with are open");
    assert_eq!(lookup_string("j65e_probe"), "before");
    let journal = begin_assignment_journal().expect("no journal open");
    assign_value("j65e_probe", "kept", Some(Scope::Global));
    journal.keep();
    assert_eq!(lookup_string("j65e_probe"), "kept");
  }

  /// A journal that began inside a frame closed since cannot roll back: the assignments stay.
  #[test]
  fn rollback_refused_when_its_frames_changed() {
    push_frame();
    let journal = begin_assignment_journal().expect("no journal open");
    assign_value("j65e_frames", "set", Some(Scope::Global));
    assert!(pop_frame().is_ok());
    assert!(!journal.rollback(), "a frame it began with is gone");
    assert_eq!(lookup_string("j65e_frames"), "set");
  }

  /// A guard dropped while the state is borrowed leaves its journal stale: nothing records into it, and the next begin
  /// discards it.
  #[test]
  fn stale_journal_records_nothing_and_is_discarded() {
    let journal = begin_assignment_journal().expect("no journal open");
    {
      let _held = STATE.borrow_mut();
      drop(journal);
    }
    assert!(state!().journal.is_some(), "the guard could not close it");
    assign_value("j65e_stale", "x", Some(Scope::Global));
    assert!(
      state!()
        .journal
        .as_ref()
        .is_some_and(|j| j.saved.is_empty()),
      "a stale journal records nothing"
    );
    let journal = begin_assignment_journal().expect("the stale journal is discarded");
    journal.keep();
    assert!(state!().journal.is_none());
  }
}
