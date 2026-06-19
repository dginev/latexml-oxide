//! CorTeX Worker for latexml-oxide
//!
//! Implements the pericortex Worker trait to integrate latexml_oxide
//! with the CorTeX distributed processing framework.
//!
//! Two modes:
//! - Worker mode (default): connects to CorTeX dispatcher via ZMQ
//! - Standalone mode (--standalone): single ZIP-to-ZIP conversion

#![feature(alloc_error_hook)]

use std::{
  alloc::{Layout, set_alloc_error_hook},
  borrow::Cow,
  cell::{Cell, RefCell},
  error::Error,
  ffi::OsString,
  fs::{self, File},
  io::{Read, Write},
  panic::AssertUnwindSafe,
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

/// Per-process allocator. Default is **mimalloc** (lock-free per-thread heaps,
/// avoids glibc's arena-mutex contention). Building with `--features jemalloc`
/// swaps in **jemalloc**, whose decay-based background purging returns freed
/// pages to the OS so a long-lived worker's RSS tracks the *current* paper
/// rather than retaining the high-water of the heaviest paper it has processed —
/// the disposition a persistent one-conversion-per-process worker needs (tune
/// the decay via `_RJEM_MALLOC_CONF`, set on children by `run_harness`).
#[cfg(not(feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
use std::{process, rc::Rc};

use clap::Parser;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};
use pericortex::worker::Worker;
use tempfile::TempDir;

/// CorTeX worker for LaTeXML-Oxide: distributed TeX-to-HTML conversion
#[derive(Parser, Debug)]
#[command(name = "cortex-worker", about = "CorTeX worker for latexml-oxide")]
struct Cli {
  /// Dispatcher ventilator address
  #[arg(long, default_value = "tcp://localhost:51695")]
  source_address: String,

  /// Dispatcher sink address
  #[arg(long, default_value = "tcp://localhost:51696")]
  sink_address: String,

  /// Service name as registered in CorTeX. Must match the service's `name`
  /// **exactly**: the dispatcher's ventilator leases tasks for this name, and
  /// the sink re-validates each returned result's service name against the
  /// task's service id (`src/dispatcher/sink.rs`) — a mismatch silently
  /// discards the result. The CorTeX preview registers this service as
  /// `oxidized-tex-to-html` (hyphenated), so that is the default.
  #[arg(long, default_value = "oxidized-tex-to-html")]
  service: String,

  /// Number of parallel worker threads
  #[arg(long, default_value = "1")]
  pool_size: usize,

  /// ZMQ message frame size in bytes
  #[arg(long, default_value = "100000")]
  message_size: usize,

  /// Maximum number of tasks to process before exiting
  #[arg(long)]
  limit: Option<usize>,

  /// Run in standalone mode (single conversion, no dispatcher)
  #[arg(long)]
  standalone: bool,

  /// Input ZIP file (standalone mode only)
  #[arg(long)]
  input: Option<String>,

  /// Output ZIP file (standalone mode only, default: stdout)
  #[arg(long)]
  output: Option<String>,

  /// Conversion profile: ar5iv, generic
  #[arg(long, default_value = "ar5iv")]
  profile: String,

  /// Additional packages to preload
  #[arg(long)]
  preload: Vec<String>,

  /// Additional search paths (repeatable). Mirrors Perl LaTeXML's --path.
  /// Each entry is prepended to the per-job source-dir search path so
  /// e.g. `--path=ar5iv-bindings/originals` makes raw .tex files in
  /// the ar5iv repo available to InputDefinitions.
  #[arg(long = "path")]
  search_paths: Vec<String>,

  /// Per-document timeout in seconds. Default 180s.
  ///
  /// **Canvas/benchmark runs MUST use a `--release` (or `maxperf`) build.**
  /// In release, even the xy-pic / pgfplots-heavy long-tail (witness
  /// 2308.16841: 3 large xymatrix diagrams) completes in <10s; the
  /// budget then catches genuine infinite loops. Debug builds are ~12×
  /// slower (61s for the same paper) and will approach it — that is a
  /// tooling/profile issue, not a reason to widen the timeout.
  ///
  /// **Coupled to the dispatcher lease** (`cortex.toml lease_timeout_seconds`):
  /// the lease MUST stay above this with margin, so a lease expiry reliably
  /// means a *dead* worker (not a slow-but-live one) — otherwise the reaper
  /// double-leases a paper still being converted. At 180s here the lease is
  /// 240s (60s margin). Move the two together.
  ///
  /// History: 60s→120s (8-way contention pushed 21-48s standalone runs past
  /// 60s: 2306.16591, 2307.05570, 2312.13092, 2404.17751, 2311.03376,
  /// 2307.10800). 120s→180s (2026-06-18: genuine 120-180s *converging* papers
  /// — e.g. 1810.05740 @131s produces valid output — were lost to the 120s
  /// wall; the deeper >180s tail goes to the quarantine lane, not a wider
  /// global budget).
  #[arg(long, default_value = "180")]
  timeout: u64,

  /// Per-document resident-memory ceiling in MiB (0 disables). The shared
  /// `Watchdog` exits with code 137 if RSS exceeds this, giving a clean
  /// `Fatal:oom:rss` artifact instead of relying solely on the external
  /// `ulimit -v` (which the OS enforces by failing an allocation, harder to
  /// attribute). Defaults to 6 GiB, matching the sandbox `ulimit -v`.
  #[arg(long, default_value = "6144")]
  max_rss_mb: u64,

  /// Disable Presentation MathML
  #[arg(long)]
  no_pmml: bool,

  /// Disable TeX annotations in MathML
  #[arg(long)]
  no_mathtex: bool,

  /// Verbose output
  #[arg(short, long)]
  verbose: bool,

  /// Quiet output
  #[arg(short, long)]
  quiet: bool,

  /// Run as a process-supervising **harness**: spawn and keep alive a fleet of
  /// single-conversion (`--pool-size 1`) worker child processes — respawning
  /// any that exit — instead of converting in this process. This is the robust
  /// deployment model: one conversion per process gives each paper its own RAM
  /// ceiling and wall-clock timeout, so a timeout / OOM / panic / segfault
  /// kills only that one worker (the dispatcher's lease reaper recovers its
  /// task) and the harness respawns a fresh process. A single `--pool-size N`
  /// process would instead share one RAM ceiling across N concurrent
  /// conversions, false-positiving the memory guards on every paper.
  #[arg(long)]
  harness: bool,

  /// Harness mode: number of worker child processes to keep alive. Default:
  /// CPU-derived, reserving 1–4 logical cores for the OS + dispatcher.
  #[arg(long)]
  workers: Option<usize>,

  /// Harness mode: per-child **address-space** ceiling in MiB, enforced via
  /// `setrlimit(RLIMIT_AS)` before each child execs (0 disables). Default 8192
  /// (8 GiB) — sized so a *legitimate* heavy paper (≈6 GB RSS observed) reliably
  /// completes with headroom. NB: this bounds address space (VSZ), and with
  /// mimalloc VSZ runs ~1–1.5 GiB above true RSS; at 8 GiB the soft `--max-rss-mb`
  /// guard sits at 7936 MiB RSS (1.75 GiB above 6 GB) and the VSZ cap clears a
  /// 6 GB job's ~7–7.5 GiB address space, so the effective resident ceiling is
  /// ~6.5–7 GB RSS. This cap contains a **single** runaway job; the fleet-wide
  /// `--mem-pressure-floor-mb` governor contains the **aggregate** (a cluster of
  /// concurrently-heavy jobs). A breach makes the child's next allocation fail
  /// with `ENOMEM`, which `custom_alloc_error_hook` turns into a clean
  /// `Fatal:oom` + exit 137, and the harness respawns it.
  #[arg(long, default_value = "8192")]
  child_mem_limit_mb: u64,

  /// Harness mode: fleet **memory-pressure governor** floor in MiB. Omit for
  /// auto (`max(one per-child cap, 10% of RAM)`); `0` disables the governor;
  /// a positive value sets an explicit floor. While system `MemAvailable` is
  /// below the floor, the harness sheds its **largest-RSS** worker (its task is
  /// re-leased) and pauses respawns until memory recovers past 1.5× the floor —
  /// so the fleet may safely over-commit (the common case runs the full worker
  /// count) yet survives a rare cluster of heavy jobs with a deliberate,
  /// attributable shed instead of an uncontrolled kernel OOM-kill. Pair with an
  /// outer cgroup `memory.max` for a hard backstop.
  #[arg(long)]
  mem_pressure_floor_mb: Option<u64>,

  /// Per-worker memory-based **recycle** threshold in MiB (0 disables). After each
  /// conversion — with the result already returned to the dispatcher, so nothing is
  /// lost — if the worker's RSS exceeds this, it exits cleanly and the harness forks
  /// a fresh one. This reclaims the engine's accumulated interner/arena (a
  /// thread-local that is never reset mid-process — it assumes a short-lived,
  /// one-conversion-per-process model) by process replacement, without failing a
  /// paper, while keeping startup amortized for the common case of light papers. In
  /// harness mode it auto-sets to **25% of `--child-mem-limit-mb`** (= 2048 MiB at the
  /// 8 GiB default — the validated value: a fresh worker keeps the *full* ceiling for any
  /// single — even aberrant — paper, then retires once it has consumed a quarter of it). This
  /// keeps the fleet's aggregate RSS clear of the mem-pressure governor floor on a full-corpus
  /// sweep (~125 GB vs ~188 GB at 50%), so the governor does not shed workers and produce
  /// transient `never_completed_with_retries` Fatals. A standalone worker leaves it 0.
  #[arg(long, default_value = "0")]
  allocation_limit_mb: u64,
}

/// Conversion profile presets
#[allow(dead_code)] // Fields used when post-processing is enabled
#[derive(Clone, Debug)]
struct ConversionProfile {
  preloads:           Vec<String>,
  pmml:               bool,
  mathtex:            bool,
  noinvisibletimes:   bool,
  nodefaultresources: bool,
  timeout:            u64,
  /// Resident-memory ceiling in KiB for the shared `Watchdog` (0 = disabled).
  max_rss_kb:         u64,
}

impl ConversionProfile {
  fn ar5iv(
    extra_preloads: &[String],
    timeout: u64,
    max_rss_mb: u64,
    no_pmml: bool,
    no_mathtex: bool,
  ) -> Self {
    // Preload only ar5iv.sty (which RequirePackages latexml.sty). LaTeX.pool
    // is loaded lazily by \documentclass / \documentstyle digestion. Eagerly
    // preloading LaTeX.pool here previously clobbered plain-TeX papers'
    // primitives (`\magnification`, `\end`, `\bye`) — Perl LaTeXML matches
    // this lazy-load policy.
    let mut preloads = vec!["ar5iv.sty".to_string()];
    preloads.extend(extra_preloads.iter().cloned());
    ConversionProfile {
      preloads,
      pmml: !no_pmml,
      mathtex: !no_mathtex,
      noinvisibletimes: true,
      nodefaultresources: true,
      timeout,
      max_rss_kb: max_rss_mb * 1024,
    }
  }

  fn generic(
    extra_preloads: &[String],
    timeout: u64,
    max_rss_mb: u64,
    no_pmml: bool,
    no_mathtex: bool,
  ) -> Self {
    ConversionProfile {
      preloads: extra_preloads.to_vec(),
      pmml: !no_pmml,
      mathtex: !no_mathtex,
      noinvisibletimes: false,
      nodefaultresources: false,
      timeout,
      max_rss_kb: max_rss_mb * 1024,
    }
  }
}

/// The CorTeX worker implementation for latexml-oxide
#[derive(Clone)]
struct LatexmlWorker {
  service:        String,
  source_address: String,
  sink_address:   String,
  identity:       String,
  msg_size:       usize,
  threads:        usize,
  profile:        ConversionProfile,
  verbosity:      i32,
  /// Extra --path search dirs from CLI, prepended to per-job source_dir.
  search_paths:   Vec<String>,
  /// Recycle the worker after a conversion once RSS exceeds this many MiB (0 =
  /// never). See `--allocation-limit-mb` and `Worker::recycle_after_task`.
  alloc_limit_mb: u64,
}

impl LatexmlWorker {
  /// Run the conversion pipeline on an input ZIP archive.
  /// Returns the path to the output ZIP file.
  fn convert_archive(&self, input_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let wall_start = std::time::Instant::now();
    let arxiv_id = input_path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("")
      .to_string();
    // Per-document timeout: two-layer guard.
    //   1. Watchdog thread forcibly aborts after profile.timeout seconds. Catches tight native
    //      loops (Marpa, libxml2, libxslt) that never return to the Rust digestion loop.
    //   2. Cooperative stomach::set_timeout gives a graceful Err(Fatal) for the common case where
    //      digestion polls check_timeout.
    // Watchdog cancels automatically on drop at end of this function.
    let _watchdog =
      latexml_core::watchdog::Watchdog::with_limits(self.profile.timeout, self.profile.max_rss_kb);
    if self.profile.timeout > 0 {
      latexml_core::stomach::set_timeout(self.profile.timeout);
    }

    // 1. Unpack the input archive
    let (tempdir, main_tex) = unpack_archive(input_path)?;
    let source_dir = tempdir.path().to_string_lossy().to_string();

    // 2. Set up the converter with the profile
    let mut preloads = vec!["TeX.pool".to_string()];
    preloads.extend(self.profile.preloads.iter().cloned());

    // Source dir first (per-job temp), then user --path entries.
    let mut search_paths = vec![source_dir.clone()];
    search_paths.extend(self.search_paths.iter().cloned());

    let opts = Config {
      verbosity:               self.verbosity,
      format:                  OutputFormat::HTML5,
      whatsin:                 DataSize::Document,
      whatsout:                DataSize::Document,
      preamble:                None,
      postamble:               None,
      mode:                    None,
      bindings_dispatch:       Some(Rc::new(latexml_package::dispatch)),
      extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
      preload:                 Some(self.profile.preloads.clone()),
      search_paths:            Some(search_paths),
      include_comments:        Some(false),
      nomathparse:             None,
      // Corpus/parity sweeps never emit locators — keep the zero-cost path.
      source_map:              None,
    };

    let mut converter = Converter::from_config(opts.clone());
    if let Err(e) = converter.prepare_session(&opts) {
      return Err(format!("Failed to prepare converter: {}", e).into());
    }

    // 3. Convert
    let response = converter.convert(main_tex.clone());
    let xml = response
      .result
      .ok_or_else(|| format!("Conversion failed for {}", main_tex))?;

    // 4. Create destination directory for images/resources Perl LaTeXML.pm L200-205: derive HTML
    //    name from source TeX name e.g., 9256.tex → 9256.html (format "html5" → extension "html")
    let source_name = Path::new(&main_tex)
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("document");
    let html_filename = format!("{}.html", source_name);
    let dest_dir = TempDir::new()?;
    let dest_html = dest_dir.path().join(&html_filename);
    let dest_html_str = dest_html.to_string_lossy().to_string();

    // 5. Post-process: MathML + XSLT (matching CorTeX tex_to_html settings)
    let html = latexml::post::run_post_processing(&xml, &latexml::post::PostOptions {
      pmml:                      self.profile.pmml,
      cmml:                      true, // CorTeX produces both pmml and cmml
      keep_xmath:                false,
      stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
      destination:               Some(&dest_html_str),
      source_directory:          Some(&source_dir),
      search_paths:              &[],
      nodefaultresources:        self.profile.nodefaultresources,
      css_files:                 &[],
      js_files:                  &[],
      noinvisibletimes:          self.profile.noinvisibletimes,
      mathtex:                   self.profile.mathtex,
      navigationtoc:             None,
      split:                     false,
      split_xpath:               None,
      split_naming:              None,
      xslt_parameters:           &[],
      schemadocs:                false,
      // Vector-SVG fast path. 0 = auto-detect: scan the PDF header for
      // `/Subtype /Image` markers; if absent (and the file is at most
      // 500 KB), route through inkscape→SVG for vector-clean output
      // and the documented 100×+ speedup over ImageMagick rasterisation
      // on pgfplots/matplotlib PDFs (PERFORMANCE.md §"Vector-SVG fast
      // path"). Raster-bearing PDFs detect their image XObject and
      // stay on the gs/convert path. Override with a positive integer
      // (KB threshold) to force the legacy size-only gate, or set
      // `LATEXML_GRAPHICS_VECTOR_AUTO_OFF=1` to disable auto-detect
      // entirely. Replaces the prior hard-coded 200 KB cutoff (which
      // was strictly looser — would attempt SVG on 200 KB raster PDFs
      // even when their image XObject was visible in the header).
      graphics_svg_threshold_kb: 0,
      // cortex_worker is the canvas-bulk path — always emit the full
      // document, never the fragment / math extraction variants.
      whatsout:                  latexml_post::extract::Whatsout::Document,
    });

    // 6. Get log and status (Perl: status line is last line of log)
    let status_str = format!("Status:conversion:{}", response.status_code);
    let log = format!("{}\n{}", response.log, status_str);

    // 7. Finalize per-job telemetry. Phase counters were populated by the converter/post guards;
    //    here we fill in identifiers, wall, and resource peaks before serializing.
    let telemetry_json = {
      use latexml_core::{
        common::error::{LogStatus, get_status},
        telemetry,
      };
      telemetry::set_paper_id(&arxiv_id);
      telemetry::set_wall_us(wall_start.elapsed().as_micros() as u64);
      telemetry::set_category(match response.status_code {
        0 | 1 => "ok",
        2 => "conversion_error",
        _ => "conversion_fatal",
      });
      telemetry::set_exit_code(response.status_code as i32);
      telemetry::set_output_bytes(html.len() as u64);
      telemetry::set_max_rss_kb(read_max_rss_kb_proc());
      let (cu, cs) = read_child_rusage_us_proc();
      telemetry::set_child_rusage_us(cu, cs);
      // Snapshot Error!/Warn!/Fatal! counts from common::error::REPORT
      // (the canonical counter populated by note_status). Without this
      // copy the telemetry `errors`/`warnings`/`fatal_errors` fields
      // stay 0 even when the log shows hundreds of errors — observed
      // across stages 01-10 (190k jobs).
      telemetry::set_status_counts(
        get_status(LogStatus::Warning) as u32,
        get_status(LogStatus::Error) as u32,
        get_status(LogStatus::Fatal) as u32,
      );
      telemetry::take().to_json_line()
    };

    // 8. Pack output ZIP: HTML (named after source) + images + log + status + telemetry.
    //    The path must be UNIQUE per task, not just per process: with `--pool-size N`
    //    (N worker threads sharing one PID) a PID-only name would have every thread
    //    write the same `/tmp/cortex_output_<pid>.zip` concurrently — interleaved
    //    writes corrupt the archive and threads stream each other's bytes back. A
    //    process-wide atomic sequence makes it collision-free across threads + papers.
    let output_path = unique_output_path();
    latexml_post::pack::pack_archive(&latexml_post::pack::PackOptions {
      zip_path:          &output_path.to_string_lossy(),
      html_filename:     &html_filename,
      html:              &html,
      log_filename:      Some("cortex.log"),
      log:               &log,
      status:            &status_str,
      resource_dir:      Some(dest_dir.path()),
      telemetry_json:    Some(&telemetry_json),
      // Canvas bundles are ephemeral (extracted then discarded); leave
      // member timestamps at the crate default.
      source_date_epoch: None,
    })?;

    Ok(output_path)
  }
}

/// Read peak RSS in KB from /proc/self/status's VmHWM. 0 on failure.
fn read_max_rss_kb_proc() -> u64 {
  fs::read_to_string("/proc/self/status")
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
#[cfg(unix)]
fn read_child_rusage_us_proc() -> (u64, u64) {
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
fn read_child_rusage_us_proc() -> (u64, u64) { (0, 0) }

/// Maximum number of *consecutive* caught panics before the worker process
/// exits for a clean supervisor restart. A single poison paper among healthy
/// ones never trips this (the streak resets on the next success); a sustained
/// run of panics means the thread-local engine state is likely corrupted
/// (an unwind left a lock/`RefCell` mid-mutation), so a fresh process is safer
/// than risking silently-wrong output on subsequent papers.
const MAX_CONSECUTIVE_PANICS: u32 = 5;

thread_local! {
  /// Consecutive caught-panic counter for *this* worker thread (each pool
  /// thread keeps its own engine state, hence its own streak).
  static PANIC_STREAK: Cell<u32> = const { Cell::new(0) };

  /// Detail of the most recent panic on this thread — the panic **location**
  /// (`file:line:col`) plus message — stashed by the panic hook
  /// ([`install_panic_hook`]) at unwind time, then consumed by [`Worker::convert`]
  /// for the failure artifact. `catch_unwind` only hands back the payload (the
  /// bare message, e.g. "called `Option::unwrap()` on a `None` value"), which has
  /// no location; capturing it in the hook is the only way to record *where* a
  /// conversion panicked. The full backtrace is emitted to stderr by the hook.
  static LAST_PANIC_DETAIL: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Install a process-wide panic hook that records, for every panic, the panic
/// **location** + message (into [`LAST_PANIC_DETAIL`] for the failing task's
/// artifact) and prints the **full backtrace** to stderr (captured in the
/// worker/harness log) so a caught conversion panic is debuggable. Uses
/// `Backtrace::force_capture`, so a backtrace is taken regardless of
/// `RUST_BACKTRACE`; the panic location is present even in a stripped release
/// build (panic-location strings survive `strip`), while symbolicated frames
/// need a build with debug info.
fn install_panic_hook() {
  std::panic::set_hook(Box::new(|info| {
    let loc = info
      .location()
      .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
      .unwrap_or_else(|| "<unknown location>".to_string());
    let payload = info.payload();
    let msg = payload
      .downcast_ref::<&str>()
      .map(|s| (*s).to_string())
      .or_else(|| payload.downcast_ref::<String>().cloned())
      .unwrap_or_else(|| "<non-string panic payload>".to_string());
    let bt = std::backtrace::Backtrace::force_capture();
    // Full backtrace → stderr (captured in the worker/harness log) for debugging.
    eprintln!("Fatal:panic:backtrace panicked at {loc}: {msg}\n{bt}");
    // Bounded location+message → the per-task failure artifact (parser-safe).
    LAST_PANIC_DETAIL.with(|p| *p.borrow_mut() = Some(format!("panicked at {loc}: {msg}")));
  }));
}

impl Worker for LatexmlWorker {
  /// Convert one task archive, **isolating every failure mode of the
  /// latexml-oxide conversion call** so a single paper can never take the
  /// worker down (the "anything that can go wrong will go wrong" contract):
  ///
  /// * A returned `Err` (unpack failure, disk-write failure, …) → a structured
  ///   `Status:conversion:3` archive carrying a `Fatal:conversion:*` log line.
  /// * A **panic** inside digestion/post-processing (the most dangerous case:
  ///   it would otherwise unwind through `pericortex`'s loop and kill the
  ///   process) → caught here (release builds use `panic = "unwind"`), turned
  ///   into a `Fatal:panic:*` archive, and counted toward [`MAX_CONSECUTIVE_PANICS`].
  ///
  /// Either way the dispatcher receives an attributable result for the paper
  /// instead of losing the task (and the worker), and the next paper proceeds.
  /// Timeouts / OOM are handled out-of-band by the `Watchdog` + alloc hook
  /// (they exit the process; the dispatcher's lease reaper recovers the task).
  fn convert(&self, path: &Path) -> Result<File, Box<dyn Error>> {
    let arxiv_id = path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("document")
      .to_string();

    // `AssertUnwindSafe`: the engine state is a thread-local singleton rebuilt
    // per paper (`prepare_session`), so a caught unwind does not hand a logically
    // half-updated value across the boundary — the next paper re-initialises it.
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| self.convert_archive(path)));

    let output_path = match outcome {
      Ok(Ok(path)) => {
        PANIC_STREAK.with(|s| s.set(0));
        path
      },
      Ok(Err(e)) => {
        // The conversion failed cleanly (returned `Err`). Not a panic, so don't
        // touch the streak — produce an attributable Fatal result and continue.
        //
        // An *unprocessable input* (no convertible TeX source) is sentinel-prefixed
        // `invalid:<what>: <message>` (find_main_tex / PDF-magic / binary). Route it to
        // the `invalid` category — Perl emits `Fatal('invalid', …)` for the same cases,
        // and the dispatcher maps a fatal/`invalid`-category message to the **Invalid**
        // status (its own report row, discounted from the total) rather than a plain
        // Fatal. Anything else is a genuine conversion failure: `conversion:caught`.
        let es = e.to_string();
        let (category, what, message) = match es.strip_prefix("invalid:") {
          Some(rest) => {
            let (what, msg) = rest.split_once(": ").unwrap_or((rest, ""));
            ("invalid", what, msg)
          },
          None => ("conversion", "caught", es.as_str()),
        };
        log::warn!(
          target: "cortex_worker",
          "conversion of {arxiv_id} failed ({category}:{what}): {message}; returning a Fatal result archive"
        );
        write_failure_zip(&arxiv_id, category, what, message)?
      },
      Err(panic) => {
        // Prefer the panic hook's captured detail (location + message); fall back
        // to the bare payload if the hook didn't run for some reason.
        let msg = LAST_PANIC_DETAIL
          .with(|p| p.borrow_mut().take())
          .unwrap_or_else(|| panic_message(&*panic));
        let streak = PANIC_STREAK.with(|s| {
          let n = s.get() + 1;
          s.set(n);
          n
        });
        log::error!(
          target: "cortex_worker",
          "PANIC caught converting {arxiv_id} (consecutive: {streak}/{MAX_CONSECUTIVE_PANICS}): {msg}"
        );
        if streak >= MAX_CONSECUTIVE_PANICS {
          log::error!(
            target: "cortex_worker",
            "{MAX_CONSECUTIVE_PANICS} consecutive panics — engine state is likely corrupted; \
             exiting (code 70) for a clean supervisor restart. The dispatcher's lease reaper \
             re-leases the in-flight task to another worker."
          );
          process::exit(70);
        }
        write_failure_zip(&arxiv_id, "panic", "caught", &msg)?
      },
    };

    let file = File::open(&output_path)?;
    // Clean up temp file after opening (the open fd keeps the bytes readable).
    let _ = fs::remove_file(&output_path);
    // NB: do NOT reset the thread engine here. The persistent worker relies on
    // the dump definitions (loaded once) PERSISTING across papers — `prepare_session`
    // re-initialises only the ephemeral per-conversion state. Returning memory to
    // the OS between papers is the **allocator's** job (jemalloc decay under
    // `--features jemalloc`), not an explicit engine reset: an earlier attempt to
    // call `reset_thread_engine()` here wiped the persisted definitions and made
    // every subsequent conversion panic (`unwrap() on None`). See the harness
    // validation, 2026-06-17.
    Ok(file)
  }

  fn message_size(&self) -> usize { self.msg_size }

  fn get_service(&self) -> &str { &self.service }

  fn get_source_address(&self) -> Cow<'_, str> { Cow::Borrowed(&self.source_address) }

  fn get_sink_address(&self) -> Cow<'_, str> { Cow::Borrowed(&self.sink_address) }

  fn pool_size(&self) -> usize { self.threads }

  fn set_identity(&mut self, identity: String) { self.identity = identity; }

  fn get_identity(&self) -> &str { &self.identity }

  /// Recycle this worker (clean exit → fresh respawn) once its resident memory has
  /// grown past `--allocation-limit-mb`. Called by `start_single` AFTER the result is
  /// returned, so no paper is lost. Reads current `VmRSS` (one cheap `/proc` read);
  /// `0` disables. Bounds the engine's never-reset thread-local interner/arena by
  /// process replacement rather than a fragile mid-life reset.
  fn recycle_after_task(&self) -> bool {
    self.alloc_limit_mb > 0
      && latexml_core::watchdog::process_rss_kb()
        .is_some_and(|rss_kb| rss_kb / 1024 > self.alloc_limit_mb)
  }
}

// --- Helper functions (shared with latexml_oxide.rs) ---

fn unpack_archive(archive_path: &Path) -> Result<(TempDir, String), Box<dyn Error>> {
  let tempdir = TempDir::new()?;
  let dest = tempdir.path();

  let path_str = archive_path.to_string_lossy();
  if path_str.ends_with(".zip") {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(dest)?;
  } else if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
    let file = File::open(archive_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(dest)?;
  } else if path_str.ends_with(".tar") {
    let file = File::open(archive_path)?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(dest)?;
  } else {
    return Err(format!("Unsupported archive format: {}", path_str).into());
  }

  let main_tex = find_main_tex(dest)?;
  Ok((tempdir, main_tex))
}

/// Faithful port of Perl Pack.pm detect_source / arXiv::FileGuess.
/// Identical to the find_main_tex in latexml_oxide.rs.
fn find_main_tex(dir: &Path) -> Result<String, Box<dyn Error>> {
  use once_cell::sync::Lazy;
  use regex::Regex;

  // Phase I.1: Check 00README.json (2025 arXiv format)
  if let Some(filename) = parse_readme_json(dir) {
    let main_path = dir.join(&filename);
    if main_path.exists() {
      return Ok(main_path.to_string_lossy().to_string());
    }
  }

  // Phase I.1.2: Check 00README.XXX (legacy arXiv format)
  let readme_xxx = dir.join("00README.XXX");
  if readme_xxx.exists()
    && let Ok(content) = fs::read_to_string(&readme_xxx)
  {
    for line in content.lines() {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() >= 2 && parts[1] == "toplevelfile" {
        let main_path = dir.join(parts[0]);
        if main_path.exists() {
          return Ok(main_path.to_string_lossy().to_string());
        }
      }
    }
  }

  // Phase I.2: Heuristic detection (ported from arXiv::FileGuess via Pack.pm)
  // Perl Pack.pm L25 TEX_EXT = qr/\.(?:[tT](:?[eE][xX]|[xX][tT])|ltx|LTX)$/
  // → .tex, .txt, .ltx (case-insensitive).
  fn collect_tex_files(dir: &Path, files: &mut Vec<PathBuf>, fallback: bool) {
    if let Ok(entries) = fs::read_dir(dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          collect_tex_files(&path, files, fallback);
        } else if !fallback {
          if path.extension().is_some_and(|e| {
            let e = e.to_ascii_lowercase();
            e == "tex" || e == "txt" || e == "ltx"
          }) {
            files.push(path);
          }
        } else {
          // Perl Pack/Dir.pm L47 fallback: `!/\./ || /\.[^.]{4,}$/`
          //   → files with no extension, or with extension ≥4 chars.
          // arxiv 0908.4110 ships a bare "birkhoffproofrev1" LaTeX source.
          let ext_opt = path.extension().and_then(|e| e.to_str());
          let keep = match ext_opt {
            None => true,
            Some(ext) => ext.len() >= 4,
          };
          if keep {
            files.push(path);
          }
        }
      }
    }
  }

  // Skip files whose magic bytes identify them as PDF (e.g. arXiv source
  // archives that contain a PDF mis-named with a `.tex` extension). Perl
  // Pack.pm doesn't probe for this, but treating a PDF as TeX produces
  // thousands of spurious parse errors, so emit the Perl-canonical
  // `Fatal:invalid:not_tex_source` (status 3) and bail.
  fn is_pdf_magic(path: &Path) -> bool {
    let mut buf = [0u8; 5];
    if let Ok(mut f) = File::open(path) {
      use std::io::Read;
      if f.read(&mut buf).is_ok_and(|n| n == 5) {
        return &buf == b"%PDF-";
      }
    }
    false
  }

  let mut tex_files: Vec<PathBuf> = Vec::new();
  collect_tex_files(dir, &mut tex_files, false);
  let candidates_before_pdf_filter = tex_files.len();
  tex_files.retain(|p| !is_pdf_magic(p));
  if tex_files.is_empty() && candidates_before_pdf_filter > 0 {
    return Err(
      "invalid:not_tex_source: PDF magic detected in source file (no TeX-format files in archive)"
        .into(),
    );
  }
  if tex_files.is_empty() {
    collect_tex_files(dir, &mut tex_files, true);
    tex_files.retain(|p| !is_pdf_magic(p));
  }
  if tex_files.is_empty() {
    return Err("invalid:no_tex_source: no .tex files found in archive".into());
  }

  // Regexes for content-based detection (Perl Pack.pm L116-166)
  static RE_AUTOIGNORE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%auto-ignore").unwrap());
  static RE_TEXINFO: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\input texinfo").unwrap());
  static RE_AUTOINCLUDE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%auto-include").unwrap());
  static RE_FORMAT_HINT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\r?%&(\S+)").unwrap());
  static RE_DOCCLASS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|\r)\s*\\document(?:style|class)").unwrap());
  static RE_MAYBE_TEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|\r)\s*\\(?:font|magnification|input|def|special|baselineskip|begin)").unwrap()
  });
  static RE_INPUT_INCLUDE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\(?:input|include)(?:\s+|\{)([^ \}]+)").unwrap());
  static RE_END_BYE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|\r)\s*\\(?:end|bye)(?:\s|$)").unwrap());
  static RE_END_BYE2: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\(?:end|bye)(?:\s|$)").unwrap());
  static RE_MAC_TEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\input *(?:harv|lanl)mac|\\input\s+phyzzx").unwrap());
  static RE_METAFONT: Lazy<Regex> = Lazy::new(|| Regex::new(r"beginchar\(").unwrap());
  static RE_BIBTEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(?:^|\r)@(?:book|article|inbook|unpublished)\{").unwrap());
  static RE_UUENCODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^begin \d{1,4}\s+\S+\r?$").unwrap());
  static RE_WITHDRAWN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"paper deliberately replaced by what little").unwrap());
  static RE_AMSTEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^amstex$").unwrap());
  // Perl Pack.pm L128: `s/\%[^\r]*//`. Strip a single `%`-comment, stopping at
  // the next `\r` (or string end). \r-aware so bare-\r-line-ended files (Mac
  // classic) — which Perl reads as one big <$fh> "line" — still expose any
  // `\documentclass` that follows a stripped comment. A naive
  // `raw_line.find('%').map(|p| &raw_line[..p])` truncates everything past
  // the first `%`, hiding post-comment `\documentclass` on \r-only files.
  // Witness: cond-mat0002096, 0708.2784 in 100k canvas.
  static RE_STRIP_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"%[^\r]*").unwrap());

  // Score each file: likelihood 0-3 (Perl: Main_TeX_likelihood)
  let mut likelihood: rustc_hash::FxHashMap<PathBuf, f32> = rustc_hash::FxHashMap::default();
  // Each entry is (vetoed_path, vetoer_path). A veto from a low-score
  // wrapper file (e.g. a 2-line `\input{main}` shim) MUST NOT remove a
  // high-score documentclass-bearing file from the candidate pool. The
  // vetoer's score is known only after the scoring loop completes, so
  // we record the vetoer and apply the veto post-loop with the score
  // comparison. Witness 2307.13586.
  let mut vetoed: Vec<(PathBuf, PathBuf)> = Vec::new();
  // Phase D pre-screen: track sentinel reasons so the empty-candidates
  // branch can return a categorized `Fatal:invalid:<reason>` (per
  // SYNC_STATUS.md "Phase E asymptote: convert intractable papers to
  // Fatal:invalid:<reason> via Phase D pre-screen"). Witness:
  // 0903.3183.tex contains exactly `%auto-ignore` (12 bytes) — Perl
  // skips, Rust pre-fix returned a generic "No viable .tex files"
  // error indistinguishable from real failures.
  let mut had_auto_ignore = false;

  for tex_file in &tex_files {
    if !tex_file.exists() {
      continue;
    }
    let Ok(raw) = fs::read(tex_file) else {
      continue;
    };
    let content = String::from_utf8_lossy(&raw);
    let mut maybe_tex = false;
    let mut maybe_tex_priority = false;
    let mut maybe_tex_priority2 = false;
    let mut determined = false;

    for (lineno, raw_line) in content.lines().enumerate() {
      let lineno1 = lineno + 1;
      if lineno1 <= 10
        && (RE_AUTOIGNORE.is_match(raw_line)
          || RE_TEXINFO.is_match(raw_line)
          || RE_AUTOINCLUDE.is_match(raw_line))
      {
        likelihood.insert(tex_file.clone(), 0.0);
        if RE_AUTOIGNORE.is_match(raw_line) {
          had_auto_ignore = true;
        }
        determined = true;
        break;
      }
      if lineno1 <= 12
        && let Some(cap) = RE_FORMAT_HINT.captures(raw_line)
      {
        let fmt = &cap[1];
        if fmt == "latex209" || fmt == "biglatex" || fmt == "latex" || fmt == "LaTeX" {
          likelihood.insert(tex_file.clone(), 3.0);
        } else {
          likelihood.insert(tex_file.clone(), 1.0);
        }
        determined = true;
        break;
      }
      // Perl L128: strip ONE `%`-comment up to the next `\r`. `\r`-aware
      // so bare-`\r` line-ended files (read as one big "line" in Perl
      // because `$/=\n`) preserve subsequent `\r\documentclass` chunks.
      let stripped: Cow<str> = RE_STRIP_COMMENT.replacen(raw_line, 1, "");
      let line: &str = &stripped;

      if RE_DOCCLASS.is_match(line) {
        likelihood.insert(tex_file.clone(), 3.0);
        determined = true;
        break;
      }
      if RE_MAYBE_TEX.is_match(line) {
        maybe_tex = true;
      }
      if let Some(cap) = RE_INPUT_INCLUDE.captures(line) {
        maybe_tex = true;
        let mut vetoed_name = cap[1].to_string();
        if RE_AMSTEX.is_match(&vetoed_name) {
          likelihood.insert(tex_file.clone(), 2.0);
          determined = true;
          break;
        }
        if !vetoed_name.contains('.') {
          vetoed_name = vetoed_name.trim_end().to_string() + ".tex";
        }
        let base_dir = tex_file.parent().unwrap_or(dir);
        // Tag veto with vetoer's path; we'll only honor the veto when
        // the vetoer's eventual score >= vetee's score. Prevents a tiny
        // wrapper file (`\input{main}`) from removing a documentclass-
        // bearing main.tex from the candidate set. Witness 2307.13586.
        vetoed.push((base_dir.join(&vetoed_name), tex_file.clone()));
      }
      if RE_END_BYE.is_match(line) {
        maybe_tex_priority = true;
      }
      if RE_END_BYE2.is_match(line) {
        maybe_tex_priority2 = true;
      }
      if RE_MAC_TEX.is_match(line) {
        likelihood.insert(tex_file.clone(), 1.0);
        determined = true;
        break;
      }
      if RE_METAFONT.is_match(line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if RE_BIBTEX.is_match(raw_line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if RE_UUENCODE.is_match(raw_line) {
        if maybe_tex_priority {
          likelihood.insert(tex_file.clone(), 2.0);
        } else if maybe_tex {
          likelihood.insert(tex_file.clone(), 1.0);
        } else {
          likelihood.insert(tex_file.clone(), 0.0);
        }
        determined = true;
        break;
      }
      if RE_WITHDRAWN.is_match(line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
    }
    if !determined {
      let score = if maybe_tex_priority {
        2.0
      } else if maybe_tex_priority2 {
        1.5
      } else if maybe_tex {
        1.0
      } else {
        0.0
      };
      likelihood.insert(tex_file.clone(), score);
    }
  }

  // Apply each veto only if the vetoer's score >= vetee's score.
  // Honors the wrapper-vs-main-doc case (see `vetoed` declaration).
  for (vetee, vetoer) in &vetoed {
    let vetee_score = likelihood.get(vetee).copied().unwrap_or(0.0);
    let vetoer_score = likelihood.get(vetoer).copied().unwrap_or(0.0);
    if vetoer_score >= vetee_score {
      likelihood.remove(vetee);
    }
  }

  // Filter to score > 0, sort by score descending
  let mut candidates: Vec<PathBuf> = likelihood
    .keys()
    .filter(|f| likelihood[*f] > 0.0)
    .cloned()
    .collect();
  candidates.sort_by(|a, b| likelihood[b].partial_cmp(&likelihood[a]).unwrap());
  if candidates.is_empty() {
    if had_auto_ignore {
      // Perl-faithful: an arxiv `%auto-ignore` source still gets opened
      // — the `%` line is a comment, the rest is empty, and Perl
      // happily reports "Conversion complete: No obvious problems" with
      // an empty XML body. Witness: 2307.10758 (a 12-byte `%auto-ignore`
      // .tex). We were emitting `Fatal:invalid:auto-ignore` here, but
      // that turned 90 wp4 corpus entries into hard failures vs Perl's
      // OK. Fall through to pick the auto-ignore file and let the
      // normal pipeline produce an empty document.
      //
      // File-pick: prefer the dirname-matching file (arxiv convention is
      // `<id>/<id>.tex`); else the first listed file.
      let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or_default();
      let auto_ignore_main = tex_files
        .iter()
        .find(|p| {
          p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|stem| stem == dir_name)
        })
        .cloned()
        .or_else(|| tex_files.first().cloned());
      if let Some(p) = auto_ignore_main {
        return Ok(p.to_string_lossy().to_string());
      }
    }
    return Err("invalid:no_viable_tex: .tex file(s) present but none is convertible LaTeX (PostScript/binary/encrypted source)".into());
  }

  // Keep only max-scoring candidates
  let max_score = likelihood[&candidates[0]];
  candidates.retain(|f| (likelihood[f] - max_score).abs() < f32::EPSILON);

  // Heuristic 1: prefer shallowest path
  if candidates.len() > 1 {
    let min_depth = candidates
      .iter()
      .map(|f| f.strip_prefix(dir).unwrap_or(f).components().count())
      .min()
      .unwrap_or(0);
    candidates.retain(|f| f.strip_prefix(dir).unwrap_or(f).components().count() == min_depth);
  }

  // NOTE: Perl Pack.pm L196-218 only applies these heuristics in this
  // exact order: shallowest-path → PDF-like \includegraphics → .bbl →
  // common-name (`main`/`ms`/`paper`.tex) → lexicographic. A previous
  // "Heuristic 1.5" filtered candidates by filename keywords (template,
  // elsdoc, readme) — but Perl doesn't have that filter, and the Perl-
  // equivalent lexicographic tiebreaker handles the class-self-docs
  // case correctly (e.g. 2107.07756: `quantum-template.tex` <
  // `quantumarticle.tex` so the user's paper wins by lex order).
  // Reverting to strict Perl-parity per feedback_prefer_root_cause
  // guidance: prefer matching upstream heuristics over a hand-curated
  // SURPASS-PERL filter that diverges from arXiv::FileGuess.

  // Heuristic 2: prefer files with a matching .bbl file
  // (.bbl is the strongest "this is the main file" signal — present
  // only when the user has compiled the doc through bibtex/biber.)
  // Was H3, swapped before .includegraphics: 2011.11637 ships
  // arxiv_paper.tex (with arxiv_paper.bbl) AND cvpr.tex (CVPR template
  // with .includegraphics + .pdf refs). The .pdf-includegraphics
  // heuristic was eliminating arxiv_paper.tex and leaving the template
  // cvpr.tex — which doesn't define `\eg` etc., causing R=many.
  if candidates.len() > 1 {
    let bbl_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| f.with_extension("bbl").exists())
      .cloned()
      .collect();
    if !bbl_candidates.is_empty() {
      candidates = bbl_candidates;
    }
  }

  // Heuristic 3: prefer files with PDF-like \includegraphics
  // Perl Pack.pm L222-244 heuristic_check_for_pdftex: requires the
  // strict form `\includegraphics[^%]*\.(pdf|png|gif|jpg)\s?\}` on a
  // non-commented line, OR `\pdfoutput=1` in the first 5 such lines.
  // The previous substring-based check (`contains("\\includegraphics")
  // && contains(".pdf")`) was too lax: elsdoc.tex (1907.06674) has both
  // tokens — `\includegraphics` examples in code samples, `.pdf` in
  // unrelated discussion text — without any actual `\includegraphics
  // {file.pdf}` invocation. Strict regex eliminates the false positive
  // and aligns with arXiv::FileGuess.
  static RE_INCLUDEGRAPHICS_PDF: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?im)^[^%\n\r]*\\includegraphics[^%\n\r]*\.(?:pdf|png|gif|jpg)\s?\}").unwrap()
  });
  static RE_PDFOUTPUT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^%\n\r]*\\pdfoutput(?:\s+)?=(?:\s+)?1").unwrap());
  if candidates.len() > 1 {
    let pdf_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        fs::read(f).ok().is_some_and(|raw| {
          let c = String::from_utf8_lossy(&raw);
          if RE_INCLUDEGRAPHICS_PDF.is_match(&c) {
            return true;
          }
          // Perl: $pdfoutput_checks >= 0 limits to first 5 matching candidate
          // lines (any line matching `\pdfoutput=1`). Approximate by scanning
          // the whole file — the regex requires non-commented context already.
          RE_PDFOUTPUT.is_match(&c)
        })
      })
      .cloned()
      .collect();
    if !pdf_candidates.is_empty() {
      candidates = pdf_candidates;
    }
  }

  // Heuristic 4: prefer common main file names
  if candidates.len() > 1 {
    let common: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        f.file_name().is_some_and(|n| {
          let n = n.to_str().unwrap_or("");
          n == "main.tex" || n == "ms.tex" || n == "paper.tex"
        })
      })
      .cloned()
      .collect();
    if !common.is_empty() {
      candidates = common;
    }
  }

  // Final tiebreaker: lexicographic order
  candidates.sort();
  Ok(candidates[0].to_string_lossy().to_string())
}

/// Parse 00README.json for toplevel source filename.
fn parse_readme_json(dir: &Path) -> Option<String> {
  let content = fs::read_to_string(dir.join("00README.json")).ok()?;
  let sources_start = content.find("\"sources\"")?;
  let rest = &content[sources_start..];
  let arr_start = rest.find('[')?;
  let arr_end = rest.find(']')?;
  let arr = &rest[arr_start + 1..arr_end];
  for obj_str in arr.split('}') {
    if !obj_str.contains("\"toplevel\"") {
      continue;
    }
    if let Some(fn_pos) = obj_str.find("\"filename\"") {
      let after_key = &obj_str[fn_pos + 10..];
      let after_key = after_key.trim_start();
      let after_key = after_key.strip_prefix(':')?;
      let after_key = after_key.trim_start();
      let after_key = after_key.strip_prefix('"')?;
      let mut result = String::new();
      for ch in after_key.chars() {
        match ch {
          '"' => break,
          '\\' => continue,
          c => result.push(c),
        }
      }
      if !result.is_empty() {
        return Some(result);
      }
    }
  }
  None
}

/// A process-unique temp path for a packed result archive. Combines the PID
/// (distinguishes concurrent worker *processes* on a shared `/tmp`) with a
/// monotonic per-process counter (distinguishes pool *threads* and successive
/// papers within one process), so no two in-flight conversions ever target the
/// same file. Replaces the former PID-only name that pooled threads raced on.
fn unique_output_path() -> PathBuf {
  static OUTPUT_SEQ: AtomicU64 = AtomicU64::new(0);
  let seq = OUTPUT_SEQ.fetch_add(1, Ordering::Relaxed);
  std::env::temp_dir().join(format!("cortex_output_{}_{}.zip", process::id(), seq))
}

/// Extract a human-readable message from a caught-panic payload — the usual
/// `&str` / `String` cases, else a generic label.
fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
  if let Some(s) = panic.downcast_ref::<&str>() {
    (*s).to_string()
  } else if let Some(s) = panic.downcast_ref::<String>() {
    s.clone()
  } else {
    "<non-string panic payload>".to_string()
  }
}

/// Build a minimal, **attributable** failure archive at a unique temp path and
/// return that path. Carries exactly what the dispatcher needs to record a hard
/// failure for a paper whose conversion `Err`'d or panicked: a `status` member
/// (`Status:conversion:3`), a `cortex.log` with one `Fatal:<category>:<what> …`
/// line (so the dispatcher's log parse + telemetry count it as a fatal, never a
/// silent "0 errors"), and a minimal `telemetry.json`. Mirrors the member shape
/// of [`write_timeout_placeholder_zip`] so the same parser handles both. For an
/// unprocessable input (no convertible TeX source) the caller passes
/// `category = "invalid"` with a specific `what` (e.g. `no_tex_source`), mirroring
/// Perl's `Fatal('invalid', …)` — the dispatcher maps a fatal/`invalid`-category
/// message to the **Invalid** status (own report row, discounted from the total).
fn write_failure_zip(
  arxiv_id: &str,
  category: &str,
  what: &str,
  message: &str,
) -> Result<PathBuf, Box<dyn Error>> {
  let output_path = unique_output_path();
  // Single-line + length-bounded so a giant or newline-laden panic payload can
  // neither bloat the archive nor smuggle a fake `Status:`/`Fatal:` line into
  // the log parser.
  let sanitized: String = message
    .replace(['\n', '\r'], " ")
    .chars()
    .take(2000)
    .collect();
  let log = format!(
    "Fatal:{category}:{what} latexml-oxide failed to convert {arxiv_id}: {sanitized}\n\
     Status:conversion:3\n"
  );
  // `serde_json` so a hostile filename can't break the JSON (robustness).
  let telemetry = serde_json::json!({
    "paper_id": arxiv_id,
    "category": "conversion_fatal",
    "exit_code": 3,
  })
  .to_string();

  let file = File::create(&output_path)?;
  let mut zip = zip::ZipWriter::new(file);
  let options =
    zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
  zip.start_file("status", options)?;
  zip.write_all(b"Status:conversion:3")?;
  zip.start_file("cortex.log", options)?;
  zip.write_all(log.as_bytes())?;
  zip.start_file("telemetry.json", options)?;
  zip.write_all(telemetry.as_bytes())?;
  zip.finish()?;
  Ok(output_path)
}

/// Minimal "we timed out" placeholder zip. Written only from the
/// watchdog's pre-exit hook (`set_pre_exit_hook` in `latexml_core::
/// watchdog`), so the happy-path overhead is zero. Contains just
/// two members: `status` (`Status:conversion:3`) and `cortex.log`
/// (a single `Fatal:timeout:wallclock …` line). The parent harness
/// can stat the output file and parse `status` exactly as for any
/// other failed conversion, instead of seeing a missing file plus
/// an `Aborted (core dumped)` shell message.
fn write_timeout_placeholder_zip(
  output_path: &str,
  input_path: &str,
  timeout_secs: u64,
) -> Result<(), Box<dyn Error>> {
  let file = File::create(output_path)?;
  let mut zip = zip::ZipWriter::new(file);
  let options =
    zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
  zip.start_file("status", options)?;
  zip.write_all(b"Status:conversion:3")?;
  zip.start_file("cortex.log", options)?;
  let log = format!(
    "Fatal:timeout:wallclock latexml-oxide hit the {timeout_secs}s \
     main-level wall-clock timeout converting {input_path}; the worker \
     thread was presumed wedged in a tight native loop and the watchdog \
     exited the process via exit(124). No output produced.\n\
     Status:conversion:3\n"
  );
  zip.write_all(log.as_bytes())?;
  zip.finish()?;
  Ok(())
}

// `pack_output_zip_with_resources` + `add_dir_to_zip` moved into
// `latexml_post::pack::pack_archive` (2026-05-18) — single source of
// truth shared with `latexml_oxide --post`. Perl analog: `LaTeXML::Post::Pack`.

// --- Main ---

/// Custom allocation-failure hook: detects the runaway-macro-expansion
/// pathology at the moment it manifests (Rust's `alloc::handle_alloc_error`)
/// and emits a `Fatal:` line matching the project's logging convention so
/// aggregation tooling (`grep Fatal:`, `tools/parity_stats.sh`,
/// `tools/benchmark_canvas.sh`, telemetry's `fatal_errors` field) records
/// it. Exits with code 137 → canvas categorises as `oom_or_kill` rather
/// than `abort`.
///
/// Witness pathology: paper 2305.16331 + `\u\i` under `mathtext + T2A`
/// drives `gullet::pushback` into runaway growth via repeated unread of
/// growing-each-cycle invoked-Tokens; the next `Vec::reserve` doubles
/// capacity past the 4 GiB / 8 GB ulimit boundary and fails.
///
/// No overhead on the normal digestion path — fires only when the
/// allocator returns null. The hook avoids any heap allocation in its
/// body (no `format!`, no string concat) because the global allocator
/// has just failed: `eprintln!` writes via a stack-allocated formatter.
fn custom_alloc_error_hook(layout: Layout) {
  // Single Fatal: line on its own — matches `log_fatal` output shape
  // (`Fatal:<Target>:<Category> <message>`) so aggregation grep keys
  // on it cleanly. `oom`/`alloc_failed` are not enum variants of
  // `ErrorTarget`/`ErrorCategory` because we can't construct those
  // here without an `arena`/allocator round-trip; using the same shape
  // string is enough for the harness.
  eprintln!(
    "Fatal:oom:alloc_failed allocation of {} bytes (align {}) failed; \
     likely runaway macro expansion (gullet pushback Vec growth past \
     worker memory budget). Witness: paper 2305.16331 + `\\u\\i` under \
     `mathtext + T2A`. Exiting with code 137.",
    layout.size(),
    layout.align()
  );
  // When `RUST_BACKTRACE=1` is set, print the captured backtrace too.
  // Helps localise the call site of the failing allocation. The
  // backtrace API allocates internally; if that re-trips the OOM the
  // worker still exits cleanly via the exit() below.
  if std::env::var_os("RUST_BACKTRACE").is_some() {
    let bt = std::backtrace::Backtrace::force_capture();
    eprintln!("{bt}");
  }
  process::exit(137);
}

fn main() -> Result<(), Box<dyn Error>> {
  set_alloc_error_hook(custom_alloc_error_hook);
  // Capture panic location + full backtrace for every panic (the conversion
  // catch_unwind only sees the bare payload otherwise). Process-wide, so it also
  // covers the harness supervisor.
  install_panic_hook();

  // R35.A: install a default gullet pushback limit so runaway
  // macro expansion (witnessed on plain-TeX `\displaylines{ …
  // \picture(800,250) … }` chains, 7 sandbox papers from
  // 1999-2006) trips a clean `Fatal:timeout:PushbackLimit`
  // instead of the small-alloc OOM cascade that the
  // post-cap watchdog would catch only after the worker has
  // burned ~1.6 GB in `Vec<Token>` accumulation. Override or
  // remove via LaTeXML.sty's `pushbacklimit=N` keyval per
  // `latexml_package::package::latexml_sty.rs`.
  //
  // 5 million tokens × ~24 B = ~120 MB at trip — well below
  // the 6 GB ulimit headroom and large enough that real
  // documents never reach it (witness wp5: median pushback
  // peaks well under 100k tokens).
  if std::env::var_os("LATEXML_NO_DEFAULT_PUSHBACK_LIMIT").is_none() {
    latexml_core::gullet::set_pushback_limit(Some(5_000_000));
  }

  // Run all work on a worker thread with a 256 MB stack so deeply
  // nested math trees (XMApp(op, [XMApp(...)]) chains in grammar-
  // ambiguous papers — sandbox 0711.4787 et al, #17) don't overflow
  // the OS-default 8 MB main-thread stack during finalize/post-
  // processing. Validated: 0711.4787 converts cleanly under
  // `ulimit -s unlimited` (959 maths, Status:conversion:1).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(|| real_main().map_err(|e| e.to_string()))
    .expect("spawn worker thread")
    .join()
    .expect("worker thread panicked")
    .map_err(|s| s.into())
}

/// Run the process-supervising harness (`--harness`): keep a fleet of
/// single-conversion (`--pool-size 1`) worker child processes alive, each
/// capped at `--child-mem-limit-mb` of address space via `setrlimit(RLIMIT_AS)`
/// (applied by `pericortex::harness` in the forked child before `exec`), and
/// respawn any that die — until SIGTERM/SIGINT tears the fleet down cleanly.
///
/// Children are this same binary re-invoked with the dispatcher/profile flags
/// forwarded, `--pool-size 1`, and **without** `--harness` (so no fork bomb).
/// Each child's polled `--max-rss-mb` soft guard is set ~256 MiB under the hard
/// `RLIMIT_AS` cap so the watchdog emits an attributable `Status:conversion:3`
/// just before the hard `ENOMEM` kill.
fn run_harness(cli: &Cli) -> Result<(), Box<dyn Error>> {
  use pericortex::harness::{HarnessConfig, default_worker_count, supervise, total_ram_bytes};

  const MIB: u64 = 1024 * 1024;

  let mem_limit_bytes =
    (cli.child_mem_limit_mb > 0).then(|| cli.child_mem_limit_mb.saturating_mul(MIB));

  // Worker count: CPU-derived by default — a deliberate over-commit. Most jobs
  // use a small fraction of the per-child cap (observed: median a few hundred
  // MB), so sizing the fleet to the worst-case cap would idle most of the box.
  // The fleet memory-pressure governor below is what makes the over-commit safe
  // against the rare cluster of concurrently-heavy jobs. An explicit `--workers`
  // overrides.
  let workers = cli.workers.unwrap_or_else(default_worker_count);

  // Memory-pressure governor floor. Omitted → auto = max(one per-child cap, 10%
  // of RAM): enough free headroom for one heavy job to keep growing before the
  // governor steps in. `--mem-pressure-floor-mb 0` disables it (rely on the
  // per-child cap + any outer cgroup); an explicit value sets the MiB floor.
  let mem_pressure_floor_bytes = match cli.mem_pressure_floor_mb {
    Some(0) => None,
    Some(mb) => Some(mb.saturating_mul(MIB)),
    None => total_ram_bytes().map(|total| mem_limit_bytes.unwrap_or(0).max(total / 10)),
  };

  // Soft RSS guard ~256 MiB under the hard cap (or the user's value if no hard
  // cap), so the in-process watchdog fires an attributable Fatal before ENOMEM.
  let child_max_rss_mb = if cli.child_mem_limit_mb > 0 {
    cli.child_mem_limit_mb.saturating_sub(256)
  } else {
    cli.max_rss_mb
  };

  // Memory-based recycle threshold: 25% of the per-child ceiling (validated default). After a
  // clean conversion, a worker whose RSS has passed this exits and is re-forked fresh, reclaiming
  // the engine's never-reset interner/arena by process replacement (the common case of light
  // papers never trips it). A fresh worker keeps the full ceiling for any single (even aberrant)
  // paper, then retires once it has consumed a quarter of it. `--allocation-limit-mb` overrides;
  // 0 disables.
  //
  // Why 25% (=2048 MiB at the default 8 GiB ceiling) and not 50%: a full-corpus 10k sweep at the
  // CPU-optimal 72 workers (see docs/CORTEX_WORKER_HARNESS.md) peaks ~125 GB aggregate RSS at 25%
  // vs ~188 GB at 50% on a 247 GiB box — and the 50% peak drops MemAvailable below the
  // mem-pressure governor's floor, triggering ~25 worker sheds whose re-leased tasks can exhaust
  // their retry budget and surface as `cortex:never_completed_with_retries` Fatals. 25% recycles
  // ~2× more often (cheap: fork + dump-load) but stays clear of the floor → 0 sheds, and is
  // *faster* in the fleet because it avoids the shed/pressure stall. (Measured 2026-06-18.)
  let child_alloc_limit_mb = if cli.allocation_limit_mb > 0 {
    cli.allocation_limit_mb
  } else if cli.child_mem_limit_mb > 0 {
    cli.child_mem_limit_mb / 4
  } else {
    cli.max_rss_mb / 4
  };

  let exe = std::env::current_exe()?;
  let governor = match mem_pressure_floor_bytes {
    Some(b) => format!("shed below {} MiB MemAvailable", b / MIB),
    None => "off".to_string(),
  };
  eprintln!(
    "Starting CorTeX harness: {workers} single-conversion worker process(es), \
     service '{}', RLIMIT_AS {} MiB/child (soft RSS guard {} MiB, recycle @ {} MiB RSS), \
     mem-governor: {governor}",
    cli.service, cli.child_mem_limit_mb, child_max_rss_mb, child_alloc_limit_mb
  );

  // Owned clones for the build closure: `supervise` calls it once per spawn
  // (including every respawn), so it must outlive `cli`'s borrow and be `Fn`.
  let source_address = cli.source_address.clone();
  let sink_address = cli.sink_address.clone();
  let service = cli.service.clone();
  let profile = cli.profile.clone();
  let timeout = cli.timeout;
  let message_size = cli.message_size;
  let limit = cli.limit;
  let preload = cli.preload.clone();
  let search_paths = cli.search_paths.clone();
  let (no_pmml, no_mathtex, verbose, quiet) = (cli.no_pmml, cli.no_mathtex, cli.verbose, cli.quiet);

  let config = HarnessConfig {
    workers,
    mem_limit_bytes,
    mem_pressure_floor_bytes,
    ..Default::default()
  };

  supervise(&config, move |_index| {
    let mut cmd = process::Command::new(&exe);
    // jemalloc tuning for children (read at allocator init, so it must be set by
    // us before exec — too late from inside the child). Enable the background
    // purge thread and a short dirty-page decay so a worker returns freed memory
    // to the OS promptly between papers, keeping per-process RSS tracking the
    // current paper. Harmless for a mimalloc-built binary (ignores these). Both
    // the jemalloc-prefixed and unprefixed names are set for build-config safety.
    cmd
      .env(
        "_RJEM_MALLOC_CONF",
        "background_thread:true,dirty_decay_ms:500,muzzy_decay_ms:0",
      )
      .env(
        "MALLOC_CONF",
        "background_thread:true,dirty_decay_ms:500,muzzy_decay_ms:0",
      );
    // Align the engine's in-process RSS fuse (`stomach::check_timeout`, default
    // 4.5 GB) with this fleet's actual per-child ceiling. That default is a
    // one-conversion-per-process leftover: in a long-lived worker the working set
    // creeps across papers and trips the 4.5 GB fuse on otherwise-fine papers,
    // producing a false `Fatal:Timeout:MemoryBudget RSS … > cap` cascade (observed
    // 2026-06-17). Raise it to the polled `--max-rss-mb` watchdog value so the
    // cooperative fuse fires only for a genuine single-paper runaway (just before
    // the watchdog), and the fleet governor handles aggregate pressure.
    if child_max_rss_mb > 0 {
      cmd.env(
        "LATEXML_RSS_CAP_BYTES",
        child_max_rss_mb.saturating_mul(1024 * 1024).to_string(),
      );
    }
    cmd
      .arg("--pool-size")
      .arg("1")
      .arg("--source-address")
      .arg(&source_address)
      .arg("--sink-address")
      .arg(&sink_address)
      .arg("--service")
      .arg(&service)
      .arg("--profile")
      .arg(&profile)
      .arg("--timeout")
      .arg(timeout.to_string())
      .arg("--max-rss-mb")
      .arg(child_max_rss_mb.to_string())
      .arg("--allocation-limit-mb")
      .arg(child_alloc_limit_mb.to_string())
      .arg("--message-size")
      .arg(message_size.to_string());
    // Forwarding `--limit` recycles each child after N tasks (the harness
    // respawns it), bounding any slow per-process memory creep.
    if let Some(l) = limit {
      cmd.arg("--limit").arg(l.to_string());
    }
    for p in &preload {
      cmd.arg("--preload").arg(p);
    }
    for p in &search_paths {
      cmd.arg("--path").arg(p);
    }
    if no_pmml {
      cmd.arg("--no-pmml");
    }
    if no_mathtex {
      cmd.arg("--no-mathtex");
    }
    if verbose {
      cmd.arg("--verbose");
    }
    if quiet {
      cmd.arg("--quiet");
    }
    cmd
  })
}

fn real_main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::parse();

  // Spawn kpathsea pre-init in a background thread (overlaps the
  // ~30-40 ms `kpathsea_init_db` cost with arg parse + dump load).
  // See `latexml_core::util::pathname::prewarm_kpathsea`. Skipped in harness
  // mode — the supervisor never converts, so it would only waste a thread; each
  // spawned child prewarms its own.
  let _kpse_warmup_handle = if std::env::var("LATEXML_NO_KPATHSEA_PREWARM").is_err() && !cli.harness
  {
    Some(std::thread::spawn(
      latexml_core::util::pathname::prewarm_kpathsea,
    ))
  } else {
    None
  };

  let verbosity = if cli.quiet {
    -1
  } else if cli.verbose {
    1
  } else {
    0
  };
  let log_level = match verbosity {
    v if v < 0 => log::LevelFilter::Warn,
    0 => log::LevelFilter::Info,
    _ => log::LevelFilter::Debug,
  };
  latexml_core::util::logger::init(log_level).ok();

  // Harness mode: become a process supervisor for a fleet of single-conversion
  // child workers, each `RLIMIT_AS`-capped. The supervisor itself does no
  // conversion, so we branch out before building any engine state.
  if cli.harness {
    return run_harness(&cli);
  }

  let profile = match cli.profile.as_str() {
    "ar5iv" => ConversionProfile::ar5iv(
      &cli.preload,
      cli.timeout,
      cli.max_rss_mb,
      cli.no_pmml,
      cli.no_mathtex,
    ),
    "generic" => ConversionProfile::generic(
      &cli.preload,
      cli.timeout,
      cli.max_rss_mb,
      cli.no_pmml,
      cli.no_mathtex,
    ),
    other => {
      eprintln!("Unknown profile '{}', using ar5iv", other);
      ConversionProfile::ar5iv(
        &cli.preload,
        cli.timeout,
        cli.max_rss_mb,
        cli.no_pmml,
        cli.no_mathtex,
      )
    },
  };

  let hostname = hostname::get()
    .unwrap_or_else(|_| OsString::from("localhost"))
    .into_string()
    .unwrap_or_else(|_| "localhost".to_string());

  let mut worker = LatexmlWorker {
    service: cli.service.clone(),
    source_address: cli.source_address.clone(),
    sink_address: cli.sink_address.clone(),
    identity: format!("{}:{}:01", hostname, cli.service),
    msg_size: cli.message_size,
    threads: cli.pool_size,
    profile,
    verbosity,
    search_paths: cli.search_paths.clone(),
    alloc_limit_mb: cli.allocation_limit_mb,
  };

  if cli.standalone {
    // Standalone mode: single conversion
    let input = cli.input.unwrap_or_else(|| {
      eprintln!("Error: --input required in standalone mode");
      process::exit(1);
    });

    // Register a watchdog-firing callback so a timed-out conversion
    // still leaves a structured failure artifact at `--output`
    // (Status:conversion:3 + a fatal:timeout log line) instead of
    // a missing file plus "Aborted (core dumped)" from the shell.
    // Per-conversion overhead in the happy path is zero — the
    // callback is only invoked from the watchdog thread immediately
    // before `exit(124)`. Witnesses: 2602.11915, 2604.11500,
    // 2604.13944, hep-ph9205242, q-alg9604005/9605003/9605028 — the
    // 7 "Aborted" rows in the 2026-05-13 588-paper sweep.
    if let Some(ref out) = cli.output {
      let out_clone = out.clone();
      let input_clone = input.clone();
      let timeout_secs = worker.profile.timeout;
      latexml_core::watchdog::set_pre_exit_hook(Box::new(move || {
        let _ = write_timeout_placeholder_zip(&out_clone, &input_clone, timeout_secs);
      }));
    }

    eprintln!("Converting {} ...", input);
    let result_path = worker.convert_archive(Path::new(&input))?;

    // Read result and write to output (overwrites the placeholder).
    let mut result_data = Vec::new();
    File::open(&result_path)?.read_to_end(&mut result_data)?;

    if let Some(output) = cli.output {
      fs::write(&output, &result_data)?;
      eprintln!("Output written to {}", output);
    } else {
      std::io::stdout().write_all(&result_data)?;
    }
    // Clean up the temp file created by convert_archive
    // (`std::env::temp_dir().join("cortex_output_<pid>.zip")`).
    // The dispatcher path (Worker::convert) removes it after consuming;
    // the standalone path forgot, leaking one ~10-100 KB zip per run.
    // Under a 947K-run canvas these accumulated to ~685 GB in /tmp.
    let _ = fs::remove_file(&result_path);
    // Propagate FATAL conversion status into the process exit code so
    // pathological failures (memory-budget guard, wall-clock timeout
    // surfaced via Error::log_fatal, etc.) classify as failures
    // instead of zero-byte-HTML "OK" runs. Only fatal (status_code 3)
    // exits non-zero — status_code 2 ("errors but recoverable") is the
    // normal canvas-accepted case for papers with minor TeX issues
    // that still produce useful HTML. Mirrors
    // `latexml_core::common::error::get_status_code`:
    //   0 success, 1 warnings, 2 errors, 3 fatal.
    let final_status = latexml_core::common::error::get_status_code();
    if final_status >= 3 {
      process::exit(final_status as i32);
    }
  } else {
    // Worker mode: connect to CorTeX dispatcher
    eprintln!(
      "Starting CorTeX worker '{}' (pool_size={}, profile={})",
      cli.service, cli.pool_size, cli.profile
    );
    eprintln!("  source: {}", cli.source_address);
    eprintln!("  sink:   {}", cli.sink_address);

    worker.start(cli.limit)?;
  }

  Ok(())
}
