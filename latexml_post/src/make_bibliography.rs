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

use std::path::Path;

use libxml::tree::Node;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  document::{NodeData, PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::{ProcessResult, Processor},
  radix::radix_alpha,
};

/// Citation style.
#[derive(Debug, Clone, PartialEq)]
pub enum CitationStyle {
  /// Numeric: [1], [2], ...
  Numbers,
  /// Author-Year: Author (Year)
  AuthorYear,
  /// Alphabetic: [ABC24]
  Alpha,
}

/// A collected bibliography entry with metadata.
///
/// Port of the `%entries` hash entries in `getBibEntries`.
#[derive(Debug)]
struct BibEntryData {
  bib_key:       String,
  cited_key:     Option<String>,
  sort_key:      String,
  initial:       String,
  author_year:   String,
  suffix:        Option<String>,
  /// Author names for display (short form: "Smith et al").
  authors_short: String,
  /// Full author names.
  authors_full:  String,
  /// Sort-form of author names.
  sort_names:    String,
  year:          String,
  title:         String,
  /// BibTeX type (article, book, inproceedings, etc.).
  entry_type:    String,
  /// Reference style (number within bibliography).
  number:        u32,
  /// IDs that cite this entry (from outside bibliography).
  referrers:     HashSet<String>,
  /// Bib keys that cite this entry (from other bib entries).
  bibreferrers:  HashSet<String>,
  /// Keys cited from within this entry.
  citations:     Vec<String>,
  /// The bibentry XML node (from .bib.xml), if available.
  bibentry:      Option<Node>,
}

impl BibEntryData {
  /// Get the normalized bibliography type for CSS class.
  fn bib_type(&self) -> &str {
    if self.entry_type.is_empty() {
      "misc"
    } else {
      &self.entry_type
    }
  }

  /// Get the canonical format type (for FMT_SPEC mapping).
  ///
  /// Port of `%FMT_SPEC` aliases.
  fn format_type(&self) -> &str {
    match self.entry_type.as_str() {
      "article" => "article",
      "book" | "periodical" | "collection" | "proceedings" | "manual" | "misc" | "unpublished"
      | "booklet" => "book",
      "incollection"
      | "collection.article"
      | "proceedings.article"
      | "inproceedings"
      | "inbook" => "incollection",
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
  name:           String,
  pub db:         ObjectDB,
  split:          bool,
  bibliographies: Vec<String>,
}

impl MakeBibliography {
  pub fn new(db: ObjectDB, split: bool) -> Self {
    MakeBibliography {
      name: "MakeBibliography".to_string(),
      db,
      split,
      bibliographies: Vec::new(),
    }
  }

  pub fn set_bibliographies(&mut self, bibs: Vec<String>) { self.bibliographies = bibs; }

  /// Load bibliography source documents.
  ///
  /// Port of `getBibliographies`.
  /// Locates .bib.xml files from:
  ///   - Command-line options (overrides)
  ///   - //ltx:bibliography[@files] attribute
  fn get_bibliographies(&self, doc: &PostDocument) -> Vec<PostDocument> {
    let mut bibnames: Vec<String> = Vec::new();
    let mut from_bibliography = false;

    // Use command-line bibliographies if explicitly given
    if !self.bibliographies.is_empty() {
      bibnames = self.bibliographies.clone();
    } else {
      // Otherwise, read from the bibliography element's files attribute
      if let Some(bibnode) = doc.findnode("//ltx:bibliography") {
        let files = bibnode
          .get_attribute("files")
          .or_else(|| bibnode.get_parent().and_then(|p| p.get_attribute("files")));
        if let Some(f) = files {
          from_bibliography = true;
          bibnames = f.split(',').map(|s| s.trim().to_string()).collect();
        }
      }
    }

    let search_paths = doc.get_search_paths();
    let mut bibs: Vec<PostDocument> = Vec::new();

    for bib in &bibnames {
      let mut loaded = false;

      // Try as .xml file
      if bib.ends_with(".xml") {
        if let Some(path) = find_file(bib, search_paths) {
          match PostDocument::new_from_file(&path, PostDocumentOptions {
            source_directory: Some(".".to_string()),
            ..PostDocumentOptions::default()
          }) {
            Ok(bibdoc) => {
              bibs.push(bibdoc);
              loaded = true;
            },
            Err(e) => Warn!("I/O", bib, "Failed to load bibliography '{}': {}", bib, e),
          }
        }
      }
      // Try as .bib or from \bibliography command
      else if bib.ends_with(".bib") || bib.ends_with(".bib.xml") || from_bibliography {
        let xmlbib = if from_bibliography && !bib.ends_with(".bib") {
          format!("{}.bib", bib)
        } else {
          bib.clone()
        };
        // Look for pre-compiled .bib.xml
        let xml_candidate = if xmlbib.ends_with(".xml") {
          xmlbib.clone()
        } else {
          format!("{}.xml", xmlbib)
        };
        if let Some(path) = find_file(&xml_candidate, search_paths) {
          match PostDocument::new_from_file(&path, PostDocumentOptions {
            source_directory: Some(".".to_string()),
            ..PostDocumentOptions::default()
          }) {
            Ok(bibdoc) => {
              bibs.push(bibdoc);
              loaded = true;
            },
            Err(e) => Warn!("I/O", path, "Failed to load bibliography '{}': {}", path, e),
          }
        }
      }

      // If not loaded yet, try raw .bib file and convert it
      if !loaded {
        let bib_file = if from_bibliography && !bib.ends_with(".bib") {
          format!("{}.bib", bib)
        } else {
          bib.clone()
        };
        if let Some(bib_path) = find_file(&bib_file, search_paths) {
          match convert_bib_file_to_xml(&bib_path) {
            Ok(bibdoc) => {
              bibs.push(bibdoc);
              loaded = true;
            },
            Err(e) => Warn!(
              "bibliography",
              "convert",
              "Failed to convert bibliography '{}': {}",
              bib_path,
              e
            ),
          }
        }
      }

      if !loaded {
        Info!(
          "bibliography",
          "missing",
          "Couldn't find usable bibliography for '{}'",
          bib
        );
      }
    }

    Info!(
      "bibliography",
      "using",
      "MakeBibliography: using {} bibliographies",
      bibs.len()
    );
    bibs
  }

  /// Collect all cited bibliography entries.
  ///
  /// Port of `getBibEntries`.
  /// Scans BIBLABEL:* entries in ObjectDB, resolves to ID:* entries,
  /// extracts author/year/title, transitively includes cited-from-cited entries,
  /// assigns suffixes for duplicate author+year pairs.
  fn get_bib_entries(
    &self,
    doc: &PostDocument,
    bib_node: &Node,
  ) -> (HashMap<String, BibEntryData>, Vec<PostDocument>) {
    let lists_str = bib_node
      .get_attribute("lists")
      .unwrap_or_else(|| "bibliography".to_string());
    let lists: Vec<&str> = lists_str.split_whitespace().collect();

    // Step 1: Scan bibliography source documents for ltx:bibentry elements.
    // Build a map: lc(bibkey) → { bibkey, bibentry, citations }
    // Import bibentry nodes into the main document so that XPath queries
    // (which use the main document's namespace context) work correctly.
    let mut entries: HashMap<String, BibEntryData> = HashMap::default();
    let bib_docs = self.get_bibliographies(doc);
    for bibdoc in &bib_docs {
      for bibentry in bibdoc.findnodes("//ltx:bibentry") {
        let bibkey = match bibentry.get_attribute("key") {
          Some(k) => k,
          None => continue,
        };
        let lc_key = bibkey.to_lowercase();
        // Extract citations from within this bibentry
        let citations: Vec<String> = bibdoc
          .findnodes_at(".//@bibrefs", Some(&bibentry))
          .iter()
          .filter_map(|n| {
            let val = n.get_content();
            if val.is_empty() { None } else { Some(val) }
          })
          .flat_map(|s| s.split(',').map(String::from).collect::<Vec<_>>())
          .filter(|s| !s.is_empty())
          .collect();

        let imported = bibentry.clone();

        entries.insert(lc_key, BibEntryData {
          bib_key: bibkey,
          cited_key: None,
          sort_key: String::new(),
          initial: String::new(),
          author_year: String::new(),
          suffix: None,
          authors_short: String::new(),
          authors_full: String::new(),
          sort_names: String::new(),
          year: String::new(),
          title: String::new(),
          entry_type: String::new(),
          number: 0,
          referrers: HashSet::default(),
          bibreferrers: HashSet::default(),
          citations,
          bibentry: Some(imported),
        });
      }
    }

    // Step 2: Collect all cited bibliography keys from BIBLABEL entries in ObjectDB.
    // Note referrers (from outside the bibliography).
    let cite_star = lists
      .iter()
      .any(|list| self.db.lookup(&format!("BIBLABEL:{}:*", list)).is_some());

    let mut queue: Vec<String> = Vec::new();
    for db_key in self.db.get_keys() {
      if !db_key.starts_with("BIBLABEL:") {
        continue;
      }
      let parts: Vec<&str> = db_key.splitn(3, ':').collect();
      if parts.len() < 3 {
        continue;
      }
      let (list, bibkey) = (parts[1], parts[2]);
      if !lists.contains(&list) {
        continue;
      }

      let lc_key = bibkey.to_lowercase();
      if let Some(bentry) = self.db.lookup(db_key) {
        let has_refs = bentry
          .get_value("referrers")
          .map(|v| v.is_truthy())
          .unwrap_or(false);
        if has_refs {
          // Filter referrers: walk up parent chain, skip if inside ltx:bibitem
          if let Some(crate::object_db::Value::Hash(refs)) = bentry.get_value("referrers") {
            for ref_id in refs.keys() {
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
                // Check for case mismatch
                if let Some(existing) = entries.get(&lc_key) {
                  if let Some(ref prev_key) = existing.cited_key {
                    if prev_key != bibkey {
                      Warn!(
                        "bibliography",
                        "case_mismatch",
                        "Case mismatch in bib key '{}' vs '{}'",
                        prev_key,
                        bibkey
                      );
                    }
                  }
                }
                let entry = entries
                  .entry(lc_key.clone())
                  .or_insert_with(|| BibEntryData {
                    bib_key:       bibkey.to_string(),
                    cited_key:     None,
                    sort_key:      String::new(),
                    initial:       String::new(),
                    author_year:   String::new(),
                    suffix:        None,
                    authors_short: String::new(),
                    authors_full:  String::new(),
                    sort_names:    String::new(),
                    year:          String::new(),
                    title:         String::new(),
                    entry_type:    String::new(),
                    number:        0,
                    referrers:     HashSet::default(),
                    bibreferrers:  HashSet::default(),
                    citations:     Vec::new(),
                    bibentry:      None,
                  });
                entry.cited_key = Some(bibkey.to_string());
                entry.referrers.insert(ref_id.clone());
              }
            }
          }
          if entries
            .get(&lc_key)
            .map(|e| !e.referrers.is_empty())
            .unwrap_or(false)
          {
            queue.push(bibkey.to_string());
          }
        } else if cite_star {
          queue.push(bibkey.to_string());
        }
      }
    }

    // Step 3: Process queue — transitively include cited entries.
    // For each key, extract names/year/title from bibentry XML.
    let mut seen: HashSet<String> = HashSet::default();
    let mut included: HashMap<String, BibEntryData> = HashMap::default();
    let mut missing_keys: Vec<String> = Vec::new();

    while let Some(bibkey) = queue.pop() {
      if seen.contains(&bibkey) || bibkey == "*" {
        continue;
      }
      seen.insert(bibkey.clone());
      let lc_key = bibkey.to_lowercase();

      match entries.remove(&lc_key) {
        Some(mut entry) => {
          // Extract metadata from bibentry XML node (if available)
          if let Some(ref bibentry) = entry.bibentry {
            let (sort_names, short_names, _full_names) = extract_names(doc, bibentry);
            entry.sort_names = sort_names.clone();
            entry.authors_short = short_names;

            let names_for_sort = if sort_names.is_empty() {
              // Try bib-key or bib-title as fallback
              match PostDocument::findnodes_foreign("ltx:bib-key", bibentry)
                .into_iter()
                .next()
              {
                Some(key_node) => key_node.get_content(),
                _ => {
                  match PostDocument::findnodes_foreign("ltx:bib-title", bibentry)
                    .into_iter()
                    .next()
                  {
                    Some(title_node) => title_node.get_content(),
                    _ => bibkey.clone(),
                  }
                },
              }
            } else {
              sort_names
            };

            // Year
            let date_content =
              PostDocument::findnodes_foreign("ltx:bib-date[@role='publication']", bibentry)
                .into_iter()
                .next()
                .map(|n| n.get_content())
                .unwrap_or_default();
            let year = extract_four_digit_year(&date_content);
            entry.year = year.clone();

            // Title
            let title = PostDocument::findnodes_foreign("ltx:bib-title", bibentry)
              .into_iter()
              .next()
              .map(|n| n.get_content())
              .unwrap_or_default();
            entry.title = title.clone();

            // Type
            let entry_type = bibentry
              .get_attribute("type")
              .unwrap_or_else(|| "misc".to_string());
            entry.entry_type = entry_type;

            // Author+year for suffix detection
            entry.author_year = format!("{}.{}", names_for_sort, year);
            entry.initial = PostDocument::initial(&names_for_sort, true);

            // Sort key
            let sort_key =
              format!("{}.{}.{}.{}", names_for_sort, year, title, bibkey).to_lowercase();
            entry.sort_key = sort_key.clone();

            // Enqueue transitive citations
            let citations = entry.citations.clone();
            for c in &citations {
              queue.push(c.clone());
            }
            included.insert(sort_key, entry);
          } else {
            // No bibentry XML — use ObjectDB metadata
            let id = self.find_bib_id(&bibkey, &lists);
            if let Some(id) = id {
              let id_key = format!("ID:{}", id);
              let authors = self
                .db
                .lookup(&id_key)
                .and_then(|e| e.get_value("authors").map(|v| v.to_string()))
                .unwrap_or_default();
              let full_authors = self
                .db
                .lookup(&id_key)
                .and_then(|e| e.get_value("fullauthors").map(|v| v.to_string()))
                .unwrap_or_else(|| authors.clone());
              let year = self
                .db
                .lookup(&id_key)
                .and_then(|e| e.get_value("year").map(|v| v.to_string()))
                .unwrap_or_default();
              let title = self
                .db
                .lookup(&id_key)
                .and_then(|e| e.get_value("title").map(|v| v.to_string()))
                .unwrap_or_default();
              let entry_type = self
                .db
                .lookup(&id_key)
                .and_then(|e| e.get_value("type").map(|v| v.to_string()))
                .unwrap_or_else(|| "misc".to_string());

              let year_short = extract_four_digit_year(&year);
              let names = if authors.is_empty() {
                bibkey.clone()
              } else {
                authors.clone()
              };
              let author_year = format!("{}.{}", names, year_short);
              let initial = PostDocument::initial(&names, true);
              let sort_key =
                format!("{}.{}.{}.{}", names, year_short, title, bibkey).to_lowercase();

              entry.authors_short = authors;
              entry.authors_full = full_authors;
              entry.sort_names = names;
              entry.year = year_short;
              entry.title = title;
              entry.entry_type = entry_type;
              entry.author_year = author_year;
              entry.initial = initial;
              entry.sort_key = sort_key.clone();

              included.insert(sort_key, entry);
            } else {
              missing_keys.push(bibkey);
            }
          }
        },
        _ => {
          // Not found in entries map
          missing_keys.push(bibkey);
        },
      }
    }

    if !missing_keys.is_empty() {
      Warn!(
        "bibliography",
        "missing_keys",
        "Missing bibkeys: {}",
        missing_keys.join(", ")
      );
    }

    // Step 4: Note bibreferrers — for each included entry's citations,
    // mark the cited entry as having this entry as a bibreferrer.
    let citations_map: Vec<(String, Vec<String>)> = included
      .values()
      .map(|e| (e.bib_key.clone(), e.citations.clone()))
      .collect();
    for (bibkey, citations) in &citations_map {
      for cited in citations {
        let lc = cited.to_lowercase();
        // Find which sort_key corresponds to this lc bibkey
        for entry in included.values_mut() {
          if entry.bib_key.to_lowercase() == lc {
            entry.bibreferrers.insert(bibkey.clone());
          }
        }
      }
    }

    Info!(
      "bibliography",
      "count",
      "MakeBibliography: {} bibentries, {} cited",
      entries.len() + included.len(),
      included.len()
    );

    // Step 5: Sort and detect duplicate author+year → assign suffixes.
    let mut sorted_keys: Vec<String> = included.keys().cloned().collect();
    sorted_keys.sort();

    // Port of suffix detection: track by author_year, assign suffixes when duplicated.
    let mut ay_last: HashMap<String, String> = HashMap::default(); // ay → last sort_key with this ay
    for key in &sorted_keys {
      if let Some(entry) = included.get(key) {
        let ay = entry.author_year.clone();
        if let Some(prev_key) = ay_last.get(&ay) {
          let prev_key = prev_key.clone();
          // Previous entry with same ay needs a suffix too
          if let Some(prev) = included.get_mut(&prev_key) {
            if prev.suffix.is_none() {
              prev.suffix = Some(radix_alpha(1));
            }
          }
          let prev_counter = included
            .get(&prev_key)
            .and_then(|p| p.suffix.as_ref())
            .map(|s| suffix_to_counter(s))
            .unwrap_or(1);
          if let Some(e) = included.get_mut(key) {
            e.suffix = Some(radix_alpha(prev_counter + 1));
          }
        }
        ay_last.insert(ay, key.clone());
      }
    }

    // Step 6: Remove sort ERROR nodes from bibentries
    for entry in included.values() {
      if let Some(ref bibentry) = entry.bibentry {
        let sort_errors = PostDocument::findnodes_foreign(".//ltx:ERROR[@class='sort']", bibentry);
        for mut sortnode in sort_errors {
          sortnode.unlink();
        }
      }
    }

    // Assign numbers in sort order
    let mut number = 0u32;
    for key in &sorted_keys {
      number += 1;
      if let Some(entry) = included.get_mut(key) {
        entry.number = number;
      }
    }

    (included, bib_docs)
  }

  /// Find the ID for a bibliography key in the ObjectDB.
  fn find_bib_id(&self, bibkey: &str, lists: &[&str]) -> Option<String> {
    for list in lists {
      let bkey = format!("BIBLABEL:{}:{}", list, bibkey);
      if let Some(bentry) = self.db.lookup(&bkey) {
        if let Some(id) = bentry.get_string("id") {
          return Some(id.to_string());
        }
      }
    }
    None
  }

  /// Format a bibliography list.
  ///
  /// Port of `makeBibliographyList`.
  fn make_bibliography_list(
    &self,
    doc: &PostDocument,
    bib_id: &str,
    initial: Option<&str>,
    entries: &HashMap<String, BibEntryData>,
    style: &CitationStyle,
  ) -> NodeData {
    let id = if let Some(init) = initial {
      format!("{}.L1.{}", bib_id, init)
    } else {
      format!("{}.L1", bib_id)
    };

    let mut sorted_keys: Vec<&String> = entries.keys().collect();
    sorted_keys.sort();
    let items: Vec<NodeData> = sorted_keys
      .iter()
      .filter_map(|key| entries.get(*key))
      .map(|entry| self.format_bib_entry(doc, bib_id, entry, style))
      .collect();

    NodeData::Element {
      tag:        "ltx:biblist".to_string(),
      attributes: Some(HashMap::from_iter([("xml:id".to_string(), id)])),
      children:   items,
    }
  }

  /// Format a single bibentry into a bibitem.
  ///
  /// Port of `formatBibEntry`.
  fn format_bib_entry(
    &self,
    doc: &PostDocument,
    bib_id: &str,
    entry: &BibEntryData,
    style: &CitationStyle,
  ) -> NodeData {
    // ID generation: match Perl's $id =~ s/^bib//; $id = $bibid . $id;
    let id = if let Some(ref bibentry) = entry.bibentry {
      let orig_id = bibentry.get_attribute("xml:id").unwrap_or_default();
      if orig_id.is_empty() {
        // No xml:id on bibentry (e.g. from raw .bib parsing) — use number
        format!("{}.bib{}", bib_id, entry.number)
      } else {
        let stripped = orig_id.strip_prefix("bib").unwrap_or(&orig_id);
        format!("{}{}", bib_id, stripped)
      }
    } else {
      format!("{}.bib{}", bib_id, entry.number)
    };

    let cited_key = entry.cited_key.as_deref().unwrap_or(&entry.bib_key);
    let mut children = Vec::new();

    // --- Tags ---
    let mut tags = Vec::new();

    // Number tag
    tags.push(NodeData::Element {
      tag:        "ltx:tag".to_string(),
      attributes: Some(HashMap::from_iter([
        ("role".to_string(), "number".to_string()),
        ("class".to_string(), "ltx_bib_number".to_string()),
      ])),
      children:   vec![NodeData::Text(entry.number.to_string())],
    });

    // Authors/fullauthors tags — extracted from bibentry XML if available
    let (author_tag_nodes, has_names, has_key, has_year, has_typetag) =
      self.build_author_year_tags(doc, entry);
    tags.extend(author_tag_nodes);

    // Refnum tag: depends on citation style
    let mut effective_style = style.clone();
    // Perl: $style = 'numbers' unless (@names || $keytag) && (@year || $typetag)
    if !((has_names || has_key) && (has_year || has_typetag)) {
      effective_style = CitationStyle::Numbers;
    }

    let mut skip_first_block = false;
    match effective_style {
      CitationStyle::Numbers => {
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "refnum".to_string()),
            ("class".to_string(), "ltx_bib_key".to_string()),
            ("open".to_string(), "[".to_string()),
            ("close".to_string(), "]".to_string()),
          ])),
          children:   vec![NodeData::Text(entry.number.to_string())],
        });
      },
      CitationStyle::Alpha => {
        // AY-style: abbreviation from author names + 2-digit year
        let aa = self.make_alpha_label(doc, entry);
        let yy = if entry.year.len() >= 4 {
          entry.year[2..4].to_string()
        } else {
          entry.year.clone()
        };
        let suffix = entry.suffix.as_deref().unwrap_or("");
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "refnum".to_string()),
            ("class".to_string(), "ltx_bib_abbrv".to_string()),
            ("open".to_string(), "[".to_string()),
            ("close".to_string(), "]".to_string()),
          ])),
          children:   vec![NodeData::Text(format!("{}{}{}", aa, yy, suffix))],
        });
      },
      CitationStyle::AuthorYear => {
        // Author-Year style: "Author (Year)" — skip first block (redundant)
        skip_first_block = true;
        let suffix = entry.suffix.as_deref().unwrap_or("");
        let author_text = if !entry.authors_short.is_empty() {
          entry.authors_short.clone()
        } else {
          entry.bib_key.clone()
        };
        let year_text = if !entry.year.is_empty() {
          format!("{}{}", entry.year, suffix)
        } else {
          String::new()
        };
        let mut refnum_children = vec![NodeData::Text(author_text)];
        if !year_text.is_empty() {
          refnum_children.push(NodeData::Text(format!(" ({})", year_text)));
        }
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "refnum".to_string()),
            ("class".to_string(), "ltx_bib_author-year".to_string()),
          ])),
          children:   refnum_children,
        });
      },
    }

    if !tags.is_empty() {
      children.push(NodeData::Element {
        tag:        "ltx:tags".to_string(),
        attributes: None,
        children:   tags,
      });
    }

    // --- Content blocks ---
    let blocks = self.format_blocks(doc, entry, skip_first_block);
    children.extend(blocks);

    // --- Cited-by block ---
    let mut citedby: Vec<NodeData> = Vec::new();
    let mut sorted_referrers: Vec<&String> = entry.referrers.iter().collect();
    sorted_referrers.sort();
    for ref_id in &sorted_referrers {
      citedby.push(NodeData::Element {
        tag:        "ltx:ref".to_string(),
        attributes: Some(HashMap::from_iter([
          ("idref".to_string(), (*ref_id).clone()),
          ("show".to_string(), "typerefnum".to_string()),
        ])),
        children:   vec![],
      });
    }
    if !entry.bibreferrers.is_empty() {
      let mut sorted_bibrefs: Vec<&String> = entry.bibreferrers.iter().collect();
      sorted_bibrefs.sort();
      citedby.push(NodeData::Element {
        tag:        "ltx:bibref".to_string(),
        attributes: Some(HashMap::from_iter([
          (
            "bibrefs".to_string(),
            sorted_bibrefs
              .iter()
              .map(|s| s.as_str())
              .collect::<Vec<_>>()
              .join(","),
          ),
          ("show".to_string(), "refnum".to_string()),
        ])),
        children:   vec![],
      });
    }
    if !citedby.is_empty() {
      let conjoined = PostDocument::conjoin(
        crate::document::Conjunction::Simple(",\n".to_string()),
        citedby,
      );
      let mut block_children = vec![NodeData::Text("Cited by: ".to_string())];
      block_children.extend(conjoined);
      block_children.push(NodeData::Text(".".to_string()));
      children.push(NodeData::Element {
        tag:        "ltx:bibblock".to_string(),
        attributes: Some(HashMap::from_iter([(
          "class".to_string(),
          "ltx_bib_cited".to_string(),
        )])),
        children:   block_children,
      });
    }

    NodeData::Element {
      tag: "ltx:bibitem".to_string(),
      attributes: Some(HashMap::from_iter([
        ("xml:id".to_string(), id),
        ("key".to_string(), cited_key.to_string()),
        ("type".to_string(), entry.entry_type.clone()),
        ("class".to_string(), format!("ltx_bib_{}", entry.bib_type())),
      ])),
      children,
    }
  }

  /// Build author/year/key/title/type tags from bibentry XML.
  ///
  /// Returns: (tag nodes, has_names, has_key, has_year, has_typetag)
  fn build_author_year_tags(
    &self,
    doc: &PostDocument,
    entry: &BibEntryData,
  ) -> (Vec<NodeData>, bool, bool, bool, bool) {
    let mut tags = Vec::new();
    let mut has_names = false;
    let mut has_key = false;
    let mut has_year = false;
    let mut has_typetag = false;

    if let Some(ref bibentry) = entry.bibentry {
      // Author surnames from bibentry XML
      let mut surnames: Vec<Node> =
        doc.findnodes_at("ltx:bib-name[@role='author']/ltx:surname", Some(bibentry));
      if surnames.is_empty() {
        surnames = doc.findnodes_at("ltx:bib-name[@role='editor']/ltx:surname", Some(bibentry));
      }

      if surnames.len() > 2 {
        has_names = true;
        // Short: first author + et al.
        let first_text = surnames[0].get_content();
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "authors".to_string()),
            ("class".to_string(), "ltx_bib_author".to_string()),
          ])),
          children:   vec![NodeData::Text(first_text), NodeData::Element {
            tag:        "ltx:text".to_string(),
            attributes: Some(HashMap::from_iter([(
              "class".to_string(),
              "ltx_bib_etal".to_string(),
            )])),
            children:   vec![NodeData::Text(" et al.".to_string())],
          }],
        });
        // Full: all names
        let mut full_children: Vec<NodeData> = Vec::new();
        for (i, surname) in surnames.iter().enumerate() {
          if i > 0 && i < surnames.len() - 1 {
            full_children.push(NodeData::Text(", ".to_string()));
          } else if i == surnames.len() - 1 {
            full_children.push(NodeData::Text(" and ".to_string()));
          }
          full_children.push(NodeData::Text(surname.get_content()));
        }
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "fullauthors".to_string()),
            ("class".to_string(), "ltx_bib_author".to_string()),
          ])),
          children:   full_children,
        });
      } else if surnames.len() == 2 {
        has_names = true;
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "authors".to_string()),
            ("class".to_string(), "ltx_bib_author".to_string()),
          ])),
          children:   vec![
            NodeData::Text(surnames[0].get_content()),
            NodeData::Text(" and ".to_string()),
            NodeData::Text(surnames[1].get_content()),
          ],
        });
      } else if !surnames.is_empty() {
        has_names = true;
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "authors".to_string()),
            ("class".to_string(), "ltx_bib_author".to_string()),
          ])),
          children:   vec![NodeData::Text(surnames[0].get_content())],
        });
      }

      // Key tag
      if let Some(key_node) = PostDocument::findnodes_foreign("ltx:bib-key", bibentry)
        .into_iter()
        .next()
      {
        has_key = true;
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "key".to_string()),
            ("class".to_string(), "ltx_bib_key".to_string()),
          ])),
          children:   vec![NodeData::Text(key_node.get_content())],
        });
      }

      // Year tag
      if let Some(date_node) =
        PostDocument::findnodes_foreign("ltx:bib-date[@role='publication']", bibentry)
          .into_iter()
          .next()
      {
        has_year = true;
        let year_text = extract_four_digit_year(&date_node.get_content());
        let suffix = entry.suffix.as_deref().unwrap_or("");
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "year".to_string()),
            ("class".to_string(), "ltx_bib_year".to_string()),
          ])),
          children:   vec![NodeData::Text(format!("{}{}", year_text, suffix))],
        });
      }

      // Type tag
      if let Some(type_node) = PostDocument::findnodes_foreign("ltx:bib-type", bibentry)
        .into_iter()
        .next()
      {
        has_typetag = true;
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "bibtype".to_string()),
            ("class".to_string(), "ltx_bib_type".to_string()),
          ])),
          children:   vec![NodeData::Text(type_node.get_content())],
        });
      }

      // Title tag
      if let Some(title_node) = PostDocument::findnodes_foreign("ltx:bib-title", bibentry)
        .into_iter()
        .next()
      {
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "title".to_string()),
            ("class".to_string(), "ltx_bib_title".to_string()),
          ])),
          children:   vec![NodeData::Text(title_node.get_content())],
        });
      }
    } else {
      // No bibentry XML — use ObjectDB metadata strings
      if !entry.authors_short.is_empty() {
        has_names = true;
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "authors".to_string()),
            ("class".to_string(), "ltx_bib_author".to_string()),
          ])),
          children:   vec![NodeData::Text(entry.authors_short.clone())],
        });
        if entry.authors_full != entry.authors_short {
          tags.push(NodeData::Element {
            tag:        "ltx:tag".to_string(),
            attributes: Some(HashMap::from_iter([
              ("role".to_string(), "fullauthors".to_string()),
              ("class".to_string(), "ltx_bib_author".to_string()),
            ])),
            children:   vec![NodeData::Text(entry.authors_full.clone())],
          });
        }
      }
      if !entry.year.is_empty() {
        has_year = true;
        let suffix = entry.suffix.as_deref().unwrap_or("");
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "year".to_string()),
            ("class".to_string(), "ltx_bib_year".to_string()),
          ])),
          children:   vec![NodeData::Text(format!("{}{}", entry.year, suffix))],
        });
      }
      if !entry.title.is_empty() {
        tags.push(NodeData::Element {
          tag:        "ltx:tag".to_string(),
          attributes: Some(HashMap::from_iter([
            ("role".to_string(), "title".to_string()),
            ("class".to_string(), "ltx_bib_title".to_string()),
          ])),
          children:   vec![NodeData::Text(entry.title.clone())],
        });
      }
    }

    (tags, has_names, has_key, has_year, has_typetag)
  }

  /// Generate alphabetic label for AY/alpha citation style.
  ///
  /// Port of the alpha refnum logic in `formatBibEntry`.
  fn make_alpha_label(&self, doc: &PostDocument, entry: &BibEntryData) -> String {
    if let Some(ref bibentry) = entry.bibentry {
      let mut surnames: Vec<Node> =
        doc.findnodes_at("ltx:bib-name[@role='author']/ltx:surname", Some(bibentry));
      if surnames.is_empty() {
        surnames = doc.findnodes_at("ltx:bib-name[@role='editor']/ltx:surname", Some(bibentry));
      }
      if surnames.len() > 1 {
        let mut aa: String = surnames
          .iter()
          .map(|n| n.get_content().chars().next().unwrap_or('?').to_string())
          .collect();
        if aa.len() > 3 {
          aa = format!("{}+", &aa[..3]);
        }
        aa.to_uppercase()
      } else if !surnames.is_empty() {
        let text = surnames[0].get_content();
        text.chars().take(3).collect::<String>().to_uppercase()
      } else {
        entry
          .bib_key
          .chars()
          .take(3)
          .collect::<String>()
          .to_uppercase()
      }
    } else {
      // Fallback: use author short name
      if !entry.authors_short.is_empty() {
        entry
          .authors_short
          .split_whitespace()
          .filter_map(|w| w.chars().next())
          .map(|c| c.to_uppercase().to_string())
          .collect::<Vec<_>>()
          .join("")
      } else {
        entry
          .bib_key
          .chars()
          .take(3)
          .collect::<String>()
          .to_uppercase()
      }
    }
  }

  /// Format content blocks using the FMT_SPEC table.
  ///
  /// Port of the block formatting loop in `formatBibEntry` + `%FMT_SPEC`.
  fn format_blocks(
    &self,
    doc: &PostDocument,
    entry: &BibEntryData,
    skip_first: bool,
  ) -> Vec<NodeData> {
    let format_type = entry.format_type();
    let block_specs = get_fmt_spec(format_type);
    let mut blocks = Vec::new();

    for (i, block_spec) in block_specs.iter().enumerate() {
      if skip_first && i == 0 {
        continue;
      }

      let mut items: Vec<NodeData> = Vec::new();
      for field_spec in block_spec {
        let (nodes_found, negated) = if let Some(ref bibentry) = entry.bibentry {
          let xpath = field_spec.xpath.trim_start_matches('!').trim();
          let negated = field_spec.xpath.starts_with('!');
          if xpath == "true" {
            (true, false)
          } else {
            let found = !PostDocument::findnodes_foreign(xpath, bibentry).is_empty();
            (found, negated)
          }
        } else {
          // No bibentry — try to match from metadata
          let found = match_metadata_field(field_spec.xpath, entry);
          (found, field_spec.xpath.starts_with('!'))
        };

        // Check condition
        if field_spec.xpath != "true" {
          if negated {
            if nodes_found {
              continue;
            }
          } else {
            if !nodes_found {
              continue;
            }
          }
        }

        // Add punctuation if there are preceding items
        if !field_spec.punct.is_empty() && !items.is_empty() {
          items.push(NodeData::Text(field_spec.punct.to_string()));
        }
        // Pre-text
        if !field_spec.pre.is_empty() {
          items.push(NodeData::Text(field_spec.pre.to_string()));
        }
        // Content (wrapped in ltx:text with class)
        if !field_spec.class.is_empty() {
          let content = if let Some(ref bibentry) = entry.bibentry {
            let xpath = field_spec.xpath.trim_start_matches('!').trim();
            if xpath == "true" {
              Vec::new()
            } else {
              let nodes = PostDocument::findnodes_foreign(xpath, bibentry);
              apply_formatter(doc, field_spec.formatter, &nodes)
            }
          } else {
            get_metadata_content(field_spec.xpath, entry)
          };
          if !content.is_empty() {
            items.push(NodeData::Element {
              tag:        "ltx:text".to_string(),
              attributes: Some(HashMap::from_iter([(
                "class".to_string(),
                format!("ltx_bib_{}", field_spec.class),
              )])),
              children:   content,
            });
          }
        }
        // Post-text
        if !field_spec.post.is_empty() {
          items.push(NodeData::Text(field_spec.post.to_string()));
        }
      }

      if !items.is_empty() {
        blocks.push(make_bibblock("", &items));
      }
    }

    // Note + External Links are part of every type's FMT_SPEC via the
    // `meta_block` appended in get_fmt_spec, so the loop above already emits
    // them. (A second, hard-coded copy here previously duplicated every
    // entry's final Note/External-Links bibblock.)

    blocks
  }
}

impl Processor for MakeBibliography {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> { doc.findnodes("//ltx:bibliography") }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    for bib in &nodes {
      // Skip if already populated
      if !doc.findnodes_at(".//ltx:bibitem", Some(bib)).is_empty() {
        continue;
      }

      // Read citation style from element attributes
      let citestyle_str = bib
        .get_attribute("citestyle")
        .unwrap_or_else(|| "numbers".to_string());
      let style = match citestyle_str.as_str() {
        "AY" | "authoryear" | "author-year" => CitationStyle::AuthorYear,
        "alpha" | "Alpha" => CitationStyle::Alpha,
        _ => CitationStyle::Numbers,
      };

      // bib_docs must be kept alive as long as entries (bibentry Nodes reference them)
      let (entries, _bib_docs) = self.get_bib_entries(&doc, bib);
      if entries.is_empty() {
        Info!(
          "bibliography",
          "empty",
          "MakeBibliography: no entries to process"
        );
        continue;
      }

      let bib_id = bib
        .get_attribute("xml:id")
        .or_else(|| {
          doc
            .get_document_element()
            .and_then(|r| r.get_attribute("xml:id"))
        })
        .unwrap_or_else(|| "bib".to_string());

      if self.split {
        // Split by initial letter
        let mut by_initial: HashMap<String, HashMap<String, &BibEntryData>> = HashMap::default();
        for (key, entry) in &entries {
          by_initial
            .entry(entry.initial.clone())
            .or_default()
            .insert(key.clone(), entry);
        }
        let mut initials: Vec<&String> = by_initial.keys().collect();
        initials.sort();
        for initial in initials {
          // Build a subset HashMap for this initial
          let subset: HashMap<String, BibEntryData> = by_initial[initial]
            .iter()
            .map(|(k, e)| (k.clone(), clone_entry(e)))
            .collect();
          let biblist = self.make_bibliography_list(&doc, &bib_id, Some(initial), &subset, &style);
          let mut bib_mut = bib.clone();
          doc.add_nodes(&mut bib_mut, &[biblist]);
        }
      } else {
        let biblist = self.make_bibliography_list(&doc, &bib_id, None, &entries, &style);
        let mut bib_mut = bib.clone();
        doc.add_nodes(&mut bib_mut, &[biblist]);
      }

      Info!(
        "bibliography",
        "formatted",
        "MakeBibliography: formatted {} entries",
        entries.len()
      );

      // Register formatted bibitems in ObjectDB so CrossRef can resolve citations.
      // Port of Perl's approach where bibitems are registered during Scan,
      // but here we must register them after MakeBibliography creates them.
      let lists_str = bib
        .get_attribute("lists")
        .unwrap_or_else(|| "bibliography".to_string());
      for entry in entries.values() {
        let cited_key = entry.cited_key.as_deref().unwrap_or(&entry.bib_key);
        // Compute the same ID as format_bib_entry
        let bibitem_id = if let Some(ref bibentry) = entry.bibentry {
          let orig_id = bibentry.get_attribute("xml:id").unwrap_or_default();
          if orig_id.is_empty() {
            format!("{}.bib{}", bib_id, entry.number)
          } else {
            let stripped = orig_id.strip_prefix("bib").unwrap_or(&orig_id);
            format!("{}{}", bib_id, stripped)
          }
        } else {
          format!("{}.bib{}", bib_id, entry.number)
        };

        // Register BIBLABEL:{list}:{key} → id
        for list in lists_str.split_whitespace() {
          let label_key = format!("BIBLABEL:{}:{}", list, cited_key);
          self.db.register(&label_key, vec![(
            "id",
            crate::object_db::Value::from(bibitem_id.as_str()),
          )]);
        }

        // Register ID:{id} with type, location, and number for CrossRef URL generation
        let location = doc.site_relative_destination().unwrap_or_default();
        self.db.register(&format!("ID:{}", bibitem_id), vec![
          ("type", crate::object_db::Value::from("ltx:bibitem")),
          ("location", crate::object_db::Value::from(location.as_str())),
          ("fragid", crate::object_db::Value::from(bibitem_id.as_str())),
          (
            "number",
            crate::object_db::Value::from(entry.number.to_string().as_str()),
          ),
        ]);
      }
    }

    // Remove any remaining bibentry elements (they've been converted to bibitems)
    let bibentries = doc.findnodes("//ltx:bibentry");
    if !bibentries.is_empty() {
      doc.remove_nodes(&bibentries);
    }

    // Remove empty biblists
    let biblists = doc.findnodes("//ltx:biblist");
    let empty_lists: Vec<Node> = biblists
      .into_iter()
      .filter(|n| {
        n.get_first_child()
          .map(|c| {
            let mut has_element = false;
            let mut current = Some(c);
            while let Some(ref node) = current {
              if node.get_type() == Some(libxml::tree::NodeType::ElementNode) {
                has_element = true;
                break;
              }
              current = node.get_next_sibling();
            }
            !has_element
          })
          .unwrap_or(true)
      })
      .collect();
    if !empty_lists.is_empty() {
      doc.remove_nodes(&empty_lists);
    }

    Ok(vec![doc])
  }
}

// ======================================================================
// FMT_SPEC table — defines the block structure for each bibliography type.
//
// Port of the `%FMT_SPEC` table from MakeBibliography.pm.
// Each type has a sequence of blocks.
// Each block has a sequence of field specifications.

/// A field specification for bibliography formatting.
#[derive(Clone)]
struct FieldSpec {
  /// XPath expression (or "true" for unconditional). Prefix "!" for negation.
  xpath:     &'static str,
  /// Punctuation to insert before this field (if preceding content exists).
  punct:     &'static str,
  /// Text prefix.
  pre:       &'static str,
  /// CSS class (without ltx_bib_ prefix).
  class:     &'static str,
  /// Formatter function name.
  formatter: Formatter,
  /// Text suffix.
  post:      &'static str,
}

#[derive(Clone, Copy)]
enum Formatter {
  Any,
  Authors,
  EditorsA,
  EditorsB,
  Year,
  Type,
  Title,
  ThesisType,
  Edition,
  Pages,
  CrossRef,
  Links,
  None,
}

/// Get the FMT_SPEC block specifications for a bibliography type.
fn get_fmt_spec(format_type: &str) -> Vec<Vec<FieldSpec>> {
  let meta_block: Vec<Vec<FieldSpec>> = vec![
    vec![FieldSpec {
      xpath:     "ltx:bib-note",
      punct:     "",
      pre:       "Note: ",
      class:     "note",
      formatter: Formatter::Any,
      post:      "",
    }],
    vec![FieldSpec {
      xpath:     "ltx:bib-links | ltx:bib-review | ltx:bib-identifier | ltx:bib-url",
      punct:     "",
      pre:       "External Links: ",
      class:     "links",
      formatter: Formatter::Links,
      post:      "",
    }],
  ];

  let mut blocks = match format_type {
    "article" => vec![
      // Block 1: authors + year
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
      ],
      // Block 2: title
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Title,
        post:      ".",
      }],
      // Block 3: journal details
      vec![
        FieldSpec {
          xpath:     "ltx:bib-part[@role='part']",
          punct:     "",
          pre:       "",
          class:     "part",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-related/ltx:bib-title",
          punct:     ", ",
          pre:       "",
          class:     "journal",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='volume']",
          punct:     " ",
          pre:       "",
          class:     "volume",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='number']",
          punct:     " ",
          pre:       "(",
          class:     "number",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "ltx:bib-status",
          punct:     ", ",
          pre:       "(",
          class:     "status",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='pages']",
          punct:     ", ",
          pre:       "",
          class:     "pages",
          formatter: Formatter::Pages,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-language",
          punct:     " ",
          pre:       "(",
          class:     "language",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    "book" => vec![
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-name[@role='editor']",
          punct:     "",
          pre:       "",
          class:     "editor",
          formatter: Formatter::EditorsA,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
      ],
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Title,
        post:      ".",
      }],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-type",
          punct:     "",
          pre:       "",
          class:     "type",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-edition",
          punct:     ", ",
          pre:       "",
          class:     "edition",
          formatter: Formatter::Edition,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='series']",
          punct:     ", ",
          pre:       "",
          class:     "series",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='volume']",
          punct:     ", ",
          pre:       "Vol. ",
          class:     "volume",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='part']",
          punct:     ", ",
          pre:       "Part ",
          class:     "part",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-publisher",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-organization",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-place",
          punct:     ", ",
          pre:       "",
          class:     "place",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-status",
          punct:     " ",
          pre:       "(",
          class:     "status",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "ltx:bib-language",
          punct:     " ",
          pre:       "(",
          class:     "language",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    "incollection" => vec![
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
      ],
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Title,
        post:      ".",
      }],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-type",
          punct:     "",
          pre:       "",
          class:     "type",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-related[@bibrefs]",
          punct:     " ",
          pre:       "See ",
          class:     "crossref",
          formatter: Formatter::CrossRef,
          post:      ",",
        },
        FieldSpec {
          xpath:     "ltx:bib-related[@type][not(../ltx:bib-related[@bibrefs])]/ltx:bib-title",
          punct:     " ",
          pre:       "In ",
          class:     "inbook",
          formatter: Formatter::Title,
          post:      ",",
        },
        FieldSpec {
          xpath:     "ltx:bib-related[@type][not(../ltx:bib-related[@bibrefs])]/ltx:bib-name[@role='editor']",
          punct:     " ",
          pre:       " ",
          class:     "editor",
          formatter: Formatter::EditorsA,
          post:      ",",
        },
      ],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-edition",
          punct:     "",
          pre:       "",
          class:     "edition",
          formatter: Formatter::Edition,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-name[@role='editor']",
          punct:     ", ",
          pre:       "",
          class:     "editor",
          formatter: Formatter::EditorsB,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-related/ltx:bib-part[@role='series']",
          punct:     ", ",
          pre:       "",
          class:     "series",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-related/ltx:bib-part[@role='volume']",
          punct:     ", ",
          pre:       "Vol. ",
          class:     "volume",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-related/ltx:bib-part[@role='part']",
          punct:     ", ",
          pre:       "Part ",
          class:     "part",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-publisher",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-organization",
          punct:     ", ",
          pre:       "",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-place",
          punct:     ", ",
          pre:       "",
          class:     "place",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='pages']",
          punct:     ", ",
          pre:       "",
          class:     "pages",
          formatter: Formatter::Pages,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-status",
          punct:     " ",
          pre:       "(",
          class:     "status",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "ltx:bib-language",
          punct:     " ",
          pre:       "(",
          class:     "language",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    "report" => vec![
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-name[@role='editor']",
          punct:     "",
          pre:       "",
          class:     "editor",
          formatter: Formatter::EditorsA,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
      ],
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Title,
        post:      ".",
      }],
      vec![FieldSpec {
        xpath:     "ltx:bib-type",
        punct:     "",
        pre:       "",
        class:     "type",
        formatter: Formatter::Any,
        post:      "",
      }],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-part[@role='number']",
          punct:     "",
          pre:       "Technical Report ",
          class:     "number",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='series']",
          punct:     ", ",
          pre:       "",
          class:     "series",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='volume']",
          punct:     ", ",
          pre:       "Vol. ",
          class:     "volume",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='part']",
          punct:     ", ",
          pre:       "Part ",
          class:     "part",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-publisher",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-organization",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-place",
          punct:     ", ",
          pre:       " ",
          class:     "place",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-status",
          punct:     ", ",
          pre:       "(",
          class:     "status",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "ltx:bib-language",
          punct:     " ",
          pre:       "(",
          class:     "language",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    "thesis" => vec![
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-name[@role='editor']",
          punct:     "",
          pre:       "",
          class:     "editor",
          formatter: Formatter::EditorsA,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
      ],
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Title,
        post:      ".",
      }],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-type",
          punct:     " ",
          pre:       "",
          class:     "type",
          formatter: Formatter::ThesisType,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-part[@role='part']",
          punct:     ", ",
          pre:       "Part ",
          class:     "part",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-publisher",
          punct:     ", ",
          pre:       "",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-organization",
          punct:     ", ",
          pre:       "",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-place",
          punct:     ", ",
          pre:       "",
          class:     "place",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-status",
          punct:     ", ",
          pre:       "(",
          class:     "status",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "ltx:bib-language",
          punct:     ", ",
          pre:       "(",
          class:     "language",
          formatter: Formatter::Any,
          post:      ")",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    "website" => vec![
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-name[@role='editor']",
          punct:     "",
          pre:       "",
          class:     "editor",
          formatter: Formatter::EditorsA,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-title",
          punct:     "",
          pre:       "",
          class:     "title",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-type",
          punct:     "",
          pre:       "",
          class:     "type",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "! ltx:bib-type",
          punct:     "",
          pre:       "",
          class:     "type",
          formatter: Formatter::None,
          post:      "(Website)",
        },
      ],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-organization",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-place",
          punct:     ", ",
          pre:       "",
          class:     "place",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    "software" => vec![
      vec![
        FieldSpec {
          xpath:     "ltx:bib-key",
          punct:     "",
          pre:       "",
          class:     "key",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-type",
          punct:     "",
          pre:       "",
          class:     "type",
          formatter: Formatter::Type,
          post:      "",
        },
      ],
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Any,
        post:      "",
      }],
      vec![
        FieldSpec {
          xpath:     "ltx:bib-organization",
          punct:     ", ",
          pre:       " ",
          class:     "publisher",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-place",
          punct:     ", ",
          pre:       "",
          class:     "place",
          formatter: Formatter::Any,
          post:      "",
        },
        FieldSpec {
          xpath:     "true",
          punct:     ".",
          pre:       "",
          class:     "",
          formatter: Formatter::None,
          post:      "",
        },
      ],
    ],
    _ => vec![
      // Default: same as book
      vec![
        FieldSpec {
          xpath:     "ltx:bib-name[@role='author']",
          punct:     "",
          pre:       "",
          class:     "author",
          formatter: Formatter::Authors,
          post:      "",
        },
        FieldSpec {
          xpath:     "ltx:bib-date[@role='publication']",
          punct:     "",
          pre:       "",
          class:     "year",
          formatter: Formatter::Year,
          post:      "",
        },
      ],
      vec![FieldSpec {
        xpath:     "ltx:bib-title",
        punct:     "",
        pre:       "",
        class:     "title",
        formatter: Formatter::Title,
        post:      ".",
      }],
    ],
  };
  blocks.extend(meta_block);
  blocks
}

// ======================================================================
// Formatting helpers

/// Apply a formatter function to the given nodes.
///
/// Port of the various `do_*` functions.
fn apply_formatter(doc: &PostDocument, formatter: Formatter, nodes: &[Node]) -> Vec<NodeData> {
  match formatter {
    Formatter::Any => nodes
      .iter()
      .map(|n| NodeData::Text(n.get_content()))
      .collect(),
    Formatter::Authors => format_author_nodes(doc, nodes),
    Formatter::EditorsA => {
      let mut result = format_author_nodes(doc, nodes);
      let suffix = if nodes.len() > 1 { " (Eds.)" } else { " (Ed.)" };
      result.push(NodeData::Text(suffix.to_string()));
      result
    },
    Formatter::EditorsB => {
      let mut result = vec![NodeData::Text("(".to_string())];
      result.extend(format_author_nodes(doc, nodes));
      let suffix = if nodes.len() > 1 { " Eds.)" } else { " Ed.)" };
      result.push(NodeData::Text(suffix.to_string()));
      result
    },
    Formatter::Year => {
      let suffix = ""; // Suffix is handled elsewhere
      let content: Vec<NodeData> = nodes
        .iter()
        .map(|n| {
          let text = n.get_content();
          let year = extract_four_digit_year(&text);
          NodeData::Text(year)
        })
        .collect();
      let mut result = vec![NodeData::Text(" (".to_string())];
      result.extend(content);
      result.push(NodeData::Text(format!("{})", suffix)));
      result
    },
    Formatter::Type => {
      let content: Vec<NodeData> = nodes
        .iter()
        .map(|n| NodeData::Text(n.get_content()))
        .collect();
      let mut result = vec![NodeData::Text("(".to_string())];
      result.extend(content);
      result.push(NodeData::Text(")".to_string()));
      result
    },
    Formatter::Title => nodes
      .iter()
      .map(|n| NodeData::Text(n.get_content()))
      .collect(),
    Formatter::ThesisType => nodes
      .iter()
      .map(|n| NodeData::Text(n.get_content()))
      .collect(),
    Formatter::Edition => {
      let mut result: Vec<NodeData> = nodes
        .iter()
        .map(|n| NodeData::Text(n.get_content()))
        .collect();
      result.push(NodeData::Text(" edition".to_string()));
      result
    },
    Formatter::Pages => {
      let mut result = vec![NodeData::Text("pp.\u{00A0}".to_string())]; // Non-breaking space
      result.extend(nodes.iter().map(|n| NodeData::Text(n.get_content())));
      result
    },
    Formatter::CrossRef => {
      // Port of do_crossref
      if let Some(node) = nodes.first() {
        if let Some(bibrefs) = node.get_attribute("bibrefs") {
          return vec![NodeData::Element {
            tag:        "ltx:cite".to_string(),
            attributes: None,
            children:   vec![NodeData::Element {
              tag:        "ltx:bibref".to_string(),
              attributes: Some(HashMap::from_iter([
                ("bibrefs".to_string(), bibrefs),
                ("show".to_string(), "title, author".to_string()),
              ])),
              children:   vec![],
            }],
          }];
        }
      }
      Vec::new()
    },
    Formatter::Links => format_links(doc, nodes),
    Formatter::None => Vec::new(),
  }
}

/// Format author name nodes.
///
/// Port of `do_names` / `do_name`.
fn format_author_nodes(_doc: &PostDocument, name_nodes: &[Node]) -> Vec<NodeData> {
  let mut result: Vec<NodeData> = Vec::new();
  let mut names: Vec<Node> = name_nodes.to_vec();

  // Check for "others" sentinel (et al.)
  let etal = names
    .last()
    .map(|n| n.get_content().trim() == "others")
    .unwrap_or(false);
  if etal {
    names.pop();
  }

  let sep = if names.len() > 2 { ", " } else { " " };

  for (i, name) in names.iter().enumerate() {
    if i > 0 {
      result.push(NodeData::Text(sep.to_string()));
      if !etal && i == names.len() - 1 {
        result.push(NodeData::Text("and ".to_string()));
      }
    }
    // Format single name: initials + surname
    if let Some(givenname) = PostDocument::findnodes_foreign("ltx:givenname", name)
      .into_iter()
      .next()
    {
      let given_text = givenname.get_content();
      let initials: String = given_text
        .split_whitespace()
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
      result.push(NodeData::Text(initials));
    }
    if let Some(surname) = PostDocument::findnodes_foreign("ltx:surname", name)
      .into_iter()
      .next()
    {
      result.push(NodeData::Text(surname.get_content()));
    }
  }

  if etal {
    result.push(NodeData::Text(sep.to_string()));
    result.push(NodeData::Element {
      tag:        "ltx:text".to_string(),
      attributes: Some(HashMap::from_iter([(
        "class".to_string(),
        "ltx_bib_etal".to_string(),
      )])),
      children:   vec![NodeData::Text("et al.".to_string())],
    });
  }

  result
}

/// Format external links.
///
/// Port of `do_links`.
fn format_links(doc: &PostDocument, nodes: &[Node]) -> Vec<NodeData> {
  let mut links: Vec<NodeData> = Vec::new();

  for node in nodes {
    let tag = doc.get_qname(node).unwrap_or_default();
    let scheme = node.get_attribute("scheme").unwrap_or_default();
    let href = node.get_attribute("href");
    let content_text = node.get_content();

    // General rule (user, 2026-07-04): bibliography links are EXTERNAL.
    // DOIs always resolve via https://doi.org/, and scheme-less hrefs are
    // an authoring mistake that would resolve relative to the article —
    // normalize here so every source path (post .bib conversion,
    // .bbl-borne XML, pre-compiled .bib.xml) gets absolute links.
    let href = match (&href, scheme.as_str()) {
      (None, "doi") if !content_text.trim().is_empty() && content_text.contains('/') => {
        Some(doi_href(&content_text))
      },
      (Some(h), "doi") if !h.contains("://") => Some(doi_href(h.trim_start_matches('/'))),
      (Some(h), _) => Some(force_absolute_url(h)),
      (None, _) => None,
    };
    match tag.as_str() {
      "ltx:bib-identifier" | "ltx:bib-review" => {
        if let Some(href) = href {
          links.push(NodeData::Element {
            tag:        "ltx:ref".to_string(),
            attributes: Some(HashMap::from_iter([
              ("href".to_string(), href),
              ("class".to_string(), format!("{} ltx_bib_external", scheme)),
            ])),
            children:   vec![NodeData::Text(content_text)],
          });
        } else {
          links.push(NodeData::Element {
            tag:        "ltx:text".to_string(),
            attributes: Some(HashMap::from_iter([(
              "class".to_string(),
              format!("{} ltx_bib_external", scheme),
            )])),
            children:   vec![NodeData::Text(content_text)],
          });
        }
      },
      "ltx:bib-links" => {
        links.push(NodeData::Element {
          tag:        "ltx:text".to_string(),
          attributes: Some(HashMap::from_iter([(
            "class".to_string(),
            "ltx_bib_external".to_string(),
          )])),
          children:   vec![NodeData::Text(content_text)],
        });
      },
      "ltx:bib-url" => {
        if let Some(href) = href {
          links.push(NodeData::Element {
            tag:        "ltx:ref".to_string(),
            attributes: Some(HashMap::from_iter([
              ("href".to_string(), href),
              ("class".to_string(), "ltx_bib_external".to_string()),
            ])),
            children:   vec![NodeData::Text(content_text)],
          });
        }
      },
      _ => {},
    }
  }

  // Join with ",\n"
  if links.len() > 1 {
    let mut result = Vec::new();
    for (i, link) in links.into_iter().enumerate() {
      if i > 0 {
        result.push(NodeData::Text(",\n".to_string()));
      }
      result.push(link);
    }
    result
  } else {
    links
  }
}

// ======================================================================
// Utility functions

/// Extract author names from a bibentry node.
///
/// Port of the name extraction logic in `getBibEntries`.
/// Returns (sort_names, short_names, full_names).
fn extract_names(doc: &PostDocument, bibentry: &Node) -> (String, String, String) {
  let mut name_nodes: Vec<Node> =
    PostDocument::findnodes_foreign("ltx:bib-name[@role='author']", bibentry);
  if name_nodes.is_empty() {
    name_nodes = PostDocument::findnodes_foreign("ltx:bib-name[@role='editor']", bibentry);
  }

  if name_nodes.is_empty() {
    // Try bib-key
    if let Some(key_node) = PostDocument::findnodes_foreign("ltx:bib-key", bibentry)
      .into_iter()
      .next()
    {
      let text = key_node.get_content();
      return (text.clone(), text.clone(), text);
    }
    // Try bib-title
    if let Some(title_node) = PostDocument::findnodes_foreign("ltx:bib-title", bibentry)
      .into_iter()
      .next()
    {
      let text = title_node.get_content();
      return (text.clone(), text.clone(), text);
    }
    return (String::new(), String::new(), String::new());
  }

  // Sort names: "Surname Givenname" for each
  let sort_names: String = name_nodes
    .iter()
    .map(|n| get_name_text(doc, n))
    .collect::<Vec<_>>()
    .join(" ");

  // Short names: surnames only, with "et al" for >2
  let surnames: Vec<String> = name_nodes
    .iter()
    .filter_map(|n| {
      PostDocument::findnodes_foreign("ltx:surname", n)
        .into_iter()
        .next()
        .map(|s| s.get_content())
    })
    .collect();

  let short_names = if surnames.len() > 2 {
    format!("{} et al", surnames[0])
  } else if surnames.len() == 2 {
    format!("{} and {}", surnames[0], surnames[1])
  } else if !surnames.is_empty() {
    surnames[0].clone()
  } else {
    String::new()
  };

  let full_names = surnames.join(", ");
  (sort_names, short_names, full_names)
}

/// Get sort-friendly name text from a bib-name node.
///
/// Port of `getNameText`.
fn get_name_text(_doc: &PostDocument, namenode: &Node) -> String {
  let surname = PostDocument::findnodes_foreign("ltx:surname", namenode)
    .into_iter()
    .next()
    .map(|n| n.get_content());
  let givenname = PostDocument::findnodes_foreign("ltx:givenname", namenode)
    .into_iter()
    .next()
    .map(|n| n.get_content());
  match (surname, givenname) {
    (Some(s), Some(g)) => format!("{} {}", s, g),
    (Some(s), None) => s,
    (None, Some(g)) => g,
    (None, None) => String::new(),
  }
}

/// Extract a 4-digit year from a date string.
fn extract_four_digit_year(text: &str) -> String {
  if let Some(start) = text.find(|c: char| c.is_ascii_digit()) {
    let digits: String = text[start..]
      .chars()
      .take_while(|c| c.is_ascii_digit())
      .collect();
    if digits.len() >= 4 {
      return digits[..4].to_string();
    }
  }
  text.to_string()
}

/// Convert a suffix string back to a counter value.
fn suffix_to_counter(suffix: &str) -> u32 {
  let mut n = 0u32;
  for c in suffix.chars() {
    n = n * 26 + (c as u32 - 'a' as u32 + 1);
  }
  n
}

/// Check if metadata field matches an XPath-like selector.
fn match_metadata_field(xpath: &str, entry: &BibEntryData) -> bool {
  let xpath = xpath.trim_start_matches('!').trim();
  match xpath {
    "true" => true,
    s if s.contains("bib-name[@role='author']") => !entry.authors_short.is_empty(),
    s if s.contains("bib-name[@role='editor']") => false, // No editor in metadata
    s if s.contains("bib-date[@role='publication']") => !entry.year.is_empty(),
    s if s.contains("bib-title") => !entry.title.is_empty(),
    _ => false,
  }
}

/// Get content from metadata fields matching an XPath-like selector.
fn get_metadata_content(xpath: &str, entry: &BibEntryData) -> Vec<NodeData> {
  let xpath = xpath.trim_start_matches('!').trim();
  match xpath {
    s if s.contains("bib-name[@role='author']") && !entry.authors_full.is_empty() => {
      vec![NodeData::Text(format_authors_text(&entry.authors_full))]
    },
    s if s.contains("bib-date[@role='publication']") && !entry.year.is_empty() => {
      vec![NodeData::Text(entry.year.clone())]
    },
    s if s.contains("bib-title") && !entry.title.is_empty() => {
      vec![NodeData::Text(entry.title.clone())]
    },
    _ => Vec::new(),
  }
}

/// Format author names for display (from metadata string).
fn format_authors_text(authors: &str) -> String {
  let names: Vec<&str> = authors.split(" and ").collect();
  let n = names.len();
  if n == 0 {
    return authors.to_string();
  }

  let has_etal = names.last().map(|n| n.trim() == "others").unwrap_or(false);
  let real_names: Vec<&str> = if has_etal {
    names[..n - 1].to_vec()
  } else {
    names
  };

  let formatted: Vec<String> = real_names
    .iter()
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
fn format_single_name(name: &str) -> String {
  if let Some((surname, given)) = name.split_once(',') {
    let surname = surname.trim();
    let initials: String = given
      .split_whitespace()
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

/// Clone a BibEntryData (for split operation).
fn clone_entry(e: &BibEntryData) -> BibEntryData {
  BibEntryData {
    bib_key:       e.bib_key.clone(),
    cited_key:     e.cited_key.clone(),
    sort_key:      e.sort_key.clone(),
    initial:       e.initial.clone(),
    author_year:   e.author_year.clone(),
    suffix:        e.suffix.clone(),
    authors_short: e.authors_short.clone(),
    authors_full:  e.authors_full.clone(),
    sort_names:    e.sort_names.clone(),
    year:          e.year.clone(),
    title:         e.title.clone(),
    entry_type:    e.entry_type.clone(),
    number:        e.number,
    referrers:     e.referrers.clone(),
    bibreferrers:  e.bibreferrers.clone(),
    citations:     e.citations.clone(),
    bibentry:      e.bibentry.clone(),
  }
}

/// Create a bibblock element with xml:space="preserve".
fn make_bibblock(class: &str, content: &[NodeData]) -> NodeData {
  let mut attrs = HashMap::default();
  attrs.insert("xml:space".to_string(), "preserve".to_string());
  if !class.is_empty() {
    attrs.insert("class".to_string(), class.to_string());
  }
  NodeData::Element {
    tag:        "ltx:bibblock".to_string(),
    attributes: Some(attrs),
    children:   content.to_vec(),
  }
}

/// Find a file in the given search paths.
fn find_file(name: &str, search_paths: &[String]) -> Option<String> {
  if Path::new(name).is_file() {
    return Some(name.to_string());
  }
  for sp in search_paths {
    let p = format!("{}/{}", sp, name);
    if Path::new(&p).is_file() {
      return Some(p);
    }
  }
  None
}

// ================================================================================
// BibTeX → XML conversion
// ================================================================================

/// A parsed BibTeX entry.
struct BibEntry {
  entry_type: String,
  key:        String,
  fields:     Vec<(String, String)>,
}

/// Parse a raw `.bib` file into BibTeX entries.
///
/// Handles `@type{key, field = {value}, field = "value", field = number}`.
/// Supports nested braces in values and string concatenation with `#`.
fn parse_bibtex(input: &str) -> Vec<BibEntry> {
  let mut entries = Vec::new();
  let chars: Vec<char> = input.chars().collect();
  let len = chars.len();
  let mut i = 0;

  while i < len {
    // Skip to next @
    if chars[i] != '@' {
      i += 1;
      continue;
    }
    i += 1; // skip @

    // Read entry type
    let type_start = i;
    while i < len && chars[i].is_alphanumeric() {
      i += 1;
    }
    let entry_type = chars[type_start..i]
      .iter()
      .collect::<String>()
      .to_lowercase();

    // Skip @string, @preamble, @comment
    if entry_type == "string" || entry_type == "preamble" || entry_type == "comment" {
      // Skip to matching brace
      while i < len && chars[i] != '{' && chars[i] != '(' {
        i += 1;
      }
      if i < len {
        i += 1;
        let open = if chars[i - 1] == '{' { '{' } else { '(' };
        let close = if open == '{' { '}' } else { ')' };
        let mut depth = 1;
        while i < len && depth > 0 {
          if chars[i] == open {
            depth += 1;
          }
          if chars[i] == close {
            depth -= 1;
          }
          i += 1;
        }
      }
      continue;
    }

    // Skip whitespace
    while i < len && chars[i].is_whitespace() {
      i += 1;
    }

    // Opening brace or paren
    if i >= len || (chars[i] != '{' && chars[i] != '(') {
      continue;
    }
    let close_ch = if chars[i] == '{' { '}' } else { ')' };
    i += 1;

    // Skip whitespace
    while i < len && chars[i].is_whitespace() {
      i += 1;
    }

    // Read citation key (until comma or whitespace)
    let key_start = i;
    while i < len && chars[i] != ',' && chars[i] != close_ch && !chars[i].is_whitespace() {
      i += 1;
    }
    let key = chars[key_start..i]
      .iter()
      .collect::<String>()
      .trim()
      .to_string();

    // Skip to comma
    while i < len && chars[i] != ',' && chars[i] != close_ch {
      i += 1;
    }
    if i < len && chars[i] == ',' {
      i += 1;
    }

    // Read fields
    let mut fields = Vec::new();
    loop {
      // Skip whitespace and commas
      while i < len && (chars[i].is_whitespace() || chars[i] == ',') {
        i += 1;
      }
      if i >= len || chars[i] == close_ch {
        break;
      }
      // An entry-level unbalance (missing final close) would otherwise feed
      // the NEXT entry's `@` into the field-name reader and swallow it —
      // resync at the boundary instead (BibTeX-style), loudly.
      if chars[i] == '@' {
        crate::Warn!(
          "bibtex",
          "unbalanced",
          "Entry '{}' not closed before the next '@'; resyncing",
          key
        );
        break;
      }

      // Read field name
      let fname_start = i;
      while i < len && chars[i] != '=' && !chars[i].is_whitespace() && chars[i] != close_ch {
        i += 1;
      }
      let fname = chars[fname_start..i]
        .iter()
        .collect::<String>()
        .trim()
        .to_lowercase();

      // Skip whitespace and =
      while i < len && (chars[i].is_whitespace() || chars[i] == '=') {
        i += 1;
      }
      if i >= len || chars[i] == close_ch {
        break;
      }

      // Read field value
      let (value, balanced) = read_bib_value(&chars, &mut i, close_ch);
      if !fname.is_empty() {
        fields.push((fname.clone(), value));
      }
      if !balanced {
        // BibTeX errors and resyncs at the next entry; mirror that loudly
        // instead of silently swallowing every later entry.
        crate::Warn!(
          "bibtex",
          "unbalanced",
          "Unbalanced braces in field '{}' of entry '{}'; resyncing at the next '@'",
          fname,
          key
        );
        break;
      }

      // Skip trailing comma
      while i < len && chars[i].is_whitespace() {
        i += 1;
      }
      if i < len && chars[i] == ',' {
        i += 1;
      }
    }

    // Skip closing brace
    if i < len && chars[i] == close_ch {
      i += 1;
    }

    if !key.is_empty() {
      entries.push(BibEntry { entry_type, key, fields });
    }
  }

  entries
}

/// Read a BibTeX field value (braced, quoted, or bare number/string).
/// The bool is false when a braced value ran unbalanced to EOF or to the
/// next entry boundary (`@` at line start) — BibTeX-style error resync.
fn read_bib_value(chars: &[char], i: &mut usize, _entry_close: char) -> (String, bool) {
  let len = chars.len();
  let mut result = String::new();

  loop {
    while *i < len && chars[*i].is_whitespace() {
      *i += 1;
    }
    if *i >= len {
      break;
    }

    if chars[*i] == '{' {
      // Braced value — handle nested braces. An unbalanced value must not
      // silently swallow every later entry: stop at the next entry boundary
      // (`@` at line start, BibTeX's own resync point) and report it.
      *i += 1;
      let mut depth = 1;
      while *i < len && depth > 0 {
        if chars[*i] == '{' {
          depth += 1;
        } else if chars[*i] == '}' {
          depth -= 1;
          if depth == 0 {
            *i += 1;
            break;
          }
        } else if chars[*i] == '@' && *i > 0 && chars[*i - 1] == '\n' {
          return (result, false); // leave *i AT the '@' for resync
        }
        result.push(chars[*i]);
        *i += 1;
      }
      if depth > 0 {
        return (result, false); // ran to EOF unbalanced
      }
    } else if chars[*i] == '"' {
      // Quoted value
      *i += 1;
      while *i < len && chars[*i] != '"' {
        if chars[*i] == '{' {
          // Nested braces in quoted strings: a `"` inside them is literal.
          // KEEP the braces — BibTeX treats them as grouping that stays
          // significant for name splitting (`author = "{W3C Group}"` must
          // still read as a corporate author).
          result.push('{');
          *i += 1;
          let mut depth = 1;
          while *i < len && depth > 0 {
            if chars[*i] == '{' {
              depth += 1;
            } else if chars[*i] == '}' {
              depth -= 1;
              if depth == 0 {
                result.push('}');
                *i += 1;
                break;
              }
            }
            result.push(chars[*i]);
            *i += 1;
          }
        } else {
          result.push(chars[*i]);
          *i += 1;
        }
      }
      if *i < len && chars[*i] == '"' {
        *i += 1;
      }
    } else if chars[*i].is_alphanumeric() {
      // Bare word or number
      while *i < len && (chars[*i].is_alphanumeric() || chars[*i] == '-' || chars[*i] == '_') {
        result.push(chars[*i]);
        *i += 1;
      }
    } else {
      break;
    }

    // Check for # concatenation
    while *i < len && chars[*i].is_whitespace() {
      *i += 1;
    }
    if *i < len && chars[*i] == '#' {
      *i += 1;
      continue;
    }
    break;
  }

  (result, true)
}

thread_local! {
  // Per-document backstop: after this many failed field digests, stop
  // interpreting (raw passthrough) instead of flooding the log. Reset at
  // each .bib conversion.
  static BIB_INTERPRET_FAILURES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
const MAX_BIB_INTERPRET_FAILURES: usize = 50;

/// Interpret a BibTeX field value through the REAL TeX engine — Perl's
/// `ToString(Digest(Tokenize($x)))` idiom. The post-processor runs in the
/// same process right after the core conversion, so the engine state is
/// still live: accents (`{\'\i}`), letter macros (`\ss`), ties (`~` ->
/// non-breaking space) and — crucially — macros DEFINED BY THE ARTICLE'S
/// CLASS (`\aap` etc.) all take their true meaning. This replaces a
/// ~150-line hand-rolled transliterator (user directive 2026-07-04: reuse
/// our TeX interpretation, no special-case TeX parser). Perl instead spins
/// a recursive BibTeX.pool session with class/package preloads — the full
/// re-port is tracked in SYNC_STATUS ("MakeBibliography: convert raw .bib
/// through the core engine"). Plain strings skip the engine round-trip; on
/// any engine failure the raw text passes through unchanged (the
/// pre-decoder behavior).
fn interpret_tex_text(s: &str) -> String {
  if !s.contains('\\') && !s.contains('~') && !s.contains('$') {
    return s.to_string();
  }
  if BIB_INTERPRET_FAILURES.with(|c| c.get()) > MAX_BIB_INTERPRET_FAILURES {
    return s.to_string();
  }
  // Diagnostic policy (user, 2026-07-04, final form): with live-state
  // interpretation the diagnostics are trustworthy, so Warn!/Error! from
  // a field digest report at NATIVE severity and count against the
  // document — matching Perl's MergeStatus accounting (Common/Error.pm
  // L669; its recursive session also prints at native severity). ONLY
  // Fatal! is demoted (to a counted+logged Error): a broken bibliography
  // must never lose the document. The Err return still aborts just this
  // field's digest — the raw text passes through below.
  let prev = latexml_core::common::error::set_demote_fatals(true);
  let interpreted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    latexml_core::stomach::digest(latexml_core::mouth::tokenize(s)).map(|d| d.to_string())
  }));
  latexml_core::common::error::set_demote_fatals(prev);
  match interpreted {
    Ok(Ok(text)) => text,
    _ => {
      let n = BIB_INTERPRET_FAILURES.with(|c| {
        let n = c.get() + 1;
        c.set(n);
        n
      });
      if n == MAX_BIB_INTERPRET_FAILURES + 1 {
        Warn!(
          "bibliography",
          "interpret",
          format!(
            "Disabling TeX interpretation of bibliography fields after {} failures; \
             remaining fields pass through raw.",
            MAX_BIB_INTERPRET_FAILURES
          )
        );
      }
      s.to_string()
    },
  }
}

/// Percent-encode a DOI into its canonical absolute resolver URL.
/// Mirrors the engine's `\bib@field@default@doi` (bibtex.rs; Perl
/// BibTeX.pool L750-756): `[^0-9a-zA-Z./\-+]` chars are %-encoded.
fn doi_href(doi: &str) -> String {
  let mut href = String::from("https://doi.org/");
  for c in doi.trim().chars() {
    if c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '-' | '+') {
      href.push(c);
    } else {
      let mut buf = [0u8; 4];
      for &b in c.encode_utf8(&mut buf).as_bytes() {
        href.push_str(&format!("%{:02X}", b));
      }
    }
  }
  href
}

/// Bibliography links are external: a scheme-less href would resolve
/// relative to the article. Prepend https:// when no scheme is present.
fn force_absolute_url(url: &str) -> String {
  let u = url.trim();
  if u.is_empty() || u.contains("://") || u.starts_with("mailto:") {
    u.to_string()
  } else {
    format!("https://{}", u)
  }
}

/// Strip outer braces from a BibTeX field value.
/// "{My Title}" → "My Title"
fn strip_braces(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  for c in s.chars() {
    if c != '{' && c != '}' {
      result.push(c);
    }
  }
  result
}

/// True if `s` is a single brace group wrapping its entire content, e.g.
/// `{W3C Math Working Group}` — i.e. the opening brace's match is the final
/// character. Used to detect brace-protected corporate author names.
fn is_braced_group(s: &str) -> bool {
  let s = s.trim();
  if s.len() < 2 || !s.starts_with('{') || !s.ends_with('}') {
    return false;
  }
  let mut depth = 0i32;
  for (i, b) in s.bytes().enumerate() {
    match b {
      b'{' => depth += 1,
      b'}' => {
        depth -= 1;
        if depth == 0 {
          return i == s.len() - 1;
        }
      },
      _ => {},
    }
  }
  false
}

/// Parse BibTeX author field into individual author names.
/// "Lastname, Firstname and Lastname2, Firstname2" → vec of (surname, givenname)
fn parse_bib_authors(authors_str: &str) -> Vec<(String, String)> {
  let mut result = Vec::new();
  // Perl processBibNameList (BibTeX.pool.ltxml L872+): names split on the
  // STANDALONE word "and" (case-insensitive, any whitespace incl. newlines),
  // over brace-respecting words — `{Barnes and Noble}` is ONE author, and a
  // line-wrapped "...Smith and\nJones..." still splits.
  let mut parts: Vec<String> = Vec::new();
  let mut cur = String::new();
  let mut depth: usize = 0;
  for tok in authors_str.split_whitespace() {
    if depth == 0 && tok.eq_ignore_ascii_case("and") {
      parts.push(std::mem::take(&mut cur));
      continue;
    }
    if !cur.is_empty() {
      cur.push(' ');
    }
    cur.push_str(tok);
    for c in tok.chars() {
      match c {
        '{' => depth += 1,
        '}' => depth = depth.saturating_sub(1),
        _ => {},
      }
    }
  }
  parts.push(cur);
  for part in &parts {
    let part = part.trim();
    if part.is_empty() {
      continue;
    }
    // Corporate/institutional author wrapped in braces, e.g.
    // `{W3C Math Working Group}`. BibTeX treats a fully brace-protected name as
    // a single unit (a "last" name with no first/von parts), so keep it verbatim
    // as the surname instead of splitting "last word = surname" (which produced
    // "W. M. W. Group"). Witness 2605.16562.
    if is_braced_group(part) {
      result.push((strip_braces(part).trim().to_string(), String::new()));
      continue;
    }
    let clean = strip_braces(part);
    if let Some((surname, given)) = clean.split_once(',') {
      result.push((surname.trim().to_string(), given.trim().to_string()));
    } else {
      // "Firstname Lastname" format — last word is surname
      let words: Vec<&str> = clean.split_whitespace().collect();
      if words.len() >= 2 {
        let surname = words.last().unwrap().to_string();
        let given = words[..words.len() - 1].join(" ");
        result.push((surname, given));
      } else if words.len() == 1 {
        result.push((words[0].to_string(), String::new()));
      }
    }
  }
  result
}

/// Convert a raw `.bib` file to a PostDocument containing ltx:bibentry elements.
///
/// This is a simplified port of Perl's `convertBibliography` that directly parses
/// BibTeX instead of spawning a full LaTeXML sub-session with BibTeX.pool.
fn convert_bib_file_to_xml(bib_path: &str) -> Result<PostDocument, String> {
  BIB_INTERPRET_FAILURES.with(|c| c.set(0));
  let content = std::fs::read_to_string(bib_path)
    .map_err(|e| format!("Failed to read '{}': {}", bib_path, e))?;

  let entries = parse_bibtex(&content);
  Info!(
    "bibtex",
    "parse",
    "Parsed {} BibTeX entries from '{}'",
    entries.len(),
    bib_path
  );

  // Build XML document with ltx:bibentry elements
  let mut xml = String::from(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <bibliography xmlns=\"http://dlmf.nist.gov/LaTeXML\">\n",
  );

  for entry in &entries {
    xml.push_str(&format!(
      "  <bibentry key=\"{}\" type=\"{}\">\n",
      xml_escape(&entry.key),
      xml_escape(&entry.entry_type),
    ));

    // Process fields into ltx:bib-* elements
    for (field, value) in &entry.fields {
      // FIRST STAGE toward Perl parity — NOT yet a faithful match. Perl's
      // BibTeX.pool.ltxml has ~28 `\bib@field@default@*` constructors that
      // digest their values with live catcodes, INCLUDING abstract (L708),
      // keywords (L732), annote (L680), series, institution, ... So Perl DOES
      // raise the undefined-macro errors these fields carry and MergeStatus'es
      // them into the document; this 13-field whitelist deliberately under-
      // reports vs Perl for now, to avoid the junk-field error floods of
      // ADS/Zotero exports (2605.02213: a .bib with 291 abstract fields ->
      // 500+ errors on an otherwise-clean paper). Widening this set to Perl's
      // full rendering-field coverage is an open parity task (SYNC_STATUS,
      // "bibliography field-interpretation parity"). url/doi/eprint render but
      // are verbatim identifiers; isbn/issn are plain digits.
      let interpreted;
      let value = match field.as_str() {
        "author" | "editor" | "title" | "year" | "journal" | "journaltitle" | "booktitle"
        | "volume" | "number" | "issue" | "pages" | "publisher" | "note" => {
          interpreted = interpret_tex_text(value);
          &interpreted
        },
        _ => value,
      };
      let clean = strip_braces(value);
      match field.as_str() {
        "author" => {
          let authors = parse_bib_authors(value);
          for (surname, given) in authors {
            xml.push_str("    <bib-name role=\"author\">");
            xml.push_str(&format!(
              "<surname>{}</surname>",
              xml_escape(&strip_braces(&surname))
            ));
            if !given.is_empty() {
              xml.push_str(&format!(
                "<givenname>{}</givenname>",
                xml_escape(&strip_braces(&given))
              ));
            }
            xml.push_str("</bib-name>\n");
          }
        },
        "editor" => {
          let editors = parse_bib_authors(value);
          for (surname, given) in editors {
            xml.push_str("    <bib-name role=\"editor\">");
            xml.push_str(&format!(
              "<surname>{}</surname>",
              xml_escape(&strip_braces(&surname))
            ));
            if !given.is_empty() {
              xml.push_str(&format!(
                "<givenname>{}</givenname>",
                xml_escape(&strip_braces(&given))
              ));
            }
            xml.push_str("</bib-name>\n");
          }
        },
        "title" => {
          xml.push_str(&format!(
            "    <bib-title>{}</bib-title>\n",
            xml_escape(&clean)
          ));
        },
        "year" => {
          xml.push_str(&format!(
            "    <bib-date role=\"publication\">{}</bib-date>\n",
            xml_escape(&clean)
          ));
        },
        "journal" | "journaltitle" => {
          xml.push_str(&format!(
            "    <bib-related type=\"journal\"><bib-title>{}</bib-title></bib-related>\n",
            xml_escape(&clean)
          ));
        },
        "booktitle" => {
          xml.push_str(&format!(
            "    <bib-related type=\"book\"><bib-title>{}</bib-title></bib-related>\n",
            xml_escape(&clean)
          ));
        },
        "volume" => {
          xml.push_str(&format!(
            "    <bib-part role=\"volume\">{}</bib-part>\n",
            xml_escape(&clean)
          ));
        },
        "number" | "issue" => {
          xml.push_str(&format!(
            "    <bib-part role=\"number\">{}</bib-part>\n",
            xml_escape(&clean)
          ));
        },
        "pages" => {
          let pages_clean = clean.replace("--", "\u{2013}");
          xml.push_str(&format!(
            "    <bib-part role=\"pages\">{}</bib-part>\n",
            xml_escape(&pages_clean)
          ));
        },
        "doi" => {
          // Perl BibTeX.pool L750-756: DOIs are ALWAYS external — emit an
          // absolute https href (percent-encoding non url-safe chars), never
          // the bare identifier (which renders as dead text / a relative
          // link; witness 2605.00223 "External Links: 10.1051/...").
          xml.push_str(&format!(
            "    <bib-identifier scheme=\"doi\" href=\"{}\">{}</bib-identifier>\n",
            xml_escape(&doi_href(&clean)),
            xml_escape(&clean)
          ));
        },
        "url" => {
          // Bibliography URLs are external by nature; authors often write
          // them scheme-less ("www.x.org/...") which the browser then
          // resolves RELATIVE to the article — force an absolute https://.
          xml.push_str(&format!(
            "    <bib-url href=\"{}\">{}</bib-url>\n",
            xml_escape(&force_absolute_url(&clean)),
            xml_escape(&clean)
          ));
        },
        "isbn" => {
          xml.push_str(&format!(
            "    <bib-identifier scheme=\"isbn\">{}</bib-identifier>\n",
            xml_escape(&clean)
          ));
        },
        "issn" => {
          xml.push_str(&format!(
            "    <bib-identifier scheme=\"issn\">{}</bib-identifier>\n",
            xml_escape(&clean)
          ));
        },
        "publisher" => {
          xml.push_str(&format!(
            "    <bib-publisher>{}</bib-publisher>\n",
            xml_escape(&clean)
          ));
        },
        "note" => {
          xml.push_str(&format!(
            "    <bib-note>{}</bib-note>\n",
            xml_escape(&clean)
          ));
        },
        // Remaining fields: skip or log
        _ => {},
      }
    }

    xml.push_str("  </bibentry>\n");
  }

  xml.push_str("</bibliography>\n");

  PostDocument::new_from_string(&xml, PostDocumentOptions {
    source_directory: Some(".".to_string()),
    ..PostDocumentOptions::default()
  })
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_four_digit_year() {
    assert_eq!(extract_four_digit_year("2024"), "2024");
    assert_eq!(
      extract_four_digit_year("Published in 2024, January"),
      "2024"
    );
    assert_eq!(extract_four_digit_year("99"), "99");
    assert_eq!(extract_four_digit_year(""), "");
  }

  #[test]
  fn test_suffix_to_counter() {
    assert_eq!(suffix_to_counter("a"), 1);
    assert_eq!(suffix_to_counter("b"), 2);
    assert_eq!(suffix_to_counter("z"), 26);
    assert_eq!(suffix_to_counter("aa"), 27);
  }

  #[test]
  fn test_format_single_name() {
    assert_eq!(format_single_name("Smith, John"), "J. Smith");
    assert_eq!(format_single_name("Smith, J."), "J. Smith");
    assert_eq!(format_single_name("Smith, John Robert"), "J. R. Smith");
    assert_eq!(format_single_name("Smith"), "Smith");
  }

  #[test]
  fn test_format_authors_text() {
    assert_eq!(format_authors_text("Smith"), "Smith");
    assert_eq!(
      format_authors_text("Smith, John and Doe, Jane"),
      "J. Smith and J. Doe"
    );
    assert_eq!(
      format_authors_text("Smith, J. and Doe, J. and Roe, R."),
      "J. Smith, J. Doe, and R. Roe"
    );
  }

  #[test]
  fn test_fmt_spec_coverage() {
    // Ensure all format types produce non-empty specs
    for fmt in &[
      "article",
      "book",
      "incollection",
      "report",
      "thesis",
      "website",
      "software",
    ] {
      let specs = get_fmt_spec(fmt);
      assert!(
        !specs.is_empty(),
        "FMT_SPEC for '{}' should not be empty",
        fmt
      );
    }
  }
}

#[cfg(test)]
mod bib_parse_tests {
  use super::*;

  #[test]
  fn corporate_author_brace_protected() {
    // Braced groups protect the inner "and" and read as single corporate names.
    let a = parse_bib_authors("{Barnes and Noble} and Smith, John");
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].0, "Barnes and Noble");
    assert_eq!(a[1].0, "Smith");
  }

  #[test]
  fn newline_wrapped_and_splits() {
    let a = parse_bib_authors("Smith, John and\nJones, Mary");
    assert_eq!(a.len(), 2);
  }

  #[test]
  fn quoted_field_keeps_braces() {
    let entries = parse_bibtex("@article{k, author = \"{W3C Math Working Group}\", title={T}}");
    assert_eq!(entries.len(), 1);
    let author = &entries[0]
      .fields
      .iter()
      .find(|(n, _)| n == "author")
      .unwrap()
      .1;
    assert!(
      author.starts_with('{') && author.ends_with('}'),
      "braces kept: {author}"
    );
    let names = parse_bib_authors(author);
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].0, "W3C Math Working Group");
  }

  #[test]
  fn unbalanced_entry_resyncs_at_next_at() {
    let src = "@article{bad, title = {unclosed\n}\n@article{good, title={ok}, author={A}}";
    let entries = parse_bibtex(src);
    // The good entry must survive the bad one's unbalanced brace.
    assert!(
      entries.iter().any(|e| e.key == "good"),
      "entries: {:?}",
      entries.iter().map(|e| e.key.clone()).collect::<Vec<_>>()
    );
  }
}
