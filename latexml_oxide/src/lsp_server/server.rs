//! The `Server`: warm-preamble cache state + the in-process
//! fallback conversion. The unix warm-fork pipeline (`unix.rs`)
//! adds `run_warm` in its own `impl Server` block.

use std::collections::BTreeMap;

use crate::converter::Converter;

use super::*;

// ======================================================================
// Server — warm preamble cache + in-process fallback conversion.
// ======================================================================

pub(crate) struct Server {
  pub(crate) begin_doc_regex:          regex::Regex,
  /// Per-conversion wall-clock budget in seconds (`--timeout`; 0 disables).
  pub(crate) timeout_secs:             u64,
  /// Per-conversion resident-memory ceiling in KiB (`--max-memory`; 0 disables).
  pub(crate) max_rss_kb:               u64,
  pub(crate) warmed_uri:               Option<String>,
  pub(crate) warmed_preamble:          Option<String>,
  pub(crate) warmed_preamble_digested: Option<latexml_core::digested::Digested>,
  /// Log captured while digesting the preamble, so preamble diagnostics
  /// survive across body-only fork conversions (the warmup only re-runs on a
  /// cache miss).
  pub(crate) warmed_preamble_log:      String,
  pub(crate) warmed_dependencies:      BTreeMap<String, std::time::SystemTime>,
}

impl Server {
  pub(crate) fn new(timeout_secs: u64, max_rss_kb: u64) -> Self {
    Server {
      begin_doc_regex: regex::Regex::new(r"\\begin\s*\{\s*document\s*\}").unwrap(),
      timeout_secs,
      max_rss_kb,
      warmed_uri: None,
      warmed_preamble: None,
      warmed_preamble_digested: None,
      warmed_preamble_log: String::new(),
      warmed_dependencies: BTreeMap::new(),
    }
  }

  /// Drop the warm preamble cache. MUST be called whenever the thread-local
  /// engine state is reset by an in-process conversion — otherwise a later
  /// cache-hit fork would inherit a state that no longer matches the cached
  /// preamble boxes (the bug that made `didChange` corrupt the next
  /// `latexml/convert`).
  pub(crate) fn invalidate_cache(&mut self) {
    self.warmed_uri = None;
    self.warmed_preamble = None;
    self.warmed_preamble_digested = None;
    self.warmed_preamble_log.clear();
    self.warmed_dependencies.clear();
  }

  /// Full, in-process conversion of `text`. Resets and rebuilds engine state,
  /// so it invalidates the warm cache. Used as the fallback whenever the
  /// warm-fork path is unavailable (no `\begin{document}`, fork failure,
  /// non-Unix). Line numbers are naturally file-relative (the whole document
  /// is one source), and the source-map table is read back after conversion.
  pub(crate) fn convert_in_process(&mut self, uri: &str, text: &str) -> ConvertOutput {
    self.invalidate_cache();
    latexml_core::state::reset_thread_state();
    // Cooperative wall-clock guard for the in-process path. (There is no child
    // to reap here — this path runs on the server's own thread — so the hard
    // RAM/time backstops don't apply; the cooperative deadline + the engine's
    // RSS fuse are what bound a runaway fallback conversion.)
    latexml_core::stomach::set_timeout(self.timeout_secs);

    let opts = make_config(uri);
    let mut converter = Converter::from_config(opts.clone());
    if let Err(e) = converter.prepare_session(&opts) {
      return ConvertOutput::error(format!("Fatal: prepare_session failed: {e}"));
    }
    converter.bind_log();
    // Fallback path (no `\begin{document}`, fork failure, or non-Unix). Use a
    // *named* in-memory source (the document path) so `--source-map` stamps
    // locators here too, matching the warm-fork path.
    let resp = converter.convert_content_with_provenance(&get_file_path(uri), text.to_string());
    let sources = collect_sources(uri);
    let status_code = resp.status_code as i64;
    let status = if resp.status.is_empty() {
      status_label(status_code).to_string()
    } else {
      resp.status
    };
    let log = resp.log;
    let diags = parse_log_diagnostics(&log);
    let html = post_process_html(&resp.result.unwrap_or_default(), uri);
    ConvertOutput {
      html,
      log,
      diags,
      sources,
      status,
      status_code,
    }
  }
}

