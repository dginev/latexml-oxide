//! Process-parallel page rendering for pass B of the post pipeline
//! (`docs/archive/STREAMING_POST_DESIGN_2026-07-06.md` §6).
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
//! never taken for success).
//!
//! Env knob: `LATEXML_RENDER_JOBS` (usize; `1` = the serial path). Unset, the
//! `latexml_oxide` binary renders a split of `MIN_PAGES_FOR_PARALLEL` pages
//! or more on min(available cores, `AUTO_JOBS_MAX`) workers; every other
//! embedder (the library, `cortex_worker`) stays serial. The pages are
//! byte-identical either way; a worker that fails to spawn, cannot set up, or
//! dies without its status report sends the whole document back to the serial
//! render at once, stopping the rest of the fleet (an Info; a Warning for a
//! worker killed by a signal, a crash).

use std::path::{Path, PathBuf};

use latexml_core::{
  Info, Warn,
  common::error::{
    LogStatus, ReportCounts, emit_error, get_status_code, note_status, snapshot_report_counts,
  },
  s,
  util::logger::{CapturedDiagnostics, replay_captured},
};
use latexml_post::object_db::{DbAttachOptions, ObjectDB};
use serde::{Deserialize, Serialize};

/// Fewer spilled pages than this and the db save + child spawn overhead beats
/// the parallel win — the serial path is taken regardless of the jobs knob.
pub(crate) const MIN_PAGES_FOR_PARALLEL: usize = 64;

/// The automatic fleet's ceiling: on Nasser's 2,400-problem book (9,713 pages)
/// 8 workers took the two-invocation run from 115 s to 79 s.
pub(crate) const AUTO_JOBS_MAX: usize = 8;

/// Set by a binary that serves the worker mode (`LATEXML_RENDER_WORKER`, see
/// [`worker_main`]): only it may spawn itself as a worker, so only it renders
/// in parallel without being asked.
static AUTO_JOBS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Let an unset `LATEXML_RENDER_JOBS` pick the fleet size (`render_jobs`).
pub fn enable_auto_jobs() { AUTO_JOBS.store(true, std::sync::atomic::Ordering::Relaxed); }

/// How many page-render worker processes pass B may spawn: an explicit
/// `LATEXML_RENDER_JOBS` wins (an unparsable or zero value is 1 = serial);
/// unset, min(available cores, `AUTO_JOBS_MAX`) where [`enable_auto_jobs`]
/// was called, else 1. Read once per conversion, so no hot-path caching is
/// needed.
pub(crate) fn render_jobs() -> usize {
  match std::env::var("LATEXML_RENDER_JOBS") {
    Ok(v) => v.parse::<usize>().ok().filter(|&n| n >= 1).unwrap_or(1),
    Err(_) if AUTO_JOBS.load(std::sync::atomic::Ordering::Relaxed) => {
      std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(AUTO_JOBS_MAX)
    },
    Err(_) => 1,
  }
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
#[derive(Serialize, Deserialize, Clone, Default)]
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
  pub unicodemath:               bool,
  pub intent_literal:            bool,
  pub stylesheet:                Option<String>,
  /// The fully-resolved XSLT parameter map (CSS/JS/LATEXML_VERSION/user
  /// overrides), captured after the parent computed it.
  pub xslt_params:               Vec<(String, String)>,
  pub default_resources:         bool,
  /// The resolved stylesheet/resource search paths the parent's `XSLT::new`
  /// received.
  pub searchpaths:               Vec<String>,
  /// Run the SVG processor before MathML (the parent's `svg`).
  pub svg:                       bool,
  /// Serialize the pages as HTML (Perl `is_html`).
  pub is_html_out:               bool,
  pub schemadocs:                bool,
  /// [`latexml_post::extract::Whatsout`] as its canonical CLI tag
  /// (`as_cli`/`from_cli` round-trip).
  pub whatsout:                  String,
  pub page_opts:                 PageOpts,
  pub pages:                     Vec<PageJob>,
  /// This worker's share of the run's memory budget, in KiB (0 = none): its
  /// watchdog's ceiling, so the fleet stays within `--max-memory`.
  #[serde(default)]
  pub max_rss_kb:                u64,
  /// The parent's STATE values the render reads: a worker is a fresh process, whose STATE
  /// holds none (MathML's `<mpadded>` widths read `NOMINAL_FONT_SIZE`).
  #[serde(default)]
  pub post_values:               latexml_core::PostStateValues,
  /// The parent's stderr verbosity (`LevelFilter` name: `OFF` under `--quiet`, `DEBUG` under `--verbose`). The
  /// worker logs at it, never below `Info`, so its folded log is the serial render's.
  #[serde(default)]
  pub log_level:                 String,
}

/// The parent-side result of a parallel render in which workers were actually
/// spawned (successfully or not — failures are already folded as diagnostics).
pub(crate) struct ParallelResult {
  /// The first page's finalized output, read back from its destination file —
  /// the same content the serial driver would have returned as `main_output`.
  pub(crate) main_output:    Option<String>,
  /// Total pages written, summed from the workers' `Status:pages:` lines.
  pub(crate) pages_rendered: usize,
  /// A worker's MathML/XSLT step died (`Status:discard:1`): the pages written are removed and
  /// the render leaves no output (#782).
  pub(crate) discarded:      bool,
  /// The pages the workers wrote, folded in chunk order — the serial render's page order. For an
  /// abandoned fleet, every page it may have written ([`pages_the_fleet_may_have_written`]).
  pub(crate) written:        Vec<String>,
  /// A worker failed, so nothing was folded and the document renders serially. A page's Fatal
  /// there removes `written` too (#782); a fuse or timeout stop keeps only the pages it rewrote.
  pub(crate) abandoned:      bool,
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
  log:          String,
  counts:       Option<ReportCounts>,
  status:       Option<usize>,
  pages:        usize,
  /// `Status:discard:1`: a MathML/XSLT step died, so no page of the run is kept (#782).
  discard:      bool,
  /// `Status:written:<path>`: the pages this worker wrote, the only ones a discard removes.
  written:      Vec<String>,
  /// `Status:setup-failed:<why>`: the worker rendered nothing (`print_setup_failure`).
  setup_failed: Option<String>,
}

fn parse_child_report(stderr_text: &str) -> ChildReport {
  let mut log = String::new();
  let mut counts = None;
  let mut status = None;
  let mut pages = 0usize;
  let mut discard = false;
  let mut written = Vec::new();
  let mut setup_failed = None;
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
    } else if let Some(rest) = line.strip_prefix("Status:discard:") {
      discard = rest.trim() == "1";
    } else if let Some(path) = line.strip_prefix("Status:written:") {
      written.push(path.to_string());
    } else if let Some(why) = line.strip_prefix("Status:setup-failed:") {
      setup_failed = Some(why.to_string());
    } else {
      log.push_str(line);
      log.push('\n');
    }
  }
  ChildReport {
    log,
    counts,
    status,
    pages,
    discard,
    written,
    setup_failed,
  }
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

/// A joined worker: its parsed report and exit status, or why it could not be waited for.
type Joined = Result<(ChildReport, std::process::ExitStatus), String>;

fn join_worker(p: Pending) -> Joined {
  match p {
    Pending::Failed(e) => Err(e),
    Pending::Spawned(handle, _) => match handle.join() {
      Err(_) => Err("drain thread panicked".to_string()),
      Ok(Err(e)) => Err(format!("wait failed: {e}")),
      Ok(Ok(output)) => {
        let stderr_text = strip_ansi(&String::from_utf8_lossy(&output.stderr));
        Ok((parse_child_report(&stderr_text), output.status))
      },
    },
  }
}

/// Why a worker sends the document to the serial render.
struct WorkerFailure {
  why:       String,
  /// Ended by a signal (no exit code): a crash, not one of the worker's own exits.
  by_signal: bool,
}

/// The failure of a joined worker that did not render its range, `None` for one that reported. A death names its
/// signal and ends with the worker's last stderr lines (its `Fatal:oom:rss`, a panic), which are not folded.
fn worker_failure(r: &Joined) -> Option<WorkerFailure> {
  let (why, by_signal) = match r {
    Err(e) => (e.clone(), false),
    Ok((report, _)) if let Some(why) = &report.setup_failed => {
      (format!("setup failed: {why}"), false)
    },
    Ok((report, status)) if report.status.is_none() => {
      let by_signal = status.code().is_none();
      let why = format!(
        "{} without a Status:conversion line",
        describe_death(status)
      );
      match stderr_tail(&report.log) {
        Some(tail) => (format!("{why}; its stderr ended: {tail}"), by_signal),
        None => (why, by_signal),
      }
    },
    Ok(_) => return None,
  };
  Some(WorkerFailure { why, by_signal })
}

/// The last three non-blank lines of a failed worker's stderr, on one line (each cut at 300 bytes), so a line the
/// worker began with `Fatal:` stays inside the parent's message.
fn stderr_tail(log: &str) -> Option<String> {
  let mut lines: Vec<&str> = log
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .rev()
    .take(3)
    .map(|l| &l[..l.floor_char_boundary(300)])
    .collect();
  lines.reverse();
  (!lines.is_empty()).then(|| lines.join(" | "))
}

/// How a worker ended: `exited (code 137)`, or `killed by SIGSEGV (signal 11)`.
fn describe_death(status: &std::process::ExitStatus) -> String {
  match status.code() {
    Some(code) => format!("exited (code {code})"),
    None => format!("killed by {}", describe_signal(status)),
  }
}

/// The signal by name and number (`SIGSEGV (signal 11)`), and whether it dumped core.
#[cfg(unix)]
fn describe_signal(status: &std::process::ExitStatus) -> String {
  use std::os::unix::process::ExitStatusExt;
  let Some(n) = status.signal() else {
    return "a signal".to_string();
  };
  let name = match n {
    libc::SIGSEGV => "SIGSEGV",
    libc::SIGBUS => "SIGBUS",
    libc::SIGABRT => "SIGABRT",
    libc::SIGILL => "SIGILL",
    libc::SIGFPE => "SIGFPE",
    libc::SIGKILL => "SIGKILL",
    libc::SIGTERM => "SIGTERM",
    libc::SIGINT => "SIGINT",
    libc::SIGHUP => "SIGHUP",
    libc::SIGQUIT => "SIGQUIT",
    libc::SIGPIPE => "SIGPIPE",
    libc::SIGTRAP => "SIGTRAP",
    libc::SIGSYS => "SIGSYS",
    libc::SIGXCPU => "SIGXCPU",
    libc::SIGXFSZ => "SIGXFSZ",
    _ => return format!("signal {n}"),
  };
  let core = if status.core_dumped() {
    ", core dumped"
  } else {
    ""
  };
  format!("{name} (signal {n}{core})")
}

#[cfg(not(unix))]
fn describe_signal(_: &std::process::ExitStatus) -> String { "a signal".to_string() }

/// A worker's expected peak resident set: it decodes the whole db at attach, and its pages add
/// little. Measured VmHWM, Nasser's book: 95 MB on the 600-problem part (11 MB db), 260 MB on the
/// 2,400-problem part (45 MB db), i.e. ~42 MB + 4.85 x db; this adds a quarter's margin. The old
/// `max(3 x db, 256 MB)` sized k2400 workers below their start, and one died (exit 137).
fn worker_estimate_bytes(db_bytes: u64) -> u64 {
  db_bytes.saturating_mul(6).saturating_add(64 << 20)
}

/// Pure fleet-sizing calculation for [`parallel_render`].
///
/// `soft_cap` is the cooperative RSS fuse (`stomach::resolve_rss_cap`, already
/// 3/4 of `--max-memory` via `soft_cap_from_ceiling`) — it must NOT be scaled
/// by 3/4 a second time. Returns `(jobs, affordable, budget, per_worker, max_rss_kb)`.
fn compute_fleet_budget(
  available_mem: Option<u64>,
  soft_cap: Option<u64>,
  parent_rss_bytes: u64,
  db_bytes: u64,
  requested_jobs: usize,
  total_pages: usize,
) -> (usize, usize, u64, u64, u64) {
  let per_worker = worker_estimate_bytes(db_bytes);
  let mut budget = available_mem.unwrap_or(u64::MAX);
  if let Some(fuse) = soft_cap {
    budget = budget.min(fuse.saturating_sub(parent_rss_bytes));
  }
  let affordable = (budget / per_worker) as usize;
  let jobs = requested_jobs.min(total_pages).min(affordable);
  let max_rss_kb = if budget != u64::MAX && jobs >= 2 {
    budget / jobs as u64 / 1024
  } else {
    0
  };
  (jobs, affordable, budget, per_worker, max_rss_kb)
}

/// The pages an abandoned fleet may have written, for the serial render to remove if it ends in a
/// Fatal (#782): a worker that reported (or could not set up) names its pages, and one that died
/// before its `Status:written:` lines may have written any page of its range. The serial render
/// rewrites them; one that stops (the fuse, the timeout, a page's Error) removes those it did not.
fn pages_the_fleet_may_have_written(joined: &[Joined], ranges: &[Vec<String>]) -> Vec<String> {
  joined
    .iter()
    .zip(ranges)
    .flat_map(|(r, range)| match r {
      Ok((report, _)) if report.status.is_some() || report.setup_failed.is_some() => {
        report.written.clone()
      },
      Err(e) if e.starts_with("worker spawn failed") => Vec::new(),
      _ => range.clone(),
    })
    .collect()
}

/// Spawn `jobs` page-range workers over `pages` and fold their results.
///
/// Returns `None` when setup failed or the fleet was declined, and an `abandoned` result when a
/// worker failed to spawn, could not set up, or died without its status report (an Info, a Warning
/// when a signal killed it) — the caller falls
/// back to the serial render, which is still fully correct since the workers
/// keep the spilled pages. Only a timeout breach folds the killed workers as an
/// Error + FATAL status (fail toward flagging, never silent success).
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
  // 64 MiB + 6x the on-disk db (`worker_estimate_bytes`), and size the
  // fleet from the smaller of MemAvailable and the
  // distance to this run's own memory fuse. Degrade to fewer workers — or
  // decline to engage — rather than let the kernel choose a victim.
  crate::heap_trim::release_free_heap();
  let db_bytes = std::fs::metadata(&dbfile).map(|m| m.len()).unwrap_or(0);
  let parent_rss_bytes = latexml_core::watchdog::process_rss_kb()
    .unwrap_or(0)
    .saturating_mul(1024);
  let (jobs, affordable, budget, per_worker, max_rss_kb) = compute_fleet_budget(
    latexml_core::watchdog::available_memory_bytes(),
    latexml_core::stomach::resolve_rss_cap(),
    parent_rss_bytes,
    db_bytes,
    jobs,
    total_pages,
  );
  // Each worker's watchdog holds it to its share, so parent + fleet stay
  // under the fuse; one that outgrows it dies and the document renders serially.
  manifest.max_rss_kb = max_rss_kb;
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
  let mut ranges: Vec<Vec<String>> = Vec::with_capacity(jobs);
  for (i, chunk) in pages.chunks(chunk_size).enumerate() {
    ranges.push(chunk.iter().filter_map(|p| p.destination.clone()).collect());
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
      discarded:      false,
      written:        Vec::new(),
      abandoned:      false,
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
        // Register with the watchdog so timeout/OOM exits never orphan the
        // worker pool across Unix and Windows. Moved into the drain closure so
        // each worker's PID unregisters immediately when `wait_with_output`
        // reaps it (never outliving an early-finishing worker).
        let kill_guard = latexml_core::watchdog::kill_pid_on_exit(pid, false);
        // Each child gets a drain thread calling `wait_with_output` — piped
        // stderr MUST be consumed while the parent polls, or a chatty child
        // blocks on a full pipe and the poll below never sees it exit.
        let handle = std::thread::Builder::new()
          .name(format!("render-worker-drain-{pid}"))
          .spawn(move || {
            let _kill_guard = kill_guard;
            child.wait_with_output()
          });
        match handle {
          Ok(h) => pending.push(Pending::Spawned(h, pid)),
          Err(e) => {
            // `child` and `kill_guard` were moved into the closure and dropped
            // on spawn failure, unregistering the PID; kill it explicitly so we
            // never leak a live worker.
            latexml_core::watchdog::kill_pid(pid, false);
            pending.push(Pending::Failed(format!("drain thread spawn failed: {e}")));
          },
        }
      },
      Err(e) => pending.push(Pending::Failed(format!("worker spawn failed: {e}"))),
    }
  }

  // Poll (rather than block) so the parent's cooperative timeout keeps
  // running between waits, and join each worker as it ends: the first one
  // that fails sends the document to the serial render at once, and a breach
  // ends the run, so either way the rest of the fleet is killed rather than
  // waited for (Nasser k2400: a worker's death was acted on only after its
  // seven siblings finished).
  let mut slots: Vec<Option<Pending>> = pending.into_iter().map(Some).collect();
  let mut joined: Vec<Option<Joined>> = slots.iter().map(|_| None).collect();
  let failed: Option<(usize, WorkerFailure)> = loop {
    for (slot, out) in slots.iter_mut().zip(joined.iter_mut()) {
      if slot
        .as_ref()
        .is_some_and(|p| !matches!(p, Pending::Spawned(h, _) if !h.is_finished()))
      {
        *out = slot.take().map(join_worker);
      }
    }
    let failure = joined
      .iter()
      .enumerate()
      .find_map(|(i, r)| r.as_ref().and_then(worker_failure).map(|f| (i, f)));
    if failure.is_some() || slots.iter().all(Option::is_none) {
      break failure;
    }
    if let Err(e) = latexml_core::stomach::check_timeout() {
      e.log_fatal();
      break None;
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  };
  for slot in slots.iter().flatten() {
    if let Pending::Spawned(h, pid) = slot
      && !h.is_finished()
    {
      latexml_core::watchdog::kill_pid(*pid, false);
    }
  }
  for (slot, out) in slots.into_iter().zip(joined.iter_mut()) {
    if let Some(p) = slot {
      *out = Some(join_worker(p));
    }
  }

  // Every worker is joined before any child log is folded: one that could not
  // run, or died without its status report (a crash, its memory share
  // exceeded, a broken pipe), sends the whole range to the serial render, so
  // nothing is counted twice. A worker's own diagnostics, Fatal included, are
  // the document's and fold as they are. After a timeout breach the killed
  // workers fold as failures (the Fatal is already logged).
  let mut joined: Vec<Joined> = joined
    .into_iter()
    .map(|r| r.unwrap_or_else(|| Err("not joined".to_string())))
    .collect();
  if let Some((i, failure)) = failed {
    let message = s!(
      "parallel render abandoned (worker {}: {}) — rendering serially",
      i,
      failure.why
    );
    // A worker killed by a signal crashed (or the kernel's OOM killer took
    // it): the serial render recovers the pages, but the crash is counted. Its
    // own watchdog's memory exit and a setup failure are expected outcomes.
    if failure.by_signal {
      Warn!("post", "parallel-render", message);
    } else {
      Info!("post", "parallel-render", message);
    }
    cleanup_handoff(&dbfile, &manifest_paths);
    return Some(ParallelResult {
      main_output:    None,
      pages_rendered: 0,
      discarded:      false,
      written:        pages_the_fleet_may_have_written(&joined, &ranges),
      abandoned:      true,
    });
  }

  // Fold IN CHUNK ORDER (deterministic log/tally, mirroring the graphics
  // worker fold): forward each child's stderr, replay its log + counts into
  // the parent LOG_BUFFER/REPORT, and flag anything that did not report.
  let mut pages_rendered = 0usize;
  let mut discard = false;
  let mut written: Vec<String> = Vec::new();
  for (i, r) in joined.drain(..).enumerate() {
    match r {
      Err(e) => {
        emit_error("post", "render_worker", &format!("worker {i}: {e}"));
        note_status(LogStatus::Fatal, None);
      },
      Ok((report, exit_status)) => {
        if !report.log.is_empty() {
          // The child's stderr was piped, so nothing reached the live
          // stderr yet — forward it now, then fold the same text + counts
          // into the captured log and the REPORT tally.
          eprint!("{}", stderr_share(&report.log));
        }
        replay_captured(CapturedDiagnostics {
          log:    report.log,
          counts: report.counts.unwrap_or_default(),
        });
        pages_rendered += report.pages;
        discard |= report.discard;
        written.extend(report.written);
        match report.status {
          None => {
            // No status line = the child died before its final report (or
            // never got that far). Fail toward flagging: this chunk's pages
            // cannot be assumed rendered.
            emit_error(
              "post",
              "render_worker",
              &format!(
                "worker {i} {} without a Status:conversion line",
                describe_death(&exit_status)
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
    }
  }

  // The serial driver returns the first page's finalized output as
  // `main_output`; here that page is already on disk (written by worker 0),
  // so read it back. A missing/unreadable file leaves `None` and the caller's
  // fallback applies — the diagnostics above already flagged the failure.
  if discard {
    // Only this run's pages: a stale page of an earlier run in the destination stays.
    crate::post::remove_pages(&written);
    cleanup_handoff(&dbfile, &manifest_paths);
    return Some(ParallelResult {
      main_output:    None,
      pages_rendered: 0,
      discarded:      true,
      written:        Vec::new(),
      abandoned:      false,
    });
  }
  let main_output = first_destination.and_then(|d| std::fs::read_to_string(&d).ok());
  cleanup_handoff(&dbfile, &manifest_paths);
  Some(ParallelResult {
    main_output,
    pages_rendered,
    discarded: false,
    written,
    abandoned: false,
  })
}

/// Print the worker's canonical trailing status report (the LAST lines on
/// stderr): `Status:pages:`, `Status:written:` per page, then `Status:counts:`, then
/// `Status:conversion:` — the parent parses them positionally-independently by prefix. Returns
/// the process exit code (0 below fatal, 1 at fatal).
fn print_status_report(written: &[String], discard: bool) -> i32 {
  let c = snapshot_report_counts();
  let status = get_status_code();
  eprintln!("Status:pages:{}", written.len());
  for path in written {
    eprintln!("Status:written:{path}");
  }
  if discard {
    eprintln!("Status:discard:1");
  }
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

/// The lines of a worker's log the parent's stderr shows: its records at the parent's verbosity (`--quiet` hides
/// every one, `Error`/`Fatal` included; only `--verbose` shows `Debug`), each with its continuation lines.
fn stderr_share(log: &str) -> String {
  stderr_share_at(log, latexml_core::util::logger::stderr_admits)
}

/// [`stderr_share`] at the verbosity `admits`. A line that opens no record is either a progress note (an `Info`)
/// or an unindented continuation of a multi-line message, which the log cannot tell apart: it is shown when either
/// would be, so a shown record keeps every line (a note after it may show under `--quiet`).
fn stderr_share_at(log: &str, admits: impl Fn(log::Level) -> bool) -> String {
  use log::Level;
  let mut shown = String::with_capacity(log.len());
  let mut admitted = true;
  let mut record_admitted = false;
  for line in log.split_inclusive('\n') {
    let level = [
      ("Error:", Level::Error),
      ("Fatal:", Level::Error),
      ("Warning:", Level::Warn),
      ("Info:", Level::Info),
      ("Debug:", Level::Debug),
      ("Trace:", Level::Trace),
    ]
    .into_iter()
    .find_map(|(head, level)| line.starts_with(head).then_some(level));
    match level {
      Some(level) => {
        admitted = admits(level);
        record_admitted = admitted;
      },
      // An indented line (`\tat …`, a blank line) goes with the line before it.
      None if line.starts_with(char::is_whitespace) => {},
      None => admitted = record_admitted || admits(Level::Info),
    }
    if admitted {
      shown.push_str(line);
    }
  }
  shown
}

/// A worker that could not set up (its manifest or db unreadable) has rendered
/// nothing and its failure is not the document's: no status report, so the
/// parent renders serially, and this line names why (an Info there).
fn print_setup_failure(why: &str) -> i32 {
  eprintln!("Status:setup-failed:{}", why.replace('\n', " "));
  1
}

/// Entry point of the hidden worker mode (`LATEXML_RENDER_WORKER=<manifest>`),
/// dispatched from the binary's `main` before any CLI handling. Renders the
/// manifest's page range and ends its stderr with the status report, or with
/// the setup-failure line when it could not start, so the parent never
/// mistakes a broken worker for a clean one.
pub fn worker_main(manifest_path: &str) -> i32 {
  latexml_core::util::logger::init(log::LevelFilter::Info).ok();
  let manifest: RenderManifest = match std::fs::read_to_string(manifest_path)
    .map_err(|e| e.to_string())
    .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
  {
    Ok(m) => m,
    Err(e) => {
      return print_setup_failure(&format!(
        "cannot read the render manifest {manifest_path}: {e}"
      ));
    },
  };
  // Mirror `api.rs::on_worker`: a 256 MiB-stack thread for deeply nested
  // math, and an explicit engine reset before the thread exits (the
  // `#[thread_local]` roots do not Drop on a bare thread exit).
  #[cfg(feature = "test-utils")]
  let first_worker = manifest_path.ends_with("render-manifest-0.json");
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      #[cfg(feature = "test-utils")]
      match test_fault().as_deref() {
        None | Some("attach" | "unreported" | "rss") => {},
        Some("signal") if first_worker => {
          latexml_core::watchdog::kill_pid(std::process::id(), false)
        },
        Some("signal") => std::thread::sleep(std::time::Duration::from_secs(600)),
        Some(_) => latexml_core::watchdog::exit_now(70),
      }
      // Hold this worker to its share of the run's memory budget, and align
      // `heap_trim::step_kb()` with that ceiling instead of the 4.5 GB default.
      // The share is the worker's HARD (watchdog) ceiling, where the parent's cap
      // is the soft 3/4 fuse: a `check_timeout` added to the page loop must not
      // read it as the cooperative fuse.
      if manifest.max_rss_kb > 0 {
        latexml_core::stomach::set_memory_cap(Some(manifest.max_rss_kb.saturating_mul(1024)));
      }
      manifest.post_values.restore();
      let level = manifest.log_level.parse().unwrap_or(log::LevelFilter::Info);
      latexml_core::util::logger::init(level.max(log::LevelFilter::Info)).ok();
      let watchdog_kb = manifest.max_rss_kb;
      #[cfg(feature = "test-utils")]
      let watchdog_kb = if test_fault().as_deref() == Some("rss") {
        1
      } else {
        watchdog_kb
      };
      let _watchdog =
        (watchdog_kb > 0).then(|| latexml_core::watchdog::Watchdog::with_limits(0, watchdog_kb));
      let code = match render_manifest_pages(manifest) {
        #[cfg(feature = "test-utils")]
        Ok(_) if test_fault().as_deref() == Some("unreported") => {
          latexml_core::watchdog::exit_now(70)
        },
        Ok((written, discard)) => print_status_report(&written, discard),
        Err(why) => print_setup_failure(&why),
      };
      latexml_core::reset_thread_engine();
      code
    })
    .expect("spawn render worker thread")
    .join()
    .expect("render worker thread panicked")
}

/// Test hook `LATEXML_RENDER_WORKER_TEST_ABORT` (guard: 120_midscale_streaming):
/// `attach` fails the worker's db attach; `signal` SIGKILLs worker 0 (a crash)
/// and holds its siblings for ten minutes, unless the parent stops them;
/// `unreported` renders the range and exits before the status report, as a
/// worker killed late would; `rss` holds the worker's watchdog to 1 KB, so it
/// exits 137 at its first poll, while the render attaches; any other value
/// exits the worker before it renders.
#[cfg(feature = "test-utils")]
fn test_fault() -> Option<String> { std::env::var("LATEXML_RENDER_WORKER_TEST_ABORT").ok() }

/// Rebuild the pass-B processor set from the manifest — the EXACT construction
/// the parent's `run_post_processing_inner` performs (same MathML
/// primary/secondary parallel model, same XSLT wiring, same error paths) —
/// then render every page in the manifest's range through the shared
/// [`crate::post::render_spilled_page`]. Returns the pages written, and whether a Fatal discards them;
/// `Err` = the worker could not set up (see `print_setup_failure`).
fn render_manifest_pages(m: RenderManifest) -> Result<(Vec<String>, bool), String> {
  use latexml_post::{
    crossref::{CrossRef, UrlStyle},
    processor::Processor,
  };
  #[cfg(feature = "test-utils")]
  let m = RenderManifest {
    dbfile: if test_fault().as_deref() == Some("attach") {
      m.dbfile.with_extension("missing")
    } else {
      m.dbfile
    },
    ..m
  };
  let db = match ObjectDB::attach(&m.dbfile, DbAttachOptions {
    readonly: true,
    clean:    false,
  }) {
    Ok(db) => db,
    Err(e) => {
      return Err(format!(
        "cannot attach the render db {}: {e}",
        m.dbfile.display()
      ));
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
  // Same chain as the in-process renderer (`post.rs`): SVG on the live page
  // DOM before MathML/XSLT for the HTML formats (Perl `LaTeXML.pm` order).
  if m.svg {
    processors.push(Box::new(latexml_post::svg::SVG::new()));
  }
  let math_formats = crate::post::MathFormats {
    pmml:            m.pmml,
    cmml:            m.cmml,
    unicodemath:     m.unicodemath,
    keep_xmath:      m.keep_xmath,
    invisible_times: m.invisible_times,
    plane1:          m.plane1,
    hackplane1:      m.hackplane1,
    mathtex:         m.mathtex,
    intent_literal:  m.intent_literal,
  };
  if let Some(math) = math_formats.processor() {
    processors.push(math);
  }
  if let Some(xsl_path) = m.stylesheet.as_deref() {
    let params: rustc_hash::FxHashMap<String, String> = m.xslt_params.iter().cloned().collect();
    match latexml_post::xslt::XSLT::new(
      xsl_path,
      params,
      !m.default_resources,
      None,
      m.searchpaths.clone(),
    ) {
      Ok(xslt) => processors.push(Box::new(xslt)),
      Err(e) => emit_error("post", "xslt", &format!("XSLT error: {e}")),
    }
  }
  let ctx = crate::post::PageRenderCtx {
    page_opts:      m.page_opts.clone().into(),
    is_html_out:    m.is_html_out,
    schemadocs:     m.schemadocs,
    whatsout:       latexml_post::extract::Whatsout::from_cli(&m.whatsout).unwrap_or_default(),
    remove_spilled: false,
  };
  let mut procs = crate::post::PageProcessors {
    crossref,
    graphics,
    post,
    processors,
  };
  let mut written: Vec<String> = Vec::new();
  let mut made_dirs = latexml_post::writer::MadeDirs::default();
  for job in &m.pages {
    // Same OS give-back cadence as the serial render loop: each page cycles a
    // DOM + XSLT result through the C heap, and a chunk can be tens of
    // thousands of pages.
    if !written.is_empty() && written.len().is_multiple_of(crate::heap_trim::SAMPLE_PAGES) {
      crate::heap_trim::sample();
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
            let _ = made_dirs.ensure_parent(path);
            written.push(path.to_string());
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
      Err(crate::post::PageFailure::Error) => break,
      // The parent removes every page of the run (#782).
      Err(crate::post::PageFailure::Fatal) => return Ok((written, true)),
    }
  }
  Ok((written, false))
}

#[cfg(test)]
mod tests {
  use super::*;

  /// At the `Warn` level a Warning keeps its unindented continuation lines (a multi-line message); the notes after an
  /// Info record stay hidden. The default verbosity shows every line, `--quiet` none (Perl `$VERBOSITY < 0`).
  #[test]
  fn stderr_share_keeps_a_records_continuation_lines() {
    let log = "note before\nInfo:a:x one\nMathML[Presentation] S1.html processing\nWarning:b:y first line\nsecond \
               line\n\tat loc\nInfo:c:z two\n\tIn here\npost-processing\nError:d:w bad\n";
    assert_eq!(
      stderr_share_at(log, |level| level <= log::Level::Warn),
      "Warning:b:y first line\nsecond line\n\tat loc\nError:d:w bad\n"
    );
    assert_eq!(stderr_share_at(log, |level| level <= log::Level::Info), log);
    assert_eq!(stderr_share_at(log, |_| false), "");
  }

  #[test]
  fn child_report_parses_status_lines_and_keeps_log() {
    let stderr_text = "Warning:post:x something odd\nInfo:post:y fine\nStatus:pages:41\nStatus:written:/d/S1.html\nStatus:discard:1\nStatus:counts:0,2,1,3,1\nStatus:conversion:3\n";
    let r = parse_child_report(stderr_text);
    assert_eq!(r.pages, 41);
    assert_eq!(r.written, vec!["/d/S1.html"]);
    assert!(r.discard);
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

  /// The parent's STATE values reach the worker's STATE through the manifest's JSON: the raised error cap
  /// (tikz's 1000) as the nominal font size (the a0poster guard in cluster_cli).
  #[test]
  fn manifest_carries_the_parents_state_values() {
    let manifest = RenderManifest {
      post_values: latexml_core::PostStateValues {
        nominal_font_size: Some(24.88),
        max_errors:        Some(1000),
      },
      ..RenderManifest::default()
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let read: RenderManifest = serde_json::from_str(&json).unwrap();
    read.post_values.restore();
    assert_eq!(
      latexml_core::state::try_lookup_int("MAX_ERRORS"),
      Some(1000)
    );
    assert_eq!(
      latexml_core::state::lookup_float("NOMINAL_FONT_SIZE").map(|f| f.0),
      Some(24.88)
    );
  }

  #[test]
  fn child_report_parses_a_setup_failure() {
    let r = parse_child_report("Status:setup-failed:cannot attach the render db /d/render.db\n");
    assert_eq!(
      r.setup_failed.as_deref(),
      Some("cannot attach the render db /d/render.db")
    );
    assert_eq!(r.status, None);
    assert!(r.log.is_empty());
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

  #[test]
  fn fleet_budget_uses_soft_cap_without_double_discount() {
    const MIB: u64 = 1024 * 1024;
    // `--max-memory=8192` → soft_cap = 6,144 MiB (3/4 of 8 GiB).
    // With 4,300 MiB parent RSS and 80 MiB db (per_worker = 64 + 6 x 80 = 544 MiB):
    // remaining fuse headroom is 1,844 MiB → 3 workers affordable.
    // (With the old double 3/4 bug, fuse dropped to 4,608 MiB → 308 MiB → 0 workers.)
    let soft_cap = latexml_core::stomach::soft_cap_from_ceiling(8192);
    assert_eq!(soft_cap, 6144 * MIB);
    let (jobs, affordable, budget, per_worker, max_rss_kb) = compute_fleet_budget(
      Some(16_000 * MIB),
      Some(soft_cap),
      4300 * MIB,
      80 * MIB,
      8,
      1000,
    );
    assert_eq!(budget, 1844 * MIB);
    assert_eq!(per_worker, 544 * MIB);
    assert_eq!(affordable, 3);
    assert_eq!(jobs, 3);
    assert_eq!(max_rss_kb, (1844 * MIB) / 3 / 1024);
  }

  #[test]
  fn fleet_shares_cover_the_measured_worker_peak() {
    const MIB: u64 = 1024 * 1024;
    // Nasser k2400 under `--max-memory=4900`: a 45 MB db, the parent at ~860 MB when the fleet
    // spawns, a worker's measured VmHWM 260 MB. Every share must hold that peak.
    let (jobs, _, _, per_worker, max_rss_kb) = compute_fleet_budget(
      Some(16_000 * MIB),
      Some(latexml_core::stomach::soft_cap_from_ceiling(4900)),
      860 * MIB,
      45 * MIB,
      8,
      9713,
    );
    assert_eq!(per_worker, 334 * MIB);
    assert_eq!(jobs, 8);
    assert!(max_rss_kb * 1024 >= 260 * MIB, "share {max_rss_kb} KiB");
  }

  #[cfg(unix)]
  #[test]
  fn only_a_signal_death_is_a_crash() {
    use std::os::unix::process::ExitStatusExt;
    let report = || parse_child_report("Info:post:x partial\n");
    let killed = worker_failure(&Ok((report(), std::process::ExitStatus::from_raw(9)))).unwrap();
    assert!(killed.by_signal);
    assert_eq!(
      killed.why,
      "killed by SIGKILL (signal 9) without a Status:conversion line; its stderr ended: Info:post:x partial"
    );
    // A crash names its signal and the core dump (wait status 11 | 0x80).
    let crashed = worker_failure(&Ok((
      parse_child_report(""),
      std::process::ExitStatus::from_raw(11 | 0x80),
    )))
    .unwrap();
    assert!(crashed.by_signal);
    assert_eq!(
      crashed.why,
      "killed by SIGSEGV (signal 11, core dumped) without a Status:conversion line"
    );
    // The worker's own memory exit is not a crash; its last three stderr lines end the message, on one line.
    let capped = worker_failure(&Ok((
      parse_child_report(
        "Info:post:a one\nInfo:post:b two\n\nWarning:post:c three\nFatal:oom:rss latexml-oxide: resident \
         memory 900MB exceeded the 800MB ceiling\nStatus:pages:0\n",
      ),
      std::process::ExitStatus::from_raw(137 << 8),
    )))
    .unwrap();
    assert!(
      !capped.by_signal,
      "the worker's own memory exit is not a crash"
    );
    assert_eq!(
      capped.why,
      "exited (code 137) without a Status:conversion line; its stderr ended: Info:post:b two | Warning:post:c \
       three | Fatal:oom:rss latexml-oxide: resident memory 900MB exceeded the 800MB ceiling"
    );
    let done = parse_child_report("Status:pages:1\nStatus:conversion:0\n");
    assert!(worker_failure(&Ok((done, std::process::ExitStatus::from_raw(0)))).is_none());
  }

  #[test]
  fn an_abandoned_fleet_names_every_page_it_may_have_written() {
    let report = |text: &str| parse_child_report(text);
    let status = std::process::ExitStatus::default();
    let joined: Vec<Joined> = vec![
      // Reported: only the pages it names (not S3, a stale page of an earlier run).
      Ok((
        report("Status:written:S1\nStatus:pages:1\nStatus:conversion:0\n"),
        status,
      )),
      // Died before its report: any page of its range.
      Ok((report("Info:post:x partial\n"), status)),
      // Could not set up: none.
      Ok((report("Status:setup-failed:no db\n"), status)),
      Err("worker spawn failed: EAGAIN".to_string()),
      Err("wait failed: EINTR".to_string()),
    ];
    let range = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    let ranges = vec![
      range(&["S1", "S3"]),
      range(&["S4", "S5"]),
      range(&["S6"]),
      range(&["S7"]),
      range(&["S8"]),
    ];
    assert_eq!(pages_the_fleet_may_have_written(&joined, &ranges), vec![
      "S1", "S4", "S5", "S8"
    ]);
  }
}
