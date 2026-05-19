//! Native Rust port of LaTeXML's `LaTeXML::Common::Model::RelaxNG`.
//!
//! Walks a RelaxNG XML schema, builds an in-memory pattern AST, simplifies
//! it (binding/grammar/include resolution, definition recording), and can
//! emit the LaTeX manual.tex consumed by `latexmlman.sty` for schema
//! documentation.
//!
//! Three sub-modules carry the implementation, mirroring the natural
//! sections of the upstream Perl source:
//!
//! * [`scan`]      — RNG XML → AST  (port of `scanPattern` etc., L100–390).
//! * [`simplify`]  — AST normalization (port of `simplify*`, L438–525).
//! * [`tex`]       — schema-doc TeX emission (port of `documentModules`,
//!   `toTeX*`, L550–815).
//!
//! The shared state — definition tables, element index, "Used by" graph —
//! lives on [`Relaxng`] and is populated by `scan` + `simplify`, then
//! consumed by `tex`. The Perl original mutates `$$self{...}` from
//! several methods at once; the Rust version threads `&mut self` the
//! same way.

use crate::common::model::LTX_NAMESPACE;
use crate::common::xml::XML_NS;
use crate::document::Document;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub mod scan;
pub mod simplify;
pub mod tex;

// ----- AST ----------------------------------------------------------------

/// Combiner kind on a `<define>` element.
///
/// Bare `<define>` is `Group`; `<define combine="choice">` is `Choice`;
/// `<define combine="interleave">` is `Interleave`. Mirrors the suffix on
/// upstream's `def`/`defchoice`/`definterleave` ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefCombiner {
  Group,
  Choice,
  Interleave,
}

/// Combiner kind for a `<group|interleave|choice|optional|zeroOrMore|
/// oneOrMore|list>` pattern wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombineOp {
  Group,
  Interleave,
  Choice,
  Optional,
  ZeroOrMore,
  OneOrMore,
  List,
}

/// One node in the RelaxNG AST, mirroring Perl `RelaxNG.pm`'s
/// `[$op, $name, @forms]` arrays.
///
/// The names here line up 1:1 with the Perl op strings:
///
/// | Perl op            | Rust variant       |
/// |--------------------|--------------------|
/// | `ref`              | [`Pattern::Ref`]   |
/// | `parentref`        | [`Pattern::ParentRef`] |
/// | `elementref`       | [`Pattern::ElementRef`] (added during simplify) |
/// | `def`/`defchoice`/`definterleave` | [`Pattern::Def`] (combiner discriminates) |
/// | `element`          | [`Pattern::Element`] |
/// | `attribute`        | [`Pattern::Attribute`] |
/// | `start`            | [`Pattern::Start`] |
/// | `value`            | [`Pattern::Value`] |
/// | `data`             | [`Pattern::Data`] |
/// | `doc`              | [`Pattern::Doc`] |
/// | `combination`      | [`Pattern::Combination`] |
/// | `grammar`          | [`Pattern::Grammar`] |
/// | `module`           | [`Pattern::Module`] |
/// | `override`         | [`Pattern::Override`] (consumed by simplify) |
/// | `'#PCDATA'` (string leaf) | [`Pattern::Text`] |
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
  /// Reference to a defined pattern. `qname` is the bare name during
  /// `scan` and `binding:name` after `simplify`.
  Ref { qname: String },
  /// Reference to a parent grammar's defined pattern (replaced by
  /// `Ref` during simplify).
  ParentRef { qname: String },
  /// Reference to an element by tag name (introduced during simplify
  /// when a `Def` resolves to a single `Element`).
  ElementRef { qname: String },
  /// `<define>` (or `combine="choice"|"interleave"`).
  Def {
    combiner: DefCombiner,
    name:     String,
    body:     Vec<Pattern>,
  },
  /// `<element name="...">CONTENT</element>`.
  Element { name: String, body: Vec<Pattern> },
  /// `<attribute name="...">CONTENT</attribute>`.
  Attribute { name: String, body: Vec<Pattern> },
  /// `<start>...</start>`.
  Start { body: Vec<Pattern> },
  /// `<value>X</value>` — a literal value (typically for attributes).
  Value(String),
  /// `<data type="X"/>` — a typed datum.
  Data(String),
  /// `<a:documentation>X</a:documentation>` — annotation text.
  Doc(String),
  /// `<group|interleave|choice|optional|zeroOrMore|oneOrMore|list>...</...>`.
  ///
  /// `<mixed>` is normalised here into `Combination { Interleave, [Text, …] }`.
  Combination { op: CombineOp, body: Vec<Pattern> },
  /// `<grammar>...</grammar>` — defines a fresh symbol scope. Replaced
  /// by its `start` pattern after simplify.
  Grammar { name: String, body: Vec<Pattern> },
  /// External / included module: contents from a separate schema file,
  /// recorded in [`Relaxng::modules`] for documentation.
  Module { name: String, body: Vec<Pattern> },
  /// `<include>...</include>` with override rules (consumed by simplify
  /// — patches the inner `Module` and disappears).
  Override {
    module:       Box<Pattern>,
    replacements: Vec<Pattern>,
  },
  /// `#PCDATA` — text leaf.
  Text,
}

// ----- Schema state -------------------------------------------------------

/// Internal representation of a RelaxNG schema. Built by [`scan`] and
/// [`simplify`]; consumed by [`tex`] (and, for runtime validation,
/// would be consumed by `Model::add_tag_content` etc.).
///
/// The mutable fields beyond `name` and `modules` are populated during
/// `simplify`:
///
/// * [`elementdefs`]      — pattern qname → element tag, when a pattern
///   resolves to a single element.
/// * [`element_reverse_defs`] — inverse of `elementdefs`.
/// * [`elements`]         — element tag → list of body patterns,
///   accumulating across overrides / re-definitions.
/// * [`defs`]             — pattern qname → its (combined) body pattern.
/// * [`def_combiner`]     — pattern qname → the combiner that won the
///   most recent definition.
/// * [`uses_name`]        — pattern qname → set of containers (pattern or
///   element ids) that reference it. Drives the "Used by" lists in the
///   schema docs.
/// * [`internal_grammars`] — counter for naming embedded `<grammar>`
///   blocks (`grammar1`, `grammar2`, …).
#[derive(Debug)]
pub struct Relaxng {
  /// Top-level schema name (typically the .rng filename without ext).
  pub name:    String,
  /// Modules in document-order. Populated by [`simplify`]; each entry is
  /// a `Pattern::Module` whose body is populated retroactively (the
  /// Perl push-then-extend pattern).
  pub modules: Vec<Pattern>,

  pub elementdefs:          HashMap<String, String>,
  pub element_reverse_defs: HashMap<String, String>,
  pub elements:             HashMap<String, Vec<Pattern>>,
  pub defs:                 HashMap<String, Pattern>,
  pub def_combiner:         HashMap<String, DefCombiner>,
  pub uses_name:            HashMap<String, HashSet<String>>,
  pub internal_grammars:    u32,

  /// Document-namespace prefix → URI, populated as the scanner sees
  /// `xmlns:` attributes on RelaxNG nodes.
  pub document_namespaces: HashMap<String, String>,

  /// The master grammar's `<grammar ns="…">` URI — populated by the
  /// first call to `scan_external` (i.e. the schema entry point).
  /// Subsequent included grammars don't overwrite it. Used by the
  /// schema-doc emitter to auto-register the corresponding namespace
  /// prefix for elision in display names.
  pub primary_namespace: Option<String>,

  /// Namespace prefixes whose `prefix:` part should be elided from
  /// rendered display names in the schema docs (`clean_tex_name`),
  /// since they're contextually obvious for the schema. Auto-populated
  /// from `primary_namespace` when the schema-doc emission starts —
  /// e.g. LaTeXML's `default namespace = "http://dlmf.nist.gov/LaTeXML"`
  /// (mapped to the `ltx` prefix) becomes a strip-prefix so display
  /// names read `para` rather than `ltx:para`.
  pub display_strip_prefixes: Vec<String>,
}

impl Default for Relaxng {
  fn default() -> Self {
    Relaxng {
      name:                 String::from("LaTeXML"),
      modules:              Vec::new(),
      elementdefs:          HashMap::default(),
      element_reverse_defs: HashMap::default(),
      elements:             HashMap::default(),
      defs:                 HashMap::default(),
      def_combiner:         HashMap::default(),
      uses_name:            HashMap::default(),
      internal_grammars:    0,
      document_namespaces:  HashMap::default(),
      primary_namespace:    None,
      display_strip_prefixes: Vec::new(),
    }
  }
}

impl Relaxng {
  /// Construct an empty schema state. Use [`Self::load_schema`] to
  /// populate from an RNG file.
  pub fn new(name: impl Into<String>) -> Self {
    Relaxng { name: name.into(), ..Self::default() }
  }

  /// Register a `prefix → URI` binding ahead of scanning. Mirrors
  /// `Model::register_namespace` for standalone callers (which don't
  /// have a live `Model` to consult). Callers that already populated
  /// the schema's `xmlns:` declarations dynamically don't need this;
  /// it's intended for namespaces that trang flattens away — the most
  /// common case is a `.rnc` whose `default namespace = "..."` carries
  /// no prefix, so the URI is preserved on `<grammar ns="..."/>` but
  /// no `xmlns:` survives. Later calls overwrite earlier ones.
  pub fn register_namespace(
    &mut self,
    prefix: impl Into<String>,
    uri: impl Into<String>,
  ) {
    self.document_namespaces.insert(prefix.into(), uri.into());
  }

  /// Register the prefixes that `Model::new_default()` ships with the
  /// LaTeXML schema (`xml`, `ltx`, `svg`, `xlink`, `m`, `xhtml`). The
  /// runtime `Model` resolves these via its own registry, so this is
  /// only needed for *standalone* tooling (the `genschema_oxide`
  /// binary, integration tests against `LaTeXML.rng`) where we don't
  /// have a Model object to consult. Returns `&mut self` for chaining.
  pub fn with_latexml_defaults(&mut self) -> &mut Self {
    self.register_namespace("xml", XML_NS);
    self.register_namespace("ltx", LTX_NAMESPACE);
    self.register_namespace("svg", "http://www.w3.org/2000/svg");
    self.register_namespace("xlink", "http://www.w3.org/1999/xlink");
    self.register_namespace("m", "http://www.w3.org/1998/Math/MathML");
    self.register_namespace("xhtml", "http://www.w3.org/1999/xhtml");
    self
  }

  /// Register a namespace prefix to elide from rendered display names
  /// in the schema docs. Idempotent — duplicates are dropped.
  pub fn register_display_prefix_strip(&mut self, prefix: impl Into<String>) {
    let prefix = prefix.into();
    if !self.display_strip_prefixes.contains(&prefix) {
      self.display_strip_prefixes.push(prefix);
    }
  }

  /// Look up `primary_namespace` in `document_namespaces` and, if it
  /// resolves to a non-empty prefix, register that prefix for elision
  /// from rendered display names. Call after scan + simplify, before
  /// `tex::document_modules`. No-op when there's no primary namespace
  /// or when its prefix is the default (empty) one.
  ///
  /// When two prefixes map to the same URI (rare — a schema author
  /// could declare `xmlns:foo="…"` and `xmlns:bar="…"` to the same
  /// namespace), the lexicographically smallest prefix wins. We sort
  /// the candidates explicitly so the chosen prefix is identical
  /// between builds: `document_namespaces` is a `FxHashMap`, whose
  /// iteration order isn't a stable contract.
  pub fn auto_strip_primary_namespace(&mut self) {
    let Some(uri) = self.primary_namespace.clone() else { return; };
    let mut candidates: Vec<&String> = self
      .document_namespaces
      .iter()
      .filter(|(p, u)| !p.is_empty() && u.as_str() == uri.as_str())
      .map(|(p, _)| p)
      .collect();
    candidates.sort();
    if let Some(p) = candidates.first().map(|s| (*s).clone()) {
      self.register_display_prefix_strip(p);
    }
  }

  /// Insert a `<?latexml RelaxNGSchema="..."?>` processing instruction
  /// on the given document.
  pub fn add_schema_declaration(&self, document: &mut Document) {
    let mut attributes = HashMap::default();
    attributes.insert(String::from("RelaxNGSchema"), self.name.clone());
    document
      .insert_pi("latexml", Some(attributes))
      .expect("should never fail");
  }

  /// Load + scan + simplify the schema named in `self.name` (or
  /// `name_override`). Searches `search_paths` for the .rng file. After
  /// success, the AST sits in [`Self::modules`] and the lookup tables
  /// are populated.
  pub fn load_schema(
    &mut self,
    name: &str,
    search_paths: &[&std::path::Path],
  ) -> Result<(), scan::ScanError> {
    let raw = scan::scan_external(self, name, None, search_paths)?;
    let _start = simplify::simplify_top(self, raw);
    Ok(())
  }
}
