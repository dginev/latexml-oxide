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
//! * [`tex`]       — schema-doc TeX emission (port of `documentModules`, `toTeX*`, L550–815).
//!
//! The shared state — definition tables, element index, "Used by" graph —
//! lives on [`Relaxng`] and is populated by `scan` + `simplify`, then
//! consumed by `tex`. The Perl original mutates `$$self{...}` from
//! several methods at once; the Rust version threads `&mut self` the
//! same way.

use std::collections::BTreeSet;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  common::{model::LTX_NAMESPACE, xml::XML_NS},
  document::Document,
};

pub mod embedded;
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
/// * [`elementdefs`](Relaxng::elementdefs) — pattern qname → element tag, when a pattern resolves
///   to a single element.
/// * [`element_reverse_defs`](Relaxng::element_reverse_defs) — inverse of `elementdefs`.
/// * [`elements`](Relaxng::elements) — element tag → list of body patterns, accumulating across
///   overrides / re-definitions.
/// * [`defs`](Relaxng::defs) — pattern qname → its (combined) body pattern.
/// * [`def_combiner`](Relaxng::def_combiner) — pattern qname → the combiner that won the most
///   recent definition.
/// * [`uses_name`](Relaxng::uses_name) — pattern qname → set of containers that reference it: `pattern:QNAME`
///   for refs at define scope, `element:TAG@pattern:HOST` for refs inside an `element TAG {…}`
///   hosted by define HOST (bare `element:TAG` when the element sits outside any define). Drives
///   the "Used by" lists in the schema docs; `tex::symbol_uses` reports the element or the host
///   pattern, whichever identifies the definition uniquely.
/// * [`internal_grammars`](Relaxng::internal_grammars) — counter for naming embedded `<grammar>`
///   blocks (`grammar1`,
///   `grammar2`, …).
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

  /// URI → code prefix, seeded from the model's `code_namespace_prefixes`
  /// before scanning (`Model::load_schema`). Lets a namespace referenced only
  /// as a default `ns=` (no `xmlns:` prefix in the schema) resolve to the
  /// conventional prefix the engine already registered — e.g.
  /// `http://dlmf.nist.gov/LaTeXML` → `ltx` from `base_schema` — instead of a
  /// synthetic `namespaceN`. Port of Perl `encodeQName` → `getNamespacePrefix`
  /// consulting `code_namespace_prefixes` (#652).
  pub code_namespace_prefixes: HashMap<String, String>,

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

  /// The simplified `<start>` patterns — the grammar's document root
  /// content. Captured by [`Self::load_schema`] so [`Self::compute_model_data`]
  /// can distil `#Document`'s allowed children (Perl `RelaxNG.pm`'s
  /// `extractContent('#Document', @schema)`, L71).
  pub start: Vec<Pattern>,
}

impl Default for Relaxng {
  fn default() -> Self {
    Relaxng {
      name:                    String::from("LaTeXML"),
      modules:                 Vec::new(),
      elementdefs:             HashMap::default(),
      element_reverse_defs:    HashMap::default(),
      elements:                HashMap::default(),
      defs:                    HashMap::default(),
      def_combiner:            HashMap::default(),
      uses_name:               HashMap::default(),
      internal_grammars:       0,
      document_namespaces:     HashMap::default(),
      code_namespace_prefixes: HashMap::default(),
      primary_namespace:       None,
      display_strip_prefixes:  Vec::new(),
      start:                   Vec::new(),
    }
  }
}

impl Relaxng {
  /// Construct an empty schema state. Use [`Self::load_schema`] to
  /// populate from an RNG file.
  pub fn new(name: impl Into<String>) -> Self {
    Relaxng {
      name: name.into(),
      ..Self::default()
    }
  }

  /// Register a `prefix → URI` binding ahead of scanning. Mirrors
  /// `Model::register_namespace` for standalone callers (which don't
  /// have a live `Model` to consult). Callers that already populated
  /// the schema's `xmlns:` declarations dynamically don't need this;
  /// it's intended for namespaces that trang flattens away — the most
  /// common case is a `.rnc` whose `default namespace = "..."` carries
  /// no prefix, so the URI is preserved on `<grammar ns="..."/>` but
  /// no `xmlns:` survives. Later calls overwrite earlier ones.
  pub fn register_namespace(&mut self, prefix: impl Into<String>, uri: impl Into<String>) {
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
    let Some(uri) = self.primary_namespace.clone() else {
      return;
    };
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
    self.start = simplify::simplify_top(self, raw);
    Ok(())
  }

  /// Distil the scanned+simplified schema into the tag/attribute/namespace/class
  /// tables the runtime `Model` consults (`canContain`/`canHaveAttribute`/
  /// `isInSchemaClass`). A faithful port of Perl `Common/Model/RelaxNG.pm`'s
  /// post-scan loop (L70-95) and `extractContent` (L100-128): for each element
  /// tag, walk its body to the set of child elements and attributes it allows;
  /// for `#Document`, the same over the grammar `<start>`; class-defining symbols
  /// (`grammarN:NAME.class`) become schema classes. This is the step the compiled
  /// `.model` path bakes ahead of time (`Model::load_compiled_schema_str`); a
  /// schema loaded from raw `.rng` at runtime (`RelaxNGSchema()`, no compiled
  /// `.model`) needs it computed here, or `tagprop` stays empty and every element
  /// is rejected as "not allowed" (dginev/latexml-oxide#652).
  pub fn compute_model_data(&self) -> ModelData {
    let mut tag_contents: Vec<(String, Vec<String>)> = Vec::new();
    let mut tag_attributes: Vec<(String, Vec<String>)> = Vec::new();
    let mut schema_classes: Vec<(String, Vec<String>)> = Vec::new();

    // `#Document`: the start's content (Perl L71). Attributes are irrelevant.
    let (doc_children, _) = self.extract_content(&self.start);
    tag_contents.push(("#Document".to_string(), doc_children.into_iter().collect()));

    // Each element tag → (allowed children, allowed attributes). `*:*` (the
    // any-name element) carries no internal structure (Perl L82-85).
    let mut tags: Vec<&String> = self.elements.keys().collect();
    tags.sort();
    for tag in tags {
      if tag == "*:*" {
        tag_contents.push((tag.clone(), vec!["*:*".to_string()]));
        continue;
      }
      let (children, attrs) = self.extract_content(&self.elements[tag]);
      tag_contents.push((tag.clone(), children.into_iter().collect()));
      tag_attributes.push((tag.clone(), attrs.into_iter().collect()));
    }

    // Schema classes: a `def` named `grammarN:NAME.class` (Perl L91-95).
    let mut defs: Vec<&String> = self.defs.keys().collect();
    defs.sort();
    for sym in defs {
      if let Some(name) = sym
        .strip_suffix(".class")
        .and_then(|s| s.split_once(':').map(|(_, n)| n))
        && let Some(def) = self.defs.get(sym)
      {
        let (members, _) = self.extract_content(std::slice::from_ref(def));
        schema_classes.push((name.to_string(), members.into_iter().collect()));
      }
    }

    ModelData {
      tag_contents,
      tag_attributes,
      schema_classes,
      namespaces: self
        .document_namespaces
        .iter()
        .map(|(p, u)| (p.clone(), u.clone()))
        .collect(),
    }
  }

  /// Walk a pattern body to the sets of (child elements, attributes) it permits,
  /// resolving `ref`s through `elementdefs` (single-element defs → a child) and
  /// `defs` (other defs → inline expansion). Port of Perl `extractContent`
  /// (`RelaxNG.pm` L100-128); the `seen` guard is Rust-only defence against a
  /// self-referential `def` (a recursive content model) looping forever.
  fn extract_content(&self, body: &[Pattern]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut children: BTreeSet<String> = BTreeSet::new();
    let mut attrs: BTreeSet<String> = BTreeSet::new();
    let mut seen: HashSet<String> = HashSet::default();
    let mut work: Vec<Pattern> = body.to_vec();
    while let Some(item) = work.pop() {
      match item {
        Pattern::Attribute { name, .. } => {
          attrs.insert(name);
        },
        Pattern::Element { name, .. } | Pattern::ElementRef { qname: name } => {
          children.insert(name);
        },
        Pattern::Combination { body, .. } | Pattern::Def { body, .. } | Pattern::Start { body } => {
          work.extend(body);
        },
        Pattern::Grammar { body, .. } | Pattern::Module { body, .. } => {
          work.extend(simplify::extract_start(&body));
        },
        Pattern::Ref { qname } | Pattern::ParentRef { qname } => {
          if let Some(el) = self.elementdefs.get(&qname) {
            children.insert(el.clone());
          } else if seen.insert(qname.clone())
            && let Some(expansion) = self.defs.get(&qname)
          {
            work.push(expansion.clone());
          }
        },
        Pattern::Value(_) | Pattern::Data(_) | Pattern::Text => {
          children.insert("#PCDATA".to_string());
        },
        Pattern::Doc(_) | Pattern::Override { .. } => {},
      }
    }
    (children, attrs)
  }
}

/// The tables [`Relaxng::compute_model_data`] distils out of a scanned schema,
/// ready to be pushed into the runtime `Model` (`add_tag_content` /
/// `add_tag_attribute` / `set_schema_class` / `register_document_namespace`).
#[derive(Debug, Default)]
pub struct ModelData {
  pub tag_contents:   Vec<(String, Vec<String>)>,
  pub tag_attributes: Vec<(String, Vec<String>)>,
  pub schema_classes: Vec<(String, Vec<String>)>,
  pub namespaces:     Vec<(String, String)>,
}

#[cfg(test)]
mod distill_tests {
  use super::*;
  use crate::common::relaxng::scan::scan_string;

  /// #652: a raw `.rng` scanned at runtime must yield the same tag→children /
  /// tag→attributes tables the compiled `.model` bakes ahead of time. Before the
  /// fix the runtime scan built the AST but never distilled `tagprop`, so every
  /// element was rejected ("<ltx:document> isn't allowed in <#Document>") and the
  /// output was empty.
  #[test]
  fn compute_model_data_distills_tagprop_from_scanned_schema() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <start><ref name="document"/></start>
        <define name="document">
          <element name="document"><zeroOrMore><ref name="para"/></zeroOrMore></element>
        </define>
        <define name="para">
          <element name="para"><attribute name="class"/><text/></element>
        </define>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let raw = scan_string(&mut rng, xml).expect("scan");
    rng.start = simplify::simplify_top(&mut rng, raw);
    let data = rng.compute_model_data();

    let content = |tag: &str| -> Vec<String> {
      data
        .tag_contents
        .iter()
        .find(|(t, _)| t == tag)
        .map(|(_, c)| c.clone())
        .unwrap_or_default()
    };
    let attrs = |tag: &str| -> Vec<String> {
      data
        .tag_attributes
        .iter()
        .find(|(t, _)| t == tag)
        .map(|(_, a)| a.clone())
        .unwrap_or_default()
    };

    assert_eq!(
      content("#Document"),
      vec!["document".to_string()],
      "the grammar <start> makes `document` the only allowed document root; got {:?}",
      data.tag_contents
    );
    assert_eq!(
      content("document"),
      vec!["para".to_string()],
      "document's content model allows para"
    );
    assert_eq!(
      content("para"),
      vec!["#PCDATA".to_string()],
      "para's <text/> becomes #PCDATA content"
    );
    assert_eq!(
      attrs("para"),
      vec!["class".to_string()],
      "para allows @class"
    );
  }
}
