//! Streaming split: partition a huge core XML **file** into per-page spill
//! files without ever building the whole-document DOM.
//!
//! The DOM `Split` processor ([`crate::split::Split`], port of Perl
//! `Post::Split`) needs the entire document parsed first — measured at ~16 GB
//! for a 614 MB core XML, and past what a 32 GB host can parse at all for the
//! 131 MB book witness's 2.68 GB core XML (OOM before Split, zero pages
//! written; laptop UAT 2026-07-31). This module is the
//! `STREAMING_POST_DESIGN_2026-07-06.md` §3 front-end: a
//! [`libxml::reader::TextReader`] pull-parse over the file, materializing one
//! *non-page* subtree at a time, assembling each page's XML as text and
//! spilling it the moment the page closes. Peak memory is the open ancestor
//! chain plus one content subtree.
//!
//! # Fidelity contract
//!
//! The spill files must re-parse into the same per-page DOMs the whole-DOM
//! pipeline (parse → `Split::process` → per-page spill) produces — the parity
//! gate is byte-equality of the final rendered pages across the two paths
//! (guard: `latexml_oxide/tests/118_streaming_split_parity.rs`). Every quirk
//! of `Split::process_pages` is replicated deliberately:
//!
//! * **Run adjacency.** A TOC (`<ltx:TOC><ltx:toclist class="ltx_toclist_X">`)
//!   is emitted per maximal run of *adjacent* page siblings; ANY intervening
//!   sibling node — whitespace text included — breaks a run, exactly as the
//!   `entries[0].node == removed[0]` check does. (`ltx:navigation` siblings do
//!   NOT break runs: the DOM path excises them before any page surgery.)
//! * **TOC suppression probe.** A run's TOC is suppressed iff an
//!   `ltx:TOC[@lists='toc']` (exact match — generated TOCs carry no `lists`)
//!   already occurs among the parent's descendants *at flush time*, i.e. in
//!   content preceding the run.
//! * **`inlist="toc"` propagation** is per *tree level* (all page children of
//!   one page, across different DOM parents), with the DOM path's substring
//!   semantics (`inlist.contains("toc")`). It needs lookahead, so the
//!   attribute is patched into already-written spill files afterwards
//!   (`Splitter::patch_inlist_toc`).
//! * **`new_document` template copies**: every page gets the `<?latexml …?>`
//!   PIs, the document's `ltx:resource` elements, the root's direct
//!   `ltx:date` children (only when the page has no direct `ltx:date`), the
//!   root `class` merged into its own — in exactly that order — then the
//!   saved `ltx:navigation` elements.
//! * **Inherited attributes** (`xml:lang`, `backgroundcolor`): nearest
//!   ancestor-or-self value, copied onto each page root.
//! * **Naming** ports `Split::{presort,prename,get_page_name}` including the
//!   `FOO{n}` unnamed-page counter and its name-level-then-descend ordering.
//! * The root page gets `xml:id="TEMPORARY_DOCUMENT_ID"` when it has no id
//!   (the Writer removes it later), and pre-order spill order is the DOM
//!   path's docs order (root first, each page before its descendants).
//!
//! # Wrapper descent
//!
//! A non-page subtree that *contains* page matches (e.g. a back-matter
//! wrapper holding `ltx:appendix` pages) or `ltx:navigation` elements cannot
//! be bulk-copied: it is expanded to an owned mini-document
//! ([`TextReader::expand_to_document`]) and the same split surgery runs on
//! that DOM — page matches are pre-collected on the intact subtree (the DOM
//! path evaluates its predicates before any surgery), nested pages are
//! unlinked and written, TOC elements are inserted in place, and the
//! remaining shell is serialized into the enclosing page. Detection is a
//! conservative substring probe (`Splitter::needs_dom_descent`) —
//! over-triggering costs one expand+copy, never correctness.
//!
//! # Out of scope (fail loud, not wrong)
//!
//! * Documents whose `ltx` namespace mapping cannot be resolved from the
//!   declaration stack error out with a pointer to
//!   `LATEXML_POST_STREAM_SPLIT=0` (the whole-DOM path).
//! * `ltx:resource` elements / `<?latexml?>` PIs first appearing *after* a
//!   page has been written cannot retroactively reach it; a `Warn` flags the
//!   (never observed in practice) case.

use std::{
  io::Write,
  path::{Path, PathBuf},
};

use libxml::{
  reader::{ReaderEvent, TextReader},
  tree::{Document, Namespace, Node, NodeType},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet};

use crate::{
  document::{LTX_NSURI, SplitArm, SplitCond, collect_split_pages, is_ltx, parse_split_union},
  split::SplitNaming,
};

/// One spilled page, in pre-order (index 0 is the root page).
pub struct StreamedSplitPage {
  /// Spill file holding the page's XML.
  pub path:        PathBuf,
  /// The page's destination pathname (`Split::get_page_name` result; the
  /// original `--dest` for the root page).
  pub destination: String,
}

/// Result of a successful streaming split.
pub struct StreamSplitOutcome {
  /// Pages in pre-order; `[0]` is the root page.
  pub pages:       Vec<StreamedSplitPage>,
  /// Bodies of every `<?latexml …?>` PI (for the ar5iv literal-intent sniff).
  pub latexml_pis: Vec<String>,
  /// Concatenated serializations of every `ltx:picture` subtree, ready for
  /// the driver's `extract_svg_fragments`.
  pub picture_xml: String,
}

/// Can this `--splitpath` union be evaluated by the streaming split? (The
/// `make_splitpaths` grammar parses; a custom hand-written XPath may not.)
/// The driver gates on this so a genuine mid-stream failure can be told apart
/// from "not applicable".
pub fn supports_union(union_xpath: &str) -> bool { parse_split_union(union_xpath).is_some() }

/// Split a core-XML file into per-page spill files, streaming.
///
/// Returns `Ok(None)` when the union selected no pages (the "\[not split\]"
/// case) — the caller should use the whole-DOM pipeline. `Err` means the
/// stream could not be processed faithfully (malformed XML, unresolvable
/// namespace shape, I/O failure); the caller decides whether to fall back.
pub fn stream_split(
  source_path: &str,
  union_xpath: &str,
  naming: SplitNaming,
  destination: Option<&str>,
  spill_dir: &Path,
) -> Result<Option<StreamSplitOutcome>, String> {
  let arms = parse_split_union(union_xpath)
    .ok_or_else(|| format!("split union not streamable: {union_xpath}"))?;
  // Leniency flags as in `XmlParser::default().parse_file` (recover + noerror
  // + nowarning), PLUS `huge`: without XML_PARSE_HUGE, libxml2's hard limits
  // corrupt a multi-GB parse long before any real malformation — measured on
  // the 131 MB witness's 2.68 GB core XML, the per-document dictionary cap
  // poisons the ID table from ~1.47 GB on (237,732 bogus "ID X already
  // defined" reports for ids that each occur exactly once) and the parse
  // dies outright at ~1.71 GB ("outer_xml failed mid-stream", same byte in
  // every run). `xmllint --stream` reproduces both; `--huge` clears both.
  const OPTIONS: i32 = 1 /* recover */ + 32 /* noerror */ + 64 /* nowarning */
    + 524_288 /* huge */;
  let reader = TextReader::from_file(source_path, OPTIONS)
    .map_err(|()| format!("cannot open '{source_path}' for streaming"))?;
  let mut splitter = Splitter::new(arms, naming, destination, spill_dir);
  splitter.run(reader)?;
  if splitter.metas.len() <= 1 {
    // No page matched: mirror the DOM path's `[not split]` outcome and let
    // the caller run the ordinary pipeline (the root spill alone would be a
    // needless re-serialization of the whole document).
    Info!("split", "result", "[not split]");
    for meta in &splitter.metas {
      let _ = std::fs::remove_file(&meta.file);
    }
    return Ok(None);
  }
  splitter.prename();
  splitter.patch_inlist_toc()?;
  let n = splitter.metas.len();
  Info!("split", "result", " [Split into {} pages]", n);
  let Splitter {
    metas,
    latexml_pis,
    picture_xml,
    ..
  } = splitter;
  let pages = metas
    .into_iter()
    .map(|m| StreamedSplitPage {
      path:        m.file,
      destination: m.name,
    })
    .collect();
  Ok(Some(StreamSplitOutcome {
    pages,
    latexml_pis,
    picture_xml,
  }))
}

/// Metadata for one page, mirroring `Split`'s `PageEntry` plus what the
/// deferred naming/patching passes need.
struct PageMeta {
  /// Spill file (named by pre-order index at creation).
  file:           PathBuf,
  /// Tree children (pages whose nearest enclosing page is this one), in
  /// document order.
  children:       Vec<usize>,
  localname:      String,
  xml_id:         Option<String>,
  labels:         Option<String>,
  inlist:         Option<String>,
  /// The root `class` was *appended* as a new attribute (rather than merged
  /// into an existing one) — the `inlist="toc"` patch must insert *before*
  /// it to reproduce the DOM path's attribute order (inherit → inlist →
  /// class).
  class_appended: bool,
  /// Destination pathname; filled by [`Splitter::prename`].
  name:           String,
}

/// One open level of the stream: the root document element or an open page.
/// (Wrappers never open a reader level — they are handled wholesale by the
/// DOM descent.)
struct Level {
  kind:            LevelKind,
  localname:       String,
  ltx:             bool,
  /// Serialized children accumulated so far (after the open tag).
  content:         String,
  /// `ltx:` element-children localnames seen so far — the streaming
  /// evaluation of the `preceding-sibling::ltx:NAME` split predicate.
  seen_ltx:        FxHashSet<String>,
  /// The `xml:id`s of the current adjacent page run (a tocentry each).
  run_toc:         Vec<String>,
  run_active:      bool,
  /// An `ltx:TOC[@lists='toc']` occurs among this element's descendants
  /// streamed so far (the TOC-suppression probe).
  has_lists_toc:   bool,
  /// A direct `ltx:date` child was appended (suppresses the date copy).
  has_direct_date: bool,
  /// `(qname, value)` attributes from the source, document order.
  attrs:           Vec<(String, String)>,
  /// Effective inherited `xml:lang` / `backgroundcolor` (self-or-ancestor).
  lang:            Option<String>,
  bg:              Option<String>,
  /// prefix → namespace-URI declarations introduced ON this element
  /// (`""` = default).
  ns_decls:        Vec<(String, String)>,
}

enum LevelKind {
  Root,
  /// Index into [`Splitter::metas`].
  Page(usize),
}

struct Splitter {
  arms:               Vec<SplitArm>,
  naming:             SplitNaming,
  root_destination:   String,
  spill_dir:          PathBuf,
  levels:             Vec<Level>,
  metas:              Vec<PageMeta>,
  /// The root spill's prolog: pre-root PIs/comments, reconstructed verbatim.
  root_prolog:        String,
  /// Root page trailing misc (post-root comments/PIs).
  root_tail:          String,
  /// `<?latexml …?>` bodies, document order (ar5iv sniff + page templates).
  latexml_pis:        Vec<String>,
  /// Serialized `ltx:resource` elements (page template).
  resources_xml:      Vec<String>,
  /// Serialized root-direct `ltx:date` elements (page template).
  dates_xml:          Vec<String>,
  /// Serialized `ltx:navigation` elements, excised from content.
  navs_xml:           Vec<String>,
  /// Root element `class` attribute (merged into every page).
  root_class:         Option<String>,
  /// Concatenated `ltx:picture` serializations for SVG extraction.
  picture_xml:        String,
  first_page_spilled: bool,
  warned_late:        bool,
  unnamed_counter:    u32,
}

impl Splitter {
  fn new(
    arms: Vec<SplitArm>,
    naming: SplitNaming,
    destination: Option<&str>,
    spill_dir: &Path,
  ) -> Self {
    Splitter {
      arms,
      naming,
      root_destination: destination.unwrap_or("").to_string(),
      spill_dir: spill_dir.to_path_buf(),
      levels: Vec::new(),
      metas: Vec::new(),
      root_prolog: String::new(),
      root_tail: String::new(),
      latexml_pis: Vec::new(),
      resources_xml: Vec::new(),
      dates_xml: Vec::new(),
      navs_xml: Vec::new(),
      root_class: None,
      picture_xml: String::new(),
      first_page_spilled: false,
      warned_late: false,
      unnamed_counter: 0,
    }
  }

  // ====================================================================
  // Reader-level pump

  fn run(&mut self, mut reader: TextReader) -> Result<(), String> {
    let mut advanced = reader.read().map_err(|()| "XML parse error".to_string())?;
    while advanced {
      match reader.event() {
        ReaderEvent::Element => {
          let localname = reader.local_name().unwrap_or_default();
          let ns = reader.namespace_uri();
          let ltx = ns.as_deref() == Some(LTX_NSURI);
          let empty = reader.is_empty_element();
          if self.levels.is_empty() {
            if !self.metas.is_empty() {
              // A second top-level element (recover-mode oddity): opening a
              // "root" again would clobber the root spill's slot. Fail loud.
              return Err("multiple root elements in stream".to_string());
            }
            // The root element (a root matching a split arm is NOT a page:
            // the DOM path filters pages to those with a grandparent).
            let attrs = reader.attributes_qname();
            self.open_root(localname, ltx, attrs);
            if empty {
              self.close_top()?;
            }
          } else if ltx && self.is_page_here(&localname) {
            let attrs = reader.attributes_qname();
            self.open_page(localname, attrs);
            if empty {
              self.close_top()?;
            }
          } else {
            self.top().seen_ltx_insert(ltx, &localname);
            let outer = reader
              .outer_xml()
              .ok_or_else(|| "outer_xml failed mid-stream".to_string())?;
            if ltx && localname == "navigation" {
              // Excised BEFORE page surgery in the DOM path — deliberately
              // does not break a page run.
              self.navs_xml.push(outer);
            } else if self.needs_dom_descent(&outer) {
              let mut minidoc = reader
                .expand_to_document()
                .ok_or_else(|| "expand_to_document failed mid-stream".to_string())?;
              self.descend_wrapper(&mut minidoc)?;
            } else {
              self.append_bulk(&outer, &localname, ltx);
            }
            // Skip the subtree; the reader is then positioned on the next
            // event, so bypass the trailing read().
            advanced = reader
              .read_next()
              .map_err(|()| "XML parse error".to_string())?;
            continue;
          }
        },
        ReaderEvent::EndElement => {
          self.close_top()?;
        },
        ReaderEvent::Text
        | ReaderEvent::SignificantWhitespace
        | ReaderEvent::Whitespace
        | ReaderEvent::CData => {
          // CDATA is normalized to an escaped text node (identical parsed
          // content; core XML carries no CDATA).
          if !self.levels.is_empty() {
            let text = reader.value().unwrap_or_default();
            self.flush_run();
            self.top().content.push_str(&text_escape(&text));
          }
          // Pre/post-root whitespace is layout-only; dropped.
        },
        ReaderEvent::Comment => {
          let text = reader.value().unwrap_or_default();
          let serialized = format!("<!--{text}-->");
          self.append_misc(serialized);
        },
        ReaderEvent::ProcessingInstruction => {
          let target = reader.local_name().unwrap_or_default();
          let body = reader.value().unwrap_or_default();
          let serialized = if body.is_empty() {
            format!("<?{target}?>")
          } else {
            format!("<?{target} {body}?>")
          };
          if target == "latexml" {
            self.template_pi(body);
          }
          self.append_misc(serialized);
        },
        ReaderEvent::EntityReference => {
          return Err("unexpected unresolved entity reference in stream".to_string());
        },
        _ => {},
      }
      advanced = reader.read().map_err(|()| "XML parse error".to_string())?;
    }
    if !self.levels.is_empty() {
      return Err("premature end of input (unclosed elements)".to_string());
    }
    if self.metas.is_empty() {
      return Err("no root element found".to_string());
    }
    Ok(())
  }

  /// A comment/PI: into the current level's content, the pre-root prolog, or
  /// the post-root tail.
  fn append_misc(&mut self, serialized: String) {
    if self.levels.is_empty() {
      if self.metas.is_empty() {
        self.root_prolog.push_str(&serialized);
        self.root_prolog.push('\n');
      } else {
        self.root_tail.push_str(&serialized);
        self.root_tail.push('\n');
      }
    } else {
      self.flush_run();
      self.top().content.push_str(&serialized);
    }
  }

  /// Record a `<?latexml …?>` PI body for the page template + ar5iv sniff,
  /// warning once if it arrives after a page has already been written.
  fn template_pi(&mut self, body: String) {
    self.warn_if_late("a <?latexml?> PI");
    self.latexml_pis.push(body);
  }

  fn warn_if_late(&mut self, what: &str) {
    if self.first_page_spilled && !self.warned_late {
      self.warned_late = true;
      Warn!(
        "split",
        "stream",
        "{} appeared after the first page was written; already-spilled pages do not carry it",
        what
      );
    }
  }

  // ====================================================================
  // Level operations

  fn top(&mut self) -> &mut Level {
    self
      .levels
      .last_mut()
      .expect("level stack must be non-empty")
  }

  /// Streaming evaluation of the split union for an `ltx:` element opening
  /// as a direct child of the current top level. Sibling/parent state is the
  /// *intact* document's (extracted pages remain in `seen_ltx`), matching
  /// the DOM path's evaluate-before-surgery order.
  fn is_page_here(&self, localname: &str) -> bool {
    let parent = self.levels.last().expect("checked non-empty");
    self.arms.iter().any(|arm| {
      arm.element == localname
        && (arm.any_of.is_empty()
          || arm.any_of.iter().any(|cond| match cond {
            SplitCond::PrecedingSibling(name) => parent.seen_ltx.contains(name),
            SplitCond::Parent(name) => parent.ltx && parent.localname == *name,
          }))
    })
  }

  fn open_root(&mut self, localname: String, ltx: bool, attrs: Vec<(String, String)>) {
    self.root_class = attr_value(&attrs, "class");
    let lang = attr_value(&attrs, "xml:lang");
    let bg = attr_value(&attrs, "backgroundcolor");
    let ns_decls = decl_attrs(&attrs);
    self.metas.push(PageMeta {
      file:           self.spill_dir.join("page-0000000.xml"),
      children:       Vec::new(),
      localname:      localname.clone(),
      xml_id:         attr_value(&attrs, "xml:id"),
      labels:         attr_value(&attrs, "labels"),
      inlist:         attr_value(&attrs, "inlist"),
      class_appended: false,
      name:           self.root_destination.clone(),
    });
    self.levels.push(Level {
      kind: LevelKind::Root,
      localname,
      ltx,
      content: String::new(),
      seen_ltx: FxHashSet::default(),
      run_toc: Vec::new(),
      run_active: false,
      has_lists_toc: false,
      has_direct_date: false,
      attrs,
      lang,
      bg,
      ns_decls,
    });
  }

  fn open_page(&mut self, localname: String, attrs: Vec<(String, String)>) {
    let parent_level = self.levels.last().expect("page under an open level");
    let parent_meta = match parent_level.kind {
      LevelKind::Root => 0,
      LevelKind::Page(i) => i,
    };
    let lang = attr_value(&attrs, "xml:lang").or_else(|| parent_level.lang.clone());
    let bg = attr_value(&attrs, "backgroundcolor").or_else(|| parent_level.bg.clone());
    let idx = self.metas.len();
    let xml_id = attr_value(&attrs, "xml:id");
    if let Some(id) = &xml_id {
      let id = id.clone();
      self.top().run_toc.push(id);
    }
    self.top().run_active = true;
    self.top().seen_ltx_insert(true, &localname);
    self.metas.push(PageMeta {
      file: self.spill_dir.join(format!("page-{idx:07}.xml")),
      children: Vec::new(),
      localname: localname.clone(),
      xml_id,
      labels: attr_value(&attrs, "labels"),
      inlist: attr_value(&attrs, "inlist"),
      class_appended: false,
      name: String::new(),
    });
    self.metas[parent_meta].children.push(idx);
    let ns_decls = decl_attrs(&attrs);
    self.levels.push(Level {
      kind: LevelKind::Page(idx),
      localname,
      ltx: true,
      content: String::new(),
      seen_ltx: FxHashSet::default(),
      run_toc: Vec::new(),
      run_active: false,
      has_lists_toc: false,
      has_direct_date: false,
      attrs,
      lang,
      bg,
      ns_decls,
    });
  }

  /// Close the current level: flush its trailing run and write its spill.
  fn close_top(&mut self) -> Result<(), String> {
    self.flush_run();
    let level = self.levels.pop().expect("close without an open level");
    match level.kind {
      LevelKind::Root => self.write_root_spill(level),
      LevelKind::Page(idx) => self.write_page_spill(level, idx),
    }
  }

  /// A non-page, non-wrapper subtree: append its serialization to the
  /// current level's content, with the shared bookkeeping.
  fn append_bulk(&mut self, outer: &str, localname: &str, ltx: bool) {
    self.flush_run();
    if ltx && localname == "date" {
      if matches!(self.levels.last().map(|l| &l.kind), Some(LevelKind::Root)) {
        self.warn_if_late("an ltx:date");
        self.dates_xml.push(outer.to_string());
      }
      self.top().has_direct_date = true;
    }
    let is_resource = ltx && localname == "resource";
    if is_resource {
      self.warn_if_late("an ltx:resource");
      self.resources_xml.push(outer.to_string());
    }
    self.bulk_probes(outer, is_resource);
    self.top().content.push_str(outer);
  }

  /// Content probes shared by every bulk append: the TOC-suppression flag,
  /// picture collection for SVG extraction, embedded `<?latexml?>` PI
  /// bodies, and the nested-resource warning.
  fn bulk_probes(&mut self, outer: &str, expected_resource: bool) {
    if probe_lists_toc(outer) {
      self.top().has_lists_toc = true;
    }
    if outer.contains("<picture") || outer.contains(":picture") {
      collect_pictures(outer, &mut self.picture_xml);
    }
    // `expected_resource`: the subtree IS a direct-child ltx:resource that
    // append_bulk already collected — its own serialization must not trip the
    // nested-resource flag.
    if !expected_resource
      && (outer.contains("<resource") || outer.contains(":resource"))
      && !self.warned_late
    {
      self.warned_late = true;
      Warn!(
        "split",
        "stream",
        "an ltx:resource nested inside content is not propagated to page templates by the streaming split"
      );
    }
    if outer.contains("<?latexml") {
      for body in extract_pi_bodies(outer) {
        self.latexml_pis.push(body);
      }
    }
  }

  /// Flush the current adjacent-page run: emit its TOC (unless the
  /// suppression probe fired) at the current content position.
  fn flush_run(&mut self) {
    let level = self.top();
    if !level.run_active {
      return;
    }
    level.run_active = false;
    if level.run_toc.is_empty() {
      return;
    }
    let entries = std::mem::take(&mut level.run_toc);
    if level.has_lists_toc {
      return;
    }
    let parent_type = level.localname.clone();
    let toc = self.toc_xml(&parent_type, &entries);
    self.top().content.push_str(&toc);
  }

  fn toc_xml(&self, parent_type: &str, ids: &[String]) -> String {
    let te = self.ltx_qname("tocentry");
    let re = self.ltx_qname("ref");
    let entries: String = ids
      .iter()
      .map(|id| {
        format!(
          "<{te}><{re} idref=\"{id}\" show=\"toctitle\"/></{te}>",
          id = attr_escape(id)
        )
      })
      .collect();
    format!(
      "<{toc}><{list} class=\"ltx_toclist_{ptype}\">{entries}</{list}></{toc}>",
      toc = self.ltx_qname("TOC"),
      list = self.ltx_qname("toclist"),
      ptype = attr_escape(parent_type),
    )
  }

  // ====================================================================
  // Spill assembly

  /// The serialized qname for an `ltx:` element in this document's
  /// vocabulary (empty prefix when ltx is the default namespace — the
  /// standard core-XML shape).
  fn ltx_qname(&self, localname: &str) -> String {
    for level in &self.levels {
      for (prefix, uri) in &level.ns_decls {
        if uri == LTX_NSURI {
          return if prefix.is_empty() {
            localname.to_string()
          } else {
            format!("{prefix}:{localname}")
          };
        }
      }
    }
    localname.to_string()
  }

  /// The qname for a level's own element, resolved against its declarations
  /// plus the enclosing stack.
  fn qname_for(&self, level: &Level) -> String {
    if level.ltx {
      for (p, u) in decl_attrs(&level.attrs) {
        if u == LTX_NSURI {
          return if p.is_empty() {
            level.localname.clone()
          } else {
            format!("{}:{}", p, level.localname)
          };
        }
      }
      self.ltx_qname(&level.localname)
    } else {
      // Non-ltx roots keep their serialized shape via their own attrs; pages
      // are always ltx (the split union is ltx-only).
      level.localname.clone()
    }
  }

  fn write_page_spill(&mut self, level: Level, idx: usize) -> Result<(), String> {
    let mut attrs = level.attrs.clone();
    // Inherited attributes: nearest ancestor-or-self, appended when absent
    // (the DOM path's `set_attribute` appends), xml:lang before
    // backgroundcolor.
    if attr_value(&attrs, "xml:lang").is_none()
      && let Some(lang) = &level.lang
    {
      attrs.push(("xml:lang".to_string(), lang.clone()));
    }
    if attr_value(&attrs, "backgroundcolor").is_none()
      && let Some(bg) = &level.bg
    {
      attrs.push(("backgroundcolor".to_string(), bg.clone()));
    }
    // Root class merge (Perl Post.pm L779-782 via `new_document`).
    let mut class_appended = false;
    if let Some(pclass) = &self.root_class {
      match attrs.iter_mut().find(|(k, _)| k == "class") {
        Some((_, existing)) if !existing.is_empty() => {
          existing.push(' ');
          existing.push_str(pclass);
        },
        Some((_, existing)) => *existing = pclass.clone(),
        None => {
          attrs.push(("class".to_string(), pclass.clone()));
          class_appended = true;
        },
      }
    }
    self.metas[idx].class_appended = class_appended;
    // The standalone page file must re-declare every namespace its content
    // may reference: the enclosing declarations (root + open ancestors) the
    // element does not redeclare itself.
    let mut decls: Vec<(String, String)> = Vec::new();
    for lvl in &self.levels {
      for (p, u) in &lvl.ns_decls {
        if !decls.iter().any(|(dp, _)| dp == p) {
          decls.push((p.clone(), u.clone()));
        }
      }
    }
    let own_decls = decl_attrs(&level.attrs);
    decls.retain(|(p, _)| !own_decls.iter().any(|(op, _)| op == p));
    let qname = self.qname_for(&level);
    let mut content = level.content;
    // `new_document` template: resources, then dates (when the page has no
    // direct `ltx:date` child), then the saved navigation. Class was merged
    // above; PIs go in the prolog.
    for r in &self.resources_xml {
      content.push_str(r);
    }
    if !level.has_direct_date {
      for d in &self.dates_xml {
        content.push_str(d);
      }
    }
    for nav in &self.navs_xml {
      content.push_str(nav);
    }
    let mut tag = String::with_capacity(qname.len() + 64);
    tag.push('<');
    tag.push_str(&qname);
    for (p, u) in &decls {
      if p.is_empty() {
        tag.push_str(&format!(" xmlns=\"{}\"", attr_escape(u)));
      } else {
        tag.push_str(&format!(" xmlns:{}=\"{}\"", p, attr_escape(u)));
      }
    }
    for (k, v) in &attrs {
      tag.push_str(&format!(" {}=\"{}\"", k, attr_escape(v)));
    }
    let out = assemble_spill(&self.spill_prolog(), &tag, &qname, &content, "");
    write_spill(&self.metas[idx].file, &out)?;
    self.first_page_spilled = true;
    Ok(())
  }

  fn write_root_spill(&mut self, level: Level) -> Result<(), String> {
    let mut attrs = level.attrs.clone();
    // `Split::process`: the root gets a placeholder id (the Writer removes
    // it from the final output).
    if attr_value(&attrs, "xml:id").is_none() {
      attrs.push(("xml:id".to_string(), "TEMPORARY_DOCUMENT_ID".to_string()));
      self.metas[0].xml_id = Some("TEMPORARY_DOCUMENT_ID".to_string());
    }
    let qname = self.qname_for(&level);
    let mut content = level.content;
    for nav in &self.navs_xml {
      content.push_str(nav);
    }
    let mut tag = String::with_capacity(128);
    tag.push('<');
    tag.push_str(&qname);
    for (k, v) in &attrs {
      tag.push_str(&format!(" {}=\"{}\"", k, attr_escape(v)));
    }
    let prolog = format!(
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}",
      self.root_prolog
    );
    let out = assemble_spill(&prolog, &tag, &qname, &content, &self.root_tail);
    write_spill(&self.metas[0].file, &out)
  }

  /// The prolog every non-root page file starts with: XML declaration plus
  /// the `<?latexml …?>` template PIs.
  fn spill_prolog(&self) -> String {
    let mut prolog = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    for body in &self.latexml_pis {
      prolog.push_str(&format!("<?latexml {body}?>\n"));
    }
    prolog
  }

  // ====================================================================
  // DOM descent (wrapper subtrees)

  /// Conservative probe: does this serialized subtree contain anything that
  /// requires real DOM surgery (a potential page match, or an
  /// `ltx:navigation` to excise)? Over-triggering costs one expand+copy, not
  /// correctness.
  fn needs_dom_descent(&self, outer: &str) -> bool {
    if contains_element_probe(outer, "navigation") {
      return true;
    }
    self
      .arms
      .iter()
      .any(|arm| contains_element_probe(outer, &arm.element))
  }

  /// Run the split surgery on an owned mini-document (a wrapper subtree):
  /// excise `ltx:navigation`, pre-collect page matches on the intact tree
  /// (the DOM path evaluates its predicates before any surgery), extract
  /// them (writing their spills, TOCs inserted in place), then serialize the
  /// remaining shell into the current level's content.
  fn descend_wrapper(&mut self, minidoc: &mut Document) -> Result<(), String> {
    let root = minidoc
      .get_root_element()
      .ok_or_else(|| "wrapper mini-document has no root".to_string())?;
    // Navigation excision first, exactly as `Split::process` removes
    // `descendant::ltx:navigation` before page surgery.
    let mut navs: Vec<Node> = Vec::new();
    collect_ltx_descendants(&root, "navigation", &mut navs);
    for nav in &mut navs {
      let s = minidoc.node_to_string(nav);
      nav.unlink_node();
      self.navs_xml.push(s);
    }
    // Pre-collect page matches on the intact (nav-free) tree.
    let mut page_nodes: Vec<Node> = Vec::new();
    collect_split_pages(&root, &self.arms, &mut page_nodes);
    let parent_level = self.levels.last().expect("wrapper under an open level");
    let (lang, bg) = (parent_level.lang.clone(), parent_level.bg.clone());
    self.descend_element(minidoc, &root, &page_nodes, lang, bg)?;
    // The remaining shell (with TOCs in place of extracted runs) is ordinary
    // content of the current level. Do NOT re-run the picture/PI probes —
    // the per-child descent already collected them; only the enclosing
    // TOC-suppression flag needs the shell.
    let shell = minidoc.node_to_string(&root);
    self.flush_run();
    if probe_lists_toc(&shell) {
      self.top().has_lists_toc = true;
    }
    self.top().content.push_str(&shell);
    Ok(())
  }

  /// The DOM flavor of the reader pump: iterate `node`'s children, extract
  /// page children (and their descendants), insert run TOCs in place.
  /// `lang`/`bg` are the effective inherited attributes *above* `node`.
  fn descend_element(
    &mut self,
    doc: &Document,
    node: &Node,
    page_nodes: &[Node],
    lang: Option<String>,
    bg: Option<String>,
  ) -> Result<(), String> {
    let lang = xml_ns_attr(node, "lang").or(lang);
    let bg = node.get_attribute("backgroundcolor").or(bg);
    let mut run_toc: Vec<String> = Vec::new();
    let mut run_active = false;
    // Incremental suppression probe: only content BEFORE a run counts, as in
    // the DOM path (following siblings are unlinked before its probe runs).
    let mut has_lists_toc = false;
    let node_name = node.get_name();
    let mut child_opt = node.get_first_child();
    while let Some(child) = child_opt {
      let next = child.get_next_sibling();
      let is_page_child =
        child.get_type() == Some(NodeType::ElementNode) && page_nodes.contains(&child);
      if is_page_child {
        run_active = true;
        self.dom_extract_page(
          doc,
          &child,
          page_nodes,
          lang.clone(),
          bg.clone(),
          &mut run_toc,
        )?;
        let mut extracted = child;
        extracted.unlink_node();
      } else {
        // Any other sibling breaks the run: flush its TOC *before* this
        // child.
        if run_active {
          run_active = false;
          if !run_toc.is_empty() && !has_lists_toc {
            let entries = std::mem::take(&mut run_toc);
            self.dom_insert_toc(doc, node, Some(&child), &node_name, &entries)?;
          }
          run_toc.clear();
        }
        if child.get_type() == Some(NodeType::ElementNode) {
          let serialized = doc.node_to_string(&child);
          if self.needs_dom_descent(&serialized) {
            self.descend_element(doc, &child, page_nodes, lang.clone(), bg.clone())?;
          } else {
            self.bulk_probes_dom(&serialized);
          }
          if has_lists_toc_node(&child) || has_lists_toc_descendant(&child) {
            has_lists_toc = true;
          }
        }
      }
      child_opt = next;
    }
    if run_active && !run_toc.is_empty() && !has_lists_toc {
      let entries = std::mem::take(&mut run_toc);
      self.dom_insert_toc(doc, node, None, &node_name, &entries)?;
    }
    Ok(())
  }

  /// The picture/PI probes for DOM-descent bulk content (`has_lists_toc` is
  /// tracked by the caller's incremental probe, and the shell append handles
  /// the enclosing level's flag).
  fn bulk_probes_dom(&mut self, serialized: &str) {
    if serialized.contains("<picture") || serialized.contains(":picture") {
      collect_pictures(serialized, &mut self.picture_xml);
    }
    if serialized.contains("<?latexml") {
      for body in extract_pi_bodies(serialized) {
        self.latexml_pis.push(body);
      }
    }
  }

  /// Extract one page found during DOM descent: register its metadata, add
  /// its tocentry to the enclosing run, recursively extract ITS page
  /// descendants, then serialize + amend + write its spill.
  fn dom_extract_page(
    &mut self,
    doc: &Document,
    page: &Node,
    page_nodes: &[Node],
    lang: Option<String>,
    bg: Option<String>,
    run_toc: &mut Vec<String>,
  ) -> Result<(), String> {
    let parent_meta = match self.levels.last().expect("open level").kind {
      LevelKind::Root => 0,
      LevelKind::Page(i) => i,
    };
    let idx = self.metas.len();
    let xml_id = crate::document::get_xml_id(page);
    if let Some(id) = &xml_id {
      run_toc.push(id.clone());
    }
    self.metas.push(PageMeta {
      file: self.spill_dir.join(format!("page-{idx:07}.xml")),
      children: Vec::new(),
      localname: page.get_name(),
      xml_id,
      labels: page.get_attribute("labels"),
      inlist: page.get_attribute("inlist"),
      class_appended: false,
      name: String::new(),
    });
    self.metas[parent_meta].children.push(idx);
    // Bookkeeping level so deeper extractions attach to this page in the
    // metadata tree (content stays in the live mini-DOM).
    self.levels.push(Level {
      kind:            LevelKind::Page(idx),
      localname:       page.get_name(),
      ltx:             true,
      content:         String::new(),
      seen_ltx:        FxHashSet::default(),
      run_toc:         Vec::new(),
      run_active:      false,
      has_lists_toc:   false,
      has_direct_date: false,
      attrs:           Vec::new(),
      lang:            lang.clone(),
      bg:              bg.clone(),
      ns_decls:        Vec::new(),
    });
    let descend_result = self.descend_element(doc, page, page_nodes, lang.clone(), bg.clone());
    self.levels.pop();
    descend_result?;
    // Serialize the (now child-page-free) page subtree; the serialization
    // preserves the original attribute order natively. Then amend the tag
    // with the inherited/class attributes and namespace re-declarations, and
    // splice the template trailer before the close tag.
    let mut serialized = doc.node_to_string(page);
    let has_direct_date = page
      .get_child_elements()
      .iter()
      .any(|c| c.get_name() == "date" && is_ltx(c));
    let mut trailer = String::new();
    for r in &self.resources_xml {
      trailer.push_str(r);
    }
    if !has_direct_date {
      for d in &self.dates_xml {
        trailer.push_str(d);
      }
    }
    for nav in &self.navs_xml {
      trailer.push_str(nav);
    }
    let effective_lang = xml_ns_attr(page, "lang").or(lang);
    let effective_bg = page.get_attribute("backgroundcolor").or(bg);
    let mut extra_attrs: Vec<(String, String)> = Vec::new();
    if xml_ns_attr(page, "lang").is_none()
      && let Some(l) = &effective_lang
    {
      extra_attrs.push(("xml:lang".to_string(), l.clone()));
    }
    if page.get_attribute("backgroundcolor").is_none()
      && let Some(b) = &effective_bg
    {
      extra_attrs.push(("backgroundcolor".to_string(), b.clone()));
    }
    let mut class_appended = false;
    let mut class_merge: Option<String> = None;
    if let Some(pclass) = &self.root_class {
      if page.get_attribute("class").is_some() {
        class_merge = Some(pclass.clone());
      } else {
        extra_attrs.push(("class".to_string(), pclass.clone()));
        class_appended = true;
      }
    }
    self.metas[idx].class_appended = class_appended;
    let mut decls: Vec<(String, String)> = Vec::new();
    for lvl in &self.levels {
      for (p, u) in &lvl.ns_decls {
        if !decls.iter().any(|(dp, _)| dp == p) {
          decls.push((p.clone(), u.clone()));
        }
      }
    }
    amend_serialized_page(
      &mut serialized,
      &decls,
      &extra_attrs,
      class_merge.as_deref(),
      &trailer,
    )?;
    let mut out = self.spill_prolog();
    out.push_str(&serialized);
    out.push('\n');
    write_spill(&self.metas[idx].file, &out)?;
    self.first_page_spilled = true;
    Ok(())
  }

  /// Insert a generated TOC element into the live mini-DOM before `anchor`
  /// (or append, when the run ended at the element's close). Built with
  /// direct node construction (the `PostDocument::add_nodes` pattern: prefer
  /// the in-scope default ltx declaration so serialization matches the
  /// document's own vocabulary).
  fn dom_insert_toc(
    &mut self,
    doc: &Document,
    parent: &Node,
    anchor: Option<&Node>,
    parent_type: &str,
    ids: &[String],
  ) -> Result<(), String> {
    let mut parent = parent.clone();
    let ns = parent
      .get_namespaces(doc)
      .into_iter()
      .find(|ns| ns.get_href() == LTX_NSURI && ns.get_prefix().is_empty())
      .or_else(|| {
        parent
          .get_namespaces(doc)
          .into_iter()
          .find(|ns| ns.get_href() == LTX_NSURI)
      });
    let mut toc = parent
      .new_child(ns.clone(), "TOC")
      .map_err(|e| format!("cannot create TOC element: {e}"))?;
    let ns = ns.or_else(|| Namespace::new("", LTX_NSURI, &mut toc).ok());
    let mut toclist = toc
      .new_child(ns.clone(), "toclist")
      .map_err(|e| format!("cannot create toclist: {e}"))?;
    toclist
      .set_attribute("class", &format!("ltx_toclist_{parent_type}"))
      .map_err(|e| format!("cannot set toclist class: {e:?}"))?;
    for id in ids {
      let mut entry = toclist
        .new_child(ns.clone(), "tocentry")
        .map_err(|e| format!("cannot create tocentry: {e}"))?;
      let mut r = entry
        .new_child(ns.clone(), "ref")
        .map_err(|e| format!("cannot create ref: {e}"))?;
      r.set_attribute("idref", id)
        .map_err(|e| format!("cannot set idref: {e:?}"))?;
      r.set_attribute("show", "toctitle")
        .map_err(|e| format!("cannot set show: {e:?}"))?;
    }
    if let Some(a) = anchor {
      // `new_child` appended the TOC; move it to the run's position.
      a.clone()
        .add_prev_sibling(&mut toc)
        .map_err(|e| format!("cannot position TOC: {e:?}"))?;
    }
    Ok(())
  }

  // ====================================================================
  // Deferred passes

  /// Port of `Split::prename_pages` + `get_page_name` over the metadata tree
  /// (names all children of a node, then recurses into each — the order the
  /// `FOO{n}` counter depends on).
  fn prename(&mut self) {
    let ext = Path::new(&self.root_destination)
      .extension()
      .map(|e| e.to_string_lossy().to_string())
      .unwrap_or_else(|| "xml".to_string());
    // `haschildren`: keyed by the localname of any element with page
    // children (the root's included) — drives `*Relative` dir naming.
    let mut haschildren: HashMap<String, bool> = HashMap::default();
    for m in &self.metas {
      if !m.children.is_empty() {
        haschildren.insert(m.localname.clone(), true);
      }
    }
    self.prename_rec(0, &ext, &haschildren);
  }

  fn prename_rec(&mut self, node: usize, ext: &str, haschildren: &HashMap<String, bool>) {
    let children = self.metas[node].children.clone();
    for &child in &children {
      let recursive = haschildren
        .get(&self.metas[child].localname)
        .copied()
        .unwrap_or(false);
      let name = self.get_page_name(child, node, ext, recursive);
      self.metas[child].name = name;
    }
    for &child in &children {
      self.prename_rec(child, ext, haschildren);
    }
  }

  /// Port of `Split::get_page_name` over metadata.
  fn get_page_name(&mut self, page: usize, parent: usize, ext: &str, recursive: bool) -> String {
    let use_labels = matches!(self.naming, SplitNaming::Label | SplitNaming::LabelRelative);
    let attr_name = if use_labels { "labels" } else { "xml:id" };
    let raw = if use_labels {
      self.metas[page].labels.clone()
    } else {
      self.metas[page].xml_id.clone()
    };
    let mut name = raw.unwrap_or_default();
    if let Some(first) = name.split_whitespace().next() {
      name = first.to_string();
    }
    if let Some(stripped) = name.strip_prefix("LABEL:") {
      name = stripped.to_string();
    }
    if name.is_empty() {
      if use_labels && let Some(id) = self.metas[page].xml_id.clone() {
        Info!(
          "split",
          "pathname",
          "Using '{}' to create page pathname, instead of missing '{}'",
          id,
          attr_name
        );
        name = id;
      } else {
        self.unnamed_counter += 1;
        name = format!("FOO{}", self.unnamed_counter);
        Info!(
          "split",
          "pathname",
          "Using '{}' to create page pathname, instead of missing '{}'",
          name,
          attr_name
        );
      }
    }
    let as_dir = match self.naming {
      SplitNaming::IdRelative | SplitNaming::LabelRelative => {
        let parent_attr = if use_labels {
          self.metas[parent].labels.clone()
        } else {
          self.metas[parent].xml_id.clone()
        };
        if let Some(pname) = parent_attr {
          let pname = pname.split_whitespace().next().unwrap_or("");
          let pname = pname.strip_prefix("LABEL:").unwrap_or(pname);
          if let Some(rest) = name.strip_prefix(pname) {
            let rest = rest.trim_start_matches(['.', '_', ':']);
            if !rest.is_empty() {
              name = rest.to_string();
            }
          }
        }
        recursive
      },
      _ => false,
    };
    name = name.replace(':', "_");
    let parent_path = &self.metas[parent].name;
    let parent_dir = Path::new(parent_path)
      .parent()
      .and_then(|p| p.to_str())
      .unwrap_or(".");
    let parent_dir = if parent_dir.is_empty() {
      "."
    } else {
      parent_dir
    };
    if as_dir {
      format!("{}/{}/index.{}", parent_dir, name, ext)
    } else {
      format!("{}/{}.{}", parent_dir, name, ext)
    }
  }

  /// The `inlist="toc"` lookahead pass: for each tree level where any page's
  /// `inlist` contains `"toc"` (substring semantics, mirroring the DOM
  /// path), patch `inlist="toc"` into the spilled root tag of every sibling
  /// page that has no `inlist` of its own.
  fn patch_inlist_toc(&mut self) -> Result<(), String> {
    for node in 0..self.metas.len() {
      let children = &self.metas[node].children;
      if children.is_empty() {
        continue;
      }
      let intoc = children.iter().any(|&c| {
        self.metas[c]
          .inlist
          .as_deref()
          .is_some_and(|il| il.contains("toc"))
      });
      if !intoc {
        continue;
      }
      let to_patch: Vec<usize> = children
        .iter()
        .copied()
        .filter(|&c| self.metas[c].inlist.is_none())
        .collect();
      for c in to_patch {
        patch_spill_root_tag(
          &self.metas[c].file,
          "inlist",
          "toc",
          self.metas[c].class_appended,
        )?;
        self.metas[c].inlist = Some("toc".to_string());
      }
    }
    Ok(())
  }
}

impl Level {
  fn seen_ltx_insert(&mut self, ltx: bool, localname: &str) {
    if ltx {
      self.seen_ltx.insert(localname.to_string());
    }
  }
}

// ======================================================================
// Text-level helpers

/// Assemble one spill file: prolog, then the element (self-closed when it
/// has no content), then the tail.
fn assemble_spill(prolog: &str, tag: &str, qname: &str, content: &str, tail: &str) -> String {
  let mut out = String::with_capacity(prolog.len() + tag.len() + content.len() + tail.len() + 16);
  out.push_str(prolog);
  out.push_str(tag);
  if content.is_empty() {
    out.push_str("/>\n");
  } else {
    out.push('>');
    out.push_str(content);
    out.push_str("</");
    out.push_str(qname);
    out.push_str(">\n");
  }
  out.push_str(tail);
  out
}

/// libxml-compatible attribute-value escaping (`xmlAttrSerializeTxtContent`).
fn attr_escape(value: &str) -> String {
  let mut out = String::with_capacity(value.len());
  for ch in value.chars() {
    match ch {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      '"' => out.push_str("&quot;"),
      '\n' => out.push_str("&#10;"),
      '\r' => out.push_str("&#13;"),
      '\t' => out.push_str("&#9;"),
      _ => out.push(ch),
    }
  }
  out
}

/// libxml-compatible text-node escaping.
fn text_escape(value: &str) -> String {
  let mut out = String::with_capacity(value.len());
  for ch in value.chars() {
    match ch {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      '\r' => out.push_str("&#13;"),
      _ => out.push(ch),
    }
  }
  out
}

fn attr_value(attrs: &[(String, String)], name: &str) -> Option<String> {
  attrs
    .iter()
    .find(|(k, _)| k == name)
    .map(|(_, v)| v.clone())
}

/// The namespace declarations among an element's attributes, as
/// `(prefix, uri)` pairs (`""` = default).
fn decl_attrs(attrs: &[(String, String)]) -> Vec<(String, String)> {
  let mut out = Vec::new();
  for (k, v) in attrs {
    if k == "xmlns" {
      out.push((String::new(), v.clone()));
    } else if let Some(p) = k.strip_prefix("xmlns:") {
      out.push((p.to_string(), v.clone()));
    }
  }
  out
}

/// Read an `xml:`-namespaced attribute from a DOM node. A plain
/// `get_attribute("xml:lang")` can miss the namespaced form (the same trap
/// `get_xml_id` guards against), so try the namespace-aware read first.
fn xml_ns_attr(node: &Node, localname: &str) -> Option<String> {
  const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
  node
    .get_attribute_ns(localname, XML_NS)
    .or_else(|| node.get_attribute(&format!("xml:{localname}")))
}

/// Conservative substring probe for "this serialized fragment may contain an
/// element with `localname`": matches `<localname` or `<pfx:localname`
/// followed by a name-boundary character, so `<indexmark` does NOT register
/// as `<index` (a real document's `\\index` markers would otherwise trigger a
/// mini-DOM descent per paragraph). Text content cannot introduce `<`
/// (escaped), so `<`-anchored matches are always element starts;
/// `:`-anchored matches can over-trigger on text (harmless — descent is
/// correct, just slower).
fn contains_element_probe(outer: &str, localname: &str) -> bool {
  let boundary = |rest: &str| {
    rest
      .as_bytes()
      .first()
      .is_none_or(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/'))
  };
  let bare = format!("<{localname}");
  for (at, _) in outer.match_indices(&bare) {
    if boundary(&outer[at + bare.len()..]) {
      return true;
    }
  }
  let prefixed = format!(":{localname}");
  for (at, _) in outer.match_indices(&prefixed) {
    if boundary(&outer[at + prefixed.len()..]) {
      return true;
    }
  }
  false
}

/// Does this serialized fragment contain an `ltx:TOC` with `lists` exactly
/// `"toc"`? (The DOM probe is `@lists='toc'`, exact equality; generated TOCs
/// never match — they carry no `lists`.)
fn probe_lists_toc(outer: &str) -> bool {
  for (start, _) in outer
    .match_indices("<TOC")
    .chain(outer.match_indices(":TOC"))
  {
    let rest = &outer[start..];
    if let Some(end) = rest.find('>')
      && rest[..end].contains(" lists=\"toc\"")
    {
      return true;
    }
  }
  false
}

/// Extract `ltx:picture` spans from a serialized fragment into the
/// SVG-extraction buffer. (The DOM path serializes each `//ltx:picture` node
/// and hands the concatenation to a regex-based fragment table; span
/// extraction at the text level is equivalent input for that table.)
fn collect_pictures(outer: &str, into: &mut String) {
  let mut search_from = 0;
  while let Some(rel) = outer[search_from..].find("<picture") {
    let start = search_from + rel;
    match outer[start..].find("</picture>") {
      Some(rel_end) => {
        let end = start + rel_end + "</picture>".len();
        into.push_str(&outer[start..end]);
        search_from = end;
      },
      None => break,
    }
  }
  // Prefixed form (`<pfx:picture …>` in prefixed documents).
  let mut search_from = 0;
  while let Some(rel) = outer[search_from..].find(":picture") {
    let colon = search_from + rel;
    let after = colon + ":picture".len();
    let is_open = outer[..colon].rfind('<').is_some_and(|lt| {
      lt + 1 < colon
        && !outer[lt..].starts_with("</")
        && outer[lt + 1..colon]
          .chars()
          .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    });
    if is_open {
      let lt = outer[..colon].rfind('<').expect("checked above");
      if let Some(rel_end) = outer[after..].find(":picture>") {
        let end = after + rel_end + ":picture>".len();
        into.push_str(&outer[lt..end]);
        search_from = end;
        continue;
      }
    }
    search_from = after;
  }
}

/// Extract the bodies of `<?latexml …?>` PIs embedded in a serialized
/// fragment.
fn extract_pi_bodies(outer: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut from = 0;
  while let Some(rel) = outer[from..].find("<?latexml") {
    let start = from + rel + "<?latexml".len();
    if let Some(rel_end) = outer[start..].find("?>") {
      out.push(outer[start..start + rel_end].trim().to_string());
      from = start + rel_end + 2;
    } else {
      break;
    }
  }
  out
}

/// Collect `ltx:` descendants named `localname` (pre-order, limit-safe walk).
fn collect_ltx_descendants(node: &Node, localname: &str, out: &mut Vec<Node>) {
  let mut child = node.get_first_child();
  while let Some(c) = child {
    if c.get_type() == Some(NodeType::ElementNode) {
      if c.get_name() == localname && is_ltx(&c) {
        out.push(c.clone());
      }
      collect_ltx_descendants(&c, localname, out);
    }
    child = c.get_next_sibling();
  }
}

/// The `descendant::ltx:TOC[@lists='toc']` probe, DOM flavor.
fn has_lists_toc_descendant(node: &Node) -> bool {
  let mut child = node.get_first_child();
  while let Some(c) = child {
    if c.get_type() == Some(NodeType::ElementNode)
      && (has_lists_toc_node(&c) || has_lists_toc_descendant(&c))
    {
      return true;
    }
    child = c.get_next_sibling();
  }
  false
}

fn has_lists_toc_node(node: &Node) -> bool {
  node.get_type() == Some(NodeType::ElementNode)
    && node.get_name() == "TOC"
    && is_ltx(node)
    && node.get_attribute("lists").as_deref() == Some("toc")
}

/// Find the end of the first tag in a well-formed serialized element: the
/// first `>` outside quoted attribute values (attribute values may legally
/// contain `>`).
fn first_tag_end(xml: &str) -> Option<usize> {
  let bytes = xml.as_bytes();
  let mut in_quote: Option<u8> = None;
  for (i, &b) in bytes.iter().enumerate() {
    match in_quote {
      Some(q) => {
        if b == q {
          in_quote = None;
        }
      },
      None => match b {
        b'"' | b'\'' => in_quote = Some(b),
        b'>' => return Some(i),
        _ => {},
      },
    }
  }
  None
}

/// Amend a DOM-serialized page: inject namespace re-declarations and extra
/// attributes into the opening tag, optionally merge the root class into an
/// existing `class` attribute, and splice the template trailer before the
/// closing tag.
fn amend_serialized_page(
  serialized: &mut String,
  decls: &[(String, String)],
  extra_attrs: &[(String, String)],
  class_merge: Option<&str>,
  trailer: &str,
) -> Result<(), String> {
  let tag_end =
    first_tag_end(serialized).ok_or_else(|| "malformed page serialization".to_string())?;
  let self_closing = serialized[..tag_end].ends_with('/');
  let insert_at = if self_closing { tag_end - 1 } else { tag_end };
  let mut additions = String::new();
  for (p, u) in decls {
    let probe = if p.is_empty() {
      " xmlns=".to_string()
    } else {
      format!(" xmlns:{p}=")
    };
    if !serialized[..tag_end].contains(&probe) {
      if p.is_empty() {
        additions.push_str(&format!(" xmlns=\"{}\"", attr_escape(u)));
      } else {
        additions.push_str(&format!(" xmlns:{}=\"{}\"", p, attr_escape(u)));
      }
    }
  }
  for (k, v) in extra_attrs {
    additions.push_str(&format!(" {}=\"{}\"", k, attr_escape(v)));
  }
  serialized.insert_str(insert_at, &additions);
  if let Some(pclass) = class_merge {
    let tag_end = first_tag_end(serialized).ok_or("malformed tag")?;
    if let Some(cpos) = serialized[..tag_end].find(" class=\"") {
      let vstart = cpos + " class=\"".len();
      if let Some(vlen) = serialized[vstart..tag_end].find('"') {
        let existing = serialized[vstart..vstart + vlen].to_string();
        let merged = if existing.is_empty() {
          attr_escape(pclass)
        } else {
          format!("{} {}", existing, attr_escape(pclass))
        };
        serialized.replace_range(vstart..vstart + vlen, &merged);
      }
    }
  }
  if !trailer.is_empty() {
    let tag_end = first_tag_end(serialized).ok_or("malformed tag")?;
    if serialized[..=tag_end].ends_with("/>") {
      // `<x/>` → `<x>trailer</x>`: re-open the element.
      let qname_end = serialized
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(tag_end);
      let qname = serialized[1..qname_end].to_string();
      serialized.truncate(tag_end - 1);
      serialized.push('>');
      serialized.push_str(trailer);
      serialized.push_str(&format!("</{qname}>"));
    } else if let Some(close_at) = serialized.rfind("</") {
      serialized.insert_str(close_at, trailer);
    }
  }
  Ok(())
}

/// Patch ` name="value"` into a spill file's root element tag, inserting
/// before the trailing `class` attribute when the class was appended by the
/// merge (reproducing the DOM path's attribute order: inherit → inlist →
/// class).
fn patch_spill_root_tag(
  file: &Path,
  name: &str,
  value: &str,
  before_appended_class: bool,
) -> Result<(), String> {
  let content = std::fs::read_to_string(file)
    .map_err(|e| format!("cannot read spill {} for patching: {e}", file.display()))?;
  // Locate the root element: the first `<` not opening a PI or comment.
  let mut pos = 0;
  let root_start = loop {
    let rel = content[pos..]
      .find('<')
      .ok_or_else(|| "no root tag in spill".to_string())?;
    let at = pos + rel;
    if content[at..].starts_with("<?") {
      pos = at + content[at..].find("?>").ok_or("unterminated PI")? + 2;
    } else if content[at..].starts_with("<!--") {
      pos = at + content[at..].find("-->").ok_or("unterminated comment")? + 3;
    } else {
      break at;
    }
  };
  let tag_end = root_start
    + first_tag_end(&content[root_start..]).ok_or_else(|| "malformed root tag".to_string())?;
  let tag_self_close_adjust = if content[..tag_end].ends_with('/') {
    tag_end - 1
  } else {
    tag_end
  };
  let insert_at = if before_appended_class {
    content[root_start..tag_end]
      .rfind(" class=\"")
      .map(|rel| root_start + rel)
      .unwrap_or(tag_self_close_adjust)
  } else {
    tag_self_close_adjust
  };
  let mut patched = String::with_capacity(content.len() + 16);
  patched.push_str(&content[..insert_at]);
  patched.push_str(&format!(" {}=\"{}\"", name, attr_escape(value)));
  patched.push_str(&content[insert_at..]);
  std::fs::write(file, patched).map_err(|e| format!("cannot rewrite spill: {e}"))
}

fn write_spill(path: &Path, content: &str) -> Result<(), String> {
  let mut f = std::fs::File::create(path)
    .map_err(|e| format!("cannot create spill {}: {e}", path.display()))?;
  f.write_all(content.as_bytes())
    .map_err(|e| format!("cannot write spill {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn escaping_matches_libxml() {
    assert_eq!(
      attr_escape("a<b>&\"c\"\nd\te\r"),
      "a&lt;b&gt;&amp;&quot;c&quot;&#10;d&#9;e&#13;"
    );
    assert_eq!(text_escape("a<b>&c\r"), "a&lt;b&gt;&amp;c&#13;");
  }

  #[test]
  fn first_tag_end_skips_gt_inside_quotes() {
    assert_eq!(first_tag_end(r#"<x a="1>2">rest</x>"#), Some(10));
    assert_eq!(first_tag_end("<x/>"), Some(3));
    assert_eq!(first_tag_end("<never-closed"), None);
  }

  #[test]
  fn probe_lists_toc_is_exact() {
    assert!(probe_lists_toc(r#"<p><TOC lists="toc"><t/></TOC></p>"#));
    assert!(probe_lists_toc(r#"<ltx:TOC lists="toc"/>"#));
    // Generated TOCs (no lists attribute) never match.
    assert!(!probe_lists_toc(r#"<TOC><toclist class="c"/></TOC>"#));
    // Exact equality: 'toc lof' does not match the @lists='toc' probe.
    assert!(!probe_lists_toc(r#"<TOC lists="toc lof"/>"#));
  }

  #[test]
  fn amend_injects_attrs_and_trailer() {
    let mut s = r#"<section xml:id="S1"><p>t</p></section>"#.to_string();
    amend_serialized_page(
      &mut s,
      &[(String::new(), "urn:ns".to_string())],
      &[("xml:lang".to_string(), "en".to_string())],
      None,
      "<date>d</date>",
    )
    .unwrap();
    // Additions (declarations + attributes) append after the original
    // attributes; declaration position within a tag is semantically free.
    assert_eq!(
      s,
      r#"<section xml:id="S1" xmlns="urn:ns" xml:lang="en"><p>t</p><date>d</date></section>"#
    );
  }

  #[test]
  fn amend_reopens_self_closed_page_for_trailer() {
    let mut s = "<chapter/>".to_string();
    amend_serialized_page(&mut s, &[], &[], None, "<x/>").unwrap();
    assert_eq!(s, "<chapter><x/></chapter>");
  }

  #[test]
  fn amend_merges_class() {
    let mut s = r#"<section class="own"><p/></section>"#.to_string();
    amend_serialized_page(&mut s, &[], &[], Some("root"), "").unwrap();
    assert_eq!(s, r#"<section class="own root"><p/></section>"#);
  }

  #[test]
  fn patch_inserts_before_appended_class() {
    let dir = std::env::temp_dir();
    let file = dir.join(format!("lxo-patch-test-{}.xml", std::process::id()));
    std::fs::write(
      &file,
      "<?xml version=\"1.0\"?>\n<?latexml p?>\n<section xml:id=\"S1\" class=\"root\"><p/></section>\n",
    )
    .unwrap();
    patch_spill_root_tag(&file, "inlist", "toc", true).unwrap();
    let out = std::fs::read_to_string(&file).unwrap();
    assert!(
      out.contains(r#"<section xml:id="S1" inlist="toc" class="root">"#),
      "patched tag order wrong: {out}"
    );
    // Without the appended-class flag, the attribute lands last.
    std::fs::write(
      &file,
      "<?xml version=\"1.0\"?>\n<section xml:id=\"S1\" class=\"own\"><p/></section>\n",
    )
    .unwrap();
    patch_spill_root_tag(&file, "inlist", "toc", false).unwrap();
    let out = std::fs::read_to_string(&file).unwrap();
    assert!(
      out.contains(r#"<section xml:id="S1" class="own" inlist="toc">"#),
      "patched tag order wrong: {out}"
    );
    std::fs::remove_file(&file).ok();
  }

  #[test]
  fn collect_pictures_extracts_spans() {
    let mut buf = String::new();
    collect_pictures(
      r#"<p>x</p><picture xml:id="p1"><g/></picture><q/><picture xml:id="p2"/>...</picture>"#,
      &mut buf,
    );
    assert!(buf.starts_with(r#"<picture xml:id="p1"><g/></picture>"#));
  }

  /// End-to-end mini split: a wrapper (backmatter) page must carry the ltx
  /// namespace declaration in its spill, or the XSLT will not recognize it.
  #[test]
  fn wrapper_page_spill_is_namespaced() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("mini.xml");
    std::fs::write(
      &src,
      r#"<?xml version="1.0" encoding="UTF-8"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" class="rc">
  <title>T</title>
  <chapter xml:id="C1"><title>One</title><para xml:id="C1.p1"><p>x</p></para></chapter>
  <backmatter>
    <section xml:id="BM.S1"><title>BS</title></section>
    <appendix xml:id="A1"><title>App</title></appendix>
  </backmatter>
</document>
"#,
    )
    .unwrap();
    let spill = dir.path().join("spill");
    std::fs::create_dir(&spill).unwrap();
    let union = "//ltx:section | //ltx:chapter | //ltx:appendix[preceding-sibling::ltx:section or parent::ltx:chapter]";
    let outcome = stream_split(
      &src.to_string_lossy(),
      union,
      SplitNaming::Id,
      Some("out/mini.html"),
      &spill,
    )
    .expect("split runs")
    .expect("split produces pages");
    let names: Vec<&str> = outcome
      .pages
      .iter()
      .map(|p| p.destination.as_str())
      .collect();
    assert_eq!(
      names,
      vec![
        "out/mini.html",
        "out/C1.html",
        "out/BM.S1.html",
        "out/A1.html"
      ],
      "pre-order destinations"
    );
    for page in &outcome.pages[1..] {
      let content = std::fs::read_to_string(&page.path).unwrap();
      assert!(
        content.contains("xmlns=\"http://dlmf.nist.gov/LaTeXML\""),
        "page {} must declare the ltx namespace:\n{content}",
        page.destination
      );
      // Reparse: root element must be in the ltx namespace.
      let doc = libxml::parser::Parser::default()
        .parse_string(&content)
        .expect("page reparses");
      let root = doc.get_root_element().unwrap();
      assert_eq!(
        root.get_namespace().map(|n| n.get_href()),
        Some(LTX_NSURI.to_string()),
        "page {} root not in ltx namespace:\n{content}",
        page.destination
      );
    }
  }

  #[test]
  fn element_probe_respects_name_boundaries() {
    // `<indexmark>` must NOT register as an `index` element…
    assert!(!contains_element_probe(
      "<p><indexmark k=\"x\"/></p>",
      "index"
    ));
    // …while real starts do, in every tag shape and prefixed form.
    assert!(contains_element_probe("<p><index r=\"1\"/></p>", "index"));
    assert!(contains_element_probe("<index>", "index"));
    assert!(contains_element_probe("<index/>", "index"));
    assert!(contains_element_probe("<ltx:index>x</ltx:index>", "index"));
    assert!(!contains_element_probe("<subsubsection>", "subsection"));
    assert!(!contains_element_probe("plain text", "section"));
  }

  #[test]
  fn pi_bodies_extracted() {
    assert_eq!(
      extract_pi_bodies(r#"<a><?latexml package="x"?><?other y?><?latexml class="c"?></a>"#),
      vec![r#"package="x""#.to_string(), r#"class="c""#.to_string()]
    );
  }
}
