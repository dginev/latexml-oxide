#![feature(alloc_error_hook)]

use std::{
  alloc::{Layout, set_alloc_error_hook},
  error::Error,
  fs::File,
  io::prelude::*,
  path::Path,
  process,
  rc::Rc,
};

use clap::Parser;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, DigestionMode, OutputFormat, error::emit_info};

/// Per-process allocator: mimalloc avoids glibc's arena-mutex contention
/// which dominates multi-process workloads (seen as 3.4x slowdown at 16 workers).
#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Heap-profiling allocator (`--features dhat-heap`): replaces mimalloc so dhat
/// can attribute every allocation to its call site. Diagnostic only.
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static GLOBAL: dhat::Alloc = dhat::Alloc;

/// LaTeXML-oxide: convert TeX/LaTeX documents to XML/HTML/MathML
#[derive(Parser, Debug)]
#[command(name = "latexml_oxide", version, about)]
struct Cli {
  /// The TeX/LaTeX source file(s) to convert (overridden by --source). A `.bib`,
  /// a `.zip`/`.tar.gz` archive, or a directory is auto-detected. Several files
  /// given side-by-side (`main.tex supplement.tex`) are converted independently
  /// and joined into one document, main first, the rest as appendices.
  #[arg(value_name = "SOURCE")]
  source_positional: Vec<String>,

  /// Output file (default: stdout). The extension can imply --format (e.g.
  /// .html → html5, .xml → xml, .zip → archive).
  #[arg(long, alias = "destination")]
  dest: Option<String>,

  /// Source file, overriding the positional SOURCE argument.
  #[arg(long)]
  source: Option<String>,

  /// Output format: html5, html, xhtml, xml, epub. Inferred from the --dest
  /// extension when omitted; falls back to xml.
  #[arg(long)]
  format: Option<String>,

  /// Shortcut for --format=xml: emit the raw LaTeXML XML with no HTML
  /// post-processing.
  #[arg(long)]
  xml: bool,

  /// Custom XSLT stylesheet path (overrides the format's built-in default).
  #[arg(long)]
  stylesheet: Option<String>,

  // === Post-processing flags ===
  /// Enable HTML/MathML post-processing (auto-enabled for HTML/ePub formats).
  #[arg(long, overrides_with = "nopost")]
  post: bool,

  /// Skip post-processing, emitting the raw LaTeXML XML even for an
  /// HTML-implying destination.
  #[arg(long, overrides_with = "post")]
  nopost: bool,

  /// Generate Presentation MathML (on by default for HTML formats).
  #[arg(long, alias = "presentationmathml", overrides_with = "nopmml")]
  pmml: bool,

  /// Suppress Presentation MathML even when the format would enable it.
  #[arg(long, alias = "nopresentationmathml", overrides_with = "pmml")]
  nopmml: bool,

  /// Generate Content MathML.
  #[arg(long, alias = "contentmathml", overrides_with = "nocmml")]
  cmml: bool,

  /// Suppress Content MathML.
  #[arg(long, alias = "nocontentmathml", overrides_with = "cmml")]
  nocmml: bool,

  /// Keep the intermediate XMath in the output alongside MathML.
  #[arg(long, alias = "xmath", overrides_with = "noxmath")]
  #[arg(name = "keepXMath")]
  keep_xmath: bool,

  /// Drop the XMath representation from the output.
  #[arg(long, alias = "nokeepXMath", overrides_with = "keepXMath")]
  noxmath: bool,

  /// Wrap MathML in a `<semantics>` element with the TeX source as annotation.
  #[arg(long, overrides_with = "nomathtex")]
  mathtex: bool,

  /// Suppress the TeX-source annotation on MathML.
  #[arg(long, overrides_with = "mathtex")]
  nomathtex: bool,

  /// Replace invisible-times operators (U+2062) with a zero-width space.
  #[arg(long, overrides_with = "invisibletimes")]
  noinvisibletimes: bool,

  /// Keep invisible-times operators (the default). If both are given, the last
  /// one on the command line wins (Perl GetOpt::Long).
  #[arg(long, overrides_with = "noinvisibletimes")]
  invisibletimes: bool,

  /// Remap styled alphanumerics to Unicode's Plane-1 Mathematical Alphanumeric
  /// Symbols (the default). If both --plane1 and --noplane1 are given, the last
  /// one on the command line wins (Perl GetOpt::Long).
  #[arg(long, overrides_with = "noplane1")]
  plane1: bool,

  /// Keep styled alphanumerics as ASCII and carry the style in a `mathvariant`
  /// attribute instead of remapping to Plane-1 codepoints, whose font coverage
  /// is patchy and which some screen readers announce poorly.
  #[arg(long, overrides_with = "plane1")]
  noplane1: bool,

  /// Remap to Plane-1 only for the variants whose doubly-styled blocks are
  /// worst supported (script, fraktur, double-struck), and to the simpler
  /// variant: `\mathbf{\mathcal{E}}` becomes the plain script codepoint rather
  /// than a bold-script one no font has. Implies --plane1.
  #[arg(long)]
  hackplane1: bool,

  /// Suppress the built-in CSS/JS resources.
  #[arg(long, overrides_with = "defaultresources")]
  nodefaultresources: bool,

  /// Include the built-in CSS/JS resources (the default). If both are given,
  /// the last one on the command line wins (Perl GetOpt::Long).
  #[arg(long, overrides_with = "nodefaultresources")]
  defaultresources: bool,

  /// Omit source comments from the output.
  #[arg(long, overrides_with = "comments")]
  nocomments: bool,

  /// Preserve source `%` comments in the output. This Rust port omits them by
  /// default (Perl keeps them). If both are given, the last one on the command
  /// line wins (Perl GetOpt::Long).
  // Divergence from Perl's default-on: see OXIDIZED_DESIGN #2.
  #[arg(long, overrides_with = "nocomments")]
  comments: bool,

  /// Strict mode: treat selected recoverable conditions as hard errors.
  // Perl Core.pm L43: State STRICT.
  #[arg(long)]
  strict: bool,

  /// Raw-load `.sty`/`.cls` sources from the search path instead of relying on
  /// LaTeXML's own bindings.
  ///
  /// WARNING: this enables raw TeX loading for BOTH packages (.sty) AND
  /// document classes (.cls) at once — a common source of errors, since raw
  /// class code is unlikely to convert cleanly.
  // Perl --includestyles / Core.pm L55-57: sets INCLUDE_STYLES + INCLUDE_CLASSES.
  #[arg(long)]
  includestyles: bool,

  /// Reuse an existing `.bbl` file instead of running BibTeX (for arXiv-style
  /// builds that ship their bibliography pre-compiled).
  #[arg(long)]
  nobibtex: bool,

  /// Process the input as a BibTeX `.bib` bibliography. Auto-detected when
  /// SOURCE ends in `.bib` or starts with `literal:@`.
  #[arg(long)]
  bibtex: bool,

  /// Disable math parsing (leave formulae as unparsed token lists).
  #[arg(long, alias = "noparse", overrides_with = "mathparse")]
  nomathparse: bool,

  /// Enable math parsing (the default). Restores it if a profile/package
  /// disabled it. If both are given, the last one on the command line wins
  /// (Perl GetOpt::Long).
  #[arg(long, overrides_with = "nomathparse")]
  mathparse: bool,

  /// Emit source locators: record each construct's source range as a
  /// `data-sourcepos` attribute, plus a document-level tag→file table. Off by
  /// default (a normal conversion pays nothing for it). Powers editor/preview
  /// sync and precise linting. Also enabled via `LATEXML_SOURCE_MAP=1`.
  // Issues #47/#92; see docs/performance/SOURCE_PROVENANCE.md.
  #[arg(long = "source-map")]
  source_map: bool,

  /// Disable section numbering.
  #[arg(long, alias = "nosectionnumbers", overrides_with = "numbersections")]
  nonumbersections: bool,

  /// Enable section numbering (the default). Restores it if a profile/package
  /// turned it off. If both are given, the last one on the command line wins
  /// (Perl GetOpt::Long).
  #[arg(long, overrides_with = "nonumbersections")]
  numbersections: bool,

  /// Vector-SVG fast path for PDF graphics. `0` (default) auto-detects: vector
  /// PDFs (no raster image, at most 500 KB) go through the SVG converters
  /// (mutool → pdftocairo); raster-bearing PDFs stay on the gs/convert path.
  /// `N > 0` forces the SVG path for any PDF at most N KB. Env
  /// `LATEXML_GRAPHICS_VECTOR_AUTO_OFF=1` disables auto-detect.
  #[arg(
    long = "graphics-svg-threshold-kb",
    value_name = "N",
    default_value = "0"
  )]
  graphics_svg_threshold_kb: u32,

  /// Convert `\includegraphics` figures to web images (the default). If both
  /// are given, the last one on the command line wins (Perl GetOpt::Long).
  #[arg(long, overrides_with = "nographicimages")]
  graphicimages: bool,

  /// Skip figure conversion: leave the raw `<ltx:graphics>` references in the
  /// output. Faster, and works on hosts without the image tools installed.
  #[arg(long, overrides_with = "graphicimages")]
  nographicimages: bool,

  /// What to emit: `document` (default; the full page), `fragment` (an
  /// embeddable inline snippet), `math` (just the math subtree), or `archive`
  /// (the page + resources zipped — also implied by a `.zip` --dest, and writes
  /// `<source-name>.zip` when --dest is omitted).
  #[arg(long, value_name = "TYPE")]
  whatsout: Option<String>,

  /// Shortcut for --whatsout=fragment: emit an embeddable inline snippet.
  #[arg(long)]
  embed: bool,

  // === Repeatable flags ===
  /// Add a CSS stylesheet link to the HTML output (repeatable).
  #[arg(long = "css", value_name = "URL")]
  css_files: Vec<String>,

  /// Add a JavaScript link to the HTML output (repeatable).
  #[arg(long = "javascript", value_name = "URL")]
  js_files: Vec<String>,

  /// Preload a package/module before processing, e.g. --preload=amsmath
  /// (repeatable).
  #[arg(long = "preload", value_name = "FILE")]
  preload_files: Vec<String>,

  /// Add a directory to the file/package search path, like TEXINPUTS
  /// (repeatable).
  #[arg(long = "path", value_name = "DIR")]
  search_paths: Vec<String>,

  // === Value flags ===
  /// Conversion timeout in seconds (default: 60). Use 0 to disable.
  #[arg(long, value_name = "SECONDS", default_value = "60")]
  timeout: u64,

  /// The RAM budget for this conversion, in MiB — the one memory knob.
  /// Defaults to the machine as it is right now: 90% of AVAILABLE RAM at
  /// startup, capped at 64 GiB (2048 floor; half of total RAM if
  /// availability cannot be probed, 6144 if nothing can be).
  ///
  /// Everything else follows from it. A conversion works within the budget,
  /// spilling completed parts of the document to disk as it approaches it, so
  /// a document far larger than RAM still converts (see --streaming). Nearing
  /// the budget anyway raises a graceful Fatal that keeps the partial output,
  /// and a hard watchdog aborts the process (exit 137) at the ceiling itself.
  ///
  /// Use 0 to lift the ceiling: nothing will abort the conversion for memory,
  /// but spilling still engages (derived from the machine) — "do not kill me"
  /// is not "let the machine run out". Also settable via the
  /// `LATEXML_MAX_MEMORY` env var; this flag wins when both are given.
  ///
  /// Left as `Option` so "unset" is distinguishable from an explicit value —
  /// the default has to be computed from the host, which clap's static
  /// `default_value` cannot express.
  #[arg(long, value_name = "MIB", env = "LATEXML_MAX_MEMORY")]
  max_memory: Option<u64>,

  /// Streaming (fragmented) conversion: digest and build interleave in
  /// bounded fragments, closed subtrees spill to disk beside the source, and
  /// a second, streaming pass finishes them — so peak memory is bounded by
  /// fragment size instead of document size. Output is byte-identical to the
  /// normal path (guarded by the 114_streaming_* sweep). Off by default;
  /// also AUTO-activates when the projected memory need of a large source
  /// exceeds the --max-memory ceiling, i.e. only where the normal path is
  /// certain to exhaust memory anyway.
  ///
  /// `--streaming=false` (or `LATEXML_STREAMING=false`) disables BOTH the flag
  /// and the auto-activation, forcing the eager path even for a document
  /// projected to exhaust the ceiling.
  #[arg(long, env = "LATEXML_STREAMING", num_args = 0..=1, default_missing_value = "true")]
  streaming: Option<bool>,

  /// Abort after processing this many tokens — guards against runaway macro
  /// expansion (default: 400M; env `LATEXML_TOKEN_LIMIT`, 0 disables).
  #[arg(long, value_name = "N")]
  token_limit: Option<usize>,

  /// Navigation table-of-contents style: context or none.
  #[arg(long, alias = "navtoc", value_name = "STYLE")]
  navigationtoc: Option<String>,

  /// Cross-reference URL style for the serving environment: `server` (strip a
  /// trailing `index.html`), `negotiated` (also strip the `.html` extension), or
  /// `file` (keep full paths; the default, best for local `file://` viewing).
  #[arg(long, value_name = "STYLE", value_parser = ["server", "negotiated", "file"])]
  urlstyle: Option<String>,

  /// Favicon for the generated site: emitted as `<link rel="icon">` and copied
  /// to the destination.
  #[arg(long, value_name = "FILE")]
  icon: Option<String>,

  /// Timestamp string embedded in the page (e.g. a build date in the footer);
  /// --timestamp=0 omits it. Omitted by default, for reproducible output.
  #[arg(long, value_name = "STRING")]
  timestamp: Option<String>,

  /// Apply scholarly-schema post-processing: kind chips on definitions,
  /// pretty-printed content models, and a per-module item index. Intended for
  /// the `tools/generate-scholarly-schema-docs` pipeline; harmless (no effect)
  /// on a generic document.
  #[arg(long)]
  schemadocs: bool,

  /// Write the conversion log to this file (default: stderr).
  #[arg(long, value_name = "PATH")]
  log: Option<String>,

  /// What the input is: `document` (default; a standalone file), `fragment` (a
  /// snippet wrapped with --preamble/--postamble or a standard pre/postamble,
  /// implied if either is given), `math` (a bare formula), `archive` (a `.zip`
  /// bundle, also implied by a `.zip` source), `directory` (a source dir, also
  /// implied by a trailing `/`), or `xml` (an already-converted LaTeXML core
  /// document to post-process directly — forces the XML-input path regardless of
  /// the file's extension; also implied by a `.xml`/`*-xml`/`*_xml` extension).
  #[arg(long, value_name = "TYPE")]
  whatsin: Option<String>,

  /// TeX file effectively prepended to the document (implies --whatsin=fragment).
  #[arg(long, value_name = "FILE")]
  preamble: Option<String>,

  /// TeX file effectively appended to the document (implies --whatsin=fragment).
  #[arg(long, value_name = "FILE")]
  postamble: Option<String>,

  /// Input encoding for decoding source bytes to UTF-8 (default: utf-8), e.g.
  /// iso-8859-1. Translates bytes only, not catcodes — use the inputenc
  /// package for those.
  #[arg(long, value_name = "ENC")]
  inputencoding: Option<String>,

  // === Split options ===
  /// Split the output into multiple linked pages (by section, by default). If
  /// both --split and --nosplit are given, the last one on the command line
  /// wins (Perl GetOpt::Long).
  #[arg(long, overrides_with = "nosplit")]
  split: bool,

  /// Force splitting off even when --splitat/--splitpath would enable it. If
  /// both --split and --nosplit are given, the last one on the command line
  /// wins (Perl GetOpt::Long).
  #[arg(long, overrides_with = "split")]
  nosplit: bool,

  /// Level to split at: part, chapter, section, subsection, ... (default:
  /// section).
  #[arg(long, value_name = "LEVEL")]
  splitat: Option<String>,

  /// Naming strategy for split files: id, idrelative, label, labelrelative
  #[arg(long, value_name = "STRATEGY")]
  splitnaming: Option<String>,

  /// XPath expression for split points (overrides --splitat)
  #[arg(long, value_name = "XPATH")]
  splitpath: Option<String>,

  // === Verbosity ===
  /// Increase output verbosity
  #[arg(short = 'v', long)]
  verbose: bool,

  /// Suppress most output
  #[arg(short = 'q', long)]
  quiet: bool,

  /// Enable a named debug feature, e.g. --debug frontmatter (repeatable).
  /// Implies debug-level logging.
  #[arg(long = "debug", value_name = "FEATURE")]
  debug: Vec<String>,

  /// Assign an ID to the document's root element.
  #[arg(long, value_name = "ID")]
  documentid: Option<String>,

  /// Root directory of the generated site; resource URLs are made relative to
  /// it (default: the destination's directory).
  #[arg(long, value_name = "DIR")]
  sitedirectory: Option<String>,

  /// Directory of the original source, searched for graphics and resources
  /// during post-processing (default: the source file's directory).
  #[arg(long, value_name = "DIR")]
  sourcedirectory: Option<String>,

  /// Additional XSLT parameters (repeatable, key=value)
  #[arg(long = "xsltparameter", value_name = "KEY=VALUE")]
  xslt_parameters: Vec<String>,

  // === Dev/internal flags ===
  /// Developer tool: process a format file and dump its compiled engine state.
  #[arg(long, value_name = "FILE")]
  init: Option<String>,

  /// Developer tool: generate Rust source from a dump file.
  #[arg(long, value_name = "DUMP")]
  codegen: Option<String>,

  /// Developer tool: dump the compiled schema model (.model text) to stdout
  /// and exit. Only the embedded LaTeXML schema is supported.
  #[arg(long)]
  dump_model: bool,

  /// Append a one-line JSON telemetry record for this job to this file (or set
  /// env `LATEXML_TELEMETRY_OUT`); batch runs accumulate a JSONL. Written only
  /// on a successful conversion.
  #[arg(long, value_name = "PATH")]
  telemetry_out: Option<String>,

  /// Run as a persistent JSON-RPC-over-stdio (LSP) server for editor/preview
  /// integration.
  #[arg(long)]
  server: bool,
}

/// Allocation-failure hook — emits a `Fatal:` line in the project's
/// logging convention so aggregation tooling records the failure, then
/// exits with code 137. See `cortex_worker.rs::custom_alloc_error_hook`
/// for full rationale + witness paper.
fn custom_alloc_error_hook(layout: Layout) {
  eprintln!(
    "Fatal:oom:alloc_failed allocation of {} bytes (align {}) failed; \
     likely runaway macro expansion (gullet pushback Vec growth past \
     worker memory budget). Exiting with code 137.",
    layout.size(),
    layout.align()
  );
  process::exit(137);
}

/// Read back which side of a `--opt`/`--noopt` boolean pair clap kept, or `None`
/// if neither flag was given.
///
/// The rightmost-wins resolution happens *before* this call, in **clap**: each
/// pair carries a mutual `overrides_with`, so clap discards every occurrence but
/// the last one on the command line and leaves at most one of the two `bool`s
/// set. That mirrors Perl `Common/Config.pm`, where each option assigns the same
/// accumulator scalar and the final assignment wins (`undef` until touched).
/// This function only takes that already-resolved state as clap decided it and
/// folds the two `bool`s into a tri-state: `Some(true)` if `pos` was the flag
/// clap kept, `Some(false)` if `neg` was, `None` if neither appeared — so callers
/// can fall back to the option's default via `.unwrap_or(default)`.
///
/// Because clap has already collapsed the pair, at most one input is ever `true`,
/// so the body is a plain precedence-disjunction — the `pos`-first ordering never
/// arbitrates. The `debug_assert!` pins that contract: if a future pair is wired
/// without its mutual `overrides_with`, both inputs can be `true` and this trips
/// (loudly, in the `every_negatable_pair_is_last_wins` test) rather than silently
/// letting `pos` win regardless of command-line order.
fn chosen_by_clap(pos: bool, neg: bool) -> Option<bool> {
  debug_assert!(
    !(pos && neg),
    "a --opt/--noopt pair reached chosen_by_clap with both set — its clap \
     `overrides_with` is missing, so rightmost-wins is not enforced"
  );
  pos.then_some(true).or(neg.then_some(false))
}

/// The `--opt`/`--noopt` flag pairs resolved once from the raw [`Cli`], mirroring
/// Perl `Common/Config.pm`'s `_prepare_options` (L369): every pair is collapsed to
/// its rightmost-wins value (via [`chosen_by_clap`]) and its static default applied
/// here, in **one place**, instead of re-deciding at each construction site.
///
/// Two options — `post` and `pmml` — carry a *format-dependent* default (post is
/// implied by an HTML destination/requested reps; pmml is defaulted on by the
/// format), so this keeps only the user's explicit tri-state choice for them and
/// leaves the default to be applied where the output format is known — exactly as
/// Perl resolves `format` first, then the post-format defaults, within the same
/// `_prepare_options`. Every other pair has a static default and is fully resolved.
struct ResolvedOptions {
  /// Core tri-states (`None` ⇒ leave the engine/state default untouched).
  include_comments:   Option<bool>,
  nomathparse:        Option<bool>,
  number_sections:    Option<bool>,
  /// Whether output splitting is enabled (Perl `split!` + `split*`-implies rule).
  split_enabled:      bool,
  /// User's explicit `--post`/`--nopost` choice; format-dependent default applied
  /// at the post-processing site.
  post:               Option<bool>,
  /// User's explicit `--pmml`/`--nopmml` choice; format-dependent default applied
  /// at the post-processing site.
  pmml:               Option<bool>,
  /// Post reps/flags with static defaults, fully resolved.
  cmml:               bool,
  keep_xmath:         bool,
  mathtex:            bool,
  noinvisibletimes:   bool,
  plane1:             bool,
  nodefaultresources: bool,
  graphicimages:      bool,
}

impl ResolvedOptions {
  /// Resolve all `--opt`/`--noopt` pairs from the parsed [`Cli`] — the single
  /// `_prepare_options`-shaped step. The `chosen_by_clap` folds live only here.
  fn from_cli(cli: &Cli) -> Self {
    ResolvedOptions {
      // Perl Core.pm L45 `comments!`; `noparse`/`parse` (mathparse); `numbersections!`
      // (default on). `None` leaves the setting unset (Rust comments default OFF —
      // OXIDIZED_DESIGN #2).
      include_comments:   chosen_by_clap(cli.comments, cli.nocomments),
      nomathparse:        chosen_by_clap(cli.nomathparse, cli.mathparse),
      number_sections:    chosen_by_clap(cli.numbersections, cli.nonumbersections),
      // Perl `Common/Config.pm` L124-130: `split!` is last-wins; the value options
      // `--splitat`/`--splitpath`/`--splitnaming` do `split = 1 unless defined split`,
      // i.e. they enable splitting ONLY when neither `--split` nor `--nosplit`
      // decided it (a split! always overwrites; a split* never overwrites a decided
      // value, so only the split! pair's order — which clap resolves — matters).
      split_enabled:      chosen_by_clap(cli.split, cli.nosplit)
        .unwrap_or(cli.splitat.is_some() || cli.splitnaming.is_some() || cli.splitpath.is_some()),
      post:               chosen_by_clap(cli.post, cli.nopost),
      pmml:               chosen_by_clap(cli.pmml, cli.nopmml),
      cmml:               chosen_by_clap(cli.cmml, cli.nocmml).unwrap_or(false),
      keep_xmath:         chosen_by_clap(cli.keep_xmath, cli.noxmath).unwrap_or(false),
      mathtex:            chosen_by_clap(cli.mathtex, cli.nomathtex).unwrap_or(false),
      noinvisibletimes:   chosen_by_clap(cli.noinvisibletimes, cli.invisibletimes).unwrap_or(false),
      // Perl `MathML.pm` L70: `--hackplane1` forces plane1 on; otherwise the
      // pair is last-wins over a default-on.
      plane1:             cli.hackplane1
        || chosen_by_clap(cli.plane1, cli.noplane1).unwrap_or(true),
      nodefaultresources: chosen_by_clap(cli.nodefaultresources, cli.defaultresources)
        .unwrap_or(false),
      graphicimages:      chosen_by_clap(cli.graphicimages, cli.nographicimages).unwrap_or(true),
    }
  }
}

/// Resolve the `--max-memory` ceiling in MiB.
///
/// `None` (flag and env both unset) means "derive it from this machine as it
/// is right now" — `min(64 GiB, 90 % of AVAILABLE RAM)`, portable via
/// `watchdog::default_ceiling_mib` (rule and fallbacks documented there). An
/// explicit value is honoured verbatim, including `0` for "no limit".
///
/// The old behaviour was a flat 6144 MiB regardless of hardware: on a 256 GB
/// host that refuses conversions which would fit comfortably, and on an 8 GB
/// laptop the machine starts swapping long before the guard fires.
fn resolve_max_memory(explicit: Option<u64>) -> u64 {
  match explicit {
    Some(mib) => mib,
    None => {
      let derived = latexml_core::watchdog::default_ceiling_mib();
      log::debug!(
        "--max-memory unset; derived {derived} MiB from this machine ({})",
        match (
          latexml_core::watchdog::available_memory_bytes(),
          latexml_core::watchdog::total_memory_bytes(),
        ) {
          (Some(a), Some(t)) => format!(
            "{} MiB available of {} MiB physical RAM",
            a / (1024 * 1024),
            t / (1024 * 1024)
          ),
          (None, Some(t)) => format!(
            "availability unknown, {} MiB physical RAM",
            t / (1024 * 1024)
          ),
          _ => "physical RAM unknown, using the fallback".to_string(),
        }
      );
      derived
    },
  }
}

/// Streaming activation (user policy 2026-07-29: flag + auto-when-doomed).
///
/// Forced by `--streaming`; otherwise auto-enabled only when the PROJECTED
/// peak of the eager path exceeds the memory ceiling — measured ~1.84 GB of
/// peak RSS per MB of math-heavy source on the 131 MB witness
/// (`docs/performance/STREAMING_CORE_DESIGN_2026-07-29.md` §1), i.e. only for
/// documents that today would die at the ceiling with certainty. An explicit
/// `--streaming=false` (or `LATEXML_STREAMING=false`) suppresses BOTH — the
/// escape hatch for a caller who would rather have the eager path's Fatal
/// than a fragmented conversion, and the only way to A/B the two paths on a
/// document large enough for auto to fire. The returned
/// budget is the pass-1 fragment yield threshold in BOXES: an eighth of the
/// ceiling at the measured ~2.4 KB per retained box. The bite must leave REAL
/// headroom under the RSS fuse (75% of the ceiling): a fragment's live cost
/// is boxes + the DOM built from them (~1.4×), yields only fire at legal
/// seams (a large alignment digests un-yieldingly past any knob), and
/// per-run bookkeeping (FragmentIndex, spine) creeps monotonically — on the
/// 131 MB witness at a 48 GB ceiling, a quarter-bite steadied at ~33 GB and
/// the ~5 GB creep then walked it into the 37.7 GB fuse.
fn resolve_streaming(requested: Option<bool>, max_memory_mib: u64, source: &str) -> Option<usize> {
  const PEAK_BYTES_PER_SOURCE_BYTE: u64 = 1900; // ~1.84 GB/MB, measured
  const BYTES_PER_BOX: u64 = 2416; // stomach::BYTES_PER_LIGHT_BOX's basis
  // `--max-memory=0` disables the death ceiling, but the machine is still
  // finite and such a document may still need to spill. Judge — and size
  // fragments — against the ceiling we WOULD have derived. Without this the
  // arithmetic degenerated: `projected > 0` always fired, and
  // `0 / 8 / 2416 -> max(1)` gave a ONE-BOX budget, i.e. a yield after every
  // box.
  let yardstick_mib = if max_memory_mib == 0 {
    latexml_core::watchdog::default_ceiling_mib()
  } else {
    max_memory_mib
  };
  // Compare against the cooperative FUSE, not the ceiling: death happens at
  // 75% of the ceiling, so comparing against the ceiling left a band
  // (0.75-1.0x) that was judged "fits", took the eager path, and was then
  // killed by the fuse — 8.1-10.8 MB of source on a 16 GB laptop.
  let fuse_mib = latexml_core::stomach::soft_cap_from_ceiling(yardstick_mib) / (1024 * 1024);
  let auto = || {
    let projected_mib =
      projected_source_bytes(source).saturating_mul(PEAK_BYTES_PER_SOURCE_BYTE) / (1024 * 1024);
    (projected_mib > fuse_mib).then_some(())
  };
  match requested {
    // Explicit opt-out: never stream, not even when projected to die.
    Some(false) => return None,
    Some(true) => {},
    None if auto().is_none() => return None,
    None => {},
  }
  // The eighth is MEASURED, not guessed, and shrinking it buys nothing: on the
  // 19.8 MB witness at an 8192 MiB ceiling, divisors 8/16/32 peak at
  // 4747/4719/4714 MB with an invariant ramp (3788/3784/3787 MB at fragment 2)
  // and byte-identical output. Peak there is a STARTUP TRANSIENT that this knob
  // does not size — see task #158.
  let budget_boxes = (yardstick_mib.saturating_mul(1024 * 1024) / 8 / BYTES_PER_BOX) as usize;
  Some(budget_boxes.max(1))
}

/// The byte size the memory projection must reason from: the DOCUMENT, not the
/// main file.
///
/// A 2 KB `index.tex` that `\input`s a thousand chapters is a half-gigabyte
/// document, but `metadata(main).len()` projects it at 2 KB — "fits easily" —
/// and the eager path then dies on it. When the main file actually names an
/// inclusion command, sum the source tree (`.tex`/`.ltx`/`.bbl`) instead.
///
/// Gated on the command being present so a SELF-CONTAINED paper sitting in a
/// directory of unused alternates (a common arXiv bundle shape) still projects
/// as itself and keeps the eager path.
///
/// Known limitation: an inclusion assembled by macro expansion
/// (`\myinput{ch1}`) names no literal command and is not detected; such a
/// document needs an explicit `--streaming`.
fn projected_source_bytes(source: &str) -> u64 {
  /// Enough of the main file to see its inclusion commands without reading a
  /// 131 MB self-contained source in full (whose own size already dominates).
  const SCAN_BYTES: u64 = 4 * 1024 * 1024;
  /// Backstop against a pathological tree (a home directory as source dir).
  const WALK_ENTRIES: usize = 50_000;
  const INCLUSION_COMMANDS: [&str; 6] = [
    "\\input",
    "\\include",
    "\\import",
    "\\subimport",
    "\\subfile",
    "\\includeonly",
  ];

  let own = std::fs::metadata(source).map(|m| m.len()).unwrap_or(0);
  let Some(dir) = Path::new(source).parent() else {
    return own;
  };
  let mut head = Vec::new();
  if File::open(source)
    .map(|f| f.take(SCAN_BYTES).read_to_end(&mut head))
    .is_err()
  {
    return own;
  }
  let head = String::from_utf8_lossy(&head);
  if !INCLUSION_COMMANDS.iter().any(|cmd| head.contains(cmd)) {
    return own;
  }
  let mut total = 0u64;
  let mut seen = 0usize;
  let mut stack = vec![dir.to_path_buf()];
  while let Some(d) = stack.pop() {
    let Ok(entries) = std::fs::read_dir(&d) else {
      continue;
    };
    for entry in entries.flatten() {
      seen += 1;
      if seen > WALK_ENTRIES {
        return total.max(own);
      }
      match entry.file_type() {
        Ok(t) if t.is_dir() => stack.push(entry.path()),
        Ok(t) if t.is_file() => {
          let path = entry.path();
          if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("tex" | "ltx" | "bbl")
          ) {
            total = total.saturating_add(entry.metadata().map(|m| m.len()).unwrap_or(0));
          }
        },
        _ => {},
      }
    }
  }
  total.max(own)
}

fn main() -> Result<(), Box<dyn Error>> {
  set_alloc_error_hook(custom_alloc_error_hook);

  // Hidden page-render worker mode (streaming-post design §6): a parent
  // conversion doing process-parallel page rendering re-invokes this very
  // binary with `LATEXML_RENDER_WORKER` pointing at a page-range manifest.
  // Dispatched before ANY CLI handling so the worker path can never drift
  // into a normal conversion; it exits with its own status protocol.
  if let Ok(manifest) = std::env::var("LATEXML_RENDER_WORKER") {
    process::exit(latexml::render_workers::worker_main(&manifest));
  }

  // Run all work on a worker thread with a 256 MB stack so deeply
  // nested math trees don't overflow the OS-default 8 MB main-thread
  // stack during finalize/post-processing. See cortex_worker.rs for
  // full rationale (sandbox 0711.4787 et al, #17).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(|| real_main().map_err(|e| e.to_string()))
    .expect("spawn worker thread")
    .join()
    .expect("worker thread panicked")
    .map_err(|s| s.into())
}

fn real_main() -> Result<(), Box<dyn Error>> {
  // Heap profiler (`--features dhat-heap`). Held for the whole conversion, which
  // runs on this (worker) thread. The success/fatal exits below go through
  // `process::exit`, which skips destructors, so the profile is flushed
  // explicitly via `_dhat.take()` before those exits (writing `dhat-heap.json`);
  // a normal `return` still drops it as a fallback.
  #[cfg(feature = "dhat-heap")]
  let mut _dhat = Some(dhat::Profiler::new_heap());

  let wall_start = std::time::Instant::now();
  // Set when the post-processing phase runs; drives the end-of-run combined
  // verdict (see the exit guard at the bottom of `main`).
  let mut post_ran = false;
  // The max of the per-phase status codes (core `ConversionResponse` and
  // `PostOutcome`). The live REPORT counter is *usually* identical — it is
  // shared across the phases — but a phase can carry a status FORCED outside
  // the counter (a `catch_unwind`-trapped panic maps to a fatal code without
  // a REPORT increment), so every final-status declaration folds
  // `max(REPORT, phase codes)`, exactly as cortex_worker does.
  let mut phase_status_max: usize = 0;
  let cli = Cli::parse();
  // Perl `_prepare_options`: collapse every `--opt`/`--noopt` pair to its
  // rightmost-wins value once, up front. The rest of `real_main` reads `resolved`
  // rather than re-deciding at each site.
  let resolved = ResolvedOptions::from_cli(&cli);

  // Kick off kpathsea pre-init in a background thread. Force-runs
  // `kpathsea_init_db` + per-format `kpathsea_init_format` so the
  // first real `find_file` from digest sees the fast post-init path
  // instead of paying ~30-40 ms of setup on its first lookup. The
  // worker briefly holds the `KPSE` Mutex while it probes — a main-
  // thread `kpsewhich(...)` racing in early would block briefly, but
  // dump load + arg parsing run for >50 ms before any digest-time
  // package resolution, so the warm-up usually completes before its
  // first real consumer arrives. Disable with
  // `LATEXML_NO_KPATHSEA_PREWARM=1` for A/B benchmarking.
  //
  // This is now purely a *latency* pre-warm (overlap the init cost with dump
  // load). The CORRECTNESS invariant — tables warm before the first `find_file`
  // — is enforced by the shared `Converter::initialize_session`, the single path
  // both this binary and the library (`latexml::api`, tests) funnel through, so
  // the library can no longer drift from the binaries the way it did (the flaky
  // spurious "1 warning" root-caused 2026-07-16).
  let _kpse_warmup_handle = if std::env::var("LATEXML_NO_KPATHSEA_PREWARM").is_err() {
    Some(std::thread::spawn(
      latexml_core::util::pathname::prewarm_kpathsea,
    ))
  } else {
    None
  };

  // Initialize logger with verbosity level
  let verbosity: i32 = if cli.quiet {
    -1
  } else if cli.verbose {
    1
  } else {
    0
  };
  let log_level = if !cli.debug.is_empty() {
    // --debug NAME implies debug-level logging (Perl: Debug() output is
    // emitted whenever the feature flag is set).
    log::LevelFilter::Debug
  } else {
    match verbosity {
      v if v < 0 => log::LevelFilter::Warn,
      0 => log::LevelFilter::Info,
      _ => log::LevelFilter::Debug,
    }
  };
  latexml_core::util::logger::init(log_level).ok();
  // Perl: --debug=NAME sets $LaTeXML::DEBUG{NAME}; gates DebugFeature! sites.
  for feature in &cli.debug {
    latexml_core::common::error::enable_debug_feature(feature);
  }

  // Dump-model mode — load the embedded LaTeXML schema, serialise to
  // stdout in `.model` format, exit. Mirrors Perl
  // `LaTeXML::Common::Model::compileSchema` (Model.pm L121-136). Used
  // by tools/compileschema.sh stage 2 to regenerate `LaTeXML.model`
  // from the same source the runtime sees.
  if cli.dump_model {
    print!("{}", latexml::dump_compiled_latexml_model());
    process::exit(0);
  }

  // Codegen mode — handle early, no source file needed
  if let Some(dump_path) = cli.codegen {
    let output = cli.dest.unwrap_or_else(|| "latex_dump.rs".to_string());
    match latexml::ini_tex::codegen_from_dump(&dump_path, &output) {
      Ok(count) => {
        eprintln!("Codegen complete: {} entries → {}", count, output);
        process::exit(0);
      },
      Err(e) => {
        eprintln!("Codegen failed: {}", e);
        process::exit(1);
      },
    }
  }

  // Persistent LSP Server mode — handle early before source file checks
  if cli.server {
    emit_info("lsp", "server", "Starting persistent LSP server...");
    let max_memory = resolve_max_memory(cli.max_memory);
    latexml::lsp_server::run_lsp_server(cli.timeout, max_memory)?;
    process::exit(0);
  }

  // Ordered supplementary top-level sources joined onto the main output
  // (populated by CLI multi-file input below, or by directory auto-detection).
  let mut supplement_sources: Vec<String> = Vec::new();
  // Determine source: --source > --init > positional. Multiple positional files
  // are an explicit multi-document submission — first is the main, the rest are
  // appended supplements (no archive/directory auto-detection in that case).
  let source = if let Some(ref init) = cli.init {
    init.clone()
  } else if let Some(ref src) = cli.source {
    src.clone()
  } else if cli.source_positional.is_empty() {
    eprintln!("Error: no source file specified. Use: latexml_oxide [OPTIONS] <SOURCE>");
    process::exit(1);
  } else {
    if cli.source_positional.len() > 1 {
      supplement_sources = cli.source_positional[1..].to_vec();
    }
    cli.source_positional[0].clone()
  };
  let target = cli.dest.clone();

  // Resolve `--whatsout <mode>` (Perl Pack.pm `whatsout` option +
  // Config.pm L421-439). Explicit `--whatsout` wins; otherwise a `.zip`
  // destination extension implies `archive` (Config.pm L421-426).
  // Unknown explicit values fall back to `document`, like Perl
  // `pack_collection`. Hoisted here (rather than inside the post block)
  // so both the post stage and the post-run `--log` guard can see it.
  let dest_ext_is_zip = target
    .as_deref()
    .is_some_and(|t| t.to_ascii_lowercase().ends_with(".zip"));
  let whatsout_mode = match cli.whatsout.as_deref() {
    Some(s) => latexml_post::extract::Whatsout::from_cli(s).unwrap_or_default(),
    None if dest_ext_is_zip => latexml_post::extract::Whatsout::Archive,
    // `--embed` is Perl's shortcut for `--whatsout=fragment` (Config.pm L72).
    None if cli.embed => latexml_post::extract::Whatsout::Fragment,
    None => latexml_post::extract::Whatsout::Document,
  };
  let is_archive_out = whatsout_mode.is_archive();

  // --whatsin=archive: extract archive to temp directory, find main .tex file
  let mut path_flags = cli.search_paths.clone();
  let _archive_tempdir; // hold tempdir alive for the duration of processing
  let is_archive_mode = cli.whatsin.as_deref() == Some("archive")
    || source.ends_with(".tar.gz")
    || source.ends_with(".tgz")
    || source.ends_with(".zip")
    || source.ends_with(".tar");
  let source = if is_archive_mode {
    let (tempdir, main_tex) = match unpack_archive(&source) {
      Ok(r) => r,
      Err(e) => {
        eprintln!("Failed to unpack archive '{}': {}", source, e);
        process::exit(1);
      },
    };
    let dir_str = tempdir.path().to_string_lossy().to_string();
    path_flags.push(dir_str);
    _archive_tempdir = Some(tempdir);
    main_tex
  } else {
    _archive_tempdir = None;
    source
  };

  // --whatsin=directory: auto-detect from trailing / or explicit flag
  let is_directory_mode = cli.whatsin.as_deref() == Some("directory") || source.ends_with('/');
  let source = if is_directory_mode {
    let dir_path = Path::new(&source);
    if let Ok(abs_source) = std::fs::canonicalize(dir_path) {
      path_flags.push(abs_source.to_string_lossy().to_string());
    } else {
      path_flags.push(source.clone());
    }
    // Find the ordered top-level file(s): the main first, then any detected
    // Supplementary-Material documents (joined onto the output below).
    match latexml::main_tex::find_top_level_texs(dir_path) {
      Ok(mut tops) => {
        let main_tex = tops.remove(0).to_string_lossy().to_string();
        for supp in tops {
          supplement_sources.push(supp.to_string_lossy().to_string());
        }
        main_tex
      },
      Err(e) => {
        eprintln!("Failed to find main .tex file in '{}': {}", source, e);
        process::exit(1);
      },
    }
  } else {
    source
  };

  // Perl latexmlc parity (bin/latexmlc L103-120): ALWAYS write a conversion
  // log — `--log` names it, otherwise `<jobname>.latexml.log` in the current
  // directory (literal/anonymous sources fall back to plain `latexml.log`),
  // and a pre-existing file is removed up front so the log is always this
  // run's. The archive output packs its log into the zip instead, exactly as
  // before. Reported missing on the 0.7.5-rc5 witness UAT (2026-08-03): a
  // fatal scrolled away with nothing on disk to consult.
  let cli_log: Option<String> = cli.log.clone().or_else(|| {
    // Dump-building (`--init`) is not a conversion; no default log there.
    if cli.init.is_some() {
      return None;
    }
    let name = if source.starts_with("literal:") {
      "latexml.log".to_string()
    } else {
      match Path::new(&source).file_stem().and_then(|s| s.to_str()) {
        Some(stem) => format!("{stem}.latexml.log"),
        None => "latexml.log".to_string(),
      }
    };
    Some(name)
  });
  if let Some(ref lp) = cli_log
    && Path::new(lp).is_file()
  {
    let _ = std::fs::remove_file(lp);
  }

  // Some arXiv source archives ship a PDF mis-named with a `.tex` extension
  // (e.g. 2301.04210.tex). Perl LaTeXML detects the `%PDF-` magic and bails
  // with a single Fatal; without this guard the binary catcode-tokenizes
  // the PDF stream and emits ~100 Error:undefined / Error:unexpected lines.
  if Path::new(&source).is_file() && latexml::main_tex::is_pdf_magic(Path::new(&source)) {
    eprintln!(
      "Fatal:invalid:not_tex_source PDF magic detected in source file '{}'",
      source
    );
    process::exit(1);
  }

  // Stash a copy of the resolved main-tex path for end-of-run telemetry,
  // since `source` itself is moved into `converter.convert(...)`.
  let telemetry_source = source.clone();

  // Prepare converter
  let preload = if cli.preload_files.is_empty() {
    None
  } else {
    Some(cli.preload_files.clone())
  };
  let search_paths = if path_flags.is_empty() {
    None
  } else {
    Some(path_flags)
  };

  // Perl `Common/Config.pm:24,216`: `$is_bibtex = qr/(^literal:\s*@)|(\.bib$)/`.
  // `--bibtex` forces the type; otherwise auto-detect when the
  // source ends in `.bib` or begins with `literal:@`.
  let is_literal_bib = {
    let trimmed = source.trim_start_matches("literal:");
    trimmed.trim_start().starts_with('@') && trimmed.len() < source.len()
  };
  let mode = if cli.bibtex || source.ends_with(".bib") || is_literal_bib {
    Some(DigestionMode::BibTeX)
  } else {
    None
  };

  // Map `--whatsin` to the core input-chunk size (Perl Config.pm
  // L399-404 + LaTeXML.pm:165-194). `archive`/`directory` have already
  // been resolved to a concrete main `.tex` above, so the core digests
  // them as a plain document; only `math`/`fragment` change the core's
  // preamble/postamble wrapping. When `--whatsin` is unset, a supplied
  // `--preamble`/`--postamble` implies a `fragment` input.
  let whatsin_size = match cli.whatsin.as_deref() {
    Some("math") => DataSize::Math,
    Some("fragment") => DataSize::Fragment,
    Some("document") | Some("archive") | Some("directory") => DataSize::Document,
    None if cli.preamble.is_some() || cli.postamble.is_some() => DataSize::Fragment,
    _ => DataSize::Document,
  };

  let opts = Config {
    verbosity,
    format: OutputFormat::HTML5,
    whatsin: whatsin_size,
    whatsout: DataSize::Document,
    preamble: cli.preamble.clone(),
    postamble: cli.postamble.clone(),
    mode,
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    preload,
    search_paths,
    // Perl Core.pm L45 INCLUDE_COMMENTS / L63-65 mathparse — resolved once in
    // `ResolvedOptions`. `None` leaves the setting unset (Rust comments default
    // OFF — OXIDIZED_DESIGN #2).
    include_comments: resolved.include_comments,
    // Perl Core.pm L43: STRICT; L55-57: INCLUDE_STYLES/INCLUDE_CLASSES.
    strict: if cli.strict { Some(true) } else { None },
    include_styles: if cli.includestyles { Some(true) } else { None },
    nomathparse: resolved.nomathparse,
    // `--source-map` flag OR `LATEXML_SOURCE_MAP` env enables locator
    // tracking + emission; otherwise leave unset (off). The env reads
    // once here, off the hot path. See `docs/performance/SOURCE_PROVENANCE.md`.
    source_map: if cli.source_map || std::env::var_os("LATEXML_SOURCE_MAP").is_some() {
      Some(true)
    } else {
      None
    },
    // Perl Config.pm L57 / Core.pm L60-61: --inputencoding seeds State
    // PERL_INPUT_ENCODING, which the Mouth reads to decode source bytes
    // (default utf-8 when unset).
    inputencoding: cli.inputencoding.clone(),
    streaming: resolve_streaming(cli.streaming, resolve_max_memory(cli.max_memory), &source),
  };
  // CRITICAL: must be set BEFORE `prepare_session`. `tex.rs` /
  // `latex.rs`'s LoadFormat split (plain_bootstrap → plain_dump|base
  // → plain_constructs and the latex equivalent) reads
  // `LATEXML_INI_MODE` to decide whether to fully load the format
  // or stop after the bootstrap pool. If it's not set yet,
  // `prepare_session` pre-loads `plain_base` / `latex_base`, which
  // pollutes the snapshot taken later in `ini_tex::dump_format` and
  // silences the diff for everything raw plain.tex / latex.ltx defines
  // (the `\countdef\allocationnumber=21` → `Stored::Register{...}`
  // problem from 2026-04-26).
  if cli.init.is_some() {
    // SAFETY: setting the var before any thread is spawned. `prepare_session`
    // and `ini_tex::dump_format` both read it but neither mutates env.
    unsafe {
      std::env::set_var("LATEXML_INI_MODE", "1");
    }
  }

  let mut converter = Converter::from_config(opts.clone());
  // `--whatsin=xml` forces the already-converted-XML input path regardless of the
  // file's extension (Perl-style: the input analog of `--format` for output),
  // covering a core document under a name `is_xml_input` doesn't recognise (#655).
  let xml_input = is_xml_input(&source) || cli.whatsin.as_deref() == Some("xml");
  // Skip engine init for already-converted XML input: post-processing is pure
  // libxml2 and never touches the TeX engine or its dump, so loading
  // TeX.pool/latex + the format dump (~85–160 ms and a chunk of RSS) is wasted
  // work. Init mode (`--init`) and every real TeX conversion still need it.
  if (cli.init.is_some() || !xml_input)
    && let Err(e) = converter.prepare_session(&opts)
  {
    eprintln!("Could not prepare converter session: {}", e);
    process::exit(1);
  }

  // Per-document state a fresh converter session resets — factored so the main
  // document and each joined supplement (see the multi-document branch below)
  // apply it identically. DOCUMENTID is intentionally excluded: it names the
  // main document's root, and supplements get their own prefixed id space.
  fn apply_document_state(nobibtex: bool, number_sections: Option<bool>) {
    if nobibtex {
      // BIB_CONFIG = ['bbl'] — skip BibTeX, use the pre-existing `.bbl` file.
      latexml_core::state::assign_value(
        "BIB_CONFIG",
        latexml_core::common::store::Stored::Strings(Rc::new([latexml_core::common::arena::pin(
          "bbl",
        )])),
        Some(latexml_core::state::Scope::Global),
      );
    }
    // Perl `numbersections!` (default on), last-wins: `Some(true)` numbers,
    // `Some(false)` suppresses, `None` leaves the setting untouched.
    if let Some(ns) = number_sections {
      latexml_core::state::assign_value(
        "no_number_sections",
        !ns,
        Some(latexml_core::state::Scope::Global),
      );
    }
  }
  apply_document_state(cli.nobibtex, resolved.number_sections);
  // Perl Core.pm L48: DOCUMENTID value
  if let Some(ref docid) = cli.documentid {
    latexml_core::state::assign_value(
      "DOCUMENTID",
      latexml_core::common::store::Stored::String(latexml_core::common::arena::pin(docid)),
      Some(latexml_core::state::Scope::Global),
    );
  }

  if cli.init.is_some() {
    // Init mode: process file and dump state
    match latexml::ini_tex::dump_format(&mut converter, &source, target.as_deref()) {
      Ok(count) => eprintln!("Format dump complete: {} entries written", count),
      Err(e) => {
        eprintln!("Format dump failed: {}", e);
        process::exit(1);
      },
    }
  } else {
    // Normal mode: convert document
    //
    // Two-layer timeout: the cooperative stomach::check_timeout gives a graceful
    // Err(Fatal) when the digestion loop can poll it, and the Watchdog forcibly
    // aborts the process if the deadline is reached without cooperation (e.g. a
    // tight native loop in Marpa / libxml2 / libxslt). The Watchdog cancels
    // automatically on drop at end of main. `--max-memory` rides the same
    // Watchdog (it was previously a silent no-op outside `--server`).
    // Resolve the ceiling ONCE: unset means "derive it from this machine"
    // (90 % of physical RAM, capped at 64 GiB), so both the hard Watchdog below
    // and the cooperative fuse agree on the same number.
    let max_memory = resolve_max_memory(cli.max_memory);
    let _watchdog =
      latexml_core::watchdog::Watchdog::with_limits(cli.timeout, max_memory.saturating_mul(1024));
    // `--max-memory` (or its `LATEXML_MAX_MEMORY` env) is the SINGLE memory
    // knob. The hard Watchdog ceiling above rides it directly; the cooperative
    // stomach RSS fuse is DERIVED from it (a fixed fraction below, leaving
    // post-processing headroom) rather than being an independent number — so
    // one flag governs one limit and `--max-memory=0` disables both. It also
    // WINS over `LATEXML_RSS_CAP_BYTES`: no env var may quietly override what
    // the user typed. That env still governs embedders which never parse CLI
    // flags and so never get here (the library test harness, the cortex_worker
    // fleet) — see `stomach::apply_memory_ceiling`.
    latexml_core::stomach::apply_memory_ceiling(max_memory);
    if max_memory == 0 {
      // Removing the surprise of one flag silently disabling two guards: say
      // so, out loud, when the whole memory limit is off.
      latexml_core::Warn!(
        "memory",
        "unlimited",
        "--max-memory=0: memory limiting disabled entirely (cooperative guard + hard watchdog); a runaway conversion may exhaust host RAM"
      );
    }
    if cli.timeout > 0 {
      latexml_core::stomach::set_timeout(cli.timeout);
    }
    if let Some(limit) = cli.token_limit {
      // 0 disables (as documented), matching the LATEXML_TOKEN_LIMIT env
      // convention (`Some(0) => None` at the gullet initializer). Passing a
      // literal `Some(0)` would instead fatal on the first token.
      latexml_core::gullet::set_token_limit((limit != 0).then_some(limit));
    }

    let source_for_post = source.clone();
    // XML-input mode: when the source is already-converted LaTeXML XML
    // (file extension `.xml`/`.xhtml` or content starts with `<?xml`
    // / `<document xmlns="…">`), skip the TeX → XML converter and feed
    // the file straight to post-processing. Mirrors what
    // `latexmlpost_oxide` did as a separate binary (per the
    // retirement plan in `docs/SYNC_STATUS.md`).
    let response = if xml_input {
      // Do NOT slurp the file into a String — a large already-converted
      // document (the reporter's index.xml is 614 MB) would sit resident on
      // top of the ~11× libxml2 DOM. Post-processing streams it from disk via
      // `PostDocument::new_from_file` (see the `run_post_processing_from_file*`
      // call below). We only need a non-empty `result` sentinel so the post
      // gate fires; the real input is the source path.
      latexml::converter::ConversionResponse {
        result:      Some(String::new()),
        log:         String::new(),
        status:      String::from("Status:conversion:0"),
        status_code: 0,
      }
    } else if supplement_sources.is_empty() {
      converter.convert(source)
    } else {
      // Multi-document submission (directory with detected supplements, or
      // several files given on the CLI): convert the main, then each supplement
      // in its own fresh session, and join them into ONE core document — main
      // first, each supplement an appendix titled by its own `\title`. In-memory
      // join (`latexml::multidoc`), which suits the common small
      // main+supplement case; the whole post pipeline downstream is unchanged.
      let main_resp = converter.convert(source);
      match main_resp.result.as_deref() {
        Some(main_xml) if !main_xml.is_empty() => {
          let mut supp_xmls: Vec<String> = Vec::new();
          let mut status_max = main_resp.status_code;
          for supp in &supplement_sources {
            let mut sconv = Converter::from_config(opts.clone());
            if sconv.prepare_session(&opts).is_err() {
              eprintln!(
                "Warning: could not prepare a session for supplement '{}'; skipping",
                supp
              );
              continue;
            }
            // A fresh session reset thread-local state — re-apply the shared
            // per-document flags before converting this supplement.
            apply_document_state(cli.nobibtex, resolved.number_sections);
            let r = sconv.convert(supp.clone());
            status_max = status_max.max(r.status_code);
            match r.result {
              Some(x) if !x.is_empty() => supp_xmls.push(x),
              _ => eprintln!(
                "Warning: supplement '{}' produced no output; skipping",
                supp
              ),
            }
          }
          match latexml::multidoc::join_core_documents(main_xml, &supp_xmls) {
            Ok(joined) => latexml::converter::ConversionResponse {
              result:      Some(joined),
              log:         main_resp.log,
              status:      main_resp.status,
              status_code: status_max,
            },
            Err(e) => {
              eprintln!("Warning: multi-document join failed: {e}; rendering main only");
              main_resp
            },
          }
        },
        _ => main_resp,
      }
    };
    let _ = &source_for_post; // keep alive for post-processing
    phase_status_max = phase_status_max.max(response.status_code);
    // Post-phase log (Graphics/MathML/XSLT) captured by
    // `run_post_processing_logged`; written after the core log into --log / the
    // archive log so BOTH conversion phases reach the persisted log (SYNC_STATUS
    // task 5; Perl LaTeXML.pm flushes once after convert_post). Declared out here
    // (not in the `Some(xml)` arm) so the post-if-let --log write can still see
    // it; stays empty when post-processing is skipped, keeping that --log
    // byte-identical to before.
    let mut post_log = String::new();
    if let Some(xml) = response.result {
      // Infer format from --dest extension if --format not specified (Perl Config.pm L408-441)
      let inferred_format: Option<String> = cli
        .format
        .clone()
        // `--xml` is Perl's shortcut for `--format=xml` (Config.pm L59).
        .or(if cli.xml {
          Some("xml".to_string())
        } else {
          None
        })
        .or_else(|| {
          target.as_ref().and_then(|dest| {
            Path::new(dest)
              .extension()
              .and_then(|ext| ext.to_str())
              .map(|ext| {
                match ext.to_lowercase().as_str() {
                  "html" | "htm" => "html5".to_string(), // Perl L435: html → html5
                  "xhtml" => "xhtml".to_string(),
                  "xml" => "xml".to_string(),
                  "zip" => "html5".to_string(), // Perl L431: zip → html5
                  "epub" | "mobi" => "epub".to_string(),
                  other => other.to_string(),
                }
              })
          })
        })
        // `--whatsout=archive` with no `--dest`/`--format` still wants a
        // web bundle — default it to html5, matching the `--dest *.zip`
        // inference above (a `.zip` dest already maps to html5).
        .or_else(|| {
          if is_archive_out {
            Some("html5".to_string())
          } else {
            None
          }
        });

      // Auto-select stylesheet from format (Perl Config.pm L543-551)
      // Shared with the library entrypoint via `post::default_stylesheet` so
      // the CLI and `latexml::api` never disagree on the per-format sheet.
      let effective_stylesheet = cli.stylesheet.clone().or_else(|| {
        latexml::post::default_stylesheet(inferred_format.as_deref()).map(String::from)
      });

      // Auto-enable post-processing when dest implies HTML (Perl Config.pm L448-455)
      let is_html_format = matches!(
        inferred_format.as_deref(),
        Some("html5") | Some("html") | Some("xhtml") | Some("epub") | Some("epub3")
      );
      // XML-input mode implies post-processing — there's nothing to
      // convert (the file is already converted XML), so the only
      // meaningful action is to run the post-pipeline on it.
      // Matches the always-on post-processing behaviour of the now-
      // retired `latexmlpost_oxide` binary.
      let xml_input_mode = xml_input;
      let split_enabled = resolved.split_enabled;
      // Perl `post!` is last-wins (resolved in `ResolvedOptions`); its
      // format-dependent default — post is implied by requested reps / an
      // HTML-ish destination — is applied here, where the format is known.
      let do_post = resolved.post.unwrap_or(
        resolved.pmml.unwrap_or(false)
          || resolved.cmml
          || effective_stylesheet.is_some()
          || is_html_format
          || split_enabled
          || xml_input_mode
          // Perl Config.pm L454: any non-`document` whatsout forces post.
          || whatsout_mode.requires_post(),
      );

      let split_xpath = if split_enabled {
        cli.splitpath.clone().or_else(|| {
          let splitat = cli.splitat.as_deref().unwrap_or("section");
          Some(make_splitpaths(splitat))
        })
      } else {
        None
      };

      if do_post {
        post_ran = true;
        // Perl LaTeXML.pm:429 passes opts{sourcedirectory} to Post as
        // `sourceDirectory`; when omitted, Post derives it from the source
        // filename (Post.pm:727-729). Mirror that: honour `--sourcedirectory`
        // if given, else fall back to the source file's own directory.
        let source_dir = cli.sourcedirectory.clone().unwrap_or_else(|| {
          Path::new(&source_for_post)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
        });

        // `--whatsout=archive` (or a `.zip` destination) bundles into a
        // zip. When `--dest` is omitted, Perl LaTeXML.pm:185-187 invents
        // a placeholder `<source-name>.zip`; mirror that so an archive
        // job always lands a file rather than dumping HTML to stdout.
        let zip_dest: Option<String> = if is_archive_out {
          Some(match &target {
            Some(t) if t.to_ascii_lowercase().ends_with(".zip") => t.clone(),
            Some(t) => format!(
              "{}.zip",
              t.trim_end_matches(".html").trim_end_matches(".xml")
            ),
            None => {
              let stem = Path::new(&source_for_post)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
              format!("{stem}.zip")
            },
          })
        } else {
          None
        };

        // For zip output, route graphics conversions through a TempDir so
        // the converted PNG/SVG files can be collected and bundled into
        // the output zip (mirroring `cortex_worker::pack_output_zip_with_resources`).
        // Without this, the Graphics post-processor wrote PNGs next to
        // `target` on the filesystem but the zip only carried HTML+log+status —
        // confirmed-bug 2026-05-18 on 1910.01256.
        let resource_tempdir: Option<tempfile::TempDir> = if is_archive_out {
          Some(tempfile::tempdir()?)
        } else {
          None
        };
        let dest_for_post: Option<String> = if let Some(tmp) = resource_tempdir.as_ref() {
          // Use a stable HTML filename derived from the zip stem so the
          // Graphics processor's relative paths resolve naturally.
          let stem = zip_dest
            .as_deref()
            .and_then(|z| Path::new(z).file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("document");
          Some(
            tmp
              .path()
              .join(format!("{stem}.html"))
              .to_string_lossy()
              .to_string(),
          )
        } else {
          target.clone()
        };

        // latexmlpost_oxide's default was "if no --pmml AND no
        // --stylesheet, default pmml = true". Apply the same rule for
        // XML-input mode so `latexml_oxide foo.xml --dest out.html`
        // does something useful out of the box. (`resolved.pmml` is `None` here
        // — else this default branch is not taken — so no `!cli.pmml` guard.)
        let default_pmml_for_xml_input = xml_input_mode && effective_stylesheet.is_none();
        let post_opts = PostOptions {
          // `--pmml`/`--nopmml` is last-wins (resolved in `ResolvedOptions`); when
          // neither is given, a rep may be defaulted on by the format/--post.
          pmml: resolved.pmml.unwrap_or(
            resolved.post.unwrap_or(false) || is_html_format || default_pmml_for_xml_input,
          ),
          cmml: resolved.cmml,
          keep_xmath: resolved.keep_xmath,
          stylesheet: effective_stylesheet.as_deref(),
          destination: dest_for_post.as_deref(),
          source_directory: Some(&source_dir),
          // Perl LaTeXML.pm:430 `siteDirectory`; None ⇒ Post defaults it to the
          // destination's directory (document.rs / Perl Config.pm L466-469).
          site_directory: cli.sitedirectory.as_deref(),
          search_paths: &cli.search_paths,
          nodefaultresources: resolved.nodefaultresources,
          css_files: &cli.css_files,
          js_files: &cli.js_files,
          noinvisibletimes: resolved.noinvisibletimes,
          mathtex: resolved.mathtex,
          plane1: resolved.plane1,
          hackplane1: cli.hackplane1,
          // Perl `--urlstyle` (Config.pm L482 defaults to `server`; we default
          // to `file` — no trailing-index stripping, safest for local `file://`
          // viewing — OXIDIZED_DESIGN #134). clap already restricted the value to
          // the three valid tags, so `from_cli` never returns None here.
          url_style: cli
            .urlstyle
            .as_deref()
            .and_then(latexml_post::crossref::UrlStyle::from_cli)
            .unwrap_or(latexml_post::crossref::UrlStyle::File),
          navigationtoc: cli.navigationtoc.as_deref(),
          schemadocs: cli.schemadocs,
          split: split_enabled,
          split_xpath,
          split_naming: cli.splitnaming.as_deref(),
          xslt_parameters: &cli.xslt_parameters,
          graphics_svg_threshold_kb: cli.graphics_svg_threshold_kb,
          graphicimages: resolved.graphicimages,
          // Perl `if ($timestamp)`: "0" (and empty) means "omit the timestamp".
          timestamp: cli
            .timestamp
            .as_deref()
            .filter(|t| !t.is_empty() && *t != "0"),
          icon: cli.icon.as_deref(),
          whatsout: whatsout_mode,
        };
        // XML-input mode parses the (possibly huge) source straight from disk
        // via the streaming file reader; TeX-conversion output is already in
        // memory as `xml`.
        let post = if xml_input_mode {
          latexml::post::run_post_processing_from_file_logged(&source_for_post, &post_opts)
        } else {
          latexml::post::run_post_processing_logged(&xml, &post_opts)
        };
        let output = post.html;
        phase_status_max = phase_status_max.max(post.status_code);
        // The canonical combined verdict, identical to cortex_worker's fold
        // (Perl LaTeXML.pm L631-634 `max(core, post)`): the framework at
        // ~/git/cortex derives a task's final severity from the LAST
        // `Status:conversion:N` in the log — and defaults to Fatal when the
        // line is missing — so every executable must end its log with the
        // combined line, and archive `status` members must carry the same
        // canonical string (this one carried the human-readable core-only
        // message instead).
        let combined_status_code =
          latexml_core::common::error::get_status_code().max(phase_status_max);
        let combined_status_line =
          latexml_core::common::error::conversion_status_line(combined_status_code);
        post_log = post.log;
        if let Some(zip_dest) = zip_dest {
          // whatsout=archive: pack the full document + resources into a ZIP.
          latexml_post::writer::ensure_parent_dir(&zip_dest)?;
          let resource_dir = resource_tempdir.as_ref().map(|t| t.path());
          let stem = Path::new(&zip_dest)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
          let html_name = format!("{stem}.html");
          let log_name = format!("{stem}.log");
          // Reproducible-build support: honour SOURCE_DATE_EPOCH for the
          // zip member timestamps (Perl Pack/Zip.pm L113-115).
          let source_date_epoch = std::env::var("SOURCE_DATE_EPOCH")
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok());
          // Emitted only after a memory-budget Fatal (`peak_memory_report`
          // is `None` otherwise): the measured peak rides just above the
          // status tail so the packed log records what the article needed —
          // see the identical placement in the `--log` file write below.
          let peak_log = latexml_core::watchdog::peak_memory_report()
            .map(|body| format!("Info:memory:peak {body}\n"))
            .unwrap_or_default();
          latexml_post::pack::pack_archive(&latexml_post::pack::PackOptions {
            zip_path: &zip_dest,
            html_filename: &html_name,
            html: &output,
            log_filename: Some(&log_name),
            log: &format!(
              "{}\n{}{}",
              assemble_conversion_log(&response.log, &post_log).trim_end(),
              peak_log,
              combined_status_line
            ),
            status: &combined_status_line,
            resource_dir,
            telemetry_json: None,
            source_date_epoch,
          })?;
          eprintln!("Output written to {}", zip_dest);
        } else {
          latexml_post::writer::write_output(&output, target.as_deref())?;
        }
        // resource_tempdir is dropped here (after pack_archive has copied
        // every file in), cleaning up the converted-PNG staging directory.
      } else {
        latexml_post::writer::write_output(&xml, target.as_deref())?;
      }
    }

    // --log: write conversion log to file (skip if already packed into
    // the ZIP by the archive output stage).
    if let Some(ref log_path) = cli_log
      && !is_archive_out
    {
      // Write the core log and the post-phase log sequentially rather than
      // concatenating — both are already-allocated and large for real
      // articles, so a merged `format!` would allocate a third copy of their
      // combined size on the conversion path.
      // Perl parity (and the cortex contract): the `Status:conversion:N`
      // line is the LAST line of the log, carrying the combined core+post
      // verdict — the REPORT counter is shared across both phases, so
      // reading it here (after post) IS the combined code.
      let status_line = format!(
        "\n{}\n",
        latexml_core::common::error::conversion_status_line(
          latexml_core::common::error::get_status_code().max(phase_status_max)
        )
      );
      // Emitted only after a memory-budget Fatal: the kernel-tracked peak,
      // recorded in the log as the honest lower bound on what THIS article
      // needs — a figure the user has no other way to learn.
      let peak_line = latexml_core::watchdog::peak_memory_report()
        .map(|body| format!("\nInfo:memory:peak {body}"))
        .unwrap_or_default();
      if post_log.is_empty() {
        latexml_post::writer::write_output_segments(
          &[
            response.log.trim_end_matches('\n'),
            &peak_line,
            &status_line,
          ],
          Some(log_path),
        )?;
      } else {
        latexml_post::writer::write_output_segments(
          &[
            response.log.trim_end_matches('\n'),
            "\n",
            post_log.trim_end_matches('\n'),
            &peak_line,
            &status_line,
          ],
          Some(log_path),
        )?;
      }
      eprintln!("Log written to {}", log_path);
    }
  }

  // Perl bin/latexml:151 — `if ($exit_message) { exit(1); }`: a Fatal
  // (status_code 3) conversion exits non-zero. cortex_worker already carries the
  // identical guard (`if final_status >= 3 { process::exit(...) }`); the standalone
  // CLI was missing it, so a 0-byte "complete" run (e.g. the plain-TeX
  // `$\displaylines{...}$` runaway that trips the memory-budget Fatal — shared with
  // Perl, which terminates at the same line) exited 0 and masqueraded as success.
  // Read the global status (thread-local REPORT, as cortex_worker does) — `response`
  // is scoped to the conversion branch. Match bin/latexml's exit(1) exactly;
  // status_code 2 ("errors but recoverable") stays a 0 exit, as in Perl.
  let final_status_code = latexml_core::common::error::get_status_code().max(phase_status_max);
  // The stderr copy of the peak-memory report — present only when a
  // memory-budget Fatal fired, so clean runs stay quiet. Info-level: `-q`
  // mutes it, the log file above keeps it regardless. Placed before the
  // verdict so the verdict stays the run's final word (101/119 guards).
  if let Some(body) = latexml_core::watchdog::peak_memory_report() {
    emit_info("memory", "peak", &body);
  }
  // The end-of-run verdict: the LAST line of a conversion names the COMBINED
  // core+post outcome. The core prints its own verdict when it finishes, but
  // on a post-processing run thousands of per-page lines follow it — a
  // mid-scroll "Conversion failed:" is exactly how a memory-Fatal'd partial
  // masqueraded as a successful site (131 MB witness UAT 2026-08-01:
  // truncated at Ch6, 1,572 plausible pages, silent exit 1). Also printed for
  // a non-post fatal, where repeating the verdict as the final line is cheap
  // emphasis. Guard: 119_final_status_report.
  if post_ran || final_status_code >= 3 {
    eprintln!(
      "{}",
      latexml_core::common::error::conversion_verdict(final_status_code)
    );
  }
  if final_status_code >= 3 {
    write_telemetry_record(
      cli.telemetry_out.as_deref(),
      &telemetry_source,
      wall_start,
      "fatal",
      final_status_code as i32,
    );
    #[cfg(feature = "dhat-heap")]
    drop(_dhat.take());
    process::exit(1);
  }
  write_telemetry_record(
    cli.telemetry_out.as_deref(),
    &telemetry_source,
    wall_start,
    "ok",
    0,
  );
  #[cfg(feature = "dhat-heap")]
  drop(_dhat.take());
  process::exit(0);
}

/// Emit a single-line JSON telemetry record. No-op when neither
/// `--telemetry-out` nor `LATEXML_TELEMETRY_OUT` is set. Errors writing
/// the file are swallowed (the conversion already succeeded; logging
/// the failure on stderr would be noise for batch runs).
fn write_telemetry_record(
  cli_path: Option<&str>,
  source: &str,
  wall_start: std::time::Instant,
  category: &str,
  exit_code: i32,
) {
  use latexml_core::telemetry;
  let path = cli_path
    .map(|s| s.to_string())
    .or_else(|| std::env::var("LATEXML_TELEMETRY_OUT").ok());
  let Some(path) = path else { return };

  // paper_id ≈ source basename without extension; cortex_worker
  // overrides this when it knows the arxiv id. Keep the binary's
  // best-effort default for direct CLI users.
  let paper_id = Path::new(source)
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("")
    .to_string();
  telemetry::set_paper_id(&paper_id);
  telemetry::set_cmdline(&std::env::args().collect::<Vec<_>>().join(" "));
  if let Ok(host) = std::env::var("HOSTNAME").or_else(|_| {
    // Linux fallback: read /etc/hostname
    std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string())
  }) {
    telemetry::set_host(&host);
  }
  telemetry::set_wall_us(wall_start.elapsed().as_micros() as u64);
  telemetry::set_category(category);
  telemetry::set_exit_code(exit_code);
  telemetry::set_max_rss_kb(read_max_rss_kb());
  let (cu, cs) = read_child_rusage_us();
  telemetry::set_child_rusage_us(cu, cs);

  let record = telemetry::take();
  let line = record.to_json_line();
  if let Some(parent) = Path::new(&path).parent()
    && !parent.as_os_str().is_empty()
  {
    let _ = std::fs::create_dir_all(parent);
  }
  // Append (JSONL): batch runs pointing LATEXML_TELEMETRY_OUT at one file
  // accumulate one record per job — the contract `perf_phase_summary.py`
  // documents. `File::create` here used to truncate, silently keeping only
  // the last job's record (2026-08-23 audit papercut).
  if let Ok(mut fh) = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&path)
  {
    let _ = writeln!(fh, "{line}");
  }
}

/// Read peak resident-set size from `/proc/self/status` (`VmHWM`).
/// Returns 0 on non-Linux or read failure.
fn read_max_rss_kb() -> u64 {
  std::fs::read_to_string("/proc/self/status")
    .ok()
    .and_then(|content| {
      content
        .lines()
        .find(|l| l.starts_with("VmHWM:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|n| n.parse::<u64>().ok())
    })
    .unwrap_or(0)
}

/// Read accumulated child user/sys CPU time in microseconds via getrusage(2).
/// Returns (0, 0) on failure or non-unix.
#[cfg(unix)]
fn read_child_rusage_us() -> (u64, u64) {
  // SAFETY: getrusage(RUSAGE_CHILDREN, &ru) is async-signal-safe and
  // populates the struct unconditionally on success.
  unsafe {
    let mut ru: libc::rusage = std::mem::zeroed();
    if libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru) == 0 {
      let user = (ru.ru_utime.tv_sec as u64) * 1_000_000 + (ru.ru_utime.tv_usec as u64);
      let sys = (ru.ru_stime.tv_sec as u64) * 1_000_000 + (ru.ru_stime.tv_usec as u64);
      (user, sys)
    } else {
      (0, 0)
    }
  }
}

#[cfg(not(unix))]
fn read_child_rusage_us() -> (u64, u64) { (0, 0) }

use latexml::post::PostOptions;

/// Assemble the persisted conversion log: core conversion log followed by the
/// captured post-phase log (Graphics/MathML/XSLT). Mirrors Perl `LaTeXML.pm`,
/// whose single `flush_log()` after `convert_post` returns both phases in one
/// buffer. `post_log` is empty when post-processing was skipped, in which case
/// the core log is returned unchanged (no behavioral drift for non-post runs).
fn assemble_conversion_log(core_log: &str, post_log: &str) -> String {
  if post_log.trim().is_empty() {
    core_log.to_string()
  } else {
    format!("{}\n{}", core_log.trim_end(), post_log.trim_end())
  }
}

// `ensure_parent_dir` now lives in `latexml_post::writer` so all
// post-processing binaries share one implementation. Perl analog:
// `LaTeXML::Post::Writer`.

fn make_splitpaths(splitat: &str) -> String {
  let ancestors: &[&str] = match splitat {
    "part" => &[],
    "chapter" => &["part"],
    "section" => &["part", "chapter"],
    "subsection" => &["part", "chapter", "section"],
    "subsubsection" => &["part", "chapter", "section", "subsection"],
    _ => &["part", "chapter"],
  };
  let back = ["bibliography", "appendix", "index"];
  let mut paths = Vec::new();
  let all_units: Vec<&str> = std::iter::once(splitat)
    .chain(ancestors.iter().copied())
    .collect();
  for unit in &all_units {
    paths.push(format!("//ltx:{}", unit));
    for b in &back {
      let mut conditions = vec![format!("preceding-sibling::ltx:{}", unit)];
      let unit_ancestors: &[&str] = match *unit {
        "part" => &[],
        "chapter" => &["part"],
        "section" => &["part", "chapter"],
        "subsection" => &["part", "chapter", "section"],
        "subsubsection" => &["part", "chapter", "section", "subsection"],
        _ => &[],
      };
      for anc in unit_ancestors {
        conditions.push(format!("parent::ltx:{}", anc));
      }
      paths.push(format!("//ltx:{}[{}]", b, conditions.join(" or ")));
    }
  }
  paths.join(" | ")
}

/// Unpack a ZIP (primary) or tar.gz archive into a temp directory.
/// Returns (TempDir, main_tex_path).
///
/// Port of Perl LaTeXML::Util::Pack::unpack_source.
/// Detect whether `source` is already-converted LaTeXML XML — i.e. a
/// `.xml` file — so the TeX → XML converter front-end can be skipped
/// and the file fed straight to post-processing. Matches what Perl
/// `latexmlpost` accepts and replaces the separate (now retired)
/// `latexmlpost_oxide` binary.
fn is_xml_input(source: &str) -> bool {
  Path::new(source)
    .extension()
    .and_then(|e| e.to_str())
    .is_some_and(|ext| {
      // `xml` itself, or any compound extension ending in `-xml`/`_xml`
      // (e.g. `.preprocessed-xml`, `.core_xml`) — an already-converted
      // LaTeXML core document under a project-specific name (#655). Force it
      // explicitly with `--whatsin=xml` for names outside this pattern.
      let ext = ext.to_ascii_lowercase();
      ext == "xml" || ext.ends_with("-xml") || ext.ends_with("_xml")
    })
}

fn unpack_archive(archive_path: &str) -> Result<(tempfile::TempDir, String), Box<dyn Error>> {
  let tempdir = tempfile::tempdir()?;
  let dest = tempdir.path();

  if archive_path.ends_with(".zip") {
    // Primary format: ZIP
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
      let mut entry = archive.by_index(i)?;
      let outpath = dest.join(entry.mangled_name());
      if entry.is_dir() {
        std::fs::create_dir_all(&outpath)?;
      } else {
        if let Some(parent) = outpath.parent() {
          std::fs::create_dir_all(parent)?;
        }
        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut entry, &mut outfile)?;
      }
    }
  } else if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
    let file = File::open(archive_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(dest)?;
  } else if archive_path.ends_with(".tar") {
    let file = File::open(archive_path)?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(dest)?;
  } else {
    return Err(format!("Unsupported archive format: {}", archive_path).into());
  }

  // Find main .tex file (Perl: LaTeXML::Util::Pack looks for the largest .tex file
  // or one containing \documentclass)
  let main_tex =
    latexml::main_tex::find_main_tex(dest).map_err(|e| -> Box<dyn Error> { e.into() })?;
  Ok((tempdir, main_tex.to_string_lossy().to_string()))
}

// Output-zip packing moved to `latexml_post::pack::pack_archive`
// (2026-05-18, audit follow-up for the latexml_oxide --post image-
// bundling fix). The previous inline `pack_output_zip` +
// `add_dir_to_zip` here and the parallel pair in `cortex_worker.rs`
// have been replaced by a single shared implementation — mirrors
// Perl `LaTeXML::Post::Pack`.

#[cfg(test)]
mod streaming_activation_tests {
  use std::io::Write;

  use super::*;

  fn write(dir: &Path, name: &str, bytes: usize) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut f = File::create(&path).expect("create fixture");
    f.write_all(&vec![b'x'; bytes]).expect("write fixture");
    path
  }

  /// A small main file that `\input`s its chapters is a BIG document: the
  /// estimate must be the tree, or auto-activation sends a half-gigabyte book
  /// down the eager path to its death.
  #[test]
  fn inclusion_fan_out_projects_the_whole_tree() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "ch1.tex", 400_000);
    write(tmp.path(), "ch2.tex", 400_000);
    let main = tmp.path().join("main.tex");
    std::fs::write(&main, "\\documentclass{book}\\input{ch1}\\input{ch2}\n").expect("write main");
    let projected = projected_source_bytes(main.to_str().unwrap());
    assert!(
      projected > 800_000,
      "expected the tree's ~800 KB, got {projected}"
    );
  }

  /// ...but a SELF-CONTAINED paper sitting in a directory of unused alternates
  /// (a common arXiv bundle shape) must still project as itself, or it would be
  /// pushed onto the fragmented path for no reason.
  #[test]
  fn self_contained_source_ignores_its_neighbours() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "old-version.tex", 900_000);
    let main = tmp.path().join("paper.tex");
    std::fs::write(
      &main,
      "\\documentclass{article}\\begin{document}x\\end{document}\n",
    )
    .expect("write main");
    let projected = projected_source_bytes(main.to_str().unwrap());
    assert!(
      projected < 100_000,
      "a self-contained paper must project as itself, got {projected}"
    );
  }

  /// `--max-memory=0` lifts the ceiling; it must not collapse the fragment
  /// budget to a single box (which spilled after every box) nor lose the
  /// watermark.
  #[test]
  fn zero_ceiling_keeps_a_workable_budget() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let main = write(tmp.path(), "src.tex", 1024);
    let budget = resolve_streaming(Some(true), 0, main.to_str().unwrap())
      .expect("forced streaming yields a budget");
    assert!(
      budget > 10_000,
      "a no-ceiling run must size fragments from the derived ceiling, got {budget} boxes"
    );
  }

  /// The explicit opt-out wins over auto-activation, however doomed the
  /// document looks — the only way to demand the eager path.
  #[test]
  fn explicit_false_defeats_auto_activation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let main = write(tmp.path(), "huge.tex", 4_000_000);
    // Tiny ceiling: the projection dwarfs it, so auto would certainly fire.
    assert!(resolve_streaming(None, 16, main.to_str().unwrap()).is_some());
    assert!(resolve_streaming(Some(false), 16, main.to_str().unwrap()).is_none());
  }

  /// Activation keys on the cooperative FUSE, not the ceiling: a document
  /// projected into the 0.75-1.0x band was judged "fits" and then killed by
  /// the fuse.
  #[test]
  fn activation_uses_the_fuse_not_the_ceiling() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // 1 MiB of source projects to ~1900 MiB. A 2048 MiB ceiling "fits" it,
    // but the fuse sits at 1536 MiB — which it does not.
    let main = write(tmp.path(), "band.tex", 1024 * 1024);
    assert!(
      resolve_streaming(None, 2048, main.to_str().unwrap()).is_some(),
      "a projection above the fuse must stream even when it is below the ceiling"
    );
    assert!(
      resolve_streaming(None, 8192, main.to_str().unwrap()).is_none(),
      "a projection comfortably under the fuse must stay eager"
    );
  }
}

/// GetOpt::Long-style last-wins parsing for `--opt`/`--noopt` boolean pairs
/// (issue #530). Perl reads options left-to-right and lets the rightmost win,
/// which lets a base flag list be overridden by appending more flags. clap's
/// mutual `overrides_with` on each pair reproduces that; these tests pin it.
#[cfg(test)]
mod boolean_flag_last_wins_tests {
  use super::*;

  /// Parse a CLI from a flag list, appending a dummy source so parsing succeeds.
  fn cli(flags: &[&str]) -> Cli {
    let mut argv = vec!["latexml_oxide"];
    argv.extend_from_slice(flags);
    argv.push("src.tex");
    Cli::parse_from(argv)
  }

  /// Resolved `split_enabled` for a flag list — exercises the real resolution path.
  fn split_on(flags: &[&str]) -> bool { ResolvedOptions::from_cli(&cli(flags)).split_enabled }

  #[test]
  fn chosen_by_clap_reads_back_the_resolved_pair() {
    assert_eq!(chosen_by_clap(true, false), Some(true));
    assert_eq!(chosen_by_clap(false, true), Some(false));
    assert_eq!(chosen_by_clap(false, false), None);
  }

  /// The reporter's core use case: appending the opposite flag overrides the
  /// earlier one, in either order and repeatedly — clap keeps only the last.
  #[test]
  fn split_pair_is_last_wins() {
    assert!(!split_on(&["--split", "--nosplit"]));
    assert!(split_on(&["--nosplit", "--split"]));
    // Appending once more flips it again — "global flags + file-specific flags".
    assert!(!split_on(&["--nosplit", "--split", "--nosplit"]));
    assert!(split_on(&["--split", "--nosplit", "--split"]));
  }

  /// Perl `Common/Config.pm` L124-130: a `--split*` value option turns splitting
  /// on only when neither `--split` nor `--nosplit` decided it (`unless defined`).
  #[test]
  fn split_value_options_imply_split_only_when_undecided() {
    // Undecided: any split* enables splitting.
    assert!(split_on(&["--splitat", "section"]));
    assert!(split_on(&["--splitpath", "//ltx:section"]));
    assert!(split_on(&["--splitnaming", "id"]));
    // An explicit --nosplit decides it; split* cannot re-enable, either order.
    assert!(!split_on(&["--nosplit", "--splitat", "section"]));
    assert!(!split_on(&["--splitat", "section", "--nosplit"]));
    // An explicit --split stays on.
    assert!(split_on(&["--split", "--splitat", "section"]));
    // Neither split! nor split*: off.
    assert!(!split_on(&[]));
  }

  /// Every negatable pair resolves to its rightmost occurrence. Guards each
  /// `overrides_with` wiring so a dropped one is caught.
  #[test]
  fn every_negatable_pair_is_last_wins() {
    let c = cli(&["--nopost", "--post"]);
    assert_eq!(chosen_by_clap(c.post, c.nopost), Some(true));
    let c = cli(&["--post", "--nopost"]);
    assert_eq!(chosen_by_clap(c.post, c.nopost), Some(false));

    let c = cli(&["--nopmml", "--pmml"]);
    assert_eq!(chosen_by_clap(c.pmml, c.nopmml), Some(true));
    let c = cli(&["--pmml", "--nopmml"]);
    assert_eq!(chosen_by_clap(c.pmml, c.nopmml), Some(false));

    let c = cli(&["--nocmml", "--cmml"]);
    assert_eq!(chosen_by_clap(c.cmml, c.nocmml), Some(true));

    let c = cli(&["--noxmath", "--keep-xmath"]);
    assert_eq!(chosen_by_clap(c.keep_xmath, c.noxmath), Some(true));

    let c = cli(&["--nomathtex", "--mathtex"]);
    assert_eq!(chosen_by_clap(c.mathtex, c.nomathtex), Some(true));

    let c = cli(&["--invisibletimes", "--noinvisibletimes"]);
    assert_eq!(
      chosen_by_clap(c.noinvisibletimes, c.invisibletimes),
      Some(true)
    );

    let c = cli(&["--plane1", "--noplane1"]);
    assert_eq!(chosen_by_clap(c.plane1, c.noplane1), Some(false));
    let c = cli(&["--noplane1", "--plane1"]);
    assert_eq!(chosen_by_clap(c.plane1, c.noplane1), Some(true));

    let c = cli(&["--defaultresources", "--nodefaultresources"]);
    assert_eq!(
      chosen_by_clap(c.nodefaultresources, c.defaultresources),
      Some(true)
    );

    let c = cli(&["--comments", "--nocomments"]);
    assert_eq!(chosen_by_clap(c.comments, c.nocomments), Some(false));
    let c = cli(&["--nocomments", "--comments"]);
    assert_eq!(chosen_by_clap(c.comments, c.nocomments), Some(true));

    let c = cli(&["--mathparse", "--nomathparse"]);
    assert_eq!(chosen_by_clap(c.nomathparse, c.mathparse), Some(true));

    let c = cli(&["--nonumbersections", "--numbersections"]);
    assert_eq!(
      chosen_by_clap(c.numbersections, c.nonumbersections),
      Some(true)
    );

    let c = cli(&["--graphicimages", "--nographicimages"]);
    assert_eq!(
      chosen_by_clap(c.graphicimages, c.nographicimages),
      Some(false)
    );
    let c = cli(&["--nographicimages", "--graphicimages"]);
    assert_eq!(
      chosen_by_clap(c.graphicimages, c.nographicimages),
      Some(true)
    );
  }

  /// The single `_prepare_options`-shaped resolve folds the whole `Cli` at once:
  /// static defaults for untouched pairs, rightmost-wins for the given ones.
  #[test]
  fn resolved_options_applies_defaults_and_last_wins() {
    // No flags: default-on options on, default-off off, tri-states unset.
    let r = ResolvedOptions::from_cli(&cli(&[]));
    assert!(r.plane1, "plane1 defaults ON");
    assert!(r.graphicimages, "graphicimages defaults ON");
    assert!(!r.cmml);
    assert!(!r.keep_xmath);
    assert!(!r.mathtex);
    assert!(!r.noinvisibletimes);
    assert!(!r.nodefaultresources);
    assert!(!r.split_enabled);
    assert_eq!(r.post, None);
    assert_eq!(r.pmml, None);
    assert_eq!(r.include_comments, None);
    assert_eq!(r.nomathparse, None);
    assert_eq!(r.number_sections, None);

    // Rightmost-wins flows through the resolve step.
    assert!(
      ResolvedOptions::from_cli(&cli(&["--nographicimages", "--graphicimages"])).graphicimages
    );
    assert!(!ResolvedOptions::from_cli(&cli(&["--plane1", "--noplane1"])).plane1);
    assert_eq!(
      ResolvedOptions::from_cli(&cli(&["--pmml", "--nopmml"])).pmml,
      Some(false)
    );
    assert!(ResolvedOptions::from_cli(&cli(&["--nosplit", "--split"])).split_enabled);

    // --hackplane1 forces plane1 on even against a later --noplane1 (Perl MathML.pm L70).
    assert!(ResolvedOptions::from_cli(&cli(&["--noplane1", "--hackplane1"])).plane1);
  }
}
