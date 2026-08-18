//! Post-processing pipeline API.
//!
//! Provides a public interface to the LaTeXML post-processing pipeline
//! (Scan → Bibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups).
//! Used by both the `latexml_oxide` binary and the `cortex_worker` binary.

use latexml_core::{
  Info, Warn,
  common::error::{emit_error, note_progress},
  s,
  telemetry::{self, Phase},
};
use latexml_post::{
  document::{PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::Processor,
};
use once_cell::sync::Lazy;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static POST_AUDIT: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_POST_AUDIT").is_ok());

/// Start an audit timer when `LATEXML_POST_AUDIT` is set. Module-level (not a
/// closure) so [`render_spilled_page`] — shared by the serial driver and the
/// parallel render worker — can use the identical instrumentation.
fn audit_start(name: &str) -> Option<(String, std::time::Instant)> {
  if *POST_AUDIT {
    Some((name.to_string(), std::time::Instant::now()))
  } else {
    None
  }
}

/// Report an audit timer started by [`audit_start`] (no-op when audit is off).
fn audit_end(started: Option<(String, std::time::Instant)>) {
  if let Some((name, t0)) = started {
    let ms = t0.elapsed().as_millis();
    Info!("audit", "phase", s!("{} took {}ms", name, ms));
  }
}

/// Options for the post-processing pipeline.
pub struct PostOptions<'a> {
  pub pmml:                      bool,
  pub cmml:                      bool,
  pub keep_xmath:                bool,
  pub stylesheet:                Option<&'a str>,
  pub destination:               Option<&'a str>,
  /// Base directory of the original LaTeX source (`--sourcedirectory`; Perl
  /// Config.pm L147, passed to Post as `sourceDirectory` in LaTeXML.pm L429).
  /// Used to locate graphics/resources the post-phase copies. When `None`,
  /// the caller defaults it to the source file's own directory.
  pub source_directory:          Option<&'a str>,
  /// Root directory of the generated site (`--sitedirectory`; Perl Config.pm
  /// L146, passed to Post as `siteDirectory` in LaTeXML.pm L430). Establishes
  /// the base against which cross-document/resource URLs are made relative.
  /// When `None`, Post defaults it to the destination's directory (Perl
  /// Config.pm L466-469; `document.rs` mirrors that fallback).
  pub site_directory:            Option<&'a str>,
  /// Extra resource search paths (the `--path` flag): directories searched
  /// for `--css`/`--javascript` files (and other post resources) to copy into
  /// the destination, in addition to the document's own paths, the current
  /// directory, and the binary's embedded resource table.
  pub search_paths:              &'a [String],
  pub nodefaultresources:        bool,
  pub css_files:                 &'a [String],
  pub js_files:                  &'a [String],
  pub noinvisibletimes:          bool,
  /// Remap styled alphanumerics to Unicode Plane-1 (Perl `--plane1`, default on).
  pub plane1:                    bool,
  /// Remap only the poorly-supported variants, to their simpler form
  /// (Perl `--hackplane1`); implies `plane1`.
  pub hackplane1:                bool,
  pub mathtex:                   bool,
  /// Cross-reference URL style (Perl `--urlstyle`; `CrossRef::generateURL`).
  /// Defaults to [`UrlStyle::File`](latexml_post::crossref::UrlStyle::File) at
  /// every non-CLI construction site (the binary threads the parsed `--urlstyle`
  /// value here).
  pub url_style:                 latexml_post::crossref::UrlStyle,
  pub navigationtoc:             Option<&'a str>,
  pub schemadocs:                bool,
  pub split:                     bool,
  pub split_xpath:               Option<String>,
  pub split_naming:              Option<&'a str>,
  pub xslt_parameters:           &'a [String],
  /// If > 0, try the vector-SVG converters (mutool → pdftocairo) for PDF
  /// graphics smaller than this many KB (vector-preservation path). Fall
  /// back to the raster ImageMagick `convert`/`gs` → PNG path on failure
  /// or timeout. Tracks upstream brucemiller/LaTeXML#902.
  pub graphics_svg_threshold_kb: u32,
  /// Convert `\includegraphics` figures to web images (Perl `graphicimages!`
  /// → `dographics`, default on). When `false` (`--nographicimages`) the
  /// Graphics post-phase is skipped entirely: the output keeps the raw
  /// `<ltx:graphics>` references untouched — useful for faster runs or hosts
  /// without the image tools.
  pub graphicimages:             bool,
  /// Timestamp string embedded in the generated page (Perl `--timestamp`,
  /// XSLT `TIMESTAMP` param). `None` → no timestamp emitted (the deterministic
  /// default; Perl instead defaults to the current time). Perl's
  /// `--timestamp=0` "omit" case is normalized to `None` by the caller.
  pub timestamp:                 Option<&'a str>,
  /// Favicon resource for the generated site (Perl `--icon`, XSLT `ICON`
  /// param): emitted as a `<link rel="icon">` and copied to the destination.
  pub icon:                      Option<&'a str>,
  /// Output extraction mode (Perl `LaTeXML::Util::Pack::whatsout`).
  /// `Document` (default) → serialize the full post-processed
  /// document; `Fragment` → embeddable HTML snippet via
  /// `latexml_post::extract::get_embeddable`; `Math` → math subtree
  /// via `get_math`. Applied to each `PostDocument` in the final
  /// serialization loop.
  pub whatsout:                  latexml_post::extract::Whatsout,
}

/// The built-in XSLT stylesheet for an output format (logical embedded-resource
/// name; Perl `Config.pm` L543-551). Returns `None` for a format with no
/// default (e.g. `xml`), so a caller keeps `--stylesheet` / no post.
///
/// SINGLE SOURCE OF TRUTH shared by the CLI (`bin/latexml_oxide.rs` picks
/// `effective_stylesheet` when `--stylesheet` is unset) and the library
/// (`crate::api::convert_to_html`), so the two can never disagree on which
/// sheet an `html5`/`xhtml`/`epub` job gets.
pub fn default_stylesheet(format: Option<&str>) -> Option<&'static str> {
  match format {
    Some("html5") => Some("resources/XSLT/LaTeXML-html5.xsl"),
    Some("html") | Some("xhtml") => Some("resources/XSLT/LaTeXML-all-xhtml.xsl"),
    Some("epub") | Some("epub3") => Some("resources/XSLT/LaTeXML-epub3.xsl"),
    _ => None,
  }
}

/// Emit a post-processing error: capture it into the active log buffer (so it
/// reaches `cortex.log`) AND bump the Error counter so `Status:conversion`
/// reflects post-phase failures (Perl `LaTeXML.pm` L633: `max(core, post)`).
///
/// Deliberately NOT the `Error!` macro: that expands to a too-many-errors
/// `Fatal!(…)` → `return Err(error::Error)`, which only typechecks inside the
/// digester's `Result<_, error::Error>` functions. The post-processing entry
/// points return `String`/`Result<_, ()>`, so we log+count directly with no
/// control-flow side effect.
fn post_error(object: &str, message: &str) { emit_error("post", object, message); }

/// Result of [`run_post_processing_logged`]: the serialized HTML plus the
/// post-phase log text and status code, mirroring Perl `LaTeXML.pm`'s
/// `convert_post`, which flushes the log AFTER post and folds the post status
/// into the final `max(core, post)` verdict (L631-634).
pub struct PostOutcome {
  /// Serialized post-processed output (same value as [`run_post_processing`]).
  pub html:        String,
  /// Log text captured during the post phase only — Graphics/MathML/XSLT
  /// `Info!`/`Warn!`/`post_error` lines. Append to the core conversion log.
  pub log:         String,
  /// Status code read from the shared REPORT counter AFTER post. Because the
  /// counter is NOT reset between core and post, this is already the combined
  /// core+post code; callers still `max()` it with the core code as a
  /// belt-and-suspenders guard for paths that force a fatal outside REPORT
  /// (e.g. a `catch_unwind`-trapped panic).
  pub status_code: usize,
}

/// Run post-processing while capturing its log into a fresh thread-local
/// LOG_BUFFER. `Converter::convert()` already flushed the *core* log into its
/// own response, leaving the buffer unbound — so without re-binding here every
/// post-phase `Info!`/`Warn!`/`post_error` (incl. a silent EPS→PNG failure)
/// reaches stderr only and never the persisted `--log`/`cortex.log`. Perl's
/// single `flush_log()` after `convert_post` sweeps up the post log for ALL
/// consumers; this is the Rust equivalent, shared by the `latexml_oxide` and
/// `cortex_worker` binaries so their logs reach parity. See SYNC_STATUS task 5.
pub fn run_post_processing_logged(xml: &str, opts: &PostOptions) -> PostOutcome {
  run_post_processing_impl_logged(PostInput::Xml(xml), opts)
}

/// File-input variant of [`run_post_processing_logged`]: parses the XML from
/// disk via libxml2's streaming reader rather than from an in-memory `String`.
/// Used for already-converted `.xml` inputs, where slurping the file would keep
/// a whole extra copy of a multi-hundred-MB document resident alongside the DOM.
pub fn run_post_processing_from_file_logged(path: &str, opts: &PostOptions) -> PostOutcome {
  run_post_processing_impl_logged(PostInput::File(path), opts)
}

fn run_post_processing_impl_logged(input: PostInput, opts: &PostOptions) -> PostOutcome {
  latexml_core::util::logger::bind_log();
  let html = run_post_processing_impl(input, opts);
  let log = latexml_core::util::logger::flush_log();
  let status_code = latexml_core::common::error::get_status_code();
  PostOutcome { html, log, status_code }
}

/// Where the post-pipeline reads its already-converted LaTeXML XML from.
#[derive(Clone, Copy)]
enum PostInput<'a> {
  /// XML already resident in memory (the TeX→XML conversion result). The common
  /// path — the 2.8M-doc arXiv fleet and every TeX conversion feed this.
  Xml(&'a str),
  /// A path to an XML file, parsed via libxml2's streaming file reader
  /// (`xmlReadIO`). Avoids vivifying a very large document as a Rust `String`
  /// on top of the ~11× DOM it becomes.
  File(&'a str),
}

/// Run the post-processing pipeline on in-memory XML output.
///
/// Executes: Split → Scan → MakeBibliography → CrossRef → Graphics → MathML → XSLT → HTML5 fixups.
pub fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  run_post_processing_impl(PostInput::Xml(xml), opts)
}

/// File-input variant of [`run_post_processing`].
pub fn run_post_processing_from_file(path: &str, opts: &PostOptions) -> String {
  run_post_processing_impl(PostInput::File(path), opts)
}

/// The in-memory parse ceiling: libxml2's `xmlReadMemory` takes the buffer
/// length as a C `int`, so a handoff string of `i32::MAX` bytes or more
/// CANNOT be parsed from memory — it fails with "Document too large for
/// i32". First witness to cross it: the 131 MB book's 2.68 GB core XML,
/// single-invocation `.tex → .htm` (laptop UAT 2026-07-31; the streamed
/// assembly exists only as serialized text, so post must re-parse it).
/// Above the ceiling the handoff spills to a temp file beside the
/// destination and takes the `PostInput::File` arm — the streaming-reader
/// path that already parses this very document in the two-invocation flow.
///
/// Env-overridable for tests only (`LATEXML_POST_MEM_PARSE_LIMIT`): a real
/// 2-GiB-plus fixture is not something CI can allocate, so the guard test
/// drives the spill path with a tiny limit instead.
fn post_mem_parse_limit() -> usize {
  std::env::var("LATEXML_POST_MEM_PARSE_LIMIT")
    .ok()
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(i32::MAX as usize)
}

/// The size at which a *split* run prefers the streaming split front-end
/// (`latexml_post::stream_split`) over the whole-DOM parse. Default 1 GiB: a
/// core XML that large parses to a ~20+ GB DOM (measured ~16 GB for 614 MB),
/// where the streaming front-end peaks at one content subtree. Test override:
/// `LATEXML_POST_STREAM_THRESHOLD` (bytes).
fn stream_split_threshold() -> u64 {
  std::env::var("LATEXML_POST_STREAM_THRESHOLD")
    .ok()
    .and_then(|v| v.parse::<u64>().ok())
    .unwrap_or(1 << 30)
}

/// Gate for the streaming split: `LATEXML_POST_STREAM_SPLIT=1` forces it on
/// (any size), `=0` disables it, unset engages it automatically for files at
/// or above [`stream_split_threshold`] — where the whole-DOM parse is beyond
/// commodity-RAM hosts anyway (the 131 MB witness's 2.68 GB core XML OOM'd a
/// 31 GB laptop *during the parse*, 0 pages written).
fn stream_split_gate(path: &str) -> bool {
  match std::env::var("LATEXML_POST_STREAM_SPLIT").as_deref() {
    Ok("0") => false,
    Ok(_) => true,
    Err(_) => std::fs::metadata(path)
      .map(|m| m.len() >= stream_split_threshold())
      .unwrap_or(false),
  }
}

/// `--splitnaming` → `SplitNaming`, shared by the DOM and streaming split
/// front-ends so the two cannot disagree.
fn resolve_split_naming(split_naming: Option<&str>) -> latexml_post::split::SplitNaming {
  use latexml_post::split::SplitNaming;
  match split_naming {
    Some("id") | None => SplitNaming::Id,
    Some("idrelative") => SplitNaming::IdRelative,
    Some("label") => SplitNaming::Label,
    Some("labelrelative") => SplitNaming::LabelRelative,
    Some(other) => {
      Warn!(
        "post",
        "split",
        format!("Unknown splitnaming '{other}', using 'id'")
      );
      SplitNaming::Id
    },
  }
}

/// Destination directory for a page pathname (empty parent → ".", mirroring
/// `PostDocument`'s internal derivation).
fn destination_directory_of(destination: &str) -> Option<String> {
  std::path::Path::new(destination).parent().map(|p| {
    let s = p.to_string_lossy();
    if s.is_empty() {
      ".".to_string()
    } else {
      s.into_owned()
    }
  })
}

/// The two placeholder probes recorded per page WHILE it is parsed, so the
/// ObjectDB-baton sweeps (MakeIndex, MakeBibliography) re-read only the
/// handful of pages that actually carry work.
fn page_placeholder_probes(d: &PostDocument) -> (bool, bool) {
  (
    !d.findnodes("//ltx:index[not(ltx:indexlist)] | //ltx:glossary[not(ltx:glossarylist)]")
      .is_empty(),
    !d.findnodes("//ltx:bibliography").is_empty(),
  )
}

/// Everything the shared pipeline tail (MakeIndex → sweeps → page-major
/// render) needs from a front-end: spilled pages plus the scanned ObjectDB.
/// Produced by BOTH front-ends — the streaming split and the whole-DOM
/// parse+Split+Scan.
struct PostFront {
  spilled_pages:  Vec<SpilledPage>,
  db:             ObjectDB,
  svg_fragments:  Vec<(String, String)>,
  intent_literal: bool,
}

/// Attempt the streaming split front-end (`latexml_post::stream_split`).
///
/// `Ok(None)` = not applicable (the union matched no pages, or the stream
/// could not be split faithfully — already reported) → the caller falls back
/// to the whole-DOM pipeline. `Err(())` = a hard failure after pages were in
/// flight (I/O, resource ceiling), already reported via `post_error`; the
/// caller bails out.
fn try_streaming_front(
  path: &str,
  split_xpath: &str,
  split_naming: Option<&str>,
  destination: Option<&str>,
  page_opts: &PostDocumentOptions,
  spill_dir: &std::path::Path,
) -> Result<Option<PostFront>, ()> {
  let naming = resolve_split_naming(split_naming);
  telemetry::phase_enter(Phase::Split);
  let outcome =
    latexml_post::stream_split::stream_split(path, split_xpath, naming, destination, spill_dir);
  telemetry::phase_exit();
  let outcome = match outcome {
    Ok(Some(o)) => o,
    Ok(None) => return Ok(None),
    Err(e) => {
      // Fail toward flagging: the whole-DOM fallback may well OOM on an input
      // this large, but a loud failure beats a silently wrong result — and
      // beats the whole-DOM fallback, whose parse of a threshold-sized file
      // is a guaranteed OOM on commodity RAM (witnessed: a mid-stream error
      // fell back into a 2.68 GB whole-DOM parse that grew past a 26 GB
      // ceiling with zero pages to show). The unions this front-end cannot
      // evaluate never reach here — the caller gates on `supports_union`.
      post_error("split", &format!("streaming split failed: {e}"));
      return Err(());
    },
  };
  // The engagement marker the parity/threshold tests assert on.
  Info!(
    "post",
    "stream-split",
    s!(
      "streaming split engaged for '{}': {} pages",
      path,
      outcome.pages.len()
    )
  );
  note_progress(&format!("Split into {} documents", outcome.pages.len()));
  let intent_literal = outcome
    .latexml_pis
    .iter()
    .any(|pi| pi.contains("package=\"ar5iv"));
  let svg_fragments = extract_svg_fragments(&outcome.picture_xml);

  // Pre-order Scan sweep over the spilled pages — the streaming equivalent of
  // `run_phase(docs, Scan)` + the driver's spill loop, ONE page resident at a
  // time. The order is semantic, not cosmetic: SITE_ROOT is taken from the
  // first page scanned (the root page), an ancestor must be in the DB before
  // its descendants (Scan's parent inference falls back to SITE_ROOT
  // otherwise), and each parent's `children` list follows scan order — all
  // three feed CrossRef's navigation.
  telemetry::phase_enter(Phase::PostScan);
  let mut scanner = latexml_post::scan::Scan::new(ObjectDB::new());
  let mut spilled: Vec<SpilledPage> = Vec::with_capacity(outcome.pages.len());
  for page in &outcome.pages {
    if let Err(e) = latexml_core::stomach::check_timeout() {
      // FATAL, not Error: a scan stopped partway means the delivered site
      // would be silently truncated (user decision 2026-08-02 — a resource
      // stop that truncates the deliverable carries fatal severity).
      Info!(
        "post",
        "stopped",
        s!(
          "Scan stopped after {} of {} pages",
          spilled.len(),
          outcome.pages.len()
        )
      );
      e.log_fatal();
      telemetry::phase_exit();
      return Err(());
    }
    let path_str = page.path.to_string_lossy().into_owned();
    let page_doc = match PostDocument::new_from_file(&path_str, page_opts.clone()) {
      Ok(mut d) => {
        d.destination = Some(page.destination.clone());
        d.destination_directory = destination_directory_of(&page.destination);
        d
      },
      Err(e) => {
        post_error("post", &format!("cannot read a streamed page: {e}"));
        telemetry::phase_exit();
        return Err(());
      },
    };
    // Scan's only page mutation: a root without an id gets
    // `xml:id="Document"`. Detect it so the spill is refreshed (pass B
    // re-parses pages from disk and must see what Scan registered).
    let root_unnamed = page_doc
      .get_document_element()
      .and_then(|r| latexml_post::document::get_xml_id(&r))
      .is_none();
    let nodes = scanner.to_process(&page_doc);
    let processed = if nodes.is_empty() {
      vec![page_doc]
    } else {
      match scanner.process(page_doc, nodes) {
        Ok(p) => p,
        Err(e) => {
          post_error("Scan", &format!("Scan failed: {e}"));
          telemetry::phase_exit();
          return Err(());
        },
      }
    };
    for d in processed {
      if root_unnamed && let Err(e) = std::fs::write(&page.path, d.to_xml_string()) {
        post_error("post", &format!("cannot refresh a disk-staged page: {e}"));
        telemetry::phase_exit();
        return Err(());
      }
      let (needs_index, needs_bib) = page_placeholder_probes(&d);
      spilled.push(SpilledPage {
        needs_index,
        needs_bib,
        path: page.path.clone(),
        destination: Some(page.destination.clone()),
        destination_directory: destination_directory_of(&page.destination),
      });
      // The page's DOM drops here — one resident at a time.
    }
  }
  telemetry::phase_exit();
  Ok(Some(PostFront {
    spilled_pages: spilled,
    db: scanner.db,
    svg_fragments,
    intent_literal,
  }))
}

/// Write an oversized handoff to a temp file for the streaming file parser.
/// Beside the destination when there is one (writing to the destination
/// directory is always allowed), else the system temp dir.
fn spill_oversized_xml(xml: &str, opts: &PostOptions) -> std::io::Result<std::path::PathBuf> {
  let dir = opts
    .destination
    .and_then(|d| {
      std::path::Path::new(d)
        .parent()
        .map(std::path::Path::to_path_buf)
    })
    .filter(|d| !d.as_os_str().is_empty())
    .unwrap_or_else(std::env::temp_dir);
  let path = dir.join(format!("latexml-post-handoff-{}.xml", std::process::id()));
  std::fs::write(&path, xml)?;
  Ok(path)
}

fn run_post_processing_impl(input: PostInput, opts: &PostOptions) -> String {
  // Oversized in-memory handoff → spill + streaming file parse (see
  // `post_mem_parse_limit`). The temp file is removed on every exit path by
  // the guard; a spill failure falls through to the memory parse, whose own
  // error reporting then names the real problem.
  let mut _spill_cleanup: Option<std::path::PathBuf> = None;
  let spilled_path: Option<String>;
  // A handoff below the i32 ceiling but at/above the streaming-split
  // threshold also spills when splitting is on: only the `File` arm can take
  // the streaming split, and a memory parse of a multi-GB handoff is the OOM
  // this work removes.
  let spill_for_streaming = |len: usize| opts.split && len as u64 >= stream_split_threshold();
  let input = match input {
    PostInput::Xml(xml)
      if xml.len() >= post_mem_parse_limit() || spill_for_streaming(xml.len()) =>
    {
      match spill_oversized_xml(xml, opts) {
        Ok(path) => {
          // Operationally worth a line: a multi-GB temp file just appeared
          // beside the destination. Also the guard test's engagement proof —
          // without it a silently-failing spill would fall back to the memory
          // parse and the test would pass vacuously.
          latexml_core::Info!(
            "post",
            "spill",
            format!(
              "oversized handoff ({} bytes) staged to {}",
              xml.len(),
              path.display()
            )
          );
          spilled_path = Some(path.to_string_lossy().into_owned());
          _spill_cleanup = Some(path);
          PostInput::File(spilled_path.as_deref().expect("just set"))
        },
        Err(e) => {
          latexml_core::Info!(
            "post",
            "spill",
            format!("could not stage an oversized handoff to disk ({e}); parsing from memory")
          );
          PostInput::Xml(xml)
        },
      }
    },
    other => other,
  };
  let result = run_post_processing_inner(input, opts);
  if let Some(path) = _spill_cleanup {
    let _ = std::fs::remove_file(path);
  }
  result
}

fn run_post_processing_inner(input: PostInput, opts: &PostOptions) -> String {
  let PostOptions {
    pmml,
    cmml,
    keep_xmath,
    stylesheet,
    destination,
    source_directory,
    site_directory,
    search_paths,
    nodefaultresources,
    css_files,
    js_files,
    noinvisibletimes,
    plane1,
    hackplane1,
    mathtex,
    url_style,
    navigationtoc,
    schemadocs,
    split,
    ref split_xpath,
    split_naming,
    xslt_parameters,
    graphics_svg_threshold_kb,
    graphicimages,
    timestamp,
    icon,
    whatsout,
  } = *opts;

  let mut doc_opts = PostDocumentOptions::default();
  if let Some(dest) = destination {
    doc_opts.destination = Some(dest.to_string());
  }
  if let Some(src_dir) = source_directory {
    doc_opts.source_directory = Some(src_dir.to_string());
    let mut sp = doc_opts.searchpaths.take().unwrap_or_default();
    sp.push(src_dir.to_string());
    doc_opts.searchpaths = Some(sp);
  }
  // --sitedirectory (Perl LaTeXML.pm L430 `siteDirectory`): the site root for
  // relativizing cross-document/resource URLs. When unset, `PostDocument::new`
  // falls back to the destination's directory (document.rs, Perl Config.pm L466).
  if let Some(site_dir) = site_directory {
    doc_opts.site_directory = Some(site_dir.to_string());
  }
  // `raw_xml` holds the in-memory serialization — `Some` only for `Xml` input.
  // `File` input streams from disk and never holds it, so `raw_xml` is `None`
  // and the three raw-string features (empty-doc substitution, SVG extraction,
  // ar5iv sniff) plus the on-failure echo are re-derived from the parsed doc.
  //
  // Perl LaTeXML.pm L330-336: an empty core result (e.g. after a Fatal) is still
  // post-processed against a bare <document/> root ("important for utility
  // features such as packing .zip archives"). Mirror that for empty in-memory
  // input; a File is never empty (an empty file simply errors on parse below).
  let raw_xml: Option<&str> = match input {
    PostInput::Xml(xml) if xml.trim().is_empty() => Some("<document/>"),
    PostInput::Xml(xml) => Some(xml),
    PostInput::File(_) => None,
  };
  // Best-effort output when a phase bails: echo the original XML for in-memory
  // input; for File input the source is on disk (and a failure here is itself
  // the case we are hardening against), so return empty and rely on the logged
  // Error + status code.
  let fallback = || raw_xml.map(str::to_string).unwrap_or_default();

  // Pass B re-parses each spilled page and must see the SAME searchpaths and
  // site directory as this parse does.
  let page_opts = doc_opts.clone();

  // The per-page spill directory, created up front: BOTH front-ends (the
  // streaming split and the whole-DOM parse) write their pages into it.
  // Beside the destination when there is one — the same volume the output
  // lands on (and NOT the system temp dir, which can be a RAM-backed tmpfs:
  // a 40k-page document spills the whole core XML's worth of pages here).
  let mut spill_builder = tempfile::Builder::new();
  spill_builder.prefix(".latexml-post-pages-");
  let page_spill = match destination
    .and_then(|d| std::path::Path::new(d).parent())
    .filter(|p| !p.as_os_str().is_empty() && p.is_dir())
  {
    Some(dest_dir) => spill_builder.tempdir_in(dest_dir),
    None => spill_builder.tempdir(),
  };
  let page_spill = match page_spill {
    Ok(dir) => dir,
    Err(e) => {
      post_error(
        "post",
        &format!("cannot create the page staging directory: {e}"),
      );
      return fallback();
    },
  };

  // ---- Front-end selection --------------------------------------------------
  //
  // For a *file* input that will be split and is past the streaming threshold
  // (or force-enabled), the streaming split front-end produces the spilled
  // pages + scanned ObjectDB without ever parsing the whole document — the
  // whole-DOM parse of a 2.68 GB core XML needs ~30+ GB and OOM'd a 31 GB
  // host before Split ran (laptop UAT 2026-07-31). Everything downstream of
  // this block is shared between the two front-ends.
  let streamed: Option<PostFront> = if let PostInput::File(path) = input
    && split
    && destination.is_some()
    && let Some(xpath) = split_xpath.as_deref()
    && latexml_post::stream_split::supports_union(xpath)
    && stream_split_gate(path)
  {
    match try_streaming_front(
      path,
      xpath,
      split_naming,
      destination,
      &page_opts,
      page_spill.path(),
    ) {
      Ok(front) => front,
      Err(()) => return fallback(),
    }
  } else {
    None
  };

  let front = if let Some(front) = streamed {
    front
  } else {
    // ==== Whole-DOM front-end (the pre-existing pipeline, indentation kept) ====
    telemetry::phase_enter(Phase::PostXmlParse);
    let t_parse = audit_start("PostDocument parse");
    let parsed = match input {
      // `raw_xml` is `Some` for `Xml` input (the two arms above), so the default
      // is dead — kept only to avoid an `unwrap`.
      PostInput::Xml(_) => {
        PostDocument::new_from_string(raw_xml.unwrap_or("<document/>"), doc_opts)
      },
      PostInput::File(path) => PostDocument::new_from_file(path, doc_opts),
    };
    let doc = match parsed {
      Ok(d) => d,
      Err(e) => {
        post_error("parse", &format!("failed to parse XML: {e}"));
        telemetry::phase_exit();
        return fallback();
      },
    };
    audit_end(t_parse);
    telemetry::phase_exit();

    // SVG extraction reads the pre-post XML before any in-tree mutation; do it
    // once up-front so the regex-based fragment table is valid for every split
    // sub-document below. In-memory input scans the string; File input locates
    // pictures via a limit-safe `//ltx:picture` DOM query and serializes only
    // those (never the whole file) — most large documents have none.
    let t_svg = audit_start("SVG extraction");
    let svg_fragments = match raw_xml {
      Some(xml) => extract_svg_fragments(xml),
      None => extract_svg_fragments_from_doc(&doc),
    };
    audit_end(t_svg);

    // ar5iv sniff: the ar5iv package emits literal-intent MathML. In-memory input
    // checks the raw string; file input checks the parsed doc's `<?latexml
    // package="ar5iv"?>` PI. Computed here — before Split moves `doc` into `docs`.
    let intent_literal = match raw_xml {
      Some(xml) => xml.contains("package=\"ar5iv"),
      None => doc
        .processing_instructions()
        .iter()
        .any(|pi| pi.contains("package=\"ar5iv")),
    };

    // Perl-faithful pipeline order (latexmlpost L223-242):
    //   Split → Scan → MakeBibliography → CrossRef → Graphics → ...
    // Split runs FIRST so each downstream pass sees the per-page
    // destination. With the previous order (Scan before Split), every
    // entry's `location` was the root document and CrossRef built every
    // ref as a within-page anchor — the user-visible TOC links pointed
    // at `#Ch1` instead of `Ch1.html`.
    //
    // Phase 1: Split (only attributes time when --split is on)
    let mut docs: Vec<PostDocument> = if split {
      telemetry::phase_enter(Phase::Split);
      let result = if let Some(xpath) = split_xpath {
        let naming = resolve_split_naming(split_naming);
        let mut splitter = latexml_post::split::Split::new(xpath, naming, false);
        let split_nodes = splitter.to_process(&doc);
        match splitter.process(doc, split_nodes) {
          Ok(docs) => {
            if docs.len() > 1 {
              note_progress(&format!("Split into {} documents", docs.len()));
            }
            Ok(docs)
          },
          Err(e) => {
            post_error("split", &format!("Split failed: {e}"));
            Err(())
          },
        }
      } else {
        Ok(vec![doc])
      };
      telemetry::phase_exit();
      match result {
        Ok(ds) => ds,
        Err(()) => return fallback(),
      }
    } else {
      vec![doc]
    };

    // Helper: run one processor across every doc in a Vec, mirroring
    // Perl Post.pm:50-65's `foreach $proc { @newdocs = ... }` loop.
    // Each processor's per-doc `process` may return multiple docs (only
    // Split actually does, and we've already run Split above), so the
    // inner result is flattened back into `docs`.
    fn run_phase<P: Processor + ?Sized>(
      docs: Vec<PostDocument>,
      proc: &mut P,
      label: &'static str,
    ) -> Result<Vec<PostDocument>, ()> {
      let mut out: Vec<PostDocument> = Vec::with_capacity(docs.len());
      for d in docs {
        // Cooperative memory check per document, so a phase that grows across
        // many documents degrades gracefully instead of being hard-killed.
        //
        // MEASURED LIMIT, so nobody reads more into this than it delivers: on the
        // 614 MB witness at `--max-memory 6000` the ceiling is breached BEFORE
        // this loop is reached — the one-time parse + split DOM alone exceeds it,
        // so the run still ends at the hard watchdog (exit 137, zero pages). No
        // cooperative check inside the phases can catch that; bounding it is
        // task #147 (streaming the split parse). What this DOES cover is the
        // document whose growth happens across the phase itself.
        if let Err(e) = latexml_core::stomach::check_timeout() {
          // FATAL, not Error — see the render-loop stop below.
          Info!("post", "stopped", s!("{label} stopped mid-phase"));
          e.log_fatal();
          return Err(());
        }
        let nodes = proc.to_process(&d);
        if nodes.is_empty() {
          out.push(d);
          continue;
        }
        match proc.process(d, nodes) {
          Ok(processed) => out.extend(processed),
          Err(e) => {
            post_error(label, &format!("{label} failed: {e}"));
            return Err(());
          },
        }
      }
      Ok(out)
    }

    // Phase 2: Scan — runs on EACH sub-document so its entries register
    // the per-page `location` and `pageid`. Single shared ObjectDB so
    // the later CrossRef pass can resolve cross-doc refs.
    let mut scanner = latexml_post::scan::Scan::new(ObjectDB::new());
    telemetry::phase_enter(Phase::PostScan);
    let t_scan = audit_start("Scan");
    docs = match run_phase(docs, &mut scanner, "Scan") {
      Ok(d) => d,
      Err(()) => {
        telemetry::phase_exit();
        return fallback();
      },
    };
    audit_end(t_scan);
    telemetry::phase_exit();

    // ---- Pass A / Pass B boundary -------------------------------------------
    //
    // Scan is the ONLY phase that needs global knowledge: it visits every page
    // and produces strings in the ObjectDB. Everything after it is page-local.
    // The driver used to be phase-major (`for each phase { for each page }`),
    // which keeps every page alive at every boundary — and a live page is not
    // cheap: each is its own `xmlDoc` with its own dictionary, id table and
    // lazily-built caches, measured at ~1.6 MB of per-document overhead. Across
    // the 40,201 pages of a 614 MB core XML that alone grew RSS from 16 GB to
    // 80 GB *during Scan*, wrote zero pages, and was killed by the ceiling.
    //
    // So: spill each scanned page to disk (into the shared `page_spill` dir
    // created before front-end selection), free it, and render page-by-page
    // below. Peak becomes the one-time split DOM plus ONE page. Spilling (rather
    // than re-deriving pages from the source) keeps Scan's per-page
    // `location`/`pageid` registration byte-identical — those semantics were
    // themselves the fix for empty cross-document TOCs, so they are the wrong
    // thing to restructure here.
    //
    // (path, destination, destination_directory) — the metadata that does NOT
    // survive an XML round-trip and must be restored in pass B.
    // Plus a flag per page: does it carry an index/glossary/bibliography
    // placeholder? Recorded HERE, while the page is already parsed, so the two
    // baton sweeps below visit only the handful of pages that have work instead
    // of re-parsing every page twice (~80 k redundant parses on a 40 k-page
    // document).
    let mut spilled_pages = Vec::with_capacity(docs.len());
    for (i, d) in docs.drain(..).enumerate() {
      let path = page_spill.path().join(format!("page-{i:07}.xml"));
      if let Err(e) = std::fs::write(&path, d.to_xml_string()) {
        post_error("post", &format!("cannot stage page {i} to disk: {e}"));
        return fallback();
      }
      let (needs_index, needs_bib) = page_placeholder_probes(&d);
      spilled_pages.push(SpilledPage {
        needs_index,
        needs_bib,
        path,
        destination: d.get_destination().map(String::from),
        destination_directory: d.get_destination_directory().map(String::from),
      });
      // `d` drops here: one page's worth of DOM and caches released before the
      // next is touched.
    }
    PostFront {
      spilled_pages,
      db: scanner.db,
      svg_fragments,
      intent_literal,
    }
  }; // ==== end of the whole-DOM front-end ====
  let PostFront {
    spilled_pages,
    db,
    svg_fragments,
    intent_literal,
  } = front;

  // Phase 2b: MakeIndex (Perl LaTeXML.pm L466-470)
  //   Runs BEFORE MakeBibliography in Perl. Populates `<ltx:indexlist>`
  //   inside `<ltx:index>` placeholders and `<ltx:glossarylist>` inside
  //   `<ltx:glossary>` placeholders, using the `GLOSSARY:*` / `INDEX:*`
  //   entries Scan registered in the ObjectDB. Without this pass, glossary
  //   sections render empty in HTML (witness: tests/structure/glossary.tex).
  let mut indexer = latexml_post::make_index::MakeIndex::new(db, false, false);

  // Phase 3: MakeBibliography
  //
  // A raw `.bib` is converted by a recursive BibTeX session (Perl
  // `MakeBibliography.pm::convertBibliography`), which needs this crate's
  // model loader — so `latexml_post` declares the hook and we fill it in here,
  // on the thread that is about to run the pipeline.
  crate::bib_session::install();

  // The output file extension CrossRef strips for `--urlstyle` (Perl
  // `extension => $$opts{extension}`, LaTeXML.pm L479; derived in Config.pm
  // L408-420 from the destination's extension, else the format). Without it the
  // `server`/`negotiated` styles would strip the wrong suffix (the `xml` default)
  // and leave an html site's links untouched.
  let out_extension = destination
    .and_then(|d| std::path::Path::new(d).extension().and_then(|e| e.to_str()))
    .map(str::to_string)
    .unwrap_or_else(|| {
      if stylesheet.is_some_and(|s| s.contains("html")) {
        "html".to_string()
      } else {
        "xml".to_string()
      }
    });

  // Phase 4: CrossRef — built only once the bibliography sweep has handed the
  // ObjectDB baton on (see `sweep_pages`), so nothing holds two owners of it.
  let build_crossref = |db| {
    let mut crossref = latexml_post::crossref::CrossRef::new(db, url_style, true);
    crossref.set_extension(&out_extension);
    if let Some(navtoc) = navigationtoc {
      crossref.set_navigation_toc(navtoc);
    }
    crossref
  };

  // Phase 5: Graphics (Perl `graphicimages!` → `dographics`, default on).
  // `--nographicimages` skips the phase wholesale, leaving the raw
  // `<ltx:graphics>` references in the output untouched.
  let graphics_proc = graphicimages.then(|| {
    latexml_post::graphics::Graphics::new(None, true)
      .with_svg_threshold_kb(graphics_svg_threshold_kb)
  });

  // Phase 3: MathML + XSLT
  let post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  // `intent_literal` was computed up-front (before Split consumed `doc`).

  // Parallel P+C markup: when both formats are requested, the Content-MathML
  // processor is a *secondary* of the Presentation primary, folded into one
  // `<m:semantics>`/`<m:annotation-xml>` by `combine_parallel` — NOT a second
  // independent pass (which left the content tree as an orphan `<apply>` sibling
  // of `<m:semantics>`, rendered as stray text by browsers). Mirrors Perl
  // `MathProcessor`'s primary→secondary parallel model.
  if pmml {
    let mut presentation = latexml_post::mathml::MathML::new_presentation()
      .with_keep_xmath(keep_xmath)
      .with_invisible_times(!noinvisibletimes)
      .with_plane1(plane1, hackplane1)
      .with_mathtex(mathtex)
      .with_intent_literal(intent_literal);
    if cmml {
      presentation = presentation.with_secondaries(vec![Box::new(
        latexml_post::mathml::MathML::new_content()
          .with_keep_xmath(keep_xmath)
          .with_invisible_times(!noinvisibletimes)
          .with_plane1(plane1, hackplane1)
          .secondary(),
      )]);
    }
    processors.push(Box::new(presentation));
  } else if cmml {
    // Content-only output (no presentation primary).
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_content()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes)
        .with_plane1(plane1, hackplane1),
    ));
  }
  // Clones of the fully-resolved XSLT inputs for the parallel-render worker
  // manifest — `XSLT::new` consumes the real map/vec below, and by the time
  // the pass-B driver decides between serial and parallel they are gone.
  let mut manifest_xslt_params: Vec<(String, String)> = Vec::new();
  let mut manifest_searchpaths: Vec<String> = Vec::new();
  if let Some(xsl_path) = stylesheet {
    let mut searchpaths = vec![".".to_string()];
    if let Ok(exe) = std::env::current_exe()
      && let Some(project_root) = exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
    {
      searchpaths.insert(0, project_root.display().to_string());
    }
    // Prepend the user's `--path` directories so `--css`/`--javascript`
    // resources are found there (and take priority over `.`/project root)
    // when copy_param_resources searches for them.
    for p in search_paths.iter().rev() {
      searchpaths.insert(0, p.clone());
    }
    let mut xslt_params = rustc_hash::FxHashMap::default();

    // When `--schemadocs` is on, auto-prepend the rustdoc-styled
    // theme assets so callers don't need to repeat them on every
    // invocation. Idempotent: skipped if the user has already
    // listed the same basename via `--css` / `--javascript`.
    let prepend = |list: &[String], extra: &str| -> Vec<String> {
      if list
        .iter()
        .any(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()) == Some(extra))
      {
        list.to_vec()
      } else {
        let mut out = Vec::with_capacity(list.len() + 1);
        out.push(extra.to_string());
        out.extend_from_slice(list);
        out
      }
    };
    let css_effective: Vec<String> = if schemadocs {
      prepend(css_files, latexml_post::schema_docs::THEME_CSS_BASENAME)
    } else {
      css_files.to_vec()
    };
    let js_effective: Vec<String> = if schemadocs {
      prepend(js_files, latexml_post::schema_docs::THEME_JS_BASENAME)
    } else {
      js_files.to_vec()
    };
    if !css_effective.is_empty() {
      xslt_params.insert(
        "CSS".to_string(),
        format!("\"{}\"", css_effective.join("|")),
      );
    }
    if !js_effective.is_empty() {
      xslt_params.insert(
        "JAVASCRIPT".to_string(),
        format!("\"{}\"", js_effective.join("|")),
      );
    }
    if let Some(navtoc) = navigationtoc {
      xslt_params.insert("NAVIGATIONTOC".to_string(), format!("\"{}\"", navtoc));
    }
    if let Some(ts) = timestamp {
      xslt_params.insert("TIMESTAMP".to_string(), format!("\"{}\"", ts));
    }
    if let Some(icon) = icon {
      xslt_params.insert("ICON".to_string(), format!("\"{}\"", icon));
    }
    // Perl `LaTeXML.pm:562` unconditionally seeds `LATEXML_VERSION => '$LaTeXML::VERSION'`,
    // which drives `LaTeXML-common.xsl`'s `LaTeXML_identifier` generator stamp
    // (`<!--Generated by LaTeXML oxide (version X) ...-->`) and BookML's `utils.xsl`
    // `b:version-leq($LATEXML_VERSION,…)`. We expose OUR own crate `X.Y.Z` (#320). Inserted
    // BEFORE the user-override loop so `--xsltparameter LATEXML_VERSION=…` still wins (this
    // is how Perl's test suite pins `version TEST` for version-independent goldens).
    xslt_params.insert(
      "LATEXML_VERSION".to_string(),
      format!("\"{}\"", crate::core_interface::LATEXML_VERSION),
    );
    for param in xslt_parameters {
      if let Some((key, value)) = param.split_once('=') {
        xslt_params.insert(key.to_string(), format!("\"{}\"", value));
      }
    }
    manifest_xslt_params = xslt_params
      .iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect();
    manifest_searchpaths = searchpaths.clone();
    match latexml_post::xslt::XSLT::new(
      xsl_path,
      xslt_params,
      nodefaultresources,
      None,
      searchpaths,
    ) {
      Ok(xslt) => processors.push(Box::new(xslt)),
      Err(e) => post_error("xslt", &format!("XSLT error: {e}")),
    }
  }

  // process_chain attributes per-processor inside latexml_post::Post::
  // process_chain (MathmlPres / MathmlCont / Xslt). No outer phase wrap
  // needed; the inner per-processor guards cover their own time.
  //
  // Perl-faithful: pass ALL split docs to ProcessChain at once. Perl's
  // `Post::ProcessChain_internal` (Post.pm L41-67) seeds `@docs = ($doc)`
  // and lets each processor return a (possibly multi-element) list which
  // becomes the input to the next processor. We've already produced the
  // post-Split list above; ProcessChain just needs to fan MathML/XSLT
  // across it.
  let is_html_out = stylesheet.is_some_and(|s| s.contains("html"));

  // ---- Pass B: render one page at a time, then FREE it --------------------
  //
  // Document-major, not phase-major: each page goes through MakeIndex ->
  // MakeBibliography -> CrossRef -> Graphics -> MathML -> XSLT -> write, and
  // is dropped before the next is parsed. Every one of these phases is
  // page-local; only Scan (pass A, above) needed the whole document, and it
  // left its results in the ObjectDB as strings. Peak is therefore ONE page
  // plus the index, whatever the page count.
  //
  // Failure semantics change deliberately: pages that finish are already on
  // disk, so a later failure leaves a partial site rather than nothing. That
  // matches how a resource Fatal already keeps partial core output.
  // (The per-page pipeline itself lives in `render_spilled_page`, shared with
  // the process-parallel worker in `crate::render_workers`.)

  // The ObjectDB is a BATON: MakeIndex, MakeBibliography and CrossRef each take
  // it by value in turn, so they cannot be alive simultaneously. Each therefore
  // gets its own sweep over the spilled pages — still ONE page resident at a
  // time. A page is only re-spilled when its phase actually had work
  // (`to_process` non-empty), which for index/bibliography placeholders is a
  // handful of pages out of tens of thousands, so the sweeps cost a parse each
  // and almost no writes.
  fn sweep_pages(
    pages: &[SpilledPage],
    opts: &PostDocumentOptions,
    proc: &mut dyn Processor,
    label: &'static str,
    selector: fn(&SpilledPage) -> bool,
  ) -> Result<(), ()> {
    for page_meta in pages.iter().filter(|p| selector(p)) {
      let (path, dest, dest_dir) = (
        &page_meta.path,
        &page_meta.destination,
        &page_meta.destination_directory,
      );
      let path_str = path.to_string_lossy().into_owned();
      let mut page = PostDocument::new_from_file(&path_str, opts.clone()).map_err(|e| {
        post_error("post", &format!("cannot re-read a disk-staged page: {e}"));
      })?;
      page.destination = dest.clone();
      page.destination_directory = dest_dir.clone();
      let nodes = proc.to_process(&page);
      if nodes.is_empty() {
        continue; // untouched: the spill on disk is still current
      }
      let processed = proc.process(page, nodes).map_err(|e| {
        post_error(label, &format!("{label} failed: {e}"));
      })?;
      // A phase may split one page into several (an index page beside its
      // host); the extras are written by their own destination in pass B, so
      // re-spill each under the original path only for the first and append
      // the rest as additional spills is NOT needed here: MakeIndex and
      // MakeBibliography fill placeholders in place and return the same page.
      for d in processed.iter().take(1) {
        if let Err(e) = std::fs::write(path, d.to_xml_string()) {
          post_error("post", &format!("cannot re-stage a page to disk: {e}"));
          return Err(());
        }
      }
    }
    Ok(())
  }

  if sweep_pages(&spilled_pages, &page_opts, &mut indexer, "MakeIndex", |p| {
    p.needs_index
  })
  .is_err()
  {
    return fallback();
  }
  let mut bibmaker = latexml_post::make_bibliography::MakeBibliography::new(indexer.db, false);
  telemetry::phase_enter(Phase::Bibliography);
  let t_bib = audit_start("MakeBibliography");
  let bib_result = sweep_pages(
    &spilled_pages,
    &page_opts,
    &mut bibmaker,
    "MakeBibliography",
    |p| p.needs_bib,
  );
  audit_end(t_bib);
  telemetry::phase_exit();
  if bib_result.is_err() {
    return fallback();
  }
  // ---- Parallel pass B: process-level page-range workers (design §6) ------
  //
  // Env-gated (`LATEXML_RENDER_JOBS`, default 1 = the serial path below,
  // unchanged), and only past a minimum page count — below it the db save +
  // child spawn overhead beats the win. The ObjectDB is COMPLETE here (Scan,
  // MakeIndex and MakeBibliography have all run), so it is saved once as a
  // SQLite file and each worker attaches it readonly; the workers run the
  // identical `render_spilled_page` pipeline over contiguous page ranges and
  // the parent folds their diagnostics deterministically, in chunk order.
  let render_jobs = crate::render_workers::render_jobs();
  if render_jobs > 1 && spilled_pages.len() >= crate::render_workers::MIN_PAGES_FOR_PARALLEL {
    let page_jobs: Vec<crate::render_workers::PageJob> = spilled_pages
      .iter()
      .map(|p| crate::render_workers::PageJob {
        path:                  p.path.clone(),
        destination:           p.destination.clone(),
        destination_directory: p.destination_directory.clone(),
      })
      .collect();
    let manifest = crate::render_workers::RenderManifest {
      // Filled in by `parallel_render` once the db is saved.
      dbfile: std::path::PathBuf::new(),
      navigation_toc: navigationtoc.map(String::from),
      url_style: url_style.as_cli().to_string(),
      out_extension: out_extension.clone(),
      graphicimages,
      graphics_svg_threshold_kb,
      pmml,
      cmml,
      keep_xmath,
      invisible_times: !noinvisibletimes,
      plane1,
      hackplane1,
      mathtex,
      intent_literal,
      stylesheet: stylesheet.map(String::from),
      xslt_params: manifest_xslt_params,
      nodefaultresources,
      searchpaths: manifest_searchpaths,
      is_html_out,
      svg_fragments: svg_fragments.clone(),
      schemadocs,
      whatsout: whatsout.as_cli().to_string(),
      page_opts: (&page_opts).into(),
      pages: Vec::new(),
    };
    let t_pages = audit_start("render_pages");
    let outcome = crate::render_workers::parallel_render(
      manifest,
      page_jobs,
      render_jobs,
      page_spill.path(),
      &bibmaker.db,
    );
    audit_end(t_pages);
    if let Some(result) = outcome {
      Info!(
        "post",
        "parallel-render",
        s!(
          "parallel render finished: {} of {} page(s) written",
          result.pages_rendered,
          spilled_pages.len()
        )
      );
      drop(page_spill);
      return result.main_output.unwrap_or_else(fallback);
    }
    // Setup failed BEFORE any worker spawned (already reported): fall through
    // to the serial render — the spilled pages are untouched on disk.
  }

  let crossref = build_crossref(bibmaker.db);
  let ctx = PageRenderCtx {
    page_opts: page_opts.clone(),
    is_html_out,
    svg_fragments,
    schemadocs,
    whatsout,
  };
  let mut procs = PageProcessors {
    crossref,
    graphics: graphics_proc,
    post,
    processors,
  };

  let mut main_output: Option<String> = None;
  let t_pages = audit_start("render_pages");
  let mut pages_rendered = 0usize;
  for SpilledPage {
    path,
    destination: dest,
    destination_directory: dest_dir,
    ..
  } in spilled_pages
  {
    // The cooperative memory guard, at the one seam post has: a page boundary.
    //
    // Nothing in latexml_post or this driver consulted it — `check_timeout`
    // appeared ZERO times across both — so an oversized post run was killed by
    // the hard watchdog at 100% of the ceiling (exit 137, `Fatal:oom:rss …
    // exceeded the … ceiling`) with nothing on disk, where the design promise
    // is a named Fatal at 75% plus whatever finished. Measured on the 131 MB
    // witness at `--max-memory 48000`: post died at 48,002 MiB having written
    // ZERO pages. Same shape as the Build phase before it got a guard.
    //
    // Stopping HERE is cheap and honest: pages already rendered are already
    // written, so the run ends with a real, if partial, site.
    if let Err(e) = latexml_core::stomach::check_timeout() {
      // FATAL, not Error (user decision 2026-08-02): a render stopped at
      // page N of M is a TRUNCATED deliverable — the same severity class as
      // the core's cheap-partial handoff. The pages already written stay on
      // disk (a real, if partial, site), the verdict says failed, and the
      // combined status the framework reads is 3. Witness that motivated
      // it: the joint 131 MB run stopped at 56,088 of 115,519 pages with
      // Error-class severity and process exit 0.
      Info!(
        "post",
        "stopped",
        s!("render stopped after {pages_rendered} page(s); partial site preserved")
      );
      e.log_fatal();
      break;
    }
    // Hand freed page memory back to the OS, exactly as core pass 1 does
    // (core_interface.rs, same rationale): every page cycles a full DOM +
    // XSLT result through GLIBC's heap (libxml2/libxslt allocate via libc,
    // not the Rust allocator), and glibc keeps freed mid-heap pages mapped —
    // the render loop's measured ~152 KB/page RSS growth on the 131 MB
    // witness is dominated by exactly this class, and it is what pushed the
    // single-invocation run over the memory fuse at page 56,088 of 115,519.
    // Rate-limited: a trim walks the heap.
    if pages_rendered > 0 && pages_rendered.is_multiple_of(512) {
      #[cfg(target_os = "linux")]
      unsafe {
        libc::malloc_trim(0);
      }
      #[cfg(not(feature = "dhat-heap"))]
      unsafe {
        libmimalloc_sys::mi_collect(true);
      }
    }
    // Retention triage probe (`LATEXML_POST_MEMDIAG=1`): splits the render
    // loop's RSS growth into live-C (libxml2/libxslt allocations still
    // reachable), free-but-mapped C, and "everything else" (Rust/mimalloc +
    // mmap). One run with this told the ~144 KB/page term apart from the
    // three candidates the first fixes addressed.
    #[cfg(target_os = "linux")]
    if pages_rendered > 0
      && pages_rendered.is_multiple_of(1024)
      && std::env::var("LATEXML_POST_MEMDIAG").is_ok()
    {
      let mi = unsafe { libc::mallinfo2() };
      let rss_mb = latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024;
      eprintln!(
        "memdiag: pages={} RSS={}MB C-live={}MB C-free={}MB",
        pages_rendered,
        rss_mb,
        mi.uordblks / (1024 * 1024),
        mi.fordblks / (1024 * 1024)
      );
    }
    let outputs = match render_spilled_page(&path, &mut procs, &ctx, dest, dest_dir) {
      Ok(o) => o,
      Err(()) => return fallback(),
    };
    for (dest, output) in outputs {
      if let Some(path) = dest.as_deref() {
        if let Some(parent) = std::path::Path::new(path).parent()
          && !parent.as_os_str().is_empty()
        {
          let _ = std::fs::create_dir_all(parent);
        }
        pages_rendered += 1;
        if let Err(e) = std::fs::write(path, &output) {
          post_error("write", &format!("failed to write page {path}: {e}"));
        }
      }
      if main_output.is_none() {
        main_output = Some(output);
      }
    }
  }
  audit_end(t_pages);
  drop(page_spill);

  main_output.unwrap_or_else(fallback)
}

/// The per-page processor set for pass B — everything [`render_spilled_page`]
/// mutates. Bundled so the serial driver and the process-parallel worker
/// ([`crate::render_workers`]) share ONE pipeline implementation and cannot
/// drift.
pub(crate) struct PageProcessors {
  pub(crate) crossref:   latexml_post::crossref::CrossRef,
  pub(crate) graphics:   Option<latexml_post::graphics::Graphics>,
  pub(crate) post:       latexml_post::Post,
  pub(crate) processors: Vec<Box<dyn Processor>>,
}

/// The read-only per-run context for pass B page rendering, shared by every
/// page of one conversion (serial driver or one parallel worker).
pub(crate) struct PageRenderCtx {
  /// Clone-source for each page's `PostDocument` parse — carries the same
  /// searchpaths/site-directory the pass-A parse used.
  pub(crate) page_opts:     PostDocumentOptions,
  pub(crate) is_html_out:   bool,
  pub(crate) svg_fragments: Vec<(String, String)>,
  pub(crate) schemadocs:    bool,
  pub(crate) whatsout:      latexml_post::extract::Whatsout,
}

/// Run one page-local phase (CrossRef / Graphics) over one page, mirroring
/// `run_phase` for a single document. Failures are reported via [`post_error`].
fn run_page_phase(
  doc: PostDocument,
  proc: &mut dyn Processor,
  label: &'static str,
) -> Result<Vec<PostDocument>, ()> {
  let nodes = proc.to_process(&doc);
  if nodes.is_empty() {
    return Ok(vec![doc]);
  }
  proc.process(doc, nodes).map_err(|e| {
    post_error(label, &format!("{label} failed: {e}"));
  })
}

/// The pass-B per-page pipeline: parse one spilled page (deleting the spill
/// file as soon as it is parsed), run CrossRef → Graphics → MathML/XSLT, and
/// return the finalized `(destination, output)` pairs for the caller to write.
/// Exactly the body the serial render loop ran in place before the
/// process-parallel split — phase order, telemetry and audit calls included —
/// so the serial path stays byte-identical (guard:
/// `118_streaming_split_parity`). `Err(())` means the failure was already
/// reported via [`post_error`] (or a phase's own diagnostics).
pub(crate) fn render_spilled_page(
  path: &std::path::Path,
  procs: &mut PageProcessors,
  ctx: &PageRenderCtx,
  destination: Option<String>,
  destination_directory: Option<String>,
) -> Result<Vec<(Option<String>, String)>, ()> {
  let path_str = path.to_string_lossy().into_owned();
  let page = match PostDocument::new_from_file(&path_str, ctx.page_opts.clone()) {
    Ok(mut d) => {
      d.destination = destination;
      d.destination_directory = destination_directory;
      d
    },
    Err(e) => {
      post_error("post", &format!("cannot re-read a disk-staged page: {e}"));
      return Err(());
    },
  };
  // Free the spill as soon as it is parsed: for a 40k-page document the
  // spill area is the size of the core XML, and it need not outlive its page.
  let _ = std::fs::remove_file(path);

  // CrossRef resolves this page's references out of the completed ObjectDB,
  // and its navigation/TOC work reads the DB rather than sibling pages — so
  // it is page-local once pass A has finished.
  telemetry::phase_enter(Phase::Crossref);
  let t_xref = audit_start("CrossRef");
  let xref = run_page_phase(page, &mut procs.crossref, "CrossRef");
  audit_end(t_xref);
  telemetry::phase_exit();
  let mut staged = xref?;
  if let Some(gfx) = procs.graphics.as_mut() {
    telemetry::phase_enter(Phase::Graphics);
    let t_gfx = audit_start("Graphics");
    let mut next = Vec::with_capacity(staged.len());
    let mut failed = false;
    for d in staged.drain(..) {
      match run_page_phase(d, gfx, "Graphics") {
        Ok(out) => next.extend(out),
        Err(()) => {
          failed = true;
          break;
        },
      }
    }
    audit_end(t_gfx);
    telemetry::phase_exit();
    if failed {
      return Err(());
    }
    staged = next;
  }

  let rendered = match procs.post.process_chain(staged, &mut procs.processors) {
    Ok(r) => r,
    Err(e) => {
      post_error("convert", &format!("Post-processing failed: {e}"));
      return Err(());
    },
  };

  let mut outputs = Vec::with_capacity(rendered.len());
  for doc in rendered {
    let dest = doc.get_destination().map(String::from);
    let output = latexml_post::extract::serialize_whatsout(&doc, ctx.whatsout);
    let output = if ctx.is_html_out {
      finalize_html5(output, &ctx.svg_fragments)
    } else {
      output
    };
    let output = if ctx.schemadocs && ctx.is_html_out {
      latexml_post::schema_docs::process_page(&output)
    } else {
      output
    };
    outputs.push((dest, output));
    // `doc` drops here — the whole point.
  }
  Ok(outputs)
}

/// One page spilled between the post pipeline's two passes: where its XML
/// lives, the metadata that does NOT survive an XML round-trip, and whether it
/// carries the placeholders the two ObjectDB-baton sweeps look for (recorded in
/// pass A, while the page is already parsed).
struct SpilledPage {
  path:                  std::path::PathBuf,
  destination:           Option<String>,
  destination_directory: Option<String>,
  needs_index:           bool,
  needs_bib:             bool,
}

/// Re-derive SVG fragments from a parsed document (the file-input equivalent of
/// [`extract_svg_fragments`]).
///
/// `//ltx:picture` is a *typed* descendant query, which libxml2 evaluates
/// without materializing `descendant-or-self::node()` — so it stays under the
/// 10M node-set ceiling even on a 600 MB document. Most large documents have no
/// pictures at all (the fast, zero-serialization path); when they do, only the
/// picture subtrees are serialized and handed to the shared string extractor,
/// never the whole file.
fn extract_svg_fragments_from_doc(doc: &PostDocument) -> Vec<(String, String)> {
  let pictures = doc.findnodes("//ltx:picture");
  if pictures.is_empty() {
    return Vec::new();
  }
  let joined: String = pictures.iter().map(|p| doc.node_to_string(p)).collect();
  extract_svg_fragments(&joined)
}

/// Apply HTML5 cleanup (XML prolog strip, void-element fixes) and inject SVG
/// fragments into empty `ltx_picture` spans. Pulled out of `run_post_processing`
/// so it can run on every split sub-document, not just the first.
fn finalize_html5(output: String, svg_fragments: &[(String, String)]) -> String {
  use std::sync::LazyLock;
  // Cached at first call — regex compile is the slow part of `Regex::new`,
  // and finalize_html5 runs on every (sub-)document in the post-pipeline.
  static XML_PROLOG_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^<\?xml[^?]*\?>\s*").unwrap());
  static NON_VOID_SELF_CLOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
      r"<(span|div|p|a|td|th|tr|section|article|figure|figcaption|pre|code|em|strong|b|i|u|sub|sup|small|cite)(\s[^>]*)?/>",
    ).unwrap()
  });
  static VOID_CLOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"</(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)>")
      .unwrap()
  });
  static VOID_SELF_CLOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
      r"<(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)(\s[^>]*?)\s*/>",
    )
    .unwrap()
  });

  let _gp_html5 = telemetry::phase(Phase::Html5Fixups);
  // Strip <?xml version...?> prolog: HTML5 must NOT have an XML declaration.
  // libxml2's to_string() includes it by default; we strip it here.
  let output = XML_PROLOG_RE.replace(&output, "").to_string();
  let output = NON_VOID_SELF_CLOSE_RE
    .replace_all(&output, "<$1$2></$1>")
    .to_string();
  let output = VOID_CLOSE_RE.replace_all(&output, "").to_string();
  let mut output = VOID_SELF_CLOSE_RE
    .replace_all(&output, "<$1$2>")
    .to_string();
  // SVG fragment injection into the empty `ltx_picture` placeholder spans.
  //
  // WHY this is a string splice and not a DOM operation (issue #398).
  // The SVG must land AFTER the XSLT stage (it is collected separately by
  // `extract_svg_fragments` / the streaming split), and the obvious DOM
  // alternative — re-parse the SVG fragment and append it under the placeholder
  // node before serialization — was deliberately abandoned: it tripped a
  // libxml2 use-after-free in `PostDocument` cleanup (see the note on
  // `extract_svg_fragments`). So we splice into the serialized string instead.
  //
  // The original splice hard-coded the placeholder as `<span id="…"
  // class="ltx_picture"…>` — coupled to id-before-class order AND double quotes,
  // neither guaranteed by a serializer. Measured (issue #398): on every LIVE
  // input the coupling holds — the XSLT emits `id` (add_id) then `class`
  // (add_classes is the first thing add_attributes does), and libxml2 preserves
  // insertion order and always double-quotes — so no real break was
  // demonstrable; the fragility is latent. We keep the string splice (the DOM
  // refactor is not worth re-entering the use-after-free for a latent concern)
  // but make the MATCH robust so a future serializer/XSLT change cannot silently
  // break it: match any empty `<span>` whose class contains `ltx_picture`,
  // regardless of attribute order or quote style, and look the fragment up by
  // the id found inside.
  //
  // WHEN TO REVISIT (do the real DOM refactor and delete this splice):
  //   * once the fork's `PostDocument` cleanup no longer use-after-frees on an
  //     inserted subtree (rust-libxml; we have fixed adjacent UAF/NULL-deref
  //     bugs there this year) — then inject the SVG as child nodes of the
  //     placeholder in the post-XSLT DOM (`doc`, before `serialize_whatsout`),
  //     which also lets the void-element normalization above move to a proper
  //     HTML serializer;
  //   * or if a demonstrated attribute-order/quote break ever appears (none
  //     today) — that would raise the priority from latent to real.
  if !svg_fragments.is_empty() {
    static EMPTY_PICTURE_SPAN: LazyLock<regex::Regex> = LazyLock::new(|| {
      // An empty `<span …></span>` carrying `ltx_picture` as a whole class
      // token, in a single- or double-quoted class value, with the other
      // attributes (notably `id`) in ANY position.
      regex::Regex::new(
        r#"<span\b(?P<attrs>[^>]*\bclass\s*=\s*(?:"[^"]*\bltx_picture\b[^"]*"|'[^']*\bltx_picture\b[^']*')[^>]*)></span>"#,
      )
      .unwrap()
    });
    static SPAN_ID: LazyLock<regex::Regex> =
      LazyLock::new(|| regex::Regex::new(r#"\bid\s*=\s*(?:"([^"]+)"|'([^']+)')"#).unwrap());
    output = EMPTY_PICTURE_SPAN
      .replace_all(&output, |caps: &regex::Captures| {
        let attrs = &caps["attrs"];
        let id = SPAN_ID
          .captures(attrs)
          .and_then(|c| c.get(1).or_else(|| c.get(2)))
          .map(|m| m.as_str());
        match id.and_then(|id| svg_fragments.iter().find(|(fid, _)| fid == id)) {
          // Preserve the placeholder's own attributes verbatim; only fill it.
          Some((_, svg_html)) => format!("<span{attrs}>{svg_html}</span>"),
          // A picture span we have no fragment for — leave it untouched.
          None => caps[0].to_string(),
        }
      })
      .to_string();
  }
  output
}

/// Extract SVG fragments from intermediate LaTeXML XML.
///
/// Finds `<picture>` elements, converts their children to inline SVG HTML.
/// Uses a lightweight regex+string approach (no libxml2) to avoid the
/// use-after-free crash in PostDocument cleanup.
///
/// Returns (picture_id, svg_html) pairs for post-XSLT injection.
fn extract_svg_fragments(xml: &str) -> Vec<(String, String)> {
  use std::sync::LazyLock;
  // Fast-fail: most documents have no `<picture>` elements (tikz / pgf
  // is uncommon in the canvas). Skip the backtracking lazy-match
  // regex (`(?s)...(.*?)`) when `<picture` doesn't appear as a literal
  // substring. `str::contains` is a SIMD-accelerated byte search and
  // takes microseconds even on ~MB inputs.
  if !xml.contains("<picture") {
    return Vec::new();
  }
  static PICTURE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<picture([^>]*)>(.*?)</picture>"#).unwrap());
  static ID_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"xml:id="([^"]+)""#).unwrap());
  static WIDTH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"width="([^"]+)""#).unwrap());
  static HEIGHT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"height="([^"]+)""#).unwrap());
  let mut fragments = Vec::new();
  let picture_re = &*PICTURE_RE;
  let id_re = &*ID_RE;
  let width_re = &*WIDTH_RE;
  let height_re = &*HEIGHT_RE;

  for pic_caps in picture_re.captures_iter(xml) {
    let attrs = &pic_caps[1];
    let content = &pic_caps[2];
    let id = id_re
      .captures(attrs)
      .map(|c| c[1].to_string())
      .unwrap_or_default();
    let width = width_re.captures(attrs).and_then(|c| parse_tex_dim(&c[1]));
    let height = height_re.captures(attrs).and_then(|c| parse_tex_dim(&c[1]));

    if id.is_empty() || content.trim().is_empty() {
      continue;
    }

    let w = width.unwrap_or(100.0);
    let h = height.unwrap_or(100.0);

    // Build inline SVG: coordinate system has y-flip (TeX origin bottom-left, SVG top-left)
    let mut svg_content = format!(
      r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{w:.2}" height="{h:.2}" overflow="visible">"#,
    );
    svg_content.push_str(&format!(
      r#"<g transform="translate(0,{h:.2}) scale(1,-1)">"#,
    ));

    // Convert ltx picture children to SVG elements
    svg_content.push_str(&convert_picture_children_to_svg(content));

    svg_content.push_str("</g></svg>");
    fragments.push((id, svg_content));
  }
  fragments
}

/// Convert LaTeXML picture children (g, line, text, circle, etc.) to SVG elements.
fn convert_picture_children_to_svg(content: &str) -> String {
  use std::sync::LazyLock;
  static G_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<g([^>]*)>(.*?)</g>"#).unwrap());
  static TRANSFORM_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"transform="([^"]+)""#).unwrap());
  static LINE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<line\s+points="([^"]+)"([^/]*)/?>"#).unwrap());
  static CIRCLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<circle([^/]*)/?>"#).unwrap());
  static ELLIPSE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<ellipse([^/]*)/?>"#).unwrap());
  static RECT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<rect([^/]*)/?>"#).unwrap());
  static POLYGON_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<polygon([^/]*)/?>"#).unwrap());
  static PATH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<path([^/]*)/?>"#).unwrap());
  static BEZIER_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<bezier\s+points="([^"]+)"([^/]*)/?>"#).unwrap());
  static TEXT_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<text([^>]*)>(.*?)</text>"#).unwrap());
  // Foreign (non-SVG) content that a picture makebox can carry: a math label or
  // an embedded graphic. Perl's SVG.pm wraps these in <foreignObject>.
  static MATH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"(?s)<Math\b[^>]*>.*?</Math>"#).unwrap());
  static GRAPHICS_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"<graphics\b[^>]*/>"#).unwrap());

  let mut svg = String::new();

  for g_caps in G_RE.captures_iter(content) {
    let g_attrs = &g_caps[1];
    let g_content = &g_caps[2];

    // Extract transform
    let transform = TRANSFORM_RE.captures(g_attrs).map(|c| c[1].to_string());

    if let Some(t) = &transform {
      svg.push_str(&format!(r#"<g transform="{t}">"#));
    } else {
      svg.push_str("<g>");
    }

    // <line points="x1,y1 x2,y2" stroke="..." stroke-width="..."/>
    for line_caps in LINE_RE.captures_iter(g_content) {
      let points = &line_caps[1];
      let rest_attrs = &line_caps[2];
      let coords: Vec<&str> = points.split_whitespace().collect();
      if coords.len() >= 2 {
        let p1: Vec<&str> = coords[0].split(',').collect();
        let p2: Vec<&str> = coords[1].split(',').collect();
        if p1.len() == 2 && p2.len() == 2 {
          svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}"{}/>"#,
            p1[0], p1[1], p2[0], p2[1], rest_attrs
          ));
        }
      }
    }

    // <circle cx="..." cy="..." r="..." .../>
    for circle_caps in CIRCLE_RE.captures_iter(g_content) {
      svg.push_str(&format!("<circle{}/>", &circle_caps[1]));
    }

    // <ellipse cx="..." cy="..." rx="..." ry="..." .../>
    for ellipse_caps in ELLIPSE_RE.captures_iter(g_content) {
      svg.push_str(&format!("<ellipse{}/>", &ellipse_caps[1]));
    }

    // <rect x="..." y="..." width="..." height="..." .../>
    for rect_caps in RECT_RE.captures_iter(g_content) {
      svg.push_str(&format!("<rect{}/>", &rect_caps[1]));
    }

    // <polygon points="..." .../>
    for polygon_caps in POLYGON_RE.captures_iter(g_content) {
      svg.push_str(&format!("<polygon{}/>", &polygon_caps[1]));
    }

    // <path d="..." .../>
    for path_caps in PATH_RE.captures_iter(g_content) {
      svg.push_str(&format!("<path{}/>", &path_caps[1]));
    }

    // <bezier points="x1,y1 x2,y2 x3,y3 x4,y4" .../>
    // Convert to SVG cubic bezier path
    for bez_caps in BEZIER_RE.captures_iter(g_content) {
      let points = &bez_caps[1];
      let rest = &bez_caps[2];
      let coords: Vec<&str> = points.split_whitespace().collect();
      if coords.len() >= 4 {
        // SVG cubic bezier: M x0,y0 C x1,y1 x2,y2 x3,y3
        let d = format!(
          "M {} C {} {} {}",
          coords[0], coords[1], coords[2], coords[3]
        );
        svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
      } else if coords.len() >= 3 {
        // Quadratic bezier: M x0,y0 Q x1,y1 x2,y2
        let d = format!("M {} Q {} {}", coords[0], coords[1], coords[2]);
        svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
      }
    }

    // <arc .../> — arc segments (rarely used, stub for now)
    // <wedge .../> — filled wedges (rarely used, stub for now)

    // <text>...</text> — wrap in SVG text with y-flip correction
    for text_caps in TEXT_RE.captures_iter(g_content) {
      let text_attrs = &text_caps[1];
      let text_content = &text_caps[2];
      svg.push_str(&format!(
        r#"<g transform="scale(1,-1)"><text{text_attrs}>{text_content}</text></g>"#,
      ));
    }

    // Foreign content (a `<Math>` label or a `<graphics>` image inside a picture
    // makebox) must survive into the SVG wrapped in a `<foreignObject>`, not be
    // dropped. Mirrors `SVG.pm::convertNode`
    // (LaTeXML/lib/LaTeXML/Post/SVG.pm:148-183): a non-SVG child takes its size
    // from its own width/imagewidth, else the containing `<g>`'s
    // innerwidth/width (defaults 1pt, non-zero required), and is placed in a
    // y-flipped `<g><foreignObject overflow="visible">`. html_feedback#74
    // (arXiv:0810.1673v3 Fig 2 — xfig math labels + `\epsfig` image).
    //
    // NOTE: `latexml_post::svg::SVG::convert_foreign` is the *canonical*,
    // DOM-based faithful port (it already handles this). This string-level
    // reimplementation exists only because the active SVG path splices serialized
    // strings post-XSLT to dodge a libxml2 PostDocument-cleanup UAF (see the
    // `finalize_html5` splice note). When that path can carry a live DOM, this
    // block and its siblings above are deleted in favor of the `svg.rs` Processor.
    for node in MATH_RE
      .find_iter(g_content)
      .chain(GRAPHICS_RE.find_iter(g_content))
      .map(|m| m.as_str())
    {
      let width = svg_attr(node, "width")
        .or_else(|| svg_attr(node, "imagewidth"))
        .or_else(|| svg_attr(g_attrs, "innerwidth"))
        .or_else(|| svg_attr(g_attrs, "width"))
        .unwrap_or_else(|| "1pt".to_string());
      let height = svg_attr(node, "height")
        .or_else(|| svg_attr(node, "imageheight"))
        .or_else(|| svg_attr(g_attrs, "innerheight"))
        .or_else(|| svg_attr(g_attrs, "height"))
        .unwrap_or_else(|| "1pt".to_string());
      let depth = svg_attr(node, "depth")
        .or_else(|| svg_attr(g_attrs, "innerdepth"))
        .or_else(|| svg_attr(g_attrs, "depth"))
        .unwrap_or_else(|| "0pt".to_string());
      let px_w = parse_tex_dim(&width).unwrap_or(1.0);
      let px_h = parse_tex_dim(&height).unwrap_or(1.0);
      let px_d = parse_tex_dim(&depth).unwrap_or(0.0);
      // Perl: y = to_px(height) + to_px(depth); flip so foreign content is upright.
      let y = px_h + px_d;
      svg.push_str(&format!(
        r#"<g transform="translate(0,{y:.2}) scale(1,-1)"><foreignObject width="{px_w:.2}" height="{px_h:.2}" overflow="visible">{node}</foreignObject></g>"#
      ));
    }

    svg.push_str("</g>");
  }

  // Also handle direct children not inside <g> (e.g. top-level <bezier>, <line>)
  // These appear directly inside <picture> without a <g> wrapper
  let direct_bezier_re =
    regex::Regex::new(r#"(?m)^\s*<bezier\s+points="([^"]+)"([^/]*)/?>"#).unwrap();
  for bez_caps in direct_bezier_re.captures_iter(content) {
    let points = &bez_caps[1];
    let rest = &bez_caps[2];
    let coords: Vec<&str> = points.split_whitespace().collect();
    if coords.len() >= 4 {
      let d = format!(
        "M {} C {} {} {}",
        coords[0], coords[1], coords[2], coords[3]
      );
      svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
    } else if coords.len() >= 3 {
      let d = format!("M {} Q {} {}", coords[0], coords[1], coords[2]);
      svg.push_str(&format!(r#"<path d="{d}"{rest} fill="none"/>"#));
    }
  }

  svg
}

/// Extract a `name="value"` attribute from a serialized element/attribute
/// string. Matches `name` only at an attribute boundary (string start,
/// whitespace, or after `<`), so a query for `width` does NOT match
/// `innerwidth="…"`. Returns the first such value.
fn svg_attr(s: &str, name: &str) -> Option<String> {
  let pat = format!("{name}=\"");
  let bytes = s.as_bytes();
  let mut from = 0;
  while let Some(rel) = s[from..].find(&pat) {
    let at = from + rel;
    let boundary = at == 0 || bytes[at - 1].is_ascii_whitespace() || bytes[at - 1] == b'<';
    if boundary {
      let vstart = at + pat.len();
      return s[vstart..]
        .find('"')
        .map(|end| s[vstart..vstart + end].to_string());
    }
    from = at + pat.len();
  }
  None
}

/// Parse a TeX dimension string (e.g. "100.0pt") to pixels.
fn parse_tex_dim(s: &str) -> Option<f64> {
  let s = s.trim();
  if let Some(rest) = s.strip_suffix("pt") {
    rest.parse::<f64>().ok().map(|v| v * 96.0 / 72.27)
  } else if let Some(rest) = s.strip_suffix("px") {
    rest.parse::<f64>().ok()
  } else {
    s.parse::<f64>().ok()
  }
}

#[cfg(test)]
mod finalize_html5_splice_tests {
  //! #398: the `ltx_picture` SVG splice must fill the placeholder span
  //! regardless of the serialized attribute ORDER or QUOTE style — the coupling
  //! the original `<span id="…" class="ltx_picture"…>` regex had. No live input
  //! perturbs the order today (the XSLT emits id-then-class, libxml2 preserves
  //! order and double-quotes), so these craft the perturbations directly to keep
  //! the match robust against a future serializer/XSLT change. The old regex
  //! FAILED cases 2-4 (class-first / single-quotes / an attribute between id and
  //! class); the hardened match passes them.
  use super::finalize_html5;

  fn splice(span: &str) -> String {
    finalize_html5(span.to_string(), &[(
      "p1".to_string(),
      "<svg>OK</svg>".to_string(),
    )])
  }

  #[test]
  fn splice_is_attribute_order_and_quote_agnostic() {
    // 1. Canonical (id-first, double-quote) — the only shape seen live.
    assert!(splice(r#"<span id="p1" class="ltx_picture"></span>"#).contains("<svg>OK</svg>"));
    // 2. class BEFORE id — the old regex required id-then-class.
    assert!(splice(r#"<span class="ltx_picture" id="p1"></span>"#).contains("<svg>OK</svg>"));
    // 3. single quotes — the old regex hard-coded double quotes.
    assert!(splice(r#"<span id='p1' class='ltx_picture'></span>"#).contains("<svg>OK</svg>"));
    // 4. an attribute BETWEEN id and class — the old regex needed them adjacent.
    assert!(
      splice(r#"<span id="p1" style="color:red" class="ltx_picture"></span>"#)
        .contains("<svg>OK</svg>")
    );
    // The placeholder's own attributes are preserved (only the content is filled).
    assert!(
      splice(r#"<span class="ltx_picture" id="p1"></span>"#).contains(r#"class="ltx_picture""#)
    );
  }

  #[test]
  fn splice_leaves_non_targets_alone() {
    // Not a picture span → untouched.
    assert_eq!(
      splice(r#"<span id="p1" class="ltx_note"></span>"#),
      r#"<span id="p1" class="ltx_note"></span>"#
    );
    // A picture span with an id we have NO fragment for → untouched.
    assert_eq!(
      splice(r#"<span id="other" class="ltx_picture"></span>"#),
      r#"<span id="other" class="ltx_picture"></span>"#
    );
    // `ltx_picture` must be a whole class token, not a substring.
    assert_eq!(
      splice(r#"<span id="p1" class="ltx_picturewide"></span>"#),
      r#"<span id="p1" class="ltx_picturewide"></span>"#
    );
  }
}

#[cfg(test)]
mod picture_svg_foreign_content_tests {
  //! html_feedback#74 (arXiv:0810.1673v3 Fig 2): a `<Math>` or `<graphics>`
  //! inside a `{picture}` `\makebox` must survive into the SVG, wrapped in a
  //! `<foreignObject>` (as Perl's `SVG.pm` does), not be dropped.
  //! `convert_picture_children_to_svg` handled the drawing primitives + `<text>`
  //! but had NO case for foreign content, so xfig/pstricks figures silently lost
  //! their math labels and `\epsfig` images. Plain text is the baseline; math and
  //! graphics are the canaries.
  use super::convert_picture_children_to_svg;

  #[test]
  fn plain_text_label_still_renders() {
    // The shape emitted for `\put(50,50){\makebox(0,0){Hello}}` — already worked.
    let input = r#"<g transform="translate(50,50)"><text>Hello</text></g>"#;
    let out = convert_picture_children_to_svg(input);
    assert!(
      out.contains("Hello"),
      "plain-text picture label regressed: {out}"
    );
  }

  #[test]
  fn math_label_survives_as_foreignobject() {
    // The shape emitted for `\put(20,20){\makebox(0,0){$X^{2}$}}`.
    let input = r#"<g transform="translate(20,20)" innerwidth="13.55pt" innerheight="8.14pt"><Math mode="inline" tex="X^{2}"><XMath><XMTok>X</XMTok></XMath></Math></g>"#;
    let out = convert_picture_children_to_svg(input);
    assert!(
      out.contains("foreignObject") && out.contains("Math"),
      "math inside a picture makebox was dropped, not wrapped in <foreignObject>: {out}"
    );
  }

  #[test]
  fn embedded_graphic_survives_as_foreignobject() {
    // The shape emitted for `\epsfig{file=link.ps}` inside a picture.
    let input = r#"<g transform="translate(0,0)" innerwidth="277pt" innerheight="277pt"><graphics candidates="link.ps" graphic="link.ps"/></g>"#;
    let out = convert_picture_children_to_svg(input);
    assert!(
      out.contains("foreignObject") && out.contains("graphic"),
      "an embedded picture graphic was dropped, not wrapped in <foreignObject>: {out}"
    );
  }
}
