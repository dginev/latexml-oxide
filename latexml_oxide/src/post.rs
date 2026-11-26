//! Post-processing pipeline API.
//!
//! Provides a public interface to the LaTeXML post-processing pipeline
//! (Scan → Bibliography → CrossRef → Graphics → Split → MathML → XSLT → HTML5 fixups).
//! Used by both the `latexml_oxide` binary and the `cortex_worker` binary.

use std::sync::LazyLock;

use latexml_core::{
  Info, Warn,
  common::error::{emit_error, emit_fatal, note_progress},
  s,
  telemetry::{self, Phase},
};
use latexml_post::{
  document::{PostDocument, PostDocumentOptions},
  object_db::ObjectDB,
  processor::Processor,
};

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static POST_AUDIT: LazyLock<bool> = LazyLock::new(|| std::env::var("LATEXML_POST_AUDIT").is_ok());

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
  /// Link and copy the bundled CSS/JS resources (Perl `defaultresources`, Config.pm:120; `--nodefaultresources`
  /// clears it). The XSLT step receives its negation as `no_resources`, as LaTeXML.pm:584 hands `noresources`.
  pub default_resources:         bool,
  pub css_files:                 &'a [String],
  pub js_files:                  &'a [String],
  pub noinvisibletimes:          bool,
  /// Remap styled alphanumerics to Unicode Plane-1 (Perl `--plane1`, default on).
  pub plane1:                    bool,
  /// Remap only the poorly-supported variants, to their simpler form
  /// (Perl `--hackplane1`); implies `plane1`.
  pub hackplane1:                bool,
  pub mathtex:                   bool,
  /// Add a UnicodeMath annotation to the parallel markup (Perl `--unicodemath`,
  /// Config.pm L113); with no MathML format requested, UnicodeMath is the only one.
  pub unicodemath:               bool,
  /// Cross-reference URL style (Perl `--urlstyle`; `CrossRef::generateURL`).
  /// Defaults to [`UrlStyle::File`](latexml_post::crossref::UrlStyle::File) at
  /// every non-CLI construction site (the binary threads the parsed `--urlstyle`
  /// value here).
  pub url_style:                 latexml_post::crossref::UrlStyle,
  /// The navigation TOC style, `context` or `normal` (Perl `--navigationtoc`, lowercased and checked by the
  /// caller; `none` is `None`, Config.pm L472-480).
  pub navigationtoc:             Option<&'a str>,
  /// Serialize the pages as HTML (Perl `is_html`, Config.pm L436: the format is `html*`), else as XML. `None`: a
  /// caller naming only a built-in sheet, whose name says it (`LaTeXML-html5.xsl`).
  pub is_html:                   Option<bool>,
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
  /// Keep the destination's cache (Perl's document cache, `LaTeXML-oxide.cache`) in memory, writing no file (Perl's
  /// `nocache`, `Post.pm` L678): the cortex worker, whose every paper gets a fresh directory that is packed.
  pub nocache:                   bool,
}

impl Default for PostOptions<'_> {
  fn default() -> Self {
    Self {
      pmml:                      false,
      cmml:                      false,
      keep_xmath:                false,
      stylesheet:                None,
      destination:               None,
      source_directory:          None,
      site_directory:            None,
      search_paths:              &[],
      default_resources:         true,
      css_files:                 &[],
      js_files:                  &[],
      noinvisibletimes:          false,
      plane1:                    true,
      hackplane1:                false,
      mathtex:                   false,
      unicodemath:               false,
      url_style:                 latexml_post::crossref::UrlStyle::File,
      navigationtoc:             None,
      is_html:                   None,
      schemadocs:                false,
      split:                     false,
      split_xpath:               None,
      split_naming:              None,
      xslt_parameters:           &[],
      graphics_svg_threshold_kb: 0,
      graphicimages:             true,
      timestamp:                 None,
      icon:                      None,
      whatsout:                  latexml_post::extract::Whatsout::Document,
      nocache:                   false,
    }
  }
}

thread_local! {
  /// One-shot CLI flag: release dead core TeX engine state on this thread right
  /// after `MakeBibliography` finishes (before `parallel_render` sizes the
  /// worker fleet and before render phases). Armed only by the one-shot
  /// `latexml_oxide` binary so persistent workers (`cortex_worker`, LSP, library
  /// API) never tear down their thread-local engine/arena.
  static RELEASE_CORE_AFTER_BIB: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// RAII guard returned by [`arm_core_release_after_bib`] that clears the flag on
/// drop even if post-processing returns early before `MakeBibliography`.
pub struct CoreReleaseAfterBibGuard;

impl Drop for CoreReleaseAfterBibGuard {
  fn drop(&mut self) { RELEASE_CORE_AFTER_BIB.with(|f| f.set(false)); }
}

/// Arm one-shot teardown of dead core TeX engine state (`STATE`, `STOMACH`,
/// `GULLET`, `StringInterner`, and per-conversion caches) on the current thread
/// right after `MakeBibliography` completes in `run_post_processing_*`.
///
/// Only call this from a one-shot CLI process whose thread will not re-enter
/// the TeX engine afterward (see [`latexml_core::release_core_state_for_post`]).
pub fn arm_core_release_after_bib() -> CoreReleaseAfterBibGuard {
  RELEASE_CORE_AFTER_BIB.with(|f| f.set(true));
  CoreReleaseAfterBibGuard
}

/// The requested math formats as the post chain's one math processor: a primary with
/// its parallel secondaries (Perl `LaTeXML.pm` L504-556, order per OXIDIZED_DESIGN
/// #1093), shared by the in-process renderer and the page-render workers.
#[derive(Clone, Copy)]
pub(crate) struct MathFormats {
  pub pmml:            bool,
  pub cmml:            bool,
  pub unicodemath:     bool,
  pub keep_xmath:      bool,
  pub invisible_times: bool,
  pub plane1:          bool,
  pub hackplane1:      bool,
  pub mathtex:         bool,
  pub intent_literal:  bool,
}

impl MathFormats {
  pub(crate) fn processor(self) -> Option<Box<dyn Processor>> {
    use latexml_post::{math_processor::MathProcessor, mathml::MathML, unicode_math::UnicodeMath};
    let content = || {
      MathML::new_content()
        .with_keep_xmath(self.keep_xmath)
        .with_invisible_times(self.invisible_times)
        .with_plane1(self.plane1, self.hackplane1)
    };
    let unicode = || UnicodeMath::new().with_keep_xmath(self.keep_xmath);
    let unicode_secondary = || -> Option<Box<dyn MathProcessor>> {
      self
        .unicodemath
        .then(|| Box::new(unicode().secondary()) as _)
    };
    if self.pmml {
      let mut secondaries: Vec<Box<dyn MathProcessor>> = Vec::new();
      if self.cmml {
        secondaries.push(Box::new(content().secondary()));
      }
      secondaries.extend(unicode_secondary());
      Some(Box::new(
        MathML::new_presentation()
          .with_keep_xmath(self.keep_xmath)
          .with_invisible_times(self.invisible_times)
          .with_plane1(self.plane1, self.hackplane1)
          .with_mathtex(self.mathtex)
          .with_intent_literal(self.intent_literal)
          .with_secondaries(secondaries),
      ))
    } else if self.cmml {
      Some(Box::new(
        content().with_secondaries(unicode_secondary().into_iter().collect()),
      ))
    } else if self.unicodemath {
      Some(Box::new(unicode()))
    } else {
      None
    }
  }
}

/// The built-in XSLT stylesheet for an output format (Perl `Config.pm` L543-551), as the
/// `embed:///` name served from the binary, never a copy on disk (#806). Returns `None` for a
/// format with no default (e.g. `xml`), so a caller keeps `--stylesheet` / no post.
///
/// SINGLE SOURCE OF TRUTH shared by the CLI (`bin/latexml_oxide.rs` picks
/// `effective_stylesheet` when `--stylesheet` is unset) and the library
/// (`crate::api::convert_to_html`), so the two can never disagree on which
/// sheet an `html5`/`xhtml`/`epub` job gets.
pub fn default_stylesheet(format: Option<&str>) -> Option<&'static str> {
  match format {
    // Config.pm L436: `html` is html5.
    Some("html5") | Some("html") => Some("embed:///LaTeXML-html5.xsl"),
    Some("xhtml") => Some("embed:///LaTeXML-xhtml.xsl"),
    Some("epub") | Some("epub3") => Some("embed:///LaTeXML-epub3.xsl"),
    _ => None,
  }
}

/// Split one `--xsltparameter` into `(name, value)`: Perl's `name:value` (LaTeXML.pm L575
/// `/^\s*(\w+)\s*:\s*(.*)$/`, bin/latexmlpost L354) first, else our `name=value`; `None` when
/// it is neither (#803).
pub fn parse_xslt_parameter(param: &str) -> Option<(String, String)> {
  let trimmed = param.trim_start();
  let name_len = trimmed
    .find(|c: char| !(c.is_alphanumeric() || c == '_'))
    .unwrap_or(trimmed.len());
  if name_len > 0
    && let Some(value) = trimmed[name_len..].trim_start().strip_prefix(':')
  {
    return Some((
      trimmed[..name_len].to_string(),
      value.trim_start().to_string(),
    ));
  }
  match param.split_once('=') {
    Some((name, value)) if !name.is_empty() => Some((name.to_string(), value.to_string())),
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
  /// The pages written, in the order the post pipeline produced them — Split's (the root, then each page
  /// before its children), the order Perl's `@docs` reaches `Manifest::Epub::process` in for the EPUB spine.
  pub pages:       Vec<String>,
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
  // An XML input has no core session to log what the front end held for it (`warn_at_session_start`).
  latexml_core::common::error::log_session_warnings();
  let mut pages = Vec::new();
  let html = run_post_processing_impl(input, opts, &mut pages);
  let log = latexml_core::util::logger::flush_log();
  let status_code = latexml_core::common::error::get_status_code();
  PostOutcome { html, log, status_code, pages }
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
  run_post_processing_impl(PostInput::Xml(xml), opts, &mut Vec::new())
}

/// File-input variant of [`run_post_processing`].
pub fn run_post_processing_from_file(path: &str, opts: &PostOptions) -> String {
  run_post_processing_impl(PostInput::File(path), opts, &mut Vec::new())
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
/// (`latexml_post::stream_split`) over the whole-DOM parse. Default 0: every
/// split run streams, since its pages are byte-identical to the whole-DOM
/// split's (guard: `118_streaming_split_parity`) while its peak is one content
/// subtree, not the whole DOM (Nasser's 2,400-problem book, 337 MB of HTML in
/// 9,713 pages, with the split run's core watermark: 3.8 → 1.35 GB peak RSS).
/// Calibration/test override: `LATEXML_POST_STREAM_THRESHOLD` (bytes).
fn stream_split_threshold() -> u64 {
  std::env::var("LATEXML_POST_STREAM_THRESHOLD")
    .ok()
    .and_then(|v| v.parse::<u64>().ok())
    .unwrap_or(0)
}

/// Below this size a failed streaming split falls back to the whole-DOM path,
/// the pre-default behaviour (a 614 MB core XML parsed to ~16 GB of DOM);
/// above it the fallback is the OOM the streaming front-end exists to avoid
/// (a 2.68 GB core XML OOM'd a 31 GB host), so the failure stays an Error.
const STREAM_SPLIT_DOM_FALLBACK_LIMIT: u64 = 1 << 30;

/// Gate for the streaming split on an input of `len` bytes:
/// `LATEXML_POST_STREAM_SPLIT=1` forces it on (any size), `=0` disables it,
/// unset engages it at or above [`stream_split_threshold`] (by default every
/// split run).
fn stream_split_gate(len: u64) -> bool {
  match std::env::var("LATEXML_POST_STREAM_SPLIT").as_deref() {
    Ok("0") => false,
    Ok(_) => true,
    Err(_) => len >= stream_split_threshold(),
  }
}

/// Will a run with `opts` over an input of `len` bytes take the streaming split
/// front-end? The front-end's own preconditions (a destination, a union it can
/// evaluate) and the gate. An in-memory handoff that answers yes is staged to a
/// file first: the streaming reader parses only files.
fn takes_streaming_split(opts: &PostOptions, len: u64) -> bool {
  opts.split
    && opts.destination.is_some()
    && opts
      .split_xpath
      .as_deref()
      .is_some_and(latexml_post::stream_split::supports_union)
    && stream_split_gate(len)
}

/// Test override: `LATEXML_POST_SPILL_PAGE=1` stages an unsplit page on disk as
/// a split one is, so the parity guard can compare the two paths.
fn spill_every_page() -> bool {
  std::env::var_os("LATEXML_POST_SPILL_PAGE").is_some_and(|v| v == "1")
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

/// The destination's directory for the staged handoff and the page spill: "."
/// for a bare file name (`--dest=index.html`), whose empty parent would
/// otherwise send them to the system temp dir (often a RAM-backed tmpfs).
fn staging_dir_of(destination: Option<&str>) -> Option<std::path::PathBuf> {
  let parent = std::path::Path::new(destination?).parent()?;
  Some(if parent.as_os_str().is_empty() {
    std::path::PathBuf::from(".")
  } else {
    parent.to_path_buf()
  })
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
    !d.findnodes(latexml_post::make_index::PLACEHOLDERS)
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
  intent_literal: bool,
}

/// Attempt the streaming split front-end (`latexml_post::stream_split`).
///
/// `Ok(None)` = not applicable (the union matched no pages, or the stream
/// could not be split faithfully — already reported) → the caller falls back
/// to the whole-DOM pipeline. `Err(())` = a hard failure after pages were in
/// flight (I/O, resource ceiling), already reported via `post_error`; the
/// caller bails out. With `dom_fallback` (an input below
/// [`STREAM_SPLIT_DOM_FALLBACK_LIMIT`]) a failed stream is `Ok(None)` instead,
/// reported at Info.
fn try_streaming_front(
  path: &str,
  split_xpath: &str,
  split_naming: Option<&str>,
  page_opts: &PostDocumentOptions,
  spill_dir: &std::path::Path,
  dom_fallback: bool,
  mut graphics: Option<&mut latexml_post::graphics::Graphics>,
) -> Result<Option<PostFront>, ()> {
  let naming = resolve_split_naming(split_naming);
  telemetry::phase_enter(Phase::Split);
  let outcome = latexml_post::stream_split::stream_split(
    path,
    split_xpath,
    naming,
    // The run's `--dest`, as `run_post_processing_inner` set it on the page options.
    page_opts.destination.as_deref(),
    spill_dir,
    dom_fallback,
  );
  telemetry::phase_exit();
  let outcome = match outcome {
    Ok(Some(o)) => o,
    Ok(None) => return Ok(None),
    // A file the whole-DOM path parses comfortably: give it that path, as
    // before every split run streamed.
    Err(e) if dom_fallback => {
      Info!(
        "post",
        "stream-split",
        s!("streaming split failed ({e}); splitting the whole document instead")
      );
      return Ok(None);
    },
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

  // Pre-order Scan sweep over the spilled pages — the streaming equivalent of
  // `run_phase(docs, Scan)` + the driver's spill loop, ONE page resident at a
  // time. The order is semantic, not cosmetic: SITE_ROOT is taken from the
  // first page scanned (the root page), an ancestor must be in the DB before
  // its descendants (Scan files a page under its Split parent's entry, the
  // page's `parent_id`, and lists it among that entry's `children`), and each
  // parent's `children` list follows scan order — all three feed CrossRef's
  // navigation.
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
        d.parent_id = page.parent_id.clone();
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
    let has_scan_nodes = !nodes.is_empty();
    let processed = if !has_scan_nodes {
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
    let scan_placeholders = if has_scan_nodes {
      scanner.last_page_placeholders()
    } else {
      (false, false)
    };
    for d in processed {
      if root_unnamed && let Err(e) = std::fs::write(&page.path, d.to_xml_string()) {
        post_error("post", &format!("cannot refresh a disk-staged page: {e}"));
        telemetry::phase_exit();
        return Err(());
      }
      #[cfg(debug_assertions)]
      debug_assert_eq!(
        scan_placeholders,
        page_placeholder_probes(&d),
        "Scan placeholder tracking diverged from XPath probe on {}",
        page.destination
      );
      if let Some(graphics) = graphics.as_deref_mut()
        && outcome.pages.len() > 1
      {
        graphics.plan_names(&d);
      }
      let (needs_index, needs_bib) = scan_placeholders;
      spilled.push(SpilledPage {
        needs_index,
        needs_bib,
        path: page.path.clone(),
        destination: Some(page.destination.clone()),
        destination_directory: destination_directory_of(&page.destination),
        resident: None,
      });
      // The page's DOM drops here — one resident at a time.
    }
    if !spilled.is_empty() && spilled.len().is_multiple_of(crate::heap_trim::SAMPLE_PAGES) {
      crate::heap_trim::sample();
    }
  }
  telemetry::phase_exit();
  Ok(Some(PostFront {
    spilled_pages: spilled,
    db: scanner.db,
    intent_literal,
  }))
}

/// A core handoff staged on disk for the streaming file parser, removed when
/// dropped and on the exits that skip destructors.
pub struct StagedHandoff {
  path:   String,
  _guard: latexml_core::watchdog::RemoveOnExit,
}

impl Drop for StagedHandoff {
  fn drop(&mut self) { let _ = std::fs::remove_file(&self.path); }
}

impl StagedHandoff {
  /// The handoff's path beside the destination ("." for a bare file name), else in the system temp dir.
  fn at(destination: Option<&str>) -> StagedHandoff {
    let dir = staging_dir_of(destination).unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!("latexml-post-handoff-{}.xml", std::process::id()));
    StagedHandoff {
      _guard: latexml_core::watchdog::remove_on_exit(&path),
      path:   path.to_string_lossy().into_owned(),
    }
  }

  pub fn path(&self) -> &str { &self.path }
}

/// The reserved handoff, kept only when the core serialized into it (`ConversionResponse::handoff_file`). An
/// unused one is dropped here, removing a file an earlier attempt wrote: a fused eager attempt's salvage, when the
/// `--streaming` rerun on the same path ends without a document.
pub fn used_core_handoff(
  reserved: Option<StagedHandoff>,
  serialized_to: Option<&str>,
) -> Option<StagedHandoff> {
  reserved.filter(|staged| serialized_to == Some(staged.path()))
}

/// The handoff file a split run's core result is serialized into (`Config::handoff_file`), reserved before the
/// conversion: when the run would take the streaming split, which parses only files, the document-sized `String`
/// would exist only to be staged (2 GiB on Nasser's whole book, issue 777). `None` when it would not.
pub fn reserve_core_handoff(
  split: bool,
  destination: Option<&str>,
  split_xpath: Option<&str>,
) -> Option<StagedHandoff> {
  let streams = split
    && destination.is_some()
    && split_xpath.is_some_and(latexml_post::stream_split::supports_union)
    && std::env::var("LATEXML_POST_STREAM_SPLIT").as_deref() != Ok("0");
  streams.then(|| StagedHandoff::at(destination))
}

/// The staging Info: operationally worth a line (a temp file the size of the core XML just appeared beside the
/// destination), and the guard tests' engagement proof — without it a silently-failing spill would fall back to the
/// memory parse and the tests would pass vacuously.
fn note_staged(len: u64, path: &str, how: &str) {
  let why = if len >= post_mem_parse_limit() as u64 {
    "oversized handoff"
  } else {
    "streaming-split handoff"
  };
  latexml_core::Info!(
    "post",
    "spill",
    format!("{why} ({len} bytes) staged to {path}{how}")
  );
}

/// Should an in-memory handoff be staged to a file: past the in-memory parse
/// ceiling (`post_mem_parse_limit`), or bound for the streaming split, which
/// parses only files. An empty handoff stays in memory: it is post-processed
/// against a bare `<document/>` (Perl LaTeXML.pm L330-336).
fn handoff_needs_file(xml: &str, opts: &PostOptions) -> bool {
  xml.len() >= post_mem_parse_limit()
    || (!xml.trim().is_empty() && takes_streaming_split(opts, xml.len() as u64))
}

/// Write the handoff to a temp file beside the destination when there is one
/// (writing to the destination directory is always allowed), else the system
/// temp dir. `None` = the write failed (reported at Info): parse from memory.
fn stage_handoff(xml: &str, opts: &PostOptions) -> Option<StagedHandoff> {
  let staged = StagedHandoff::at(opts.destination);
  // The destination's directory may not exist yet: the writer creates it, so
  // create it here, where the handoff and the page spill go first.
  let dir = std::path::Path::new(&staged.path).parent();
  if let Err(e) = dir
    .map_or(Ok(()), std::fs::create_dir_all)
    .and_then(|()| std::fs::write(&staged.path, xml))
  {
    // (the drop removes a partial file)
    latexml_core::Info!(
      "post",
      "spill",
      format!("could not stage the core handoff to disk ({e}); parsing from memory")
    );
    return None;
  }
  note_staged(xml.len() as u64, &staged.path, "");
  Some(staged)
}

/// [`run_post_processing_logged`] taking the core result by value: a handoff
/// staged to disk (see `handoff_needs_file`) is dropped before post runs, so
/// the core XML is not resident beside the post pipeline — on Nasser's
/// 2,400-problem book a few hundred MB of the single-invocation peak.
pub fn run_post_processing_owned_logged(xml: String, opts: &PostOptions) -> PostOutcome {
  // Bound first: the staging Info belongs to the post log.
  latexml_core::util::logger::bind_log();
  latexml_core::common::error::log_session_warnings();
  latexml_core::common::arena::take_overflow_interns();
  let staged = if handoff_needs_file(&xml, opts) {
    stage_handoff(&xml, opts)
  } else {
    None
  };
  let mut pages = Vec::new();
  // Straight to the inner pipeline: a handoff whose staging failed must not
  // be staged a second time.
  let html = match staged {
    Some(staged) => {
      drop(xml);
      let path = staged.path.clone();
      run_post_processing_inner(PostInput::File(&path), opts, &mut pages, Some(staged))
    },
    None => run_post_processing_inner(PostInput::Xml(&xml), opts, &mut pages, None),
  };
  let log = latexml_core::util::logger::flush_log();
  let status_code = latexml_core::common::error::get_status_code();
  PostOutcome { html, log, status_code, pages }
}

/// [`run_post_processing_owned_logged`] for a core result the conversion serialized into `staged`
/// ([`reserve_core_handoff`]), so no `String` of it ever existed. A file the gate keeps in memory (a test's
/// `LATEXML_POST_STREAM_THRESHOLD` above its size) is read back and post-processed as that `String` would be.
pub fn run_post_processing_staged_logged(staged: StagedHandoff, opts: &PostOptions) -> PostOutcome {
  latexml_core::util::logger::bind_log();
  latexml_core::common::error::log_session_warnings();
  latexml_core::common::arena::take_overflow_interns();
  let len = std::fs::metadata(&staged.path).map_or(0, |m| m.len());
  let mut pages = Vec::new();
  let html = if len >= post_mem_parse_limit() as u64 || takes_streaming_split(opts, len) {
    note_staged(len, &staged.path, " by the core serializer");
    let path = staged.path.clone();
    run_post_processing_inner(PostInput::File(&path), opts, &mut pages, Some(staged))
  } else {
    match std::fs::read_to_string(&staged.path) {
      Ok(xml) => {
        drop(staged);
        run_post_processing_inner(PostInput::Xml(&xml), opts, &mut pages, None)
      },
      // (unreadable: the file parser reports it as the failure it is)
      Err(_) => {
        let path = staged.path.clone();
        run_post_processing_inner(PostInput::File(&path), opts, &mut pages, Some(staged))
      },
    }
  };
  let log = latexml_core::util::logger::flush_log();
  let status_code = latexml_core::common::error::get_status_code();
  PostOutcome { html, log, status_code, pages }
}

/// `written` collects the pages written, in the pipeline's order (see [`PostOutcome::pages`]).
fn run_post_processing_impl(
  input: PostInput,
  opts: &PostOptions,
  written: &mut Vec<String>,
) -> String {
  // A conversion entry point: the arena's overflow count starts at zero, as in `Converter::convert`.
  latexml_core::common::arena::take_overflow_interns();
  // An in-memory handoff past the parse ceiling, or bound for the streaming
  // split, parses from a staged file (dropped, and so removed, once the front-end has read it); a staging
  // failure falls through to the memory parse, whose own error reporting then
  // names the real problem.
  let staged = match input {
    PostInput::Xml(xml) if handoff_needs_file(xml, opts) => stage_handoff(xml, opts),
    _ => None,
  };
  let path = staged.as_ref().map(|staged| staged.path.clone());
  let input = match path.as_deref() {
    Some(path) => PostInput::File(path),
    None => input,
  };
  run_post_processing_inner(input, opts, written, staged)
}

/// `staged`: the handoff file `input` reads, removed once a front-end has
/// read it (the page spill then holds the document).
fn run_post_processing_inner(
  input: PostInput,
  opts: &PostOptions,
  written: &mut Vec<String>,
  staged: Option<StagedHandoff>,
) -> String {
  let PostOptions {
    pmml,
    cmml,
    keep_xmath,
    stylesheet,
    destination,
    source_directory,
    site_directory,
    search_paths,
    default_resources,
    css_files,
    js_files,
    noinvisibletimes,
    plane1,
    hackplane1,
    mathtex,
    unicodemath,
    url_style,
    navigationtoc,
    is_html,
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
    nocache,
  } = *opts;
  // Perl `Common/Config.pm` L395-397: the destination and the source and site directories are canonical before post
  // reads them, so the root's spelling is its split pages' (`./out/index.html` is `out/index.html`, issue 1006).
  let canonical = |path: Option<&str>| path.map(latexml_core::util::pathname::canonical);
  let (destination, source_directory, site_directory) = (
    canonical(destination),
    canonical(source_directory),
    canonical(site_directory),
  );
  let (destination, source_directory, site_directory) = (
    destination.as_deref(),
    source_directory.as_deref(),
    site_directory.as_deref(),
  );

  // Perl's document cache, shared by the document and every page of it (`Post.pm` L687-690), closed (its side files
  // checkpointed away) when post ends, before anything is packed.
  let run_cache = latexml_post::document::DocCache::new(nocache);
  let _close_cache = CloseCache(run_cache.clone());
  let mut doc_opts = PostDocumentOptions {
    nocache,
    cache: Some(run_cache.clone()),
    ..Default::default()
  };
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
    // Perl `Post.pm` L698-706 (`Post::Document::new_internal`): the destination must lie within the site directory,
    // as written (Perl) or with both sides resolved. A relative destination is completed from the working directory,
    // which the OS reports resolved (macOS `/var` is `/private/var`), so a site directory named through a link fails
    // the lexical test alone (KNOWN_PERL_ERRORS #1334); a destination named inside the site through a link that
    // leads out passes it, and is written at its lexical path (batchfix995).
    if let Some(dest) = destination {
      let dest_dir = std::path::Path::new(dest)
        .parent()
        .map(|dir| dir.to_string_lossy().into_owned())
        .filter(|dir| !dir.is_empty())
        .unwrap_or_else(|| ".".to_string());
      use latexml_core::util::pathname::{is_contained, resolved};
      let inside = is_contained(&dest_dir, site_dir).is_some()
        || is_contained(&resolved(&dest_dir), &resolved(site_dir)).is_some();
      if !inside {
        emit_fatal(
          "unexpected",
          &dest_dir,
          &format!(
            "The destination directory ({dest_dir}) must be within the siteDirectory ({site_dir})"
          ),
        );
        return String::new();
      }
    }
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
  let page_spill = match staging_dir_of(destination).filter(|p| p.is_dir()) {
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
  // A watchdog or allocation-failure exit skips the `TempDir`'s drop.
  let _spill_on_exit = latexml_core::watchdog::remove_on_exit(page_spill.path());

  // ---- Front-end selection --------------------------------------------------
  //
  // For a *file* input that will be split and is past the streaming threshold
  // (or force-enabled), the streaming split front-end produces the spilled
  // pages + scanned ObjectDB without ever parsing the whole document — the
  // whole-DOM parse of a 2.68 GB core XML needs ~30+ GB and OOM'd a 31 GB
  // host before Split ran (laptop UAT 2026-07-31). Everything downstream of
  // this block is shared between the two front-ends.
  // Phase 5: Graphics (Perl `graphicimages!` → `dographics`, default on).
  // `--nographicimages` skips the phase wholesale, leaving the raw
  // `<ltx:graphics>` references in the output untouched. Made here: the pages of a split name their images as they
  // are spilled (`Graphics::plan_names`, issue 995).
  let mut graphics_proc = graphicimages.then(|| {
    latexml_post::graphics::Graphics::new(None, true)
      .with_svg_threshold_kb(graphics_svg_threshold_kb)
      .with_doc_cache(run_cache.clone())
  });
  let file_len = |path: &str| std::fs::metadata(path).map(|m| m.len()).ok();
  let streamed: Option<PostFront> = if let PostInput::File(path) = input
    && let Some(len) = file_len(path)
    && takes_streaming_split(opts, len)
    && let Some(xpath) = split_xpath.as_deref()
  {
    match try_streaming_front(
      path,
      xpath,
      split_naming,
      &page_opts,
      page_spill.path(),
      len < STREAM_SPLIT_DOM_FALLBACK_LIMIT,
      graphics_proc.as_mut(),
    ) {
      Ok(front) => front,
      Err(()) => return fallback(),
    }
  } else {
    None
  };

  let front = if let Some(front) = streamed {
    // The pages are spilled: the staged handoff is read no more.
    drop(staged);
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
    drop(staged);

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
    //
    // One unsplit page stays resident instead, as in Perl (LaTeXML.pm L445): see
    // `SpilledPage::resident` for why that is the reparse's DOM.
    let keep_resident =
      !split && docs.len() == 1 && docs.iter().all(parsed_cleanly) && !spill_every_page();
    let mut spilled_pages = Vec::with_capacity(docs.len());
    for (i, d) in docs.drain(..).enumerate() {
      let path = page_spill.path().join(format!("page-{i:07}.xml"));
      if !keep_resident && let Err(e) = std::fs::write(&path, d.to_xml_string()) {
        post_error("post", &format!("cannot stage page {i} to disk: {e}"));
        return fallback();
      }
      if let Some(graphics) = graphics_proc.as_mut()
        && !keep_resident
      {
        graphics.plan_names(&d);
      }
      let (needs_index, needs_bib) = page_placeholder_probes(&d);
      let destination = d.get_destination().map(String::from);
      let destination_directory = d.get_destination_directory().map(String::from);
      let resident = keep_resident.then(|| as_reparsed(d, &path, &page_opts));
      spilled_pages.push(SpilledPage {
        needs_index,
        needs_bib,
        path,
        destination,
        destination_directory,
        resident,
      });
      // `d` (unless resident) drops here: one page's worth of DOM and caches
      // released before the next is touched.
    }
    PostFront {
      spilled_pages,
      db: scanner.db,
      intent_literal,
    }
  }; // ==== end of the whole-DOM front-end ====
  let PostFront {
    mut spilled_pages,
    db,
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
  let is_html_out = is_html.unwrap_or_else(|| stylesheet.is_some_and(|s| s.contains("html")));
  let out_extension = destination
    .and_then(|d| std::path::Path::new(d).extension().and_then(|e| e.to_str()))
    .map(str::to_string)
    .unwrap_or_else(|| {
      if is_html_out {
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

  // Phase 3: SVG + MathML + XSLT
  let post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();
  // SVG first, on the live page DOM — Perl's chain order (`LaTeXML.pm`
  // L493-581: Graphics → SVG → MathML → XSLT; SVG for the HTML output and the
  // html-family sheets, where Perl's `$$opts{svg}` is on for every sheet but html4's,
  // Config.pm L560). The Graphics page-phase has already resolved every
  // picture-nested `<ltx:graphics>` by the time this runs, so
  // `SVG::convert_foreign` wraps the resolved node and the XSLT renders it
  // (#1291's Graphics→SVG order comes for free).
  let svg = is_html_out || stylesheet.is_some_and(|s| s.contains("html"));
  if svg {
    processors.push(Box::new(latexml_post::svg::SVG::new()));
  }

  // `intent_literal` was computed up-front (before Split consumed `doc`).
  let math_formats = MathFormats {
    pmml,
    cmml,
    unicodemath,
    keep_xmath,
    invisible_times: !noinvisibletimes,
    plane1,
    hackplane1,
    mathtex,
    intent_literal,
  };
  if let Some(math) = math_formats.processor() {
    processors.push(math);
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
      match parse_xslt_parameter(param) {
        Some((key, value)) => {
          xslt_params.insert(key, format!("\"{}\"", value));
        },
        None => post_error(
          "xsltparameter",
          &format!("xsltparameter not in recognized format: 'name:value' got: '{param}'"),
        ),
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
      !default_resources,
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
    pages: &mut [SpilledPage],
    opts: &PostDocumentOptions,
    proc: &mut dyn Processor,
    label: &'static str,
    selector: fn(&SpilledPage) -> bool,
  ) -> Result<(), ()> {
    for page_meta in pages.iter_mut().filter(|p| selector(p)) {
      let (path, dest, dest_dir) = (
        &page_meta.path,
        &page_meta.destination,
        &page_meta.destination_directory,
      );
      let path_str = path.to_string_lossy().into_owned();
      let resident = page_meta.resident.is_some();
      let mut page = match page_meta.resident.take() {
        Some(d) => d,
        None => PostDocument::new_from_file(&path_str, opts.clone()).map_err(|e| {
          post_error("post", &format!("cannot re-read a disk-staged page: {e}"));
        })?,
      };
      page.destination = dest.clone();
      page.destination_directory = dest_dir.clone();
      let nodes = proc.to_process(&page);
      if nodes.is_empty() {
        // untouched: the spill on disk (or the resident DOM) is still current
        if resident {
          page_meta.resident = Some(as_reparsed(page, path, opts));
        }
        continue;
      }
      let processed = proc.process(page, nodes).map_err(|e| {
        post_error(label, &format!("{label} failed: {e}"));
      })?;
      // A phase may split one page into several (an index page beside its
      // host); the extras are written by their own destination in pass B, so
      // re-spill each under the original path only for the first and append
      // the rest as additional spills is NOT needed here: MakeIndex and
      // MakeBibliography fill placeholders in place and return the same page.
      // (Both return `vec![doc]` today; an empty result would leave a resident
      // page with nothing to render, so it is an error, never a silent skip.)
      let Some(d) = processed.into_iter().next() else {
        post_error(label, &format!("{label} returned no page"));
        return Err(());
      };
      let xml = d.to_xml_string();
      drop(d);
      if resident {
        // A changed page is re-parsed, in memory: the processors' edits reach
        // pass B in the reparse's normal form (merged text nodes, rescoped
        // namespaces), as from disk.
        let reparsed =
          PostDocument::new_from_string(&xml, PostDocument::file_options(&path_str, opts.clone()))
            .map_err(|e| {
              post_error("post", &format!("cannot re-read a staged page: {e}"));
            })?;
        page_meta.resident = Some(reparsed);
      } else if let Err(e) = std::fs::write(path, xml) {
        post_error("post", &format!("cannot re-stage a page to disk: {e}"));
        return Err(());
      }
    }
    Ok(())
  }

  if sweep_pages(
    &mut spilled_pages,
    &page_opts,
    &mut indexer,
    "MakeIndex",
    |p| p.needs_index,
  )
  .is_err()
  {
    return fallback();
  }
  let mut bibmaker = latexml_post::make_bibliography::MakeBibliography::new(indexer.db, false);
  telemetry::phase_enter(Phase::Bibliography);
  let t_bib = audit_start("MakeBibliography");
  let bib_result = sweep_pages(
    &mut spilled_pages,
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
  // MakeBibliography (the last post-phase that can read `STATE`, `VFS_KEY`, or
  // run a recursive `.bib` session) is finished. On large/split one-shot CLI
  // runs, release dead core TeX engine state + arena + malloc trim BEFORE
  // `parallel_render` sizes the worker fleet and before `CrossRef`/`MathML`/`XSLT`.
  if RELEASE_CORE_AFTER_BIB.with(|f| f.replace(false)) {
    crate::core_interface::release_core_state_for_post();
  }
  // ---- Parallel pass B: process-level page-range workers (design §6) ------
  //
  // Sized by `render_workers::render_jobs` (`LATEXML_RENDER_JOBS`, else
  // automatic in the `latexml_oxide` binary, else 1 = the serial path below),
  // and only past a minimum page count — below it the db save +
  // child spawn overhead beats the win. The ObjectDB is COMPLETE here (Scan,
  // MakeIndex and MakeBibliography have all run), so it is saved once as a
  // SQLite file and each worker attaches it readonly; the workers run the
  // identical `render_spilled_page` pipeline over contiguous page ranges and
  // the parent folds their diagnostics deterministically, in chunk order.
  // Every page's images are named before any page is rendered, in page order (as the pages were spilled, then those
  // MakeIndex and MakeBibliography rewrote), and stored in the resource cache (Perl's document cache,
  // `LaTeXML-oxide.cache`): the serial render and every render worker look the names up, so they are the same whichever
  // process renders a page, and CrossRef's copies of other pages' titles into a page's navigation meet the names of
  // the pages they come from (issue 995). One unsplit page names its images as it renders, as Perl does.
  if let Some(gfx) = graphics_proc.as_mut() {
    plan_rewritten_pages(&spilled_pages, &page_opts, gfx);
  }
  // The pages an abandoned fleet may have written: a page's Fatal in the serial render removes them
  // too, and a stop removes those it did not rewrite.
  let mut fleet_pages: Vec<String> = Vec::new();
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
      unicodemath,
      intent_literal,
      stylesheet: stylesheet.map(String::from),
      xslt_params: manifest_xslt_params,
      default_resources,
      searchpaths: manifest_searchpaths,
      svg,
      is_html_out,
      schemadocs,
      whatsout: whatsout.as_cli().to_string(),
      page_opts: (&page_opts).into(),
      pages: Vec::new(),
      // Filled in by `parallel_render` once the fleet is sized.
      max_rss_kb: 0,
      post_values: latexml_core::PostStateValues::capture(),
      resource_names: graphics_proc
        .as_ref()
        .map(latexml_post::graphics::Graphics::planned_names)
        .unwrap_or_default(),
      log_level: latexml_core::util::logger::stderr_level().to_string(),
    };
    // The names are stored: the workers open the cache file themselves, so this process holds no handle across.
    run_cache.close_files();
    let t_pages = audit_start("render_pages");
    let outcome = crate::render_workers::parallel_render(
      manifest,
      page_jobs,
      render_jobs,
      page_spill.path(),
      &bibmaker.db,
    );
    audit_end(t_pages);
    if let Some(result) = outcome.as_ref().filter(|r| r.abandoned) {
      fleet_pages.clone_from(&result.written);
    } else if let Some(result) = outcome {
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
      // A worker's post Fatal leaves no output, as the serial render's does (#782).
      if result.discarded {
        return String::new();
      }
      *written = result.written;
      return result.main_output.unwrap_or_else(fallback);
    }
    // Setup failed, or a worker failed to spawn or died without its report
    // (already reported): fall through to the serial render — the workers keep
    // the spilled pages on disk.
  }

  let crossref = build_crossref(bibmaker.db);
  let ctx = PageRenderCtx {
    page_opts: page_opts.clone(),
    is_html_out,
    schemadocs,
    whatsout,
    remove_spilled: true,
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
  let mut made_dirs = latexml_post::writer::MadeDirs::default();
  for SpilledPage {
    path,
    destination: dest,
    destination_directory: dest_dir,
    resident,
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
    let stop = latexml_core::stomach::check_timeout();
    #[cfg(feature = "test-utils")]
    let stop = stop.and_then(|()| serial_stop_hook(pages_rendered));
    if let Err(e) = stop {
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
      remove_fleet_pages_not_rewritten(&fleet_pages, written);
      break;
    }
    // Hand freed page memory back to the OS, exactly as core pass 1 does
    // (core_interface.rs, same rationale): every page cycles a full DOM +
    // XSLT result through GLIBC's heap (libxml2/libxslt allocate via libc,
    // not the Rust allocator), and glibc keeps freed mid-heap pages mapped —
    // the render loop's measured ~152 KB/page RSS growth on the 131 MB
    // witness is dominated by exactly this class, and it is what pushed the
    // single-invocation run over the memory fuse at page 56,088 of 115,519.
    // A trim walks the heap: it runs when RSS grew by a step, not per page count (`heap_trim`).
    if pages_rendered > 0 && pages_rendered.is_multiple_of(crate::heap_trim::SAMPLE_PAGES) {
      crate::heap_trim::sample();
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
      let (c_live_mb, c_free_mb) = crate::heap_trim::c_heap_mb();
      let rss_mb = latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024;
      eprintln!(
        "memdiag: pages={pages_rendered} RSS={rss_mb}MB C-live={c_live_mb}MB C-free={c_free_mb}MB"
      );
    }
    let rendered = match resident {
      Some(mut page) => {
        page.destination = dest;
        page.destination_directory = dest_dir;
        render_page(page, &mut procs, &ctx)
      },
      None => render_spilled_page(&path, &mut procs, &ctx, dest, dest_dir),
    };
    let outputs = match rendered {
      Ok(o) => o,
      Err(PageFailure::Fatal) => {
        remove_pages(written);
        remove_pages(&fleet_pages);
        written.clear();
        return String::new();
      },
      Err(PageFailure::Error) => {
        remove_fleet_pages_not_rewritten(&fleet_pages, written);
        return fallback();
      },
    };
    for (dest, output) in outputs {
      if let Some(path) = dest.as_deref() {
        let _ = made_dirs.ensure_parent(path);
        pages_rendered += 1;
        if let Err(e) = std::fs::write(path, &output) {
          post_error("write", &format!("failed to write page {path}: {e}"));
        }
        written.push(path.to_string());
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
  pub(crate) page_opts:      PostDocumentOptions,
  pub(crate) is_html_out:    bool,
  pub(crate) schemadocs:     bool,
  pub(crate) whatsout:       latexml_post::extract::Whatsout,
  /// Delete each spilled page once parsed. A render worker keeps its pages:
  /// should one fail, the serial render redoes the document from them.
  pub(crate) remove_spilled: bool,
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
) -> Result<Vec<(Option<String>, String)>, PageFailure> {
  let path_str = path.to_string_lossy().into_owned();
  let page = match PostDocument::new_from_file(&path_str, ctx.page_opts.clone()) {
    Ok(mut d) => {
      d.destination = destination;
      d.destination_directory = destination_directory;
      d
    },
    Err(e) => {
      post_error("post", &format!("cannot re-read a disk-staged page: {e}"));
      return Err(PageFailure::Error);
    },
  };
  // Free the spill as soon as it is parsed: for a 40k-page document the
  // spill area is the size of the core XML, and it need not outlive its page.
  if ctx.remove_spilled {
    let _ = std::fs::remove_file(path);
  }
  render_page(page, procs, ctx)
}

/// [`render_spilled_page`]'s pipeline after the parse: CrossRef → Graphics →
/// MathML/XSLT on one page, returning its `(destination, output)` pairs.
fn render_page(
  page: PostDocument,
  procs: &mut PageProcessors,
  ctx: &PageRenderCtx,
) -> Result<Vec<(Option<String>, String)>, PageFailure> {
  // CrossRef resolves this page's references out of the completed ObjectDB,
  // and its navigation/TOC work reads the DB rather than sibling pages — so
  // it is page-local once pass A has finished.
  telemetry::phase_enter(Phase::Crossref);
  let t_xref = audit_start("CrossRef");
  let xref = run_page_phase(page, &mut procs.crossref, "CrossRef");
  audit_end(t_xref);
  telemetry::phase_exit();
  let mut staged = xref.map_err(|()| PageFailure::Error)?;
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
      return Err(PageFailure::Error);
    }
    staged = next;
  }

  let rendered = match procs.post.process_chain(staged, &mut procs.processors) {
    Ok(r) => r,
    Err(e) => {
      emit_fatal("post", "convert", &format!("Post-processing failed: {e}"));
      return Err(PageFailure::Fatal);
    },
  };

  let mut outputs = Vec::with_capacity(rendered.len());
  for doc in rendered {
    let dest = doc.get_destination().map(String::from);
    let output = latexml_post::extract::serialize_whatsout(&doc, ctx.whatsout);
    let output = if ctx.is_html_out {
      finalize_html5(output)
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

/// Remove the pages a render wrote before a Fatal: Perl runs each processor over every page
/// before its writer, so a die in ProcessChain leaves none (LaTeXML.pm L605-614, #782).
pub(crate) fn remove_pages<S: AsRef<str>>(pages: &[S]) {
  for page in pages {
    let _ = std::fs::remove_file(page.as_ref());
  }
}

/// A serial render that stops after an abandoned fleet removes the fleet's pages it did not
/// rewrite: Info's count and `PostOutcome::pages` name only `written`, and a worker killed mid-write
/// may have left an empty page. The site is then the contiguous partial one a serial-only run leaves.
fn remove_fleet_pages_not_rewritten(fleet_pages: &[String], written: &[String]) {
  if fleet_pages.is_empty() {
    return;
  }
  let rewritten: std::collections::HashSet<&str> = written.iter().map(String::as_str).collect();
  for page in fleet_pages
    .iter()
    .filter(|p| !rewritten.contains(p.as_str()))
  {
    let _ = std::fs::remove_file(page);
  }
}

/// Test hook `LATEXML_RENDER_TEST_SERIAL_STOP=<n>` (guard: cluster_xslt_split): the serial render
/// stops before its page `n + 1`, as the memory fuse or the timeout would.
#[cfg(feature = "test-utils")]
fn serial_stop_hook(pages_rendered: usize) -> latexml_core::common::error::Result<()> {
  let at = std::env::var("LATEXML_RENDER_TEST_SERIAL_STOP").ok();
  if at.and_then(|n| n.parse().ok()) == Some(pages_rendered) {
    latexml_core::fatal!(Timeout, Convert, "Conversion timed out!");
  }
  Ok(())
}

/// Why a page did not render (already reported). `Fatal`: a MathML/XSLT step failed, which stops
/// post-processing with no output, as a die in Perl's ProcessChain does (LaTeXML.pm L605-614, #782).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageFailure {
  Error,
  Fatal,
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
  /// The page itself, never written to `path`: the one page of an unsplit, cleanly parsed document. Its DOM is
  /// what re-parsing `path` would give (a clean parse reparses to the same tree, and Scan only adds the root's
  /// `xml:id`), and [`as_reparsed`] gives it the wrapper state of a fresh parse.
  resident:              Option<PostDocument>,
}

/// Closes the run's cache when post ends, whichever way it returns.
struct CloseCache(latexml_post::document::DocCache);

impl Drop for CloseCache {
  fn drop(&mut self) { self.0.close(); }
}

/// Name the images MakeIndex and MakeBibliography may have added to the pages they rewrote (the pages were planned
/// when spilled), then store the names of every planned page ([`Graphics::commit_names`]). A page that cannot be
/// re-read is reported by its render.
///
/// [`Graphics::commit_names`]: latexml_post::graphics::Graphics::commit_names
fn plan_rewritten_pages(
  pages: &[SpilledPage],
  opts: &PostDocumentOptions,
  graphics: &mut latexml_post::graphics::Graphics,
) {
  if !graphics.planned() {
    return;
  }
  for page_meta in pages.iter().filter(|p| p.needs_index || p.needs_bib) {
    let Ok(mut page) = PostDocument::new_from_file(&page_meta.path.to_string_lossy(), opts.clone())
    else {
      continue;
    };
    page.destination.clone_from(&page_meta.destination);
    page
      .destination_directory
      .clone_from(&page_meta.destination_directory);
    graphics.plan_names(&page);
  }
  graphics.commit_names();
}

/// Whether libxml2 parsed `doc` without errors: a recovered tree (say, adjacent text nodes around a dropped
/// invalid character) can differ from its reparse, so such a page goes to disk.
fn parsed_cleanly(doc: &PostDocument) -> bool {
  let xml = doc.get_document();
  xml.is_well_formed() && xml.is_ns_valid()
}

/// `doc` as [`PostDocument::new_from_file`] would return it from `path`: the same DOM, with fresh ids,
/// namespaces, memos, node wrappers and `source` options.
fn as_reparsed(
  doc: PostDocument,
  path: &std::path::Path,
  opts: &PostDocumentOptions,
) -> PostDocument {
  let xml = doc.into_document();
  xml.trim_node_cache();
  let path_str = path.to_string_lossy();
  let page = PostDocument::new(xml, PostDocument::file_options(&path_str, opts.clone()));
  #[cfg(debug_assertions)]
  assert_matches_reparse(&page, &path_str, opts);
  page
}

/// Test-profile check that a resident page is what its disk spill would reparse to: Scan sets the root's
/// `xml:id` with a raw `set_attribute` (scan.rs), so a page that skipped the refresh keeps a stale id cache
/// and `source`. Every post fixture runs it; release builds pay nothing.
#[cfg(debug_assertions)]
fn assert_matches_reparse(page: &PostDocument, path: &str, opts: &PostDocumentOptions) {
  let reparsed = PostDocument::new_from_string(
    &page.to_xml_string(),
    PostDocument::file_options(path, opts.clone()),
  )
  .expect("a resident page reparses");
  let root_id_resolves = |d: &PostDocument| {
    let root = d.get_document().get_root_element()?;
    let id = latexml_post::document::get_xml_id(&root)?;
    Some(
      d.find_node_by_id(&id)
        .map(|n| n.node_ptr() == root.node_ptr()),
    )
  };
  let ns_decls = |d: &PostDocument| {
    let root = d.get_document().get_root_element();
    let decls = root
      .map(|r| r.get_namespace_declarations())
      .unwrap_or_default();
    decls
      .iter()
      .map(|ns| (ns.get_prefix(), ns.get_href()))
      .collect::<Vec<_>>()
  };
  assert_eq!(
    page.to_xml_string(),
    reparsed.to_xml_string(),
    "resident page: XML"
  );
  assert_eq!(
    root_id_resolves(page),
    root_id_resolves(&reparsed),
    "resident page: root xml:id"
  );
  assert_eq!(
    page.get_source(),
    reparsed.get_source(),
    "resident page: source"
  );
  assert_eq!(
    page.source_directory, reparsed.source_directory,
    "resident page: source_directory"
  );
  assert_eq!(
    page.processing_instructions(),
    reparsed.processing_instructions(),
    "resident page: PIs"
  );
  assert_eq!(
    page.searchpaths, reparsed.searchpaths,
    "resident page: searchpaths"
  );
  assert_eq!(
    ns_decls(page),
    ns_decls(&reparsed),
    "resident page: namespace declarations"
  );
  assert_eq!(
    page.namespaces, reparsed.namespaces,
    "resident page: namespaces"
  );
  assert_eq!(
    page.namespace_uris, reparsed.namespace_uris,
    "resident page: namespace_uris"
  );
}

/// Apply HTML5 cleanup (XML prolog strip, void-element fixes) to one rendered
/// page. Pulled out of `run_post_processing` so it can run on every split
/// sub-document, not just the first.
///
/// Pictures need no string-level work here any more: `latexml_post::svg::SVG`
/// converts every `<ltx:picture>` on the live page DOM before MathML/XSLT
/// (Perl's chain order, `LaTeXML.pm` Graphics → SVG → MathML → XSLT), so the
/// XSLT's `as-svg` branch renders the `<svg:svg>` itself. The former
/// regex-built fragment table and post-XSLT placeholder splice truncated an
/// outer picture at its first nested `</picture>` and dropped every
/// `<block>`/`<p>`/`<inline-block>` child (pagelayout ×3, ticket ×2,
/// vocaltract, bookcover-example2, latex-refsheet/header-graph lost all
/// picture text at post; batch 56bz).
fn finalize_html5(mut output: String) -> String {
  use std::{borrow::Cow, sync::LazyLock};
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
  let prolog_end = if output.starts_with("<?xml") {
    XML_PROLOG_RE.find(&output).map_or(0, |m| m.end())
  } else {
    0
  };
  let mut s: Cow<'_, str> = Cow::Borrowed(&output[prolog_end..]);
  if s.contains("/>")
    && let Cow::Owned(replaced) = NON_VOID_SELF_CLOSE_RE.replace_all(&s, "<$1$2></$1>")
  {
    s = Cow::Owned(replaced);
  }
  if s.contains("</")
    && let Cow::Owned(replaced) = VOID_CLOSE_RE.replace_all(&s, "")
  {
    s = Cow::Owned(replaced);
  }
  if s.contains("/>")
    && let Cow::Owned(replaced) = VOID_SELF_CLOSE_RE.replace_all(&s, "<$1$2>")
  {
    s = Cow::Owned(replaced);
  }
  match s {
    Cow::Owned(final_str) => final_str,
    Cow::Borrowed(_) => {
      if prolog_end > 0 {
        output.drain(..prolog_end);
      }
      output
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn finalize_html5_preserves_clean_html_and_fixes_xml_artifacts() {
    let clean = "<!DOCTYPE html><html><body><p>Hello</p><br></body></html>";
    assert_eq!(finalize_html5(clean.to_string()), clean);

    let with_prolog = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html><p>ok</p>";
    assert_eq!(
      finalize_html5(with_prolog.to_string()),
      "<!DOCTYPE html><p>ok</p>"
    );

    let with_fixes =
      "<?xml version=\"1.0\"?> <span class=\"x\"/><div/><br></br><img src=\"a.png\" />";
    assert_eq!(
      finalize_html5(with_fixes.to_string()),
      "<span class=\"x\"></span><div></div><br><img src=\"a.png\">"
    );
  }

  /// An unused reservation removes the file an earlier attempt wrote; a used one keeps it for post.
  #[test]
  fn an_unused_core_handoff_is_removed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("index.html");
    let dest = dest.to_str();
    let staged = StagedHandoff::at(dest);
    let path = staged.path().to_string();
    std::fs::write(&path, "<document/>\n").expect("written");
    let kept = used_core_handoff(Some(staged), Some(&path));
    assert_eq!(kept.as_ref().map(StagedHandoff::path), Some(path.as_str()));
    assert!(
      std::path::Path::new(&path).exists(),
      "a used handoff stays for post"
    );
    assert!(used_core_handoff(kept, None).is_none());
    assert!(
      !std::path::Path::new(&path).exists(),
      "an unused handoff is removed"
    );
  }
}
