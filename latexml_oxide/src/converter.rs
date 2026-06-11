use std::rc::Rc;

use latexml_core::{
  Core, CoreOptions, Error, Fatal, Info, Note,
  common::{Config, DataSize, DigestionMode, OutputFormat, arena, error::*, object::Object},
  digested::Digested,
  document::Document,
  list::List,
  report_mut, s,
  state::{
    add_binding_names, set_bindings_dispatch, set_extra_bindings_dispatch, source_map_enabled,
    source_table_snapshot,
  },
  telemetry::{self, Phase},
};

use crate::core_interface::DigestionAPI;

const CONVERTER_IDENTITY: &str = "latexml_oxide (v0.5.0)";

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
  runtime: Runtime,
  ready:   bool,
  opts:    Config,
  core:    Core,
}

impl Converter {
  pub fn from_config(opts: Config) -> Converter {
    let core = Core::new(CoreOptions {
      verbosity: Some(opts.verbosity),
      include_comments: opts.include_comments.or(Some(false)),
      preload: opts.preload.clone(),
      search_paths: opts.search_paths.clone(),
      nomathparse: opts.nomathparse,
      source_map: opts.source_map,
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
    }
  }
  pub fn initialize_session(&mut self) -> Result<()> {
    // Add default package bindings.
    set_bindings_dispatch(Rc::new(latexml_package::dispatch));
    // Register every (name, ext) binding pair so `find_file(notex=true)`
    // can resolve compile-time bindings across all extensions
    // (.cls/.sty/.def/.pool/code.tex/...). This also feeds `load_class`'s
    // Perl-parity prefix-match fallback (Package.pm L2702-2706) via the
    // class-filtered `state::get_class_binding_names()` view. Source of
    // truth: `latexml_package::BINDINGS`.
    add_binding_names(latexml_package::binding_names());
    // Add additional binding definitions if any
    if let Some(closure) = &self.opts.extra_bindings_dispatch {
      set_extra_bindings_dispatch(closure.clone());
    }
    // Default runtime-binding discovery (docs/script_bindings_plan.md §7):
    // when no embedder-supplied dispatcher exists, resolve `<name>.<ext>.rhai`
    // (e.g. `mypkg.sty.rhai`) through the same searchpath machinery as raw
    // TeX sources and load it — downstream users of the single-file binary
    // drop a .rhai next to their document to customize without recompiling.
    #[cfg(feature = "runtime-bindings")]
    if self.opts.extra_bindings_dispatch.is_none() {
      set_extra_bindings_dispatch(Rc::new(|request: &str| {
        let candidate = format!("{request}.rhai");
        latexml_core::binding::content::find_file(&candidate, None)
          .map(|path| latexml_contrib::script_bindings::load_file(&path).map(|_| ()))
      }));
    }
    // Also expose contrib's bindings (memoir / siamltex / scrbook / etc.)
    // so they participate in the same resolution pool. We unconditionally
    // register latexml_contrib here because the canonical setup for both
    // `latexml_oxide` and `cortex_worker` binaries loads it — downstream
    // embedders that replace the dispatchers can register their own
    // (name, ext) slice the same way via `add_binding_names`.
    add_binding_names(latexml_contrib::binding_names());
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
    self.ready = true;
    Ok(())
  }

  pub fn bind_log(&mut self) { latexml_core::util::logger::bind_log(); }
  pub fn flush_log(&mut self) -> String { latexml_core::util::logger::flush_log() }

  pub fn convert(mut self, source: String) -> ConversionResponse {
    // 1 Prepare for conversion
    // 1.1 Initialize session if needed:
    if !self.ready {
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
    // 1.2 Inform of identity, increase conversion counter
    if self.opts.verbosity >= 0 {
      Note!(CONVERTER_IDENTITY);
      // info!( "invoked as [$0 " . join(' ', @ARGV) . "]\n" if $$opts{verbosity} >= 1;
      // info!("processing started " . localtime() . "\n"; )
    }

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
    let digest_result = {
      let _g = telemetry::phase(Phase::Digest);
      self.core.digest(
        source,
        current_preamble,
        current_postamble,
        self.opts.mode.clone(),
        true,
      )
    };
    let digested = match digest_result {
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
            let _g = telemetry::phase(Phase::Serialize);
            dom.serialize_to_string()
          },
          Err(e) => {
            // A resource fatal (Timeout target — e.g. a cycle-guard abort
            // propagated out of math parsing, P1-4) must surface as the
            // standard `Fatal:` log line, not a generic document error;
            // otherwise the summary counts a fatal the log never shows.
            if matches!(e.target, ErrorTarget::Timeout) {
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
    if self.opts.verbosity >= 0 {
      Info!("arena", "strings_allocated", arena::len());
      // Final token-read progress: the calibration basis for `token_limit`
      // and `CYCLE_GUARD_ACTIVATE` (the read-checkpoint accounting changed
      // in PR #249 — read_x_token/read_balanced now count too — so limits
      // must be recalibrated against THIS metric, not historical figures).
      Info!("gullet", "progress", latexml_core::gullet::final_progress());
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
    let verb = if self.runtime.status_code == 3 {
      "failed"
    } else {
      "complete"
    };
    Note!(s!("Conversion {}: {}", verb, self.runtime.status));
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

    self.runtime.status = get_status_message();
    self.runtime.status_code = get_status_code();

    let serialized = {
      let _g = telemetry::phase(Phase::Build);
      match self.core.convert_document(digested) {
        Ok(dom) => {
          let _g = telemetry::phase(Phase::Serialize);
          dom.serialize_to_string()
        },
        Err(e) => {
          // `Error!` expands into a `Result`-returning context; wrap it the
          // same way `convert` does so it composes in this `-> ConversionResponse` fn.
          // Timeout-target resource fatals get the standard `Fatal:` line
          // (see the sibling handler in `convert` — P1-4).
          if matches!(e.target, ErrorTarget::Timeout) {
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
  /// disk. It shares [`crate::core_interface::establish_source_context`] with
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
/// the *main* document load, go through [`Converter::digest_named`], which
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
}
