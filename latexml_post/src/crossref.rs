//! Cross-reference resolution processor.
//!
//! Port of `LaTeXML::Post::CrossRef` (946 lines of Perl).
//! Resolves cross-references (`ltx:ref`, `ltx:bibref`, etc.) by looking up
//! referenced IDs in the ObjectDB and filling in the reference text,
//! titles, and navigation links.

use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::LazyLock};

use libxml::tree::{Node, NodeType};
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  document::{NodeData, PostDocument, get_xml_id},
  object_db::{Entry, ObjectDB, Value},
  processor::{ProcessResult, Processor},
  scan::title_text_content,
};

/// Perl CrossRef.pm `$normaltoctypes` (L202-206): the sectional element types
/// used by `gentoc_context`'s UPWARD ancestor/sibling enclosure. This is NOT
/// the normal `gen_toc` path — that filters purely by the TOC's
/// `select`/`inlist` (issue #291). Deliberately excludes
/// `ltx:abstract`/`ltx:acknowledgements` (matching Perl exactly) so frontmatter
/// does not clutter the navigation breadcrumb's sibling rows.
const NORMAL_TOC_TYPES: &[&str] = &[
  "ltx:document",
  "ltx:part",
  "ltx:chapter",
  "ltx:section",
  "ltx:subsection",
  "ltx:subsubsection",
  "ltx:paragraph",
  "ltx:subparagraph",
  "ltx:index",
  "ltx:bibliography",
  "ltx:glossary",
  "ltx:appendix",
];

/// Memoized result of `get_child_page_ids` for one ObjectDB entry: the
/// distinct descendant page ids, plus a position index so
/// `find_previous_page_id`/`find_next_page_id` can locate a sibling in O(1)
/// instead of the Perl pop/shift scan.
struct ChildPages {
  ids:      Vec<String>,
  index_of: HashMap<String, usize>,
}

/// Fallback fields when a requested ref show key is not found.
fn ref_fallbacks(key: &str) -> &'static [&'static str] {
  match key {
    "typerefnum" => &["refnum"],
    "toctitle" => &["title", "toccaption"],
    "title" => &["toccaption"],
    "rawtoctitle" => &["toctitle", "title", "toccaption"],
    "rawtitle" => &["title", "toccaption"],
    _ => &[],
  }
}

/// Derive the STRING form of a stored value (page `<title>`, `title=` tooltip).
/// Perl `CrossRef::getTextContent` (`CrossRef.pm` L853-859). A `Value::Xml` title
/// is flattened tag-aware and math-aware (via [`title_text_content`], which routes
/// `ltx:Math` through `unicodemath`); any other value uses its plain string form.
/// Either way the result is whitespace-collapsed like Perl: trim both ends, then
/// `s/\s+/ /g` — so a multi-line math serialization cannot bloat a `title=`
/// tooltip (issue #761).
fn value_text(doc: &PostDocument, val: &Value) -> String {
  let raw = match val {
    Value::Xml(node) => title_text_content(doc, node),
    other => other.to_string(),
  };
  // Perl `getTextContent`: `s/^\s+//; s/\s+$//; s/\s+/ /g`. `split_whitespace`
  // does exactly this (drops leading/trailing runs, collapses interior runs).
  raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Build the child nodes of an `<ltx:ref>` from a stored value.
///
/// Port of Perl `CrossRef::prepRefText` = `cloneNodes(trimChildNodes($value))`:
/// deep-clone the title's child nodes — `<ltx:Math>` included — trimming
/// whitespace at the two edges. Element children become [`NodeData::XmlNode`]
/// (deep-copied at materialization by `PostDocument::add_xml_node`, which
/// uniquifies their `xml:id`s); text children become [`NodeData::Text`]. A
/// plain-string value keeps the single flat-text child.
///
/// (Perl's `fillInTitle` — resolving nested `ltx:ref`/`ltx:bibref`/`ltx:break`
/// embedded in a title before cloning — is not ported here; those are rare in
/// titles and were not handled by the previous flat-text path either.)
fn ref_content_children(val: &Value) -> Vec<NodeData> {
  let node = match val {
    Value::Xml(node) => node,
    other => return vec![NodeData::Text(other.to_string())],
  };
  let mut out = child_nodes_data(node);
  // trimChildNodes: left-trim the first text child, right-trim the last; drop
  // either if it becomes empty.
  if let Some(NodeData::Text(s)) = out.first_mut() {
    let t = s.trim_start().to_string();
    if t.is_empty() {
      out.remove(0);
    } else {
      *s = t;
    }
  }
  if let Some(NodeData::Text(s)) = out.last_mut() {
    let t = s.trim_end().to_string();
    if t.is_empty() {
      out.pop();
    } else {
      *s = t;
    }
  }
  out
}

/// Strip `fragid` from everything inside an `<ltx:ref>` (TOC entries, inline
/// refs, navigation).
///
/// Reference content is a non-anchor DISPLAY copy of a title, so it must not
/// carry a `fragid` — the XSLT `add_id` template emits the HTML `id` from
/// `fragid`, and a display copy with an `id` would spuriously duplicate the
/// real target's anchor. Perl gets this for free: its ref content is cloned
/// from Scan's `cleanNode` snapshot, taken before `fragid` is assigned
/// (`Scan.pm` L290 / `CrossRef.pm` fillInFrags). We clone the live (already
/// `fragid`'d) title, so we drop `fragid` here to match (issue #356). The
/// uniquified `xml:id` is kept, as in Perl's snapshot.
fn strip_ref_display_fragids(doc: &PostDocument) {
  for mut n in doc.findnodes("//ltx:ref//*[@fragid]") {
    let _ = n.remove_attribute("fragid");
  }
}

/// URL style for cross-references (Perl `--urlstyle`; `Config.pm` accepts
/// `server`, `negotiated`, `file`). Selects how [`CrossRef`]'s URL generation
/// rewrites a generated cross-reference URL for the serving environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlStyle {
  /// Keep the full `file.html#fragment` — nothing stripped. Correct for
  /// `file://` viewing and servers that do not rewrite `index.html`.
  File,
  /// Strip a trailing `index.ext` so a directory index links as `dir/`
  /// (Perl's `latexml` default; for servers that serve `dir/` → `dir/index.ext`).
  Server,
  /// Content negotiation: strip the `.ext` extension AND a trailing `index`
  /// (for servers that hide the extension, e.g. BookML's `--urlstyle=negotiated`).
  Negotiated,
}

impl UrlStyle {
  /// Parse a CLI `--urlstyle` value. Returns `None` for an unrecognized value
  /// (the caller reports it, mirroring Perl `_checkOptionValue`).
  pub fn from_cli(s: &str) -> Option<Self> {
    match s {
      "file" => Some(UrlStyle::File),
      "server" => Some(UrlStyle::Server),
      "negotiated" => Some(UrlStyle::Negotiated),
      _ => None,
    }
  }

  /// The canonical CLI tag — round-trips through [`UrlStyle::from_cli`]. Used to
  /// serialize the style across the parallel page-render worker manifest.
  pub fn as_cli(self) -> &'static str {
    match self {
      UrlStyle::File => "file",
      UrlStyle::Server => "server",
      UrlStyle::Negotiated => "negotiated",
    }
  }
}

/// Rewrite a generated cross-reference `url` for the given [`UrlStyle`], mirroring
/// Perl `CrossRef::generateURL` (CrossRef.pm L656-663) verbatim — including its
/// `(^|\/)` path boundary: a trailing `index[.ext]` is stripped only at the very
/// start of the URL or right after a `/`, so a filename like `myindex.html` is
/// left intact. `extension` is the output file extension (e.g. `html`).
fn apply_url_style(url: &str, style: UrlStyle, extension: &str) -> String {
  match style {
    // Perl: s/(^|\/)index.\Q$ext\E$/($1 ? $1 : '.\/')/e
    UrlStyle::Server => {
      let index_suffix = format!("index.{extension}");
      if let Some(prefix) = url.strip_suffix(&index_suffix) {
        if prefix.is_empty() {
          return "./".to_string(); // matched at start ($1 empty → './')
        } else if prefix.ends_with('/') {
          return prefix.to_string(); // matched after '/' ($1 = '/', kept)
        } // else: no path boundary before `index` → leave url unchanged
      }
      url.to_string()
    },
    // Perl: s/\.\Q$ext\E$// then s/(^|\/)index$/$1/
    UrlStyle::Negotiated => {
      let stripped = url.strip_suffix(&format!(".{extension}")).unwrap_or(url);
      if stripped == "index" {
        String::new() // matched at start ($1 empty)
      } else if let Some(prefix) = stripped.strip_suffix("index") {
        if prefix.ends_with('/') {
          prefix.to_string() // matched after '/' ($1 = '/', kept)
        } else {
          stripped.to_string() // no path boundary → leave (e.g. `myindex`)
        }
      } else {
        stripped.to_string()
      }
    },
    UrlStyle::File => url.to_string(),
  }
}

/// CrossRef post-processor.
///
/// Port of `LaTeXML::Post::CrossRef`.
pub struct CrossRef {
  name:           String,
  /// Reference to the shared ObjectDB.
  pub db:         ObjectDB,
  /// URL style for cross-references.
  url_style:      UrlStyle,
  /// File extension used for output (e.g. "html", "xml").
  extension:      String,
  /// Default show format for TOC refs.
  toc_show:       String,
  /// Default show format for regular refs.
  ref_show:       String,
  /// Minimum useful content length for refs.
  min_ref_length: usize,
  /// Join string between parent+child ref text.
  ref_join:       String,
  /// Type of navigation TOC to add (e.g. "context").
  navigation_toc: Option<String>,
  /// Track missing references for reporting.
  missing:        HashMap<String, HashMap<String, HashMap<String, u32>>>,
  /// Memoized `get_child_page_ids` results, keyed by entry id. The ObjectDB
  /// is read-only for the whole CrossRef pass, so a given entry's child pages
  /// never change — caching them across all split pages turns the per-page
  /// O(siblings) recomputation (Perl's unpruned `getChildPages`) into O(1)
  /// lookups, eliminating the `fill_in_relations` O(n²).
  child_pages:    RefCell<HashMap<String, Rc<ChildPages>>>,
}

/// One cited entry as Perl `make_bibcite` collects it (`CrossRef.pm` L516-562):
/// the trimmed display parts each `show` role clones, and the attributes of
/// every `<ltx:ref>` made for it.
struct BibciteDatum {
  key:         String,
  authors:     Vec<NodeData>,
  fullauthors: Vec<NodeData>,
  /// Text of `authors || fullauthors` — the key that groups consecutive
  /// same-author entries under one author label (L610). `''` without either,
  /// and for a missing entry (whose undef compares `eq` to `''`).
  authortext:  String,
  year:        Vec<NodeData>,
  /// The `(\d\d\d\d)(\w)` split of a suffixed year (`2001a`), so a same-author
  /// run shows `2001a, b` (L613).
  rawyear:     Option<String>,
  suffix:      Option<String>,
  number:      Vec<NodeData>,
  refnum:      Vec<NodeData>,
  title:       Vec<NodeData>,
  attr:        HashMap<String, String>,
  /// A key with no bibliography entry (L559-561): one `ltx_missing_citation`
  /// ref showing the key, whatever the `show`.
  missing:     bool,
}

/// Perl `Post::Document::trimChildNodes` (`Post.pm` L1374-1397) of a stored
/// bibitem tag value: its children with the leading whitespace of the first
/// text child and the trailing whitespace of the last trimmed away, a child left
/// empty dropped. A string value stands for the tag's text: trimmed, or nothing.
fn trim_child_nodes(val: Option<&Value>) -> Vec<NodeData> {
  match val {
    None | Some(Value::Null) => Vec::new(),
    Some(v @ Value::Xml(_)) => ref_content_children(v),
    Some(v) => {
      let s = v.to_string();
      let t = s.trim();
      if t.is_empty() {
        Vec::new()
      } else {
        vec![NodeData::Text(t.to_string())]
      }
    },
  }
}

/// A node's children as insertable data, untrimmed: text by value, elements by
/// reference (deep-copied on insertion). Perl hands `addNodes` the live
/// `childNodes`, which it copies the same way.
fn child_nodes_data(node: &Node) -> Vec<NodeData> {
  let mut out: Vec<NodeData> = Vec::new();
  let mut child = node.get_first_child();
  while let Some(c) = child {
    match c.get_type() {
      Some(NodeType::TextNode) => out.push(NodeData::Text(c.get_content())),
      Some(NodeType::ElementNode) => out.push(NodeData::XmlNode(c.clone())),
      _ => {},
    }
    child = c.get_next_sibling();
  }
  out
}

/// `['ltx:ref', $attr, @children]`.
fn bib_ref(attr: &HashMap<String, String>, children: Vec<NodeData>) -> NodeData {
  NodeData::Element {
    tag: "ltx:ref".to_string(),
    attributes: Some(attr.clone()),
    children,
  }
}

/// Perl `make_bibcite`'s show walk (`CrossRef.pm` L564-643) for one cited
/// entry, returning `(stuff, didref)`. The show string is a sequence of role
/// words — lowercased with one trailing `s` stripped (L585), so `Authors`,
/// `author` and `authors` are one role — among `{literal}` text, `~` (a
/// no-break space), whitespace and other punctuation, which pass through.
/// `author`/`fullauthor`/`title`/`refnum`/`phraseN` clone their values;
/// `year`/`number`/`super` make the `<ltx:ref>` link themselves (`didref`),
/// and absorb the following entries of the same authors when `checkdups`
/// (`Smith (2001a, b)`). Without a link made inside, the caller wraps the
/// whole label in one.
fn bibcite_show_walk(
  saveshow: &str,
  datum: &BibciteDatum,
  data: &mut VecDeque<BibciteDatum>,
  preformatted: &[NodeData],
  phrases: &[Node],
  yysep: &str,
  checkdups: bool,
) -> (Vec<NodeData>, bool) {
  static YEAR_DELIM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)(\w)year").unwrap());
  static PHRASE_DELIM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)(\w)phrase").unwrap());
  static PHRASE_NUM_DELIM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)phrase(\d)(\w)").unwrap());
  static ROLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w+").unwrap());
  static LITERAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\{([^}]*)\}").unwrap());
  static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s+").unwrap());
  static PUNCT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\W+").unwrap());
  static EMPTY_YEAR_PHRASE2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\{\}year\{\}phrase2").unwrap());

  let mut didref = false;
  let mut stuff: Vec<NodeData> = Vec::new();
  let mut show = saveshow;
  if show == "none" && !preformatted.is_empty() {
    stuff = preformatted.to_vec();
    show = "";
  } else if datum.missing {
    stuff = vec![bib_ref(&datum.attr, vec![NodeData::Text(
      datum.key.clone(),
    )])];
    didref = true;
    show = "";
  }
  // Add delimiters for parsing (L579-582): `Phrase1Year` → `Phrase1{}year`.
  let show = YEAR_DELIM.replace_all(show, "${1}{}year");
  let show = PHRASE_DELIM.replace_all(&show, "${1}{}phrase");
  let show = PHRASE_NUM_DELIM.replace_all(&show, "phrase${1}{}${2}");
  let mut rest: &str = &show;
  // The same-author run after `datum` (L610, L620, L627): the entries
  // `year`/`number`/`super` absorb.
  let same_authors = |data: &VecDeque<BibciteDatum>| {
    data
      .front()
      .is_some_and(|n| n.authortext == datum.authortext)
  };
  while !rest.is_empty() {
    if let Some(m) = ROLE.find(rest) {
      let mut role = m.as_str().to_lowercase();
      rest = &rest[m.end()..];
      if role.ends_with('s') {
        role.pop();
      }
      match role.as_str() {
        "author" => stuff.extend(datum.authors.iter().cloned()),
        "fullauthor" => stuff.extend(datum.fullauthors.iter().cloned()),
        "title" => stuff.extend(datum.title.iter().cloned()),
        "refnum" => stuff.extend(datum.refnum.iter().cloned()),
        "year" => {
          // (L605-606's "Date for citation" warning is unreachable: the year
          // list is always defined, possibly empty.)
          if !datum.year.is_empty() {
            stuff.push(bib_ref(&datum.attr, datum.year.clone()));
            didref = true;
            while checkdups && same_authors(data) {
              let next = data.pop_front().unwrap();
              stuff.push(NodeData::Text(yysep.to_string()));
              stuff.push(NodeData::Text(" ".to_string()));
              let same_year = datum.rawyear.as_deref().unwrap_or("no_year_1")
                == next.rawyear.as_deref().unwrap_or("no_year_2");
              match next.suffix {
                Some(ref suffix) if same_year => {
                  stuff.push(bib_ref(&next.attr, vec![NodeData::Text(suffix.clone())]))
                },
                _ => stuff.push(bib_ref(&next.attr, next.year.clone())),
              }
            }
          }
        },
        "number" | "super" => {
          let mut r = vec![bib_ref(&datum.attr, datum.number.clone())];
          didref = true;
          while checkdups && same_authors(data) {
            let next = data.pop_front().unwrap();
            r.push(NodeData::Text(yysep.to_string()));
            r.push(NodeData::Text(" ".to_string()));
            r.push(bib_ref(&next.attr, next.number.clone()));
          }
          if role == "super" {
            stuff.push(NodeData::Element {
              tag:        "ltx:sup".to_string(),
              attributes: None,
              children:   r,
            });
          } else {
            stuff.extend(r);
          }
        },
        _ => match role.strip_prefix("phrase").and_then(|d| {
          let mut ds = d.chars();
          ds.next()
            .and_then(|c| c.to_digit(10))
            .filter(|_| ds.next().is_none())
        }) {
          // L594-603. HACK (Perl's): an author-year show frozen before the
          // entry had a year drops the empty `( )` of `Phrase1{}Year{}Phrase2`.
          Some(n) => {
            let short = |i: usize| {
              phrases
                .get(i)
                .is_none_or(|p| p.get_content().chars().count() <= 1)
            };
            if n == 1
              && datum.year.is_empty()
              && short(0)
              && short(1)
              && let Some(m) = EMPTY_YEAR_PHRASE2.find(rest)
            {
              rest = &rest[m.end()..];
            } else {
              // `$phrases[$n - 1]`: `phrase0` is Perl's index -1, the last.
              let phrase = match n {
                0 => phrases.last(),
                n => phrases.get(n as usize - 1),
              };
              if let Some(phrase) = phrase {
                stuff.extend(child_nodes_data(phrase));
              }
            }
          },
          None => Info!("unexpected", role, "CITE ignoring show key '{}'", role),
        },
      }
    } else if let Some(c) = LITERAL.captures(rest) {
      // Pass-thru literal, quoted with {}.
      if !c[1].is_empty() {
        stuff.push(NodeData::Text(c[1].to_string()));
      }
      rest = &rest[c[0].len()..];
    } else if let Some(r) = rest.strip_prefix('~') {
      if !stuff.is_empty() {
        stuff.push(NodeData::Text("\u{A0}".to_string()));
      }
      rest = r;
    } else if let Some(m) = SPACES.find(rest) {
      if !stuff.is_empty() {
        stuff.push(NodeData::Text(m.as_str().to_string()));
      }
      rest = &rest[m.end()..];
    } else if let Some(m) = PUNCT.find(rest) {
      stuff.push(NodeData::Text(m.as_str().to_string()));
      rest = &rest[m.end()..];
    } else {
      // Unreachable: every character is `\w` or `\W`.
      break;
    }
  }
  (stuff, didref)
}

impl CrossRef {
  pub fn new(db: ObjectDB, url_style: UrlStyle, number_sections: bool) -> Self {
    CrossRef {
      name: "CrossRef".to_string(),
      db,
      url_style,
      extension: "xml".to_string(),
      toc_show: "toctitle".to_string(),
      ref_show: if number_sections {
        "refnum".to_string()
      } else {
        "title".to_string()
      },
      min_ref_length: 1,
      ref_join: " \u{2023} ".to_string(), // TRIANGULAR BULLET
      navigation_toc: None,
      missing: HashMap::default(),
      child_pages: RefCell::new(HashMap::default()),
    }
  }

  /// Set the file extension for URL generation.
  pub fn set_extension(&mut self, ext: &str) { self.extension = ext.to_string(); }

  /// Set the navigation TOC format.
  pub fn set_navigation_toc(&mut self, format: &str) {
    self.navigation_toc = Some(format.to_string());
  }

  /// Note a missing reference.
  fn note_missing(&mut self, severity: &str, ref_type: &str, key: &str) {
    self
      .missing
      .entry(severity.to_string())
      .or_default()
      .entry(ref_type.to_string())
      .or_default()
      .entry(key.to_string())
      .and_modify(|c| *c += 1)
      .or_insert(1);
  }

  /// Generate a URL for a referenced ID.
  ///
  /// Port of `CrossRef::generateURL`.
  fn generate_url(&mut self, doc: &PostDocument, id: &str) -> Option<String> {
    let entry = self.db.lookup(&format!("ID:{}", id))?;
    let location = entry.get_string("location")?;

    let doc_location = doc.site_relative_destination().unwrap_or_default();
    let mut url = relative_url(location, &doc_location);

    url = apply_url_style(&url, self.url_style, &self.extension);

    if url.is_empty() {
      url = ".".to_string();
    }

    // Add fragment ID
    let fragid = entry.get_string("fragid").map(String::from);
    let loc = location.to_string();
    if let Some(fid) = fragid {
      if url == "." || loc == doc_location {
        url = String::new();
      }
      url = format!("{}#{}", url, fid);
    } else if loc == doc_location {
      url = String::new();
    }

    Some(url)
  }

  /// Generate a title string for a referenced ID, traversing parents for context.
  ///
  /// Port of `CrossRef::generateTitle`.
  fn generate_title(&self, doc: &PostDocument, id: &str, shown: &str) -> Option<String> {
    let mut current_id = id.to_string();
    let mut result = String::new();
    let mut prefix = String::new();
    let mut shown_so_far = shown.to_string();

    while let Some(entry) = self.db.lookup(&format!("ID:{}", current_id)) {
      let mut pieces = Vec::new();
      let mut is_dup = false;

      // Try title, then typerefnum, then refnum
      if let Some(title_val) = entry.get_value("title") {
        if title_val.is_truthy() {
          is_dup = shown_so_far.contains("title");
          // The title is stored as a NODE (`Value::Xml`) for sections; derive
          // its string form tag-aware (Perl `getTextContent`). A plain-string
          // title (e.g. abstract/bibliography names) is used verbatim.
          pieces.push(value_text(doc, title_val));
        }
      }
      if pieces.is_empty() {
        let has_type = entry
          .get_value("tag:creftypecap")
          .or_else(|| entry.get_value("tag:creftype"));
        let has_refnum = entry.get_value("refnum");
        if has_type.is_some() && has_refnum.is_some() {
          is_dup = shown_so_far.contains("type") && shown_so_far.contains("refnum");
          if let Some(t) = has_type {
            pieces.push(t.to_string());
          }
          if let Some(r) = has_refnum {
            pieces.push(r.to_string());
          }
        } else if let Some(tr) = entry.get_value("typerefnum") {
          is_dup = shown_so_far.contains("type") && shown_so_far.contains("refnum");
          pieces.push(tr.to_string());
        } else if let Some(r) = has_refnum {
          is_dup = shown_so_far.contains("refnum");
          pieces.push(r.to_string());
        }
      }

      if is_dup {
        prefix = "In ".to_string();
        shown_so_far.clear();
      } else {
        let title = pieces.join(" ");
        let title = title.trim();
        if !title.is_empty() {
          result.push_str(&prefix);
          prefix = self.ref_join.clone();
          result.push_str(title);
        }
      }

      // Walk to parent for more context
      match entry.get_string("parent").map(String::from) {
        Some(pid) => current_id = pid,
        None => break,
      }
    }

    if result.is_empty() {
      None
    } else {
      Some(result)
    }
  }

  /// Generate a title for the document itself.
  ///
  /// Port of `CrossRef::generateDocumentTitle`.
  fn generate_document_title(&self, doc: &PostDocument) -> Option<String> {
    // Try to generate from the document's root ID. Use `get_xml_id` so we
    // pick up ids stored in the xml namespace (Scan's default placement)
    // as well as the bare "xml:id" attribute form.
    if let Some(docid) = doc.get_document_element().as_ref().and_then(get_xml_id) {
      // Perl `generateDocumentTile` (CrossRef.pm L809) calls
      // `generateTitle($doc, $docid)` with NO `$shown` arg → `$shown=''`. Passing
      // "toctitle" here is WRONG: `generate_title`'s dup test is `shown.contains("title")`
      // (Perl `$shown =~ /title/`), and "toctitle" contains "title", so the page's OWN
      // (deepest) title is falsely flagged a duplicate and dropped — every split section
      // page's <title> collapsed to "In <parent>" instead of "<section> ‣ <ancestors>".
      let title = self.generate_title(doc, &docid, "");
      if title.as_ref().map(|t| !t.is_empty()).unwrap_or(false) {
        return title;
      }
    }
    // Fallback: look for a title element in the document
    if let Some(node) =
      doc.findnode("//ltx:title | //ltx:toctitle | //ltx:caption | //ltx:toccaption")
    {
      let text = get_text_content_node(&node);
      if !text.is_empty() {
        return Some(text);
      }
    }
    None
  }

  /// Generate content for a glossary reference.
  ///
  /// Port of `CrossRef::generateGlossaryRefTitle` (CrossRef.pm:906-925): the
  /// entry's `phrase:<show>` — failing that, `<base>-plural` appends an `s` and
  /// `<base>-indefinite` prefixes `a`/`an` to `phrase:<base>` — wrapped in an
  /// `ltx_glossary_<show>` text, its content `prepRefText` (the stored node's
  /// trimmed children, cloned: markup survives).
  fn generate_glossary_ref_title(&self, entry_key: &str, show: &str) -> Vec<NodeData> {
    let Some(entry) = self.db.lookup(entry_key) else {
      return vec![];
    };
    let wrap = |children: Vec<NodeData>| {
      vec![NodeData::Element {
        tag: "ltx:text".to_string(),
        attributes: Some(HashMap::from_iter([(
          "class".to_string(),
          format!("ltx_glossary_{show}"),
        )])),
        children,
      }]
    };
    if let Some(val) = entry.get_value(&format!("phrase:{show}")) {
      return wrap(ref_content_children(val));
    }
    if let Some(base) = show.strip_suffix("-plural")
      && let Some(val) = entry.get_value(&format!("phrase:{base}"))
    {
      let mut children = ref_content_children(val);
      children.push(NodeData::Text("s".to_string()));
      return wrap(children);
    }
    if let Some(base) = show.strip_suffix("-indefinite")
      && let Some(val) = entry.get_value(&format!("phrase:{base}"))
    {
      // Perl tests `$phrase->textContent` against `/^[aeiou]/i`.
      let article = if val
        .as_string()
        .starts_with(|c: char| "aeiouAEIOU".contains(c))
      {
        "an "
      } else {
        "a "
      };
      let mut children = vec![NodeData::Text(article.to_string())];
      children.extend(ref_content_children(val));
      return wrap(children);
    }
    vec![]
  }

  /// Copy linked resources (non-idref hrefs) to the destination.
  ///
  /// Port of `CrossRef::copy_resources`.
  fn copy_resources(&self, doc: &PostDocument) {
    let refs = doc.findnodes("//ltx:ref[@href and not(@idref) and not(@labelref)]");
    for ref_node in &refs {
      if let Some(url) = ref_node.get_attribute("href") {
        // Only copy relative URLs (no protocol, not absolute)
        if !url.contains("://") && !url.starts_with('/') {
          // Would copy resource from search path to destination
          log::trace!("CrossRef: would copy resource '{}'", url);
        }
      }
    }
  }

  /// Generate reference content for a given ID and show pattern.
  ///
  /// Port of `CrossRef::generateRef`.
  fn generate_ref(&mut self, _doc: &PostDocument, req_id: &str, req_show: &str) -> Vec<NodeData> {
    let show_options = if !req_show.contains("title") {
      vec![req_show.to_string(), "title".to_string()]
    } else {
      vec![req_show.to_string(), "refnum".to_string()]
    };

    for show in &show_options {
      let mut stuff = Vec::new();
      let mut id = req_id.to_string();
      let mut pending = String::new();
      loop {
        let entry_exists = self.db.lookup(&format!("ID:{}", id)).is_some();
        if !entry_exists {
          break;
        }
        let s = self.generate_ref_aux(&id, show);
        if !s.is_empty() {
          if !pending.is_empty() {
            stuff.push(NodeData::Text(pending.clone()));
          }
          stuff.extend(s);
          if self.check_ref_content(&stuff) {
            return stuff;
          }
          pending = self.ref_join.clone();
        }
        let parent = self
          .db
          .lookup(&format!("ID:{}", id))
          .and_then(|e| e.get_string("parent").map(String::from));
        match parent {
          Some(pid) => id = pid,
          None => break,
        }
      }
      if !stuff.is_empty() {
        return stuff;
      }
    }

    self.note_missing("info", "Usable title for ID", req_id);
    vec![NodeData::Text(req_id.to_string())]
  }

  /// Generate ref content from a single DB entry.
  fn generate_ref_aux(&self, id: &str, show: &str) -> Vec<NodeData> {
    let entry = match self.db.lookup(&format!("ID:{}", id)) {
      Some(e) => e,
      None => return vec![],
    };

    let mut stuff = Vec::new();
    let mut ok = false;
    let mut remaining = show.to_string();

    while !remaining.is_empty() {
      if remaining.starts_with(|c: char| c.is_alphanumeric()) {
        let keyword: String = remaining
          .chars()
          .take_while(|c| c.is_alphanumeric())
          .collect();
        remaining = remaining[keyword.len()..].to_string();
        let key = keyword.to_lowercase();
        let class = if key.contains("title") {
          "ltx_ref_title"
        } else {
          "ltx_ref_tag"
        };

        let mut keys_to_try = vec![key.clone(), format!("tag:{}", key)];
        keys_to_try.extend(ref_fallbacks(&key).iter().map(|s| s.to_string()));

        for k in &keys_to_try {
          if let Some(val) = entry.get_value(k) {
            if val.is_truthy() {
              ok = true;
              // Perl `generateRef_aux` L779: `['ltx:text', {class}, prepRefText]`
              // where `prepRefText` = `cloneNodes(trimChildNodes($value))` — a
              // DEEP CLONE of the title's child nodes, `<ltx:Math>` included.
              // The CrossRef pass runs before the MathML pass, so the cloned
              // `<ltx:Math>` is later turned into `<math>` just like the body
              // copy (issue #356). A plain-string value keeps the flat-text
              // rendering.
              stuff.push(NodeData::Element {
                tag:        "ltx:text".to_string(),
                attributes: Some(HashMap::from_iter([(
                  "class".to_string(),
                  class.to_string(),
                )])),
                children:   ref_content_children(val),
              });
              break;
            }
          }
        }
      } else if remaining.starts_with('{') {
        if let Some(end) = remaining[1..].find('}') {
          let literal = &remaining[1..1 + end];
          if !literal.is_empty() {
            stuff.push(NodeData::Text(literal.to_string()));
          }
          remaining = remaining[2 + end..].to_string();
        } else {
          remaining.clear();
        }
      } else if remaining.starts_with('~') {
        remaining = remaining[1..].to_string();
        if !stuff.is_empty() {
          stuff.push(NodeData::Text("\u{00A0}".to_string()));
        }
      } else if remaining.starts_with(|c: char| c.is_whitespace()) {
        let ws: String = remaining
          .chars()
          .take_while(|c| c.is_whitespace())
          .collect();
        remaining = remaining[ws.len()..].to_string();
        if !stuff.is_empty() {
          stuff.push(NodeData::Text(ws));
        }
      } else {
        let sym: String = remaining
          .chars()
          .take_while(|c| !c.is_alphanumeric() && *c != '{' && *c != '~')
          .collect();
        remaining = remaining[sym.len()..].to_string();
        stuff.push(NodeData::Text(sym));
      }
    }

    if ok { stuff } else { vec![] }
  }

  /// Check if ref content is "good enough".
  fn check_ref_content(&self, stuff: &[NodeData]) -> bool {
    let text = text_content(stuff);
    let cleaned = text.replace("in ", "");
    cleaned.chars().any(|c| c.is_alphanumeric())
  }

  // ======================================================================
  // Fill-in methods

  fn fill_in_relations(&mut self, doc: &mut PostDocument) {
    // Same get_xml_id trick as generate_document_title: Scan stores ids
    // in the xml namespace by default; without this, sub-docs would skip
    // relation filling and never gain the prev/next/up navigation.
    let page_id = match doc.get_document_element().as_ref().and_then(get_xml_id) {
      Some(id) => id,
      None => return,
    };

    // 1. up / "up up" / "up up up" — walk ancestors that have a title.
    let mut current_id = page_id.clone();
    let mut rel = "up".to_string();
    let mut topmost = current_id.clone();
    loop {
      let parent_id = self
        .db
        .lookup(&format!("ID:{}", current_id))
        .and_then(|e| e.get_string("parent").map(String::from));
      match parent_id {
        Some(pid) => {
          let has_title = self
            .db
            .lookup(&format!("ID:{}", pid))
            .and_then(|e| e.get_value("title"))
            .map(|v| v.is_truthy())
            .unwrap_or(false);
          if has_title {
            doc.add_navigation(&rel, &pid);
            rel = format!("{} up", rel);
          }
          current_id = pid.clone();
          topmost = pid;
        },
        None => break,
      }
    }

    // 2. start — the topmost ancestor (root page), if different from us.
    if topmost != page_id {
      if let Some(top_pageid) = self
        .db
        .lookup(&format!("ID:{}", topmost))
        .and_then(|e| e.get_string("pageid").map(String::from))
      {
        doc.add_navigation("start", &top_pageid);
      }
    }

    // 3. prev / next — walk the page tree.
    if let Some(prev) = self.find_previous_page_id(&page_id) {
      doc.add_navigation("prev", &prev);
    }
    if let Some(next) = self.find_next_page_id(&page_id) {
      doc.add_navigation("next", &next);
    }

    // 4. Relation-typed links (Perl CrossRef.pm L105-130). "Dig around for other
    // interesting related documents": the sibling pages of each ancestor (walking
    // up), then this page's own child pages. Each is keyed by the page's own
    // element-name relation (`chapter`/`section`/`subsection`/…) if it is a
    // primary page, else `sidebar`. This is what gives split pages their
    // `rel="chapter"`/`rel="section"`/… head links; the whole block was unported.
    let mut xentry = page_id.clone();
    while let Some(parent) = self.get_parent_page_id(&xentry) {
      for sib in self.child_pages(&parent).ids.iter() {
        if *sib == page_id {
          continue;
        }
        self.add_typed_navigation(doc, sib);
      }
      xentry = parent;
    }
    for child in self.child_pages(&page_id).ids.iter() {
      self.add_typed_navigation(doc, child);
    }
  }

  /// Add a navigation link to `related_id` keyed by its own element-name
  /// relation (Perl: `$type =~ s/^(\w+)://` → `chapter`/`section`/…) when it is
  /// a primary page, else `sidebar`. Port of the per-entry arm of Perl
  /// `CrossRef::fill_in_relations`'s second half.
  fn add_typed_navigation(&self, doc: &mut PostDocument, related_id: &str) {
    if self.is_primary_page(related_id) {
      let rel = self
        .db
        .lookup(&format!("ID:{}", related_id))
        .and_then(|e| e.get_string("type").map(String::from))
        // Strip the namespace prefix: `ltx:chapter` → `chapter`.
        .map(|t| t.rsplit(':').next().unwrap_or(&t).to_string());
      if let Some(rel) = rel.filter(|r| !r.is_empty()) {
        doc.add_navigation(&rel, related_id);
      }
    } else {
      doc.add_navigation("sidebar", related_id);
    }
  }

  /// Return whether the given xml:id is registered as a primary page.
  /// Port of `$entry->getValue('primary')`.
  fn is_primary_page(&self, page_id: &str) -> bool {
    self
      .db
      .lookup(&format!("ID:{}", page_id))
      .and_then(|e| e.get_value("primary"))
      .map(|v| v.is_truthy())
      .unwrap_or(false)
  }

  /// Resolve `entry_id` to the pageid of the page that *contains* its
  /// parent. Port of Perl `CrossRef::getParentPage`.
  fn get_parent_page_id(&self, entry_id: &str) -> Option<String> {
    let entry = self.db.lookup(&format!("ID:{}", entry_id))?;
    let pageid = entry.get_string("pageid")?.to_string();
    let page_entry = self.db.lookup(&format!("ID:{}", pageid))?;
    let parent_id = page_entry.get_string("parent")?.to_string();
    let parent_entry = self.db.lookup(&format!("ID:{}", parent_id))?;
    Some(parent_entry.get_string("pageid")?.to_string())
  }

  /// Memoized `get_child_page_ids`. The ObjectDB is immutable for the whole
  /// CrossRef pass, so a given entry's child-page list is stable and shared
  /// across every page (see the `child_pages` field). Also records each id's
  /// position so the sibling finders skip the Perl pop/shift scan.
  fn child_pages(&self, entry_id: &str) -> Rc<ChildPages> {
    if let Some(cached) = self.child_pages.borrow().get(entry_id) {
      return cached.clone();
    }
    let ids = self.compute_child_page_ids(entry_id);
    let mut index_of = HashMap::default();
    // Last occurrence wins, matching the Perl scan that peels from the end.
    for (i, id) in ids.iter().enumerate() {
      index_of.insert(id.clone(), i);
    }
    let rc = Rc::new(ChildPages { ids, index_of });
    self
      .child_pages
      .borrow_mut()
      .insert(entry_id.to_string(), rc.clone());
    rc
  }

  /// Recursively collect distinct child page ids under `entry_id`.
  /// Port of Perl `CrossRef::getChildPages` (the uncached recursion body;
  /// recursion reuses the cache via [`child_pages`](Self::child_pages)).
  fn compute_child_page_ids(&self, entry_id: &str) -> Vec<String> {
    let entry = match self.db.lookup(&format!("ID:{}", entry_id)) {
      Some(e) => e,
      None => return Vec::new(),
    };
    let here_pageid = entry.get_string("pageid").map(String::from);
    let children = entry.get_children();
    let mut out = Vec::new();
    for ch in children {
      let ch_entry = match self.db.lookup(&format!("ID:{}", ch)) {
        Some(e) => e,
        None => continue,
      };
      let ch_pageid = match ch_entry.get_string("pageid") {
        Some(p) => p.to_string(),
        None => continue,
      };
      if here_pageid.as_deref() != Some(&ch_pageid) {
        out.push(ch_pageid);
      } else {
        out.extend(self.child_pages(&ch).ids.iter().cloned());
      }
    }
    out
  }

  /// Page immediately preceding `page_id` in tree order, restricted to
  /// `primary` pages. Port of Perl `CrossRef::findPreviousPage`: previous
  /// sibling if any, drilled into rightmost descendant.
  fn find_previous_page_id(&self, page_id: &str) -> Option<String> {
    let parent_id = self.get_parent_page_id(page_id)?;
    let siblings = self.child_pages(&parent_id);
    // Our position among the parent's child pages (None = "broken database").
    let pos = *siblings.index_of.get(page_id)?;
    // Nearest primary sibling strictly before us (Perl: peel following sibs,
    // drop self, keep primaries, take the last one). If there is NONE, Perl's
    // `$pentry` is still the PARENT page, so the previous page is the parent
    // itself (e.g. the first `\section` of a `\chapter` → the chapter page).
    // The old `?` returned None here, dropping the `rel="prev"` link entirely.
    let mut current = match siblings.ids[..pos]
      .iter()
      .rev()
      .find(|s| self.is_primary_page(s))
    {
      Some(sib) => sib.clone(),
      None => return Some(parent_id),
    };
    // Drill into the rightmost primary descendant.
    loop {
      let kids = self.child_pages(&current);
      match kids.ids.iter().rev().find(|s| self.is_primary_page(s)) {
        Some(deepest) => current = deepest.clone(),
        None => break,
      }
    }
    Some(current)
  }

  /// Page immediately following `page_id` in tree order, restricted to
  /// `primary` pages. Port of Perl `CrossRef::findNextPage`: first child,
  /// else walk up to find next sibling at progressively higher levels.
  fn find_next_page_id(&self, page_id: &str) -> Option<String> {
    // First primary child page, if any.
    if let Some(first) = self
      .child_pages(page_id)
      .ids
      .iter()
      .find(|s| self.is_primary_page(s))
    {
      return Some(first.clone());
    }
    let mut current = page_id.to_string();
    loop {
      let parent = self.get_parent_page_id(&current)?;
      let siblings = self.child_pages(&parent);
      // Our position among the parent's child pages (None = "broken database").
      let pos = *siblings.index_of.get(&current)?;
      // First primary sibling strictly after us.
      if let Some(first) = siblings.ids[pos + 1..]
        .iter()
        .find(|s| self.is_primary_page(s))
      {
        return Some(first.clone());
      }
      current = parent;
    }
  }

  fn fill_in_tocs(&mut self, doc: &mut PostDocument) {
    // Perl Post.pm L946-948: Document::findnodes defaults the XPath
    // context to documentElement. oxide's `findnodes(None)` defaults to
    // the XML document node, where libxml2's `descendant::` axis evaluates
    // differently — `descendant::ltx:TOC` matches zero from the doc node
    // even though `//ltx:TOC` matches one. Pin the root explicitly so the
    // user's `\tableofcontents` placeholder is reachable.
    let tocs = match doc.get_document_element() {
      Some(root) => doc.findnodes_at("descendant::ltx:TOC[not(ltx:toclist)]", Some(&root)),
      None => Vec::new(),
    };
    for toc in &tocs {
      // Use the unified get_xml_id helper: Scan's `Document` fallback
      // assigns xml:id via the xml namespace, which is invisible to a
      // bare `get_attribute("xml:id")` lookup but is found by
      // `get_attribute_ns("id", XML_NS)` (which get_xml_id tries first).
      let mut id = doc
        .get_document_element()
        .as_ref()
        .and_then(get_xml_id)
        .unwrap_or_default();
      // `scope="global"` retargets the TOC to the root page. Perl
      // fill_in_tocs L227-231 resolves this via `getRootPage`, walking up the
      // *page* hierarchy (parent → its pageid → …) and taking the root page's
      // `pageid`. Default scope (`current` or absent) keeps the current-page id,
      // so the inline `\tableofcontents` placeholder stays page-local.
      if toc.get_attribute("scope").as_deref() == Some("global") {
        id = self.get_root_page_id(&id);
      }
      let show = toc
        .get_attribute("show")
        .unwrap_or_else(|| self.toc_show.clone());

      // Perl CrossRef.pm fill_in_tocs L213-233: the `select` attribute (built
      // by `\tableofcontents` from `tocdepth`) restricts which element types
      // reach the ToC; absent `select` ⇒ no type restriction. The `lists`
      // attribute names which inlist buckets to draw from (default `toc`;
      // `lof`/`lot` for the figure/table lists).
      let select_attr = toc.get_attribute("select");
      let types: Option<HashSet<&str>> = select_attr.as_deref().map(|s| {
        s.split('|')
          .map(str::trim)
          .filter(|t| !t.is_empty())
          .collect()
      });
      let lists_attr = toc.get_attribute("lists");
      let lists: HashSet<&str> = match lists_attr.as_deref() {
        Some(l) => l.split_whitespace().collect(),
        None => HashSet::from_iter(["toc"]),
      };

      // Perl fill_in_tocs L232-236 dispatches on `format`: `normal` (or absent)
      // builds a plain downward TOC; `context` builds the navigation breadcrumb
      // (`gentoc_context`), which forces `lists={toc}`. Any other value yields
      // no toclist (Perl leaves `@list` empty).
      let format = toc.get_attribute("format").unwrap_or_default();
      let list = if format.is_empty() || format.starts_with("normal") {
        self.gen_toc(&id, &show, types.as_ref(), &lists, None, None)
      } else if format == "context" {
        let toc_lists: HashSet<&str> = HashSet::from_iter(["toc"]);
        self.gen_toc_context(&id, &show, types.as_ref(), &toc_lists)
      } else {
        Vec::new()
      };
      if !list.is_empty() {
        let toclist = NodeData::Element {
          tag:        "ltx:toclist".to_string(),
          attributes: None,
          children:   list,
        };
        let mut toc_mut = toc.clone();
        doc.add_nodes(&mut toc_mut, &[toclist]);
      }
    }
  }

  /// Perl CrossRef.pm getRootPage L179-186 + its caller (fill_in_tocs L229):
  /// walk up the *page* hierarchy — `parent` → that parent's `pageid` → that
  /// page's `parent` → … — to the topmost page, and return its `pageid`. For a
  /// single-page document this resolves back to the document id.
  fn get_root_page_id(&self, start_id: &str) -> String {
    let mut root_id = start_id.to_string();
    let mut cursor = start_id.to_string();
    while let Some(page_id) = self.parent_page_of(&cursor) {
      root_id = page_id.clone();
      cursor = page_id;
    }
    // Caller reads `$root->getValue('pageid')`.
    self
      .db
      .lookup(&format!("ID:{}", root_id))
      .and_then(|e| e.get_string("pageid"))
      .map(String::from)
      .unwrap_or(root_id)
  }

  /// One `getRootPage` step (Perl L182-184): the `pageid` of this entry's
  /// parent, provided that page entry exists. `None` ends the upward walk.
  fn parent_page_of(&self, id: &str) -> Option<String> {
    // $x = $x->getValue('parent')
    let parent_id = self
      .db
      .lookup(&format!("ID:{}", id))
      .and_then(|e| e.get_string("parent"))
      .filter(|s| !s.is_empty())?;
    // $x = lookup(parent)->getValue('pageid')
    let page_id = self
      .db
      .lookup(&format!("ID:{}", parent_id))
      .and_then(|e| e.get_string("pageid"))
      .filter(|s| !s.is_empty())?
      .to_string();
    // $x = lookup(pageid) — the page entry must exist to continue.
    self.db.lookup(&format!("ID:{}", page_id)).map(|_| page_id)
  }

  /// Perl CrossRef.pm gentoc L246-262. Generate the TOC for `id` and its
  /// children. `localto` (when `Some`) restricts the downward recursion to
  /// entries on that page's `location` — the mechanism a context TOC uses to
  /// stop at the current page's boundary. `selfid` marks the matching entry
  /// with `ltx_ref_self` ("you are here").
  fn gen_toc(
    &self,
    id: &str,
    show: &str,
    types: Option<&HashSet<&str>>,
    lists: &HashSet<&str>,
    localto: Option<&str>,
    selfid: Option<&str>,
  ) -> Vec<NodeData> {
    let entry = match self.db.lookup(&format!("ID:{}", id)) {
      Some(e) => e,
      None => return vec![],
    };

    // gentoc L250-252: recurse into children only when unrestricted, or this
    // entry lives on the target page.
    let recurse = match localto {
      None => true,
      Some(target) => entry.get_string("location").unwrap_or("") == target,
    };
    let kids: Vec<NodeData> = if recurse {
      entry
        .get_children()
        .iter()
        .flat_map(|child_id| self.gen_toc(child_id, show, types, lists, localto, selfid))
        .collect()
    } else {
      Vec::new()
    };

    let entry_type = entry.get_string("type").unwrap_or("");
    // gentoc L255-256: include this entry iff its type passes the `select`
    // filter (no `select` ⇒ unrestricted) AND its `inlist` shares a list with
    // the TOC's `lists`. This is what makes `\setcounter{tocdepth}` (#291) take
    // effect — the level filter rides on `select`.
    let type_ok = types.map(|t| t.contains(entry_type)).unwrap_or(true);
    let in_toc = entry
      .get_value("inlist")
      .map(|v| match v {
        Value::Hash(h) => lists.iter().any(|l| h.contains_key(*l)),
        _ => false,
      })
      .unwrap_or(false);

    if type_ok && in_toc {
      vec![self.gen_tocentry(entry, selfid, show, kids)]
    } else {
      kids
    }
  }

  /// Perl CrossRef.pm gentocentry L268-283. Build one `ltx:tocentry` for an
  /// entry: the `before < show > after` split (`generateRef_simple` for the
  /// before/after halves), the `ltx:ref` body, the `ltx_ref_self` marker when
  /// this is the `selfid`, and a nested `ltx:toclist` of `children`.
  fn gen_tocentry(
    &self,
    entry: &Entry,
    selfid: Option<&str>,
    show: &str,
    children: Vec<NodeData>,
  ) -> NodeData {
    let id = entry
      .get_string("id")
      .or_else(|| entry.get_key().strip_prefix("ID:"))
      .unwrap_or("")
      .to_string();
    let entry_type = entry.get_string("type").unwrap_or("");
    let type_name = entry_type.strip_prefix("ltx:").unwrap_or(entry_type);

    // gentocentry L272-273: `before < show > after`.
    let (mut before, mut after): (Option<&str>, Option<&str>) = (None, None);
    let mut show_mid = show;
    if let Some((b, rest)) = show_mid.split_once('<') {
      before = Some(b);
      show_mid = rest;
    }
    if let Some((mid, a)) = show_mid.split_once('>') {
      show_mid = mid;
      after = Some(a);
    }

    let self_class = if selfid == Some(id.as_str()) {
      " ltx_ref_self"
    } else {
      ""
    };

    let mut kids: Vec<NodeData> = Vec::new();
    if let Some(b) = before.filter(|b| !b.is_empty()) {
      kids.extend(self.generate_ref_simple(&id, b));
    }
    kids.push(NodeData::Element {
      tag:        "ltx:ref".to_string(),
      attributes: Some(HashMap::from_iter([
        ("show".to_string(), show_mid.to_string()),
        ("idref".to_string(), id.clone()),
      ])),
      children:   vec![],
    });
    if let Some(a) = after.filter(|a| !a.is_empty()) {
      kids.extend(self.generate_ref_simple(&id, a));
    }
    if !children.is_empty() {
      kids.push(NodeData::Element {
        tag: "ltx:toclist".to_string(),
        attributes: Some(HashMap::from_iter([(
          "class".to_string(),
          format!("ltx_toclist_{}", type_name),
        )])),
        children,
      });
    }

    NodeData::Element {
      tag:        "ltx:tocentry".to_string(),
      attributes: Some(HashMap::from_iter([(
        "class".to_string(),
        format!("ltx_tocentry_{}{}", type_name, self_class),
      )])),
      children:   kids,
    }
  }

  /// Perl CrossRef.pm generateRef_simple L...: look the entry up and, if found,
  /// render `req_show` against it. Used only by `gentocentry`'s before/after.
  fn generate_ref_simple(&self, req_id: &str, req_show: &str) -> Vec<NodeData> {
    if !req_show.is_empty()
      && !req_id.is_empty()
      && self.db.lookup(&format!("ID:{}", req_id)).is_some()
    {
      self.generate_ref_aux(req_id, req_show)
    } else {
      Vec::new()
    }
  }

  /// Perl CrossRef.pm gentoc_context L288-311. A "context" TOC: the current
  /// page's own contents (downward, page-local), enclosed upward within its
  /// ancestors and their sibling sections — the navigation-bar breadcrumb.
  fn gen_toc_context(
    &self,
    id: &str,
    show: &str,
    types: Option<&HashSet<&str>>,
    lists: &HashSet<&str>,
  ) -> Vec<NodeData> {
    let start = match self.db.lookup(&format!("ID:{}", id)) {
      Some(e) => e,
      None => return vec![],
    };

    // Downward TOC covering items WITHIN the current page (localto = this page's
    // location; selfid = this id so the current entry is marked ltx_ref_self).
    let location = start.get_string("location").unwrap_or("").to_string();
    let mut navtoc = self.gen_toc(id, show, types, lists, Some(&location), Some(id));

    // Enclose it upward, along with siblings & ancestors. `came_from` is the id
    // of the child we ascended through; its slot in each parent's sibling row is
    // replaced by the accumulated `navtoc` subtree.
    let mut came_from = id.to_string();
    let mut parent_id = start.get_string("parent").map(String::from);

    while let Some(pid) = parent_id {
      let parent = match self.db.lookup(&format!("ID:{}", pid)) {
        Some(e) => e,
        None => break,
      };

      // gentoc_context L297-303: the parent's normal-type children become plain
      // tocentries, except the one we came from (spliced with `navtoc`).
      let mut row: Vec<NodeData> = Vec::new();
      for child_id in parent.get_children() {
        let child = match self.db.lookup(&format!("ID:{}", child_id)) {
          Some(e) => e,
          None => continue,
        };
        if !NORMAL_TOC_TYPES.contains(&child.get_string("type").unwrap_or("")) {
          continue;
        }
        let child_id_val = child.get_string("id").unwrap_or(&child_id);
        if child_id_val == came_from {
          row.append(&mut navtoc);
        } else {
          row.push(self.gen_tocentry(child, None, show, Vec::new()));
        }
      }
      navtoc = row;

      // gentoc_context L304-306: wrap in the parent's own tocentry, but only if
      // the parent passes the type filter AND is itself nested (never wrap the
      // top-level document).
      let parent_type = parent.get_string("type").unwrap_or("");
      let parent_ok = types.map(|t| t.contains(parent_type)).unwrap_or(true);
      let parent_has_parent = parent
        .get_string("parent")
        .map(|s| !s.is_empty())
        .unwrap_or(false);
      if parent_ok && parent_has_parent {
        navtoc = vec![self.gen_tocentry(parent, None, show, navtoc)];
      }

      came_from = pid;
      parent_id = parent.get_string("parent").map(String::from);
    }

    navtoc
  }

  fn fill_in_frags(&self, doc: &PostDocument) {
    // Perl (CrossRef.pm L312-324) walks the page's own `//@xml:id` nodes and
    // sets `fragid` on any that have a DB entry. Iterating the ObjectDB
    // instead (one lookup per DB key) wins ONLY when a single page has far
    // more id-nodes than the DB has entries (math-heavy single documents:
    // ~60K XM* ids vs ~1K DB entries). On a *split* document the DB holds
    // every page's entries (tens of thousands) while each page has a handful
    // of id-nodes, so that inverted loop is O(db_keys) per page = O(n²)
    // overall. Pick whichever loop is bounded by the smaller set — both
    // assign fragids to exactly the same nodes, so the output is identical.
    if doc.idcache_len() <= self.db.len() {
      // Perl semantics: iterate the page's id-nodes (bounded by page size).
      for (id, node) in doc.idcache_iter() {
        if let Some(entry) = self.db.lookup(&format!("ID:{}", id)) {
          if let Some(fragid) = entry.get_string("fragid") {
            let mut n = node.clone();
            n.set_attribute("fragid", fragid).ok();
          }
        }
      }
    } else {
      // Inverted loop: fewer DB entries than page id-nodes. `find_node_by_id`
      // restricts to this page's nodes, so only page-local fragids are set.
      for key in self.db.keys_iter() {
        let id = match key.strip_prefix("ID:") {
          Some(rest) => rest,
          None => continue,
        };
        let entry = match self.db.lookup(key) {
          Some(e) => e,
          None => continue,
        };
        let fragid = match entry.get_string("fragid") {
          Some(f) => f,
          None => continue,
        };
        if let Some(node) = doc.find_node_by_id(id) {
          let mut n = node.clone();
          n.set_attribute("fragid", fragid).ok();
        }
      }
    }
  }

  fn fill_in_refs(&mut self, doc: &mut PostDocument) {
    let mut refs = doc.findnodes("//*[@idref]");
    refs.extend(doc.findnodes("//*[@labelref]"));
    for ref_node in &refs {
      let tag = doc.get_qname(ref_node).unwrap_or_default();
      if tag == "ltx:XMRef" {
        continue;
      }

      let mut ref_mut = ref_node.clone();
      let mut id = ref_node.get_attribute("idref");
      let show = ref_node
        .get_attribute("show")
        .unwrap_or_else(|| self.ref_show.clone());

      if id.is_none() {
        if let Some(label) = ref_node.get_attribute("labelref") {
          if let Some(entry) = self.db.lookup(&label) {
            if let Some(resolved_id) = entry.get_string("id") {
              ref_mut.set_attribute("idref", resolved_id).ok();
              id = Some(resolved_id.to_string());
            }
          }
          if id.is_none() {
            self.note_missing("warn", "Target for Label", &label);
            PostDocument::add_class(&mut ref_mut, "ltx_missing_label");
          }
        }
      }

      if let Some(ref id_str) = id {
        if ref_mut.get_attribute("href").is_none() {
          if let Some(url) = self.generate_url(doc, id_str) {
            ref_mut.set_attribute("href", &url).ok();
          }
        }
        if ref_mut.get_attribute("title").is_none() {
          if let Some(titlestring) = self.generate_title(doc, id_str, &show) {
            ref_mut.set_attribute("title", &titlestring).ok();
          }
          // Perl CrossRef.pm L358-361: a ref carrying a `rel` (a navigation ref)
          // ALSO gets a `fulltitle` — `generateTitle($doc, $id)` with an EMPTY
          // `show`, i.e. the full contextual breadcrumb with NO "In <context>"
          // dup-collapse (that collapse only fires for `show=~/title/`). The XSLT
          // `head-links` template prefers `@fulltitle` over `@title` for the head
          // `<link rel=… title=…>` entries, so without this split pages emitted
          // empty (or "In X") nav-link titles.
          if let Some(rel) = ref_mut.get_attribute("rel") {
            if !rel.is_empty() {
              if let Some(fulltitle) = self.generate_title(doc, id_str, "") {
                ref_mut.set_attribute("fulltitle", &fulltitle).ok();
              }
            }
          }
        }
        if ref_mut.get_first_child().is_none() && tag != "ltx:graphics" && tag != "ltx:picture" {
          let content = self.generate_ref(doc, id_str, &show);
          doc.add_nodes(&mut ref_mut, &content);
        }
      }
    }
  }

  fn fill_in_glossaryrefs(&mut self, doc: &mut PostDocument) {
    // Port of Perl CrossRef.pm:454-482 fill_in_glossaryrefs: resolve each
    // `<ltx:glossaryref key=… inlist=… show=…>` against the GLOSSARY:list:key
    // entry registered by Scan (+ MakeIndex), set `idref` (a later fill_in_refs
    // pass turns it into `href`) and the `title` tooltip, and fill an EMPTY ref
    // with the entry's `show` phrase (acronym.sty's `\ac`/`\acs`/`\acl` emit
    // empty refs: without this they rendered as their key, "NN (NN)").
    for ref_node in &doc.findnodes("descendant::ltx:glossaryref") {
      let mut ref_mut = ref_node.clone();
      let key = ref_node.get_attribute("key").unwrap_or_default();
      let list = ref_node.get_attribute("inlist").unwrap_or_default();
      let show = ref_node
        .get_attribute("show")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "name".to_string());
      let is_empty = |n: &Node| n.get_content().is_empty() && n.get_first_element_child().is_none();

      let gkey = format!("GLOSSARY:{}:{}", list, key);
      let found = self.db.lookup(&gkey).map(|entry| {
        // Perl L464-466: the tooltip is `phrase:definition`'s text (acronym's
        // role); glossaries and nomencl call it `phrase:description`, which the
        // Rust port has always used, so it stands as the fallback. The text is
        // `value_text`'s whitespace-collapsed, math-aware form (issue #761),
        // where Perl copies the raw `textContent`.
        let title = entry
          .get_value("phrase:definition")
          .or_else(|| entry.get_value("phrase:description"))
          .map(|val| value_text(doc, val))
          .filter(|t| !t.is_empty());
        (title, entry.get_string("id").map(str::to_string))
      });
      if let Some((title, id)) = found {
        if ref_mut.get_attribute("title").is_none()
          && let Some(title) = title
        {
          ref_mut.set_attribute("title", &title).ok();
        }
        if let Some(id) = id {
          ref_mut.set_attribute("idref", &id).ok();
        }
        if is_empty(&ref_mut) {
          let stuff = self.generate_glossary_ref_title(&gkey, &show);
          if stuff.is_empty() {
            self.note_missing("warn", &format!("Glossary contents ({show}) for key"), &key);
            doc.add_nodes(&mut ref_mut, &[NodeData::Text(key.clone())]);
            PostDocument::add_class(&mut ref_mut, "ltx_missing");
          } else {
            doc.add_nodes(&mut ref_mut, &stuff);
          }
        }
      } else {
        self.note_missing("warn", "Glossary Entry for key", &key);
      }

      if is_empty(&ref_mut) {
        doc.add_nodes(&mut ref_mut, &[NodeData::Text(key.clone())]);
        PostDocument::add_class(&mut ref_mut, "ltx_missing");
      }
    }
  }

  /// Resolve the RDFa subject/object references — `aboutidref`/`aboutlabelref`
  /// into `about`, and `resourceidref`/`resourcelabelref` into `resource`.
  ///
  /// Port of `CrossRef.pm::fill_in_RDFa_refs` (L372-398), and runs in Perl's
  /// position: after `fill_in_refs`, before `fill_in_bibrefs`.
  ///
  /// `lxRDFa.sty` deliberately records an intra-document RDFa subject as an
  /// `…idref`/`…labelref` pair rather than a URL, because the URL is not knowable
  /// until the document has been split and paginated — see the
  /// `LaTeXML-common.rnc` L301 note, "it will be converted to `aboutidref` and
  /// `about` during post-processing". Without this pass that conversion never
  /// happened, so `\lxRDFa{about=#thm1}` produced an `aboutidref` that no
  /// consumer reads and **no `about` at all** — the RDFa triple lost its subject.
  /// Visible on math once `outer_wrapper` began copying `about` onto `<m:math>`:
  /// Perl emits `about="#thm1"` there and Rust emitted nothing.
  ///
  /// An id that the ObjectDB knows becomes a real (possibly cross-page) URL via
  /// `generate_url`; an id it does not know still becomes a bare `#id` fragment,
  /// because — as Perl's comment puts it — "RDF 'id' need not be real, valid,
  /// ids!!!": an author may name a subject that is not a document node at all.
  ///
  /// Perl also re-runs `set_RDFa_prefixes` at the end. Not ported, and it cannot
  /// matter here: this pass only ever writes `about`/`resource` values that are
  /// absolute URLs or `#id` fragments, never prefixed CURIEs, so there is no new
  /// prefix to declare. (Prefix management itself already happens core-side, in
  /// `latexml_core::document::set_rdfa_prefixes`, as it does in Perl's
  /// `Core/Document.pm:366`.)
  fn fill_in_rdfa_refs(&mut self, doc: &mut PostDocument) {
    for key in ["about", "resource"] {
      // One query with `or`, as Perl has it — two queries concatenated would
      // visit a node carrying BOTH attributes twice and in the wrong order.
      let refs = doc.findnodes(&format!("//*[@{key}idref or @{key}labelref]"));
      for ref_node in &refs {
        let mut ref_mut = ref_node.clone();
        let idref_attr = format!("{key}idref");
        // Perl's `if (!$id)` and `if ($id)` are truth tests, so an empty
        // `aboutidref=""` counts as absent rather than resolving to `about="#"`.
        let mut id = ref_node
          .get_attribute(&idref_attr)
          .filter(|v| !v.is_empty());

        // A label reference resolves through the ObjectDB to an id, which is
        // written back so the `if let Some(id)` below treats both spellings
        // alike (Perl L379-387).
        if id.is_none()
          && let Some(label) = ref_node.get_attribute(&format!("{key}labelref"))
        {
          if let Some(entry) = self.db.lookup(&label)
            && let Some(resolved) = entry.get_string("id")
          {
            ref_mut.set_attribute(&idref_attr, resolved).ok();
            id = Some(resolved.to_string());
          }
          if id.is_none() {
            self.note_missing("warn", &format!("Target for {key} Label"), &label);
          }
        }

        // Never overwrite an `about`/`resource` the author gave outright.
        if let Some(ref id_str) = id
          && ref_mut.get_attribute(key).is_none()
        {
          let value = if self.db.lookup(&format!("ID:{id_str}")).is_some() {
            self.generate_url(doc, id_str)
          } else {
            Some(format!("#{id_str}"))
          };
          if let Some(value) = value {
            ref_mut.set_attribute(key, &value).ok();
          }
        }
      }
    }
  }

  /// Port of Perl `CrossRef::fill_in_bibrefs` (`CrossRef.pm` L486-493): every
  /// `<ltx:bibref>` is replaced by the citation
  /// [`make_bibcite`](Self::make_bibcite) builds for it — by nothing, when that
  /// is empty.
  fn fill_in_bibrefs(&mut self, doc: &mut PostDocument) {
    let bibrefs = doc.findnodes("//ltx:bibref");
    for bibref in &bibrefs {
      let cite = self.make_bibcite(doc, bibref);
      doc.replace_node(bibref, &cite);
    }
  }

  /// Port of Perl `CrossRef::make_bibcite` (`CrossRef.pm` L495-644): the links
  /// to a bibref's keys, each labelled by the bibref's `show` pattern filled
  /// with that entry's bibliography data (see [`bibcite_show_walk`]). Every link
  /// carries the entry's title as its `title` (L530-539, L555-557): the
  /// tooltip naming the work behind a bare `[1]`.
  fn make_bibcite(&mut self, doc: &PostDocument, bibref: &Node) -> Vec<NodeData> {
    let keys_str = bibref.get_attribute("bibrefs").unwrap_or_default();
    let keys: Vec<&str> = keys_str.split(',').filter(|k| !k.is_empty()).collect();
    let preformatted = child_nodes_data(bibref);
    let mut show = bibref
      .get_attribute("show")
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| "refnum".to_string());
    if show == "none" && preformatted.is_empty() {
      show = "refnum".to_string();
    }
    // Perl CrossRef.pm:507-508: `\nocite`'s bibref (`show='nothing'`,
    // latex_constructs.pool.ltxml:4214) only marks its keys for the
    // bibliography stage; `make_bibcite` returns nothing for it, so the node
    // is replaced by nothing. Filling it in printed the keys — for
    // `\nocite{*}` a visible `*` as a missing citation plus its warning.
    if show == "nothing" {
      return Vec::new();
    }
    let attr_or_comma = |name: &str| {
      bibref
        .get_attribute(name)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ",".to_string())
    };
    let sep = attr_or_comma("separator");
    let yysep = attr_or_comma("yyseparator");
    // The `<ltx:bibrefphrase>`s that `phraseN` selects.
    let phrases = crate::document::element_children(bibref);
    // The lists to search, most-specific first. Perl CrossRef.pm L515 reads
    // `inlist || 'bibliography'` — an *exclusive* choice, which strands every
    // citation of a document that loads `bibunits`/`chapterbib` but keeps a
    // single main `\bibliography`: `\cite` stamps CITE_UNIT=bu0 onto the
    // bibref, while the bibitems register under the default `bibliography`
    // list, so the unit-only lookup never matches (witness 2303.06077: 93
    // bibitems, 93 dangling keys, 0 links).
    //
    // Perl's own Scan.pm L379-380 spells the intended chain — unit lists
    // PLUS the main one ("Citation specifies main 'bibliography', as well as
    // any specific others (eg. per chapter)") — and registers the reference
    // under both. We follow Scan's convention here so the two agree; the unit
    // list still wins, since the search breaks on the first list that yields
    // an id. OXIDIZED_DESIGN #59, KNOWN_PERL_ERRORS #50.
    let inlist = bibref.get_attribute("inlist").unwrap_or_default();
    let mut lists: Vec<&str> = inlist.split_whitespace().collect();
    if !lists.contains(&"bibliography") {
      lists.push("bibliography");
    }
    // Each key's bibitem id: from the first list that has it.
    let ids: Vec<Option<String>> = keys
      .iter()
      .map(|key| {
        lists.iter().find_map(|list| {
          self
            .db
            .lookup(&format!("BIBLABEL:{}:{}", list, key))
            .and_then(|be| be.get_string("id"))
            .map(String::from)
        })
      })
      .collect();
    // \NAT@force@numbers (natbib): a numeric `.bbl` — plain `\bibitem{key}`
    // with no `[author(year)]` label — forces numbers mode globally, so every
    // `\cite` prints the bracketed number `[N]`/`[N, M]` even when a numeric
    // `\bibliographystyle{unsrt}` sits AFTER the cites (witness arXiv:2308.06262
    // / html_feedback#62). Single-pass LaTeXML froze this bibref's author-year
    // `show`; Perl `CrossRef.pm:542` keeps it because its `|| $keytag` guard is
    // always satisfied, so both engines render the raw key. When natbib's frozen
    // show (its capitalized `Authors`/`Year` keywords) wants author-year yet
    // EVERY cited entry is numeric-only (has a number, no real author/year),
    // collapse to natbib's numeric form. SURPASS-PERL: OXIDIZED_DESIGN #123,
    // KNOWN_PERL_ERRORS #89.
    let show_wants_ay = show.contains("Author") || show.contains("Year");
    let force_numeric = show_wants_ay
      && !ids.is_empty()
      && ids.iter().all(|id| {
        let Some(id) = id else {
          return false;
        };
        match self.db.lookup(&format!("ID:{}", id)) {
          Some(e) => {
            let nonempty = |k: &str| {
              e.get_value(k)
                .is_some_and(|v| !v.to_string().trim().is_empty())
            };
            !nonempty("authors")
              && !nonempty("fullauthors")
              && !nonempty("year")
              && (nonempty("number") || nonempty("refnum"))
          },
          None => false,
        }
      });
    // Do the frozen author-year delimiters sit INSIDE the bibref (a Phrase
    // after Year — `\cite`/`\citet` carry their own `( )`) or as sibling text
    // (`\citep`, whose macro adds the parens outside the bibref)? Only bracket
    // our numeric group in the former case, else `\citep`'s parens double up.
    let internal_delims = show
      .find("Year")
      .is_some_and(|yp| show[yp + "Year".len()..].contains("Phrase"));

    // Collect all the data from the bibliography (L516-562).
    static SUFFIXED_YEAR: LazyLock<Regex> =
      LazyLock::new(|| Regex::new(r"^(\d\d\d\d)(\w)$").unwrap());
    let mut data: Vec<BibciteDatum> = Vec::new();
    for (key, id) in keys.iter().zip(ids) {
      let found = id.and_then(|id| {
        let entry = self.db.lookup(&format!("ID:{}", id))?;
        let val = |k: &str| entry.get_value(k).filter(|v| !matches!(v, Value::Null));
        let (authors, fauthors, keytag) = (val("authors"), val("fullauthors"), val("keytag"));
        let (year, typetag, title) = (val("year"), val("typetag"), val("title"));
        let titlestring = title
          .map(|t| {
            t.to_string()
              .split_whitespace()
              .collect::<Vec<_>>()
              .join(" ")
          })
          .filter(|s| !s.is_empty());
        let (rawyear, suffix) = year
          .and_then(|y| {
            let text = y.to_string();
            SUFFIXED_YEAR
              .captures(&text)
              .map(|c| (Some(c[1].to_string()), Some(c[2].to_string())))
          })
          .unwrap_or_default();
        // Disable the author-year format (L542) for an entry with nothing to
        // name it by. Perl's `$show` persists: the whole citation turns refnum.
        if !(show == "none" || authors.is_some() || fauthors.is_some() || keytag.is_some()) {
          show = "refnum".to_string();
        }
        let mut attr = HashMap::default();
        attr.insert("idref".to_string(), id.clone());
        if let Some(title) = titlestring {
          attr.insert("title".to_string(), title);
        }
        Some((id, BibciteDatum {
          key: key.to_string(),
          authors: trim_child_nodes(authors.or(fauthors).or(keytag)),
          fullauthors: trim_child_nodes(fauthors.or(authors).or(keytag)),
          authortext: authors
            .or(fauthors)
            .map(Value::to_string)
            .unwrap_or_default(),
          year: trim_child_nodes(year.or(typetag)),
          rawyear,
          suffix,
          number: trim_child_nodes(val("number")),
          refnum: trim_child_nodes(val("refnum")),
          title: trim_child_nodes(title.or(keytag)),
          attr,
          missing: false,
        }))
      });
      match found {
        Some((id, mut datum)) => {
          if let Some(url) = self.generate_url(doc, &id) {
            datum.attr.insert("href".to_string(), url);
          }
          data.push(datum);
        },
        None => {
          self.note_missing("warn", "Entry for citation", key);
          data.push(BibciteDatum {
            key:         key.to_string(),
            authors:     Vec::new(),
            fullauthors: Vec::new(),
            authortext:  String::new(),
            year:        Vec::new(),
            rawyear:     None,
            suffix:      None,
            number:      Vec::new(),
            refnum:      vec![NodeData::Text(key.to_string())],
            title:       vec![NodeData::Text(key.to_string())],
            attr:        HashMap::from_iter([
              ("idref".to_string(), key.to_string()),
              ("title".to_string(), key.to_string()),
              ("class".to_string(), "ltx_missing_citation".to_string()),
            ]),
            missing:     true,
          });
        },
      }
    }

    let mut refs: Vec<NodeData> = Vec::new();
    if force_numeric {
      // Numeric collapse (#123): each entry's number, joined with ", " (natbib
      // numbers mode), bracketed as a group ([1, 2]) unless `\citep`'s external
      // parens already enclose it.
      for datum in &data {
        if !refs.is_empty() {
          refs.push(NodeData::Text(", ".to_string()));
        }
        let label = [&datum.number, &datum.refnum]
          .into_iter()
          .find(|l| !l.is_empty())
          .cloned()
          .unwrap_or_else(|| vec![NodeData::Text(datum.key.clone())]);
        refs.push(bib_ref(&datum.attr, label));
      }
      if internal_delims && !refs.is_empty() {
        refs.insert(0, NodeData::Text("[".to_string()));
        refs.push(NodeData::Text("]".to_string()));
      }
      return refs;
    }
    let lower = show.to_lowercase();
    let checkdups =
      lower.contains("author") && (lower.contains("year") || lower.contains("number"));
    let mut data: VecDeque<BibciteDatum> = data.into();
    while let Some(datum) = data.pop_front() {
      let (stuff, didref) = bibcite_show_walk(
        &show,
        &datum,
        &mut data,
        &preformatted,
        &phrases,
        &yysep,
        checkdups,
      );
      if !refs.is_empty() {
        refs.push(NodeData::Text(sep.clone()));
        refs.push(NodeData::Text(" ".to_string()));
      }
      if didref {
        refs.extend(stuff);
      } else {
        refs.push(bib_ref(&datum.attr, stuff));
      }
    }
    refs
  }

  fn fill_in_mathlinks(&mut self, doc: &PostDocument) {
    for sym in &doc.findnodes("descendant::*[@decl_id or @meaning]") {
      let tag = doc.get_qname(sym).unwrap_or_default();
      if tag == "ltx:XMRef" || sym.get_attribute("href").is_some() {
        continue;
      }
      let entry_key = sym
        .get_attribute("decl_id")
        .map(|did| format!("DECLARATION:local:{}", did))
        .or_else(|| {
          sym
            .get_attribute("meaning")
            .map(|m| format!("DECLARATION:global:{}", m))
        });
      let parent_id = entry_key
        .as_ref()
        .and_then(|ek| self.db.lookup(ek))
        .and_then(|entry| entry.get_string("parent").map(String::from));
      if let Some(pid) = parent_id {
        if let Some(url) = self.generate_url(doc, &pid) {
          let mut sym_mut = sym.clone();
          sym_mut.set_attribute("href", &url).ok();
        }
      }
    }
  }

  fn report_missing(&self) {
    for (severity, types) in &self.missing {
      for (ref_type, items) in types {
        let keys: Vec<&String> = items.keys().collect();
        let msg = format!(
          "Missing {}: {}",
          ref_type,
          keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(",")
        );
        // Perl CrossRef.pm L72-75: structured Error/Warn/Info with
        // class='expected', object='ids'. Use harness-friendly target.
        match severity.as_str() {
          "error" => Error!("expected", "ids", "{}", msg),
          "warn" => Warn!("expected", "ids", "{}", msg),
          _ => Info!("expected", "ids", "{}", msg),
        }
      }
    }
  }
}

impl Processor for CrossRef {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    match doc.get_document_element() {
      Some(el) => vec![el],
      None => vec![],
    }
  }

  fn process(&mut self, mut doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    self.missing.clear();

    // Generate document title and add navigation
    let doc_title = self.generate_document_title(&doc);
    let navtoc = self.navigation_toc.clone();

    if (navtoc.is_some() || doc_title.is_some()) && doc.findnode("//ltx:navigation").is_none() {
      if let Some(mut root) = doc.get_document_element() {
        doc.add_nodes(&mut root, &[NodeData::Element {
          tag:        "ltx:navigation".to_string(),
          attributes: None,
          children:   vec![],
        }]);
      }
    }
    if let Some(ref format) = navtoc {
      if let Some(mut nav) = doc.findnode("//ltx:navigation") {
        // Perl CrossRef.pm L50: `['ltx:TOC', {format => $navtoc}]` — format
        // ONLY, no `scope`. `fill_in_tocs` then defaults scope to `current`, so
        // this TOC is built relative to THIS page's document element. On a split
        // document that yields a per-page navigation breadcrumb via
        // `gen_toc_context` (each page's own contents enclosed within its
        // ancestors + their siblings), rather than one identical global tree.
        doc.add_nodes(&mut nav, &[NodeData::Element {
          tag:        "ltx:TOC".to_string(),
          attributes: Some(HashMap::from_iter([("format".to_string(), format.clone())])),
          children:   vec![],
        }]);
      }
    }
    if let Some(ref title) = doc_title {
      if let Some(mut nav) = doc.findnode("//ltx:navigation") {
        doc.add_nodes(&mut nav, &[NodeData::Element {
          tag:        "ltx:title".to_string(),
          attributes: None,
          children:   vec![NodeData::Text(title.clone())],
        }]);
      }
    }

    self.fill_in_relations(&mut doc);
    self.fill_in_tocs(&mut doc);
    self.fill_in_frags(&doc);
    self.fill_in_glossaryrefs(&mut doc);
    self.fill_in_refs(&mut doc);
    self.fill_in_rdfa_refs(&mut doc);
    self.fill_in_bibrefs(&mut doc);
    self.fill_in_mathlinks(&doc);
    self.copy_resources(&doc);
    strip_ref_display_fragids(&doc);
    self.report_missing();
    Ok(vec![doc])
  }
}

// ======================================================================
// Helpers

fn relative_url(target: &str, base: &str) -> String {
  if target == base {
    return ".".to_string();
  }
  let target_parts: Vec<&str> = target.split('/').collect();
  let base_parts: Vec<&str> = base.split('/').collect();
  let common = target_parts
    .iter()
    .zip(base_parts.iter())
    .take_while(|(a, b)| a == b)
    .count();
  let mut result = String::new();
  for _ in common..base_parts.len().saturating_sub(1) {
    result.push_str("../");
  }
  result.push_str(&target_parts[common..].join("/"));
  if result.is_empty() {
    ".".to_string()
  } else {
    result
  }
}

/// Get text content from an XML node, normalizing whitespace.
///
/// Port of `getTextContent`.
fn get_text_content_node(node: &Node) -> String {
  let text = node.get_content();
  let trimmed = text.trim();
  // Normalize whitespace
  trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn text_content(nodes: &[NodeData]) -> String {
  nodes
    .iter()
    .map(|n| match n {
      NodeData::Text(s) => s.clone(),
      NodeData::Element { children, .. } => text_content(children),
      NodeData::XmlNode(n) => n.get_content(),
    })
    .collect::<Vec<_>>()
    .join("")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_url_identical_paths_is_dot() {
    assert_eq!(relative_url("a/b.html", "a/b.html"), ".");
  }

  #[test]
  fn relative_url_same_dir() {
    // Both live under a/, so target becomes simply the sibling filename.
    assert_eq!(relative_url("a/other.html", "a/index.html"), "other.html");
  }

  #[test]
  fn relative_url_sibling_dir() {
    // From a/index.html to b/x.html: up once, then down.
    assert_eq!(relative_url("b/x.html", "a/index.html"), "../b/x.html");
  }

  #[test]
  fn relative_url_deeply_nested_base() {
    // Up past each intermediate dir of the base, then into the new path.
    assert_eq!(
      relative_url("top/sibling.html", "top/deep/nested/page.html"),
      "../../sibling.html"
    );
  }

  #[test]
  fn relative_url_same_prefix_different_file() {
    assert_eq!(
      relative_url("a/b/c/target.html", "a/b/c/source.html"),
      "target.html"
    );
  }

  #[test]
  fn ref_fallbacks_typerefnum_goes_to_refnum() {
    assert_eq!(ref_fallbacks("typerefnum"), &["refnum"]);
  }

  #[test]
  fn ref_fallbacks_title_chain() {
    assert_eq!(ref_fallbacks("title"), &["toccaption"]);
    assert_eq!(ref_fallbacks("toctitle"), &["title", "toccaption"]);
    assert_eq!(ref_fallbacks("rawtoctitle"), &[
      "toctitle",
      "title",
      "toccaption"
    ]);
    assert_eq!(ref_fallbacks("rawtitle"), &["title", "toccaption"]);
  }

  #[test]
  fn ref_fallbacks_unknown_key_is_empty() {
    let empty: &[&str] = &[];
    assert_eq!(ref_fallbacks("nonexistent"), empty);
    assert_eq!(ref_fallbacks(""), empty);
  }

  #[test]
  fn text_content_flattens_text() {
    let nodes = vec![
      NodeData::Text("hello ".to_string()),
      NodeData::Text("world".to_string()),
    ];
    assert_eq!(text_content(&nodes), "hello world");
  }

  #[test]
  fn text_content_recurses_into_elements() {
    let nodes = vec![NodeData::Element {
      tag:        "span".to_string(),
      attributes: None,
      children:   vec![
        NodeData::Text("inner ".to_string()),
        NodeData::Text("text".to_string()),
      ],
    }];
    assert_eq!(text_content(&nodes), "inner text");
  }

  #[test]
  fn text_content_empty_list_is_empty_string() {
    assert_eq!(text_content(&[]), "");
  }

  #[test]
  fn text_content_mixed_text_and_nested_element() {
    let nodes = vec![
      NodeData::Text("outer ".to_string()),
      NodeData::Element {
        tag:        "em".to_string(),
        attributes: None,
        children:   vec![NodeData::Text("inner".to_string())],
      },
      NodeData::Text(" tail".to_string()),
    ];
    assert_eq!(text_content(&nodes), "outer inner tail");
  }

  // --- URL-style transform (Perl CrossRef::generateURL L656-663) -------------

  #[test]
  fn url_style_file_is_identity() {
    // `file` keeps the full path untouched, whatever it is.
    for url in ["a/b.html", "index.html", "sub/index.html", "index", ""] {
      assert_eq!(apply_url_style(url, UrlStyle::File, "html"), url);
    }
  }

  #[test]
  fn url_style_server_strips_trailing_index() {
    // At start → "./"; after a slash → keep the directory (with slash).
    assert_eq!(
      apply_url_style("index.html", UrlStyle::Server, "html"),
      "./"
    );
    assert_eq!(
      apply_url_style("dir/index.html", UrlStyle::Server, "html"),
      "dir/"
    );
    assert_eq!(
      apply_url_style("a/b/index.html", UrlStyle::Server, "html"),
      "a/b/"
    );
    // A non-index page is untouched.
    assert_eq!(
      apply_url_style("dir/page.html", UrlStyle::Server, "html"),
      "dir/page.html"
    );
    // Boundary: `index.html` NOT preceded by start-or-`/` must NOT be stripped
    // (Perl's `(^|\/)`); the old `ends_with` check wrongly mangled this.
    assert_eq!(
      apply_url_style("myindex.html", UrlStyle::Server, "html"),
      "myindex.html"
    );
  }

  #[test]
  fn url_style_negotiated_strips_extension_and_index() {
    // Extension goes; a plain page keeps its stem.
    assert_eq!(
      apply_url_style("dir/page.html", UrlStyle::Negotiated, "html"),
      "dir/page"
    );
    // Trailing `index` after the extension strip: at start → ""; after "/" → keep dir.
    assert_eq!(
      apply_url_style("index.html", UrlStyle::Negotiated, "html"),
      ""
    );
    assert_eq!(
      apply_url_style("dir/index.html", UrlStyle::Negotiated, "html"),
      "dir/"
    );
    // Boundary: a bare `index` NOT after start-or-`/` stays (Perl's `(^|\/)index$`).
    assert_eq!(
      apply_url_style("myindex.html", UrlStyle::Negotiated, "html"),
      "myindex"
    );
    // Honors a non-html extension.
    assert_eq!(
      apply_url_style("dir/index.xml", UrlStyle::Negotiated, "xml"),
      "dir/"
    );
  }
}
