//! The `Server`: warm-preamble cache state + the in-process
//! fallback conversion. The unix warm-fork pipeline (`unix.rs`)
//! adds `run_warm` in its own `impl Server` block.

use std::collections::BTreeMap;
use std::path::PathBuf;

use rustc_hash::FxHashMap;

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
  /// Warm-up read-log snapshot: every source the preamble digest opened,
  /// pinned as Overlay(version)/Disk(mtime) (`overlay::warmup_dep_snapshot`).
  pub(crate) warmed_source_deps:       Vec<(String, DepState)>,
  /// Open editor buffers (didOpen/didChange/convert text), keyed by
  /// absolute path — the unsaved-overlay source of truth.
  pub(crate) open_buffers:             FxHashMap<String, Buffer>,
  /// Engine `_contents` keys applied by the last `overlay::apply`, so
  /// closed buffers get cleared on the next apply.
  pub(crate) overlay_keys:             Vec<String>,
  /// Per-directory root-detection cache.
  pub(crate) root_cache:               RootCache,
  /// Client-configured root (`initializationOptions.rootDocument`).
  pub(crate) root_override:            Option<PathBuf>,
  /// Per root-uri: the file uris diagnostics were last published to, so a
  /// newly-clean file gets an explicit empty publish (stale squiggles clear).
  pub(crate) last_published:           FxHashMap<String, Vec<String>>,
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
      warmed_source_deps: Vec::new(),
      open_buffers: FxHashMap::default(),
      overlay_keys: Vec::new(),
      root_cache: RootCache::default(),
      root_override: None,
      last_published: FxHashMap::default(),
    }
  }

  /// Record buffer state from didOpen/didChange/convert. `version` is the
  /// LSP-provided document version when present; otherwise the previous
  /// version + 1 (so the dep snapshot still observes the change).
  pub(crate) fn upsert_buffer(&mut self, path: String, text: String, version: Option<i64>) {
    let next = version.unwrap_or_else(|| {
      self.open_buffers.get(&path).map(|b| b.version + 1).unwrap_or(1)
    });
    self.open_buffers.insert(path, Buffer { version: next, text });
  }

  /// Re-apply the overlay for the project rooted at `root` onto the engine's
  /// `_contents` channel. Call after any thread-state reset and before
  /// digestion / the body fork.
  pub(crate) fn apply_overlay(&mut self, root: &std::path::Path) {
    let dir = project_dir(root);
    self.overlay_keys = overlay_apply(&self.open_buffers, &dir, &self.overlay_keys);
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
    self.warmed_source_deps.clear();
  }

  /// Full, in-process conversion of `text`. Resets and rebuilds engine state,
  /// so it invalidates the warm cache. Used as the fallback whenever the
  /// warm-fork path is unavailable (no `\begin{document}`, fork failure,
  /// non-Unix). Line numbers are naturally file-relative (the whole document
  /// is one source), and the source-map table is read back after conversion.
  pub(crate) fn convert_in_process(&mut self, uri: &str, text: &str) -> ConvertOutput {
    self.invalidate_cache();
    latexml_core::state::reset_thread_state();
    // Overlay must be re-applied after the reset wiped the engine state.
    let root_path = PathBuf::from(get_file_path(uri));
    self.overlay_keys.clear();
    self.apply_overlay(&root_path);
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
      root: None, // attributed by the trigger wrappers
      status,
      status_code,
    }
  }
}

/// Publish diagnostics grouped per attributed file uri (multi-file model);
/// unattributed ones go to the edited buffer's uri. Files diagnosed last
/// round but clean now get an explicit empty publish so stale squiggles
/// clear. Shared by the unix and generic server flavors.
pub(crate) fn publish_grouped_diagnostics(
  server: &mut Server,
  root_uri: &str,
  edited_uri: &str,
  out: &ConvertOutput,
  writer: &mut impl std::io::Write,
) -> std::io::Result<()> {
  let mut by_uri: FxHashMap<String, Vec<Diag>> = FxHashMap::default();
  for d in &out.diags {
    let uri = match d.file.as_deref() {
      Some(path) => format!("file://{path}"),
      None => edited_uri.to_string(),
    };
    by_uri.entry(uri).or_default().push(d.clone());
  }
  let published: Vec<String> = by_uri.keys().cloned().collect();
  for (uri, diags) in &by_uri {
    send_message(writer, &publish_diagnostics_notification(uri, diags))?;
  }
  // Clear files that had diagnostics last round but none now.
  if let Some(previous) = server.last_published.insert(root_uri.to_string(), published.clone()) {
    for stale in previous {
      if !published.contains(&stale) {
        send_message(writer, &publish_diagnostics_notification(&stale, &[]))?;
      }
    }
  }
  Ok(())
}
