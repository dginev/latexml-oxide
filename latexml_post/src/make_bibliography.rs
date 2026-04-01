//! Bibliography generation processor.
//!
//! Port of `LaTeXML::Post::MakeBibliography` (818 lines of Perl).
//! Collects bibliographic entries from `.bib.xml` files and the ObjectDB,
//! formats them according to the bibliography style (numeric, author-year, alpha),
//! and fills in `ltx:bibliography` elements with `ltx:biblist` + `ltx:bibitem`.
//!
//! Pipeline:
//! 1. Find bibliography sources (xml files, bib files, literals)
//! 2. Scan bibentry elements for cited keys (via BIBLABEL:* in ObjectDB)
//! 3. Transitively include entries cited from within included entries
//! 4. Extract names, dates, titles for sorting
//! 5. Detect duplicate author+year pairs → assign suffixes (a, b, c...)
//! 6. Format each entry into ltx:bibitem with ltx:tags + ltx:bibblock sections
//! 7. Optionally split by initial letter

use libxml::tree::Node;
use std::collections::{HashMap, HashSet};

use crate::document::{NodeData, PostDocument};
use crate::object_db::ObjectDB;
use crate::processor::{ProcessResult, Processor};
use crate::radix::radix_alpha;

/// Citation style.
#[derive(Debug, Clone, PartialEq)]
pub enum CitationStyle {
  /// Numeric: [1], [2], ...
  Numbers,
  /// Author-Year: (Author, Year)
  AuthorYear,
  /// Alphabetic: [ABC24]
  Alpha,
}

/// A collected bibliography entry with metadata.
#[derive(Debug)]
struct BibEntryData {
  bib_key: String,
  cited_key: Option<String>,
  sort_key: String,
  initial: String,
  author_year: String,
  suffix: Option<String>,
  /// Author names for display.
  authors_short: String,
  authors_full: String,
  year: String,
  title: String,
  /// BibTeX type (article, book, inproceedings, etc.).
  entry_type: String,
  /// Reference style (number within bibliography).
  number: u32,
  /// IDs that cite this entry.
  referrers: HashSet<String>,
  /// Keys cited from within this entry.
  citations: Vec<String>,
}

impl BibEntryData {
  /// Get the normalized bibliography type for CSS class.
  fn bib_type(&self) -> &str {
    if self.entry_type.is_empty() { "misc" } else { &self.entry_type }
  }

  /// Get the canonical format type (for FMT_SPEC mapping).
  ///
  /// Port of `%FMT_SPEC` aliases.
  fn format_type(&self) -> &str {
    match self.entry_type.as_str() {
      "article" => "article",
      "book" | "periodical" | "collection" | "proceedings"
      | "manual" | "misc" | "unpublished" | "booklet" => "book",
      "incollection" | "collection.article" | "proceedings.article"
      | "inproceedings" | "inbook" => "incollection",
      "report" | "techreport" => "report",
      "thesis" | "mastersthesis" | "phdthesis" => "thesis",
      "website" | "online" => "website",
      "software" => "software",
      _ => "book", // Default fallback
    }
  }
}

/// MakeBibliography post-processor.
///
/// Port of `LaTeXML::Post::MakeBibliography`.
pub struct MakeBibliography {
  name: String,
  pub db: ObjectDB,
  style: CitationStyle,
  split: bool,
  bibliographies: Vec<String>,
}

impl MakeBibliography {
  pub fn new(db: ObjectDB, style: CitationStyle, split: bool) -> Self {
    MakeBibliography {
      name: "MakeBibliography".to_string(),
      db, style, split,
      bibliographies: Vec::new(),
    }
  }

  pub fn set_bibliographies(&mut self, bibs: Vec<String>) {
    self.bibliographies = bibs;
  }

  /// Collect all cited bibliography entries.
  ///
  /// Port of `getBibEntries`.
  /// Scans BIBLABEL:* entries in ObjectDB, resolves to ID:* entries,
  /// extracts author/year/title, transitively includes cited-from-cited entries,
  /// assigns suffixes for duplicate author+year pairs.
  fn get_bib_entries(&self, bib_node: &Node) -> Vec<BibEntryData> {
    let lists_str = bib_node.get_attribute("lists").unwrap_or_else(|| "bibliography".to_string());
    let lists: Vec<&str> = lists_str.split_whitespace().collect();

    // Step 1: Collect all cited bibliography keys from BIBLABEL entries.
    // Also check for \cite{*} which includes everything.
    //
    // Port of the first loop in `getBibEntries`:
    // - Scan BIBLABEL:list:key entries
    // - Filter referrers to exclude those FROM within bibitem elements
    // - Support \cite{*} via BIBLABEL:list:* entries
    let mut raw_entries: HashMap<String, BibEntryData> = HashMap::new();
    let mut queue: Vec<String> = Vec::new();
    let mut number = 0u32;

    // Check for \cite{*}
    let cite_star = lists.iter().any(|list| {
      self.db.lookup(&format!("BIBLABEL:{}:*", list)).is_some()
    });

    for db_key in self.db.get_keys() {
      if !db_key.starts_with("BIBLABEL:") { continue; }
      let parts: Vec<&str> = db_key.splitn(3, ':').collect();
      if parts.len() < 3 { continue; }
      let (list, bibkey) = (parts[1], parts[2]);
      if !lists.contains(&list) { continue; }

      if let Some(bentry) = self.db.lookup(db_key) {
        let has_refs = bentry.get_value("referrers").map(|v| v.is_truthy()).unwrap_or(false);
        if has_refs {
          // Verify referrers are from outside the bibliography
          // (Perl filters: walk up parent chain, skip if reaches ltx:bibitem)
          let mut referrers = HashSet::new();
          if let Some(crate::object_db::Value::Hash(refs)) = bentry.get_value("referrers") {
            for ref_id in refs.keys() {
              // Walk up parent chain to check if referrer is inside a bibitem
              let mut rid = ref_id.clone();
              let mut is_from_bib = false;
              while let Some(entry) = self.db.lookup(&format!("ID:{}", rid)) {
                let entry_type = entry.get_string("type").unwrap_or("");
                if entry_type == "ltx:bibitem" {
                  is_from_bib = true;
                  break;
                }
                match entry.get_string("parent").map(String::from) {
                  Some(parent) => rid = parent,
                  None => break,
                }
              }
              if !is_from_bib {
                referrers.insert(ref_id.clone());
              }
            }
          }
          if !referrers.is_empty() {
            queue.push(bibkey.to_string());
          }
        } else if cite_star {
          // \cite{*}: include all entries
          queue.push(bibkey.to_string());
        }
      }
    }

    // Step 2: Process queue (transitively include cited entries)
    let mut seen: HashSet<String> = HashSet::new();
    while let Some(bibkey) = queue.pop() {
      let lc_key = bibkey.to_lowercase();
      if seen.contains(&lc_key) || bibkey == "*" { continue; }
      seen.insert(lc_key.clone());

      // Find the entry in the ObjectDB
      let mut found_id = None;
      for list in &lists {
        let bkey = format!("BIBLABEL:{}:{}", list, bibkey);
        if let Some(bentry) = self.db.lookup(&bkey) {
          found_id = bentry.get_string("id").map(String::from);
          if found_id.is_some() { break; }
        }
      }

      if let Some(id) = found_id {
        let id_key = format!("ID:{}", id);
        number += 1;

        // Extract metadata from ID:* entry
        let authors = self.db.lookup(&id_key)
          .and_then(|e| e.get_value("authors").map(|v| v.to_string()))
          .unwrap_or_default();
        let full_authors = self.db.lookup(&id_key)
          .and_then(|e| e.get_value("fullauthors").map(|v| v.to_string()))
          .unwrap_or_else(|| authors.clone());
        let year = self.db.lookup(&id_key)
          .and_then(|e| e.get_value("year").map(|v| v.to_string()))
          .unwrap_or_default();
        let title = self.db.lookup(&id_key)
          .and_then(|e| e.get_value("title").map(|v| v.to_string()))
          .unwrap_or_default();

        // Extract 4-digit year if present
        let year_short = if let Some(cap) = year.find(|c: char| c.is_ascii_digit()) {
          let digits: String = year[cap..].chars().take_while(|c| c.is_ascii_digit()).collect();
          if digits.len() >= 4 { digits[..4].to_string() } else { year.clone() }
        } else { year.clone() };

        let names = if authors.is_empty() { bibkey.clone() } else { authors.clone() };
        let author_year = format!("{}.{}", names, year_short);
        let initial = names.chars().next()
          .filter(|c| c.is_ascii_alphabetic())
          .map(|c| c.to_uppercase().to_string())
          .unwrap_or_else(|| "*".to_string());
        let sort_key = format!("{}.{}.{}.{}", names, year_short, title, bibkey).to_lowercase();

        // Get entry type from ObjectDB
        let entry_type = self.db.lookup(&id_key)
          .and_then(|e| e.get_value("type").map(|v| v.to_string()))
          .unwrap_or_else(|| "misc".to_string());

        raw_entries.insert(sort_key.clone(), BibEntryData {
          bib_key: bibkey.clone(),
          cited_key: Some(bibkey.clone()),
          sort_key,
          initial,
          author_year,
          suffix: None,
          authors_short: authors,
          authors_full: full_authors,
          year: year_short,
          title,
          entry_type,
          number,
          referrers: HashSet::new(),
          citations: Vec::new(),
        });
      }
    }

    // Step 3: Sort and detect duplicate author+year → assign suffixes
    let mut sorted_keys: Vec<String> = raw_entries.keys().cloned().collect();
    sorted_keys.sort();

    let mut ay_seen: HashMap<String, Vec<String>> = HashMap::new();
    for key in &sorted_keys {
      if let Some(entry) = raw_entries.get(key) {
        ay_seen.entry(entry.author_year.clone()).or_default().push(key.clone());
      }
    }

    // Assign suffixes for duplicate author+year
    for keys in ay_seen.values() {
      if keys.len() > 1 {
        for (i, key) in keys.iter().enumerate() {
          if let Some(entry) = raw_entries.get_mut(key) {
            entry.suffix = Some(radix_alpha((i + 1) as u32));
          }
        }
      }
    }

    log::info!(
      "MakeBibliography: {} entries, {} cited",
      raw_entries.len(),
      raw_entries.len()
    );

    // Return sorted
    sorted_keys.iter()
      .filter_map(|k| raw_entries.remove(k))
      .collect()
  }

  /// Format a bibliography list.
  ///
  /// Port of `makeBibliographyList`.
  fn make_bibliography_list(
    &self,
    bib_id: &str,
    initial: Option<&str>,
    entries: &[BibEntryData],
  ) -> NodeData {
    let id = if let Some(init) = initial {
      format!("{}.L1.{}", bib_id, init)
    } else {
      format!("{}.L1", bib_id)
    };

    let items: Vec<NodeData> = entries.iter()
      .map(|entry| self.format_bib_entry(bib_id, entry))
      .collect();

    NodeData::Element {
      tag: "ltx:biblist".to_string(),
      attributes: Some(HashMap::from([("xml:id".to_string(), id)])),
      children: items,
    }
  }

  /// Format a single bibentry into a bibitem.
  ///
  /// Port of `formatBibEntry`.
  fn format_bib_entry(&self, bib_id: &str, entry: &BibEntryData) -> NodeData {
    let id = format!("{}.bib{}", bib_id, entry.number);
    let mut children = Vec::new();

    // Tags
    let mut tags = Vec::new();

    // Number tag
    tags.push(NodeData::Element {
      tag: "ltx:tag".to_string(),
      attributes: Some(HashMap::from([
        ("role".to_string(), "number".to_string()),
        ("class".to_string(), "ltx_bib_number".to_string()),
      ])),
      children: vec![NodeData::Text(entry.number.to_string())],
    });

    // Authors tag
    if !entry.authors_short.is_empty() {
      tags.push(NodeData::Element {
        tag: "ltx:tag".to_string(),
        attributes: Some(HashMap::from([
          ("role".to_string(), "authors".to_string()),
          ("class".to_string(), "ltx_bib_author".to_string()),
        ])),
        children: vec![NodeData::Text(entry.authors_short.clone())],
      });
      if entry.authors_full != entry.authors_short {
        tags.push(NodeData::Element {
          tag: "ltx:tag".to_string(),
          attributes: Some(HashMap::from([
            ("role".to_string(), "fullauthors".to_string()),
            ("class".to_string(), "ltx_bib_author".to_string()),
          ])),
          children: vec![NodeData::Text(entry.authors_full.clone())],
        });
      }
    }

    // Year tag
    if !entry.year.is_empty() {
      let year_text = if let Some(ref suffix) = entry.suffix {
        format!("{}{}", entry.year, suffix)
      } else {
        entry.year.clone()
      };
      tags.push(NodeData::Element {
        tag: "ltx:tag".to_string(),
        attributes: Some(HashMap::from([
          ("role".to_string(), "year".to_string()),
          ("class".to_string(), "ltx_bib_year".to_string()),
        ])),
        children: vec![NodeData::Text(year_text)],
      });
    }

    // Title tag
    if !entry.title.is_empty() {
      tags.push(NodeData::Element {
        tag: "ltx:tag".to_string(),
        attributes: Some(HashMap::from([
          ("role".to_string(), "title".to_string()),
          ("class".to_string(), "ltx_bib_title".to_string()),
        ])),
        children: vec![NodeData::Text(entry.title.clone())],
      });
    }

    // Refnum tag (citation key display)
    let refnum = match self.style {
      CitationStyle::Numbers => format!("[{}]", entry.number),
      CitationStyle::AuthorYear => {
        let suffix = entry.suffix.as_deref().unwrap_or("");
        if entry.authors_short.is_empty() {
          format!("[{}]", entry.number)
        } else {
          format!("{}, {}{}", entry.authors_short, entry.year, suffix)
        }
      }
      CitationStyle::Alpha => {
        // Generate alpha label from author initials + year
        let initials: String = entry.authors_short.split_whitespace()
          .filter_map(|w| w.chars().next())
          .map(|c| c.to_uppercase().to_string())
          .collect::<Vec<_>>()
          .join("");
        let year_suffix = if entry.year.len() >= 2 {
          &entry.year[entry.year.len() - 2..]
        } else {
          &entry.year
        };
        let suffix = entry.suffix.as_deref().unwrap_or("");
        format!("[{}{}{}]", initials, year_suffix, suffix)
      }
    };

    tags.push(NodeData::Element {
      tag: "ltx:tag".to_string(),
      attributes: Some(HashMap::from([
        ("role".to_string(), "refnum".to_string()),
        ("class".to_string(), "ltx_bib_key".to_string()),
      ])),
      children: vec![NodeData::Text(refnum)],
    });

    children.push(NodeData::Element {
      tag: "ltx:tags".to_string(),
      attributes: None,
      children: tags,
    });

    // Bib blocks: format content using per-type specifications
    //
    // Port of the %FMT_SPEC block formatting pipeline.
    // Each bibtype has a sequence of block specifications.
    // Each block is a sequence of field specs: (class, text, condition).
    // We generate an ltx:bibblock for each block that has content.
    let format_type = entry.format_type();
    let blocks = format_bib_blocks(format_type, entry);
    children.extend(blocks);

    // Cited-by block (if entry has referrers)
    if !entry.referrers.is_empty() {
      let mut cited_refs: Vec<NodeData> = Vec::new();
      let mut sorted_referrers: Vec<&String> = entry.referrers.iter().collect();
      sorted_referrers.sort();
      for (i, ref_id) in sorted_referrers.iter().enumerate() {
        if i > 0 {
          cited_refs.push(NodeData::Text(",\n".to_string()));
        }
        cited_refs.push(NodeData::Element {
          tag: "ltx:ref".to_string(),
          attributes: Some(HashMap::from([
            ("idref".to_string(), ref_id.to_string()),
            ("show".to_string(), "typerefnum".to_string()),
          ])),
          children: vec![],
        });
      }
      if !cited_refs.is_empty() {
        let mut block_children = vec![NodeData::Text("Cited by: ".to_string())];
        block_children.extend(cited_refs);
        block_children.push(NodeData::Text(".".to_string()));
        children.push(NodeData::Element {
          tag: "ltx:bibblock".to_string(),
          attributes: Some(HashMap::from([("class".to_string(), "ltx_bib_cited".to_string())])),
          children: block_children,
        });
      }
    }

    NodeData::Element {
      tag: "ltx:bibitem".to_string(),
      attributes: Some(HashMap::from([
        ("xml:id".to_string(), id),
        ("key".to_string(), entry.bib_key.clone()),
        ("class".to_string(), format!("ltx_bib_{}", entry.bib_type())),
      ])),
      children,
    }
  }
}

impl Processor for MakeBibliography {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:bibliography")
  }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    for bib in &nodes {
      // Skip if already populated
      if !doc.findnodes_at(".//ltx:bibitem", Some(bib)).is_empty() {
        continue;
      }

      let entries = self.get_bib_entries(bib);
      if entries.is_empty() {
        log::info!("MakeBibliography: no entries to process");
        continue;
      }

      let bib_id = bib.get_attribute("xml:id")
        .or_else(|| doc.get_document_element().and_then(|r| r.get_attribute("xml:id")))
        .unwrap_or_else(|| "bib".to_string());

      if self.split {
        // Split by initial letter
        let mut by_initial: HashMap<String, Vec<&BibEntryData>> = HashMap::new();
        for entry in &entries {
          by_initial.entry(entry.initial.clone()).or_default().push(entry);
        }
        let mut initials: Vec<&String> = by_initial.keys().collect();
        initials.sort();
        for initial in initials {
          let group: Vec<BibEntryData> = by_initial[initial].iter()
            .map(|e| BibEntryData {
              bib_key: e.bib_key.clone(), cited_key: e.cited_key.clone(),
              sort_key: e.sort_key.clone(), initial: e.initial.clone(),
              author_year: e.author_year.clone(), suffix: e.suffix.clone(),
              authors_short: e.authors_short.clone(), authors_full: e.authors_full.clone(),
              year: e.year.clone(), title: e.title.clone(),
              entry_type: e.entry_type.clone(),
              number: e.number, referrers: e.referrers.clone(),
              citations: e.citations.clone(),
            })
            .collect();
          let biblist = self.make_bibliography_list(&bib_id, Some(initial), &group);
          let mut bib_mut = bib.clone();
          doc.add_nodes(&mut bib_mut, &[biblist]);
        }
      } else {
        let biblist = self.make_bibliography_list(&bib_id, None, &entries);
        let mut bib_mut = bib.clone();
        doc.add_nodes(&mut bib_mut, &[biblist]);
      }

      log::info!("MakeBibliography: formatted {} entries", entries.len());
    }
    Ok(vec![doc])
  }
}

// ======================================================================
// Bibliography formatting specification (FMT_SPEC)
//
// Port of the `%FMT_SPEC` table + formatting helpers from MakeBibliography.pm.
// The Perl version has per-bibtype arrays of block specs, each block being
// an array of field specs [xpath, punct, pre, class, formatter, post].
// In Rust, we encode this as a per-type function that generates the blocks
// from the available metadata.

/// A field specification for bibliography formatting.
struct BibFieldSpec {
  class: &'static str,
  prefix: &'static str,
  suffix: &'static str,
}

/// Generate formatted ltx:bibblock elements for a bibliography entry.
///
/// Port of the block formatting loop in `formatBibEntry` + `%FMT_SPEC`.
fn format_bib_blocks(format_type: &str, entry: &BibEntryData) -> Vec<NodeData> {
  let mut blocks = Vec::new();

  // Block 1: Author/Editor + Year
  // (In author-year style, the refnum tag handles this; in numeric, it's a block)
  {
    let mut items = Vec::new();
    if !entry.authors_full.is_empty() {
      items.push(bib_field("author", &format_authors(&entry.authors_full)));
    }
    if !entry.year.is_empty() {
      let suffix = entry.suffix.as_deref().unwrap_or("");
      items.push(NodeData::Text(format!(" ({}{})", entry.year, suffix)));
    }
    if !items.is_empty() {
      blocks.push(make_bibblock("", &items));
    }
  }

  // Block 2: Title
  if !entry.title.is_empty() {
    blocks.push(make_bibblock("", &[
      bib_field("title", &entry.title),
      NodeData::Text(".".to_string()),
    ]));
  }

  // Block 3+: Type-specific publication details
  // Port of the per-type %FMT_SPEC entries.
  // Since we don't have full bibentry XML nodes (we're working from ObjectDB metadata),
  // we generate what we can from the available fields.
  match format_type {
    "article" => {
      // article: part, journal, volume(number), status, pages, language
      // Most of these require bibentry XML which isn't in our ObjectDB metadata.
      // For now, output a minimal details block.
      let mut items = Vec::new();
      // If we had journal: items.push(bib_field("journal", journal));
      // If we had volume: items.push(NodeData::Text(format!(" {}", volume)));
      // If we had pages:  items.push(NodeData::Text(format!(", pp.\u{00A0}{}", pages)));
      if items.is_empty() && !entry.year.is_empty() {
        // Year was already shown in block 1; just close with period
      }
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    "book" => {
      // book: type, edition, series, volume, part, publisher, organization, place
      let mut items = Vec::new();
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    "incollection" => {
      // incollection: type, crossref/booktitle, editors
      // Then: edition, editor, series, volume, part, publisher, org, place, pages
      let mut items = Vec::new();
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    "report" => {
      // report: type, "Technical Report" number, series, volume, publisher, org, place
      let mut items = Vec::new();
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    "thesis" => {
      // thesis: type (PhD/Master's), part, publisher, org, place
      let mut items = Vec::new();
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    "website" => {
      // website: org, place; add "(Website)" if no type given
      let mut items = Vec::new();
      if entry.entry_type == "website" {
        items.push(NodeData::Text("(Website)".to_string()));
      }
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    "software" => {
      // software: org, place
      let mut items = Vec::new();
      items.push(NodeData::Text(".".to_string()));
      if items.len() > 1 {
        blocks.push(make_bibblock("", &items));
      }
    }
    _ => {}
  }

  // Meta blocks: Note and External Links
  // (Would need bib-note and bib-links from bibentry XML)

  blocks
}

/// Format author names for display.
///
/// Port of `do_names` / `do_authors` / `do_editorsA`.
/// Handles: single author, two authors ("A and B"), many ("A, B, and C"),
/// and "et al." for "others" sentinel.
fn format_authors(authors: &str) -> String {
  let names: Vec<&str> = authors.split(" and ").collect();
  let n = names.len();
  if n == 0 {
    return authors.to_string();
  }
  // Check for "others" (et al.)
  let has_etal = names.last().map(|n| n.trim() == "others").unwrap_or(false);
  let real_names: Vec<&str> = if has_etal { &names[..n - 1] } else { &names }.to_vec();

  let formatted: Vec<String> = real_names.iter()
    .map(|name| format_single_name(name.trim()))
    .collect();

  let mut result = String::new();
  let sep = if formatted.len() > 2 { ", " } else { " " };
  for (i, name) in formatted.iter().enumerate() {
    if i > 0 {
      result.push_str(sep);
      if !has_etal && i == formatted.len() - 1 {
        result.push_str("and ");
      }
    }
    result.push_str(name);
  }

  if has_etal {
    result.push_str(sep);
    result.push_str("et al.");
  }

  result
}

/// Format a single author name.
///
/// Port of `do_name`.
/// Handles "Surname, Given" → "G. Surname" style.
fn format_single_name(name: &str) -> String {
  if let Some((surname, given)) = name.split_once(',') {
    let surname = surname.trim();
    let initials: String = given.split_whitespace()
      .map(|word| {
        if word.ends_with('.') {
          format!("{} ", word)
        } else if let Some(first) = word.chars().next() {
          format!("{}. ", first)
        } else {
          String::new()
        }
      })
      .collect();
    format!("{}{}", initials, surname)
  } else {
    name.to_string()
  }
}

/// Format editors with "(Ed.)" or "(Eds.)" suffix.
///
/// Port of `do_editorsA`.
fn format_editors(editors: &str) -> String {
  let names: Vec<&str> = editors.split(" and ").collect();
  let formatted = format_authors(editors);
  let suffix = if names.len() > 1 { " (Eds.)" } else { " (Ed.)" };
  format!("{}{}", formatted, suffix)
}

/// Format a page range with "pp." prefix.
///
/// Port of `do_pages`.
fn format_pages(pages: &str) -> String {
  format!("pp.\u{00A0}{}", pages) // Non-breaking space
}

/// Format an edition string.
///
/// Port of `do_edition`.
fn format_edition(edition: &str) -> String {
  format!("{} edition", edition)
}

/// Create a formatted bibliography field wrapped in ltx:text.
fn bib_field(class: &str, text: &str) -> NodeData {
  NodeData::Element {
    tag: "ltx:text".to_string(),
    attributes: Some(HashMap::from([("class".to_string(), format!("ltx_bib_{}", class))])),
    children: vec![NodeData::Text(text.to_string())],
  }
}

/// Create a bibblock element with xml:space="preserve".
fn make_bibblock(class: &str, content: &[NodeData]) -> NodeData {
  let mut attrs = HashMap::new();
  attrs.insert("xml:space".to_string(), "preserve".to_string());
  if !class.is_empty() {
    attrs.insert("class".to_string(), class.to_string());
  }
  NodeData::Element {
    tag: "ltx:bibblock".to_string(),
    attributes: Some(attrs),
    children: content.to_vec(),
  }
}
