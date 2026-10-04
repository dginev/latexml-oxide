use std::rc::Rc;

use latexml_core::{
  Core, CoreOptions, Debug, Error, Fatal, Info, Note,
  common::{
    BindingDispatcher, BindingSource, Config, DataSize, DigestionMode, OutputFormat, arena,
    error::*, object::Object,
  },
  digested::Digested,
  document::Document,
  list::List,
  report_mut, s,
  state::{
    self, add_binding_names, set_bindings_dispatch, source_map_enabled, source_table_snapshot,
  },
  telemetry::{self, Phase},
};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::core_interface::DigestionAPI;

/// Where a runtime `.rhai` binding may be looked up — the two tiers differ in
/// *cost* and in *authority*, so they sit at opposite ends of the dispatch
/// chain (see [`install_binding_dispatch`]).
#[cfg(feature = "runtime-bindings")]
#[derive(Clone, Copy)]
enum RhaiScope {
  /// The local search paths only (source directory + `--path`) — a cheap
  /// `pathname::find`, no kpsewhich. Checked FIRST, so a `.rhai` a user put
  /// beside their document *overrides* a compiled binding of the same name.
  LocalPaths,
  /// Additionally the host TeX tree, via kpsewhich (`$TEXINPUTS`). Checked
  /// LAST, only once every compiled dispatcher has declined (#345).
  TeXTree,
}

/// Resolve a runtime `<request>.rhai` binding within `scope` and load it if
/// present. Returns `None` when no such file exists, so the caller falls
/// through to the next tier of the chain.
/// See `docs/parity/script_bindings_plan.md` §7.
#[cfg(feature = "runtime-bindings")]
fn rhai_dispatch(request: &str, scope: RhaiScope) -> Option<Result<BindingSource>> {
  use latexml_core::{
    binding::content::{FindFileOptions, find_file},
    state::record_opened_source,
  };
  let path = find_file(
    request,
    Some(FindFileOptions {
      // Append `.rhai`, so `foo.sty` resolves `foo.sty.rhai`.
      ext_type: Some("rhai".into()),
      // `TeXTree` lets the search fall through to kpsewhich, which is what
      // honours `$TEXINPUTS` — so a `<pkg>.sty.rhai` distributed in a texmf
      // tree is found by `\usepackage{pkg}` with no `--path` (#345). kpsewhich
      // locates it fine (the extension is irrelevant to a `//` recursive
      // search). This tier is deliberately NOT used for the first-priority
      // probe: that one runs on EVERY package/class request (64 of them on a
      // plain acmart paper), and a kpathsea miss is a directory-tree probe —
      // or a full fork-exec on the subprocess-`kpsewhich` backend. The memo
      // in `pathname::kpsewhich` is keyed by candidate name, so distinct
      // package names never share a hit.
      search_paths_only: matches!(scope, RhaiScope::LocalPaths),
      ..FindFileOptions::default()
    }),
  )?;
  // Pin the resolved `.rhai` in the opened-sources read-log. `load_file`
  // below reads it with a raw `std::fs::read_to_string` (it is not opened
  // through a `Mouth`, so `Mouth::create`'s `record_opened_source` never
  // fires for it). Without this, an edited binding is invisible to the warm
  // LSP preamble cache (`warmup_dep_snapshot` / `deps_still_current`) and the
  // stale macros survive every reconversion. Recording the resolved path lets
  // the cache invalidate on the file's mtime change.
  record_opened_source(arena::pin(&path));
  // #560: report the resolved on-disk path as the load's `BindingSource`, so
  // the "(Loading …)" note names the real `.rhai` file rather than the
  // synthesized `<name>_sty.rs` compiled-module proxy name — more useful, and
  // closer to Perl, which names the actual binding file.
  Some(latexml_contrib::script_bindings::load_file(&path).map(|_| Some(path)))
}

/// The DVI drivers a class option can name (graphics.cfg / the `*.def` drivers that write DVI specials).
const DVI_DRIVER_OPTIONS: [&str; 15] = [
  "dvips",
  "dvipdfm",
  "dvipdfmx",
  "dvisvgm",
  "xdvi",
  "dvipsone",
  "dviwindo",
  "emtex",
  "dvitops",
  "dvitoln03",
  "pctexps",
  "pctexwin",
  "pctexhp",
  "pctex32",
  "truetex",
];

/// `text` as TeX reads its code: each line (ended by CR, LF or CRLF, as the Mouth splits them) up to its first `%`
/// that an even run of backslashes precedes (`\%` is a character, `\\%` a control symbol then a comment), without the
/// files its `filecontents` environments write (a shipped `.sty` ends with its own `\endinput`), up to the first
/// `\endinput` control word (TeX reads no further of the file).
fn uncommented_code(text: &str) -> String {
  static FILECONTENTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)\\begin\s*\{\s*filecontents\*?\s*\}.*?\\end\s*\{\s*filecontents\*?\s*\}")
      .unwrap()
  });
  static ENDINPUT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\endinput(?:[^A-Za-z@]|$)").unwrap());
  let mut code = String::with_capacity(text.len());
  for line in text.split(['\n', '\r']) {
    let bytes = line.as_bytes();
    let mut backslashes = 0;
    let mut end = bytes.len();
    for (i, &b) in bytes.iter().enumerate() {
      match b {
        b'\\' => backslashes += 1,
        b'%' if backslashes % 2 == 0 => {
          end = i;
          break;
        },
        _ => backslashes = 0,
      }
    }
    code.push_str(&line[..end]);
    code.push('\n');
  }
  let mut code = FILECONTENTS.replace_all(&code, "").into_owned();
  if let Some(at) = ENDINPUT.find(&code) {
    code.truncate(at.start());
  }
  code
}

/// Whether `code` names a LaTeX document's cue: `\documentclass`, `\documentstyle` or `\begin{document}`, or a
/// command plain TeX cannot run (`\usepackage`, `\RequirePackage`, `\include`).
fn has_latex_cue(code: &str) -> bool {
  static LATEX_CUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
      r"\\document(?:class|style)|\\begin\s*\{\s*document\s*\}|\\(?:usepackage|RequirePackage|include)(?:[^A-Za-z@]|$)",
    )
    .unwrap()
  });
  LATEX_CUE.is_match(code)
}

/// The text of `path.tex` or, failing that, of `path` (TeX tries the `.tex` name first): a regular file of at most
/// 4 MB, so a device or a FIFO named by an `\input` is never read.
fn read_source_file(path: &std::path::Path) -> Option<String> {
  const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
  [
    std::path::PathBuf::from(format!("{}.tex", path.display())),
    path.to_path_buf(),
  ]
  .into_iter()
  .find(|candidate| {
    std::fs::metadata(candidate).is_ok_and(|m| m.is_file() && m.len() <= MAX_SOURCE_BYTES)
  })
  .and_then(|candidate| std::fs::read(candidate).ok())
  .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// How arXiv's AutoTeX compiles a main file: with plain TeX or LaTeX (which format a session preloads), and to DVI
/// (latex+dvips) or PDF (`\pdfoutput`, `core_interface::establish_pdf_output_mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompileRoute {
  /// A plain TeX document: latexml.sty's preload must not load the LaTeX format for it.
  pub plain_tex: bool,
  /// A latex+dvips document by its class options.
  pub dvi:       bool,
}

/// The [`CompileRoute`] of the main file `source` (a path, `.tex` resolved as the Mouth does, or `literal:` content).
/// A LaTeX document names `\documentclass` or `\documentstyle` outside a comment
/// (AutoTeX's tex-vs-latex test) or `\begin{document}` (a main file `\input`ting its preamble), in the main file or,
/// when it names none, in a file it `\input`s or `\include`s (a wrapper naming its preamble and body). AMSTeX's own
/// `\documentstyle{amsppt}` (after `\input amstex`) still counts as LaTeX, though Perl reads it plain: kept plain, 64
/// sampled 1996-99 amsppt papers went 192 → 319 errors (math9806005 0 → TooManyErrors), AMSTeX's plain-mode gaps. The
/// first `\documentclass`, its options naming a DVI driver, in a file setting no `\pdfoutput=1`, makes it a latex+dvips one:
/// 0908.4150's `\documentclass[12pt,dvips]{article}` (stopped full-arXiv run 329), which pdflatex stops on with
/// l3backend's "Backend request inconsistent with engine" for `dvips`, `dvipdfmx` and `dvisvgm` (expl3-code.tex's
/// backend options) and whose specials it drops for the other graphics drivers. An unreadable source counts as
/// LaTeX, PDF.
fn compile_route(source: &str) -> CompileRoute {
  static PDFOUTPUT_ONE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\pdfoutput\s*=?\s*1(?:[^0-9]|$)").unwrap());
  static INPUT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\\(?:input|include)(?:\s*\{\s*([^}]+?)\s*\}|\s+([^\s{}\\]+))").unwrap()
  });
  static IFFALSE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)\\iffalse\b.*?\\(?:else|fi)\b").unwrap());
  // A followed file's class cue must be one in use, not a dual-mode test (`\ifx\documentclass\undefined`).
  static CLASS_IN_USE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\\document(?:class|style)\s*[\[{]|\\begin\s*\{\s*document\s*\}").unwrap()
  });
  const MAX_INPUT_DEPTH: usize = 3;
  let (text, dir) = match source.strip_prefix("literal:") {
    Some(content) => (content.to_string(), None),
    None => {
      let path = std::path::Path::new(source);
      match read_source_file(path) {
        Some(text) => (text, path.parent().map(std::path::Path::to_path_buf)),
        None => return CompileRoute::default(),
      }
    },
  };
  let mut code = uncommented_code(&text);
  let mut latex = has_latex_cue(&code);
  // A main file naming no class: the files it inputs (not in `\iffalse … \fi`), breadth first, a few levels deep,
  // until one names the class; each using a LaTeX cue joins the main file's code for the DVI test.
  if !code.contains("\\documentclass")
    && let Some(dir) = dir
  {
    let mut visited = std::collections::HashSet::new();
    let mut level = vec![IFFALSE.replace_all(&code, "").into_owned()];
    'levels: for _ in 0..MAX_INPUT_DEPTH {
      let mut next_level = Vec::new();
      for parent in &level {
        for captures in INPUT.captures_iter(parent) {
          let Some(name) = captures.get(1).or_else(|| captures.get(2)) else {
            continue;
          };
          // A name only TeX can resolve (`\input{\pre}`) may be the preamble: lean to LaTeX, the previous default.
          if name.as_str().contains(['\\', '#']) {
            latex = true;
            continue;
          }
          if !visited.insert(name.as_str().to_string()) {
            continue;
          }
          let Some(input) = read_source_file(&dir.join(name.as_str())) else {
            continue;
          };
          let input_code = IFFALSE
            .replace_all(&uncommented_code(&input), "")
            .into_owned();
          if CLASS_IN_USE.is_match(&input_code) {
            latex = true;
            code = format!("{code}\n{input_code}");
            if input_code.contains("\\documentclass") {
              break 'levels;
            }
          }
          next_level.push(input_code);
        }
      }
      level = next_level;
    }
  }
  let dvi = !PDFOUTPUT_ONE.is_match(&code)
    && code.find("\\documentclass").is_some_and(|at| {
      let rest = code[at + "\\documentclass".len()..].trim_start();
      rest
        .strip_prefix('[')
        .and_then(|r| r.split_once(']'))
        .is_some_and(|(options, _)| {
          options
            .split(',')
            .any(|o| DVI_DRIVER_OPTIONS.contains(&o.trim()))
        })
    });
  CompileRoute { plain_tex: !latex, dvi }
}

/// Install the binding-resolution **priority chain** as the single dispatcher
/// and register the `latexml_package` binding-name registry.
///
/// This is the one definition of binding-resolution policy, shared by
/// [`Converter::initialize_session`] and the integration-test harness
/// (`util::test::process_texfile`) — so a test exercises the *same* chain a real
/// conversion does, and local `.rhai` fixtures are discovered identically.
///
/// Priority, highest first (installed in ONE slot, so call-site ordering can't
/// reshuffle it):
///   1. a local `<request>.rhai` in the search paths — lets a user (or a test
///      fixture next to its `.tex`) OVERRIDE any compiled binding of the same
///      name (e.g. `article.cls.rhai` shadows `article_cls`). `runtime-bindings`
///      only.
///   2. the `extra` dispatcher (`latexml_contrib` for our binaries) — consulted
///      before `latexml_package` to preserve the prior external-before-internal
///      order; the two registries are disjoint, so the order is immaterial.
///      Compiled `.rs` bindings keep precedence even under the raw-
///      interpretation preloads (`rawstyles`/`rawclasses`) — user directive
///      2026-08-31 (perfect-kernel mission): raw mode governs what happens for
///      names with NO binding (raw-load instead of the OmniBus fallback), it
///      never demotes an existing binding.
///   3. `latexml_package` — core compiled engine bindings.
///   4. a `<request>.rhai` on the host TeX tree (`$TEXINPUTS`, via kpsewhich) —
///      a binding *distributed* with a package, so it FILLS A GAP rather than
///      overriding: `\usepackage{X}` finds `X.sty.rhai` in a texmf tree with no
///      `--path` (#345), while a stray `amsmath.sty.rhai` left on that tree
///      cannot silently displace the compiled `amsmath` binding. Last, so the
///      kpathsea probe is paid only by requests nothing else could answer.
///      `runtime-bindings` only.
pub(crate) fn install_binding_dispatch(extra: Option<BindingDispatcher>) {
  set_bindings_dispatch(Rc::new(move |request: &str| {
    #[cfg(feature = "runtime-bindings")]
    if let Some(result) = rhai_dispatch(request, RhaiScope::LocalPaths) {
      return Some(result);
    }
    // The compiled tiers (contrib `extra`, then `latexml_package`) load
    // in-memory bindings with no source file, so they report `None` source.
    if let Some(extra) = extra.as_ref()
      && let Some(result) = extra(request)
    {
      return Some(result.map(|()| None));
    }
    if let Some(result) = latexml_package::dispatch(request) {
      return Some(result.map(|()| None));
    }
    #[cfg(feature = "runtime-bindings")]
    if let Some(result) = rhai_dispatch(request, RhaiScope::TeXTree) {
      return Some(result);
    }
    None
  }));
  // Register every (name, ext) binding pair so `find_file(notex=true)` can
  // resolve compile-time bindings across all extensions
  // (.cls/.sty/.def/.pool/code.tex/...). This also feeds `load_class`'s
  // Perl-parity prefix-match fallback (Package.pm L2702-2706) via the
  // class-filtered `state::get_class_binding_names()` view. Source of
  // truth: `latexml_package::BINDINGS`.
  add_binding_names(latexml_package::binding_names());
}

/// Everything a fresh `Core` needs before its first `input_definitions`:
/// the resolution chain above plus contrib's (name, ext) pairs. `Core::new`
/// resets the State, which starts with NO dispatch, so every site that
/// constructs an engine and loads pools must call this — the converter's
/// `initialize_session` and the post-only recursive bibliography session
/// (`bib_session::ensure_session`); omitting it there once left `TeX.pool`
/// "missing" and every post-only bibliography empty.
///
/// contrib (memoir / siamltex / scrbook / etc.) is registered unconditionally
/// because the canonical setup for both `latexml_oxide` and `cortex_worker`
/// loads it — downstream embedders that replace the dispatchers can register
/// their own (name, ext) slice the same way via `add_binding_names`.
pub(crate) fn install_default_binding_chain(extra: Option<BindingDispatcher>) {
  install_binding_dispatch(extra);
  add_binding_names(latexml_contrib::binding_names());
}

pub struct ConversionResponse {
  pub result:      Option<String>,
  pub log:         String,
  pub status:      String,
  pub status_code: usize,
}
pub struct Runtime {
  pub status:      String,
  pub status_code: usize,
}
pub struct Converter {
  runtime:       Runtime,
  ready:         bool,
  opts:          Config,
  core:          Core,
  /// How arXiv compiles the main source ([`Converter::note_main_source`]); a session applies it before its preloads,
  /// and a converter never told its source is a LaTeX, PDF one.
  compile_route: CompileRoute,
}

impl Converter {
  pub fn from_config(opts: Config) -> Converter {
    let core = Core::new(CoreOptions {
      verbosity: Some(opts.verbosity),
      include_comments: opts.include_comments.or(Some(false)),
      strict: opts.strict,
      include_styles: opts.include_styles,
      preload: opts.preload.clone(),
      search_paths: opts.search_paths.clone(),
      nomathparse: opts.nomathparse,
      source_map: opts.source_map,
      // Perl Core.pm L60-61: seed State PERL_INPUT_ENCODING from --inputencoding
      // (the Mouth reads it per-line to decode source bytes). `None` ⇒ utf-8.
      input_encoding: opts.inputencoding.clone(),
      ..CoreOptions::default()
    });
    Converter {
      runtime: Runtime {
        status:      String::new(),
        status_code: 3,
      },
      ready: false,
      opts,
      core,
      compile_route: CompileRoute::default(),
    }
  }

  /// Records, before a session's preloads load, whether `source` (a path or `literal:` content) is a plain TeX
  /// document: arXiv AutoTeX's tex-vs-latex test (`compile_route`'s cues). latexml.sty's preload registers an
  /// `\AddToHook`, which autoloads the LaTeX format; for a plain document that rebound `\end`, so the document's own
  /// `\end` raised "`\endgroup` Attempt to close a group that switched to mode vertical" (stopped full-arXiv run 329:
  /// 27 of 16,333 papers, 0911.4241, 1001.3079, hep-th9310069), and pstricks.tex took its LaTeX branch (repro
  /// loader/latexml_preload_keeps_plain.tex). A LaTeX document keeps the format at preload time, pdflatex's order.
  /// Also records a DVI driver class option. Every caller that prepares a session first tells it its source; a
  /// streaming restart or a supplement is a converter of its own and is told again.
  pub fn note_main_source(&mut self, source: &str) { self.compile_route = compile_route(source); }
  pub fn initialize_session(&mut self) -> Result<()> {
    // The main source's kind, for the preloads below and the document's PDF mode
    // (`core_interface::establish_pdf_output_mode`).
    state::set_plain_tex_document(self.compile_route.plain_tex);
    state::set_dvi_driver_option(self.compile_route.dvi);
    // Install the binding-resolution priority chain (rhai > contrib > package)
    // — the single source of resolution policy, shared with the integration-test
    // harness via `install_binding_dispatch`.
    install_default_binding_chain(self.opts.extra_bindings_dispatch.clone());
    // Prepare LaTeXML object — load mode-specific pool + user preloads.
    // Perl: $self->initializeState($mode.".pool", @{$$self{preload} || []})
    // For `--bibtex` (mode = BibTeX), Perl `Common/Config.pm:406`
    // unshifts ['TeX.pool', 'LaTeX.pool', 'BibTeX.pool'] into the preload
    // list. `BibTeX.pool` already begins with `LoadPool('LaTeX')` (and
    // LaTeX with TeX), so we only need the BibTeX entry — the transitive
    // chain handles the rest, and pool loads are idempotent.
    let mut preloads = match self.opts.mode {
      Some(DigestionMode::BibTeX) => vec![s!("TeX.pool"), s!("BibTeX.pool")],
      _ => vec![s!("TeX.pool")],
    };
    preloads.extend(self.core.preload.iter().cloned());
    self.core.initialize_singletons(preloads)?;
    // Warm libkpathsea's per-format lazy-init tables before the first file
    // lookup, matching the CLI binaries (`latexml_oxide.rs` / `cortex_worker.rs`
    // spawn this at startup). The library entry — tests via the trip harness,
    // `latexml::api`, and downstream embedders — otherwise skips that prewarm,
    // so libkpathsea's lazy `kpathsea_init_format` runs *during* the first
    // lookups. When many conversion threads share the one process-global
    // kpathsea handle under load (e.g. `cargo test --tests`), that mid-flight
    // lazy init can transiently mis-resolve a support file → a spurious, flaky
    // "1 warning". Running it inline here guarantees the tables are complete
    // before `convert()` looks anything up, on every thread. Idempotent and a
    // fast no-op once warm; single-process/single-thread runs (the CLI, the
    // one-conversion-per-process cortex fleet) are unaffected. Honours the
    // CLI's `LATEXML_NO_KPATHSEA_PREWARM` benchmarking opt-out.
    if std::env::var_os("LATEXML_NO_KPATHSEA_PREWARM").is_none() {
      latexml_core::util::pathname::prewarm_kpathsea();
    }
    // Record which file-resolution backend this process resolved, so every log
    // carries it. A dead or degraded kpathsea is otherwise invisible — it looks
    // exactly like a document referencing files that do not exist — and issue
    // #304 cost days for want of this one line in the reporter's log.
    let (backend, why) = latexml_core::util::pathname::kpathsea_backend();
    Info!("kpathsea", "backend", s!("{} ({why})", backend.as_str()));
    self.ready = true;
    Ok(())
  }

  pub fn bind_log(&mut self) { latexml_core::util::logger::bind_log(); }
  pub fn flush_log(&mut self) -> String { latexml_core::util::logger::flush_log() }

  pub fn convert(mut self, source: String) -> ConversionResponse {
    // 1 Prepare for conversion
    // 1.1 Initialize session if needed:
    if !self.ready {
      self.note_main_source(&source);
      let _g_bootstrap = telemetry::phase(Phase::Bootstrap);
      if let Err(e) = self.initialize_session() {
        // We can't initialize, return error:
        e.log_fatal();
      }
      drop(_g_bootstrap);
      if !self.ready {
        return ConversionResponse {
          result:      None,
          log:         self.flush_log(),
          status:      s!("Initialization failed."),
          status_code: 3,
        };
      }
    }

    self.bind_log();
    // 1.2 Inform of identity, increase conversion counter. Perl `bin/latexml`
    // L83 logs `Note("$LaTeXML::IDENTITY processing $source")` UNCONDITIONALLY;
    // our banner adds the executable name, git revision and exact start time
    // (`identity.rs`). `Note!` itself does the log-always / stderr-gated split, so
    // the banner reaches `.latexml.log` even under `--quiet` (issue #763) while
    // still being muted on the console — no verbosity guard here.
    Note!(crate::identity::identity_banner());
    // info!( "invoked as [$0 " . join(' ', @ARGV) . "]\n" if $$opts{verbosity} >= 1;

    // 1.3 Prepare for What's IN:
    // - We use a new temporary variable to avoid confusion with daemon caching
    // - Math needs to magically trigger math mode if needed
    // - Fragments need to have a default pre- and postamble, if none provided
    // Perl LaTeXML.pm:165-172 keys BOTH ambles on `whatsin`; see
    // `resolve_amble`. (The previous inline code keyed the postamble on
    // `whatsout`, dropping `\end{document}` / `\ensuremathpreceeds` for
    // fragment/math inputs.)
    let (current_preamble, current_postamble) = resolve_amble(
      &self.opts.whatsin,
      &self.opts.preamble,
      &self.opts.postamble,
    );
    // TODO:
    // 1.3.3 Archives need to get unpacked in a sandbox (with sufficient bookkeeping)
    //   elsif ($$opts{whatsin} =~ /^archive/) {
    //     // Sandbox the input
    //     $$opts{archive_sourcedirectory} = $$opts{sourcedirectory};
    //     my $sandbox_directory = File::Temp->newdir(TMPDIR => 1);
    //     $$opts{sourcedirectory} = $sandbox_directory;
    //     // Extract the archive in the sandbox
    //     $source = unpack_source($source, $sandbox_directory);
    //     if (!defined $source) {    // Unpacking failed to find a source
    //       $$opts{sourcedirectory} = $$opts{archive_sourcedirectory};
    //       my $log = $self->flush_log;
    // return { result => undef, log => $log, status => "Fatal:IO:Archive Can't detect a
    // source TeX file!", status_code => 3 }; } // Destination magic: If we expect an archive
    // on output, we need to invent the appropriate destination ourselves when not given.
    // // Since the LaTeXML API never writes the final archive file to disk, we just use a pretend
    // sourcename.zip:     if (($$opts{whatsout} =~ /^archive/) && (!$$opts{destination})) {
    //       $$opts{placeholder_destination} = 1;
    //       $$opts{destination}             = pathname_name($source) . ".zip"; } }

    //   // 1.4 Prepare for What's OUT (if we need a sandbox)
    //   if ($$opts{whatsout} =~ /^archive/) {
    //     $$opts{archive_sitedirectory} = $$opts{sitedirectory};
    //     $$opts{archive_destination}   = $$opts{destination};
    // my $destination_name = $$opts{destination} ? pathname_name($$opts{destination}) :
    // 'document';     my $sandbox_directory = File::Temp->newdir(TMPDIR => 1);
    //     my $extension = $$opts{format};
    //     $extension =~ s/\d+$//;
    //     $extension =~ s/^epub|mobi$/xhtml/;
    //     my $sandbox_destination = "$destination_name.$extension";
    //     $$opts{sitedirectory} = $sandbox_directory;

    //     if ($$opts{format} eq 'epub') {
    //       $$opts{resource_directory} = File::Spec->catdir($sandbox_directory, 'OPS');
    // $$opts{destination} = pathname_concat(File::Spec->catdir($sandbox_directory, 'OPS'),
    // $sandbox_destination); }     else {
    //       $$opts{destination} = pathname_concat($sandbox_directory, $sandbox_destination); }
    //   }

    // 1.5 Prepare a daemon frame
    // ...

    // 2 Beginning Core conversion - digest the source:
    // my ($digested, $dom, $serialized) = (undef, undef, undef);
    // Should be this, but is overridden by withState.
    // local $SIG{'ALRM'} = sub { LaTeXML::Common::Error::Fatal('conversion','timeout',
    // "Conversion timed out after " . $$opts{timeout} . " seconds!\n"); };
    // alarm($$opts{timeout});
    // my $mode = ($$opts{type} eq 'auto') ? 'TeX' : $$opts{type};
    // Streaming (fragmented) conversion: digest and build interleave inside
    // `convert_streaming`, so the eager digest-then-build sequence below does
    // not apply. TeX/Box outputs revert to eager — they serialize the DIGESTED
    // list and never build a DOM, so there is nothing to fragment.
    if let Some(budget) = self.opts.streaming
      && !matches!(self.opts.format, OutputFormat::TeX | OutputFormat::Box)
    {
      let dom_result = {
        let _g = telemetry::phase(Phase::Build);
        self.core.convert_streaming(
          source,
          current_preamble,
          current_postamble,
          self.opts.mode.clone(),
          budget,
        )
      };
      let serialized = match dom_result {
        Ok(dom) => {
          note_rootless_document(&dom);
          let _g = telemetry::phase(Phase::Serialize);
          dom.serialize_to_string()
        },
        Err(e) => {
          // Same resource-fatal surfacing as the eager DOM arm below.
          if matches!(e.category, ErrorCategory::StreamingRestart) {
            // Not a failure: the CLI reruns this document under --streaming
            // (the watermark crossed during the build; the digest arm above
            // handles the common digestion crossing).
            emit_info("streaming", "restart_watermark", &e.message);
          } else if matches!(e.target, ErrorTarget::Timeout) {
            e.log_fatal();
          } else {
            let message = s!("{:?}", e);
            let err = || {
              Error!("document", "convert", message);
              Ok(())
            };
            err().ok();
          }
          String::new()
        },
      };
      // The SHARED tail, not a hand-copy of parts of it: this arm used to
      // reproduce only the verdict fold and silently skip the rest — so a
      // streamed run emitted no MARPA_ASF_STATS line (measured: the 131 MB
      // witness under MARPA_ASF_STATS=1 produced zero stats), and a streamed
      // `--source-map` run emitted `data:sourcepos` tags with NO decoder
      // table in the log. Streaming auto-activates on exactly the large
      // documents where both matter.
      return self.finish_response(serialized);
    }

    // Adaptive digestion (single pass): eager, until an RSS-driven yield seam
    // says the process is over the spill watermark — then the accumulated
    // bodies become streaming fragment 1 and digestion CONTINUES as pass 1,
    // instead of the from-scratch `StreamingRestart` (kept as the no-seam
    // fallback). A document that never crosses the watermark is built whole,
    // exactly as before.
    let digest_result = {
      let _g = telemetry::phase(Phase::Digest);
      self.core.digest_adaptive(
        source,
        current_preamble,
        current_postamble,
        self.opts.mode.clone(),
        crate::streaming_restart::transition_budget(),
      )
    };
    let digest_result = match digest_result {
      Ok(crate::core_interface::AdaptiveOutcome::Streamed(dom)) => {
        let dom = *dom;
        note_rootless_document(&dom);
        // Pass 1 + pass 2 + finalize already ran; serialize like the
        // `--streaming` branch above.
        let serialized = {
          let _g = telemetry::phase(Phase::Serialize);
          dom.serialize_to_string()
        };
        return self.finish_response(serialized);
      },
      Ok(crate::core_interface::AdaptiveOutcome::Eager(d)) => Ok(d),
      Err(e) => Err(e),
    };
    let digested = match digest_result {
      Err(e) if matches!(e.category, ErrorCategory::StreamingRestart) => {
        // Not a failure: eager digestion stopped at the streaming-restart
        // watermark and the CLI reruns this document under --streaming. No
        // salvage (that would digest on), no Fatal line, no status change.
        emit_info("streaming", "restart_watermark", &e.message);
        return self.finish_response(String::new());
      },
      Err(e) => {
        report_mut!().status_code = 3;
        e.log_fatal();
        // Perl L251-259: If digestion failed, try finishDigestion to salvage
        // whatever was partially consumed. This allows partial recovery where
        // the beginning of the document is valid but an error occurs midway.
        match self.core.digest_internal() {
          Ok(salvaged) if !salvaged.is_empty().unwrap_or(true) => {
            Info!(
              "recovery",
              "digest",
              "Salvaged partial output after fatal error"
            );
            salvaged
          },
          _ => Digested::from(List::new(Vec::new())),
        }
      },
      Ok(d) => d,
    };
    // 2.1 Now, convert to DOM and output, if desired.
    let dom_result: Result<Document>;
    let serialized = match self.opts.format {
      OutputFormat::TeX => {
        let untex_result = { digested.untex() };
        match untex_result {
          Ok(tex) => tex,
          Err(e) => {
            return ConversionResponse {
              result:      None,
              log:         self.flush_log(),
              status:      s!("fatal:untex:{:?}", e),
              status_code: 3,
            };
          },
        }
      },
      OutputFormat::Box => {
        if self.opts.verbosity > 0 {
          digested.stringify()
        } else {
          digested.to_string()
        }
      },
      _ => {
        dom_result = {
          let _g = telemetry::phase(Phase::Build);
          self.core.convert_document(digested)
        };
        match dom_result {
          Ok(dom) => {
            note_rootless_document(&dom);
            let _g = telemetry::phase(Phase::Serialize);
            dom.serialize_to_string()
          },
          Err(e) => {
            // A resource fatal (Timeout target — e.g. a cycle-guard abort
            // propagated out of math parsing, P1-4) must surface as the
            // standard `Fatal:` log line, not a generic document error;
            // otherwise the summary counts a fatal the log never shows.
            if matches!(e.category, ErrorCategory::StreamingRestart) {
              // Not a failure: the CLI reruns this document under --streaming.
              emit_info("streaming", "restart_watermark", &e.message);
            } else if matches!(e.target, ErrorTarget::Timeout) {
              e.log_fatal();
            } else {
              let message = s!("{:?}", e);
              let err = || {
                Error!("document", "convert", message);
                Ok(())
              };
              err().ok();
            }
            String::new()
          },
        }
      },
    };

    self.runtime.status = get_status_message();
    self.runtime.status_code = get_status_code();
    // alarm(0)

    // 2.2 Bookkeeping in case fatal errors occurred
    // ...

    // 2.3 Clean up and exit if we only wanted the serialization of the core conversion
    // if ($serialized) {
    //   // If serialized has been set, we are done with the job
    //   // If we just processed an archive, clean up sandbox directory.
    //   if ($$opts{whatsin} =~ /^archive/) {
    //     rmtree($$opts{sourcedirectory});
    //     $$opts{sourcedirectory} = $$opts{archive_sourcedirectory}; }
    //   my $log = $self->flush_log;
    // return { result => $serialized, log => $log, status => $$runtime{status}, status_code =>
    // $$runtime{status_code} }; }

    // 3 If desired, post-process
    // my $result = $dom;
    // if ($$opts{post} && $dom && $dom->documentElement) {
    //   my $post_eval_return = eval {
    //     local $SIG{'ALRM'} = sub { die "alarm\n" };
    //     alarm($$opts{timeout});
    //     $result = $self->convert_post($dom);
    //     alarm(0);
    //     1;
    //   };
    //   // 3.1 Bookkeeping if a post-processing Fatal error occurred
    //   //// $$latexml{state}->noteStatus('fatal') if $latexml && $@; // Fatal Error?
    //   local $@ = 'Fatal:conversion:unknown Post-processing failed! (Unknown Reason)'
    //     if ((!$post_eval_return) && (!$@));
    //   if ($@) {    //Fatal occured!
    //     $$runtime{status_code} = 3;
    //     $@ = 'Fatal:conversion:unknown '.$@ unless $@ =~ /^Fatal:/;
    //     error!($@);
    //     //Since this is postprocessing, we don't need to do anything
    //     //   just avoid crashing...
    //     $result = undef; } }

    // // 4 Clean-up: undo everything we sandboxed
    // if ($$opts{whatsin} =~ /^archive/) {
    //   rmtree($$opts{sourcedirectory});
    //   $$opts{sourcedirectory} = $$opts{archive_sourcedirectory}; }
    // if ($$opts{whatsout} =~ /^archive/) {
    //   rmtree($$opts{sitedirectory});
    //   $$opts{sitedirectory} = $$opts{archive_sitedirectory};
    //   $$opts{destination}   = $$opts{archive_destination};
    //   if (delete $$opts{placeholder_destination}) {
    //     delete $$opts{destination}; } }

    // // 5 Output
    // // 5.1 Serialize the XML/HTML result (or just return the Perl object, if requested)
    // undef $serialized;
    // if ((defined $result) && ref($result) && (ref($result) =~ /^(:?LaTe)?XML/)) {
    //   if (($$opts{format} =~ 'x(ht)?ml') || ($$opts{format} eq 'jats')) {
    //     $serialized = $result->to_string(1); }
    //   elsif ($$opts{format} =~ /^html/) {
    //     if (ref($result) =~ '^LaTeXML::(Post::)?Document$') {    // Special for documents
    //       $serialized = $result->getDocument->to_stringHTML; }
    //     else {                                                   // Regular for fragments
    //       do {
    //         local $XML::LibXML::setTagCompression = 1;
    //         $serialized = $result->to_string(1);
    //         } } }
    //   elsif ($$opts{format} eq 'dom') {
    //     $serialized = $result; } }
    // else { $serialized = $result; }                              // Compressed case

    // 5.2 Finalize logging and return a response containing the document result, log and status
    self.finish_response(serialized)
  }

  /// The SHARED conversion tail: instrumentation flush (ASF stats, the
  /// `--source-map` decoder table), the Perl-faithful completion `Note!`, and
  /// response assembly. Every arm of `convert` must end here — the streaming
  /// arm used to return early with a hand-copy of the verdict fold alone,
  /// silently skipping the rest (no `MARPA_ASF_STATS` line, and a streamed
  /// `--source-map` run emitted `data:sourcepos` tags with no decoder ring).
  ///
  /// Recomputes `status`/`status_code` (idempotent reads of the REPORT
  /// counters), so diagnostics raised during serialization itself still reach
  /// the reported verdict — the streaming arm always did this; the eager path
  /// previously froze status before serializing.
  fn finish_response(&mut self, serialized: String) -> ConversionResponse {
    self.runtime.status = get_status_message();
    self.runtime.status_code = get_status_code();
    if self.opts.verbosity >= 0 {
      Debug!("arena", "strings_allocated", arena::len());
      // Final token-read progress: the calibration basis for `token_limit`
      // and `CYCLE_GUARD_ACTIVATE` (the read-checkpoint accounting changed
      // in PR #249 — read_x_token/read_balanced now count too — so limits
      // must be recalibrated against THIS metric, not historical figures).
      Debug!("gullet", "progress", latexml_core::gullet::token_progress());
    }
    // MARPA_ASF_STATS=1: emit ASF instrumentation counters once
    // per converted document. Codex instrumentation plan, see
    // marpa/docs/ASF_PERFORMANCE_FINDINGS.md. The thread-local
    // accumulator is reset after the snapshot so per-document
    // figures are independent.
    latexml_math_parser::report_and_reset_asf_stats();
    // --source-map (#47/#92): serialise the `tag → file` decoder table into the
    // `.log` — latexml-oxide's existing conversion-metadata channel — rather than
    // inlining it into the output. The output carries only the anonymous integer
    // `tag` (in each `data:sourcepos`); this is its decoder ring, Source-Map-v3
    // `sources`-style (the array index *is* the tag). Keeping it out of the
    // HTML/XML keeps that output anonymisable: a consumer without the source
    // files sees only opaque tags. In-process embedders (e.g. the ar5iv-editor
    // server) read the same table programmatically via `source_table_snapshot()`.
    // Gated on the switch, so a normal conversion emits nothing.
    if source_map_enabled() {
      for (tag, sym) in source_table_snapshot().iter().enumerate() {
        arena::with(*sym, |src| {
          Info!("source-map", "source", s!("[{tag}] {src}"));
        });
      }
    }
    // Perl: Note("Conversion complete: " . $$runtime{status}); (LaTeXML.pm:315)
    // is reached only on success — a Fatal `die`s before it, and bin/latexml:127
    // then prints `"Conversion " . ($code == 3 ? 'failed' : 'complete')`. Rust
    // recovers from a Fatal (graceful degradation) instead of dying, so it reaches
    // this note even when status_code == 3; fold in bin/latexml's verdict here so
    // a fatal run reports "failed", never the self-contradictory "complete: N fatal
    // error". Success cases (status_code < 3) stay byte-identical.
    Note!(s!("{}", conversion_verdict(self.runtime.status_code)));
    let log = self.flush_log();
    // self->sanitize($log) if ($$runtime{status_code} == 3);

    ConversionResponse {
      result: Some(serialized),
      log,
      status: self.runtime.status.clone(),
      status_code: self.runtime.status_code,
    }
  }

  /// Convert in-memory `content` under the source name `name`, producing the
  /// HTML5-format core XML (the persistent server then post-processes it).
  /// Unlike [`Converter::convert`] (`literal:` → anonymous source), the source
  /// is *named*, so `--source-map` stamps its locators. Focused on the
  /// `Document`/HTML5 path the server uses — no amble wrapping, no TeX/Box
  /// output formats.
  pub fn convert_content_with_provenance(
    mut self,
    name: &str,
    content: String,
  ) -> ConversionResponse {
    // Load + digest through the shared top-level loader, so the source-context
    // setup is not duplicated here.
    let digested = match self.digest_content_with_provenance(name, content) {
      Ok(d) => d,
      Err(e) => {
        if !self.ready {
          return ConversionResponse {
            result:      None,
            log:         self.flush_log(),
            status:      s!("Initialization failed."),
            status_code: 3,
          };
        }
        report_mut!().status_code = 3;
        e.log_fatal();
        // Salvage whatever digested before the error (mirrors `convert`).
        match self.core.digest_internal() {
          Ok(salvaged) if !salvaged.is_empty().unwrap_or(true) => salvaged,
          _ => Digested::from(List::new(Vec::new())),
        }
      },
    };

    let serialized = {
      let _g = telemetry::phase(Phase::Build);
      match self.core.convert_document(digested) {
        Ok(dom) => {
          // No `note_rootless_document` here: this in-process entry is also
          // the editor's fallback for a fragment without `\begin{document}`
          // (lsp_server/server.rs), a legitimate transient state.
          let _g = telemetry::phase(Phase::Serialize);
          dom.serialize_to_string()
        },
        Err(e) => {
          // `Error!` expands into a `Result`-returning context; wrap it the
          // same way `convert` does so it composes in this `-> ConversionResponse` fn.
          // Timeout-target resource fatals get the standard `Fatal:` line
          // (see the sibling handler in `convert` — P1-4).
          if matches!(e.category, ErrorCategory::StreamingRestart) {
            // Not a failure: the CLI reruns this document under --streaming
            // (the watermark crossed during the build; the digest arm above
            // handles the common digestion crossing).
            emit_info("streaming", "restart_watermark", &e.message);
          } else if matches!(e.target, ErrorTarget::Timeout) {
            e.log_fatal();
          } else {
            let message = s!("{:?}", e);
            let err = || {
              Error!("document", "convert", message);
              Ok(())
            };
            err().ok();
          }
          String::new()
        },
      }
    };

    // The verdict counts what building the document raised too — its own diagnostics, and the deferred ones replayed
    // once every reference is built (ruling 7e) — as `finish_response` recomputes it for `convert`; read after
    // digestion alone, a construction-time error left the status "No obvious problems".
    self.runtime.status = get_status_message();
    self.runtime.status_code = get_status_code();
    let log = self.flush_log();
    ConversionResponse {
      result: Some(serialized),
      log,
      status: self.runtime.status.clone(),
      status_code: self.runtime.status_code,
    }
  }

  pub fn prepare_session<'preplifetime>(
    &'preplifetime mut self,
    _opts: &'preplifetime Config,
  ) -> Result<()> {
    // Per-conversion cache hygiene: a persistent worker converts many papers
    // per thread; cwd-relative kpsewhich results must not leak across them.
    latexml_core::util::pathname::clear_kpsewhich_memo();
    latexml_core::util::image::clear_image_size_memo();
    latexml_core::common::font::tfm::reset_tfm_cache();
    if !self.ready {
      self.initialize_session()?
    }
    Ok(())
  }

  /// Digest in-memory `content` as the **main document** named `name`, leaving
  /// the thread-local engine state **live**. Used by the persistent server to
  /// warm a preamble once (then resume body digestion in a fork child over the
  /// inherited state) and by [`Converter::convert_content_with_provenance`] for
  /// the in-process path.
  ///
  /// This is the in-memory twin of [`crate::core_interface::DigestionAPI::digest_file`]:
  /// same top-level spine — establish the source context, open the source,
  /// `digest_internal` — but the content is *supplied* rather than read from
  /// disk. It shares `core_interface::establish_source_context` with
  /// `digest_file` (so `SOURCEFILE`/`SOURCEDIRECTORY`/`SEARCHPATHS`/
  /// `GRAPHICSPATHS`/`\jobname` can't drift), making sibling
  /// `\usepackage`/`\input`/`\includegraphics` of local files resolve. The
  /// source is opened as a *named* mouth (not the anonymous `literal:`
  /// protocol) so locators carry `name` — the **provenance** that
  /// `--source-map` needs (`stamp_source_locator` only stamps
  /// `.tex`/`.ltx`/`.bbl`/`.bib` user sources). Initializes the session if
  /// needed; does not finalize a document.
  pub fn digest_content_with_provenance(
    &mut self,
    name: &str,
    content: String,
  ) -> Result<Digested> {
    if !self.ready {
      self.initialize_session()?;
    }
    self.bind_log();
    // Top-level document load: establish the source context (SOURCEFILE,
    // SOURCEDIRECTORY, SEARCHPATHS, GRAPHICSPATHS, \jobname) so sibling
    // \usepackage/\input/\includegraphics of local files resolve. Shared with
    // `digest_file` via `establish_source_context` so the two can't drift.
    // (A continuation/nested mouth must NOT do this — see
    // `open_named_in_memory_mouth`.)
    let path = std::path::Path::new(name);
    let dir = path.parent().and_then(|p| p.to_str()).unwrap_or("");
    let jobname = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    crate::core_interface::establish_source_context(Some(name), jobname, dir);
    open_named_in_memory_mouth(name, content)?;
    self.core.digest_internal()
  }
}

/// Open a gullet mouth over in-memory `content` whose source is named `name`
/// (a real path/filename). Uses the Mouth's cached-content branch so locators
/// carry `name` rather than "Anonymous String".
///
/// Low-level and *position-agnostic*: it does NOT touch the document-global
/// `SOURCEDIRECTORY`/`SEARCHPATHS`, so it is safe for a continuation (the
/// forked child's body over already-inherited state) or a nested include. For
/// the *main* document load, go through `Converter::digest_named`, which
/// installs the document directory first.
pub fn open_named_in_memory_mouth(name: &str, content: String) -> Result<()> {
  use latexml_core::{
    gullet,
    mouth::{Mouth, MouthOptions},
  };
  let mouth = Mouth::create(name, MouthOptions {
    notes: true,
    content: Some(content),
    ..MouthOptions::default()
  })?;
  gullet::open_mouth(mouth, true);
  Ok(())
}

/// Resolve the `(preamble, postamble)` to wrap the source in, based on
/// the requested input chunk size. Faithful port of Perl `LaTeXML.pm`
/// L165-172 — note both ambles key on **`whatsin`** (not `whatsout`):
///
/// * `math` → `\begin{document}\ensuremathfollows` … `\ensuremathpreceeds\end{document}` (magic
///   math-mode trigger).
/// * `fragment` → the caller-supplied `preamble`/`postamble`, defaulting to `standard_preamble.tex`
///   / `standard_postamble.tex`.
/// * everything else (`document`, `archive`, …) → no wrapping.
pub(crate) fn resolve_amble(
  whatsin: &DataSize,
  preamble: &Option<String>,
  postamble: &Option<String>,
) -> (Option<String>, Option<String>) {
  match whatsin {
    DataSize::Math => (
      Some(s!("literal:\\begin{{document}}\\ensuremathfollows")),
      Some(s!("literal:\\ensuremathpreceeds\\end{{document}}")),
    ),
    DataSize::Fragment => (
      Some(
        preamble
          .clone()
          .unwrap_or_else(|| s!("standard_preamble.tex")),
      ),
      Some(
        postamble
          .clone()
          .unwrap_or_else(|| s!("standard_postamble.tex")),
      ),
    ),
    _ => (None, None),
  }
}

/// A built document with no root element — `\begin{document}` was never
/// digested, typically because an unbalanced `\ifX` (an undefined conditional
/// auto-`\newif`ed to `\iffalse`) skipped it through EOF — is a FATAL, not
/// an output. Both engines write the bare XML declaration for it (Perl too:
/// xwatermark-guide), but reporting it as status 2 with "output" hid a whole-
/// document loss behind an `Error:expected:\fi` line (skeyval-pokayoke2,
/// thesis-sample, xwatermark-guide in sweeps s109/s110). The messages are the
/// success signal, so the loss is named here — on the full-document `convert`
/// path (CLI, corpus harness, cortex_worker); the editor's in-process
/// fragment fallback is exempt. `Fatal!` logs and latches the sticky fatal at
/// the raise; the `Err` is not needed — serialization of the empty document
/// proceeds as before. Only a LaTeX document — one whose `\documentclass`
/// loaded a class (`document_class_filename`) — has a `\begin{document}` to
/// miss: a source that is all comments (arXiv's `%auto-ignore` placeholders,
/// 53 papers of sandbox 2605, e.g. 2605.00131; 28 of 2606) or a plain-TeX
/// file that typesets nothing lost no content, and stays clean as in Perl.
/// Guards
/// `perfect_kernel_batch56::document_without_a_root_is_a_fatal`,
/// `perfect_kernel_batch56::comment_only_source_is_not_a_fatal`.
fn note_rootless_document(dom: &Document) {
  // A Fatal already on record (TooManyErrors, a resource fuse) explains the
  // loss; do not add a second line for the same event.
  if dom.get_document().get_root_element().is_none()
    && get_status_code() < 3
    && state::has_value("document_class_filename")
  {
    let _: Result<()> = (|| {
      Fatal!(
        Document,
        Malformed,
        "The conversion built a document with no root element: \\begin{document} was never \
         reached (an unbalanced conditional skipped it?); the output is empty"
      );
    })();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn amble_math_wraps_both_ends() {
    // Perl LaTeXML.pm:166-168 — math sets BOTH preamble and postamble.
    let (pre, post) = resolve_amble(&DataSize::Math, &None, &None);
    assert_eq!(
      pre.as_deref(),
      Some("literal:\\begin{document}\\ensuremathfollows")
    );
    assert_eq!(
      post.as_deref(),
      Some("literal:\\ensuremathpreceeds\\end{document}")
    );
  }

  #[test]
  fn amble_fragment_defaults_to_standard_files() {
    let (pre, post) = resolve_amble(&DataSize::Fragment, &None, &None);
    assert_eq!(pre.as_deref(), Some("standard_preamble.tex"));
    assert_eq!(post.as_deref(), Some("standard_postamble.tex"));
  }

  #[test]
  fn amble_fragment_honors_explicit_files() {
    let (pre, post) = resolve_amble(
      &DataSize::Fragment,
      &Some("my_pre.tex".into()),
      &Some("my_post.tex".into()),
    );
    assert_eq!(pre.as_deref(), Some("my_pre.tex"));
    assert_eq!(post.as_deref(), Some("my_post.tex"));
  }

  #[test]
  fn amble_document_and_archive_have_no_wrapping() {
    assert_eq!(
      resolve_amble(&DataSize::Document, &None, &None),
      (None, None)
    );
    assert_eq!(
      resolve_amble(&DataSize::Archive, &None, &None),
      (None, None)
    );
  }

  /// `compile_route`'s tex-vs-latex and DVI cues (62a, stopped full-arXiv run 329; reviewer probes): a LaTeX
  /// document must never read as plain, which drops the preload-time format (ar5iv's locked `\today`).
  #[test]
  fn compile_route_of_main_sources() {
    const PLAIN_TEX: CompileRoute = CompileRoute {
      plain_tex: true,
      dvi:       false,
    };
    const LATEX_PDF: CompileRoute = CompileRoute {
      plain_tex: false,
      dvi:       false,
    };
    const LATEX_DVI: CompileRoute = CompileRoute {
      plain_tex: false,
      dvi:       true,
    };
    let route_of = |code: &str| compile_route(&format!("literal:{code}"));
    assert_eq!(route_of("Hello.\n\\bye\n"), PLAIN_TEX);
    assert_eq!(route_of("\\documentclass{article}\n"), LATEX_PDF);
    assert_eq!(route_of("\\documentstyle{article}\n"), LATEX_PDF);
    // CR-only (old Mac) and CRLF line ends: a comment line does not swallow the class.
    assert_eq!(
      route_of("% Mac file\r\\documentclass{article}\r"),
      LATEX_PDF
    );
    assert_eq!(
      route_of("% PC file\r\n\\documentclass{article}\r\n"),
      LATEX_PDF
    );
    // A commented class is no class; `\%` is a character, `\\%` a control symbol before a comment.
    assert_eq!(route_of("% \\documentclass{article}\nHello.\n"), PLAIN_TEX);
    assert_eq!(route_of("100\\% \\documentclass{article}\n"), LATEX_PDF);
    assert_eq!(route_of("x\\\\% \\documentclass{article}\n"), PLAIN_TEX);
    // A main file that `\input`s its preamble.
    assert_eq!(
      route_of("\\input pre\n\\begin{document}\nX\n\\end{document}\n"),
      LATEX_PDF
    );
    // A `.sty` a `filecontents` writes ends with its own `\endinput`; `\endinputs` is no `\endinput`.
    assert_eq!(
      route_of(
        "\\begin{filecontents*}{x.sty}\n\\def\\x{}\n\\endinput\n\\end{filecontents*}\n\\documentclass[dvips]{article}\n"
      ),
      LATEX_DVI
    );
    assert_eq!(
      route_of("\\def\\endinputs{}\n\\documentclass{article}\n"),
      LATEX_PDF
    );
    assert_eq!(route_of("\\begin {document}\nX\n"), LATEX_PDF);
    // TeX reads no further than `\endinput`.
    assert_eq!(
      route_of("Hello.\n\\endinput\n\\documentclass{article}\n"),
      PLAIN_TEX
    );
    // AMSTeX's own `\documentstyle` counts as LaTeX (plain AMSTeX measured worse; see `compile_route`).
    assert_eq!(
      route_of("\\input amstex\n\\documentstyle{amsppt}\n\\document\nX\n\\enddocument\n"),
      LATEX_PDF
    );
    // A DVI driver class option, unless the file sets `\pdfoutput=1` (exactly 1).
    assert_eq!(
      route_of("\\documentclass[12pt,dvips]{article}\n"),
      LATEX_DVI
    );
    assert_eq!(
      route_of("\\pdfoutput=1\n\\documentclass[dvips]{article}\n"),
      LATEX_PDF
    );
    assert_eq!(
      route_of("\\pdfoutput = 1\n\\documentclass[dvips]{article}\n"),
      LATEX_PDF
    );
    assert_eq!(
      route_of("\\pdfoutput=10\n\\documentclass[dvips]{article}\n"),
      LATEX_DVI
    );
    // A wrapper main file naming only its preamble and body (one level of `\input`/`\include`).
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
      dir.path().join("wpre.tex"),
      "\\documentclass[dvips]{article}\n",
    )
    .unwrap();
    std::fs::write(
      dir.path().join("wbody.tex"),
      "\\begin{document}X\\end{document}\n",
    )
    .unwrap();
    let main = dir.path().join("w.tex");
    std::fs::write(&main, "\\input{wpre}\n\\include{wbody}\n").unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), LATEX_DVI);
    std::fs::write(dir.path().join("wplain.tex"), "Hello.\n").unwrap();
    std::fs::write(&main, "\\input wplain\n\\bye\n").unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), PLAIN_TEX);
    // Two levels: main → setup → preamble.
    std::fs::write(dir.path().join("setup.tex"), "\\input wpre\n").unwrap();
    std::fs::write(&main, "\\input setup\n\\input wbody\n").unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), LATEX_DVI);
    // An input in `\iffalse … \fi` is not read; a dual-mode macro file's `\ifx\documentclass\undefined` is no class.
    std::fs::write(
      dir.path().join("dual.tex"),
      "\\ifx\\documentclass\\undefined\\def\\x{}\\fi\n",
    )
    .unwrap();
    std::fs::write(
      &main,
      "\\iffalse\\input{wpre}\\fi\n\\input dual\nHello.\n\\bye\n",
    )
    .unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), PLAIN_TEX);
    // An input name only TeX resolves leans LaTeX; `\iffalse … \else` keeps its `\else` branch.
    std::fs::write(&main, "\\def\\pre{wpre}\\input{\\pre}\nX \\today\n\\bye\n").unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), LATEX_PDF);
    std::fs::write(
      &main,
      "\\iffalse\\input{dual}\\else\\input{wpre}\\fi\n\\input wbody\n",
    )
    .unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), LATEX_DVI);
    // Commands plain TeX cannot run.
    assert_eq!(route_of("\\usepackage{amsmath}\nX\n"), LATEX_PDF);
    assert_eq!(route_of("\\include{chap}\n"), LATEX_PDF);
    assert_eq!(route_of("\\includegraphics{x}\n\\bye\n"), PLAIN_TEX);
    // The main file's `\pdfoutput=1` holds against a followed file's `[dvips]`.
    std::fs::write(&main, "\\pdfoutput=1\n\\input{wpre}\n\\include{wbody}\n").unwrap();
    assert_eq!(compile_route(&main.to_string_lossy()), LATEX_PDF);
  }
}
