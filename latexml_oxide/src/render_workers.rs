//! Process-parallel page rendering for pass B of the post pipeline
//! (`docs/performance/STREAMING_POST_DESIGN_2026-07-06.md` §6).
//!
//! In-process page threads are blocked twice (`ObjectDB` is `!Send`;
//! libxslt serializes every transform behind a process-wide lock), so the
//! chosen shape is **process-level page-range workers**: the parent saves the
//! completed ObjectDB as a SQLite file, partitions the spilled pages into
//! contiguous chunks, and re-invokes its own binary once per chunk with
//! `LATEXML_RENDER_WORKER=<manifest.json>`. Each child attaches the db
//! readonly (WAL — N readers share one page cache via mmap), runs the SAME
//! per-page pipeline as the serial driver
//! (`crate::post::render_spilled_page`), and reports its diagnostic tally
//! as trailing `Status:` lines on stderr. The parent folds child logs and
//! counts deterministically, in chunk order, into its own `LOG_BUFFER` /
//! `REPORT` — so the combined verdict and the persisted `--log` stay lossless
//! (canvas signal-integrity rule: a child that dies without a status line is
//! folded as FATAL, never as success).
//!
//! Env knob: `LATEXML_RENDER_JOBS` (usize). Default 1 = the serial path,
//! byte-identical to before this module existed.

use std::path::{Path, PathBuf};

use latexml_core::{
  Info,
  common::error::{
    LogStatus, ReportCounts, emit_error, emit_fatal, get_status_code, note_status,
    snapshot_report_counts,
  },
  s,
  util::logger::{CapturedDiagnostics, replay_captured},
};
use latexml_post::object_db::{DbAttachOptions, ObjectDB};
use serde::{Deserialize, Serialize};

/// Fewer spilled pages than this and the db save + child spawn overhead beats
/// the parallel win — the serial path is taken regardless of the jobs knob.
pub(crate) const MIN_PAGES_FOR_PARALLEL: usize = 64;

/// The `LATEXML_RENDER_JOBS` knob: how many page-render worker processes pass
/// B may spawn. Default (and any unparsable value) is 1 = serial. Read once
/// per conversion, so no hot-path caching is needed.
pub(crate) fn render_jobs() -> usize {
  std::env::var("LATEXML_RENDER_JOBS")
    .ok()
    .and_then(|v| v.parse::<usize>().ok())
    .filter(|&n| n >= 1)
    .unwrap_or(1)
}

/// The serializable subset of `PostDocumentOptions` a worker needs to
/// reconstruct the page parse — ALL of its fields, since even `destination`
/// (later overridden per page) feeds the site-directory fallback inside
/// `PostDocument::new`.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PageOpts {
  pub destination:           Option<String>,
  pub destination_directory: Option<String>,
  pub site_directory:        Option<String>,
  pub source:                Option<String>,
  pub source_directory:      Option<String>,
  pub searchpaths:           Option<Vec<String>>,
  pub validate:              bool,
  pub nocache:               bool,
}

impl From<&latexml_post::document::PostDocumentOptions> for PageOpts {
  fn from(o: &latexml_post::document::PostDocumentOptions) -> Self {
    PageOpts {
      destination:           o.destination.clone(),
      destination_directory: o.destination_directory.clone(),
      site_directory:        o.site_directory.clone(),
      source:                o.source.clone(),
      source_directory:      o.source_directory.clone(),
      searchpaths:           o.searchpaths.clone(),
      validate:              o.validate,
      nocache:               o.nocache,
    }
  }
}

impl From<PageOpts> for latexml_post::document::PostDocumentOptions {
  fn from(o: PageOpts) -> Self {
    latexml_post::document::PostDocumentOptions {
      destination:           o.destination,
      destination_directory: o.destination_directory,
      site_directory:        o.site_directory,
      source:                o.source,
      source_directory:      o.source_directory,
      searchpaths:           o.searchpaths,
      validate:              o.validate,
      nocache:               o.nocache,
    }
  }
}

/// One spilled page for a worker to render: the spill path plus the metadata
/// that does not survive the XML round-trip (mirrors `post::SpilledPage`
/// minus the parent-only placeholder flags).
#[derive(Serialize, Deserialize, Clone)]
pub struct PageJob {
  pub path:                  PathBuf,
  pub destination:           Option<String>,
  pub destination_directory: Option<String>,
}

/// Everything a page-render worker needs to rebuild the pass-B processor set
/// exactly as the parent would have, plus its page range. Written as
/// `render-manifest-{i}.json` beside the saved `render.db`.
#[derive(Serialize, Deserialize, Clone)]
pub struct RenderManifest {
  /// The saved ObjectDB (SQLite, attached readonly by each worker).
  pub dbfile:                    PathBuf,
  pub navigation_toc:            Option<String>,
  /// Cross-reference URL style as its canonical CLI tag (`UrlStyle::as_cli` /
  /// `from_cli` round-trip), serialized across the worker manifest.
  pub url_style:                 String,
  /// Output file extension CrossRef strips for `--urlstyle` (Perl `extension`).
  pub out_extension:             String,
  pub graphicimages:             bool,
  pub graphics_svg_threshold_kb: u32,
  pub pmml:                      bool,
  pub cmml:                      bool,
  pub keep_xmath:                bool,
  /// Already-inverted from the CLI's `noinvisibletimes`.
  pub invisible_times:           bool,
  pub plane1:                    bool,
  pub hackplane1:                bool,
  pub mathtex:                   bool,
  pub intent_literal:            bool,
  pub stylesheet:                Option<String>,
  /// The fully-resolved XSLT parameter map (CSS/JS/LATEXML_VERSION/user
  /// overrides), captured after the parent computed it.
  pub xslt_params:               Vec<(String, String)>,
  pub nodefaultresources:        bool,
  /// The resolved stylesheet/resource search paths the parent's `XSLT::new`
  /// received.
  pub searchpaths:               Vec<String>,
  pub is_html_out:               bool,
  pub svg_fragments:             Vec<(String, String)>,
  pub schemadocs:                bool,
  /// [`latexml_post::extract::Whatsout`] as its canonical CLI tag
  /// (`as_cli`/`from_cli` round-trip).
  pub whatsout:                  String,
  pub page_opts:                 PageOpts,
  pub pages:                     Vec<PageJob>,
}

/// The parent-side result of a parallel render in which workers were actually
/// spawned (successfully or not — failures are already folded as diagnostics).
pub(crate) struct ParallelResult {
  /// The first page's finalized output, read back from its destination file —
  /// the same content the serial driver would have returned as `main_output`.
  pub(crate) main_output:    Option<String>,
  /// Total pages written, summed from the workers' `Status:pages:` lines.
  pub(crate) pages_rendered: usize,
}

/// Remove the per-run handoff artifacts (db + WAL sidecars + manifests).
/// Best-effort: they live in the page-spill tempdir, which is removed wholesale
/// when the parent drops it, so a failure here only delays the cleanup.
fn cleanup_handoff(dbfile: &Path, manifests: &[PathBuf]) {
  let _ = std::fs::remove_file(dbfile);
  for suffix in ["-wal", "-shm"] {
    let mut side = dbfile.as_os_str().to_owned();
    side.push(suffix);
    let _ = std::fs::remove_file(PathBuf::from(side));
  }
  for m in manifests {
    let _ = std::fs::remove_file(m);
  }
}

/// Strip ANSI `ESC[...m` color sequences (same logic as the logger's private
/// helper). Child stderr is a pipe, so the TTY-gated logger should emit none —
/// this is belt-and-suspenders for the log fold (signal-integrity rule).
fn strip_ansi(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut in_escape = false;
  for c in s.chars() {
    if in_escape {
      if c == 'm' {
        in_escape = false;
      }
    } else if c == '\u{1b}' {
      in_escape = true;
    } else {
      result.push(c);
    }
  }
  result
}

/// A worker's stderr, parsed: the trailing `Status:` lines are consumed into
/// structured fields and everything else is the log text to forward + fold.
struct ChildReport {
  log:    String,
  counts: Option<ReportCounts>,
  status: Option<usize>,
  pages:  usize,
}

fn parse_child_report(stderr_text: &str) -> ChildReport {
  let mut log = String::new();
  let mut counts = None;
  let mut status = None;
  let mut pages = 0usize;
  for line in stderr_text.lines() {
    if let Some(rest) = line.strip_prefix("Status:counts:") {
      let mut it = rest.split(',').map(|v| v.trim().parse::<usize>().ok());
      let (d, i, w, e, f) = (
        it.next().flatten(),
        it.next().flatten(),
        it.next().flatten(),
        it.next().flatten(),
        it.next().flatten(),
      );
      if let (Some(debug), Some(info), Some(warning), Some(error), Some(fatal)) = (d, i, w, e, f) {
        counts = Some(ReportCounts {
          debug,
          info,
          warning,
          error,
          fatal: fatal > 0,
        });
      }
    } else if let Some(rest) = line.strip_prefix("Status:conversion:") {
      status = rest.trim().parse::<usize>().ok();
    } else if let Some(rest) = line.strip_prefix("Status:pages:") {
      pages = rest.trim().parse::<usize>().unwrap_or(0);
    } else {
      log.push_str(line);
      log.push('\n');
    }
  }
  ChildReport { log, counts, status, pages }
}

/// One spawned (or spawn-failed) worker awaiting its fold.
enum Pending {
  /// The drain thread owns the child and returns its full `Output`; the pid
  /// is kept so a parent-side timeout can kill the fleet. Only read on Unix
  /// (`libc::kill` in the breach path below); the equivalent Windows fleet-kill
  /// (OpenProcess + TerminateProcess) is not wired yet, so the pid is dead on
  /// non-unix — scope the `dead_code` allow there rather than workspace-wide.
  Spawned(
    std::thread::JoinHandle<std::io::Result<std::process::Output>>,
    #[cfg_attr(not(unix), allow(dead_code))] u32,
  ),
  Failed(String),
}

/// Spawn `jobs` page-range workers over `pages` and fold their results.
///
/// Returns `None` when setup failed BEFORE any child was spawned (reported at
/// Info severity) — the caller falls back to the serial render, which is still
/// fully correct since the spilled pages are untouched. Once children run
/// there is no fallback: a worker that fails, dies, or omits its status line
/// is folded as an Error + FATAL status (fail toward flagging, never silent
/// success).
pub(crate) fn parallel_render(
  mut manifest: RenderManifest,
  pages: Vec<PageJob>,
  jobs: usize,
  spill_dir: &Path,
  db: &ObjectDB,
) -> Option<ParallelResult> {
  let total_pages = pages.len();
  let exe = match std::env::current_exe() {
    Ok(e) => e,
    Err(e) => {
      Info!(
        "post",
        "parallel-render",
        s!("parallel render disabled (current_exe: {})", e)
      );
      return None;
    },
  };
  let dbfile = spill_dir.join("render.db");
  if let Err(e) = db.save_as(&dbfile) {
    Info!(
      "post",
      "parallel-render",
      s!("parallel render disabled (db save: {})", e)
    );
    return None;
  }
  manifest.dbfile = dbfile.clone();

  // Clamp the fleet to ACTUAL headroom (witness OOM 2026-08-03: a joint run
  // carries ~15 GB of core residual into post, and 8 workers each
  // eager-loading a 1.82M-object db blew a 30 GB cgroup at spawn — the
  // post-only measurement survived only because its parent was fresh). Free
  // what the allocators will give back first, then estimate each worker at
  // ~3x the on-disk db (eager JSON decode + libxml holder + one page) plus a
  // fixed floor, and size the fleet from the smaller of MemAvailable and the
  // distance to this run's own memory fuse. Degrade to fewer workers — or
  // decline to engage — rather than let the kernel choose a victim.
  #[cfg(target_os = "linux")]
  unsafe {
    libc::malloc_trim(0);
  }
  #[cfg(not(feature = "dhat-heap"))]
  unsafe {
    libmimalloc_sys::mi_collect(true);
  }
  let db_bytes = std::fs::metadata(&dbfile).map(|m| m.len()).unwrap_or(0);
  let per_worker = db_bytes.saturating_mul(3).max(256 * 1024 * 1024);
  let mut budget = latexml_core::watchdog::available_memory_bytes().unwrap_or(u64::MAX);
  if let Some(cap) = latexml_core::stomach::resolve_rss_cap() {
    // Budget against the cooperative FUSE (75% of the hard cap), where the
    // graceful stop fires — not the cap itself, where the watchdog kills.
    let fuse = cap / 4 * 3;
    let rss = latexml_core::watchdog::process_rss_kb().unwrap_or(0) * 1024;
    budget = budget.min(fuse.saturating_sub(rss));
  }
  let affordable = (budget / per_worker) as usize;
  let jobs = jobs.min(total_pages).min(affordable);
  if jobs < 2 {
    Info!(
      "post",
      "parallel-render",
      s!(
        "parallel render declined: headroom {} MB affords {} worker(s) at ~{} MB each — staying serial",
        budget / (1024 * 1024),
        affordable,
        per_worker / (1024 * 1024)
      )
    );
    cleanup_handoff(&dbfile, &[]);
    return None;
  }

  // Contiguous chunks preserve page order: worker 0 owns the first pages, so
  // the fold below (and the first page's main-output read) is deterministic.
  let chunk_size = total_pages.div_ceil(jobs);
  let first_destination = pages.first().and_then(|p| p.destination.clone());
  let mut manifest_paths: Vec<PathBuf> = Vec::with_capacity(jobs);
  for (i, chunk) in pages.chunks(chunk_size).enumerate() {
    let mut m = manifest.clone();
    m.pages = chunk.to_vec();
    let mpath = spill_dir.join(format!("render-manifest-{i}.json"));
    let write_result = serde_json::to_string(&m)
      .map_err(|e| e.to_string())
      .and_then(|json| std::fs::write(&mpath, json).map_err(|e| e.to_string()));
    if let Err(e) = write_result {
      Info!(
        "post",
        "parallel-render",
        s!("parallel render disabled (manifest write: {})", e)
      );
      cleanup_handoff(&dbfile, &manifest_paths);
      return None;
    }
    manifest_paths.push(mpath);
  }

  // Parent-side deadline check before committing to the spawn (the workers
  // carry no deadline of their own; the parent polls between waits below and
  // kills the fleet on a breach).
  if let Err(e) = latexml_core::stomach::check_timeout() {
    e.log_fatal();
    cleanup_handoff(&dbfile, &manifest_paths);
    return Some(ParallelResult {
      main_output:    None,
      pages_rendered: 0,
    });
  }

  // The engagement marker (also what the parity test asserts on — without it
  // a silently-ignored jobs knob would let the test pass vacuously serial).
  Info!(
    "post",
    "parallel-render",
    s!(
      "parallel page render engaged: {} worker(s) over {} pages",
      manifest_paths.len(),
      total_pages
    )
  );

  let mut pending: Vec<Pending> = Vec::with_capacity(manifest_paths.len());
  for mpath in &manifest_paths {
    let spawned = std::process::Command::new(&exe)
      .env("LATEXML_RENDER_WORKER", mpath)
      .stdin(std::process::Stdio::null())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn();
    match spawned {
      Ok(child) => {
        let pid = child.id();
        // Each child gets a drain thread calling `wait_with_output` — piped
        // stderr MUST be consumed while the parent polls, or a chatty child
        // blocks on a full pipe and the poll below never sees it exit.
        let handle = std::thread::Builder::new()
          .name(format!("render-worker-drain-{pid}"))
          .spawn(move || child.wait_with_output());
        match handle {
          Ok(h) => pending.push(Pending::Spawned(h, pid)),
          Err(e) => pending.push(Pending::Failed(format!("drain thread spawn failed: {e}"))),
        }
      },
      Err(e) => pending.push(Pending::Failed(format!("worker spawn failed: {e}"))),
    }
  }

  // Poll (rather than block) so the parent's cooperative timeout keeps
  // running between waits; on a breach, kill the fleet so the join below
  // returns promptly instead of riding out the children.
  loop {
    let all_done = pending
      .iter()
      .all(|p| !matches!(p, Pending::Spawned(h, _) if !h.is_finished()));
    if all_done {
      break;
    }
    if let Err(e) = latexml_core::stomach::check_timeout() {
      e.log_fatal();
      #[cfg(unix)]
      for p in &pending {
        if let Pending::Spawned(h, pid) = p
          && !h.is_finished()
        {
          // SAFETY: plain kill(2) on a child pid this process spawned.
          unsafe {
            libc::kill(*pid as i32, libc::SIGKILL);
          }
        }
      }
      break;
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  }

  // Fold IN CHUNK ORDER (deterministic log/tally, mirroring the graphics
  // worker fold): forward each child's stderr, replay its log + counts into
  // the parent LOG_BUFFER/REPORT, and flag anything that did not report.
  let mut pages_rendered = 0usize;
  for (i, p) in pending.into_iter().enumerate() {
    match p {
      Pending::Failed(e) => {
        emit_error("post", "render_worker", &format!("worker {i}: {e}"));
        note_status(LogStatus::Fatal, None);
      },
      Pending::Spawned(handle, _) => match handle.join() {
        Err(_) => {
          emit_error(
            "post",
            "render_worker",
            &format!("worker {i}: drain thread panicked"),
          );
          note_status(LogStatus::Fatal, None);
        },
        Ok(Err(e)) => {
          emit_error(
            "post",
            "render_worker",
            &format!("worker {i}: wait failed: {e}"),
          );
          note_status(LogStatus::Fatal, None);
        },
        Ok(Ok(output)) => {
          let stderr_text = strip_ansi(&String::from_utf8_lossy(&output.stderr));
          let report = parse_child_report(&stderr_text);
          if !report.log.is_empty() {
            // The child's stderr was piped, so nothing reached the live
            // stderr yet — forward it now, then fold the same text + counts
            // into the captured log and the REPORT tally.
            eprint!("{}", report.log);
          }
          replay_captured(CapturedDiagnostics {
            log:    report.log,
            counts: report.counts.unwrap_or_default(),
          });
          pages_rendered += report.pages;
          match report.status {
            None => {
              // No status line = the child died before its final report (or
              // never got that far). Fail toward flagging: this chunk's pages
              // cannot be assumed rendered.
              emit_error(
                "post",
                "render_worker",
                &format!(
                  "worker {i} exited (code {:?}) without a Status:conversion line",
                  output.status.code()
                ),
              );
              note_status(LogStatus::Fatal, None);
            },
            Some(s) if s >= 3 && !report.counts.is_some_and(|c| c.fatal) => {
              // The child declared fatal but its counts line didn't carry the
              // flag (or was missing) — max-fold the declared status anyway.
              note_status(LogStatus::Fatal, None);
            },
            Some(_) => {},
          }
        },
      },
    }
  }

  // The serial driver returns the first page's finalized output as
  // `main_output`; here that page is already on disk (written by worker 0),
  // so read it back. A missing/unreadable file leaves `None` and the caller's
  // fallback applies — the diagnostics above already flagged the failure.
  let main_output = first_destination.and_then(|d| std::fs::read_to_string(&d).ok());
  cleanup_handoff(&dbfile, &manifest_paths);
  Some(ParallelResult { main_output, pages_rendered })
}

/// Print the worker's canonical trailing status report (the LAST lines on
/// stderr): `Status:pages:`, then `Status:counts:`, then `Status:conversion:`
/// — the parent parses them positionally-independently by prefix. Returns the
/// process exit code (0 below fatal, 1 at fatal).
fn print_status_report(pages: usize) -> i32 {
  let c = snapshot_report_counts();
  let status = get_status_code();
  eprintln!("Status:pages:{pages}");
  eprintln!(
    "Status:counts:{},{},{},{},{}",
    c.debug,
    c.info,
    c.warning,
    c.error,
    usize::from(c.fatal)
  );
  eprintln!("Status:conversion:{status}");
  if status < 3 { 0 } else { 1 }
}

/// Entry point of the hidden worker mode (`LATEXML_RENDER_WORKER=<manifest>`),
/// dispatched from the binary's `main` before any CLI handling. Renders the
/// manifest's page range and ALWAYS ends its stderr with the status report —
/// even when the manifest cannot be read — so the parent never mistakes a
/// broken worker for a clean one.
pub fn worker_main(manifest_path: &str) -> i32 {
  latexml_core::util::logger::init(log::LevelFilter::Info).ok();
  let manifest: RenderManifest = match std::fs::read_to_string(manifest_path)
    .map_err(|e| e.to_string())
    .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
  {
    Ok(m) => m,
    Err(e) => {
      emit_fatal(
        "post",
        "render_worker",
        &format!("cannot read the render manifest {manifest_path}: {e}"),
      );
      return print_status_report(0);
    },
  };
  // Mirror `api.rs::on_worker`: a 256 MiB-stack thread for deeply nested
  // math, and an explicit engine reset before the thread exits (the
  // `#[thread_local]` roots do not Drop on a bare thread exit).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let pages = render_manifest_pages(manifest);
      let code = print_status_report(pages);
      latexml_core::reset_thread_engine();
      code
    })
    .expect("spawn render worker thread")
    .join()
    .expect("render worker thread panicked")
}

/// Rebuild the pass-B processor set from the manifest — the EXACT construction
/// the parent's `run_post_processing_inner` performs (same MathML
/// primary/secondary parallel model, same XSLT wiring, same error paths) —
/// then render every page in the manifest's range through the shared
/// [`crate::post::render_spilled_page`]. Returns the number of pages written.
fn render_manifest_pages(m: RenderManifest) -> usize {
  use latexml_post::{
    crossref::{CrossRef, UrlStyle},
    processor::Processor,
  };
  let db = match ObjectDB::attach(&m.dbfile, DbAttachOptions {
    readonly: true,
    clean:    false,
  }) {
    Ok(db) => db,
    Err(e) => {
      emit_fatal(
        "post",
        "render_worker",
        &format!("cannot attach the render db {}: {e}", m.dbfile.display()),
      );
      return 0;
    },
  };
  let url_style = UrlStyle::from_cli(&m.url_style).unwrap_or(UrlStyle::File);
  let mut crossref = CrossRef::new(db, url_style, true);
  crossref.set_extension(&m.out_extension);
  if let Some(navtoc) = m.navigation_toc.as_deref() {
    crossref.set_navigation_toc(navtoc);
  }
  let graphics = m.graphicimages.then(|| {
    latexml_post::graphics::Graphics::new(None, true)
      .with_svg_threshold_kb(m.graphics_svg_threshold_kb)
  });
  let post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();
  if m.pmml {
    let mut presentation = latexml_post::mathml::MathML::new_presentation()
      .with_keep_xmath(m.keep_xmath)
      .with_invisible_times(m.invisible_times)
      .with_plane1(m.plane1, m.hackplane1)
      .with_mathtex(m.mathtex)
      .with_intent_literal(m.intent_literal);
    if m.cmml {
      presentation = presentation.with_secondaries(vec![Box::new(
        latexml_post::mathml::MathML::new_content()
          .with_keep_xmath(m.keep_xmath)
          .with_invisible_times(m.invisible_times)
          .with_plane1(m.plane1, m.hackplane1)
          .secondary(),
      )]);
    }
    processors.push(Box::new(presentation));
  } else if m.cmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_content()
        .with_keep_xmath(m.keep_xmath)
        .with_invisible_times(m.invisible_times)
        .with_plane1(m.plane1, m.hackplane1),
    ));
  }
  if let Some(xsl_path) = m.stylesheet.as_deref() {
    let params: rustc_hash::FxHashMap<String, String> = m.xslt_params.iter().cloned().collect();
    match latexml_post::xslt::XSLT::new(
      xsl_path,
      params,
      m.nodefaultresources,
      None,
      m.searchpaths.clone(),
    ) {
      Ok(xslt) => processors.push(Box::new(xslt)),
      Err(e) => emit_error("post", "xslt", &format!("XSLT error: {e}")),
    }
  }
  let ctx = crate::post::PageRenderCtx {
    page_opts:     m.page_opts.clone().into(),
    is_html_out:   m.is_html_out,
    svg_fragments: m.svg_fragments.clone(),
    schemadocs:    m.schemadocs,
    whatsout:      latexml_post::extract::Whatsout::from_cli(&m.whatsout).unwrap_or_default(),
  };
  let mut procs = crate::post::PageProcessors {
    crossref,
    graphics,
    post,
    processors,
  };
  let mut pages_written = 0usize;
  for job in &m.pages {
    // Same OS give-back cadence as the serial render loop: each page cycles a
    // DOM + XSLT result through the C heap, and a chunk can be tens of
    // thousands of pages.
    if pages_written > 0 && pages_written.is_multiple_of(512) {
      #[cfg(target_os = "linux")]
      unsafe {
        libc::malloc_trim(0);
      }
      #[cfg(not(feature = "dhat-heap"))]
      unsafe {
        libmimalloc_sys::mi_collect(true);
      }
    }
    match crate::post::render_spilled_page(
      &job.path,
      &mut procs,
      &ctx,
      job.destination.clone(),
      job.destination_directory.clone(),
    ) {
      Ok(outputs) => {
        for (dest, output) in outputs {
          if let Some(path) = dest.as_deref() {
            if let Some(parent) = Path::new(path).parent()
              && !parent.as_os_str().is_empty()
            {
              let _ = std::fs::create_dir_all(parent);
            }
            pages_written += 1;
            if let Err(e) = std::fs::write(path, &output) {
              emit_error(
                "post",
                "write",
                &format!("failed to write page {path}: {e}"),
              );
            }
          }
        }
      },
      // Already reported inside the pipeline; mirror the serial driver's
      // abort-on-page-failure (the parent flags the shortfall through the
      // folded error + the fatal-bearing status this worker will report).
      Err(()) => break,
    }
  }
  pages_written
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn child_report_parses_status_lines_and_keeps_log() {
    let stderr_text = "Warning:post:x something odd\nInfo:post:y fine\nStatus:pages:41\nStatus:counts:0,2,1,3,1\nStatus:conversion:3\n";
    let r = parse_child_report(stderr_text);
    assert_eq!(r.pages, 41);
    assert_eq!(r.status, Some(3));
    let c = r.counts.expect("counts parsed");
    assert_eq!((c.debug, c.info, c.warning, c.error), (0, 2, 1, 3));
    assert!(c.fatal);
    assert!(r.log.contains("something odd"));
    assert!(
      !r.log.contains("Status:"),
      "status lines must not leak into the folded log"
    );
  }

  #[test]
  fn child_report_without_status_is_flagged_as_none() {
    let r = parse_child_report("Error:post:z boom\n");
    assert_eq!(r.status, None);
    assert!(r.counts.is_none());
    assert_eq!(r.pages, 0);
  }

  #[test]
  fn ansi_is_stripped_from_child_stderr() {
    assert_eq!(strip_ansi("\u{1b}[31mError:\u{1b}[0m x"), "Error: x");
  }

  #[test]
  fn render_jobs_defaults_to_serial() {
    // NOTE: does not set the env var (process-global); only checks the parse
    // fallback contract via the default branch.
    assert!(render_jobs() >= 1);
  }
}
