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
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  document::{NodeData, PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::{ProcessResult, Processor, find_documentclass_and_packages},
  radix::radix_alpha,
};

// ================================================================================
// The injected raw-`.bib` converter
// ================================================================================

/// A raw bibliography source that still needs converting.
///
/// Perl accumulates these across all `\bibliography{}` names and converts them
/// in ONE pass (`MakeBibliography.pm` L146-165), so an `@string` macro defined in
/// the first file is in scope for the last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawBibSource {
  /// A resolved path to a `.bib` file on disk.
  Path(String),
  /// Literal BibTeX data, without the `literal:` protocol prefix.
  Literal(String),
}

/// Everything the recursive BibTeX session needs, gathered post-side.
///
/// Port of the inputs `convertBibliography` (`MakeBibliography.pm` L180-242)
/// assembles before calling `LaTeXML->get_converter`.
#[derive(Debug, Clone)]
pub struct BibConversionRequest {
  /// The raw sources, in document order.
  pub sources:      Vec<RawBibSource>,
  /// `$$doc{searchpaths}` — the inner session's `paths`.
  pub search_paths: Vec<String>,
  /// `[options]name.cls` / `[options]name.sty` specs recovered from the
  /// document's `<?latexml?>` processing instructions (Perl L186-199): the
  /// bibliography's fields are the article's TeX, so they need the article's
  /// class and packages to mean anything.
  pub preloads:     Vec<String>,
  /// The keys the document cites, or `None` to convert every entry.
  ///
  /// A `.bib` is a library: the converter should digest the cited entries (and
  /// their `crossref`/`\cite` closure) rather than the whole file, exactly as
  /// `bibtex(1)` does from the `.aux`'s `\citation` records. `None` means no
  /// citation record was available, or the document said `\nocite{*}`.
  pub wanted_keys:  Option<Vec<String>>,
}

/// Convert raw BibTeX into a document containing `ltx:bibentry` elements.
///
/// `latexml_post` cannot run this itself: a conversion needs the model loader
/// and `convert_document`, which live in `latexml_oxide` (depending the other
/// way would be a cycle). The orchestrating binary installs an implementation
/// with [`set_bib_converter`] before running the pipeline.
pub type BibConverterFn = fn(&BibConversionRequest) -> Option<PostDocument>;

thread_local! {
  static BIB_CONVERTER: std::cell::Cell<Option<BibConverterFn>> =
    const { std::cell::Cell::new(None) };
}

/// Install the recursive-BibTeX-session implementation for this thread.
pub fn set_bib_converter(convert: BibConverterFn) {
  BIB_CONVERTER.with(|slot| slot.set(Some(convert)));
}

fn bib_converter() -> Option<BibConverterFn> { BIB_CONVERTER.with(|slot| slot.get()) }

/// Citation style.
#[derive(Debug, Clone, PartialEq)]
pub enum CitationStyle {
  /// Numeric: `[1]`, `[2]`, ...
  Numbers,
  /// Author-Year: Author (Year)
  AuthorYear,
  /// Alphabetic: `[ABC24]`
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
  ///
  /// `wanted_keys` is forwarded to the recursive raw-`.bib` conversion so it
  /// digests only the cited entries; see [`BibConversionRequest::wanted_keys`].
  fn get_bibliographies(
    &self,
    doc: &PostDocument,
    wanted_keys: Option<&Vec<String>>,
  ) -> Vec<PostDocument> {
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
    // Raw sources accumulate rather than converting one-by-one, so that a
    // single recursive session sees them all (Perl L110, L146-165): `@string`
    // macros are file-scoped in BibTeX but shared across a combined payload,
    // and one session is also far cheaper than N.
    let mut rawbibs: Vec<RawBibSource> = Vec::new();

    for bib in &bibnames {
      let mut loaded = false;
      // Perl reaches the raw-`.bib` lookup only from inside the
      // `.bib`/`\bibliography` branch (`MakeBibliography.pm` L126-140), and it
      // is that branch alone whose failure is an `Error`. Hoisting the lookup
      // out of the branch (below) is what lost the raise, so carry the branch
      // condition along instead.
      let is_bib_style = bib.ends_with(".bib") || bib.ends_with(".bib.xml") || from_bibliography;

      // Literal BibTeX handed in on the command line (Perl L121-123).
      if let Some(data) = bib.strip_prefix("literal:") {
        rawbibs.push(RawBibSource::Literal(data.to_string()));
        continue;
      }

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
      else if is_bib_style {
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

      // If not loaded yet, queue the raw `.bib` for the recursive session.
      // `kpsewhich` is the last resort, matching Perl's
      // `pathname_find(...) || pathname_kpsewhich($bib)` (L133-134) — a `.bib`
      // installed in the host texmf tree (revtex's `apsrev`, say) resolves
      // there and nowhere else.
      if !loaded {
        let bib_file = if from_bibliography && !bib.ends_with(".bib") {
          format!("{}.bib", bib)
        } else {
          bib.clone()
        };
        if let Some(bib_path) = find_file(&bib_file, search_paths)
          .or_else(|| latexml_core::util::pathname::kpsewhich(&[bib_file.as_str()]))
        {
          rawbibs.push(RawBibSource::Path(bib_path));
          loaded = true;
        } else if is_bib_style {
          // Perl L138-140 raises this BEFORE falling through to the Info
          // below, and both reach the log. Dropping it is what let a whole
          // family of lost bibliographies report telemetry `ok`: the shipped
          // `bu1.bbl`/`bu2.bbl` of a bibunits paper are never consulted, the
          // named `.bib` does not exist, and the only trace was an Info.
          // 14 silent papers in the 2605+2606 sandboxes, 691 corpus-wide;
          // witness 2606.04416 (same loss under Perl, which DOES raise here).
          // Audit `docs/parity/BIB_ABSENCE_AUDIT_2026-07-29.md` family F2.
          Error!(
            "missing_file",
            bib,
            "Couldn't find Bibliography '{}'\nSearchpaths were {}",
            bib,
            search_paths.join(",")
          );
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

    // Lastly, if we found any raw .bib files / literal data, convert them —
    // in ONE pass (Perl L146-165).
    if !rawbibs.is_empty() {
      match bib_converter() {
        Some(convert) => {
          let (class, packages) = find_documentclass_and_packages(doc);
          // Perl L188-199: `[$classoptions]$class.cls` then `[$options]$pkg.sty`.
          let mut preloads = Vec::with_capacity(1 + packages.len());
          preloads.push(if class.options.is_empty() {
            format!("{}.cls", class.name)
          } else {
            format!("[{}]{}.cls", class.options, class.name)
          });
          for pkg in &packages {
            preloads.push(if pkg.options.is_empty() {
              format!("{}.sty", pkg.name)
            } else {
              format!("[{}]{}.sty", pkg.options, pkg.name)
            });
          }
          let request = BibConversionRequest {
            sources: rawbibs,
            search_paths: search_paths.to_vec(),
            preloads,
            wanted_keys: wanted_keys.cloned(),
          };
          match convert(&request) {
            Some(bibdoc) => bibs.push(bibdoc),
            None => Error!(
              "bibliography",
              "convert",
              "Recursive BibTeX conversion produced no bibliography"
            ),
          }
        },
        // Never silently drop the sources: an uninstalled converter would
        // otherwise render an empty References section with no diagnostic,
        // which is the exact fail-open shape this codebase forbids.
        None => Error!(
          "bibliography",
          "converter",
          "No BibTeX converter installed; cannot convert {} raw bibliography source(s) \
           (call make_bibliography::set_bib_converter before post-processing)",
          rawbibs.len()
        ),
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
  /// Fold every `ltx:bibentry` of one source document into `entries`, keyed by
  /// the lowercased bibkey. Later sources win for a repeated key, matching the
  /// upstream `getBibEntries` loop.
  fn scan_bibentries(entries: &mut HashMap<String, BibEntryData>, srcdoc: &PostDocument) {
    for bibentry in srcdoc.findnodes("//ltx:bibentry") {
      let bibkey = match bibentry.get_attribute("key") {
        Some(k) => k,
        None => continue,
      };
      let lc_key = bibkey.to_lowercase();
      // Extract citations from within this bibentry
      let citations: Vec<String> = srcdoc
        .findnodes_at(".//@bibrefs", Some(&bibentry))
        .iter()
        .filter_map(|n| {
          let val = n.get_content();
          if val.is_empty() { None } else { Some(val) }
        })
        .flat_map(|s| s.split(',').map(String::from).collect::<Vec<_>>())
        .filter(|s| !s.is_empty())
        .collect();

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
        bibentry: Some(bibentry.clone()),
      });
    }
  }

  /// The bib keys this document cites, for the recursive raw-`.bib` conversion
  /// to digest instead of the whole library.
  ///
  /// Reads the same `BIBLABEL:<list>:<key>` ObjectDB records that Step 2 of
  /// [`Self::get_bib_entries`] scans — they are written during the document
  /// conversion, so they are already complete by the time post-processing asks.
  /// Returns `None` (= convert everything) for `\nocite{*}`, and also when no
  /// `BIBLABEL` record exists at all: an empty filter and "no citation data"
  /// are indistinguishable here, and dropping every entry on a missing record
  /// would be a silent, unrecoverable loss.
  fn cited_keys(&self, lists: &[&str]) -> Option<Vec<String>> {
    let mut keys: Vec<String> = Vec::new();
    for db_key in self.db.get_keys() {
      let Some(rest) = db_key.strip_prefix("BIBLABEL:") else {
        continue;
      };
      let Some((list, bibkey)) = rest.split_once(':') else {
        continue;
      };
      if !lists.contains(&list) {
        continue;
      }
      if bibkey == "*" {
        return None; // `\nocite{*}` — the document wants the whole library.
      }
      keys.push(bibkey.to_string());
    }
    (!keys.is_empty()).then_some(keys)
  }

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
    let bib_docs = self.get_bibliographies(doc, self.cited_keys(&lists).as_ref());
    for bibdoc in &bib_docs {
      Self::scan_bibentries(&mut entries, bibdoc);
    }
    // Also scan the MAIN document for INLINE `ltx:bibentry` elements.
    // OXIDIZED_DESIGN #57 (beyond-Perl). amsrefs writes its bibliography
    // straight into the document — `\begin{bibdiv}\begin{biblist}
    // \bib{key}{article}{...}` — instead of into an external `.bib`, so the
    // entries are already children of `ltx:biblist` and there is no `@files`
    // for `getBibliographies` to resolve. Upstream `getBibEntries` only ever
    // scans `getBibliographies()`, and `process` then deletes every
    // `//ltx:bibentry`, so an amsrefs bibliography is silently dropped WHOLE:
    // empty References plus every `\cite` left dangling, with zero errors
    // reported. Confirmed identical on installed AND vendored Perl 0.8.8
    // (rev 51fea96a) — a shared upstream bug (KNOWN_PERL_ERRORS #49), fixed
    // here rather than reproduced. Papers with an external `.bib` carry no
    // inline `ltx:bibentry`, so this scan is a no-op for them.
    // Witness 2605.01646 (AIPFa.tex, 23 entries), 2605.00783, 2605.03852.
    Self::scan_bibentries(&mut entries, doc);

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
        }
      }
    }

    // `\nocite{*}` asks for the WHOLE library — that is bibtex's own semantic
    // for the star key, and `cited_keys` above already returns None for it so
    // every entry is read. Queue them all.
    //
    // Queueing per BIBLABEL record cannot do this: in a document whose only
    // citation is `\nocite{*}` the sole record IS `*`, Step 3 skips that key by
    // name, and the result was "N bibentries, **0 cited**" over an empty
    // References list — while `pdflatex`+`bibtex` on the same source prints the
    // full list. 7 papers of the 2605+2606 residual are `\nocite{*}`-only;
    // 6-line reproducer: `\nocite{*}` + `\bibliography{t}` with a 2-entry
    // `.bib` gave 0 items against bibtex's 2. Perl skips the star key the same
    // way (`MakeBibliography.pm` L279-313), so this is deliberately BEYOND Perl.
    if cite_star {
      let mut all: Vec<String> = entries.values().map(|e| e.bib_key.clone()).collect();
      all.sort();
      queue.extend(all);
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
            // Perl L318-328 computes TWO name strings and uses them for
            // different things: `$sortnames` (every "Surname Givenname",
            // joined) keys the sort, while `$names` (the SHORT form — "A and
            // B", "A et al") keys `{ay}` and `{initial}`. `extract_names`
            // already carries Perl's bib-key → bib-title fallback (both
            // strings then hold the same text), so no second fallback chain is
            // needed here.
            let (sort_names, short_names, _full_names) = extract_names(doc, bibentry);
            entry.sort_names = sort_names.clone();
            entry.authors_short = short_names.clone();

            // Year
            let date_content =
              PostDocument::findnodes_foreign("ltx:bib-date[@role='publication']", bibentry)
                .into_iter()
                .next()
                .map(|n| n.get_content())
                .unwrap_or_default();
            let year = extract_four_digit_year(&date_content);
            entry.year = year.clone();

            // The {ay}/sortkey date is the publication date ALONE, on purpose.
            // Perl L330 unions it with `ltx:bib-type`
            // (`ltx:bib-date[@role="publication"] | ltx:bib-type`, whichever
            // comes first in document order); querying the two separately is a
            // settled decision here, so that a `type` field cannot displace the
            // year — see BIBLIOGRAPHY_WORKLIST, "bib-type is safe to emit".
            // A "date else type" stand-in was written while porting the sortkey
            // and REMOVED on review: it contradicts that decision, it is not
            // Perl's rule anyway for an entry carrying both, and it would move
            // sort keys (and with them numbering) with no test or witness.

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

            // Author+year for suffix detection — Perl L336 `"$names.$date"`,
            // i.e. the SHORT names. Using the full sort-names here silently
            // disabled disambiguation for every 3+-author paper: two entries
            // that Perl groups as "Smith et al.1999" carry different coauthor
            // lists, so their full-name keys never collide and neither entry
            // ever got its `a`/`b` suffix.
            entry.author_year = format!("{}.{}", short_names, year);
            entry.initial = PostDocument::initial(&short_names, true);

            // Sort key — Perl L338, from the FULL sort-names.
            let sort_key = format!("{}.{}.{}.{}", sort_names, year, title, bibkey).to_lowercase();
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
    // Perl L357 `$doc->unisort(keys %$included)`.
    let mut sorted_keys: Vec<String> = included.keys().cloned().collect();
    unisort(&mut sorted_keys);

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

    // Numbering is NOT done here: Perl assigns it in FORMAT order
    // (`++$NUMBER` inside `formatBibEntry` L418), which is initial-major once
    // `--splitbibliography` is on. It happens in `process`, alongside the walk
    // that builds the biblists.

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

    // Perl L398 `$doc->unisort(keys %$entries)`.
    let mut sorted_keys: Vec<String> = entries.keys().cloned().collect();
    unisort(&mut sorted_keys);
    let items: Vec<NodeData> = sorted_keys
      .iter()
      .filter_map(|key| entries.get(key))
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
    // (MakeBibliography.pm L407-415; NS-aware read — the bare form always
    // returned None, so every bibitem fell to the .bibN numbering fallback)
    let id = if let Some(ref bibentry) = entry.bibentry {
      let orig_id = crate::document::get_xml_id(bibentry).unwrap_or_default();
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
    // Author-year only: keep the first block but drop its (redundant) year field.
    let mut drop_first_block_year = false;
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
        // Author-year refnum. Perl (MakeBibliography.pm else-branch L505-517)
        // builds this label from `do_authors`→`do_names`, i.e. EVERY author,
        // with "et al." only for a literal BibTeX "and others"; and it then
        // drops the first block ("Skip redundant 1st block!!") because the
        // authors are already in the label.
        //
        // We use the SHORT form instead — intentional divergence
        // OXIDIZED_DESIGN #71. On a collaboration paper Perl's rule yields a
        // 5104-character citation label (witness arXiv 2607.21432, reported as
        // arXiv/html_feedback#6797) and, with the block skipped, that label is
        // essentially the whole entry.
        //
        // pdflatex is the ground truth for the shape. `aa.bst` over the witness
        // emits
        //   \bibitem[{Abitbol {et~al.}(2025)Abitbol, Abril-Cabezas, …}]{…}
        //   Abitbol, M., Abril-Cabezas, I., Adachi, S., {et~al.} 2025, JCAP …
        // — natbib's SHORT form is the citation label, the long surname list is
        // only natbib's optional full-author form (never printed), and the
        // authors live in the entry BODY. So: short label, and keep the block.
        //
        // Perl already carries the right helper — `do_names_short`
        // (MakeBibliography.pm L586) — defined and never called; `do_names_short`
        // below is its port. Perl's own role="authors" tag likewise truncates at
        // >2 (L433-437), so the full-list label was internally inconsistent.
        skip_first_block = false;
        drop_first_block_year = true;
        let suffix = entry.suffix.as_deref().unwrap_or("");
        let mut refnum_children: Vec<NodeData> = if let Some(ref bibentry) = entry.bibentry {
          let authors = PostDocument::findnodes_foreign("ltx:bib-name[@role='author']", bibentry);
          if !authors.is_empty() {
            do_names_short(authors)
          } else {
            let editors = PostDocument::findnodes_foreign("ltx:bib-name[@role='editor']", bibentry);
            if !editors.is_empty() {
              do_editors_a(editors)
            } else {
              // Perl: $keytag->childNodes.
              let key = PostDocument::findnodes_foreign("ltx:bib-key", bibentry)
                .into_iter()
                .next()
                .map(|k| k.get_content())
                .unwrap_or_else(|| entry.bib_key.clone());
              vec![NodeData::Text(key)]
            }
          }
        } else {
          // No bibentry XML (ObjectDB path): the pre-computed full-authors
          // string, else the abbreviated string, else the key.
          let s = if !entry.authors_full.is_empty() {
            entry.authors_full.clone()
          } else if !entry.authors_short.is_empty() {
            entry.authors_short.clone()
          } else {
            entry.bib_key.clone()
          };
          vec![NodeData::Text(s)]
        };
        // Perl always wraps the year part in " ( … )": @year+suffix, else the
        // type tag (the L482 style guard guarantees one of them is present).
        let year_text = if !entry.year.is_empty() {
          format!("{}{}", entry.year, suffix)
        } else if let Some(ref bibentry) = entry.bibentry {
          PostDocument::findnodes_foreign("ltx:bib-type", bibentry)
            .into_iter()
            .next()
            .map(|t| t.get_content())
            .unwrap_or_default()
        } else {
          String::new()
        };
        refnum_children.push(NodeData::Text(format!(" ({})", year_text)));
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
    let blocks = self.format_blocks(doc, entry, skip_first_block, drop_first_block_year);
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
        // Perl L497-500: `join('', map { substr($_->textContent, 0, 1) })`,
        // truncated to `substr($aa, 0, 3) . "+"` past three. Both are
        // CHARACTER operations, and neither uppercases — only the single-name
        // branch below carries Perl's `uc`. Byte indexing here used to panic
        // outright on a multi-byte initial (`Ångström`), which the citestyle
        // repair makes reachable for every `\bibliographystyle{alpha}`
        // document rather than the handful that spelled the style `alpha`.
        let initials: Vec<char> = surnames
          .iter()
          .map(|n| n.get_content().chars().next().unwrap_or('?'))
          .collect();
        if initials.len() > 3 {
          format!("{}+", initials[..3].iter().collect::<String>())
        } else {
          initials.iter().collect::<String>()
        }
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
    drop_first_year: bool,
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
        // Author-year: the refnum label already reads "Author (Year)", so the
        // first block repeating the year would print it twice in the one entry.
        // Perl avoids that by dropping the whole block (and with it the author
        // list); we keep the block and drop only the redundant field. Matches
        // the biblatex author-year path already shipped here, whose entries
        // render as `[Smith (2020)]  John Smith  “A study of things” …` —
        // label carries the year, author block does not. See OXIDIZED_DESIGN #71.
        if drop_first_year && i == 0 && field_spec.class == "year" {
          continue;
        }
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
      // Perl L481 is `$STYLE{citestyle} || 'numbers'`, and `||` is falsy on the
      // EMPTY string as well as on absent — so an empty attribute must reach
      // `numbers`, not the `else` branch. Before the mapping repair below an
      // empty value fell through to `_ => Numbers` by luck; with `_` now
      // meaning author-year it has to be excluded explicitly, or the repair
      // would introduce an unfaithfulness of its own. Our own engine cannot
      // emit `citestyle=""` (`Document::set_attribute` skips empty values), but
      // `latexml_post` also consumes XML it did not produce.
      let citestyle_str = bib
        .get_attribute("citestyle")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "numbers".to_string());
      // Perl L481-517 branches on exactly three cases, and the mapping was
      // inverted here: `AY` is the ABBREVIATED `[AS64]` label (L485, class
      // `ltx_bib_abbrv`), NOT the spelled-out author-year one, and ANY other
      // non-`numbers` value takes the author-year `else` branch rather than
      // falling back to numbers. `\bibliographystyle{alpha}` sets
      // `CITE_STYLE=AY` (`latex_constructs.rs::lookup_bibstyle_params`, Perl
      // `$BIBSTYLES` L3953-3961), so every alpha-styled document was getting
      // author-year refnums where Perl — and the `.bst` the author chose —
      // give `[AS64]`.
      let style = match citestyle_str.as_str() {
        "numbers" => CitationStyle::Numbers,
        "AY" => CitationStyle::Alpha,
        _ => CitationStyle::AuthorYear,
      };

      // bib_docs must be kept alive as long as entries (bibentry Nodes reference them)
      let (mut entries, _bib_docs) = self.get_bib_entries(&doc, bib);
      if entries.is_empty() {
        Info!(
          "bibliography",
          "empty",
          "MakeBibliography: no entries to process"
        );
        continue;
      }

      // Perl MakeBibliography.pm L393-394: bib's id, else the document
      // element's id, else the literal 'bib'. NS-aware reads — the bare
      // forms always fell through to the literal.
      let bib_id = crate::document::get_xml_id(bib)
        .or_else(|| {
          doc
            .get_document_element()
            .as_ref()
            .and_then(crate::document::get_xml_id)
        })
        .unwrap_or_else(|| "bib".to_string());

      // Perl `local $LaTeXML::Post::MakeBibliography::NUMBER = 0` (L55) —
      // reset per `ltx:bibliography`, then incremented once per entry as it is
      // FORMATTED (L418). Under `--splitbibliography` that walk is
      // initial-major, so the counter follows the initial groups rather than
      // the document-global sortkey order; the two coincide whenever an
      // entry's `initial` is the first letter of its sortkey, which is the
      // common case but not guaranteed (`initial` skips leading non-letters).
      //
      // NOTE ON REACH: `self.split` is currently always false — `post.rs`
      // constructs this processor with `split = false` and
      // `--splitbibliography` sits in the deferred CLI cluster. So moving the
      // counter here changes NO production output today; the non-split walk
      // numbers in the same order the old in-`get_bib_entries` pass did. It is
      // done for when that flag lands, and because the counter belongs with
      // the walk it counts.
      let mut number = 0u32;

      if self.split {
        // Split by initial letter
        let mut by_initial: HashMap<String, Vec<String>> = HashMap::default();
        for (key, entry) in &entries {
          by_initial
            .entry(entry.initial.clone())
            .or_default()
            .push(key.clone());
        }
        let mut initials: Vec<String> = by_initial.keys().cloned().collect();
        initials.sort();
        for group in by_initial.values_mut() {
          unisort(group);
        }
        for initial in &initials {
          // Number this group, then format it — the order Perl's single
          // `++$NUMBER`-as-you-format walk produces.
          // The keys came out of `entries` a few lines above, so a miss is a
          // logic error, not input-dependent — `debug_assert` makes it loud in
          // dev/test without risking a production abort (`maxperf` sets
          // `panic = "abort"`). Incrementing INSIDE the lookup also means a
          // miss could never silently burn a number and shift the whole list.
          for key in &by_initial[initial] {
            debug_assert!(entries.contains_key(key), "grouped key {key} left entries");
            if let Some(entry) = entries.get_mut(key) {
              number += 1;
              entry.number = number;
            }
          }
          // Build a subset HashMap for this initial
          let subset: HashMap<String, BibEntryData> = by_initial[initial]
            .iter()
            .filter_map(|k| entries.get(k).map(|e| (k.clone(), clone_entry(e))))
            .collect();
          let biblist = self.make_bibliography_list(&doc, &bib_id, Some(initial), &subset, &style);
          let mut bib_mut = bib.clone();
          doc.add_nodes(&mut bib_mut, &[biblist]);
        }
      } else {
        let mut sorted_keys: Vec<String> = entries.keys().cloned().collect();
        unisort(&mut sorted_keys);
        for key in &sorted_keys {
          debug_assert!(entries.contains_key(key), "sorted key {key} left entries");
          if let Some(entry) = entries.get_mut(key) {
            number += 1;
            entry.number = number;
          }
        }
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
        // Compute the same ID as format_bib_entry (NS-aware, same as there)
        let bibitem_id = if let Some(ref bibentry) = entry.bibentry {
          let orig_id = crate::document::get_xml_id(bibentry).unwrap_or_default();
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

    // Register the enclosing `ltx:biblist` too — Perl's Scan runs AFTER
    // MakeBibliography and registers EVERY id'd node, which is what lets
    // `CrossRef::fill_in_frags` ("Any nodes with an ID will get a fragid",
    // CrossRef.pm L312-324) stamp `fragid` on it. This port registers the
    // bibitems by hand at this seam but never the list, so the list alone
    // reached HTML with no `id` — the XSLT's `add_id` emits the HTML `id`
    // from `@fragid` only, so Perl's `<ul id="bib.L1">` was a bare `<ul>`
    // here (SYNC_STATUS "Found, not fixed", 2026-07-28).
    for list_node in doc.findnodes("//ltx:biblist") {
      let Some(list_id) = crate::document::get_xml_id(&list_node) else {
        continue;
      };
      let location = doc.site_relative_destination().unwrap_or_default();
      self.db.register(&format!("ID:{}", list_id), vec![
        ("type", crate::object_db::Value::from("ltx:biblist")),
        ("location", crate::object_db::Value::from(location.as_str())),
        ("fragid", crate::object_db::Value::from(list_id.as_str())),
      ]);
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

/// The renderable content of one bibliography field node.
///
/// Perl's field formatters are all `do_any`-shaped — they receive
/// `$doc->cloneNodes(@nodes)` and return the CLONED NODES (`MakeBibliography.pm`
/// L525-531, L550-552), so an `ltx:ref`/`ltx:emph`/`ltx:Math` inside a field
/// reaches the bibitem intact. Taking `get_content()` here instead threw all of
/// that away and kept only the text, which is how `note = {\url{...}}` rendered
/// as dead text even once the field XML carried a proper link.
///
/// Fields whose content is plain text — the overwhelming majority — keep
/// returning a single `Text` node, so their output is byte-identical to before;
/// only a field that actually holds markup takes the cloning path. (Perl clones
/// the field element itself and lets the XSLT render it transparently; we clone
/// its CHILDREN, which yields the same HTML without changing our flatter
/// intermediate shape.)
fn field_content(node: &Node) -> Vec<NodeData> {
  let mut children = Vec::new();
  let mut child = node.get_first_child();
  let mut has_element = false;
  while let Some(n) = child {
    has_element |= n.get_type() == Some(libxml::tree::NodeType::ElementNode);
    child = n.get_next_sibling();
    children.push(n);
  }
  if has_element {
    children.into_iter().map(NodeData::XmlNode).collect()
  } else {
    vec![NodeData::Text(node.get_content())]
  }
}

/// Apply a formatter function to the given nodes.
///
/// Port of the various `do_*` functions.
fn apply_formatter(doc: &PostDocument, formatter: Formatter, nodes: &[Node]) -> Vec<NodeData> {
  match formatter {
    Formatter::Any => nodes.iter().flat_map(field_content).collect(),
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
      // NO disambiguation suffix here, deliberately — see KNOWN_PERL_ERRORS
      // #67. Perl `do_year` (L613-615) reads `@…::SUFFIX`, the ARRAY, while
      // `formatBibEntry` L417 binds `$…::SUFFIX`, the SCALAR: two different
      // Perl variables, so the array is always empty and the letter never
      // reaches the entry body. Measured, not assumed — same-host Perl 0.8.8
      // on `bib_alpha_style.tex` prints `[SBC99a]` as the label and ` (1999)`
      // as the body year. `alpha.bst` agrees, so this is the right output and
      // there is no gap to close; the audit item that flagged one was read off
      // the sigil.
      let suffix = "";
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
      let mut result = vec![NodeData::Text("(".to_string())];
      result.extend(nodes.iter().flat_map(field_content));
      result.push(NodeData::Text(")".to_string()));
      result
    },
    Formatter::Title => nodes.iter().flat_map(field_content).collect(),
    Formatter::ThesisType => nodes.iter().flat_map(field_content).collect(),
    Formatter::Edition => {
      let mut result: Vec<NodeData> = nodes.iter().flat_map(field_content).collect();
      result.push(NodeData::Text(" edition".to_string()));
      result
    },
    Formatter::Pages => {
      let mut result = vec![NodeData::Text("pp.\u{00A0}".to_string())]; // Non-breaking space
      result.extend(nodes.iter().flat_map(field_content));
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
    // Perl `MakeBibliography.pm:do_links` L655-667 uses
    // `$doc->cloneNodes($node->childNodes)` as the child list in EVERY branch —
    // it copies the marked-up children, it does not flatten them. Taking
    // `get_content()` (a plain-text collapse) instead silently dropped any
    // nested element: an amsrefs `review={\MR{849427}}` digests to
    // `<ltx:bib-review>Review <ltx:ref class="ltx_mathreviews" href="…">
    // MathReviews</ltx:ref></ltx:bib-review>`, and the flattening rendered a
    // dead "Review MathReviews" with the MathSciNet link gone. Witness
    // arXiv 2508.17585.
    let children: Vec<NodeData> = node
      .get_child_nodes()
      .into_iter()
      .map(NodeData::XmlNode)
      .collect();
    match tag.as_str() {
      "ltx:bib-identifier" | "ltx:bib-review" => {
        if let Some(href) = href {
          links.push(NodeData::Element {
            tag: "ltx:ref".to_string(),
            attributes: Some(HashMap::from_iter([
              ("href".to_string(), href),
              ("class".to_string(), format!("{} ltx_bib_external", scheme)),
            ])),
            children,
          });
        } else {
          links.push(NodeData::Element {
            tag: "ltx:text".to_string(),
            attributes: Some(HashMap::from_iter([(
              "class".to_string(),
              format!("{} ltx_bib_external", scheme),
            )])),
            children,
          });
        }
      },
      "ltx:bib-links" => {
        links.push(NodeData::Element {
          tag: "ltx:text".to_string(),
          attributes: Some(HashMap::from_iter([(
            "class".to_string(),
            "ltx_bib_external".to_string(),
          )])),
          children,
        });
      },
      "ltx:bib-url" => {
        if let Some(href) = href {
          links.push(NodeData::Element {
            tag: "ltx:ref".to_string(),
            attributes: Some(HashMap::from_iter([
              ("href".to_string(), href),
              ("class".to_string(), "ltx_bib_external".to_string()),
            ])),
            children,
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

/// Sort bibliography sort-keys the way Perl's `Post::unisort` does.
///
/// Perl (`Post.pm` L1399-1403) hands the keys to a `Unicode::Collate::Locale`
/// built with `variable => 'non-ignorable'` and `upper_before_lower => 1` — a
/// full UCA collation, so `Šmith` lands next to `Smith` rather than after every
/// ASCII letter, which is what a plain codepoint `sort()` produces.
///
/// **Documented approximation** (OXIDIZED_DESIGN #84): we reproduce UCA's
/// PRIMARY level for Latin script only — NFD-decompose, drop the combining
/// marks, case-fold — and break ties on the raw string. That is exact for
/// accented Latin (the input this actually sees: author surnames and titles).
/// Perl's `upper_before_lower` needs no counterpart because it is MOOT: the
/// sort keys are `to_lowercase()`d when built, so no comparison here can see a
/// case difference. It diverges from Perl for script-crossing orders,
/// non-decomposable letters (`Ø`, `Æ`, `Ł`) and locale tailorings — none of
/// which a DUCET table could be added for without a new dependency carrying
/// embedded collation data. `Post.pm` L123-128 shows Perl itself falling back
/// to a codepoint `DumbCollator` when `Unicode::Collate` is unavailable, so a
/// documented approximation stays inside the range of behaviours Perl ships.
///
/// The sort is otherwise total and deterministic: equal primary keys fall back
/// to the raw key, and the raw keys are the (unique) hash keys of the entry
/// map.
fn unisort(keys: &mut [String]) {
  keys.sort_by_cached_key(|k| (collation_primary_key(k), k.clone()));
}

/// The primary-level collation weight of a sort-key: NFD, combining marks
/// dropped, case-folded. See [`unisort`].
fn collation_primary_key(s: &str) -> String {
  use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
  s.nfd()
    .filter(|c| !is_combining_mark(*c))
    .flat_map(char::to_lowercase)
    .collect()
}

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

/// Perl MakeBibliography `do_name` (L555-566): the given-name rendered as
/// initials followed by the surname text — e.g. givenname "Aaron D." + surname
/// "Ames" → "A. D. Ames". Each whitespace-split given-name word already ending
/// in "." is kept verbatim (+ space); otherwise its first char + ". " is used.
/// (We flatten the surname to text, consistent with the rest of this file's
/// name handling; Perl clones the surname's child nodes to preserve any markup,
/// which bibliography surnames essentially never carry.)
fn do_name_text(namenode: &Node) -> String {
  let mut out = String::new();
  if let Some(given) = PostDocument::findnodes_foreign("ltx:givenname", namenode)
    .into_iter()
    .next()
  {
    for word in given.get_content().split_whitespace() {
      if word.ends_with('.') {
        out.push_str(word);
        out.push(' ');
      } else if let Some(c) = word.chars().next() {
        out.push(c);
        out.push_str(". ");
      }
    }
  }
  if let Some(surname) = PostDocument::findnodes_foreign("ltx:surname", namenode)
    .into_iter()
    .next()
  {
    out.push_str(&surname.get_content());
  }
  out
}

/// The SHORT author form used for the author-year citation label: surnames
/// only, `>2` collapsing to "First et al.".
///
/// Port of Perl's `do_names_short` (MakeBibliography.pm L586-593) — which is
/// **defined there and never called**; Perl's author-year refnum uses the full
/// `do_names` instead, producing labels thousands of characters long on
/// collaboration papers. See OXIDIZED_DESIGN #71 for why we call it, and
/// `cluster_bib_long_author_list_refnum` for the guard.
///
/// Beyond Perl's version: a trailing BibTeX `others` is dropped and forces the
/// "et al." form, so `Smith and others` reads "Smith et al." rather than
/// "Smith and others". Perl's unused helper has no `others` handling at all;
/// `do_names` (its called sibling) does, so this keeps the two consistent.
fn do_names_short(mut names: Vec<Node>) -> Vec<NodeData> {
  let surname_text = |n: &Node| -> String {
    PostDocument::findnodes_foreign("ltx:surname", n)
      .into_iter()
      .next()
      .map(|s| s.get_content())
      .unwrap_or_else(|| n.get_content())
      .trim()
      .to_string()
  };
  let mut etal = names
    .last()
    .map(|n| n.get_content().trim() == "others")
    .unwrap_or(false);
  if etal {
    names.pop();
  }
  if names.len() > 2 {
    etal = true;
  }
  let etal_span = || NodeData::Element {
    tag:        "ltx:text".to_string(),
    attributes: Some(HashMap::from_iter([(
      "class".to_string(),
      "ltx_bib_etal".to_string(),
    )])),
    children:   vec![NodeData::Text("et al.".to_string())],
  };
  match (names.len(), etal) {
    (0, _) => Vec::new(),
    (_, true) => vec![
      NodeData::Text(surname_text(&names[0])),
      NodeData::Text(" ".to_string()),
      etal_span(),
    ],
    (1, false) => vec![NodeData::Text(surname_text(&names[0]))],
    (_, false) => vec![
      NodeData::Text(surname_text(&names[0])),
      NodeData::Text(" and ".to_string()),
      NodeData::Text(surname_text(&names[1])),
    ],
  }
}

/// Perl MakeBibliography `do_names` (L568-584) / `do_authors` (L595-597): the
/// full author list for the author-year refnum. Each name is rendered via
/// `do_name` (initials + surname) and joined with ", " (>2 names) or " " (≤2),
/// with "and " before the last; a trailing BibTeX "others" (from "and others")
/// collapses to a trailing "et al." span instead.
fn do_names(mut names: Vec<Node>) -> Vec<NodeData> {
  let sep = if names.len() > 2 { ", " } else { " " };
  let mut etal = false;
  if names
    .last()
    .map(|n| n.get_content().trim() == "others")
    .unwrap_or(false)
  {
    names.pop();
    etal = true;
  }
  let last = names.len().saturating_sub(1);
  let mut out: Vec<NodeData> = Vec::new();
  for (i, name) in names.iter().enumerate() {
    if !out.is_empty() {
      out.push(NodeData::Text(sep.to_string()));
      if !etal && i == last {
        out.push(NodeData::Text("and ".to_string()));
      }
    }
    out.push(NodeData::Text(do_name_text(name)));
  }
  if etal {
    out.push(NodeData::Text(sep.to_string()));
    out.push(NodeData::Element {
      tag:        "ltx:text".to_string(),
      attributes: Some(HashMap::from_iter([(
        "class".to_string(),
        "ltx_bib_etal".to_string(),
      )])),
      children:   vec![NodeData::Text("et al.".to_string())],
    });
  }
  out
}

/// Perl MakeBibliography `do_editorsA` (L599-604): `do_names` plus a trailing
/// " (Eds.)" (more than one editor) or " (Ed.)" (a single editor).
fn do_editors_a(names: Vec<Node>) -> Vec<NodeData> {
  let n = names.len();
  let mut out = do_names(names);
  if n > 1 {
    out.push(NodeData::Text(" (Eds.)".to_string()));
  } else if n == 1 {
    out.push(NodeData::Text(" (Ed.)".to_string()));
  }
  out
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
/// Resolve a bibliography source file (`.bib`/`.bbl`/`.bib.xml`).
///
/// Delegates to the core `pathname::find`, the faithful translation of Perl's
/// `pathname_find` (`LaTeXML/Util/Pathname.pm`): a strict-case search over `.`
/// plus the search paths, falling back to a CASE-INSENSITIVE directory scan when
/// no strict match exists (Perl's `return @paths ? @paths : @nocase_paths`).
/// The bibliography path previously used a bespoke case-SENSITIVE lookup, so an
/// author on a case-insensitive filesystem citing `\bibliography{EvoFlock.bib}`
/// against an on-disk `Evoflock.bib` (arXiv:2606.25280) lost the ENTIRE
/// reference list on Linux — a parity gap, since Perl's `pathname_find` resolves
/// it. Delegating restores parity (Perl resolves it silently too; no warning,
/// matching how the engine already resolves e.g. `PASJ95.STY`).
fn find_file(name: &str, search_paths: &[String]) -> Option<String> {
  latexml_core::util::pathname::find(name, latexml_core::util::pathname::PathnameFindOptions {
    paths: Some(search_paths.to_vec()),
    ..Default::default()
  })
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
  fn test_find_file_case_insensitive_bib() {
    // An author on a case-insensitive filesystem cites `\bibliography{EvoFlock.bib}`
    // but the file on disk is `Evoflock.bib` (arXiv:2606.25280). `find_file`
    // delegates to the core pathname resolver, which mirrors Perl's
    // case-insensitive fallback, so the reference list is recovered on
    // case-sensitive Linux too — a parity fix (Perl's pathname_find resolves it).
    let dir = std::env::temp_dir().join("lxo_bib_case_test");
    let _ = std::fs::create_dir_all(&dir);
    let on_disk = dir.join("Evoflock.bib");
    std::fs::write(&on_disk, "@article{k, title={T}}\n").unwrap();
    let dirs = vec![dir.to_str().unwrap().to_string()];
    // Exact-case still resolves.
    assert!(find_file("Evoflock.bib", &dirs).is_some());
    // Case-mismatched request resolves via the pathname fallback.
    let hit = find_file("EvoFlock.bib", &dirs);
    assert!(
      hit.is_some(),
      "case-mismatched bib filename should resolve via the pathname fallback"
    );
    assert!(hit.unwrap().to_lowercase().ends_with("evoflock.bib"));
    // A genuinely-absent file still returns None (no phantom match).
    assert!(find_file("NoSuchBib.bib", &dirs).is_none());
    let _ = std::fs::remove_dir_all(&dir);
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
